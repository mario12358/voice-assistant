//! Lista mikrofonów, nagrywanie i normalizacja audio do 16 kHz mono.

mod devices;

pub use devices::{AudioHost, CpalHost, DeviceChoice, InputDevice, choose_device};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("nie znaleziono żadnego mikrofonu — podłącz mikrofon i spróbuj ponownie")]
    NoInputDevice,
    #[error("błąd systemu audio: {0}")]
    Backend(String),
}

pub type Result<T> = std::result::Result<T, Error>;
