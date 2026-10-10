//! Wspólne pomocniki testów z prawdziwym modelem (fixtures WAV, ścieżka modelu).

use std::path::PathBuf;

use va_config::Paths;

pub fn fixture(name: &str) -> Vec<f32> {
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

pub fn model_path() -> PathBuf {
    Paths::for_current_user()
        .unwrap()
        .models_dir
        .join("ggml-large-v3-turbo.bin")
}
