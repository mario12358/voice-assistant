//! Lista mikrofonów, nagrywanie i normalizacja audio do 16 kHz mono.

pub mod convert;
mod devices;
mod recorder;
pub mod silence;

pub use convert::TARGET_SAMPLE_RATE;
pub use devices::{AudioHost, CpalHost, DeviceChoice, InputDevice, choose_device};
pub use recorder::{CpalRecorder, Recorder};
pub use silence::{SilenceParams, compress_pauses, speech_bounds, trim_silence};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("nie znaleziono żadnego mikrofonu — podłącz mikrofon i spróbuj ponownie")]
    NoInputDevice,
    #[error("mikrofon {0:?} jest niedostępny")]
    DeviceUnavailable(String),
    #[error("nagrywanie już trwa")]
    AlreadyRecording,
    #[error("nagrywanie nie trwa")]
    NotRecording,
    #[error("błąd konwersji częstotliwości próbkowania: {0}")]
    Resampling(String),
    #[error("błąd systemu audio: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, Error>;
