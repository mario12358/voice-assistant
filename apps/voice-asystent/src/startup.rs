//! Sprawdzenia startowe i złożenie kontrolera z prawdziwych elementów.

use std::path::Path;

use crate::messages::Problem;
use va_audio::{AudioHost, CpalHost, CpalRecorder, SilenceParams, choose_device};
use va_clipboard::system_sink;
use va_config::{Config, Paths};
use va_core::controller::ControllerParts;
use va_core::history::{FileHistoryStore, History};
use va_model::{LARGE_V3_TURBO, ModelState, ModelStore, Progress};
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
    parts_from_model(config, paths, gpu, model_path)
}

fn parts_from_model(
    config: &Config,
    paths: &Paths,
    gpu: &GpuReady,
    model_path: &Path,
) -> Result<ControllerParts, Problem> {
    let stt = WhisperStt::load(model_path, gpu, config.language).map_err(|error| {
        tracing::error!(%error, "nie udało się załadować modelu");
        Problem::ModelUnavailable
    })?;
    let config_file = paths.config_file.clone();
    Ok(ControllerParts {
        recorder: Box::new(CpalRecorder::new(config.max_recording_secs)),
        stt: Box::new(stt),
        sink: Box::new(system_sink()),
        history: History::open(Box::new(FileHistoryStore::new(&paths.history_file))),
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

/// Wszystko, czego trzeba, by w tle pobrać model i złożyć z nim kontroler (5.6).
#[derive(Clone)]
pub struct ModelDownload {
    config: Config,
    paths: Paths,
    gpu: GpuReady,
}

impl ModelDownload {
    /// Pobieranie ma sens tylko z GPU i gdy modelu brak (albo jest częściowy).
    pub fn needed(
        config: &Config,
        paths: &Paths,
        gpu: Option<&GpuReady>,
        model: Option<&ModelState>,
    ) -> Option<Self> {
        let missing = matches!(model, Some(ModelState::Missing | ModelState::Partial(_)));
        gpu.filter(|_| missing).map(|gpu| Self {
            config: config.clone(),
            paths: paths.clone(),
            gpu: gpu.clone(),
        })
    }

    /// Pobiera (wznawiając) model i ładuje go — wołane w wątku roboczym.
    pub fn run(&self, progress: &mut dyn FnMut(Progress)) -> Result<ControllerParts, String> {
        let store = ModelStore::new(&self.paths.models_dir, LARGE_V3_TURBO);
        let model_path = store
            .download(LARGE_V3_TURBO.url, progress)
            .map_err(|error| error.to_string())?;
        parts_from_model(&self.config, &self.paths, &self.gpu, &model_path)
            .map_err(|_| "model pobrany, ale nie dał się załadować".to_owned())
    }
}
