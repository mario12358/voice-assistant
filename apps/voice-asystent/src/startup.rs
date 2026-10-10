//! Sprawdzenia startowe i złożenie kontrolera z prawdziwych elementów.

use std::path::{Path, PathBuf};

use crate::messages::Problem;
use crate::model_menu::ModelInfo;
use va_audio::{AudioHost, CpalHost, CpalRecorder, SilenceParams, choose_device};
use va_clipboard::system_sink;
use va_config::{Config, ModelVariant, Paths};
use va_core::controller::ControllerParts;
use va_core::history::{FileHistoryStore, History};
use va_model::{ModelState, ModelStore, Progress, spec_for};
use va_stt::{GpuReady, METAL_BUILT, MetalProbe, WhisperStt, require_metal};

/// Skąd aplikacja bierze model (VA-MODEL-3): magazyn domyślny (pobieranie, SHA-256) albo własna
/// ścieżka z `config.toml` — plik użytkownika, bez pobierania i bez sprawdzania sumy.
pub enum ModelSource {
    Store(ModelStore, ModelVariant),
    Custom(PathBuf),
}

impl ModelSource {
    pub fn from_config(config: &Config, paths: &Paths) -> Self {
        match &config.model_path {
            Some(path) => Self::Custom(path.clone()),
            None => Self::Store(
                ModelStore::new(&paths.models_dir, spec_for(config.model_variant)),
                config.model_variant,
            ),
        }
    }

    pub fn custom_path(&self) -> Option<&Path> {
        match self {
            Self::Custom(path) => Some(path),
            Self::Store(..) => None,
        }
    }

    /// Opis modelu dla podmenu „Model” (nazwa, ścieżka, rozmiar, czy własny).
    pub fn info(&self) -> ModelInfo {
        match self {
            Self::Store(store, variant) => ModelInfo {
                variant: Some(*variant),
                name: store.spec().display_name().to_owned(),
                path: store.model_path(),
                bytes: store.spec().size,
                custom: false,
                custom_present: false,
            },
            Self::Custom(path) => {
                let bytes = std::fs::metadata(path).map_or(0, |meta| meta.len());
                ModelInfo {
                    name: path
                        .file_name()
                        .map(|name| name.to_string_lossy().into_owned())
                        .unwrap_or_else(|| path.display().to_string()),
                    path: path.clone(),
                    bytes,
                    custom: true,
                    custom_present: path.is_file(),
                    variant: None,
                }
            }
        }
    }

    fn check(&self) -> va_model::Result<ModelState> {
        match self {
            Self::Store(store, _) => store.check(),
            Self::Custom(path) if path.is_file() => Ok(ModelState::Ready(path.clone())),
            Self::Custom(_) => Ok(ModelState::Missing),
        }
    }
}

/// Stan modelu przy starcie; brak modelu w magazynie uruchomi pobieranie w tle (zadanie 5.6).
pub fn check_model(source: &ModelSource) -> Option<ModelState> {
    match source {
        ModelSource::Custom(path) => {
            tracing::info!(path = %path.display(), "model: własna ścieżka z konfiguracji (model_path)");
        }
        ModelSource::Store(store, variant) => {
            tracing::info!(?variant, path = %store.model_path().display(), "model: wariant z magazynu");
        }
    }
    match source.check() {
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
    source: &ModelSource,
    gpu: Option<&GpuReady>,
    model: Option<&ModelState>,
) -> Result<ControllerParts, Problem> {
    let gpu = gpu.ok_or(Problem::GpuMissing)?;
    let Some(ModelState::Ready(model_path)) = model else {
        tracing::warn!("brak gotowego modelu — nagrywanie niedostępne");
        return Err(match source.custom_path() {
            Some(path) => Problem::CustomModelMissing(path.display().to_string()),
            None => Problem::ModelUnavailable,
        });
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
    /// Pobieranie ma sens tylko z GPU, z magazynu domyślnego (nie przy własnej ścieżce)
    /// i gdy modelu brak (albo jest częściowy).
    pub fn needed(
        config: &Config,
        paths: &Paths,
        source: &ModelSource,
        gpu: Option<&GpuReady>,
        model: Option<&ModelState>,
    ) -> Option<Self> {
        let missing = matches!(model, Some(ModelState::Missing | ModelState::Partial(_)));
        Self::available(config, paths, source, gpu).filter(|_| missing)
    }

    /// Pobieranie możliwe w ogóle (do „Pobierz ponownie” po usunięciu modelu): GPU jest,
    /// a model pochodzi z magazynu domyślnego — nie z własnej ścieżki użytkownika.
    pub fn available(
        config: &Config,
        paths: &Paths,
        source: &ModelSource,
        gpu: Option<&GpuReady>,
    ) -> Option<Self> {
        if source.custom_path().is_some() {
            return None;
        }
        gpu.map(|gpu| Self {
            config: config.clone(),
            paths: paths.clone(),
            gpu: gpu.clone(),
        })
    }

    /// Pobiera (wznawiając) model i ładuje go — wołane w wątku roboczym.
    pub fn run(&self, progress: &mut dyn FnMut(Progress)) -> Result<ControllerParts, String> {
        let spec = spec_for(self.config.model_variant);
        let store = ModelStore::new(&self.paths.models_dir, spec);
        let model_path = store
            .download(spec.url, progress)
            .map_err(|error| error.to_string())?;
        parts_from_model(&self.config, &self.paths, &self.gpu, &model_path)
            .map_err(|_| "model pobrany, ale nie dał się załadować".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths_in(dir: &Path) -> Paths {
        Paths::under_home(dir)
    }

    #[test]
    // specky: crit 01M4K6M33C1BCHBQ7DEG4GER2W
    fn custom_model_path_is_used_as_is_without_downloading() {
        let dir = tempfile::tempdir().unwrap();
        let model = dir.path().join("moj-model.bin");
        std::fs::write(&model, b"ggml").unwrap();
        let config = Config {
            model_path: Some(model.clone()),
            ..Config::default()
        };

        let source = ModelSource::from_config(&config, &paths_in(dir.path()));

        assert_eq!(source.custom_path(), Some(model.as_path()));
        assert_eq!(check_model(&source), Some(ModelState::Ready(model)));
        assert!(
            ModelDownload::needed(&config, &paths_in(dir.path()), &source, None, None).is_none()
        );
    }

    #[test]
    // specky: crit 01M4K6M33CWKR54XAT4JVG3HWG
    fn missing_custom_model_is_reported_with_its_path_and_not_downloaded() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("nie-ma.bin");
        let config = Config {
            model_path: Some(missing.clone()),
            ..Config::default()
        };
        let paths = paths_in(dir.path());

        let source = ModelSource::from_config(&config, &paths);
        let state = check_model(&source);

        assert_eq!(state, Some(ModelState::Missing));
        assert!(ModelDownload::needed(&config, &paths, &source, None, state.as_ref()).is_none());
        let problem = match controller_parts(&config, &paths, &source, None, state.as_ref()) {
            Ok(_) => panic!("kontroler bez GPU i bez modelu"),
            Err(problem) => problem,
        };
        assert_eq!(
            problem,
            Problem::GpuMissing,
            "bez GPU wygrywa komunikat o GPU"
        );
    }

    #[test]
    // specky: crit 01M4K6M33CGTMKF9VPRYQXKE1C
    fn quantized_variant_from_config_selects_its_file_and_download() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());
        let config = Config {
            model_variant: ModelVariant::Q5_0,
            ..Config::default()
        };

        let source = ModelSource::from_config(&config, &paths);
        let info = source.info();

        assert_eq!(info.name, "large-v3-turbo-q5_0");
        assert_eq!(info.variant, Some(ModelVariant::Q5_0));
        assert_eq!(info.bytes, 574_041_195);
        assert!(
            info.path.ends_with("ggml-large-v3-turbo-q5_0.bin"),
            "{:?}",
            info.path
        );
    }

    #[test]
    fn without_model_path_the_default_store_is_used() {
        let dir = tempfile::tempdir().unwrap();
        let paths = paths_in(dir.path());

        let source = ModelSource::from_config(&Config::default(), &paths);

        assert!(source.custom_path().is_none());
        assert_eq!(check_model(&source), Some(ModelState::Missing));
        assert!(matches!(source, ModelSource::Store(_, ModelVariant::Full)));
    }
}
