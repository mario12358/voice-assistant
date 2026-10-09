//! Sprawdzenia startowe i złożenie kontrolera z prawdziwych elementów.

use crate::messages::Problem;
use va_audio::{AudioHost, CpalHost, CpalRecorder, SilenceParams, choose_device};
use va_clipboard::system_sink;
use va_config::{Config, Paths};
use va_core::controller::ControllerParts;
use va_model::{LARGE_V3_TURBO, ModelState, ModelStore};
use va_stt::{GpuReady, METAL_BUILT, MetalProbe, WhisperStt, require_metal};

/// Stan modelu przy starcie; brak modelu uruchomi pobieranie w tle (zadanie 5.6).
pub fn check_model(paths: &Paths) -> Option<ModelState> {
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
pub fn check_gpu() -> Option<GpuReady> {
    match require_metal(&MetalProbe, METAL_BUILT) {
        Ok(ready) => Some(ready),
        Err(error) => {
            tracing::error!(%error, "transkrypcja zablokowana");
            None
        }
    }
}

/// Kontroler powstaje tylko z GPU i gotowym modelem; inaczej ikona działa bez nagrywania
/// i pokazuje powód w menu (5.5), a brak modelu uruchomi pobieranie (5.6).
pub fn controller_parts(
    config: &Config,
    paths: &Paths,
    gpu: Option<&GpuReady>,
    model: Option<&ModelState>,
) -> Result<ControllerParts, Problem> {
    let gpu = gpu.ok_or(Problem::GpuMissing)?;
    let Some(ModelState::Ready(model_path)) = model else {
        tracing::warn!("brak gotowego modelu — nagrywanie niedostępne");
        return Err(Problem::ModelUnavailable);
    };
    let stt = WhisperStt::load(model_path, gpu, config.language).map_err(|error| {
        tracing::error!(%error, "nie udało się załadować modelu");
        Problem::ModelUnavailable
    })?;
    let config_file = paths.config_file.clone();
    Ok(ControllerParts {
        recorder: Box::new(CpalRecorder::new(config.max_recording_secs)),
        stt: Box::new(stt),
        sink: Box::new(system_sink()),
        silence: SilenceParams {
            threshold_rms: config.silence.threshold_rms,
            padding_ms: config.silence.padding_ms,
        },
        select_device: Box::new(move || {
            let (config, _) = Config::load_or_default(&config_file);
            let devices = CpalHost::new().input_devices().map_err(|e| e.to_string())?;
            choose_device(&devices, config.microphone.as_deref())
                .map(|choice| choice.name)
                .map_err(|e| e.to_string())
        }),
    })
}
