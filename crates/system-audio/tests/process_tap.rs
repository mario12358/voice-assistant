//! Prawdziwe przechwytywanie dźwięku systemowego (VA-CONF-2): wymaga sesji graficznej i zgody
//! „Nagrywanie dźwięku systemowego” dla aplikacji uruchamiającej testy (np. Terminal).

use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};

use va_system_audio::{Authorization, CoreAudioSource, SystemAudioSource};

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name)
}

/// Urządzenia audio widziane przez INNY proces (system_profiler), tak jak widzą je Meet czy
/// Ustawienia; prywatne urządzenie tapu widzi tylko proces, który je utworzył.
fn device_names_seen_by_other_apps() -> String {
    let output = Command::new("system_profiler")
        .args(["SPAudioDataType", "-detailLevel", "mini"])
        .output()
        .expect("system_profiler");
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter(|line| line.trim_end().ends_with(':'))
        .collect::<Vec<_>>()
        .join("\n")
}

fn rms(samples: &[f32]) -> f32 {
    (samples.iter().map(|sample| sample * sample).sum::<f32>() / samples.len().max(1) as f32).sqrt()
}

#[test]
#[ignore = "wymaga sesji graficznej, wyjścia audio i zgody na nagrywanie dźwięku systemowego"]
// specky: crit 01M6JR8RQVPD5PYDQF56SZ9YVT
// specky: crit 01M6JR8RQVS9DSRZ08B9G21FX4
fn sound_played_by_another_app_is_captured_without_new_audio_device() {
    let mut source = CoreAudioSource::default();
    assert_eq!(
        source.request_authorization(),
        Authorization::Granted,
        "brak zgody: Ustawienia systemowe → Prywatność i ochrona → Nagrywanie ekranu i dźwięku systemowego"
    );
    let devices_before = device_names_seen_by_other_apps();
    let captured = Arc::new(Mutex::new(Vec::new()));
    let sink_target = Arc::clone(&captured);

    let format = source
        .start(Box::new(move |samples| {
            sink_target.lock().unwrap().extend_from_slice(samples);
        }))
        .expect("start przechwytywania");
    let devices_during = device_names_seen_by_other_apps();
    let played = Command::new("afplay")
        .arg(fixture_path("speech_en.wav"))
        .status()
        .expect("afplay");
    source.stop();
    let devices_after = device_names_seen_by_other_apps();

    assert!(played.success());
    let samples = captured.lock().unwrap().clone();
    let seconds = samples.len() as f32 / (format.sample_rate as f32 * f32::from(format.channels));
    assert!(seconds > 1.0, "przechwycono tylko {seconds:.2} s");
    assert!(
        rms(&samples) > 0.001,
        "sama cisza (RMS {}) — brak zgody?",
        rms(&samples)
    );
    assert_eq!(
        devices_during, devices_before,
        "pojawiło się nowe urządzenie audio"
    );
    assert_eq!(devices_after, devices_before);
}
