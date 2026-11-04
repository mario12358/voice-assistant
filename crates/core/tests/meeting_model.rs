//! Transkrypcja spotkania prawdziwym modelem: dwie ścieżki z przesunięciem w czasie (VA-CONF-4).

use std::path::PathBuf;

use va_audio::SilenceParams;
use va_config::{Language, Paths, SilenceConfig};
use va_core::meeting::{merge_tracks, transcribe_track};
use va_stt::{METAL_BUILT, MetalProbe, WhisperStt, require_metal};

const RATE: usize = 16_000;

fn fixture(name: &str) -> Vec<f32> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name);
    let mut reader = hound::WavReader::open(path).unwrap();
    assert_eq!(reader.spec().sample_rate, 16_000);
    reader
        .samples::<i16>()
        .map(|sample| f32::from(sample.unwrap()) / 32_768.0)
        .collect()
}

fn placed_at(samples: &[f32], start_s: usize, total_s: usize) -> Vec<f32> {
    let mut track = vec![0.0; RATE * start_s];
    track.extend_from_slice(samples);
    track.resize(RATE * total_s, 0.0);
    track
}

fn silence() -> SilenceParams {
    let config = SilenceConfig::default();
    SilenceParams {
        threshold_rms: config.threshold_rms,
        padding_ms: config.padding_ms,
    }
}

fn merged(mic_start_s: usize, system_start_s: usize) -> String {
    let gpu = require_metal(&MetalProbe, METAL_BUILT).expect("GPU Metal");
    let model = Paths::for_current_user()
        .unwrap()
        .models_dir
        .join("ggml-large-v3-turbo.bin");
    let mut stt = WhisperStt::load(&model, &gpu, Language::Auto).expect("model");
    let mic = placed_at(&fixture("speech_pl.wav"), mic_start_s, 30);
    let system = placed_at(&fixture("speech_en.wav"), system_start_s, 30);

    let mic_segments = transcribe_track(&mut stt, &mic, &silence()).unwrap();
    let system_segments = transcribe_track(&mut stt, &system, &silence()).unwrap();
    merge_tracks(&mic_segments, &system_segments, true)
}

#[test]
#[ignore = "wymaga pobranego modelu (va-dev model-download) i GPU Metal"]
// specky: crit 01M6JR8S2RJ391FHM6BF9C8PS1
fn real_model_labels_polish_user_then_english_others_by_time() {
    let mic_first = merged(0, 10);
    let others_first = merged(10, 0);

    let lines: Vec<&str> = mic_first.lines().collect();
    assert_eq!(lines.len(), 2, "{mic_first}");
    assert!(
        lines[0].starts_with("Ja:") && lines[0].to_lowercase().contains("spacer"),
        "{mic_first}"
    );
    assert!(
        lines[1].starts_with("Rozmówcy:") && lines[1].to_lowercase().contains("park"),
        "{mic_first}"
    );
    assert!(others_first.starts_with("Rozmówcy:"), "{others_first}");
    assert!(
        others_first
            .lines()
            .nth(1)
            .is_some_and(|line| line.starts_with("Ja:")),
        "{others_first}"
    );
}
