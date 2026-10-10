//! Automat stanów nagrywania i kontroler łączący audio, STT i schowek.

#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

pub mod controller;
pub mod history;
pub mod logging;
pub mod state;
