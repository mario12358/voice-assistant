//! Kontroler od polecenia do schowka: nagranie z fixture WAV, atrapy STT i schowka.

use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use va_audio::{LimitNotifier, Recorder, SilenceParams};
use va_clipboard::testing::MemoryClipboard;
use va_clipboard::{ClipboardSink, Delivery};
use va_core::controller::{
    Command, ControllerEvent, ControllerHandle, ControllerParts, TranscriptPreview, spawn,
};
use va_core::history::{FileHistoryStore, History, HistoryEntry};
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
    limit_notifiers: Arc<Mutex<Vec<Option<LimitNotifier>>>>,
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

    /// Udaje zapełnienie bufora w nagraniu o podanym numerze (od 1), jak wątek audio.
    fn reach_limit_of(&self, recording_number: usize) {
        let notifier = self.limit_notifiers.lock().unwrap()[recording_number - 1]
            .take()
            .expect("sygnał limitu już zużyty");
        notifier();
    }
}

impl Recorder for FakeRecorder {
    fn start(&mut self, device_name: &str, on_limit: LimitNotifier) -> va_audio::Result<()> {
        if self.fail_start {
            return Err(va_audio::Error::NoInputDevice);
        }
        if self.active {
            return Err(va_audio::Error::AlreadyRecording);
        }
        self.active = true;
        self.starts.lock().unwrap().push(device_name.to_owned());
        self.limit_notifiers.lock().unwrap().push(Some(on_limit));
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
    history_file: PathBuf,
    /// Lista historii opublikowana zaraz po starcie kontrolera.
    initial_history: Vec<HistoryEntry>,
    _history_dir: Option<tempfile::TempDir>,
}

impl Harness {
    fn new(recorder: FakeRecorder, stt: ScriptedStt, clipboard: MemoryClipboard) -> Self {
        let dir = tempfile::tempdir().expect("katalog tymczasowy");
        let history_file = dir.path().join("history.json");
        let mut harness = Self::with_history_file(recorder, stt, clipboard, history_file);
        harness._history_dir = Some(dir);
        harness
    }

    fn with_history_file(
        recorder: FakeRecorder,
        stt: ScriptedStt,
        clipboard: MemoryClipboard,
        history_file: PathBuf,
    ) -> Self {
        let (sender, events) = mpsc::channel();
        let handle = spawn(
            ControllerParts {
                recorder: Box::new(recorder.clone()),
                stt: Box::new(stt.clone()),
                sink: Box::new(ClipboardSink::new(clipboard.clone())),
                silence: SILENCE,
                select_device: Box::new(|| Ok("PXC 550".to_owned())),
                history: History::open(Box::new(FileHistoryStore::new(&history_file))),
            },
            Box::new(move |event| {
                let _ = sender.send(event);
            }),
        );
        let mut harness = Self {
            handle,
            events,
            stt,
            clipboard,
            recorder,
            history_file,
            initial_history: Vec::new(),
            _history_dir: None,
        };
        assert_eq!(
            harness.next_event(),
            ControllerEvent::StateChanged(State::Idle)
        );
        match harness.next_event() {
            ControllerEvent::HistoryChanged(entries) => harness.initial_history = entries,
            other => panic!("po starcie oczekiwano listy historii, było {other:?}"),
        }
        harness
    }

    /// Ostatnia opublikowana lista historii spośród podanych zdarzeń.
    fn history_in(events: &[ControllerEvent]) -> Option<&[HistoryEntry]> {
        events.iter().rev().find_map(|event| match event {
            ControllerEvent::HistoryChanged(entries) => Some(entries.as_slice()),
            _ => None,
        })
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
// specky: crit 01M4K06AY6N6SB7WBZE4J4MM09
fn stop_puts_transcript_in_clipboard_and_reports_states_to_ui() {
    let speech = fixture("speech_with_silence.wav");
    let harness = Harness::new(
        FakeRecorder::playing(speech.clone()),
        ScriptedStt::answering([Ok("Dzień dobry, test przycinania ciszy.".into())]),
        MemoryClipboard::containing("stare"),
    );

    let events = harness.record_and_stop();

    let history = Harness::history_in(&events).expect("lista historii po transkrypcji");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].text, "Dzień dobry, test przycinania ciszy.");
    assert_eq!(
        events,
        vec![
            ControllerEvent::StateChanged(State::Recording),
            ControllerEvent::StateChanged(State::Transcribing),
            ControllerEvent::Delivered(Delivery::Written),
            ControllerEvent::TranscriptReady(TranscriptPreview::of(
                "Dzień dobry, test przycinania ciszy."
            )),
            ControllerEvent::HistoryChanged(history.to_vec()),
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

#[test]
// specky: crit 01M4K06AY60EVTQQ78X8RQ4RCK
fn limit_reached_stops_recording_and_transcribes_without_stop_command() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("speech_with_silence.wav")),
        ScriptedStt::answering([Ok("tekst do limitu".into())]),
        MemoryClipboard::containing("stare"),
    );

    harness.handle.send(Command::Start);
    assert_eq!(
        harness.next_event(),
        ControllerEvent::StateChanged(State::Recording)
    );
    harness.recorder.reach_limit_of(1);
    let events = harness.events_until_idle();

    let without_history: Vec<_> = events
        .iter()
        .filter(|event| {
            !matches!(
                event,
                ControllerEvent::HistoryChanged(_) | ControllerEvent::TranscriptReady(_)
            )
        })
        .cloned()
        .collect();
    assert_eq!(
        without_history,
        vec![
            ControllerEvent::LimitReached,
            ControllerEvent::StateChanged(State::Transcribing),
            ControllerEvent::Delivered(Delivery::Written),
            ControllerEvent::StateChanged(State::Idle),
        ]
    );
    assert_eq!(
        harness.clipboard.contents().as_deref(),
        Some("tekst do limitu")
    );
    assert_eq!(harness.stt.received().len(), 1);
}

#[test]
fn stale_limit_signal_from_previous_recording_is_ignored() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("speech_with_silence.wav")),
        ScriptedStt::answering([Ok("pierwsze".into()), Ok("drugie".into())]),
        MemoryClipboard::default(),
    );
    harness.record_and_stop();
    harness.handle.send(Command::Start);
    assert_eq!(
        harness.next_event(),
        ControllerEvent::StateChanged(State::Recording)
    );

    harness.recorder.reach_limit_of(1);

    assert!(
        harness
            .events
            .recv_timeout(Duration::from_millis(200))
            .is_err(),
        "spóźniony limit poprzedniego nagrania zakończył bieżące"
    );
    harness.handle.send(Command::Stop);
    let events = harness.events_until_idle();
    assert!(
        !events.contains(&ControllerEvent::LimitReached),
        "{events:?}"
    );
    assert_eq!(harness.clipboard.contents().as_deref(), Some("drugie"));
}

#[test]
// specky: crit 01M4K06AH032985T6JXJX7EE3W
fn silence_adds_nothing_to_history_but_speech_does() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("silence_only.wav")),
        ScriptedStt::answering([Ok("po ciszy".into())]),
        MemoryClipboard::default(),
    );
    assert!(harness.initial_history.is_empty());

    let silent = harness.record_and_stop();
    assert!(Harness::history_in(&silent).is_none(), "{silent:?}");

    *harness.recorder.recording.lock().unwrap() = fixture("speech_with_silence.wav");
    let spoken = harness.record_and_stop();
    let history = Harness::history_in(&spoken).expect("wpis po mowie");
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].text, "po ciszy");
}

#[test]
// specky: crit 01M4KD152NN5K9FT1BPAY3Y250
fn ready_signal_follows_only_a_written_transcript() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("silence_only.wav")),
        ScriptedStt::answering([Ok("po ciszy".into()), Err(())]),
        MemoryClipboard::default(),
    );
    let is_ready = |event: &ControllerEvent| matches!(event, ControllerEvent::TranscriptReady(_));

    let silent = harness.record_and_stop();
    assert!(!silent.iter().any(is_ready), "{silent:?}");

    *harness.recorder.recording.lock().unwrap() = fixture("speech_with_silence.wav");
    let spoken = harness.record_and_stop();
    assert!(
        spoken.contains(&ControllerEvent::TranscriptReady(TranscriptPreview::of(
            "po ciszy"
        ))),
        "{spoken:?}"
    );

    let failed = harness.record_and_stop();
    assert!(!failed.iter().any(is_ready), "{failed:?}");
}

#[test]
// specky: crit 01M4K06AH09HZHH764X48P4GM4
fn copying_a_history_entry_fills_the_clipboard_without_recording() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("speech_with_silence.wav")),
        ScriptedStt::answering([Ok("pierwsze zdanie".into()), Ok("drugie zdanie".into())]),
        MemoryClipboard::default(),
    );
    harness.record_and_stop();
    let events = harness.record_and_stop();
    let history = Harness::history_in(&events).unwrap().to_vec();
    assert_eq!(history[0].text, "drugie zdanie");
    assert_eq!(
        harness.clipboard.contents().as_deref(),
        Some("drugie zdanie")
    );

    harness
        .handle
        .send(Command::CopyHistoryEntry(history[1].id));

    assert_eq!(
        harness.next_event(),
        ControllerEvent::Delivered(Delivery::Written)
    );
    assert_eq!(
        harness.clipboard.contents().as_deref(),
        Some("pierwsze zdanie")
    );
    assert_eq!(
        harness.recorder.starts().len(),
        2,
        "kopiowanie uruchomiło nagranie"
    );
    assert_eq!(harness.stt.received().len(), 2);
}

#[test]
// specky: crit 01M4K06AH0FMZJZZX7SHA9KR5A
fn history_is_published_again_after_restart_from_the_same_file() {
    let dir = tempfile::tempdir().unwrap();
    let file = dir.path().join("history.json");
    let first = Harness::with_history_file(
        FakeRecorder::playing(fixture("speech_with_silence.wav")),
        ScriptedStt::answering([Ok("trwałe zdanie".into())]),
        MemoryClipboard::default(),
        file.clone(),
    );
    let events = first.record_and_stop();
    let saved = Harness::history_in(&events).unwrap().to_vec();
    drop(first);

    let restarted = Harness::with_history_file(
        FakeRecorder::default(),
        ScriptedStt::answering([]),
        MemoryClipboard::default(),
        file.clone(),
    );

    assert_eq!(restarted.initial_history, saved);
    assert_eq!(restarted.initial_history[0].text, "trwałe zdanie");
    assert!(restarted.history_file.exists());
}

#[test]
// specky: crit 01M4K06AH0BR7ZQYXPH6B08P26
fn clearing_history_empties_the_list_and_deletes_the_file() {
    let harness = Harness::new(
        FakeRecorder::playing(fixture("speech_with_silence.wav")),
        ScriptedStt::answering([Ok("do wyczyszczenia".into())]),
        MemoryClipboard::default(),
    );
    harness.record_and_stop();
    assert!(harness.history_file.exists());

    harness.handle.send(Command::ClearHistory);

    assert_eq!(
        harness.next_event(),
        ControllerEvent::HistoryChanged(vec![])
    );
    assert!(!harness.history_file.exists());
}

#[test]
fn long_pause_inside_recording_is_shortened_before_stt() {
    let recording = fixture("speech_pl_long_pause.wav");
    let harness = Harness::new(
        FakeRecorder::playing(recording.clone()),
        ScriptedStt::answering([Ok("dwa zdania".into())]),
        MemoryClipboard::default(),
    );

    harness.record_and_stop();

    let received = harness.stt.received();
    assert_eq!(received.len(), 1);
    let removed_seconds = (recording.len() - received[0].len()) / 16_000;
    assert!(
        removed_seconds >= 39,
        "STT dostał {} z {} próbek — 40 s pauzy miało zostać skrócone do 0,5 s",
        received[0].len(),
        recording.len()
    );
    assert!(received[0].len() > 2 * 16_000, "mowa została wycięta");
}
