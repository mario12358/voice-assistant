//! Pobieranie modelu po instalacji (VA-MODEL-1): stan widoczny w menu i podpowiedzi ikony.
//!
//! Logika stanu jest tu, bez GUI i bez sieci: wątek pobierania tylko zgłasza postęp
//! i wynik, a ten automat decyduje, co pokazać i czy Start może nagrywać.

use va_core::state::State;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadState {
    /// Model był na dysku — nic nie pobieramy i nie łączymy się z siecią.
    NotNeeded,
    Downloading {
        percent: u8,
    },
    Failed(String),
    /// Model pobrany i załadowany — nagrywanie dostępne.
    Ready,
    /// Model usunięty przez użytkownika (VA-MODEL-2) — czeka na „Pobierz ponownie”.
    Missing,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadEvent {
    Progress(u8),
    Failed(String),
    Ready,
    Retry,
    /// Użytkownik potwierdził usunięcie modelu w podmenu „Model”.
    Removed,
}

impl DownloadState {
    pub fn next(self, event: DownloadEvent) -> Self {
        use DownloadEvent as E;
        match (self, event) {
            (Self::Downloading { .. }, E::Progress(percent)) => Self::Downloading { percent },
            (Self::Downloading { .. }, E::Failed(reason)) => Self::Failed(reason),
            (Self::Downloading { .. }, E::Ready) => Self::Ready,
            (Self::Failed(_) | Self::Missing, E::Retry) => Self::Downloading { percent: 0 },
            (Self::NotNeeded | Self::Ready | Self::Failed(_), E::Removed) => Self::Missing,
            (state, _) => state,
        }
    }

    /// Czy trzeba uruchomić wątek pobierania po tym przejściu.
    pub fn starts_download(before: &Self, after: &Self) -> bool {
        matches!(before, Self::Failed(_) | Self::Missing)
            && matches!(after, Self::Downloading { .. })
    }

    /// Stała pozycja statusu w menu; `None` = nic do pokazania.
    pub fn menu_status(&self) -> Option<String> {
        match self {
            Self::NotNeeded | Self::Ready => None,
            Self::Downloading { percent } => Some(format!("Pobieranie modelu… {percent}%")),
            Self::Failed(reason) => Some(format!("Pobieranie modelu nieudane: {reason}")),
            Self::Missing => Some("Brak modelu — nagrywanie niedostępne".to_owned()),
        }
    }

    /// Czy w menu jest aktywna pozycja „Ponów pobieranie” / „Pobierz ponownie”.
    pub fn can_retry(&self) -> bool {
        matches!(self, Self::Failed(_) | Self::Missing)
    }

    /// „Usuń model” tylko z modelem na dysku i gdy kontroler nie nagrywa ani nie transkrybuje.
    pub fn can_remove(&self, controller: State) -> bool {
        matches!(self, Self::NotNeeded | Self::Ready)
            && matches!(controller, State::Idle | State::Error)
    }

    /// Komunikat zamiast nagrania, gdy model jeszcze nie jest gotowy.
    pub fn blocks_recording(&self) -> Option<String> {
        match self {
            Self::NotNeeded | Self::Ready => None,
            Self::Downloading { percent } => Some(format!(
                "Model jest pobierany ({percent}%). Nagrywanie będzie dostępne po zakończeniu."
            )),
            Self::Failed(_) => Some(
                "Model nie został pobrany. Wybierz „Ponów pobieranie” w menu ikony.".to_owned(),
            ),
            Self::Missing => {
                Some("Model został usunięty. Wybierz „Pobierz ponownie” w menu Model.".to_owned())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn downloading(percent: u8) -> DownloadState {
        DownloadState::Downloading { percent }
    }

    #[test]
    // specky: crit 01M4EQH9E1TXJW9TH6EDZWWZVH
    fn progress_is_shown_in_menu_while_downloading() {
        let state = downloading(0).next(DownloadEvent::Progress(42));

        assert_eq!(
            state.menu_status().as_deref(),
            Some("Pobieranie modelu… 42%")
        );
    }

    #[test]
    // specky: crit 01M4EQH9E16AKTD5RP9EEEPMND
    fn start_during_download_shows_message_instead_of_recording() {
        let message = downloading(42).blocks_recording().unwrap();

        assert!(message.contains("42%"), "{message}");
        assert!(message.contains("po zakończeniu"), "{message}");
    }

    #[test]
    fn failure_offers_retry_and_retry_restarts_download() {
        let failed = downloading(10).next(DownloadEvent::Failed("brak sieci".into()));

        assert!(failed.can_retry());
        assert!(failed.blocks_recording().is_some());
        let retried = failed.clone().next(DownloadEvent::Retry);
        assert_eq!(retried, downloading(0));
        assert!(DownloadState::starts_download(&failed, &retried));
    }

    #[test]
    fn finished_download_unblocks_recording_and_clears_menu() {
        let ready = downloading(99).next(DownloadEvent::Ready);

        assert_eq!(ready, DownloadState::Ready);
        assert_eq!(ready.blocks_recording(), None);
        assert_eq!(ready.menu_status(), None);
    }

    #[test]
    // specky: crit 01M4K6M2ZYTC7D38NMHZJXF1FV
    fn removed_model_blocks_recording_until_download_is_retried() {
        for before in [
            DownloadState::NotNeeded,
            DownloadState::Ready,
            DownloadState::Failed("x".into()),
        ] {
            let missing = before.next(DownloadEvent::Removed);
            assert_eq!(missing, DownloadState::Missing);
            assert!(missing.can_retry());
            assert!(missing.blocks_recording().unwrap().contains("usunięty"));
            assert_eq!(
                missing.menu_status().as_deref(),
                Some("Brak modelu — nagrywanie niedostępne")
            );
            let retried = missing.clone().next(DownloadEvent::Retry);
            assert_eq!(retried, downloading(0));
            assert!(DownloadState::starts_download(&missing, &retried));
        }
        assert_eq!(
            downloading(5).next(DownloadEvent::Removed),
            downloading(5),
            "usuwanie w trakcie pobierania jest ignorowane"
        );
    }

    #[test]
    // specky: crit 01M4K6M2ZYCYED1VTTXDC6R542
    fn remove_is_only_allowed_with_a_model_on_disk_and_an_idle_controller() {
        assert!(DownloadState::NotNeeded.can_remove(State::Idle));
        assert!(DownloadState::Ready.can_remove(State::Error));
        assert!(!downloading(50).can_remove(State::Idle));
        assert!(!DownloadState::Ready.can_remove(State::Transcribing));
        assert!(!DownloadState::Ready.can_remove(State::Recording));
        assert!(!DownloadState::Missing.can_remove(State::Idle));
        assert!(!DownloadState::Failed("x".into()).can_remove(State::Idle));
    }

    #[test]
    // specky: crit 01M4EQH9E1X3ST7739SZQV3N0Z
    fn model_already_present_never_starts_a_download() {
        let mut state = DownloadState::NotNeeded;
        for event in [
            DownloadEvent::Retry,
            DownloadEvent::Progress(5),
            DownloadEvent::Failed("x".into()),
        ] {
            let after = state.clone().next(event);
            assert!(!DownloadState::starts_download(&state, &after));
            state = after;
        }

        assert_eq!(state, DownloadState::NotNeeded);
        assert_eq!(state.blocks_recording(), None);
    }

    #[test]
    fn retry_while_downloading_is_ignored() {
        let state = downloading(30).next(DownloadEvent::Retry);

        assert_eq!(state, downloading(30));
    }
}
