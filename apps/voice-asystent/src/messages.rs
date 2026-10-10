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
    /// `model_path` z konfiguracji wskazuje plik, którego nie ma (VA-MODEL-3) — nic nie pobieramy.
    CustomModelMissing(String),
    /// Model w trakcie pobierania albo pobieranie nieudane (tekst ze stanu pobierania).
    ModelNotReady(String),
    /// Inny błąd pojedynczego nagrania (tekst z kontrolera).
    RecordingFailed(String),
    /// Nagranie zakończone automatycznie po osiągnięciu limitu długości (VA-REC-6).
    RecordingLimitReached { limit_secs: u32 },
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
        Problem::CustomModelMissing(path) => (
            "Brak modelu",
            format!(
                "Plik modelu z config.toml (model_path) nie istnieje: {path}. Popraw ścieżkę \
                 albo usuń to pole, aby aplikacja pobrała model domyślny."
            ),
        ),
        Problem::ModelNotReady(text) => ("Model niedostępny", text.clone()),
        Problem::RecordingFailed(reason) => (
            "Nagranie nieudane",
            format!("Nagranie nie powiodło się: {reason}"),
        ),
        Problem::RecordingLimitReached { limit_secs } => (
            "Osiągnięto limit długości nagrania",
            format!(
                "Nagranie zakończono automatycznie po {} — tekst trafi do schowka jak po \
                 ctrl+cmd+s. Limit zmienisz w config.toml (max_recording_secs).",
                duration_label(*limit_secs)
            ),
        ),
    }
}

/// „10 min” dla pełnych minut, inaczej „90 s”.
fn duration_label(seconds: u32) -> String {
    if seconds >= 60 && seconds % 60 == 0 {
        format!("{} min", seconds / 60)
    } else {
        format!("{seconds} s")
    }
}

/// Krótka, stała pozycja menu dla problemów, które blokują nagrywanie do skutku.
pub fn menu_notice(problem: &Problem) -> Option<String> {
    match problem {
        Problem::GpuMissing => Some("Wymagane GPU (Metal) — nagrywanie niedostępne".to_owned()),
        Problem::ModelUnavailable => Some("Brak modelu — nagrywanie niedostępne".to_owned()),
        Problem::CustomModelMissing(path) => {
            Some(format!("Brak modelu: {path} — nagrywanie niedostępne"))
        }
        Problem::MicrophoneSilent
        | Problem::ModelNotReady(_)
        | Problem::RecordingFailed(_)
        | Problem::RecordingLimitReached { .. } => None,
    }
}

/// Powiadomienie systemowe w osobnym wątku — wysyłka nie może blokować pętli zdarzeń.
pub fn notify(problem: &Problem) {
    let (title, body) = message(problem);
    send_in_background(title, body);
}

pub const TRANSCRIPT_READY_TITLE: &str = "Transkrypcja w schowku";
/// Krótki dźwięk systemowy macOS po zapisie transkrypcji (VA-UX-1).
const READY_SOUND: &str = "/System/Library/Sounds/Glass.aiff";

/// Jak użytkownik dowiaduje się, że tekst jest w schowku (z `config.toml`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadySignal {
    pub notify: bool,
    pub sound: bool,
}

/// Sygnał „gotowe” (VA-UX-1): powiadomienie z podglądem i/lub dźwięk; treść poza logami.
pub fn signal_ready(signal: ReadySignal, preview: &str) {
    if signal.notify {
        send_in_background(TRANSCRIPT_READY_TITLE, preview.to_owned());
    }
    if signal.sound {
        std::thread::spawn(|| {
            if let Err(error) = std::process::Command::new("afplay")
                .arg(READY_SOUND)
                .status()
            {
                tracing::warn!(%error, "dźwięk „gotowe” nie został odtworzony");
            }
        });
    }
}

fn send_in_background(title: &'static str, body: String) {
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
            menu_notice(&Problem::ModelUnavailable).as_deref(),
            Some("Brak modelu — nagrywanie niedostępne")
        );
    }

    #[test]
    // specky: crit 01M4K6M33CWKR54XAT4JVG3HWG
    fn missing_custom_model_names_the_path_in_menu_and_notification() {
        let problem = Problem::CustomModelMissing("/tmp/model.bin".into());

        let (title, body) = message(&problem);

        assert_eq!(title, "Brak modelu");
        assert!(
            body.contains("/tmp/model.bin") && body.contains("model_path"),
            "{body}"
        );
        assert_eq!(
            menu_notice(&problem).as_deref(),
            Some("Brak modelu: /tmp/model.bin — nagrywanie niedostępne")
        );
    }

    #[test]
    fn one_off_failures_are_not_pinned_to_menu() {
        assert_eq!(menu_notice(&Problem::MicrophoneSilent), None);
        assert_eq!(menu_notice(&Problem::RecordingFailed("x".into())), None);
    }

    #[test]
    // specky: crit 01M4K06AY6B3424GJDBAQQK3WY
    fn recording_limit_notification_names_the_limit_in_minutes() {
        let (title, body) = message(&Problem::RecordingLimitReached { limit_secs: 600 });

        assert_eq!(title, "Osiągnięto limit długości nagrania");
        assert!(body.contains("po 10 min"), "{body}");
        assert!(body.contains("max_recording_secs"), "{body}");
        assert_eq!(
            menu_notice(&Problem::RecordingLimitReached { limit_secs: 600 }),
            None
        );
    }

    #[test]
    fn limit_below_a_minute_is_shown_in_seconds() {
        let (_, body) = message(&Problem::RecordingLimitReached { limit_secs: 20 });

        assert!(body.contains("po 20 s"), "{body}");
        assert_eq!(duration_label(90), "90 s");
        assert_eq!(duration_label(120), "2 min");
    }

    #[test]
    fn recording_failure_carries_reason() {
        let (_, body) = message(&Problem::RecordingFailed("nie znaleziono mikrofonu".into()));

        assert!(body.contains("nie znaleziono mikrofonu"));
    }
}
