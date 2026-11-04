//! Przechwytywanie dźwięku odtwarzanego przez system (głosy rozmówców w Meet/Teams) dla trybu
//! spotkania (VA-CONF-2). Core Audio process tap: bez wirtualnych urządzeń i bez zmiany wyjścia.
//!
//! To jedyny crate workspace z `unsafe` (decyzja 2026-11-04): wywołania C/Objective-C są
//! zamknięte w modułach `tap` i `tcc` za bezpiecznym API tego pliku.

#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

mod tap;
mod tcc;
#[cfg(any(test, feature = "testing"))]
pub mod testing;

use objc2_foundation::{NSOperatingSystemVersion, NSProcessInfo};

pub use tap::ProcessTap;

/// Nazwa prywatnego urządzenia tapu: widzi je tylko ta aplikacja (lista mikrofonów je pomija).
pub const MEETING_DEVICE_NAME: &str = "VoiceAsystent — spotkanie";

/// Najstarszy macOS z Core Audio process tap.
pub const MIN_MACOS: (isize, isize) = (14, 2);

/// Odbiorca próbek z wątku Core Audio: próbki f32 przeplatane (L R L R …).
pub type SampleSink = Box<dyn FnMut(&[f32]) + Send>;

/// Format strumienia dźwięku systemowego.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

/// Stan zgody „Nagrywanie dźwięku systemowego” w Prywatności macOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Authorization {
    Granted,
    Denied,
    /// Użytkownik jeszcze nie odpowiedział — pierwsze użycie pokaże pytanie systemu.
    Undetermined,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("nagrywanie dźwięku systemowego wymaga macOS 14.2 lub nowszego")]
    Unsupported,
    #[error("Core Audio: {call} zwróciło błąd {status}")]
    CoreAudio { call: &'static str, status: i32 },
    #[error("nieobsługiwany format dźwięku systemowego: {0}")]
    Format(String),
    #[error("przechwytywanie dźwięku systemowego już trwa")]
    AlreadyRunning,
}

pub type Result<T> = std::result::Result<T, Error>;

/// Źródło dźwięku systemowego wstrzykiwane do kontrolera (atrapa w testach: [`testing`]).
pub trait SystemAudioSource: Send {
    /// Stan zgody bez pytania użytkownika.
    fn authorization(&self) -> Authorization;
    /// Pokazuje pytanie systemu, gdy stan jest nieustalony, i czeka na odpowiedź.
    fn request_authorization(&mut self) -> Authorization;
    /// Zaczyna przechwytywanie; `sink` dostaje próbki z wątku Core Audio.
    fn start(&mut self, sink: SampleSink) -> Result<CaptureFormat>;
    /// Kończy przechwytywanie i sprząta tap oraz prywatne urządzenie.
    fn stop(&mut self);
}

/// Czy system ma Core Audio process tap (macOS ≥ [`MIN_MACOS`]).
pub fn os_supports_process_tap() -> bool {
    NSProcessInfo::processInfo().isOperatingSystemAtLeastVersion(NSOperatingSystemVersion {
        majorVersion: MIN_MACOS.0,
        minorVersion: MIN_MACOS.1,
        patchVersion: 0,
    })
}

/// Źródło produkcyjne: process tap ze zgodą sprawdzaną w TCC.
#[derive(Default)]
pub struct CoreAudioSource {
    tap: Option<ProcessTap>,
}

impl SystemAudioSource for CoreAudioSource {
    fn authorization(&self) -> Authorization {
        tcc::audio_capture_status()
    }

    fn request_authorization(&mut self) -> Authorization {
        match tcc::audio_capture_status() {
            Authorization::Undetermined => tcc::request_audio_capture(),
            known => known,
        }
    }

    fn start(&mut self, sink: SampleSink) -> Result<CaptureFormat> {
        if !os_supports_process_tap() {
            return Err(Error::Unsupported);
        }
        if self.tap.is_some() {
            return Err(Error::AlreadyRunning);
        }
        let tap = ProcessTap::start(sink)?;
        let format = tap.format();
        self.tap = Some(tap);
        Ok(format)
    }

    fn stop(&mut self) {
        self.tap = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn this_mac_supports_process_tap() {
        assert!(
            os_supports_process_tap(),
            "maszyna deweloperska jest platformą docelową (macOS ≥ 14.2)"
        );
    }
}
