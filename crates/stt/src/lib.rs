//! Rozpoznawanie mowy (Whisper na GPU Metal).

use std::time::Duration;

mod gpu;
#[cfg(any(test, feature = "testing"))]
pub mod testing;
mod whisper;

pub use gpu::{GpuProbe, GpuReady, METAL_BUILT, MetalProbe, require_metal};
pub use whisper::{DECODING, DecodingSettings, WhisperStt};

/// Wynik transkrypcji jednego nagrania.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transcript {
    pub text: String,
    pub inference: Duration,
}

/// Zamiana nagrania 16 kHz mono f32 na tekst.
pub trait SpeechToText: Send {
    fn transcribe(&mut self, samples: &[f32]) -> Result<Transcript>;
    /// Język kolejnych transkrypcji, bez ponownego ładowania modelu (VA-SET-1, VA-STT-3).
    fn set_language(&mut self, language: va_config::Language);
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(
        "Wymagane GPU (Metal) — praca na CPU nie jest wspierana (nie wykryto urządzenia Metal)"
    )]
    GpuUnavailable,
    #[error(
        "Wymagane GPU (Metal) — praca na CPU nie jest wspierana (aplikacja zbudowana bez obsługi Metal)"
    )]
    MetalNotBuilt,
    #[error("nie udało się załadować modelu Whisper: {0}")]
    ModelLoad(String),
    #[error("transkrypcja nie powiodła się: {0}")]
    Inference(String),
}

pub type Result<T> = std::result::Result<T, Error>;
