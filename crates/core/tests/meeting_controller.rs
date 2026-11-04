//! Tryb spotkania w kontrolerze (VA-CONF-1..3) na atrapach: mikrofon strumieniowy, dźwięk
//! systemowy i STT z zaplanowanymi odpowiedziami.

use std::path::Path;
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use va_audio::{
    LimitNotifier, Recorder, SilenceParams, StreamFormat, StreamSink, StreamingRecorder,
};
use va_clipboard::ClipboardSink;
use va_clipboard::testing::MemoryClipboard;
use va_core::controller::{Command, ControllerEvent, ControllerHandle, ControllerParts, spawn};
use va_core::history::{FileHistoryStore, History};
use va_core::meeting_capture::MeetingParts;
use va_core::state::State;
use va_stt::testing::ScriptedStt;
use va_system_audio::Authorization;
use va_system_audio::testing::FakeSystemAudio;

const SILENCE: SilenceParams = SilenceParams {
    threshold_rms: 0.01,
    padding_ms: 0,
};
const WAIT: Duration = Duration::from_secs(5);
const RATE: usize = 16_000;

fn tone(seconds: f32) -> Vec<f32> {
    let count = (RATE as f32 * seconds) as usize;
    (0..count)
        .map(|index| if index % 2 == 0 { 0.3 } else { -0.3 })
        .collect()
}

fn silence(seconds: f32) -> Vec<f32> {
    vec![0.0; (RATE as f32 * seconds) as usize]
}

/// Mikrofon strumieniowy 16 kHz mono; test podaje próbki przez [`FakeMicrophone::play`].
#[derive(Clone, Default)]
struct FakeMicrophone {
    sink: Arc<Mutex<Option<StreamSink>>>,
    starts: Arc<Mutex<usize>>,
}

impl FakeMicrophone {
    fn play(&self, samples: &[f32]) {
        let deadline = Instant::now() + WAIT;
        loop {
            if let Some(sink) = self.sink.lock().unwrap().as_mut() {
                sink(samples);
                return;
            }
            assert!(
                Instant::now() < deadline,
                "mikrofon spotkania nie wystartował"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
    }
}

impl StreamingRecorder for FakeMicrophone {
    fn start(&mut self, _device: &str, sink: StreamSink) -> va_audio::Result<StreamFormat> {
        *self.starts.lock().unwrap() += 1;
        *self.sink.lock().unwrap() = Some(sink);
        Ok(StreamFormat {
            sample_rate: 16_000,
            channels: 1,
        })
    }

    fn stop(&mut self) {
        *self.sink.lock().unwrap() = None;
    }
}

/// Zwykła nagrywarka: zapamiętuje tylko, czy ją uruchomiono.
#[derive(Clone, Default)]
struct PlainRecorder {
    starts: Arc<Mutex<usize>>,
}

impl Recorder for PlainRecorder {
    fn start(&mut self, _device: &str, _on_limit: LimitNotifier) -> va_audio::Result<()> {
        *self.starts.lock().unwrap() += 1;
        Ok(())
    }

    fn stop(&mut self) -> va_audio::Result<Vec<f32>> {
        Ok(tone(1.0))
    }

    fn set_limit(&mut self, _max_seconds: u32) {}
}

struct Meeting {
    handle: ControllerHandle,
    events: Receiver<ControllerEvent>,
    clipboard: MemoryClipboard,
    microphone: FakeMicrophone,
    system: FakeSystemAudio,
    recorder: PlainRecorder,
    history_file: std::path::PathBuf,
    spool_dir: tempfile::TempDir,
    _dir: tempfile::TempDir,
}

impl Meeting {
    fn new(stt: ScriptedStt, system: FakeSystemAudio, max_seconds: u32) -> Self {
        let dir = tempfile::tempdir().unwrap();
        let spool_dir = tempfile::tempdir().unwrap();
        let history_file = dir.path().join("history.json");
        let clipboard = MemoryClipboard::default();
        let microphone = FakeMicrophone::default();
        let recorder = PlainRecorder::default();
        let (sender, events) = mpsc::channel();
        let handle = spawn(
            ControllerParts {
                recorder: Box::new(recorder.clone()),
                stt: Box::new(stt),
                sink: Box::new(ClipboardSink::new(clipboard.clone())),
                silence: SILENCE,
                select_device: Box::new(|| Ok("PXC 550".to_owned())),
                history: History::open(Box::new(FileHistoryStore::new(&history_file))),
                meeting: Some(MeetingParts {
                    microphone: Box::new(microphone.clone()),
                    system: Box::new(system.clone()),
                    spool_dir: spool_dir.path().to_owned(),
                    max_seconds,
                    speaker_labels: true,
                }),
            },
            Box::new(move |event| {
                let _ = sender.send(event);
            }),
        );
        Self {
            handle,
            events,
            clipboard,
            microphone,
            system,
            recorder,
            history_file,
            spool_dir,
            _dir: dir,
        }
    }

    /// Zdarzenia aż do spełnienia warunku (włącznie).
    fn events_until(&self, done: impl Fn(&ControllerEvent) -> bool) -> Vec<ControllerEvent> {
        let mut seen = Vec::new();
        loop {
            let event = self
                .events
                .recv_timeout(WAIT)
                .unwrap_or_else(|_| panic!("brak oczekiwanego zdarzenia, były: {seen:?}"));
            let finished = done(&event);
            seen.push(event);
            if finished {
                return seen;
            }
        }
    }

    fn start_meeting(&self) {
        self.handle.send(Command::StartMeeting);
        self.events_until(|event| *event == ControllerEvent::MeetingRecording);
    }

    fn finish(&self) -> Vec<ControllerEvent> {
        self.events_until(|event| *event == ControllerEvent::StateChanged(State::Idle))
    }
}

fn spool_files(dir: &Path) -> usize {
    std::fs::read_dir(dir).unwrap().count()
}

#[test]
// specky: crit 01M6JR8RG2ZTCKGCJY7TX77F0J
fn meeting_transcript_with_speaker_labels_reaches_clipboard_and_history() {
    let stt = ScriptedStt::answering([Ok(" Cześć.".into()), Ok(" Hello.".into())]);
    let meeting = Meeting::new(stt, FakeSystemAudio::granted(), 600);

    meeting.start_meeting();
    meeting.microphone.play(&[tone(1.0), silence(3.0)].concat());
    meeting
        .system
        .play(&[silence(2.0), tone(1.0), silence(1.0)].concat());
    meeting.handle.send(Command::Stop);
    let events = meeting.finish();

    assert!(events.contains(&ControllerEvent::StateChanged(State::Transcribing)));
    assert_eq!(
        meeting.clipboard.contents().as_deref(),
        Some("Ja: Cześć.\nRozmówcy: Hello.")
    );
    let history = std::fs::read_to_string(&meeting.history_file).unwrap();
    assert!(history.contains("Rozmówcy: Hello."), "{history}");
    assert!(
        events
            .iter()
            .any(|event| matches!(event, ControllerEvent::TranscriptReady(_)))
    );
    assert_eq!(
        spool_files(meeting.spool_dir.path()),
        0,
        "pliki tymczasowe zostały"
    );
}

#[test]
// specky: crit 01M6JR8RG2WP1XDDA0VP7BHVWG
fn start_and_second_meeting_during_meeting_are_ignored() {
    let stt = ScriptedStt::answering([Ok(" raz".into()), Ok(String::new())]);
    let meeting = Meeting::new(stt, FakeSystemAudio::granted(), 600);

    meeting.start_meeting();
    meeting.handle.send(Command::Start);
    meeting.handle.send(Command::StartMeeting);
    meeting.microphone.play(&tone(1.0));
    meeting.handle.send(Command::Stop);
    meeting.finish();

    assert_eq!(*meeting.recorder.starts.lock().unwrap(), 0);
    assert_eq!(*meeting.microphone.starts.lock().unwrap(), 1);
    assert_eq!(meeting.system.starts(), 1);
}

#[test]
// specky: crit 01M6JR8RG26JZ4ABY3BDB5VK4G
fn regular_recording_does_not_capture_system_audio() {
    let stt = ScriptedStt::answering([Ok("zwykłe".into())]);
    let meeting = Meeting::new(stt, FakeSystemAudio::granted(), 600);

    meeting.handle.send(Command::Start);
    meeting.events_until(|event| *event == ControllerEvent::StateChanged(State::Recording));
    meeting.handle.send(Command::StartMeeting);
    meeting.handle.send(Command::Stop);
    meeting.finish();

    assert_eq!(*meeting.recorder.starts.lock().unwrap(), 1);
    assert_eq!(meeting.system.starts(), 0);
    assert_eq!(meeting.clipboard.contents().as_deref(), Some("zwykłe"));
}

#[test]
// specky: crit 01M6JR8RXSA5K1J1JTN5Q1GED2
fn meeting_stops_by_itself_at_its_own_limit_with_audio_until_the_limit() {
    let stt = ScriptedStt::answering([Ok(" do limitu".into()), Ok(String::new())]);
    let observer = stt.clone();
    let meeting = Meeting::new(stt, FakeSystemAudio::granted(), 1);

    meeting.start_meeting();
    meeting.microphone.play(&tone(1.5));
    let events = meeting.finish();

    assert!(
        events.contains(&ControllerEvent::LimitReached),
        "{events:?}"
    );
    assert_eq!(
        meeting.clipboard.contents().as_deref(),
        Some("Ja: do limitu")
    );
    let received = observer.received();
    assert_eq!(received[0].len(), RATE, "dźwięk do limitu 1 s");
}

#[test]
// specky: crit 01M6JR8RQVZ2YBPQM5M57YKV2S
fn denied_system_audio_permission_keeps_idle_and_reports() {
    let stt = ScriptedStt::answering([Ok("zwykłe".into())]);
    let denied = FakeSystemAudio::with_authorization(Authorization::Denied, Authorization::Denied);
    let meeting = Meeting::new(stt, denied, 600);
    meeting.events_until(|event| matches!(event, ControllerEvent::HistoryChanged(_)));

    meeting.handle.send(Command::StartMeeting);
    let events = meeting.events_until(|event| *event == ControllerEvent::MeetingDenied);
    meeting.handle.send(Command::Start);
    let after =
        meeting.events_until(|event| *event == ControllerEvent::StateChanged(State::Recording));

    assert!(
        !events.contains(&ControllerEvent::StateChanged(State::Recording)),
        "{events:?}"
    );
    assert_eq!(meeting.system.starts(), 0);
    assert_eq!(*meeting.microphone.starts.lock().unwrap(), 0);
    assert!(after.contains(&ControllerEvent::StateChanged(State::Recording)));
    assert_eq!(
        *meeting.recorder.starts.lock().unwrap(),
        1,
        "zwykłe nagranie działa"
    );
}

#[test]
fn first_use_asks_for_permission_and_starts_after_consent() {
    let stt = ScriptedStt::answering([Ok(" tak".into()), Ok(String::new())]);
    let undetermined =
        FakeSystemAudio::with_authorization(Authorization::Undetermined, Authorization::Granted);
    let meeting = Meeting::new(stt, undetermined, 600);

    meeting.start_meeting();

    assert_eq!(meeting.system.starts(), 1);
}

#[test]
// specky: crit 01M6JR8RXSJBV4P9TSK2Y03F2Z
fn temporary_files_are_removed_after_transcription_error() {
    let stt = ScriptedStt::answering([Err(())]);
    let meeting = Meeting::new(stt, FakeSystemAudio::granted(), 600);

    meeting.start_meeting();
    assert_eq!(spool_files(meeting.spool_dir.path()), 2);
    meeting.microphone.play(&tone(1.0));
    meeting.handle.send(Command::Stop);
    let events = meeting.finish();

    assert!(
        events
            .iter()
            .any(|event| matches!(event, ControllerEvent::Failed(_))),
        "{events:?}"
    );
    assert_eq!(spool_files(meeting.spool_dir.path()), 0);
}
