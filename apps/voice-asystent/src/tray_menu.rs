//! Menu pod prawym kliknięciem: podmenu „Mikrofon” i „Zakończ”.

use std::path::PathBuf;

use anyhow::Context;
use tray_icon::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use va_audio::{AudioHost, CpalHost};
use va_config::Config;

use crate::download::DownloadState;
use crate::microphones::{QUIT_ID, microphone_items};

pub const RETRY_DOWNLOAD_ID: &str = "retry-download";

pub struct TrayMenu {
    pub menu: Menu,
    microphones: Submenu,
    config_path: PathBuf,
    download: Option<(MenuItem, MenuItem)>,
}

impl TrayMenu {
    pub fn new(config_path: PathBuf) -> anyhow::Result<Self> {
        let microphones = Submenu::new("Mikrofon", true);
        let quit = MenuItem::with_id(QUIT_ID, "Zakończ", true, None);
        let menu = Menu::new();
        menu.append_items(&[&microphones, &PredefinedMenuItem::separator(), &quit])
            .context("menu ikony")?;
        let tray_menu = Self {
            menu,
            microphones,
            config_path,
            download: None,
        };
        tray_menu.refresh_microphones();
        Ok(tray_menu)
    }

    /// Stała, nieklikalna pozycja z komunikatem na górze menu (np. skróty niedostępne).
    pub fn show_notice(&self, text: &str) {
        let notice = MenuItem::new(text, false, None);
        if let Err(error) = self
            .menu
            .insert_items(&[&notice, &PredefinedMenuItem::separator()], 0)
        {
            tracing::error!(%error, "komunikat w menu");
        }
    }

    /// Status pobierania modelu na górze menu i „Ponów pobieranie” po błędzie; znika po pobraniu.
    pub fn show_download(&mut self, state: &DownloadState) {
        let status = state.menu_status();
        match (&self.download, status) {
            (None, Some(text)) => {
                let status_item = MenuItem::new(&text, false, None);
                let retry = MenuItem::with_id(
                    RETRY_DOWNLOAD_ID,
                    "Ponów pobieranie",
                    state.can_retry(),
                    None,
                );
                if let Err(error) = self.menu.insert_items(&[&status_item, &retry], 0) {
                    tracing::error!(%error, "status pobierania w menu");
                }
                self.download = Some((status_item, retry));
            }
            (Some((status_item, retry)), Some(text)) => {
                status_item.set_text(text);
                retry.set_enabled(state.can_retry());
            }
            (Some((status_item, retry)), None) => {
                let _ = self.menu.remove(status_item);
                let _ = self.menu.remove(retry);
                self.download = None;
            }
            (None, None) => {}
        }
    }

    /// Lista urządzeń aktualna na chwilę otwarcia menu (mikrofon mógł zostać podłączony).
    pub fn refresh_microphones(&self) {
        while self.microphones.remove_at(0).is_some() {}
        let devices = match CpalHost::new().input_devices() {
            Ok(devices) => devices,
            Err(error) => {
                tracing::warn!(%error, "lista mikrofonów niedostępna");
                Vec::new()
            }
        };
        let (config, _) = Config::load_or_default(&self.config_path);
        let items = microphone_items(&devices, config.microphone.as_deref());
        if items.is_empty() {
            let _ = self
                .microphones
                .append(&MenuItem::new("Brak mikrofonu", false, None));
            return;
        }
        for item in items {
            let entry = CheckMenuItem::with_id(item.id, item.label, true, item.checked, None);
            if let Err(error) = self.microphones.append(&entry) {
                tracing::error!(%error, "pozycja mikrofonu w menu");
            }
        }
    }
}
