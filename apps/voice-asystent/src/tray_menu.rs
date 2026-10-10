//! Menu pod prawym kliknięciem: podmenu „Historia”, „Model”, „Mikrofon” i „Zakończ”.

use std::path::PathBuf;

use anyhow::Context;
use tray_icon::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem, Submenu};
use va_audio::{AudioHost, CpalHost};
use va_config::Config;
use va_core::history::HistoryEntry;

use crate::download::DownloadState;
use crate::history_menu::{CLEAR_HISTORY_ID, history_items};
use crate::microphones::{QUIT_ID, microphone_items};
use crate::model_menu::{
    CANCEL_REMOVE_ID, CONFIRM_REMOVE_ID, ModelMenu, REDOWNLOAD_MODEL_ID, REMOVE_MODEL_ID,
    RemovePrompt, SHOW_MODEL_ID,
};

use crate::settings_menu::{SettingsMenu, settings_menu};

pub const RETRY_DOWNLOAD_ID: &str = "retry-download";
pub const SHOW_LOGS_ID: &str = "show-logs";

pub struct TrayMenu {
    pub menu: Menu,
    history: Submenu,
    model: Submenu,
    microphones: Submenu,
    languages: Submenu,
    limits: Submenu,
    config_path: PathBuf,
    download: Option<(MenuItem, MenuItem)>,
}

impl TrayMenu {
    pub fn new(config_path: PathBuf) -> anyhow::Result<Self> {
        let history = Submenu::new("Historia", true);
        let model = Submenu::new("Model", true);
        let microphones = Submenu::new("Mikrofon", true);
        let languages = Submenu::new("Język", true);
        let limits = Submenu::new("Limit nagrania", true);
        let settings = Submenu::new("Ustawienia", true);
        settings
            .append_items(&[&languages, &limits])
            .context("podmenu Ustawienia")?;
        let show_logs = MenuItem::with_id(SHOW_LOGS_ID, "Pokaż logi", true, None);
        let quit = MenuItem::with_id(QUIT_ID, "Zakończ", true, None);
        let menu = Menu::new();
        menu.append_items(&[
            &history,
            &model,
            &microphones,
            &settings,
            &PredefinedMenuItem::separator(),
            &show_logs,
            &quit,
        ])
        .context("menu ikony")?;
        let tray_menu = Self {
            menu,
            history,
            model,
            microphones,
            languages,
            limits,
            config_path,
            download: None,
        };
        tray_menu.show_history(&[]);
        tray_menu.refresh_microphones();
        tray_menu.show_settings(&settings_menu(
            &Config::load_or_default(&tray_menu.config_path).0,
        ));
        Ok(tray_menu)
    }

    /// Podmenu „Ustawienia”: język i limit nagrania z zaznaczoną wartością z konfiguracji.
    pub fn show_settings(&self, settings: &SettingsMenu) {
        for (submenu, items) in [
            (&self.languages, &settings.languages),
            (&self.limits, &settings.limits),
        ] {
            while submenu.remove_at(0).is_some() {}
            for item in items {
                let entry =
                    CheckMenuItem::with_id(&item.id, &item.label, item.enabled, item.checked, None);
                if let Err(error) = submenu.append(&entry) {
                    tracing::error!(%error, "pozycja ustawień w menu");
                }
            }
        }
    }

    /// Podmenu „Model”: linie informacyjne, „Pokaż w Finderze”, „Usuń model…” (albo pytanie
    /// o potwierdzenie z „Anuluj”), „Pobierz ponownie”.
    pub fn show_model(&self, menu: &ModelMenu, prompt: RemovePrompt) {
        while self.model.remove_at(0).is_some() {}
        for line in &menu.lines {
            let _ = self.model.append(&MenuItem::new(line, false, None));
        }
        let show = MenuItem::with_id(SHOW_MODEL_ID, "Pokaż w Finderze", true, None);
        let redownload = MenuItem::with_id(
            REDOWNLOAD_MODEL_ID,
            "Pobierz ponownie",
            menu.redownload_enabled,
            None,
        );
        let result = match prompt {
            RemovePrompt::Idle => {
                let remove =
                    MenuItem::with_id(REMOVE_MODEL_ID, "Usuń model…", menu.remove_enabled, None);
                self.model.append_items(&[
                    &PredefinedMenuItem::separator(),
                    &show,
                    &remove,
                    &redownload,
                ])
            }
            RemovePrompt::Confirming => {
                let confirm = MenuItem::with_id(CONFIRM_REMOVE_ID, &menu.confirm_label, true, None);
                let cancel = MenuItem::with_id(CANCEL_REMOVE_ID, "Anuluj", true, None);
                self.model.append_items(&[
                    &PredefinedMenuItem::separator(),
                    &show,
                    &confirm,
                    &cancel,
                    &redownload,
                ])
            }
        };
        if let Err(error) = result {
            tracing::error!(%error, "podmenu Model");
        }
    }

    /// Podmenu „Historia”: wpisy od najnowszego (kliknięcie kopiuje), na dole „Wyczyść historię”.
    pub fn show_history(&self, entries: &[HistoryEntry]) {
        while self.history.remove_at(0).is_some() {}
        let items = history_items(entries);
        if items.is_empty() {
            let _ = self
                .history
                .append(&MenuItem::new("Brak wpisów", false, None));
        }
        for item in &items {
            let entry = MenuItem::with_id(&item.id, &item.label, true, None);
            if let Err(error) = self.history.append(&entry) {
                tracing::error!(%error, "pozycja historii w menu");
            }
        }
        let clear = MenuItem::with_id(
            CLEAR_HISTORY_ID,
            "Wyczyść historię",
            !items.is_empty(),
            None,
        );
        if let Err(error) = self
            .history
            .append_items(&[&PredefinedMenuItem::separator(), &clear])
        {
            tracing::error!(%error, "czyszczenie historii w menu");
        }
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
