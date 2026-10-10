//! Transkrypcja prawdziwym modelem large-v3-turbo na GPU Metal (wymaga pobranego modelu).

use std::io::Write;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use va_config::{Language, Paths};
use va_stt::{METAL_BUILT, MetalProbe, SpeechToText, WhisperStt, require_metal};

#[derive(Clone, Default)]
struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

impl Write for CapturedLogs {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.lock().unwrap().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl CapturedLogs {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().unwrap()).into_owned()
    }
}

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

fn model_path() -> PathBuf {
    Paths::for_current_user()
        .unwrap()
        .models_dir
        .join("ggml-large-v3-turbo.bin")
}

#[test]
#[ignore = "wymaga pobranego modelu (va-dev model-download) i GPU Metal"]
// specky: crit 01M4EKHCMW0ACD0XED5QNZ4WGX
fn model_loads_once_on_metal_and_transcribes_polish_and_english() {
    let logs = CapturedLogs::default();
    let writer = logs.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_ansi(false)
        .with_writer(move || writer.clone())
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);
    let gpu = require_metal(&MetalProbe, METAL_BUILT).expect("GPU Metal");

    let mut stt = WhisperStt::load(&model_path(), &gpu, Language::Auto).expect("model");
    let polish = stt.transcribe(&fixture("speech_pl.wav")).unwrap();
    let english = stt.transcribe(&fixture("speech_en.wav")).unwrap();

    let polish_text = polish.text.to_lowercase();
    assert!(
        polish_text.contains("pogoda") && polish_text.contains("spacer"),
        "PL: {}",
        polish.text
    );
    let english_text = english.text.to_lowercase();
    assert!(
        english_text.contains("weather") && english_text.contains("park"),
        "EN: {}",
        english.text
    );
    let logs = logs.text();
    let whisper_lines = |pattern: &str| {
        logs.lines()
            .filter(|line| line.contains("whisper_rs::") && line.contains(pattern))
            .count()
    };
    assert!(
        whisper_lines("whisper_backend_init_gpu: device 0: Metal") > 0,
        "whisper.cpp nie zgłosił backendu GPU Metal"
    );
    assert!(
        whisper_lines("Metal total size") > 0,
        "wagi modelu nie trafiły do pamięci Metal"
    );
    assert_eq!(logs.matches("model Whisper załadowany").count(), 1);
}

fn loaded(language: Language) -> WhisperStt {
    let gpu = require_metal(&MetalProbe, METAL_BUILT).expect("GPU Metal");
    WhisperStt::load(&model_path(), &gpu, language).expect("model")
}

#[test]
#[ignore = "wymaga pobranego modelu (va-dev model-download) i GPU Metal"]
// specky: crit 01M4KD15J53HQD32A2N55TWF2R
fn noise_without_speech_gives_empty_transcript() {
    let mut stt = loaded(Language::Auto);

    let transcript = stt.transcribe(&fixture("noise_only.wav")).unwrap();

    assert_eq!(
        transcript.text, "",
        "szum bez mowy dał tekst — pusty schowek (VA-REC-3) wymaga pustego wyniku"
    );
}

#[test]
#[ignore = "wymaga pobranego modelu (va-dev model-download) i GPU Metal"]
// specky: crit 01M4KD15J5D855VXXS167FCEPD
fn polish_setting_keeps_mixed_sentence_in_polish_after_set_language() {
    let mut stt = loaded(Language::Auto);
    stt.set_language(Language::Pl);

    let text = stt
        .transcribe(&fixture("speech_pl_mixed.wav"))
        .unwrap()
        .text
        .to_lowercase();

    assert!(
        text.contains("muszę") && text.contains("sprawdzić"),
        "zdanie nie jest po polsku: {text}"
    );
}

#[test]
#[ignore = "wymaga pobranego modelu (va-dev model-download) i GPU Metal"]
// specky: crit 01M4KD15J5A523398FWNFTBTT1
fn decoding_settings_keep_existing_fixtures_recognised() {
    let mut stt = loaded(Language::Auto);

    let polish = stt.transcribe(&fixture("speech_pl.wav")).unwrap().text;
    let english = stt.transcribe(&fixture("speech_en.wav")).unwrap().text;

    assert!(polish.to_lowercase().contains("spacer"), "PL: {polish}");
    assert!(english.to_lowercase().contains("park"), "EN: {english}");
}
