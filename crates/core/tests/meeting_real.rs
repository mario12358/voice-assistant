//! Cały tor spotkania na prawdziwych komponentach: mikrofon (cpal), dźwięk systemowy (process
//! tap) i Whisper na GPU. Wymaga sesji graficznej, mikrofonu, zgody na dźwięk systemowy
//! i pobranego modelu.

use std::path::PathBuf;
use std::process::Command as Process;
use std::sync::mpsc;
use std::time::Duration;

use va_audio::{
    AudioHost, CpalHost, CpalRecorder, CpalStreamRecorder, SilenceParams, choose_device,
};
use va_clipboard::ClipboardSink;
use va_clipboard::testing::MemoryClipboard;
use va_config::{Language, Paths, SilenceConfig};
use va_core::controller::{Command, ControllerEvent, ControllerParts, spawn};
use va_core::history::{FileHistoryStore, History};
use va_core::meeting_capture::MeetingParts;
use va_core::state::State;
use va_stt::{METAL_BUILT, MetalProbe, WhisperStt, require_metal};
use va_system_audio::CoreAudioSource;

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

#[test]
#[ignore = "wymaga mikrofonu, wyjścia audio, zgody na dźwięk systemowy, modelu i GPU Metal"]
// specky: crit 01M6JR8RQVPD5PYDQF56SZ9YVT
fn sound_played_during_meeting_is_transcribed_as_others() {
    let _ = tracing_subscriber::fmt().with_env_filter("info").try_init();
    let gpu = require_metal(&MetalProbe, METAL_BUILT).expect("GPU Metal");
    let paths = Paths::for_current_user().unwrap();
    let stt = WhisperStt::load(
        &paths.models_dir.join("ggml-large-v3-turbo.bin"),
        &gpu,
        Language::En,
    )
    .expect("model");
    let defaults = SilenceConfig::default();
    let dir = tempfile::tempdir().unwrap();
    let clipboard = MemoryClipboard::default();
    let (sender, events) = mpsc::channel();
    let handle = spawn(
        ControllerParts {
            recorder: Box::new(CpalRecorder::new(600)),
            stt: Box::new(stt),
            sink: Box::new(ClipboardSink::new(clipboard.clone())),
            silence: SilenceParams {
                threshold_rms: defaults.threshold_rms,
                padding_ms: defaults.padding_ms,
            },
            select_device: Box::new(|| {
                let devices = CpalHost::new().input_devices().map_err(|e| e.to_string())?;
                choose_device(&devices, None)
                    .map(|choice| choice.name)
                    .map_err(|e| e.to_string())
            }),
            history: History::open(Box::new(FileHistoryStore::new(
                dir.path().join("history.json"),
            ))),
            meeting: Some(MeetingParts {
                microphone: Box::new(CpalStreamRecorder::default()),
                system: Box::new(CoreAudioSource::default()),
                spool_dir: dir.path().to_owned(),
                max_seconds: 120,
                speaker_labels: true,
            }),
        },
        Box::new(move |event| {
            let _ = sender.send(event);
        }),
    );
    let wait_for = |wanted: ControllerEvent, timeout: Duration| loop {
        let event = events.recv_timeout(timeout).expect("zdarzenie kontrolera");
        assert!(
            !matches!(
                event,
                ControllerEvent::Failed(_) | ControllerEvent::MeetingDenied
            ),
            "{event:?}"
        );
        if event == wanted {
            break;
        }
    };

    handle.send(Command::StartMeeting);
    wait_for(ControllerEvent::MeetingRecording, Duration::from_secs(150));
    let played = Process::new("afplay")
        .arg(fixture_path("speech_en.wav"))
        .status()
        .expect("afplay");
    std::thread::sleep(Duration::from_millis(500));
    handle.send(Command::Stop);
    wait_for(
        ControllerEvent::StateChanged(State::Idle),
        Duration::from_secs(120),
    );

    assert!(played.success());
    let text = clipboard.contents().expect("tekst w schowku");
    let others = text
        .lines()
        .find(|line| line.starts_with("Rozmówcy:"))
        .unwrap_or_else(|| panic!("brak wypowiedzi rozmówców: {text}"));
    assert!(others.to_lowercase().contains("park"), "{text}");
    assert_eq!(
        std::fs::read_dir(dir.path())
            .unwrap()
            .filter(|e| {
                e.as_ref()
                    .is_ok_and(|e| e.file_name().to_string_lossy().ends_with(".pcm"))
            })
            .count(),
        0
    );
}
