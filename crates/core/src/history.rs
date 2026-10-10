//! Historia wypowiedzi (VA-HIST-1): ostatnie transkrypcje, trwałe między uruchomieniami.
//!
//! Treść wpisów nigdy nie trafia do logów — logujemy wyłącznie liczby i błędy I/O.
//! Plik historii jest jedynym miejscem, w którym tekst transkrypcji ląduje na dysku
//! (decyzja właściciela 2026-10-10); dostaje uprawnienia tylko dla użytkownika.

use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

/// Tyle najnowszych wpisów zostaje; starsze wypadają.
pub const MAX_ENTRIES: usize = 30;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: u64,
    pub text: String,
    pub at: DateTime<Local>,
}

/// Trwały magazyn wpisów; brak pliku to pusta historia, nie błąd.
pub trait HistoryStore: Send {
    fn load(&mut self) -> Result<Vec<HistoryEntry>, String>;
    fn save(&mut self, entries: &[HistoryEntry]) -> Result<(), String>;
    fn clear(&mut self) -> Result<(), String>;
}

/// Lista ostatnich wypowiedzi, najnowsza pierwsza, zapisywana po każdej zmianie.
pub struct History {
    entries: Vec<HistoryEntry>,
    next_id: u64,
    store: Box<dyn HistoryStore>,
}

impl History {
    pub fn open(mut store: Box<dyn HistoryStore>) -> Self {
        let entries = match store.load() {
            Ok(entries) => entries,
            Err(error) => {
                tracing::warn!(%error, "historia wypowiedzi nieczytelna — zaczynam od pustej");
                Vec::new()
            }
        };
        let next_id = entries.iter().map(|entry| entry.id + 1).max().unwrap_or(1);
        tracing::info!(entries = entries.len(), "historia wypowiedzi wczytana");
        Self {
            entries,
            next_id,
            store,
        }
    }

    pub fn entries(&self) -> &[HistoryEntry] {
        &self.entries
    }

    pub fn find(&self, id: u64) -> Option<&HistoryEntry> {
        self.entries.iter().find(|entry| entry.id == id)
    }

    /// Dopisuje wpis na początek listy; sam biały znak nie tworzy wpisu. `true` = dodano.
    pub fn push(&mut self, text: &str, at: DateTime<Local>) -> bool {
        if text.trim().is_empty() {
            return false;
        }
        let entry = HistoryEntry {
            id: self.next_id,
            text: text.to_owned(),
            at,
        };
        self.next_id += 1;
        self.entries.insert(0, entry);
        self.entries.truncate(MAX_ENTRIES);
        self.persist();
        true
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        if let Err(error) = self.store.clear() {
            tracing::error!(%error, "czyszczenie historii wypowiedzi");
        }
        tracing::info!("historia wypowiedzi wyczyszczona");
    }

    fn persist(&mut self) {
        if let Err(error) = self.store.save(&self.entries) {
            tracing::error!(%error, "zapis historii wypowiedzi");
        }
        tracing::info!(entries = self.entries.len(), "historia wypowiedzi zapisana");
    }
}

/// Plik JSON dostępny tylko dla użytkownika (0600), zapis atomowy przez plik tymczasowy.
pub struct FileHistoryStore {
    path: PathBuf,
}

impl FileHistoryStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }
}

impl HistoryStore for FileHistoryStore {
    fn load(&mut self) -> Result<Vec<HistoryEntry>, String> {
        match fs::read(&self.path) {
            Ok(bytes) => serde_json::from_slice(&bytes).map_err(|error| error.to_string()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(error) => Err(error.to_string()),
        }
    }

    fn save(&mut self, entries: &[HistoryEntry]) -> Result<(), String> {
        let json = serde_json::to_vec_pretty(entries).map_err(|error| error.to_string())?;
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).map_err(io_text)?;
        }
        let temporary = self.path.with_extension("json.tmp");
        fs::write(&temporary, json).map_err(io_text)?;
        fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600)).map_err(io_text)?;
        fs::rename(&temporary, &self.path).map_err(io_text)
    }

    fn clear(&mut self) -> Result<(), String> {
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.to_string()),
        }
    }
}

fn io_text(error: io::Error) -> String {
    error.to_string()
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::path::Path;
    use std::sync::{Arc, Mutex};

    use super::*;

    fn file_history(path: &Path) -> History {
        History::open(Box::new(FileHistoryStore::new(path)))
    }

    fn now() -> DateTime<Local> {
        Local::now()
    }

    #[test]
    // specky: crit 01M4K06AH0CZS3Z7N0FXXSKE1T
    fn newest_entry_comes_first_and_only_thirty_are_kept() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = file_history(&dir.path().join("history.json"));

        for number in 1..=35 {
            assert!(history.push(&format!("wpis {number}"), now()));
        }

        let entries = history.entries();
        assert_eq!(entries.len(), 30, "limit z VA-HIST-1 to 30 wpisów");
        assert_eq!(entries[0].text, "wpis 35");
        assert_eq!(entries[29].text, "wpis 6");
        assert!(entries.windows(2).all(|pair| pair[0].id > pair[1].id));
    }

    #[test]
    // specky: crit 01M4K06AH032985T6JXJX7EE3W
    fn blank_text_does_not_create_an_entry() {
        let dir = tempfile::tempdir().unwrap();
        let mut history = file_history(&dir.path().join("history.json"));

        assert!(!history.push("   \n", now()));
        assert!(history.push("zdanie", now()));

        assert_eq!(history.entries().len(), 1);
    }

    #[test]
    // specky: crit 01M4K06AH0FMZJZZX7SHA9KR5A
    fn entries_survive_reopening_and_file_is_private() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nested/history.json");
        let mut history = file_history(&path);
        history.push("pierwsze", now());
        history.push("drugie", now());
        let saved = history.entries().to_vec();
        drop(history);

        let mut reopened = file_history(&path);

        assert_eq!(reopened.entries(), &saved[..]);
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "tryb {mode:o}");
        assert!(reopened.push("trzecie", now()));
        assert_eq!(reopened.entries()[0].id, saved[0].id + 1);
        assert!(!path.with_extension("json.tmp").exists());
    }

    #[test]
    // specky: crit 01M4K06AH0BR7ZQYXPH6B08P26
    fn clear_empties_the_list_and_removes_the_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        let mut history = file_history(&path);
        history.push("do usunięcia", now());
        assert!(path.exists());

        history.clear();

        assert!(history.entries().is_empty());
        assert!(!path.exists());
        assert!(file_history(&path).entries().is_empty());
    }

    #[test]
    fn unreadable_file_starts_an_empty_history() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");
        fs::write(&path, b"to nie jest json").unwrap();

        let history = file_history(&path);

        assert!(history.entries().is_empty());
    }

    #[derive(Clone, Default)]
    struct CapturedLogs(Arc<Mutex<Vec<u8>>>);

    impl Write for CapturedLogs {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    // specky: crit 01M4K06AH0B2HW26D29WR4H4F3
    fn entry_text_never_appears_in_logs() {
        let logs = CapturedLogs::default();
        let writer = logs.clone();
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::TRACE)
            .with_ansi(false)
            .with_writer(move || writer.clone())
            .finish();
        let _guard = tracing::subscriber::set_default(subscriber);
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("history.json");

        let mut history = file_history(&path);
        history.push("sekretna wypowiedź użytkownika", now());
        history.clear();
        drop(file_history(&path));

        let text = String::from_utf8_lossy(&logs.0.lock().unwrap()).into_owned();
        assert!(text.contains("historia wypowiedzi"), "logi puste: {text}");
        assert!(!text.contains("sekretna"), "treść w logach: {text}");
    }
}
