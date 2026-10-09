//! Podmenu „Mikrofon” (VA-REC-4): lista urządzeń z zaznaczonym używanym, wybór do konfiguracji.

use std::path::Path;

use va_audio::{InputDevice, choose_device};
use va_config::Config;

const MIC_ID_PREFIX: &str = "mic:";
pub const QUIT_ID: &str = "quit";

/// Pozycja podmenu: identyfikator menu, etykieta, czy zaznaczona.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MicItem {
    pub id: String,
    pub label: String,
    pub checked: bool,
}

/// Zaznaczony jest mikrofon, którego nagrywanie faktycznie użyje (także po przejściu na domyślny).
pub fn microphone_items(devices: &[InputDevice], configured: Option<&str>) -> Vec<MicItem> {
    let used = choose_device(devices, configured)
        .ok()
        .map(|choice| choice.name);
    devices
        .iter()
        .map(|device| MicItem {
            id: format!("{MIC_ID_PREFIX}{}", device.name),
            label: device.name.clone(),
            checked: used.as_deref() == Some(device.name.as_str()),
        })
        .collect()
}

/// Nazwa mikrofonu z identyfikatora pozycji menu.
pub fn microphone_from_menu_id(id: &str) -> Option<&str> {
    id.strip_prefix(MIC_ID_PREFIX)
}

/// Zapisuje wybór w konfiguracji; kontroler czyta ją przy każdym starcie nagrania.
pub fn select_microphone(config_path: &Path, name: &str) -> va_config::Result<()> {
    let (mut config, _) = Config::load_or_default(config_path);
    config.microphone = Some(name.to_owned());
    config.save(config_path)?;
    tracing::info!(microphone = name, "wybrano mikrofon");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(name: &str, is_default: bool) -> InputDevice {
        InputDevice {
            name: name.to_owned(),
            is_default,
        }
    }

    fn devices() -> Vec<InputDevice> {
        vec![
            device("PXC 550", false),
            device("BlackHole 2ch", true),
            device("Mikrofon (MacBook Air)", false),
        ]
    }

    #[test]
    // specky: crit 01M4EKHCZMYASX9YG6BR69Q94Y
    fn menu_lists_all_microphones_with_configured_one_checked() {
        let items = microphone_items(&devices(), Some("PXC 550"));

        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
        assert_eq!(
            labels,
            ["PXC 550", "BlackHole 2ch", "Mikrofon (MacBook Air)"]
        );
        let checked: Vec<_> = items.iter().filter(|item| item.checked).collect();
        assert_eq!(checked.len(), 1);
        assert_eq!(checked[0].label, "PXC 550");
    }

    #[test]
    fn without_configured_microphone_system_default_is_checked() {
        let items = microphone_items(&devices(), None);

        let checked: Vec<_> = items.iter().filter(|item| item.checked).collect();
        assert_eq!(checked.len(), 1);
        assert_eq!(checked[0].label, "BlackHole 2ch");
    }

    #[test]
    fn missing_configured_microphone_checks_the_default_actually_used() {
        let items = microphone_items(&devices(), Some("Odłączony USB"));

        let checked: Vec<_> = items.iter().filter(|item| item.checked).collect();
        assert_eq!(checked[0].label, "BlackHole 2ch");
    }

    #[test]
    fn menu_id_round_trips_microphone_name() {
        let items = microphone_items(&devices(), None);

        assert_eq!(
            microphone_from_menu_id(&items[2].id),
            Some("Mikrofon (MacBook Air)")
        );
        assert_eq!(microphone_from_menu_id(QUIT_ID), None);
    }

    #[test]
    // specky: crit 01M4EKHCZM7G8GHHNJ56MSNY51
    fn selection_is_saved_and_survives_restart() {
        let dir = tempfile::tempdir().unwrap();
        let config_path = dir.path().join("config.toml");
        std::fs::write(&config_path, "language = \"pl\"\n").unwrap();

        select_microphone(&config_path, "Mikrofon (MacBook Air)").unwrap();

        let reloaded = Config::load(&config_path).unwrap();
        assert_eq!(
            reloaded.microphone.as_deref(),
            Some("Mikrofon (MacBook Air)")
        );
        assert_eq!(reloaded.language, va_config::Language::Pl);
    }
}
