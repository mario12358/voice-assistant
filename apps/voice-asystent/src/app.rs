//! Pętla zdarzeń aplikacji paska menu (wątek główny, wymóg AppKit).

use std::path::PathBuf;

use anyhow::Context;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::menu::MenuEvent;
use tray_icon::{Icon, MouseButton, TrayIcon, TrayIconBuilder, TrayIconEvent};
use va_core::controller::{self, ControllerEvent, ControllerHandle, ControllerParts};
use va_core::state::State;

use crate::click::command_for;
use crate::indicator::{ICON_PIXELS, Indicator, dot_rgba, indicator_for};
use crate::microphones::{QUIT_ID, microphone_from_menu_id, select_microphone};
use crate::tray_menu::TrayMenu;

#[derive(Debug)]
enum UserEvent {
    Controller(ControllerEvent),
    Tray(TrayIconEvent),
    Menu(MenuEvent),
}

pub fn run(controller_parts: Option<ControllerParts>, config_path: PathBuf) -> anyhow::Result<()> {
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    event_loop.set_activation_policy(ActivationPolicy::Accessory);
    let proxy = event_loop.create_proxy();
    let tray_proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy.send_event(UserEvent::Tray(event));
    }));
    let menu_proxy = event_loop.create_proxy();
    MenuEvent::set_event_handler(Some(move |event| {
        let _ = menu_proxy.send_event(UserEvent::Menu(event));
    }));
    let controller: Option<ControllerHandle> = controller_parts.map(|parts| {
        controller::spawn(
            parts,
            Box::new(move |event| {
                let _ = proxy.send_event(UserEvent::Controller(event));
            }),
        )
    });
    let mut tray: Option<(TrayIcon, TrayMenu)> = None;
    let mut shown = State::Idle;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => match build_tray(State::Idle, &config_path) {
                Ok(built) => tray = Some(built),
                Err(error) => {
                    tracing::error!(%error, "nie udało się utworzyć ikony w pasku menu");
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::UserEvent(UserEvent::Controller(event)) => {
                if let ControllerEvent::StateChanged(state) = event {
                    shown = state;
                    if let Some((icon, _)) = &tray {
                        show(icon, indicator_for(state));
                    }
                }
                tracing::debug!(?event, "zdarzenie kontrolera");
            }
            Event::UserEvent(UserEvent::Tray(event)) => {
                if let Some((_, menu)) = &tray
                    && menu_may_open(&event)
                {
                    menu.refresh_microphones();
                }
                let Some(command) = command_for(&event, shown) else {
                    return;
                };
                match &controller {
                    Some(controller) => controller.send(command),
                    None => {
                        tracing::warn!(?command, "nagrywanie niedostępne (brak GPU albo modelu)")
                    }
                }
            }
            Event::UserEvent(UserEvent::Menu(event)) => {
                let id = event.id.as_ref();
                if id == QUIT_ID {
                    *control_flow = ControlFlow::Exit;
                } else if let Some(name) = microphone_from_menu_id(id) {
                    if let Err(error) = select_microphone(&config_path, name) {
                        tracing::error!(%error, "zapis wyboru mikrofonu");
                    }
                    if let Some((_, menu)) = &tray {
                        menu.refresh_microphones();
                    }
                }
            }
            _ => {}
        }
    })
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

fn build_tray(state: State, config_path: &std::path::Path) -> anyhow::Result<(TrayIcon, TrayMenu)> {
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
