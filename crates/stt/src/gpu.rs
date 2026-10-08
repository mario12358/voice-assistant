//! Warunek uruchomienia modelu: GPU Metal. Pracy na CPU nie ma — ani jako fallbacku.

use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice};

use crate::{Error, Result};

/// Czy binarka została zbudowana z backendem Metal (feature `metal`).
pub const METAL_BUILT: bool = cfg!(feature = "metal");

/// Źródło informacji o GPU — wymienne w testach.
pub trait GpuProbe {
    /// Nazwa domyślnego urządzenia Metal albo `None`, gdy go nie ma.
    fn metal_device_name(&self) -> Option<String>;
}

/// Pyta system o domyślne urządzenie Metal.
pub struct MetalProbe;

impl GpuProbe for MetalProbe {
    fn metal_device_name(&self) -> Option<String> {
        MTLCreateSystemDefaultDevice().map(|device| device.name().to_string())
    }
}

/// Potwierdzenie, że transkrypcja może ruszyć na GPU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GpuReady {
    pub device_name: String,
}

/// Dopuszcza transkrypcję tylko przy buildzie z Metal i obecnym urządzeniu Metal.
pub fn require_metal(probe: &dyn GpuProbe, metal_built: bool) -> Result<GpuReady> {
    if !metal_built {
        return Err(Error::MetalNotBuilt);
    }
    let device_name = probe.metal_device_name().ok_or(Error::GpuUnavailable)?;
    tracing::info!(device = %device_name, "GPU Metal dostępne");
    Ok(GpuReady { device_name })
}

#[cfg(test)]
mod tests {
    use super::*;

    struct FakeProbe(Option<&'static str>);

    impl GpuProbe for FakeProbe {
        fn metal_device_name(&self) -> Option<String> {
            self.0.map(str::to_owned)
        }
    }

    #[test]
    fn metal_device_present_allows_transcription() {
        let ready = require_metal(&FakeProbe(Some("Apple M2")), true).unwrap();

        assert_eq!(ready.device_name, "Apple M2");
    }

    #[test]
    // specky: crit 01M4EKHCMW0ZW9K5GYM7D2NM9H
    fn missing_metal_device_blocks_transcription_with_gpu_message() {
        let error = require_metal(&FakeProbe(None), true).unwrap_err();

        assert!(matches!(error, Error::GpuUnavailable));
        assert!(
            error
                .to_string()
                .starts_with("Wymagane GPU (Metal) — praca na CPU nie jest wspierana")
        );
    }

    #[test]
    fn build_without_metal_blocks_transcription_even_with_gpu() {
        let error = require_metal(&FakeProbe(Some("Apple M2")), false).unwrap_err();

        assert!(matches!(error, Error::MetalNotBuilt));
        assert!(error.to_string().starts_with("Wymagane GPU (Metal)"));
    }

    #[test]
    #[ignore = "wymaga Maca z GPU Metal"]
    fn this_mac_has_a_metal_device() {
        let name = MetalProbe.metal_device_name().expect("urządzenie Metal");

        assert!(!name.is_empty());
    }
}
