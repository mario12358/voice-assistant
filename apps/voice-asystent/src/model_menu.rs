//! Podmenu „Model” (VA-MODEL-2, VA-MODEL-3): jaki model jest używany i w jakim stanie,
//! „Pokaż w Finderze”, „Usuń model…”, „Pobierz ponownie”.

use std::path::PathBuf;

use va_core::state::State;

use crate::download::DownloadState;
use va_config::ModelVariant;

pub const SHOW_MODEL_ID: &str = "model-show";
pub const REMOVE_OTHER_VARIANT_ID: &str = "model-remove-other";
const VARIANT_ID_PREFIX: &str = "model-variant:";
const VARIANTS: [(ModelVariant, &str, &str); 2] = [
    (ModelVariant::Full, "full", "Pełny"),
    (ModelVariant::Q5_0, "q5_0", "Skwantyzowany q5_0"),
];

/// Pozycja wyboru wariantu w podmenu „Model” (VA-MODEL-4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariantItem {
    pub id: String,
    pub label: String,
    pub checked: bool,
    pub enabled: bool,
}

/// Pozycje „Pełny (1,6 GB)” / „Skwantyzowany q5_0 (0,6 GB)” z zaznaczonym używanym wariantem.
/// Przełączenie tylko z magazynem aplikacji, gdy model nie pracuje (`switch_allowed`).
pub fn variant_items(info: &ModelInfo, switch_allowed: bool) -> Vec<VariantItem> {
    VARIANTS
        .iter()
        .map(|(variant, code, label)| VariantItem {
            id: format!("{VARIANT_ID_PREFIX}{code}"),
            label: format!("{label} ({})", gigabytes(va_model::spec_for(*variant).size)),
            checked: info.variant == Some(*variant),
            enabled: switch_allowed && info.variant.is_some() && info.variant != Some(*variant),
        })
        .collect()
}

pub fn variant_from_menu_id(id: &str) -> Option<ModelVariant> {
    let code = id.strip_prefix(VARIANT_ID_PREFIX)?;
    VARIANTS
        .iter()
        .find(|(_, known, _)| *known == code)
        .map(|(variant, _, _)| *variant)
}

/// Drugi (nieużywany) wariant: ten, który nie jest wybrany.
pub fn other_variant(variant: ModelVariant) -> ModelVariant {
    match variant {
        ModelVariant::Full => ModelVariant::Q5_0,
        ModelVariant::Q5_0 => ModelVariant::Full,
    }
}

/// Etykieta „Usuń nieużywany wariant …”, gdy drugi wariant leży na dysku (`other_bytes`).
pub fn remove_other_label(info: &ModelInfo, other_bytes: Option<u64>) -> Option<String> {
    let current = info.variant?;
    let bytes = other_bytes?;
    let name = VARIANTS
        .iter()
        .find(|(variant, _, _)| *variant == other_variant(current))
        .map(|(_, _, label)| *label)?;
    Some(format!(
        "Usuń nieużywany wariant: {name} ({})",
        gigabytes(bytes)
    ))
}
pub const REMOVE_MODEL_ID: &str = "model-remove";
pub const CONFIRM_REMOVE_ID: &str = "model-remove-confirm";
pub const CANCEL_REMOVE_ID: &str = "model-remove-cancel";
pub const REDOWNLOAD_MODEL_ID: &str = "model-redownload";

/// Potwierdzenie usunięcia w samym menu (bez okien dialogowych): po „Usuń model…” podmenu
/// pokazuje „Potwierdź usunięcie (1,6 GB)” i „Anuluj”; każde inne kliknięcie zamyka pytanie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum RemovePrompt {
    #[default]
    Idle,
    Confirming,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RemoveAction {
    None,
    ShowPrompt,
    Remove,
    Cancel,
}

impl RemovePrompt {
    /// Przejście po kliknięciu pozycji menu `id`; `remove_allowed` = flaga z [`model_menu`].
    pub fn on_menu_click(self, id: &str, remove_allowed: bool) -> (Self, RemoveAction) {
        match (self, id) {
            (Self::Idle, REMOVE_MODEL_ID) if remove_allowed => {
                (Self::Confirming, RemoveAction::ShowPrompt)
            }
            (Self::Idle, _) => (Self::Idle, RemoveAction::None),
            (Self::Confirming, CONFIRM_REMOVE_ID) => (Self::Idle, RemoveAction::Remove),
            (Self::Confirming, _) => (Self::Idle, RemoveAction::Cancel),
        }
    }
}

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
    /// Wariant z magazynu aplikacji; `None` dla własnej ścieżki (VA-MODEL-4).
    pub variant: Option<va_config::ModelVariant>,
}

/// Co pokazać w podmenu: linie informacyjne (nieklikalne) i aktywność poleceń.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelMenu {
    pub lines: Vec<String>,
    pub remove_enabled: bool,
    pub redownload_enabled: bool,
    /// Etykieta pozycji potwierdzającej usunięcie, z rozmiarem zwalnianego pliku.
    pub confirm_label: String,
    /// Wybór wariantu (VA-MODEL-4); uzupełnia aplikacja, bo zależy od dysku i zadania pobierania.
    pub variants: Vec<VariantItem>,
    /// „Usuń nieużywany wariant …”, gdy drugi wariant leży na dysku.
    pub remove_other: Option<String>,
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
        confirm_label: format!("Potwierdź usunięcie ({})", gigabytes(info.bytes)),
        variants: Vec::new(),
        remove_other: None,
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

/// „1,6 GB” — zaokrąglone do najbliższej dziesiątej (jak Finder), po polsku; 0 bajtów → „0 GB”.
pub fn gigabytes(bytes: u64) -> String {
    if bytes == 0 {
        return "0 GB".to_owned();
    }
    let tenths = (bytes * 10 + 500_000_000) / 1_000_000_000;
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
            variant: Some(va_config::ModelVariant::Full),
        }
    }

    fn custom_model(present: bool) -> ModelInfo {
        ModelInfo {
            name: "moj-model.bin".into(),
            path: "/Users/test/Modele/moj-model.bin".into(),
            bytes: if present { 500_000_000 } else { 0 },
            custom: true,
            custom_present: present,
            variant: None,
        }
    }

    #[test]
    // specky: crit 01M4K6M2ZYSP1Y9M0VD39GV7J8
    fn status_line_shows_name_size_and_state_for_every_download_state() {
        let cases = [
            (DownloadState::NotNeeded, "large-v3-turbo · 1,6 GB · gotowy"),
            (DownloadState::Ready, "large-v3-turbo · 1,6 GB · gotowy"),
            (
                DownloadState::Downloading { percent: 42 },
                "large-v3-turbo · 1,6 GB · pobieranie",
            ),
            (DownloadState::Missing, "large-v3-turbo · 1,6 GB · brak"),
            (
                DownloadState::Failed("x".into()),
                "large-v3-turbo · 1,6 GB · brak (pobieranie nieudane)",
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
    fn size_is_rounded_to_the_nearest_tenth_of_a_gigabyte() {
        assert_eq!(gigabytes(1_624_555_275), "1,6 GB");
        assert_eq!(gigabytes(574_041_195), "0,6 GB");
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
    // specky: crit 01M4KD15M5E90XXM60PPS2S33J
    fn variant_items_show_both_sizes_and_mark_the_one_in_use() {
        let items = variant_items(&store_model(), true);

        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
        assert_eq!(labels, ["Pełny (1,6 GB)", "Skwantyzowany q5_0 (0,6 GB)"]);
        assert!(items[0].checked && !items[1].checked);
        assert!(
            !items[0].enabled,
            "wybranego wariantu nie wybiera się ponownie"
        );
        assert!(items[1].enabled);
        assert_eq!(variant_from_menu_id(&items[1].id), Some(ModelVariant::Q5_0));
        assert_eq!(variant_from_menu_id("model-variant:q8"), None);
    }

    #[test]
    fn variants_cannot_be_switched_while_busy_or_with_custom_model() {
        assert!(
            variant_items(&store_model(), false)
                .iter()
                .all(|item| !item.enabled)
        );
        let custom = variant_items(&custom_model(true), true);
        assert!(custom.iter().all(|item| !item.enabled && !item.checked));
    }

    #[test]
    // specky: crit 01M4KD15M63SWHPZAMN9BYDK0W
    fn unused_variant_on_disk_can_be_removed_separately() {
        assert_eq!(
            remove_other_label(&store_model(), Some(574_041_195)).as_deref(),
            Some("Usuń nieużywany wariant: Skwantyzowany q5_0 (0,6 GB)")
        );
        assert_eq!(remove_other_label(&store_model(), None), None);
        assert_eq!(remove_other_label(&custom_model(true), Some(1)), None);
        assert_eq!(other_variant(ModelVariant::Q5_0), ModelVariant::Full);
    }

    #[test]
    // specky: crit 01M4K6M2ZYTC7D38NMHZJXF1FV
    fn removal_needs_a_confirmation_click_and_any_other_click_cancels() {
        let (prompt, action) = RemovePrompt::Idle.on_menu_click(REMOVE_MODEL_ID, true);
        assert_eq!(
            (prompt, action),
            (RemovePrompt::Confirming, RemoveAction::ShowPrompt)
        );

        assert_eq!(
            prompt.on_menu_click(CONFIRM_REMOVE_ID, true),
            (RemovePrompt::Idle, RemoveAction::Remove)
        );
        assert_eq!(
            prompt.on_menu_click(CANCEL_REMOVE_ID, true),
            (RemovePrompt::Idle, RemoveAction::Cancel)
        );
        assert_eq!(
            prompt.on_menu_click("quit", true),
            (RemovePrompt::Idle, RemoveAction::Cancel)
        );
        assert_eq!(
            RemovePrompt::Idle.on_menu_click(CONFIRM_REMOVE_ID, true),
            (RemovePrompt::Idle, RemoveAction::None),
            "potwierdzenie bez pytania nie usuwa"
        );
        assert_eq!(
            RemovePrompt::Idle.on_menu_click(REMOVE_MODEL_ID, false),
            (RemovePrompt::Idle, RemoveAction::None),
            "nieaktywne Usuń nie otwiera pytania"
        );
        assert_eq!(
            model_menu(&store_model(), &DownloadState::Ready, State::Idle).confirm_label,
            "Potwierdź usunięcie (1,6 GB)"
        );
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
