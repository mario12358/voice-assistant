//! Konfiguracja aplikacji (TOML) i ścieżki macOS.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

const APP_DIR: &str = "VoiceAsystent";

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("nie udało się odczytać pliku {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("błąd w pliku {path}, linia {line}: {message}")]
    Parse {
        path: PathBuf,
        line: usize,
        message: String,
    },
    #[error("nie udało się zapisać pliku {path}: {source}")]
    Write { path: PathBuf, source: io::Error },
    #[error("nie udało się zapisać konfiguracji: {0}")]
    Serialize(#[from] toml::ser::Error),
    #[error("nie można ustalić katalogu domowego użytkownika")]
    NoHomeDir,
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    #[default]
    Auto,
    Pl,
    En,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SilenceConfig {
    /// Próg RMS (0.0–1.0), poniżej którego ramka jest traktowana jako cisza.
    pub threshold_rms: f32,
    /// Margines zostawiany wokół mowy przy przycinaniu ciszy.
    pub padding_ms: u32,
}

impl Default for SilenceConfig {
    fn default() -> Self {
        Self {
            threshold_rms: 0.01,
            padding_ms: 200,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Nazwa wybranego mikrofonu; brak = mikrofon domyślny systemu.
    pub microphone: Option<String>,
    pub language: Language,
    /// Własna ścieżka modelu; brak = model w katalogu danych aplikacji.
    pub model_path: Option<PathBuf>,
    pub max_recording_secs: u32,
    pub silence: SilenceConfig,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            microphone: None,
            language: Language::Auto,
            model_path: None,
            max_recording_secs: 600,
            silence: SilenceConfig::default(),
        }
    }
}

impl Config {
    /// Odczytuje konfigurację; brak pliku daje wartości domyślne.
    pub fn load(path: &Path) -> Result<Self> {
        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Self::default()),
            Err(source) => {
                return Err(Error::Read {
                    path: path.to_owned(),
                    source,
                });
            }
        };
        toml::from_str(&text).map_err(|error| Error::Parse {
            path: path.to_owned(),
            line: line_of(&text, error.span().map_or(0, |span| span.start)),
            message: error.message().to_owned(),
        })
    }

    /// Jak [`Config::load`], ale błędny plik nie zatrzymuje aplikacji:
    /// zwraca wartości domyślne razem z błędem do zalogowania.
    pub fn load_or_default(path: &Path) -> (Self, Option<Error>) {
        match Self::load(path) {
            Ok(config) => (config, None),
            Err(error) => (Self::default(), Some(error)),
        }
    }

    /// Zapisuje konfigurację atomowo (plik tymczasowy + zmiana nazwy).
    pub fn save(&self, path: &Path) -> Result<()> {
        let text = toml::to_string_pretty(self)?;
        let write_error = |source| Error::Write {
            path: path.to_owned(),
            source,
        };
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(write_error)?;
        }
        let temporary = path.with_extension("toml.tmp");
        fs::write(&temporary, text).map_err(write_error)?;
        fs::rename(&temporary, path).map_err(write_error)
    }
}

fn line_of(text: &str, offset: usize) -> usize {
    text[..offset.min(text.len())].matches('\n').count() + 1
}

/// Ścieżki aplikacji w katalogu użytkownika macOS.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub config_file: PathBuf,
    pub models_dir: PathBuf,
    pub logs_dir: PathBuf,
    /// Historia wypowiedzi (VA-HIST-1) — jedyny plik z treścią transkrypcji.
    pub history_file: PathBuf,
}

impl Paths {
    pub fn under_home(home: &Path) -> Self {
        let data_dir = home.join("Library/Application Support").join(APP_DIR);
        Self {
            config_file: data_dir.join("config.toml"),
            models_dir: data_dir.join("models"),
            logs_dir: home.join("Library/Logs").join(APP_DIR),
            history_file: data_dir.join("history.json"),
        }
    }

    pub fn for_current_user() -> Result<Self> {
        std::env::home_dir()
            .map(|home| Self::under_home(&home))
            .ok_or(Error::NoHomeDir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_config(dir: &tempfile::TempDir, text: &str) -> PathBuf {
        let path = dir.path().join("config.toml");
        fs::write(&path, text).unwrap();
        path
    }

    #[test]
    fn missing_file_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();

        let config = Config::load(&dir.path().join("brak.toml")).unwrap();

        assert_eq!(config, Config::default());
        assert_eq!(config.language, Language::Auto);
        assert_eq!(config.microphone, None);
    }

    #[test]
    fn empty_file_yields_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(&dir, "");

        assert_eq!(Config::load(&path).unwrap(), Config::default());
    }

    #[test]
    fn full_file_overrides_every_field() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(
            &dir,
            r#"
microphone = "MacBook Pro Microphone"
language = "pl"
model_path = "/tmp/model.bin"
max_recording_secs = 60

[silence]
threshold_rms = 0.05
padding_ms = 100
"#,
        );

        let config = Config::load(&path).unwrap();

        assert_eq!(
            config,
            Config {
                microphone: Some("MacBook Pro Microphone".into()),
                language: Language::Pl,
                model_path: Some("/tmp/model.bin".into()),
                max_recording_secs: 60,
                silence: SilenceConfig {
                    threshold_rms: 0.05,
                    padding_ms: 100,
                },
            }
        );
    }

    #[test]
    fn partial_file_keeps_defaults_for_missing_fields() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(&dir, "language = \"en\"\n[silence]\npadding_ms = 50\n");

        let config = Config::load(&path).unwrap();

        assert_eq!(config.language, Language::En);
        assert_eq!(config.silence.padding_ms, 50);
        assert_eq!(config.silence.threshold_rms, 0.01);
        assert_eq!(config.max_recording_secs, 600);
    }

    #[test]
    // specky: crit 01M4K06AY61DWRJ5R2DFNDJD6W
    fn default_recording_limit_is_ten_minutes_and_file_overrides_it() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(&dir, "max_recording_secs = 20\n");

        assert_eq!(Config::default().max_recording_secs, 600);
        assert_eq!(Config::load(&path).unwrap().max_recording_secs, 20);
    }

    #[test]
    fn invalid_toml_reports_line_number() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(&dir, "language = \"pl\"\n\nmicrophone = \n");

        let error = Config::load(&path).unwrap_err();

        assert!(matches!(error, Error::Parse { line: 3, .. }), "{error:?}");
        assert!(error.to_string().contains("linia 3"));
    }

    #[test]
    fn unknown_language_is_a_parse_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(&dir, "language = \"de\"\n");

        assert!(matches!(
            Config::load(&path),
            Err(Error::Parse { line: 1, .. })
        ));
    }

    #[test]
    fn load_or_default_falls_back_on_invalid_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = write_config(&dir, "max_recording_secs = \"dużo\"\n");

        let (config, error) = Config::load_or_default(&path);

        assert_eq!(config, Config::default());
        assert!(matches!(error, Some(Error::Parse { .. })));
    }

    // specky: crit 01M4EKHCZM7G8GHHNJ56MSNY51
    #[test]
    fn saved_microphone_survives_reload() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/VoiceAsystent/config.toml");
        let config = Config {
            microphone: Some("USB Audio Device".into()),
            ..Config::default()
        };

        config.save(&path).unwrap();

        assert_eq!(Config::load(&path).unwrap(), config);
        assert!(!path.with_extension("toml.tmp").exists());
    }

    #[test]
    fn paths_follow_macos_layout() {
        let paths = Paths::under_home(Path::new("/Users/test"));

        assert_eq!(
            paths.config_file,
            Path::new("/Users/test/Library/Application Support/VoiceAsystent/config.toml")
        );
        assert_eq!(
            paths.models_dir,
            Path::new("/Users/test/Library/Application Support/VoiceAsystent/models")
        );
        assert_eq!(
            paths.history_file,
            Path::new("/Users/test/Library/Application Support/VoiceAsystent/history.json")
        );
        assert_eq!(
            paths.logs_dir,
            Path::new("/Users/test/Library/Logs/VoiceAsystent")
        );
    }
}
