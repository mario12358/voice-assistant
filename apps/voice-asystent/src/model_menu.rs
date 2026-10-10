//! Podmenu „Model” (VA-MODEL-2, VA-MODEL-3): jaki model jest używany i w jakim stanie,
//! „Pokaż w Finderze”, „Usuń model…”, „Pobierz ponownie”.

use std::path::PathBuf;

use va_core::state::State;

use crate::download::DownloadState;

pub const SHOW_MODEL_ID: &str = "model-show";
pub const REMOVE_MODEL_ID: &str = "model-remove";
pub const REDOWNLOAD_MODEL_ID: &str = "model-redownload";

/// Model wskazany przy starcie: z magazynu domyślnego albo z własnej ścieżki użytkownika.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    /// „large-v3-turbo” albo nazwa pliku własnego modelu.
    pub name: String,
    pub path: PathBuf,
    /// Rozmiar pliku (oczekiwany dla magazynu, faktyczny dla własnej ścieżki; 0 gdy brak).
    pub bytes: u64,
    pub custom: bool,
    /// Czy własny plik istniał przy starcie (dla magazynu stan niesie `DownloadState`).
    pub custom_present: bool,
}

/// Co pokazać w podmenu: linie informacyjne (nieklikalne) i aktywność poleceń.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelMenu {
    pub lines: Vec<String>,
    pub remove_enabled: bool,
    pub redownload_enabled: bool,
}

pub fn model_menu(info: &ModelInfo, download: &DownloadState, controller: State) -> ModelMenu {
    let state = if info.custom {
        if info.custom_present {
            "gotowy"
        } else {
            "brak"
        }
    } else {
        store_state(download)
    };
    let mut lines = vec![format!(
        "{} · {} · {state}",
        info.name,
        gigabytes(info.bytes)
    )];
    if info.custom {
        lines.push(format!("Własna ścieżka: {}", info.path.display()));
    }
    ModelMenu {
        lines,
        remove_enabled: !info.custom && download.can_remove(controller),
        redownload_enabled: !info.custom && download.can_retry(),
    }
}

fn store_state(download: &DownloadState) -> &'static str {
    match download {
        DownloadState::NotNeeded | DownloadState::Ready => "gotowy",
        DownloadState::Downloading { .. } => "pobieranie",
        DownloadState::Failed(_) => "brak (pobieranie nieudane)",
        DownloadState::Missing => "brak",
    }
}

/// Tekst stanu z procentem — osobno, bo `DownloadState` zna procent, a etykieta ma być krótka.
pub fn state_with_percent(download: &DownloadState) -> Option<String> {
    match download {
        DownloadState::Downloading { percent } => Some(format!("pobieranie {percent}%")),
        _ => None,
    }
}

/// „1,6 GB” — jedno miejsce po przecinku, po polsku; 0 bajtów → „0 GB”.
pub fn gigabytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 GB".to_owned();
    }
    let tenths = (bytes * 10).div_ceil(1_000_000_000);
    format!("{},{} GB", tenths / 10, tenths % 10)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_model() -> ModelInfo {
        ModelInfo {
            name: "large-v3-turbo".into(),
            path: "/Users/test/Library/Application Support/VoiceAsystent/models/ggml-large-v3-turbo.bin"
                .into(),
            bytes: 1_624_555_275,
            custom: false,
            custom_present: false,
        }
    }

    fn custom_model(present: bool) -> ModelInfo {
        ModelInfo {
            name: "moj-model.bin".into(),
            path: "/Users/test/Modele/moj-model.bin".into(),
            bytes: if present { 500_000_000 } else { 0 },
            custom: true,
            custom_present: present,
        }
    }

    #[test]
    // specky: crit 01M4K6M2ZYSP1Y9M0VD39GV7J8
    fn status_line_shows_name_size_and_state_for_every_download_state() {
        let cases = [
            (DownloadState::NotNeeded, "large-v3-turbo · 1,7 GB · gotowy"),
            (DownloadState::Ready, "large-v3-turbo · 1,7 GB · gotowy"),
            (
                DownloadState::Downloading { percent: 42 },
                "large-v3-turbo · 1,7 GB · pobieranie",
            ),
            (DownloadState::Missing, "large-v3-turbo · 1,7 GB · brak"),
            (
                DownloadState::Failed("x".into()),
                "large-v3-turbo · 1,7 GB · brak (pobieranie nieudane)",
            ),
        ];
        for (download, expected) in cases {
            let menu = model_menu(&store_model(), &download, State::Idle);
            assert_eq!(menu.lines, vec![expected.to_owned()], "{download:?}");
        }
        assert_eq!(
            state_with_percent(&DownloadState::Downloading { percent: 42 }).as_deref(),
            Some("pobieranie 42%")
        );
    }

    #[test]
    fn size_is_rounded_up_to_tenths_of_a_gigabyte() {
        assert_eq!(gigabytes(1_624_555_275), "1,7 GB");
        assert_eq!(gigabytes(1_600_000_000), "1,6 GB");
        assert_eq!(gigabytes(500_000_000), "0,5 GB");
        assert_eq!(gigabytes(0), "0 GB");
    }

    #[test]
    // specky: crit 01M4K6M2ZYCYED1VTTXDC6R542
    fn remove_is_disabled_while_downloading_or_transcribing() {
        let ready = model_menu(&store_model(), &DownloadState::Ready, State::Idle);
        assert!(ready.remove_enabled);
        assert!(!ready.redownload_enabled);

        let downloading = model_menu(
            &store_model(),
            &DownloadState::Downloading { percent: 3 },
            State::Idle,
        );
        assert!(!downloading.remove_enabled);

        let transcribing = model_menu(&store_model(), &DownloadState::Ready, State::Transcribing);
        assert!(!transcribing.remove_enabled);

        let missing = model_menu(&store_model(), &DownloadState::Missing, State::Idle);
        assert!(!missing.remove_enabled);
        assert!(missing.redownload_enabled);
    }

    #[test]
    // specky: crit 01M4K6M33CMHX4SQDRACYG60ZV
    fn custom_model_shows_file_and_path_and_disables_remove_and_redownload() {
        let present = model_menu(&custom_model(true), &DownloadState::NotNeeded, State::Idle);

        assert_eq!(
            present.lines,
            vec![
                "moj-model.bin · 0,5 GB · gotowy".to_owned(),
                "Własna ścieżka: /Users/test/Modele/moj-model.bin".to_owned(),
            ]
        );
        assert!(!present.remove_enabled);
        assert!(!present.redownload_enabled);

        let missing = model_menu(&custom_model(false), &DownloadState::NotNeeded, State::Idle);
        assert_eq!(missing.lines[0], "moj-model.bin · 0 GB · brak");
        assert!(!missing.remove_enabled && !missing.redownload_enabled);
    }
}
