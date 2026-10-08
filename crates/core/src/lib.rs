//! Automat stanów nagrywania i kontroler łączący audio, STT i schowek.

#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");
