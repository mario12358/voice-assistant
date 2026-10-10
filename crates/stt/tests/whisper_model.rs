//! Transkrypcja prawdziwym modelem large-v3-turbo na GPU Metal (wymaga pobranego modelu).

mod common;

use common::{fixture, model_path};
use va_config::{Language, Paths};
use va_stt::{METAL_BUILT, MetalProbe, SpeechToText, WhisperStt, require_metal};

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

#[test]
#[ignore = "wymaga pobranego wariantu q5_0 (va-dev model-download --variant q5_0) i GPU Metal"]
// specky: crit 01M4KD15M64RDY5XTBQV6MK4FK
fn quantized_variant_recognises_the_same_polish_keywords() {
    let path = Paths::for_current_user()
        .unwrap()
        .models_dir
        .join("ggml-large-v3-turbo-q5_0.bin");
    assert!(
        path.exists(),
        "brak {} — uruchom va-dev model-download --variant q5_0",
        path.display()
    );
    let gpu = require_metal(&MetalProbe, METAL_BUILT).expect("GPU Metal");
    let mut stt = WhisperStt::load(&path, &gpu, Language::Auto).expect("model q5_0");

    let text = stt
        .transcribe(&fixture("speech_pl.wav"))
        .unwrap()
        .text
        .to_lowercase();

    assert!(
        text.contains("pogoda") && text.contains("spacer"),
        "q5_0: {text}"
    );
}
