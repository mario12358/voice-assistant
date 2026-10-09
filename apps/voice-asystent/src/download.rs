//! Pobieranie modelu po instalacji (VA-MODEL-1): stan widoczny w menu i podpowiedzi ikony.
//!
//! Logika stanu jest tu, bez GUI i bez sieci: wątek pobierania tylko zgłasza postęp
//! i wynik, a ten automat decyduje, co pokazać i czy Start może nagrywać.

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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadEvent {
    Progress(u8),
    Failed(String),
    Ready,
    Retry,
}

impl DownloadState {
    pub fn next(self, event: DownloadEvent) -> Self {
        use DownloadEvent as E;
        match (self, event) {
            (Self::Downloading { .. }, E::Progress(percent)) => Self::Downloading { percent },
            (Self::Downloading { .. }, E::Failed(reason)) => Self::Failed(reason),
            (Self::Downloading { .. }, E::Ready) => Self::Ready,
            (Self::Failed(_), E::Retry) => Self::Downloading { percent: 0 },
            (state, _) => state,
        }
    }

    /// Czy trzeba uruchomić wątek pobierania po tym przejściu.
    pub fn starts_download(before: &Self, after: &Self) -> bool {
        matches!(before, Self::Failed(_)) && matches!(after, Self::Downloading { .. })
    }

    /// Stała pozycja statusu w menu; `None` = nic do pokazania.
    pub fn menu_status(&self) -> Option<String> {
        match self {
            Self::NotNeeded | Self::Ready => None,
            Self::Downloading { percent } => Some(format!("Pobieranie modelu… {percent}%")),
            Self::Failed(reason) => Some(format!("Pobieranie modelu nieudane: {reason}")),
        }
    }

    /// Czy w menu jest aktywna pozycja „Ponów pobieranie”.
    pub fn can_retry(&self) -> bool {
        matches!(self, Self::Failed(_))
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
