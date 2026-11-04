//! Plik tymczasowy długiego nagrania (VA-CONF-3): ścieżka spotkania trafia na dysk na bieżąco
//! zamiast do pamięci. Plik ma prawa tylko dla użytkownika, znika po odczycie albo porzuceniu,
//! a pozostałości po awarii usuwa [`remove_leftovers`] przy starcie aplikacji.

use std::fs::{self, File, OpenOptions};
use std::io::{self, BufWriter, Read, Write};
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const PREFIX: &str = "meeting-";
const EXTENSION: &str = "pcm";
const SAMPLE_BYTES: usize = 4;

static NEXT_ID: AtomicU64 = AtomicU64::new(0);

/// Otwarty plik nagrania: próbki 16 kHz mono f32 (little endian) dopisywane na bieżąco.
pub struct SpoolWriter {
    path: PathBuf,
    file: Option<BufWriter<File>>,
    samples: u64,
}

impl SpoolWriter {
    /// Tworzy nowy plik `meeting-<pid>-<n>-<track>.pcm` w `dir` z prawami 0600.
    pub fn create(dir: &Path, track: &str) -> io::Result<Self> {
        fs::create_dir_all(dir)?;
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let name = format!("{PREFIX}{}-{id}-{track}.{EXTENSION}", std::process::id());
        let path = dir.join(name);
        let file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        Ok(Self {
            path,
            file: Some(BufWriter::new(file)),
            samples: 0,
        })
    }

    pub fn append(&mut self, samples: &[f32]) -> io::Result<()> {
        let file = self.file.as_mut().ok_or_else(closed)?;
        for sample in samples {
            file.write_all(&sample.to_le_bytes())?;
        }
        self.samples += samples.len() as u64;
        Ok(())
    }

    /// Liczba zapisanych próbek.
    pub fn samples(&self) -> u64 {
        self.samples
    }

    /// Domyka zapis; plik zostaje do odczytu przez [`SpooledAudio`].
    pub fn finish(mut self) -> io::Result<SpooledAudio> {
        let file = self.file.take().ok_or_else(closed)?;
        file.into_inner().map_err(io::IntoInnerError::into_error)?;
        Ok(SpooledAudio {
            path: std::mem::take(&mut self.path),
        })
    }
}

impl Drop for SpoolWriter {
    fn drop(&mut self) {
        if self.file.is_some() {
            remove_quietly(&self.path);
        }
    }
}

/// Zamknięty plik nagrania; usuwany przy zniszczeniu, także gdy odczyt albo transkrypcja
/// zakończy się błędem.
pub struct SpooledAudio {
    path: PathBuf,
}

impl SpooledAudio {
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Wczytuje próbki do transkrypcji.
    pub fn read(&self) -> io::Result<Vec<f32>> {
        let mut bytes = Vec::new();
        File::open(&self.path)?.read_to_end(&mut bytes)?;
        Ok(bytes
            .chunks_exact(SAMPLE_BYTES)
            .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
            .collect())
    }
}

impl Drop for SpooledAudio {
    fn drop(&mut self) {
        remove_quietly(&self.path);
    }
}

/// Usuwa pozostawione po awarii pliki `meeting-*.pcm`; inne pliki katalogu zostają.
pub fn remove_leftovers(dir: &Path) -> io::Result<usize> {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error),
    };
    let mut removed = 0;
    for entry in entries {
        let path = entry?.path();
        if is_spool_file(&path) {
            fs::remove_file(&path)?;
            removed += 1;
        }
    }
    if removed > 0 {
        tracing::info!(removed, "usunięte pozostałości nagrań spotkań");
    }
    Ok(removed)
}

fn is_spool_file(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    name.starts_with(PREFIX) && path.extension().is_some_and(|ext| ext == EXTENSION)
}

fn remove_quietly(path: &Path) {
    if let Err(error) = fs::remove_file(path)
        && error.kind() != io::ErrorKind::NotFound
    {
        tracing::warn!(%error, "nie udało się usunąć pliku nagrania spotkania");
    }
}

fn closed() -> io::Error {
    io::Error::other("plik nagrania jest już zamknięty")
}

#[cfg(test)]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use va_audio::StreamNormalizer;

    use super::*;

    fn stereo_sine_48k(seconds: f32) -> Vec<f32> {
        let frames = (48_000.0 * seconds) as usize;
        (0..frames)
            .flat_map(|index| {
                let value = (std::f32::consts::TAU * 440.0 * index as f32 / 48_000.0).sin() * 0.5;
                [value, value]
            })
            .collect()
    }

    #[test]
    // specky: crit 01M6JR8RXS734Y55256W14CWPJ
    fn stereo_48k_chunks_are_stored_as_16k_mono_readable_only_by_user() {
        let dir = tempfile::tempdir().unwrap();
        let mut normalizer = StreamNormalizer::new(2, 48_000).unwrap();
        let mut writer = SpoolWriter::create(dir.path(), "mic").unwrap();
        for chunk in stereo_sine_48k(2.0).chunks(960) {
            writer.append(&normalizer.push(chunk).unwrap()).unwrap();
        }
        writer.append(&normalizer.finish().unwrap()).unwrap();

        let spooled = writer.finish().unwrap();
        let mode = fs::metadata(spooled.path()).unwrap().permissions().mode();
        let samples = spooled.read().unwrap();

        assert_eq!(mode & 0o777, 0o600);
        assert!(spooled.path().starts_with(dir.path()));
        assert_eq!(samples.len(), 32_000);
        let peak = samples[1_000..31_000]
            .iter()
            .fold(0.0f32, |a, b| a.max(b.abs()));
        assert!((peak - 0.5).abs() < 0.05, "amplituda {peak}");
    }

    #[test]
    // specky: crit 01M6JR8RXSJBV4P9TSK2Y03F2Z
    fn spooled_file_is_removed_after_reading_and_dropping() {
        let dir = tempfile::tempdir().unwrap();
        let mut writer = SpoolWriter::create(dir.path(), "system").unwrap();
        writer.append(&[0.25, -0.5]).unwrap();
        let spooled = writer.finish().unwrap();
        let path = spooled.path().to_owned();

        assert_eq!(spooled.read().unwrap(), vec![0.25, -0.5]);
        drop(spooled);

        assert!(!path.exists());
    }

    #[test]
    fn abandoned_writer_removes_its_file() {
        let dir = tempfile::tempdir().unwrap();
        let writer = SpoolWriter::create(dir.path(), "mic").unwrap();

        drop(writer);

        assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
    }

    #[test]
    // specky: crit 01M6JR8RXSJBV4P9TSK2Y03F2Z
    fn leftovers_cleanup_removes_only_meeting_recordings() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("meeting-123-0-mic.pcm"), b"x").unwrap();
        fs::write(dir.path().join("meeting-123-1-system.pcm"), b"x").unwrap();
        fs::write(dir.path().join("history.json"), b"[]").unwrap();
        fs::write(dir.path().join("meeting-notes.txt"), b"x").unwrap();

        let removed = remove_leftovers(dir.path()).unwrap();

        assert_eq!(removed, 2);
        let mut left: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .map(|entry| entry.unwrap().file_name().into_string().unwrap())
            .collect();
        left.sort();
        assert_eq!(left, ["history.json", "meeting-notes.txt"]);
    }

    #[test]
    fn missing_directory_has_no_leftovers() {
        let dir = tempfile::tempdir().unwrap();

        assert_eq!(remove_leftovers(&dir.path().join("brak")).unwrap(), 0);
    }
}
