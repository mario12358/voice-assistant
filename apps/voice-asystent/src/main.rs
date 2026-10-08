#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

use va_config::{Config, Paths};
use va_core::logging::{self, LogOptions};
use va_stt::{GpuReady, METAL_BUILT, MetalProbe, require_metal};

fn main() -> anyhow::Result<()> {
    let paths = Paths::for_current_user()?;
    let _log_guard = logging::init(&LogOptions {
        logs_dir: Some(paths.logs_dir.clone()),
        verbosity: 0,
    });
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "start VoiceAsystent");
    let (_config, config_error) = Config::load_or_default(&paths.config_file);
    if let Some(error) = config_error {
        tracing::warn!(%error, "używam ustawień domyślnych");
    }
    let _gpu = check_gpu();
    Ok(())
}

/// Bez GPU Metal transkrypcja jest zablokowana; aplikacja działa dalej i pokazuje komunikat.
fn check_gpu() -> Option<GpuReady> {
    match require_metal(&MetalProbe, METAL_BUILT) {
        Ok(ready) => Some(ready),
        Err(error) => {
            tracing::error!(%error, "transkrypcja zablokowana");
            None
        }
    }
}
