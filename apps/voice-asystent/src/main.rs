#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

use va_config::{Config, Paths};
use va_core::logging::{self, LogOptions};
use va_model::{LARGE_V3_TURBO, ModelState, ModelStore};
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
    let _model = check_model(&paths);
    Ok(())
}

/// Stan modelu przy starcie; brak modelu uruchomi pobieranie w tle (zadanie 5.6).
fn check_model(paths: &Paths) -> Option<ModelState> {
    match ModelStore::new(&paths.models_dir, LARGE_V3_TURBO).check() {
        Ok(state) => {
            tracing::info!(?state, "model: stan przy starcie");
            Some(state)
        }
        Err(error) => {
            tracing::error!(%error, "model: nie udało się sprawdzić");
            None
        }
    }
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
