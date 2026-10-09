//! Kontroler od polecenia do schowka: nagranie z fixture WAV, atrapy STT i schowka.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use va_audio::{Recorder, SilenceParams};
use va_clipboard::testing::MemoryClipboard;
use va_clipboard::{ClipboardSink, Delivery};
use va_core::controller::{Command, ControllerEvent, ControllerHandle, ControllerParts, spawn};
use va_core::state::State;
use va_stt::testing::ScriptedStt;

const SILENCE: SilenceParams = SilenceParams {
    threshold_rms: 0.01,
    padding_ms: 200,
};
const WAIT: Duration = Duration::from_secs(5);

fn fixture(name: &str) -> Vec<f32> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name);
    let mut reader = hound::WavReader::open(path).unwrap();
    reader
        .samples::<i16>()
        .map(|sample| f32::from(sample.unwrap()) / 32_768.0)
        .collect()
}

/// Nagrywarka odtwarzająca zadane nagranie; liczy starty i zapamiętuje urządzenie.
#[derive(Clone, Default)]
struct FakeRecorder {
    recording: Arc<Mutex<Vec<f32>>>,
    starts: Arc<Mutex<Vec<String>>>,
    fail_start: bool,
    active: bool,
}

impl FakeRecorder {
    fn playing(samples: Vec<f32>) -> Self {
        Self {
            recording: Arc::new(Mutex::new(samples)),
            ..Self::default()
        }
    }

    fn starts(&self) -> Vec<String> {
        self.starts.lock().unwrap().clone()
    }
}

impl Recorder for FakeRecorder {
    fn start(&mut self, device_name: &str) -> va_audio::Result<()> {
        if self.fail_start {
            return Err(va_audio::Error::NoInputDevice);
        }
        if self.active {
            return Err(va_audio::Error::AlreadyRecording);
        }
        self.active = true;
        self.starts.lock().unwrap().push(device_name.to_owned());
        Ok(())
    }

    fn stop(&mut self) -> va_audio::Result<Vec<f32>> {
        if !self.active {
            return Err(va_audio::Error::NotRecording);
        }
        self.active = false;
        Ok(self.recording.lock().unwrap().clone())
    }
}

struct Harness {
    handle: ControllerHandle,
    events: Receiver<ControllerEvent>,
    stt: ScriptedStt,
    clipboard: MemoryClipboard,
    recorder: FakeRecorder,
}

impl Harness {
    fn new(recorder: FakeRecorder, stt: ScriptedStt, clipboard: MemoryClipboard) -> Self {
        let (sender, events) = mpsc::channel();
        let handle = spawn(
            ControllerParts {
                recorder: Box::new(recorder.clone()),
                stt: Box::new(stt.clone()),
                sink: Box::new(ClipboardSink::new(clipboard.clone())),
                silence: SILENCE,
                select_device: Box::new(|| Ok("PXC 550".to_owned())),
            },
            Box::new(move |event| {
                let _ = sender.send(event);
            }),
        );
        let harness = Self {
            handle,
            events,
            stt,
            clipboard,
            recorder,
        };
        assert_eq!(
            harness.next_event(),
            ControllerEvent::StateChanged(State::Idle)
        );
        harness
    }

    fn next_event(&self) -> ControllerEvent {
        self.events
            .recv_timeout(WAIT)
            .expect("zdarzenie kontrolera")
    }

    /// Zdarzenia aż do powrotu do Idle (włącznie).
    fn events_until_idle(&self) -> Vec<ControllerEvent> {
        let mut seen = Vec::new();
        loop {
            let event = self.next_event();
            let idle = event == ControllerEvent::StateChanged(State::Idle);
            seen.push(event);
            if idle {
                return seen;
            }
        }
    }

    fn record_and_stop(&self) -> Vec<ControllerEvent> {
        self.handle.send(Command::Start);
        self.handle.send(Command::Stop);
        self.events_until_idle()
    }
}

#[test]
fn stop_puts_transcript_in_clipboard_and_reports_states_to_ui() {
    let speech = fixture("speech_with_silence.wav");
    let harness = Harness::new(
        FakeRecorder::playing(speech.clone()),
        ScriptedStt::answering([Ok("Dzień dobry, test przycinania ciszy.".into())]),
        MemoryClipboard::containing("stare"),
    );

    let events = harness.record_and_stop();

    assert_eq!(
        events,
        vec![
            ControllerEvent::StateChanged(State::Recording),
            ControllerEvent::StateChanged(State::Transcribing),
            ControllerEvent::Delivered(Delivery::Written),
            ControllerEvent::StateChanged(State::Idle),
        ]
    );
    assert_eq!(
        harness.clipboard.contents().as_deref(),
        Some("Dzień dobry, test przycinania ciszy.")
    );
    assert_eq!(harness.recorder.starts(), vec!["PXC 550".to_owned()]);
    let received = harness.stt.received();
    assert_eq!(received.len(), 1);
    assert!(
        received[0].len() < speech.len(),
        "cisza na brzegach nie została przycięta"
    );
}

#[test]
// specky: crit 01M4EKHCXW1S92G9Y6AS115MW9
fn silence_only_recording_skips_stt_and_leaves_clipboard() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("silence_only.wav")),
        ScriptedStt::answering([]),
        MemoryClipboard::containing("poprzednia transkrypcja"),
    );

    let events = harness.record_and_stop();

    assert!(events.contains(&ControllerEvent::Delivered(Delivery::SkippedEmpty)));
    assert!(harness.stt.received().is_empty(), "cisza trafiła do STT");
    assert_eq!(
        harness.clipboard.contents().as_deref(),
        Some("poprzednia transkrypcja")
    );
}

#[test]
fn stt_error_goes_through_error_to_idle_and_next_recording_works() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("speech_with_silence.wav")),
        ScriptedStt::answering([Err(()), Ok("druga próba".into())]),
        MemoryClipboard::containing("bez zmian"),
    );

    let failed = harness.record_and_stop();

    assert!(failed.contains(&ControllerEvent::StateChanged(State::Error)));
    assert!(
        failed
            .iter()
            .any(|event| matches!(event, ControllerEvent::Failed(message) if message.contains("transkrypcja")))
    );
    assert_eq!(harness.clipboard.contents().as_deref(), Some("bez zmian"));
    harness.record_and_stop();
    assert_eq!(harness.clipboard.contents().as_deref(), Some("druga próba"));
}

#[test]
fn second_start_while_recording_does_not_start_second_recording() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("speech_with_silence.wav")),
        ScriptedStt::answering([Ok("raz".into())]),
        MemoryClipboard::default(),
    );

    harness.handle.send(Command::Start);
    harness.handle.send(Command::Start);
    harness.handle.send(Command::Stop);
    harness.events_until_idle();

    assert_eq!(harness.recorder.starts().len(), 1);
    assert_eq!(harness.stt.received().len(), 1);
}

#[test]
fn missing_microphone_reports_error_and_returns_to_idle() {
    let recorder = FakeRecorder {
        fail_start: true,
        ..FakeRecorder::default()
    };
    let harness = Harness::new(
        recorder,
        ScriptedStt::answering([]),
        MemoryClipboard::default(),
    );

    harness.handle.send(Command::Start);
    let events = harness.events_until_idle();

    assert_eq!(
        events,
        vec![
            ControllerEvent::StateChanged(State::Recording),
            ControllerEvent::StateChanged(State::Error),
            ControllerEvent::Failed(
                "nie znaleziono żadnego mikrofonu — podłącz mikrofon i spróbuj ponownie".into()
            ),
            ControllerEvent::StateChanged(State::Idle),
        ]
    );
}

#[test]
fn stop_without_recording_publishes_nothing() {
    let harness = Harness::new(
        FakeRecorder::default(),
        ScriptedStt::answering([]),
        MemoryClipboard::default(),
    );

    harness.handle.send(Command::Stop);

    assert!(
        harness
            .events
            .recv_timeout(Duration::from_millis(200))
            .is_err()
    );
}

#[test]
fn digital_silence_reports_no_signal_instead_of_transcribing() {
    let harness = Harness::new(
        FakeRecorder::playing(vec![0.0; 16_000]),
        ScriptedStt::answering([]),
        MemoryClipboard::containing("bez zmian"),
    );

    let events = harness.record_and_stop();

    assert!(events.contains(&ControllerEvent::NoSignal), "{events:?}");
    assert!(events.contains(&ControllerEvent::StateChanged(State::Error)));
    assert!(harness.stt.received().is_empty());
    assert_eq!(harness.clipboard.contents().as_deref(), Some("bez zmian"));
}
