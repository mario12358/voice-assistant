//! Pętla zdarzeń aplikacji paska menu (wątek główny, wymóg AppKit).

use std::path::{Path, PathBuf};

use anyhow::Context;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};
use va_core::controller::{
    self, Command, ControllerEvent, ControllerHandle, ControllerParts, TranscriptPreview,
};
use va_core::history::HistoryEntry;
use va_core::state::State;

use crate::click::command_for;
use crate::download::{DownloadEvent, DownloadState};
use crate::history_menu::command_from_menu_id;
use crate::hotkeys::Hotkeys;
use crate::indicator::{ICON_PIXELS, Indicator, dot_rgba, indicator_for};
use crate::login_item::{LOGIN_ITEM_ID, LoginItem};
use crate::messages::{Problem, ReadySignal, menu_notice, notify, signal_ready};
use crate::microphones::{QUIT_ID, microphone_from_menu_id, select_microphone};
use crate::model_menu::{
    ModelInfo, REDOWNLOAD_MODEL_ID, RemoveAction, RemovePrompt, SHOW_MODEL_ID, model_menu,
    state_with_percent,
};
use crate::settings_menu::{change_from_menu_id, save_change, settings_menu};
use crate::startup::ModelDownload;
use crate::tray_menu::{RETRY_DOWNLOAD_ID, SHOW_LOGS_ID, TrayMenu};
use va_model::{LARGE_V3_TURBO, ModelStore};

enum UserEvent {
    Controller(ControllerEvent),
    Tray(TrayIconEvent),
    Menu(MenuEvent),
    Hotkey(GlobalHotKeyEvent),
    Download(DownloadEvent),
    ModelLoaded(ControllerParts),
}

/// Stałe ustawienia pętli zdarzeń, wyliczone przy starcie z konfiguracji i ścieżek.
pub struct AppSettings {
    pub config_path: PathBuf,
    pub logs_dir: PathBuf,
    pub recording_limit_secs: u32,
    pub ready_signal: ReadySignal,
    pub login: LoginItem,
}

/// `download` = zadanie pobierania, jeśli w ogóle możliwe (GPU, magazyn domyślny);
/// `needs_download` = czy uruchomić je od razu (modelu brak przy starcie).
pub fn run(
    controller_parts: Result<ControllerParts, Problem>,
    download: Option<ModelDownload>,
    needs_download: bool,
    model_info: ModelInfo,
    settings: AppSettings,
) -> anyhow::Result<()> {
    let AppSettings {
        config_path,
        logs_dir,
        mut recording_limit_secs,
        ready_signal,
        login,
    } = settings;
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    event_loop.set_activation_policy(ActivationPolicy::Accessory);
    let proxy = event_loop.create_proxy();
    forward_tray_menu_and_hotkey_events(&event_loop.create_proxy());
    let hotkeys = Hotkeys::default();
    let hotkey_manager = GlobalHotKeyManager::new().context("menedżer skrótów")?;
    let hotkeys_problem = hotkeys.register(&hotkey_manager).err();
    let (controller_parts, mut unavailable) = match controller_parts {
        Ok(parts) => (Some(parts), None),
        Err(problem) => (None, Some(problem)),
    };
    let mut controller = controller_parts.map(|parts| spawn_controller(parts, proxy.clone()));
    let mut download_state = match &download {
        Some(job) if needs_download => {
            spawn_download(job.clone(), proxy.clone());
            DownloadState::Downloading { percent: 0 }
        }
        _ => DownloadState::NotNeeded,
    };
    let mut remove_prompt = RemovePrompt::default();
    let mut tray: Option<(TrayIcon, TrayMenu)> = None;
    let mut shown = State::Idle;
    let mut history: Vec<HistoryEntry> = Vec::new();
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => {
                match build_tray(State::Idle, &config_path, &login) {
                    Ok(mut built) => {
                        if let Some(problem) = &hotkeys_problem {
                            tracing::error!(%problem, "rejestracja skrótów");
                            built.1.show_notice(problem);
                        }
                        if download_state == DownloadState::NotNeeded
                            && let Some(notice) = unavailable.as_ref().and_then(menu_notice)
                        {
                            built.1.show_notice(&notice);
                        }
                        show_download(&mut built, &download_state, shown);
                        built.1.show_history(&history);
                        show_model(&built.1, &model_info, &download_state, shown, remove_prompt);
                        tray = Some(built);
                    }
                    Err(error) => {
                        tracing::error!(%error, "nie udało się utworzyć ikony w pasku menu");
                        *control_flow = ControlFlow::Exit;
                    }
                }
            }
            Event::UserEvent(UserEvent::Controller(event)) => {
                if let ControllerEvent::StateChanged(state) = &event {
                    shown = *state;
                    if let Some((icon, menu)) = &tray {
                        show(icon, indicator_for(*state));
                        show_model(menu, &model_info, &download_state, shown, remove_prompt);
                    }
                }
                if let ControllerEvent::HistoryChanged(entries) = &event {
                    history = entries.clone();
                    if let Some((_, menu)) = &tray {
                        menu.show_history(&history);
                    }
                }
                if let Some(problem) = problem_for(&event, recording_limit_secs) {
                    notify(&problem);
                }
                if let Some(preview) = ready_preview(&event, ready_signal) {
                    signal_ready(ready_signal, preview.text());
                }
                tracing::debug!(event = event_kind(&event), "zdarzenie kontrolera");
            }
            Event::UserEvent(UserEvent::Tray(event)) => {
                if let Some((_, menu)) = &tray
                    && menu_may_open(&event)
                {
                    menu.refresh_microphones();
                }
                if let Some(command) = command_for(&event, shown) {
                    send(
                        controller.as_ref(),
                        &download_state,
                        unavailable.as_ref(),
                        command,
                    );
                }
            }
            Event::UserEvent(UserEvent::Hotkey(event)) => {
                let _keep_registered = &hotkey_manager;
                if let Some(command) = hotkeys.command_for(&event) {
                    send(
                        controller.as_ref(),
                        &download_state,
                        unavailable.as_ref(),
                        command,
                    );
                }
            }
            Event::UserEvent(UserEvent::Menu(event)) => {
                let id = event.id.as_ref();
                let remove_allowed = model_menu(&model_info, &download_state, shown).remove_enabled;
                let (next_prompt, action) = remove_prompt.on_menu_click(id, remove_allowed);
                remove_prompt = next_prompt;
                if action == RemoveAction::Remove {
                    controller = None;
                    remove_model_files(&model_info);
                    download_state = download_state.clone().next(DownloadEvent::Removed);
                    unavailable = Some(Problem::ModelUnavailable);
                    if let Some(built) = &mut tray {
                        show_download(built, &download_state, shown);
                    }
                }
                if action != RemoveAction::None
                    && let Some((_, menu)) = &tray
                {
                    show_model(menu, &model_info, &download_state, shown, remove_prompt);
                }
                if id == QUIT_ID {
                    *control_flow = ControlFlow::Exit;
                } else if id == RETRY_DOWNLOAD_ID || id == REDOWNLOAD_MODEL_ID {
                    let before = download_state.clone();
                    download_state = before.clone().next(DownloadEvent::Retry);
                    if DownloadState::starts_download(&before, &download_state)
                        && let Some(job) = &download
                    {
                        spawn_download(job.clone(), proxy.clone());
                    }
                    if let Some(built) = &mut tray {
                        show_download(built, &download_state, shown);
                        show_model(&built.1, &model_info, &download_state, shown, remove_prompt);
                    }
                } else if id == SHOW_MODEL_ID {
                    reveal_in_finder(&model_info.path);
                } else if id == SHOW_LOGS_ID {
                    reveal_in_finder(&logs_dir);
                } else if id == LOGIN_ITEM_ID {
                    if let Err(error) = login.toggle() {
                        tracing::error!(%error, "uruchamianie przy logowaniu");
                    }
                    if let Some((_, menu)) = &tray {
                        menu.show_login(&login);
                    }
                } else if let Some(name) = microphone_from_menu_id(id) {
                    if let Err(error) = select_microphone(&config_path, name) {
                        tracing::error!(%error, "zapis wyboru mikrofonu");
                    }
                    if let Some((_, menu)) = &tray {
                        menu.refresh_microphones();
                    }
                } else if let Some(command) = command_from_menu_id(id) {
                    send(
                        controller.as_ref(),
                        &download_state,
                        unavailable.as_ref(),
                        command,
                    );
                } else if let Some(change) = change_from_menu_id(id) {
                    match save_change(&config_path, change) {
                        Ok(saved) => {
                            recording_limit_secs = saved.max_recording_secs;
                            if let Some(controller) = &controller {
                                controller.send(change.command());
                            }
                            if let Some((_, menu)) = &tray {
                                menu.show_settings(&settings_menu(&saved));
                            }
                        }
                        Err(error) => tracing::error!(%error, "zapis ustawienia"),
                    }
                }
            }
            Event::UserEvent(UserEvent::Download(event)) => {
                download_state = download_state.clone().next(event);
                if let Some(built) = &mut tray {
                    show_download(built, &download_state, shown);
                    show_model(&built.1, &model_info, &download_state, shown, remove_prompt);
                }
            }
            Event::UserEvent(UserEvent::ModelLoaded(parts)) => {
                let fresh = spawn_controller(parts, proxy.clone());
                // Zadanie pobierania niesie konfigurację ze startu — ustawienia zmienione
                // w menu w trakcie pobierania przekazujemy nowemu kontrolerowi.
                let (current, _) = va_config::Config::load_or_default(&config_path);
                fresh.send(Command::SetLanguage(current.language));
                fresh.send(Command::SetRecordingLimit(current.max_recording_secs));
                controller = Some(fresh);
                unavailable = None;
                download_state = download_state.clone().next(DownloadEvent::Ready);
                if let Some(built) = &mut tray {
                    show_download(built, &download_state, shown);
                    show_model(&built.1, &model_info, &download_state, shown, remove_prompt);
                }
            }
            _ => {}
        }
    })
}

/// Które zdarzenia kontrolera kończą się powiadomieniem dla użytkownika.
fn problem_for(event: &ControllerEvent, recording_limit_secs: u32) -> Option<Problem> {
    match event {
        ControllerEvent::NoSignal => Some(Problem::MicrophoneSilent),
        ControllerEvent::Failed(reason) => Some(Problem::RecordingFailed(reason.clone())),
        ControllerEvent::LimitReached => Some(Problem::RecordingLimitReached {
            limit_secs: recording_limit_secs,
        }),
        ControllerEvent::StateChanged(_)
        | ControllerEvent::Delivered(_)
        | ControllerEvent::HistoryChanged(_)
        | ControllerEvent::TranscriptReady(_) => None,
    }
}

/// Podgląd do sygnału „gotowe” (VA-UX-1), gdy zdarzenie to zapisana transkrypcja i użytkownik
/// nie wyłączył obu form sygnału.
fn ready_preview(event: &ControllerEvent, signal: ReadySignal) -> Option<&TranscriptPreview> {
    match event {
        ControllerEvent::TranscriptReady(preview) if signal.notify || signal.sound => Some(preview),
        _ => None,
    }
}

/// Rodzaj zdarzenia do logu — bez treści (transkrypcja i historia to dane użytkownika).
fn event_kind(event: &ControllerEvent) -> &'static str {
    match event {
        ControllerEvent::StateChanged(_) => "StateChanged",
        ControllerEvent::Failed(_) => "Failed",
        ControllerEvent::NoSignal => "NoSignal",
        ControllerEvent::Delivered(_) => "Delivered",
        ControllerEvent::LimitReached => "LimitReached",
        ControllerEvent::HistoryChanged(_) => "HistoryChanged",
        ControllerEvent::TranscriptReady(_) => "TranscriptReady",
    }
}

fn forward_tray_menu_and_hotkey_events(proxy: &EventLoopProxy<UserEvent>) {
    let tray_proxy = proxy.clone();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy.send_event(UserEvent::Tray(event));
    }));
    let menu_proxy = proxy.clone();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::Menu(event));
    }));
    let hotkey_proxy = proxy.clone();
    GlobalHotKeyEvent::set_event_handler(Some(move |event| {
        let _ = hotkey_proxy.send_event(UserEvent::Hotkey(event));
    }));
}

fn spawn_controller(parts: ControllerParts, proxy: EventLoopProxy<UserEvent>) -> ControllerHandle {
    controller::spawn(
        parts,
        Box::new(move |event| {
            let _ = proxy.send_event(UserEvent::Controller(event));
        }),
    )
}

/// Pobieranie i ładowanie modelu w tle; postęp zgłaszany co pełny procent.
fn spawn_download(job: ModelDownload, proxy: EventLoopProxy<UserEvent>) {
    std::thread::Builder::new()
        .name("va-model-download".into())
        .spawn(move || {
            let mut last_percent = None;
            let result = job.run(&mut |progress| {
                let percent = progress.percent();
                if last_percent != Some(percent) {
                    last_percent = Some(percent);
                    let _ = proxy.send_event(UserEvent::Download(DownloadEvent::Progress(percent)));
                }
            });
            let event = match result {
                Ok(parts) => UserEvent::ModelLoaded(parts),
                Err(reason) => {
                    tracing::error!(%reason, "pobieranie modelu nieudane");
                    UserEvent::Download(DownloadEvent::Failed(reason))
                }
            };
            let _ = proxy.send_event(event);
        })
        .expect("wątek pobierania modelu");
}

/// Start/Stop: przy pobieraniu modelu i bez kontrolera — komunikat zamiast nagrania.
fn send(
    controller: Option<&ControllerHandle>,
    download: &DownloadState,
    unavailable: Option<&Problem>,
    command: Command,
) {
    if command == Command::Start
        && let Some(message) = download.blocks_recording()
    {
        notify(&Problem::ModelNotReady(message));
        return;
    }
    match (controller, unavailable) {
        (Some(controller), _) => controller.send(command),
        (None, Some(problem)) if command == Command::Start => notify(problem),
        (None, _) => tracing::warn!(?command, "nagrywanie niedostępne"),
    }
}

/// Najechanie na ikonę albo prawe kliknięcie — odśwież listę, zanim użytkownik ją zobaczy.
fn menu_may_open(event: &TrayIconEvent) -> bool {
    matches!(
        event,
        TrayIconEvent::Enter { .. }
            | TrayIconEvent::Click {
                button: MouseButton::Right,
                ..
            }
    )
}

/// Podmenu „Model” odświeżane przy każdej zmianie stanu pobierania albo kontrolera; w trakcie
/// pobierania linia stanu niesie procent.
fn show_model(
    menu: &TrayMenu,
    info: &ModelInfo,
    download: &DownloadState,
    shown: State,
    prompt: RemovePrompt,
) {
    let mut model = model_menu(info, download, shown);
    if let Some(progress) = state_with_percent(download)
        && let Some(line) = model.lines.first_mut()
    {
        *line = line.replace("· pobieranie", &format!("· {progress}"));
    }
    menu.show_model(&model, prompt);
}

/// Usuwa plik modelu z magazynu domyślnego (i plik częściowy); kontroler jest już zamknięty,
/// więc model nie siedzi w pamięci. Własna ścieżka nigdy tu nie trafia (pozycja nieaktywna).
fn remove_model_files(info: &ModelInfo) {
    let Some(dir) = info.path.parent() else {
        return;
    };
    match ModelStore::new(dir, LARGE_V3_TURBO).remove() {
        Ok(()) => tracing::info!(path = %info.path.display(), "model usunięty na życzenie"),
        Err(error) => tracing::error!(%error, "usuwanie modelu"),
    }
}

/// `open -R` zaznacza plik w Finderze; katalog otwiera wprost; brakujący plik — jego katalog.
fn reveal_in_finder(path: &Path) {
    let mut command = std::process::Command::new("open");
    if path.is_file() {
        command.arg("-R").arg(path);
    } else if path.is_dir() {
        command.arg(path);
    } else {
        command.arg(path.parent().unwrap_or(path));
    }
    if let Err(error) = command.spawn() {
        tracing::error!(%error, path = %path.display(), "Pokaż w Finderze");
    }
}

fn show_download(tray: &mut (TrayIcon, TrayMenu), state: &DownloadState, shown: State) {
    tray.1.show_download(state);
    let tooltip = state
        .menu_status()
        .unwrap_or_else(|| indicator_for(shown).tooltip.to_owned());
    let _ = tray.0.set_tooltip(Some(tooltip));
}

fn build_tray(
    state: State,
    config_path: &Path,
    login: &LoginItem,
) -> anyhow::Result<(TrayIcon, TrayMenu)> {
    let indicator = indicator_for(state);
    let menu = TrayMenu::new(config_path.to_owned(), login)?;
    let icon = TrayIconBuilder::new()
        .with_icon(icon(indicator)?)
        .with_tooltip(indicator.tooltip)
        .with_menu(Box::new(menu.menu.clone()))
        .with_menu_on_left_click(false)
        .build()
        .context("ikona w pasku menu")?;
    Ok((icon, menu))
}

fn show(tray: &TrayIcon, indicator: Indicator) {
    match icon(indicator) {
        Ok(icon) => {
            if let Err(error) = tray.set_icon(Some(icon)) {
                tracing::error!(%error, "zmiana ikony");
            }
        }
        Err(error) => tracing::error!(%error, "ikona"),
    }
    let _ = tray.set_tooltip(Some(indicator.tooltip));
}

fn icon(indicator: Indicator) -> anyhow::Result<Icon> {
    Icon::from_rgba(dot_rgba(indicator.dot), ICON_PIXELS, ICON_PIXELS).context("obraz ikony")
}

#[cfg(test)]
mod tests {
    use va_clipboard::Delivery;

    use super::*;

    #[test]
    // specky: crit 01M4K06AY6B3424GJDBAQQK3WY
    fn limit_reached_event_becomes_a_notification_with_configured_limit() {
        assert_eq!(
            problem_for(&ControllerEvent::LimitReached, 600),
            Some(Problem::RecordingLimitReached { limit_secs: 600 })
        );
    }

    const NOTIFY_ONLY: ReadySignal = ReadySignal {
        notify: true,
        sound: false,
    };

    #[test]
    // specky: crit 01M4KD152NZM9VP8S1V8VX1E6R
    fn transcript_ready_event_triggers_the_ready_signal_with_preview() {
        let event = ControllerEvent::TranscriptReady(TranscriptPreview::of("Gotowy tekst."));

        let preview = ready_preview(&event, NOTIFY_ONLY).expect("sygnał gotowe");

        assert_eq!(preview.text(), "Gotowy tekst.");
        assert_eq!(problem_for(&event, 600), None, "to nie jest problem");
    }

    #[test]
    // specky: crit 01M4KD152NK6764Z8AMZPSHQQN
    fn ready_signal_respects_configuration() {
        let event = ControllerEvent::TranscriptReady(TranscriptPreview::of("x"));
        let off = ReadySignal {
            notify: false,
            sound: false,
        };
        let sound_only = ReadySignal {
            notify: false,
            sound: true,
        };

        assert!(ready_preview(&event, off).is_none());
        assert!(ready_preview(&event, sound_only).is_some());
        assert!(
            ready_preview(&ControllerEvent::Delivered(Delivery::Written), NOTIFY_ONLY).is_none()
        );
    }

    #[test]
    // specky: crit 01M4KD152NR0QTPMC1RHSY5D6P
    fn event_log_names_the_kind_without_content() {
        let event = ControllerEvent::TranscriptReady(TranscriptPreview::of("sekret"));

        assert_eq!(event_kind(&event), "TranscriptReady");
        assert_eq!(
            event_kind(&ControllerEvent::HistoryChanged(vec![])),
            "HistoryChanged"
        );
    }

    #[test]
    fn state_and_delivery_events_do_not_notify() {
        assert_eq!(
            problem_for(&ControllerEvent::StateChanged(State::Recording), 600),
            None
        );
        assert_eq!(
            problem_for(&ControllerEvent::Delivered(Delivery::Written), 600),
            None
        );
        assert_eq!(
            problem_for(&ControllerEvent::NoSignal, 600),
            Some(Problem::MicrophoneSilent)
        );
    }
}
