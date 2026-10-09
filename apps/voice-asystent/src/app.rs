//! Pętla zdarzeń aplikacji paska menu (wątek główny, wymóg AppKit).

use anyhow::Context;
use tao::event::{Event, StartCause};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tao::platform::macos::{ActivationPolicy, EventLoopExtMacOS};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder, TrayIconEvent};
use va_core::controller::{self, ControllerEvent, ControllerHandle, ControllerParts};
use va_core::state::State;

use crate::click::command_for;
use crate::indicator::{ICON_PIXELS, Indicator, dot_rgba, indicator_for};

#[derive(Debug)]
enum UserEvent {
    Controller(ControllerEvent),
    Tray(TrayIconEvent),
}

pub fn run(controller_parts: Option<ControllerParts>) -> anyhow::Result<()> {
    let mut event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    event_loop.set_activation_policy(ActivationPolicy::Accessory);
    let proxy = event_loop.create_proxy();
    let tray_proxy = event_loop.create_proxy();
    TrayIconEvent::set_event_handler(Some(move |event| {
        let _ = tray_proxy.send_event(UserEvent::Tray(event));
    }));
    let controller: Option<ControllerHandle> = controller_parts.map(|parts| {
        controller::spawn(
            parts,
            Box::new(move |event| {
                let _ = proxy.send_event(UserEvent::Controller(event));
            }),
        )
    });
    let mut tray: Option<TrayIcon> = None;
    let mut shown = State::Idle;
    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        match event {
            Event::NewEvents(StartCause::Init) => match build_tray(State::Idle) {
                Ok(icon) => tray = Some(icon),
                Err(error) => {
                    tracing::error!(%error, "nie udało się utworzyć ikony w pasku menu");
                    *control_flow = ControlFlow::Exit;
                }
            },
            Event::UserEvent(UserEvent::Controller(event)) => {
                if let ControllerEvent::StateChanged(state) = event {
                    shown = state;
                    if let Some(tray) = &tray {
                        show(tray, indicator_for(state));
                    }
                }
                tracing::debug!(?event, "zdarzenie kontrolera");
            }
            Event::UserEvent(UserEvent::Tray(event)) => {
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
            _ => {}
        }
    })
}

fn build_tray(state: State) -> anyhow::Result<TrayIcon> {
    let indicator = indicator_for(state);
    TrayIconBuilder::new()
        .with_icon(icon(indicator)?)
        .with_tooltip(indicator.tooltip)
        .with_menu_on_left_click(false)
        .build()
        .context("ikona w pasku menu")
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
