//! Kontroler: Start → nagrywanie; Stop → przycięcie ciszy → STT (wątek roboczy) → schowek.
//!
//! Kontroler żyje we własnym wątku i przyjmuje polecenia kanałem, więc wątek główny (pętla
//! zdarzeń UI) nigdy nie czeka na mikrofon ani na model. Transkrypcja idzie w osobnym wątku
//! roboczym — dzięki temu Start w trakcie transkrypcji trafia do automatu od razu i zostaje
//! zignorowany, zamiast czekać w kolejce i odpalić nagranie po fakcie.

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::JoinHandle;

use va_audio::{Recorder, SilenceParams, TARGET_SAMPLE_RATE, trim_silence};
use va_clipboard::{Delivery, TextSink};
use va_stt::{SpeechToText, Transcript};

use crate::logging;
use crate::state::{Action, Input, State, StateMachine};

/// Polecenie użytkownika — takie samo ze skrótu i z kliknięcia ikony.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Command {
    Start,
    Stop,
}

/// Zdarzenia dla UI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ControllerEvent {
    StateChanged(State),
    /// Komunikat błędu dla użytkownika (po nim automat wraca do Idle).
    Failed(String),
    /// Transkrypcja dostarczona albo pominięta (pusta).
    Delivered(Delivery),
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
}

enum Message {
    Command(Command),
    Transcribed(Result<Option<Transcript>, String>),
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
}

impl Controller {
    fn run(mut self, inbox: &Receiver<Message>) {
        (self.publish)(ControllerEvent::StateChanged(self.machine.state()));
        while let Ok(message) = inbox.recv() {
            match message {
                Message::Command(Command::Start) => self.apply(Input::Start),
                Message::Command(Command::Stop) => self.apply(Input::Stop),
                Message::Transcribed(result) => self.on_transcribed(result),
                Message::Shutdown => break,
            }
        }
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
        let started = (self.select_device)()
            .and_then(|device| self.recorder.start(&device).map_err(|e| e.to_string()));
        if let Err(message) = started {
            self.fail(message);
        }
    }

    fn stop_and_transcribe(&mut self) {
        match self.recorder.stop() {
            Ok(samples) => {
                let speech = trim_silence(&samples, TARGET_SAMPLE_RATE, &self.silence).to_vec();
                self.worker.transcribe(speech);
            }
            Err(error) => self.fail(error.to_string()),
        }
    }

    fn on_transcribed(&mut self, result: Result<Option<Transcript>, String>) {
        match result {
            Ok(transcript) => {
                let delivery = match transcript {
                    Some(transcript) => {
                        logging::transcription_finished(&transcript.text, transcript.inference);
                        Delivery::Written
                    }
                    None => Delivery::SkippedEmpty,
                };
                (self.publish)(ControllerEvent::Delivered(delivery));
                self.apply(Input::TranscriptionFinished);
            }
            Err(message) => self.fail(message),
        }
    }

    /// Błąd jednego nagrania: komunikat dla UI, Error → Idle, aplikacja działa dalej.
    fn fail(&mut self, message: String) {
        tracing::error!(error = %message, "nagranie nieudane");
        self.apply(Input::Failed);
        (self.publish)(ControllerEvent::Failed(message));
        self.apply(Input::ErrorAcknowledged);
    }
}

/// Wątek roboczy z modelem i schowkiem: nagranie → tekst → schowek.
struct TranscriptionWorker {
    jobs: Sender<Vec<f32>>,
}

impl TranscriptionWorker {
    fn spawn(
        mut stt: Box<dyn SpeechToText>,
        mut sink: Box<dyn TextSink>,
        results: Sender<Message>,
    ) -> Self {
        let (jobs, queue) = mpsc::channel::<Vec<f32>>();
        std::thread::Builder::new()
            .name("va-transcription".into())
            .spawn(move || {
                for speech in queue {
                    let result = transcribe_and_deliver(stt.as_mut(), sink.as_mut(), &speech);
                    if results.send(Message::Transcribed(result)).is_err() {
                        break;
                    }
                }
            })
            .expect("wątek transkrypcji");
        Self { jobs }
    }

    fn transcribe(&self, speech: Vec<f32>) {
        let _ = self.jobs.send(speech);
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
