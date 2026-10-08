//! Pobieranie i weryfikacja modelu Whisper large-v3-turbo.
//!
//! Model nie jest częścią instalatora: pobiera się go raz do katalogu danych aplikacji,
//! najpierw do pliku `.part` (wznawianego żądaniem HTTP Range), a po zgodności sumy
//! SHA-256 plik dostaje nazwę docelową. Po pobraniu nic tu nie łączy się z siecią.

use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};

const CHUNK_BYTES: usize = 1 << 20;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(60);

/// Opis pliku modelu: skąd go wziąć i jak sprawdzić, że jest cały.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModelSpec {
    pub file_name: &'static str,
    pub url: &'static str,
    pub sha256: &'static str,
    pub size: u64,
}

pub const LARGE_V3_TURBO: ModelSpec = ModelSpec {
    file_name: "ggml-large-v3-turbo.bin",
    url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-large-v3-turbo.bin",
    sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
    size: 1_624_555_275,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pobieranie modelu nie powiodło się: {0}")]
    Http(String),
    #[error("serwer modelu odpowiedział kodem {0}")]
    Status(u16),
    #[error("pobieranie modelu przerwane po {received} z {expected} bajtów — ponów, aby wznowić")]
    Incomplete { received: u64, expected: u64 },
    #[error("plik modelu jest uszkodzony (zła suma SHA-256) — usunięty, pobierz ponownie")]
    ChecksumMismatch,
    #[error("błąd zapisu modelu {path}: {source}")]
    Io { path: PathBuf, source: io::Error },
}

pub type Result<T> = std::result::Result<T, Error>;

/// Postęp pobierania w bajtach.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Progress {
    pub downloaded: u64,
    pub total: u64,
}

impl Progress {
    pub fn percent(&self) -> u8 {
        if self.total == 0 {
            return 0;
        }
        (self.downloaded.min(self.total) * 100 / self.total) as u8
    }
}

/// Stan modelu na dysku.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModelState {
    Ready(PathBuf),
    /// Częściowo pobrany: tyle bajtów czeka w pliku `.part`.
    Partial(u64),
    Missing,
}

/// Plik modelu w katalogu `models/` aplikacji.
pub struct ModelStore {
    dir: PathBuf,
    spec: ModelSpec,
}

impl ModelStore {
    pub fn new(dir: impl Into<PathBuf>, spec: ModelSpec) -> Self {
        Self {
            dir: dir.into(),
            spec,
        }
    }

    pub fn model_path(&self) -> PathBuf {
        self.dir.join(self.spec.file_name)
    }

    fn part_path(&self) -> PathBuf {
        self.dir.join(format!("{}.part", self.spec.file_name))
    }

    /// Sprawdza obecność i sumę modelu; uszkodzony plik docelowy jest usuwany.
    pub fn check(&self) -> Result<ModelState> {
        let model = self.model_path();
        if model.exists() {
            if self.checksum_matches(&model)? {
                return Ok(ModelState::Ready(model));
            }
            tracing::warn!(path = %model.display(), "model z błędną sumą SHA-256 — usuwam");
            remove(&model)?;
        }
        match fs::metadata(self.part_path()) {
            Ok(meta) => Ok(ModelState::Partial(meta.len())),
            Err(_) => Ok(ModelState::Missing),
        }
    }

    /// Pobiera model z `url` (wznawiając `.part`), chyba że poprawny już jest na dysku.
    pub fn download(&self, url: &str, progress: &mut dyn FnMut(Progress)) -> Result<PathBuf> {
        if let ModelState::Ready(path) = self.check()? {
            return Ok(path);
        }
        fs::create_dir_all(&self.dir).map_err(|source| self.io(&self.dir, source))?;
        let part = self.part_path();
        self.fetch_into(url, &part, progress)?;
        if !self.checksum_matches(&part)? {
            remove(&part)?;
            return Err(Error::ChecksumMismatch);
        }
        let model = self.model_path();
        fs::rename(&part, &model).map_err(|source| self.io(&model, source))?;
        tracing::info!(path = %model.display(), "model pobrany i zweryfikowany");
        Ok(model)
    }

    fn fetch_into(&self, url: &str, part: &Path, progress: &mut dyn FnMut(Progress)) -> Result<()> {
        let offset = fs::metadata(part).map_or(0, |meta| meta.len());
        if offset >= self.spec.size {
            return Ok(());
        }
        let response = agent()
            .get(url)
            .header("Range", format!("bytes={offset}-"))
            .call()
            .map_err(|error| Error::Http(error.to_string()))?;
        let resumed = match response.status().as_u16() {
            206 => true,
            200 => false,
            code => return Err(Error::Status(code)),
        };
        tracing::info!(offset, resumed, "pobieranie modelu");
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .append(resumed)
            .truncate(!resumed)
            .open(part)
            .map_err(|source| self.io(part, source))?;
        let start = if resumed { offset } else { 0 };
        let mut body = response.into_body().into_reader();
        let received = self.copy_with_progress(&mut body, &mut file, part, start, progress)?;
        if received < self.spec.size {
            return Err(Error::Incomplete {
                received,
                expected: self.spec.size,
            });
        }
        Ok(())
    }

    fn copy_with_progress(
        &self,
        body: &mut dyn Read,
        file: &mut File,
        part: &Path,
        start: u64,
        progress: &mut dyn FnMut(Progress),
    ) -> Result<u64> {
        let mut buffer = vec![0; CHUNK_BYTES];
        let mut downloaded = start;
        loop {
            let read = match body.read(&mut buffer) {
                Ok(0) => break,
                Ok(read) => read,
                Err(error) => {
                    tracing::warn!(%error, downloaded, "połączenie przerwane");
                    break;
                }
            };
            file.write_all(&buffer[..read])
                .map_err(|source| self.io(part, source))?;
            downloaded += read as u64;
            progress(Progress {
                downloaded,
                total: self.spec.size,
            });
        }
        file.flush().map_err(|source| self.io(part, source))?;
        Ok(downloaded)
    }

    fn checksum_matches(&self, path: &Path) -> Result<bool> {
        let mut file = File::open(path).map_err(|source| self.io(path, source))?;
        let mut hasher = Sha256::new();
        io::copy(&mut file, &mut hasher).map_err(|source| self.io(path, source))?;
        Ok(hex(&hasher.finalize()) == self.spec.sha256)
    }

    fn io(&self, path: &Path, source: io::Error) -> Error {
        Error::Io {
            path: path.to_owned(),
            source,
        }
    }
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .http_status_as_error(false)
        .timeout_connect(Some(CONNECT_TIMEOUT))
        .timeout_recv_response(Some(RESPONSE_TIMEOUT))
        .build()
        .into()
}

fn remove(path: &Path) -> Result<()> {
    fs::remove_file(path).map_err(|source| Error::Io {
        path: path.to_owned(),
        source,
    })
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
