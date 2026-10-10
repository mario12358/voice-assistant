//! Logowanie aplikacji: JSON do pliku w `~/Library/Logs/VoiceAsystent/`, tekst na stderr.
//!
//! Treść transkrypcji jest danymi użytkownika — trafia do logu wyłącznie na poziomie `debug`.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{EnvFilter, fmt};

pub const LOG_FILE_PREFIX: &str = "voice-asystent.log";
/// Tyle trzymamy dzienne pliki logów (VA-OPS-1); starsze znikają przy starcie.
pub const LOG_RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

/// Usuwa pliki `voice-asystent.log.*` starsze niż `max_age` (po dacie modyfikacji); inne pliki
/// w katalogu zostają. Zwraca liczbę usuniętych; brak katalogu to 0.
pub fn prune_old_logs(logs_dir: &Path, max_age: Duration) -> usize {
    let Ok(entries) = fs::read_dir(logs_dir) else {
        return 0;
    };
    let now = SystemTime::now();
    let mut removed = 0;
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        let is_log = name
            .to_string_lossy()
            .starts_with(&format!("{LOG_FILE_PREFIX}."));
        if !is_log || !path.is_file() {
            continue;
        }
        let Ok(modified) = entry.metadata().and_then(|meta| meta.modified()) else {
            continue;
        };
        let expired = now.duration_since(modified).is_ok_and(|age| age > max_age);
        if !expired {
            continue;
        }
        match fs::remove_file(&path) {
            Ok(()) => removed += 1,
            Err(error) => tracing::warn!(%error, path = %path.display(), "stary log nie usunięty"),
        }
    }
    tracing::info!(removed, "stare logi usunięte");
    removed
}

#[derive(Debug, Clone, Default)]
pub struct LogOptions {
    /// Katalog dziennego pliku logu; `None` = tylko stderr.
    pub logs_dir: Option<PathBuf>,
    /// Liczba flag `-v`: 0 = info, 1 = debug, 2+ = trace. `RUST_LOG` ma pierwszeństwo.
    pub verbosity: u8,
}

/// Instaluje globalny subscriber. Zwrócony guard musi żyć do końca programu,
/// inaczej końcówka logu w pliku przepadnie.
pub fn init(options: &LogOptions) -> Option<WorkerGuard> {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(default_level(options.verbosity)));
    let stderr_layer = fmt::layer()
        .with_writer(std::io::stderr)
        .with_span_events(FmtSpan::CLOSE);

    let Some(logs_dir) = &options.logs_dir else {
        tracing_subscriber::registry()
            .with(filter)
            .with(stderr_layer)
            .init();
        return None;
    };
    let (file_writer, guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(logs_dir, LOG_FILE_PREFIX));
    let file_layer = fmt::layer()
        .json()
        .with_writer(file_writer)
        .with_span_events(FmtSpan::CLOSE);
    tracing_subscriber::registry()
        .with(filter)
        .with(stderr_layer)
        .with(file_layer)
        .init();
    Some(guard)
}

fn default_level(verbosity: u8) -> &'static str {
    match verbosity {
        0 => "info",
        1 => "debug",
        _ => "trace",
    }
}

/// Zdarzenie zakończonej transkrypcji: metryki na `info`, treść tylko na `debug`.
pub fn transcription_finished(text: &str, inference: Duration) {
    tracing::info!(
        chars = text.chars().count(),
        inference_ms = inference.as_millis() as u64,
        "transkrypcja zakończona"
    );
    tracing::debug!(text, "treść transkrypcji");
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use std::sync::{Arc, Mutex};

    use tracing_subscriber::fmt::MakeWriter;

    use super::*;

    #[derive(Clone, Default)]
    struct CapturedLog(Arc<Mutex<Vec<u8>>>);

    impl CapturedLog {
        fn text(&self) -> String {
            String::from_utf8(self.0.lock().unwrap().clone()).unwrap()
        }
    }

    impl Write for CapturedLog {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    impl<'a> MakeWriter<'a> for CapturedLog {
        type Writer = Self;

        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    fn capture_at(level: &str, action: impl FnOnce()) -> String {
        let log = CapturedLog::default();
        let subscriber = tracing_subscriber::registry()
            .with(EnvFilter::new(level))
            .with(fmt::layer().json().with_writer(log.clone()));
        tracing::subscriber::with_default(subscriber, action);
        log.text()
    }

    const SECRET_TEXT: &str = "Spotkanie z zarządem przenosimy na piątek";

    #[test]
    fn info_level_logs_metrics_without_transcript_text() {
        let log = capture_at("info", || {
            transcription_finished(SECRET_TEXT, Duration::from_millis(420));
        });

        assert!(log.contains("transkrypcja zakończona"), "{log}");
        assert!(log.contains("\"inference_ms\":420"), "{log}");
        assert!(log.contains("\"chars\":41"), "{log}");
        assert!(!log.contains("zarządem"), "{log}");
    }

    #[test]
    fn debug_level_includes_transcript_text() {
        let log = capture_at("debug", || {
            transcription_finished(SECRET_TEXT, Duration::from_millis(420));
        });

        assert!(log.contains(SECRET_TEXT), "{log}");
    }

    fn aged_file(dir: &Path, name: &str, age: Duration) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, b"log").unwrap();
        let file = fs::File::options().write(true).open(&path).unwrap();
        file.set_modified(SystemTime::now() - age).unwrap();
        path
    }

    #[test]
    // specky: crit 01M4KD15FXMVKSVBYXV6QS4ED6
    fn logs_older_than_retention_are_removed_and_counted() {
        let dir = tempfile::tempdir().unwrap();
        let days = |n: u64| Duration::from_secs(n * 24 * 60 * 60);
        let old = aged_file(dir.path(), "voice-asystent.log.2026-10-01", days(8));
        let fresh = aged_file(dir.path(), "voice-asystent.log.2026-10-09", days(6));

        let removed = prune_old_logs(dir.path(), LOG_RETENTION);

        assert_eq!(removed, 1);
        assert!(!old.exists());
        assert!(fresh.exists());
    }

    #[test]
    // specky: crit 01M4KD15FXMYC7NHSDBX6BGCTJ
    fn foreign_files_and_directories_are_left_alone() {
        let dir = tempfile::tempdir().unwrap();
        let days = |n: u64| Duration::from_secs(n * 24 * 60 * 60);
        let foreign = aged_file(dir.path(), "notatki.txt", days(30));
        let other_app = aged_file(dir.path(), "inna-aplikacja.log.2026-01-01", days(30));
        fs::create_dir(dir.path().join("voice-asystent.log.katalog")).unwrap();

        assert_eq!(prune_old_logs(dir.path(), LOG_RETENTION), 0);
        assert!(foreign.exists() && other_app.exists());
        assert_eq!(prune_old_logs(&dir.path().join("nie-ma"), LOG_RETENTION), 0);
    }

    #[test]
    fn verbosity_flags_map_to_levels() {
        assert_eq!(default_level(0), "info");
        assert_eq!(default_level(1), "debug");
        assert_eq!(default_level(5), "trace");
    }
}
