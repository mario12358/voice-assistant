//! Kontroler: Start → nagrywanie; Stop → przycięcie ciszy → STT (wątek roboczy) → schowek.
//!
//! Kontroler żyje we własnym wątku i przyjmuje polecenia kanałem, więc wątek główny (pętla
//! zdarzeń UI) nigdy nie czeka na mikrofon ani na model. Transkrypcja idzie w osobnym wątku
//! roboczym — dzięki temu Start w trakcie transkrypcji trafia do automatu od razu i zostaje
//! zignorowany, zamiast czekać w kolejce i odpalić nagranie po fakcie.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use va_audio::{
    LimitNotifier, Recorder, SilenceParams, TARGET_SAMPLE_RATE, compress_pauses, trim_silence,
};
use va_clipboard::{Delivery, TextSink};
use va_stt::{SpeechToText, Transcript};

use crate::history::{History, HistoryEntry};
use crate::logging;
use crate::state::{Action, Input, State, StateMachine};

/// Polecenie użytkownika — takie samo ze skrótu i z kliknięcia ikony.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Start,
    Stop,
    /// Kopiuje wpis historii o tym id do schowka, bez nagrywania (VA-HIST-1).
    CopyHistoryEntry(u64),
    ClearHistory,
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
    let worker = TranscriptionWorker::spawn(parts.stt, parts.sink, messages.clone());
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
}

impl Controller {
    fn run(mut self, inbox: &Receiver<Message>) {
        (self.publish)(ControllerEvent::StateChanged(self.machine.state()));
        self.publish_history();
        while let Ok(message) = inbox.recv() {
            match message {
                Message::Command(Command::Start) => self.apply(Input::Start),
                Message::Command(Command::Stop) => self.apply(Input::Stop),
                Message::Command(Command::CopyHistoryEntry(id)) => self.copy_history_entry(id),
                Message::Command(Command::ClearHistory) => {
                    self.history.clear();
                    self.publish_history();
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
            Action::StopRecordingAndTranscribe => self.stop_and_transcribe(),
            Action::ReportError | Action::None => {}
        }
    }

    fn start_recording(&mut self) {
        self.recording_number += 1;
        let number = self.recording_number;
        let messages = self.messages.clone();
        let on_limit: LimitNotifier = Box::new(move || {
            let _ = messages.send(Message::LimitReached(number));
        });
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
                if let Some(transcript) = transcript
                    && self.history.push(&transcript.text, chrono::Local::now())
                {
                    self.publish_history();
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
    Deliver(String),
}

impl TranscriptionWorker {
    fn spawn(
        mut stt: Box<dyn SpeechToText>,
        mut sink: Box<dyn TextSink>,
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
                            Job::Deliver(text) => Message::Copied(
                                sink.deliver(&text).map_err(|error| error.to_string()),
                            ),
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
