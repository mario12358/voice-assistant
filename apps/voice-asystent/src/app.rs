//! Pętla zdarzeń aplikacji paska menu (wątek główny, wymóg AppKit).

use std::path::{Path, PathBuf};

use anyhow::Context;
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager};
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};
use va_core::controller::{self, Command, ControllerEvent, ControllerHandle, ControllerParts};
use va_core::state::State;

use crate::click::command_for;
use crate::download::{DownloadEvent, DownloadState};
use crate::hotkeys::Hotkeys;
use crate::indicator::{ICON_PIXELS, Indicator, dot_rgba, indicator_for};
use crate::messages::{Problem, menu_notice, notify};
use crate::microphones::{QUIT_ID, microphone_from_menu_id, select_microphone};
use crate::startup::ModelDownload;
use crate::tray_menu::{RETRY_DOWNLOAD_ID, TrayMenu};

enum UserEvent {
    Controller(ControllerEvent),
    Tray(TrayIconEvent),
    Menu(MenuEvent),
    Hotkey(GlobalHotKeyEvent),
    Download(DownloadEvent),
    ModelLoaded(ControllerParts),
}

pub fn run(
    controller_parts: Result<ControllerParts, Problem>,
    download: Option<ModelDownload>,
    config_path: PathBuf,
    recording_limit_secs: u32,
) -> anyhow::Result<()> {
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
        Some(job) => {
            spawn_download(job.clone(), proxy.clone());
            DownloadState::Downloading { percent: 0 }
        }
        None => DownloadState::NotNeeded,
    };
    let mut tray: Option<(TrayIcon, TrayMenu)> = None;
    let mut shown = State::Idle;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => match build_tray(State::Idle, &config_path) {
                Ok(mut built) => {
                    if let Some(problem) = &hotkeys_problem {
                        tracing::error!(%problem, "rejestracja skrótów");
                        built.1.show_notice(problem);
                    }
                    if download_state == DownloadState::NotNeeded
                        && let Some(notice) = unavailable.as_ref().and_then(menu_notice)
                    {
                        built.1.show_notice(notice);
                    }
                    show_download(&mut built, &download_state, shown);
                    tray = Some(built);
                }
                Err(error) => {
                    tracing::error!(%error, "nie udało się utworzyć ikony w pasku menu");
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::UserEvent(UserEvent::Controller(event)) => {
                if let ControllerEvent::StateChanged(state) = &event {
                    shown = *state;
                    if let Some((icon, _)) = &tray {
                        show(icon, indicator_for(*state));
                    }
                }
                if let Some(problem) = problem_for(&event, recording_limit_secs) {
                    notify(&problem);
                }
                tracing::debug!(?event, "zdarzenie kontrolera");
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
                if id == QUIT_ID {
                    *control_flow = ControlFlow::Exit;
                } else if id == RETRY_DOWNLOAD_ID {
                    let before = download_state.clone();
                    download_state = before.clone().next(DownloadEvent::Retry);
                    if DownloadState::starts_download(&before, &download_state)
                        && let Some(job) = &download
                    {
                        spawn_download(job.clone(), proxy.clone());
                    }
                    if let Some(built) = &mut tray {
                        show_download(built, &download_state, shown);
                    }
                } else if let Some(name) = microphone_from_menu_id(id) {
                    if let Err(error) = select_microphone(&config_path, name) {
                        tracing::error!(%error, "zapis wyboru mikrofonu");
                    }
                    if let Some((_, menu)) = &tray {
                        menu.refresh_microphones();
                    }
                }
            }
            Event::UserEvent(UserEvent::Download(event)) => {
                download_state = download_state.clone().next(event);
                if let Some(built) = &mut tray {
                    show_download(built, &download_state, shown);
                }
            }
            Event::UserEvent(UserEvent::ModelLoaded(parts)) => {
                controller = Some(spawn_controller(parts, proxy.clone()));
                unavailable = None;
                download_state = download_state.clone().next(DownloadEvent::Ready);
                if let Some(built) = &mut tray {
                    show_download(built, &download_state, shown);
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
        ControllerEvent::StateChanged(_) | ControllerEvent::Delivered(_) => None,
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

fn show_download(tray: &mut (TrayIcon, TrayMenu), state: &DownloadState, shown: State) {
    tray.1.show_download(state);
    let tooltip = state
        .menu_status()
        .unwrap_or_else(|| indicator_for(shown).tooltip.to_owned());
    let _ = tray.0.set_tooltip(Some(tooltip));
}

fn build_tray(state: State, config_path: &Path) -> anyhow::Result<(TrayIcon, TrayMenu)> {
    let indicator = indicator_for(state);
    let menu = TrayMenu::new(config_path.to_owned())?;
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
