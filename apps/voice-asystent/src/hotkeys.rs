//! Skróty globalne (VA-REC-1, VA-REC-2): ctrl+cmd+r → Start, ctrl+cmd+s → Stop.
//!
//! global-hotkey rejestruje je przez Carbon `RegisterEventHotKey`, więc działają przy dowolnej
//! aktywnej aplikacji i nie wymagają uprawnienia „Dostępność”.

use global_hotkey::hotkey::{Code, HotKey, Modifiers};
use global_hotkey::{GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState};
use va_core::controller::Command;

pub struct Hotkeys {
    start: HotKey,
    stop: HotKey,
}

impl Default for Hotkeys {
    fn default() -> Self {
        let modifiers = Some(Modifiers::CONTROL | Modifiers::SUPER);
        Self {
            start: HotKey::new(modifiers, Code::KeyR),
            stop: HotKey::new(modifiers, Code::KeyS),
        }
    }
}

impl Hotkeys {
    /// Rejestracja w systemie; konflikt z inną aplikacją zwraca komunikat dla użytkownika.
    pub fn register(&self, manager: &GlobalHotKeyManager) -> Result<(), String> {
        manager
            .register_all(&[self.start, self.stop])
            .map_err(|error| {
                format!(
                    "Skróty ctrl+cmd+r / ctrl+cmd+s niedostępne ({error}) — użyj kliknięcia ikony"
                )
            })
    }

    /// Polecenie po naciśnięciu skrótu; zwolnienie klawiszy jest pomijane.
    pub fn command_for(&self, event: &GlobalHotKeyEvent) -> Option<Command> {
        if event.state != HotKeyState::Pressed {
            return None;
        }
        if event.id == self.start.id() {
            Some(Command::Start)
        } else if event.id == self.stop.id() {
            Some(Command::Stop)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pressed(hotkey: HotKey) -> GlobalHotKeyEvent {
        GlobalHotKeyEvent {
            id: hotkey.id(),
            state: HotKeyState::Pressed,
        }
    }

    #[test]
    // specky: crit 01M4EKHCREF952J778E6M72D9J
    fn ctrl_cmd_r_starts_recording() {
        let hotkeys = Hotkeys::default();
        let shortcut = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SUPER), Code::KeyR);

        assert_eq!(
            hotkeys.command_for(&pressed(shortcut)),
            Some(Command::Start)
        );
    }

    #[test]
    // specky: crit 01M4EKHCW5QES1NN0PEX6G2BJR
    fn ctrl_cmd_s_stops_recording() {
        let hotkeys = Hotkeys::default();
        let shortcut = HotKey::new(Some(Modifiers::CONTROL | Modifiers::SUPER), Code::KeyS);

        assert_eq!(hotkeys.command_for(&pressed(shortcut)), Some(Command::Stop));
    }

    #[test]
    fn key_release_is_ignored() {
        let hotkeys = Hotkeys::default();
        let released = GlobalHotKeyEvent {
            id: hotkeys.start.id(),
            state: HotKeyState::Released,
        };

        assert_eq!(hotkeys.command_for(&released), None);
    }

    #[test]
    fn other_shortcuts_are_ignored() {
        let hotkeys = Hotkeys::default();
        let other = HotKey::new(Some(Modifiers::SUPER), Code::KeyR);

        assert_eq!(hotkeys.command_for(&pressed(other)), None);
    }
}
