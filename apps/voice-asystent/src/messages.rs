//! Komunikaty dla użytkownika (VA-PLAT-2, VA-STT-2, VA-MODEL-1) — tylko tu powstaje tekst po polsku.

/// Problem, o którym trzeba powiedzieć użytkownikowi.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Problem {
    /// Mikrofon dał ciszę cyfrową — prawie zawsze brak zgody w Prywatności.
    MicrophoneSilent,
    /// Brak GPU Metal — transkrypcja na CPU nie jest wspierana.
    GpuMissing,
    /// Model nie jest pobrany albo nie dał się załadować.
    ModelUnavailable,
    /// Inny błąd pojedynczego nagrania (tekst z kontrolera).
    RecordingFailed(String),
}

/// Tytuł i treść powiadomienia / pozycji menu.
pub fn message(problem: &Problem) -> (&'static str, String) {
    match problem {
        Problem::MicrophoneSilent => (
            "Brak dostępu do mikrofonu",
            "Mikrofon nie przekazał dźwięku. Zezwól na dostęp: Ustawienia systemowe → \
             Prywatność i ochrona → Mikrofon → włącz VoiceAsystent, potem spróbuj ponownie."
                .to_owned(),
        ),
        Problem::GpuMissing => (
            "Wymagane GPU (Metal)",
            "Wymagane GPU (Metal) — praca na CPU nie jest wspierana. Nagrywanie jest niedostępne."
                .to_owned(),
        ),
        Problem::ModelUnavailable => (
            "Model niedostępny",
            "Model rozpoznawania mowy nie jest pobrany. Nagrywanie będzie dostępne po pobraniu \
             modelu."
                .to_owned(),
        ),
        Problem::RecordingFailed(reason) => (
            "Nagranie nieudane",
            format!("Nagranie nie powiodło się: {reason}"),
        ),
    }
}

/// Krótka, stała pozycja menu dla problemów, które blokują nagrywanie do skutku.
pub fn menu_notice(problem: &Problem) -> Option<&'static str> {
    match problem {
        Problem::GpuMissing => Some("Wymagane GPU (Metal) — nagrywanie niedostępne"),
        Problem::ModelUnavailable => Some("Brak modelu — nagrywanie niedostępne"),
        Problem::MicrophoneSilent | Problem::RecordingFailed(_) => None,
    }
}

/// Powiadomienie systemowe w osobnym wątku — wysyłka nie może blokować pętli zdarzeń.
pub fn notify(problem: &Problem) {
    let (title, body) = message(problem);
    std::thread::spawn(move || {
        if let Err(error) = mac_notification_sys::send_notification(title, None, &body, None) {
            tracing::warn!(%error, "powiadomienie systemowe nie zostało wysłane");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_microphone_permission_explains_where_to_grant_it() {
        let (title, body) = message(&Problem::MicrophoneSilent);

        assert_eq!(title, "Brak dostępu do mikrofonu");
        assert!(body.contains("Prywatność i ochrona → Mikrofon"), "{body}");
    }

    #[test]
    // specky: crit 01M4EKHCMW0ZW9K5GYM7D2NM9H
    fn missing_gpu_says_cpu_is_not_supported_and_stays_in_menu() {
        let (_, body) = message(&Problem::GpuMissing);

        assert!(body.starts_with("Wymagane GPU (Metal) — praca na CPU nie jest wspierana"));
        assert!(menu_notice(&Problem::GpuMissing).is_some());
    }

    #[test]
    fn missing_model_stays_in_menu() {
        assert_eq!(
            menu_notice(&Problem::ModelUnavailable),
            Some("Brak modelu — nagrywanie niedostępne")
        );
    }

    #[test]
    fn one_off_failures_are_not_pinned_to_menu() {
        assert_eq!(menu_notice(&Problem::MicrophoneSilent), None);
        assert_eq!(menu_notice(&Problem::RecordingFailed("x".into())), None);
    }

    #[test]
    fn recording_failure_carries_reason() {
        let (_, body) = message(&Problem::RecordingFailed("nie znaleziono mikrofonu".into()));

        assert!(body.contains("nie znaleziono mikrofonu"));
    }
}
