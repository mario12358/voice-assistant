//! Podmenu „Ustawienia” (VA-SET-1): język transkrypcji i limit długości nagrania, zapis do
//! `config.toml` i polecenie dla kontrolera — działa od następnego nagrania, bez restartu.

use std::path::Path;

use va_config::{Config, Language};
use va_core::controller::Command;

const LANGUAGE_ID_PREFIX: &str = "setting-language:";
const LIMIT_ID_PREFIX: &str = "setting-limit:";
/// Limity do wyboru w menu (sekundy): 5, 10, 20, 30 minut.
pub const LIMIT_CHOICES: [u32; 4] = [300, 600, 1200, 1800];
const LANGUAGES: [(Language, &str, &str); 3] = [
    (Language::Auto, "auto", "Automatycznie"),
    (Language::Pl, "pl", "Polski"),
    (Language::En, "en", "Angielski"),
];

/// Pozycja podmenu z zaznaczeniem; `enabled = false` dla „inne: …” (tylko informacja).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingItem {
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsMenu {
    pub languages: Vec<SettingItem>,
    pub limits: Vec<SettingItem>,
}

/// Wybór z menu: co zapisać w konfiguracji i jakie polecenie wysłać do kontrolera.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingChange {
    Language(Language),
    RecordingLimit(u32),
}

impl SettingChange {
    pub fn command(self) -> Command {
        match self {
            Self::Language(language) => Command::SetLanguage(language),
            Self::RecordingLimit(seconds) => Command::SetRecordingLimit(seconds),
        }
    }

    fn apply_to(self, config: &mut Config) {
        match self {
            Self::Language(language) => config.language = language,
            Self::RecordingLimit(seconds) => config.max_recording_secs = seconds,
        }
    }
}

pub fn settings_menu(config: &Config) -> SettingsMenu {
    let languages = LANGUAGES
        .iter()
        .map(|(language, code, label)| SettingItem {
            id: format!("{LANGUAGE_ID_PREFIX}{code}"),
            label: (*label).to_owned(),
            checked: config.language == *language,
            enabled: true,
        })
        .collect();
    let mut limits: Vec<SettingItem> = LIMIT_CHOICES
        .iter()
        .map(|seconds| SettingItem {
            id: format!("{LIMIT_ID_PREFIX}{seconds}"),
            label: format!("{} min", seconds / 60),
            checked: config.max_recording_secs == *seconds,
            enabled: true,
        })
        .collect();
    if !LIMIT_CHOICES.contains(&config.max_recording_secs) {
        limits.push(SettingItem {
            id: format!("{LIMIT_ID_PREFIX}other"),
            label: format!("inne: {} s", config.max_recording_secs),
            checked: true,
            enabled: false,
        });
    }
    SettingsMenu { languages, limits }
}

/// Zmiana z identyfikatora klikniętej pozycji; inne pozycje (i „inne: …”) dają `None`.
pub fn change_from_menu_id(id: &str) -> Option<SettingChange> {
    if let Some(code) = id.strip_prefix(LANGUAGE_ID_PREFIX) {
        return LANGUAGES
            .iter()
            .find(|(_, known, _)| *known == code)
            .map(|(language, _, _)| SettingChange::Language(*language));
    }
    id.strip_prefix(LIMIT_ID_PREFIX)
        .and_then(|seconds| seconds.parse().ok())
        .filter(|seconds| LIMIT_CHOICES.contains(seconds))
        .map(SettingChange::RecordingLimit)
}

/// Zapisuje zmianę w `config.toml`; pozostałe pola zostają bez zmian.
pub fn save_change(config_path: &Path, change: SettingChange) -> va_config::Result<Config> {
    let (mut config, _) = Config::load_or_default(config_path);
    change.apply_to(&mut config);
    config.save(config_path)?;
    tracing::info!(?change, "ustawienie zapisane");
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checked(items: &[SettingItem]) -> Vec<&str> {
        items
            .iter()
            .filter(|item| item.checked)
            .map(|item| item.label.as_str())
            .collect()
    }

    #[test]
    // specky: crit 01M4KD15A2F54BF8ASK6CVDBEW
    fn menu_marks_language_and_limit_from_configuration() {
        let config = Config {
            language: Language::Pl,
            max_recording_secs: 1200,
            ..Config::default()
        };

        let menu = settings_menu(&config);

        let languages: Vec<_> = menu.languages.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(languages, ["Automatycznie", "Polski", "Angielski"]);
        assert_eq!(checked(&menu.languages), ["Polski"]);
        let limits: Vec<_> = menu.limits.iter().map(|i| i.label.as_str()).collect();
        assert_eq!(limits, ["5 min", "10 min", "20 min", "30 min"]);
        assert_eq!(checked(&menu.limits), ["20 min"]);
    }

    #[test]
    // specky: crit 01M4KD15A22K5XNV2DMD3KEBHK
    fn limit_outside_the_list_is_shown_as_other_and_kept() {
        let config = Config {
            max_recording_secs: 90,
            ..Config::default()
        };

        let menu = settings_menu(&config);

        let other = menu.limits.last().unwrap();
        assert_eq!(other.label, "inne: 90 s");
        assert!(other.checked && !other.enabled);
        assert_eq!(checked(&menu.limits), ["inne: 90 s"]);
        assert_eq!(
            change_from_menu_id(&other.id),
            None,
            "„inne” nic nie zapisuje"
        );
    }

    #[test]
    fn menu_ids_map_to_changes_and_commands() {
        let menu = settings_menu(&Config::default());

        assert_eq!(
            change_from_menu_id(&menu.languages[2].id),
            Some(SettingChange::Language(Language::En))
        );
        assert_eq!(
            change_from_menu_id(&menu.limits[3].id).map(SettingChange::command),
            Some(Command::SetRecordingLimit(1800))
        );
        assert_eq!(change_from_menu_id("setting-limit:7"), None);
        assert_eq!(change_from_menu_id("quit"), None);
    }

    #[test]
    // specky: crit 01M4KD15A2V2GZ00PWDJ2RSFZV
    // specky: crit 01M4KD15A2ZRGP0QWYMMJVVSY6
    fn saving_a_change_updates_only_that_field_in_config_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        Config {
            microphone: Some("PXC 550".into()),
            ..Config::default()
        }
        .save(&path)
        .unwrap();

        save_change(&path, SettingChange::Language(Language::Pl)).unwrap();
        save_change(&path, SettingChange::RecordingLimit(300)).unwrap();

        let saved = Config::load(&path).unwrap();
        assert_eq!(saved.language, Language::Pl);
        assert_eq!(saved.max_recording_secs, 300);
        assert_eq!(saved.microphone.as_deref(), Some("PXC 550"));
    }
}
