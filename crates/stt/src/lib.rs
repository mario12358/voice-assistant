//! Rozpoznawanie mowy (Whisper na GPU Metal).

mod gpu;

pub use gpu::{GpuProbe, GpuReady, METAL_BUILT, MetalProbe, require_metal};

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
}

pub type Result<T> = std::result::Result<T, Error>;
