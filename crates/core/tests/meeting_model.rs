//! Transkrypcja spotkania prawdziwym modelem (VA-CONF-4): kolejność wypowiedzi z dwóch ścieżek.

use std::path::PathBuf;

use va_audio::SilenceParams;
use va_config::{Language, Paths, SilenceConfig};
use va_core::meeting::{format_transcript, transcribe_turns};
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

fn model(language: Language) -> WhisperStt {
    let gpu = require_metal(&MetalProbe, METAL_BUILT).expect("GPU Metal");
    let path = Paths::for_current_user()
        .unwrap()
        .models_dir
        .join("ggml-large-v3-turbo.bin");
    WhisperStt::load(&path, &gpu, language).expect("model")
}

fn transcript(stt: &mut WhisperStt, microphone: &[f32], system: &[f32]) -> String {
    let spoken = transcribe_turns(stt, microphone, system, &silence()).unwrap();
    format_transcript(&spoken, true)
}

#[test]
#[ignore = "wymaga pobranego modelu (va-dev model-download) i GPU Metal"]
// specky: crit 01M6JR8S2RJ391FHM6BF9C8PS1
fn real_model_labels_polish_user_then_english_others_by_time() {
    let mut stt = model(Language::Auto);
    let polish = fixture("speech_pl.wav");
    let english = fixture("speech_en.wav");

    let mic_first = transcript(
        &mut stt,
        &placed_at(&polish, 0, 30),
        &placed_at(&english, 10, 30),
    );
    let others_first = transcript(
        &mut stt,
        &placed_at(&polish, 10, 30),
        &placed_at(&english, 0, 30),
    );

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
    let reversed: Vec<&str> = others_first.lines().collect();
    assert!(
        reversed.len() == 2
            && reversed[0].starts_with("Rozmówcy:")
            && reversed[1].starts_with("Ja:"),
        "{others_first}"
    );
}

#[test]
#[ignore = "wymaga pobranego modelu (va-dev model-download) i GPU Metal"]
// specky: crit 01M6JR8S2RWX49RC3BZ7YSK3HW
// specky: crit 01M6JR8S2RRK4A1VQD6ECZY8E9
fn alternating_dialogue_keeps_the_real_order_of_utterances() {
    let mut stt = model(Language::Pl);
    let expected = [
        ("Ja:", "pomóc"),
        ("Rozmówcy:", "obiad"),
        ("Ja:", "pomidorowa"),
        ("Rozmówcy:", "deser"),
        ("Ja:", "lody"),
        ("Rozmówcy:", "widzenia"),
    ];

    let text = transcript(
        &mut stt,
        &fixture("dialogue_me.wav"),
        &fixture("dialogue_others.wav"),
    );

    let lines: Vec<String> = text.lines().map(str::to_lowercase).collect();
    assert_eq!(lines.len(), expected.len(), "{text}");
    for (line, (label, keyword)) in lines.iter().zip(expected) {
        assert!(
            line.starts_with(&label.to_lowercase()) && line.contains(keyword),
            "oczekiwano {label} …{keyword}…, jest:\n{text}"
        );
    }
}
