//! Lewe kliknięcie ikony (VA-UI-2): szare kółko → Start, czerwone → Stop.
//!
//! Decyzja zależy od stanu, który pokazuje ikona — użytkownik klika to, co widzi. Menu
//! (mikrofon, zakończ) zostaje pod prawym kliknięciem, więc lewe nie otwiera menu.

use tray_icon::{MouseButton, MouseButtonState, TrayIconEvent};
use va_core::controller::Command;
use va_core::state::State;

/// Polecenie dla kontrolera po zdarzeniu ikony; `None` = zdarzenie nie dotyczy nagrywania.
pub fn command_for(event: &TrayIconEvent, shown: State) -> Option<Command> {
    let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    else {
        return None;
    };
    match shown {
        State::Idle | State::Error => Some(Command::Start),
        State::Recording => Some(Command::Stop),
        State::Transcribing => None,
    }
}

#[cfg(test)]
mod tests {
    use tray_icon::{Rect, TrayIconId};

    use super::*;

    fn click(button: MouseButton, button_state: MouseButtonState) -> TrayIconEvent {
        TrayIconEvent::Click {
            id: TrayIconId::new("va"),
            position: Default::default(),
            rect: Rect::default(),
            button,
            button_state,
        }
    }

    fn left_click() -> TrayIconEvent {
        click(MouseButton::Left, MouseButtonState::Up)
    }

    #[test]
    // specky: crit 01M4EKHD49V4AHGCTQQE585MYW
    fn clicking_gray_dot_starts_recording() {
        assert_eq!(
            command_for(&left_click(), State::Idle),
            Some(Command::Start)
        );
    }

    #[test]
    // specky: crit 01M4EKHD492278HSQ2MBTPD3CC
    fn clicking_red_dot_stops_recording() {
        assert_eq!(
            command_for(&left_click(), State::Recording),
            Some(Command::Stop)
        );
    }

    #[test]
    fn click_during_transcription_does_nothing() {
        assert_eq!(command_for(&left_click(), State::Transcribing), None);
    }

    #[test]
    fn only_left_button_release_counts() {
        assert_eq!(
            command_for(
                &click(MouseButton::Left, MouseButtonState::Down),
                State::Idle
            ),
            None
        );
        assert_eq!(
            command_for(
                &click(MouseButton::Right, MouseButtonState::Up),
                State::Idle
            ),
            None
        );
    }

    #[test]
    fn other_tray_events_are_ignored() {
        let enter = TrayIconEvent::Enter {
            id: TrayIconId::new("va"),
            position: Default::default(),
            rect: Rect::default(),
        };

        assert_eq!(command_for(&enter, State::Idle), None);
    }
}
