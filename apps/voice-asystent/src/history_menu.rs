//! Podmenu „Historia” (VA-HIST-1): ostatnie wypowiedzi od najnowszej, kliknięcie kopiuje
//! pełny tekst do schowka przez kontroler, „Wyczyść historię” kasuje listę i plik.

use va_core::controller::Command;
use va_core::history::HistoryEntry;

const ENTRY_ID_PREFIX: &str = "history:";
pub const CLEAR_HISTORY_ID: &str = "history-clear";
/// Tyle znaków tekstu pokazuje pozycja menu; dłuższy tekst dostaje wielokropek.
pub const PREVIEW_CHARS: usize = 40;

/// Pozycja podmenu: identyfikator menu i etykieta `HH:MM · początek tekstu…`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HistoryItem {
    pub id: String,
    pub label: String,
}

/// Pozycje w kolejności wpisów (kontroler podaje najnowszy pierwszy).
pub fn history_items(entries: &[HistoryEntry]) -> Vec<HistoryItem> {
    entries
        .iter()
        .map(|entry| HistoryItem {
            id: format!("{ENTRY_ID_PREFIX}{}", entry.id),
            label: format!("{} · {}", entry.at.format("%H:%M"), preview(&entry.text)),
        })
        .collect()
}

/// Polecenie dla kontrolera z identyfikatora klikniętej pozycji; inne pozycje dają `None`.
pub fn command_from_menu_id(id: &str) -> Option<Command> {
    if id == CLEAR_HISTORY_ID {
        return Some(Command::ClearHistory);
    }
    id.strip_prefix(ENTRY_ID_PREFIX)
        .and_then(|number| number.parse().ok())
        .map(Command::CopyHistoryEntry)
}

/// Początek tekstu w jednej linii (białe znaki zwinięte), najwyżej [`PREVIEW_CHARS`] znaków.
fn preview(text: &str) -> String {
    let single_line = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let mut preview: String = single_line.chars().take(PREVIEW_CHARS).collect();
    if single_line.chars().count() > PREVIEW_CHARS {
        preview = preview.trim_end().to_owned();
        preview.push('…');
    }
    preview
}

#[cfg(test)]
mod tests {
    use chrono::{Local, TimeZone};

    use super::*;

    fn entry(id: u64, hour: u32, minute: u32, text: &str) -> HistoryEntry {
        HistoryEntry {
            id,
            text: text.to_owned(),
            at: Local
                .with_ymd_and_hms(2026, 10, 10, hour, minute, 0)
                .unwrap(),
        }
    }

    #[test]
    // specky: crit 01M4K06AH0CZS3Z7N0FXXSKE1T
    fn items_show_time_and_text_start_in_controller_order() {
        let entries = vec![
            entry(7, 14, 5, "Drugie, najnowsze zdanie."),
            entry(3, 9, 30, "Pierwsze zdanie."),
        ];

        let items = history_items(&entries);

        let labels: Vec<_> = items.iter().map(|item| item.label.as_str()).collect();
        assert_eq!(
            labels,
            [
                "14:05 · Drugie, najnowsze zdanie.",
                "09:30 · Pierwsze zdanie."
            ]
        );
        assert_eq!(items[0].id, "history:7");
        assert!(history_items(&[]).is_empty());
    }

    #[test]
    fn long_text_is_cut_to_preview_with_ellipsis_and_newlines_collapsed() {
        let text =
            "Pierwsza linia tekstu,\n  która jest   naprawdę długa i nie zmieści się w menu.";

        let items = history_items(&[entry(1, 8, 0, text)]);

        assert_eq!(
            items[0].label,
            "08:00 · Pierwsza linia tekstu, która jest napraw…"
        );
        assert!(items[0].label.chars().count() <= 8 + PREVIEW_CHARS + 1);
    }

    #[test]
    // specky: crit 01M4K06AH09HZHH764X48P4GM4
    fn clicking_an_entry_maps_to_copy_and_clear_to_clear() {
        let items = history_items(&[entry(42, 8, 0, "tekst")]);

        assert_eq!(
            command_from_menu_id(&items[0].id),
            Some(Command::CopyHistoryEntry(42))
        );
        assert_eq!(
            command_from_menu_id(CLEAR_HISTORY_ID),
            Some(Command::ClearHistory)
        );
        assert_eq!(command_from_menu_id("quit"), None);
        assert_eq!(command_from_menu_id("history:abc"), None);
    }
}
