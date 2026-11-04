//! Kontroler: Start → nagrywanie; Stop → przycięcie ciszy → STT (wątek roboczy) → schowek.
//!
//! Kontroler żyje we własnym wątku i przyjmuje polecenia kanałem, więc wątek główny (pętla
//! zdarzeń UI) nigdy nie czeka na mikrofon ani na model. Transkrypcja idzie w osobnym wątku
//! roboczym — dzięki temu Start w trakcie transkrypcji trafia do automatu od razu i zostaje
//! zignorowany, zamiast czekać w kolejce i odpalić nagranie po fakcie.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;
use std::time::Instant;

use va_audio::{
    LimitNotifier, Recorder, SilenceParams, TARGET_SAMPLE_RATE, compress_pauses, trim_silence,
};
use va_clipboard::{Delivery, TextSink};
use va_config::Language;
use va_stt::{SpeechToText, Transcript};
use va_system_audio::Authorization;

use crate::history::{History, HistoryEntry};
use crate::logging;
use crate::meeting::{merge_tracks, transcribe_track};
use crate::meeting_capture::{MeetingAudio, MeetingParts, MeetingRecording};
use crate::state::{Action, Input, State, StateMachine};

/// Polecenie użytkownika — takie samo ze skrótu i z kliknięcia ikony.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Start,
    /// Nagranie spotkania: mikrofon + dźwięk systemowy (VA-CONF-1).
    StartMeeting,
    Stop,
    /// Kopiuje wpis historii o tym id do schowka, bez nagrywania (VA-HIST-1).
    CopyHistoryEntry(u64),
    ClearHistory,
    /// Język następnych transkrypcji (VA-SET-1), bez ładowania modelu od nowa.
    SetLanguage(Language),
    /// Limit długości następnych nagrań w sekundach (VA-SET-1).
    SetRecordingLimit(u32),
}

/// Zdarzenia dla UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerEvent {
    StateChanged(State),
    /// Komunikat błędu dla użytkownika (po nim automat wraca do Idle).
    Failed(String),
    /// Mikrofon nie dał żadnego sygnału (same zera) — zwykle brak zgody na mikrofon w macOS.
    NoSignal,
    /// Transkrypcja dostarczona albo pominięta (pusta).
    Delivered(Delivery),
    /// Nagranie osiągnęło limit długości i zostało zakończone automatycznie (VA-REC-6).
    LimitReached,
    /// Aktualna lista wpisów historii, najnowszy pierwszy (także raz po starcie).
    HistoryChanged(Vec<HistoryEntry>),
    /// Niepusta transkrypcja trafiła do schowka (VA-UX-1): podgląd do powiadomienia.
    TranscriptReady(TranscriptPreview),
    /// Trwające nagranie (po `StateChanged(Recording)`) to nagranie spotkania (VA-CONF-1).
    MeetingRecording,
    /// Brak zgody na nagrywanie dźwięku systemowego — nagranie spotkania nie wystartowało.
    MeetingDenied,
}

/// Początek transkrypcji do powiadomienia „gotowe”; `Debug` nie pokazuje treści, żeby
/// wypisanie zdarzenia w logu nie wyniosło tekstu użytkownika.
#[derive(Clone, PartialEq, Eq)]
pub struct TranscriptPreview(String);

/// Tyle znaków transkrypcji pokazuje powiadomienie; dłuższy tekst dostaje wielokropek.
pub const PREVIEW_CHARS: usize = 60;

impl TranscriptPreview {
    /// Jedna linia (białe znaki zwinięte), najwyżej [`PREVIEW_CHARS`] znaków + „…”.
    pub fn of(text: &str) -> Self {
        let single_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
        let mut preview: String = single_line.chars().take(PREVIEW_CHARS).collect();
        if single_line.chars().count() > PREVIEW_CHARS {
            preview = preview.trim_end().to_owned();
            preview.push('…');
        }
        Self(preview)
    }

    pub fn text(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for TranscriptPreview {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "TranscriptPreview({} znaków)", self.0.chars().count())
    }
}

/// Wybór mikrofonu w chwili startu nagrania (konfiguracja może się zmienić w trakcie pracy).
pub type DeviceSelector = Box<dyn FnMut() -> Result<String, String> + Send>;
/// Odbiorca zdarzeń kontrolera (w aplikacji: proxy pętli zdarzeń UI).
pub type EventPublisher = Box<dyn FnMut(ControllerEvent) + Send>;

/// Elementy wstrzykiwane do kontrolera.
pub struct ControllerParts {
    pub recorder: Box<dyn Recorder>,
    pub stt: Box<dyn SpeechToText>,
    pub sink: Box<dyn TextSink>,
    pub silence: SilenceParams,
    pub select_device: DeviceSelector,
    pub history: History,
    /// Tryb spotkania; `None` = tylko zwykłe nagrania (np. brak pliku modelu w testach).
    pub meeting: Option<MeetingParts>,
}

enum Message {
    Command(Command),
    Transcribed(Result<Option<Transcript>, String>),
    /// Wynik kopiowania wpisu historii do schowka.
    Copied(Result<Delivery, String>),
    /// Nagrywarka zapełniła bufor; numer nagrania chroni przed spóźnionym sygnałem z poprzedniego.
    LimitReached(u64),
    Shutdown,
}

/// Uchwyt do działającego kontrolera.
pub struct ControllerHandle {
    messages: Sender<Message>,
    thread: Option<JoinHandle<()>>,
}

impl ControllerHandle {
    pub fn send(&self, command: Command) {
        if self.messages.send(Message::Command(command)).is_err() {
            tracing::error!(?command, "kontroler nie działa — polecenie utracone");
        }
    }

    pub fn shutdown(mut self) {
        self.stop_thread();
    }

    fn stop_thread(&mut self) {
        let _ = self.messages.send(Message::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

impl Drop for ControllerHandle {
    fn drop(&mut self) {
        self.stop_thread();
    }
}

/// Uruchamia kontroler w osobnym wątku.
pub fn spawn(parts: ControllerParts, publish: EventPublisher) -> ControllerHandle {
    let (messages, inbox) = mpsc::channel();
    let worker = TranscriptionWorker::spawn(parts.stt, parts.sink, parts.silence, messages.clone());
    let controller = Controller {
        machine: StateMachine::default(),
        recorder: parts.recorder,
        select_device: parts.select_device,
        silence: parts.silence,
        worker,
        publish,
        messages: messages.clone(),
        recording_number: 0,
        history: parts.history,
        meeting: parts.meeting,
        meeting_recording: None,
    };
    let thread = std::thread::Builder::new()
        .name("va-controller".into())
        .spawn(move || controller.run(&inbox))
        .expect("wątek kontrolera");
    ControllerHandle {
        messages,
        thread: Some(thread),
    }
}

struct Controller {
    machine: StateMachine,
    recorder: Box<dyn Recorder>,
    select_device: DeviceSelector,
    silence: SilenceParams,
    worker: TranscriptionWorker,
    publish: EventPublisher,
    messages: Sender<Message>,
    recording_number: u64,
    history: History,
    meeting: Option<MeetingParts>,
    meeting_recording: Option<MeetingRecording>,
}

impl Controller {
    fn run(mut self, inbox: &Receiver<Message>) {
        (self.publish)(ControllerEvent::StateChanged(self.machine.state()));
        self.publish_history();
        while let Ok(message) = inbox.recv() {
            match message {
                Message::Command(Command::Start) => self.apply(Input::Start),
                Message::Command(Command::StartMeeting) => self.request_meeting(),
                Message::Command(Command::Stop) => self.apply(Input::Stop),
                Message::Command(Command::CopyHistoryEntry(id)) => self.copy_history_entry(id),
                Message::Command(Command::ClearHistory) => {
                    self.history.clear();
                    self.publish_history();
                }
                Message::Command(Command::SetLanguage(language)) => {
                    self.worker.set_language(language);
                }
                Message::Command(Command::SetRecordingLimit(seconds)) => {
                    self.recorder.set_limit(seconds);
                }
                Message::Transcribed(result) => self.on_transcribed(result),
                Message::Copied(result) => self.on_copied(result),
                Message::LimitReached(number) => self.on_limit_reached(number),
                Message::Shutdown => break,
            }
        }
    }

    fn publish_history(&mut self) {
        (self.publish)(ControllerEvent::HistoryChanged(
            self.history.entries().to_vec(),
        ));
    }

    /// Kopiowanie idzie przez ten sam schowek co transkrypcja, w wątku roboczym; automat
    /// stanów nie bierze w tym udziału — nagrywanie nie startuje.
    fn copy_history_entry(&mut self, id: u64) {
        match self.history.find(id) {
            Some(entry) => self.worker.deliver(entry.text.clone()),
            None => tracing::warn!(id, "wpis historii nie istnieje — nic do skopiowania"),
        }
    }

    fn on_copied(&mut self, result: Result<Delivery, String>) {
        match result {
            Ok(delivery) => (self.publish)(ControllerEvent::Delivered(delivery)),
            Err(message) => {
                tracing::error!(error = %message, "kopiowanie wpisu historii nieudane");
                (self.publish)(ControllerEvent::Failed(message));
            }
        }
    }

    /// Limit długości: to samo co Stop od użytkownika, plus zdarzenie dla UI (powiadomienie).
    fn on_limit_reached(&mut self, number: u64) {
        if number != self.recording_number || self.machine.state() != State::Recording {
            tracing::debug!(number, "spóźniony sygnał limitu — pominięty");
            return;
        }
        tracing::info!("osiągnięto limit długości nagrania — kończę nagranie");
        (self.publish)(ControllerEvent::LimitReached);
        self.apply(Input::Stop);
    }

    fn apply(&mut self, input: Input) {
        let transition = self.machine.handle(input);
        if transition.ignored {
            return;
        }
        (self.publish)(ControllerEvent::StateChanged(transition.to));
        match transition.action {
            Action::StartRecording => self.start_recording(),
            Action::StartMeetingRecording => self.start_meeting_recording(),
            Action::StopRecordingAndTranscribe => self.stop_and_transcribe(),
            Action::ReportError | Action::None => {}
        }
    }

    fn next_limit_notifier(&mut self) -> LimitNotifier {
        self.recording_number += 1;
        let number = self.recording_number;
        let messages = self.messages.clone();
        Box::new(move || {
            let _ = messages.send(Message::LimitReached(number));
        })
    }

    /// Zgoda na dźwięk systemowy przed zmianą stanu: przy odmowie ikona zostaje szara.
    fn request_meeting(&mut self) {
        if self.machine.state() != State::Idle {
            tracing::info!(state = ?self.machine.state(), "nagranie spotkania zignorowane w tym stanie");
            return;
        }
        let Some(meeting) = self.meeting.as_mut() else {
            tracing::warn!("tryb spotkania niedostępny w tej konfiguracji");
            return;
        };
        if meeting.authorize() == Authorization::Denied {
            tracing::warn!("brak zgody na nagrywanie dźwięku systemowego");
            (self.publish)(ControllerEvent::MeetingDenied);
            return;
        }
        self.apply(Input::StartMeeting);
    }

    fn start_meeting_recording(&mut self) {
        let on_limit = self.next_limit_notifier();
        let started = (self.select_device)().and_then(|device| {
            let meeting = self
                .meeting
                .as_mut()
                .ok_or_else(|| "tryb spotkania niedostępny".to_owned())?;
            MeetingRecording::start(meeting, &device, on_limit)
        });
        match started {
            Ok(recording) => {
                self.meeting_recording = Some(recording);
                (self.publish)(ControllerEvent::MeetingRecording);
            }
            Err(message) => self.fail(message),
        }
    }

    fn stop_meeting(&mut self, recording: MeetingRecording) {
        let Some(meeting) = self.meeting.as_mut() else {
            return;
        };
        let labels = meeting.speaker_labels;
        match recording.stop(meeting) {
            Ok(audio) => self.worker.transcribe_meeting(audio, labels),
            Err(message) => self.fail(message),
        }
    }

    fn start_recording(&mut self) {
        let on_limit = self.next_limit_notifier();
        let started = (self.select_device)().and_then(|device| {
            self.recorder
                .start(&device, on_limit)
                .map_err(|e| e.to_string())
        });
        if let Err(message) = started {
            self.fail(message);
        }
    }

    fn stop_and_transcribe(&mut self) {
        if let Some(recording) = self.meeting_recording.take() {
            self.stop_meeting(recording);
            return;
        }
        match self.recorder.stop() {
            Ok(samples) if has_no_signal(&samples) => self.no_signal(),
            Ok(samples) => {
                let speech = trim_silence(&samples, TARGET_SAMPLE_RATE, &self.silence);
                let speech = compress_pauses(speech, TARGET_SAMPLE_RATE, &self.silence);
                self.worker.transcribe(speech);
            }
            Err(error) => self.fail(error.to_string()),
        }
    }

    fn on_transcribed(&mut self, result: Result<Option<Transcript>, String>) {
        match result {
            Ok(transcript) => {
                let delivery = match &transcript {
                    Some(transcript) => {
                        logging::transcription_finished(&transcript.text, transcript.inference);
                        Delivery::Written
                    }
                    None => Delivery::SkippedEmpty,
                };
                (self.publish)(ControllerEvent::Delivered(delivery));
                if let Some(transcript) = transcript {
                    (self.publish)(ControllerEvent::TranscriptReady(TranscriptPreview::of(
                        &transcript.text,
                    )));
                    if self.history.push(&transcript.text, chrono::Local::now()) {
                        self.publish_history();
                    }
                }
                self.apply(Input::TranscriptionFinished);
            }
            Err(message) => self.fail(message),
        }
    }

    /// Bez zgody na mikrofon macOS oddaje ciszę cyfrową — to nie „sama cisza” do pominięcia.
    fn no_signal(&mut self) {
        tracing::error!("mikrofon nie dał sygnału — możliwy brak zgody na mikrofon");
        self.apply(Input::Failed);
        (self.publish)(ControllerEvent::NoSignal);
        self.apply(Input::ErrorAcknowledged);
    }

    /// Błąd jednego nagrania: komunikat dla UI, Error → Idle, aplikacja działa dalej.
    fn fail(&mut self, message: String) {
        tracing::error!(error = %message, "nagranie nieudane");
        self.apply(Input::Failed);
        (self.publish)(ControllerEvent::Failed(message));
        self.apply(Input::ErrorAcknowledged);
    }
}

/// Same dokładne zera: prawdziwy mikrofon zawsze ma choć szum, a odmowa dostępu daje ciszę cyfrową.
fn has_no_signal(samples: &[f32]) -> bool {
    !samples.is_empty() && samples.iter().all(|sample| *sample == 0.0)
}

/// Wątek roboczy z modelem i schowkiem: nagranie → tekst → schowek, albo gotowy tekst → schowek.
struct TranscriptionWorker {
    jobs: Sender<Job>,
}

enum Job {
    Transcribe(Vec<f32>),
    /// Dwie ścieżki spotkania; pliki tymczasowe znikają po zakończeniu zadania (także po błędzie).
    TranscribeMeeting {
        audio: MeetingAudio,
        labels: bool,
    },
    Deliver(String),
    /// Model żyje w wątku roboczym, więc zmiana języka idzie tą samą kolejką co nagrania —
    /// nagranie wysłane po zmianie jest już w nowym języku.
    SetLanguage(Language),
}

impl TranscriptionWorker {
    fn spawn(
        mut stt: Box<dyn SpeechToText>,
        mut sink: Box<dyn TextSink>,
        silence: SilenceParams,
        results: Sender<Message>,
    ) -> Self {
        let (jobs, queue) = mpsc::channel::<Job>();
        std::thread::Builder::new()
            .name("va-transcription".into())
            .spawn(move || {
                for job in queue {
                    let message =
                        match job {
                            Job::Transcribe(speech) => Message::Transcribed(
                                transcribe_and_deliver(stt.as_mut(), sink.as_mut(), &speech),
                            ),
                            Job::TranscribeMeeting { audio, labels } => {
                                Message::Transcribed(transcribe_meeting(
                                    stt.as_mut(),
                                    sink.as_mut(),
                                    audio,
                                    labels,
                                    &silence,
                                ))
                            }
                            Job::Deliver(text) => Message::Copied(
                                sink.deliver(&text).map_err(|error| error.to_string()),
                            ),
                            Job::SetLanguage(language) => {
                                stt.set_language(language);
                                continue;
                            }
                        };
                    if results.send(message).is_err() {
                        break;
                    }
                }
            })
            .expect("wątek transkrypcji");
        Self { jobs }
    }

    fn transcribe(&self, speech: Vec<f32>) {
        let _ = self.jobs.send(Job::Transcribe(speech));
    }

    fn transcribe_meeting(&self, audio: MeetingAudio, labels: bool) {
        let _ = self.jobs.send(Job::TranscribeMeeting { audio, labels });
    }

    fn set_language(&self, language: Language) {
        let _ = self.jobs.send(Job::SetLanguage(language));
    }

    fn deliver(&self, text: String) {
        let _ = self.jobs.send(Job::Deliver(text));
    }
}

/// Sama cisza (pusty wycinek) nie trafia do modelu ani do schowka.
fn transcribe_and_deliver(
    stt: &mut dyn SpeechToText,
    sink: &mut dyn TextSink,
    speech: &[f32],
) -> Result<Option<Transcript>, String> {
    if speech.is_empty() {
        tracing::info!("sama cisza — bez transkrypcji");
        return Ok(None);
    }
    let _span = tracing::info_span!("transcription").entered();
    let transcript = stt.transcribe(speech).map_err(|error| error.to_string())?;
    match sink
        .deliver(&transcript.text)
        .map_err(|error| error.to_string())?
    {
        Delivery::Written => Ok(Some(transcript)),
        Delivery::SkippedEmpty => Ok(None),
    }
}

/// Spotkanie: każda ścieżka osobno przez model, wynik scalony po czasie z etykietami mówców.
/// `audio` jest zwalniane na końcu, więc pliki tymczasowe znikają także po błędzie.
fn transcribe_meeting(
    stt: &mut dyn SpeechToText,
    sink: &mut dyn TextSink,
    audio: MeetingAudio,
    labels: bool,
    silence: &SilenceParams,
) -> Result<Option<Transcript>, String> {
    let _span = tracing::info_span!("meeting_transcription").entered();
    let started = Instant::now();
    let microphone = audio.microphone.read().map_err(|error| error.to_string())?;
    let mut segments = transcribe_track(stt, &microphone, silence).map_err(|e| e.to_string())?;
    drop(microphone);
    let system = audio.system.read().map_err(|error| error.to_string())?;
    let system_segments = transcribe_track(stt, &system, silence).map_err(|e| e.to_string())?;
    let text = merge_tracks(&segments, &system_segments, labels);
    if text.is_empty() {
        tracing::info!("spotkanie bez mowy — bez zmiany schowka");
        return Ok(None);
    }
    segments.extend(system_segments);
    match sink.deliver(&text).map_err(|error| error.to_string())? {
        Delivery::Written => Ok(Some(Transcript {
            text,
            inference: started.elapsed(),
            segments,
        })),
        Delivery::SkippedEmpty => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // specky: crit 01M4KD152NZM9VP8S1V8VX1E6R
    fn preview_keeps_sixty_characters_and_adds_ellipsis() {
        let short = TranscriptPreview::of("  Krótkie\nzdanie.  ");
        assert_eq!(short.text(), "Krótkie zdanie.");

        let long = TranscriptPreview::of(&"ąbc ".repeat(30));
        assert!(long.text().ends_with('…'), "{}", long.text());
        assert!(long.text().chars().count() <= PREVIEW_CHARS + 1);
    }

    #[test]
    // specky: crit 01M4KD152NR0QTPMC1RHSY5D6P
    fn preview_debug_output_hides_the_text() {
        let preview = TranscriptPreview::of("sekretna treść");

        let debug = format!("{:?}", ControllerEvent::TranscriptReady(preview));

        assert!(!debug.contains("sekretna"), "{debug}");
        assert!(debug.contains("14 znaków"), "{debug}");
    }
}
