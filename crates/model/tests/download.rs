//! Pobieranie modelu z lokalnego serwera HTTP: pełne, wznowione, z błędną sumą, zbędne.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use sha2::{Digest, Sha256};
use va_model::{Error, ModelSpec, ModelState, ModelStore, Progress};

const MODEL_BYTES: usize = 3 * 1024 * 1024 + 123;

/// Serwer pliku z obsługą `Range: bytes=N-`; pierwszą odpowiedź może urwać po `cut_after` bajtach.
struct FileServer {
    url: String,
    ranges: Arc<Mutex<Vec<Option<String>>>>,
}

impl FileServer {
    fn start(content: Vec<u8>, cut_after: Option<usize>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}/model.bin", listener.local_addr().unwrap());
        let ranges = Arc::new(Mutex::new(Vec::new()));
        let seen = Arc::clone(&ranges);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let first = seen.lock().unwrap().is_empty();
                let range = serve(stream.unwrap(), &content, cut_after.filter(|_| first));
                seen.lock().unwrap().push(range);
            }
        });
        Self { url, ranges }
    }

    fn ranges(&self) -> Vec<Option<String>> {
        self.ranges.lock().unwrap().clone()
    }
}

fn serve(mut stream: TcpStream, content: &[u8], cut_after: Option<usize>) -> Option<String> {
    let mut range = None;
    for line in BufReader::new(stream.try_clone().unwrap()).lines() {
        let line = line.unwrap();
        if line.is_empty() {
            break;
        }
        if let Some(value) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
            range = Some(value.trim_end_matches('-').to_owned());
        }
    }
    let offset: usize = range.as_deref().map_or(0, |value| value.parse().unwrap());
    let body = &content[offset..];
    let status = if offset > 0 {
        "206 Partial Content"
    } else {
        "200 OK"
    };
    let headers = format!(
        "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    );
    stream.write_all(headers.as_bytes()).unwrap();
    let sent = cut_after.map_or(body.len(), |limit| limit.min(body.len()));
    let _ = stream.write_all(&body[..sent]);
    range
}

fn model_content() -> Vec<u8> {
    (0..MODEL_BYTES)
        .map(|index| (index * 31 % 251) as u8)
        .collect()
}

fn spec_for(content: &[u8]) -> ModelSpec {
    let digest = Sha256::digest(content);
    let sha256: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    ModelSpec {
        file_name: "ggml-test.bin",
        url: "http://niewykorzystany",
        sha256: Box::leak(sha256.into_boxed_str()),
        size: content.len() as u64,
    }
}

fn no_progress(_: Progress) {}

#[test]
fn downloads_model_verifies_checksum_and_reports_progress() {
    let content = model_content();
    let server = FileServer::start(content.clone(), None);
    let dir = tempfile::tempdir().unwrap();
    let store = ModelStore::new(dir.path(), spec_for(&content));
    let mut last = None;

    let path = store
        .download(&server.url, &mut |progress| last = Some(progress))
        .unwrap();

    assert_eq!(std::fs::read(&path).unwrap(), content);
    assert_eq!(last.unwrap().percent(), 100);
    assert_eq!(store.check().unwrap(), ModelState::Ready(path));
}

#[test]
// specky: crit 01M4EQH9E1WNK7YQA216CGDQX8
fn interrupted_download_resumes_from_where_it_stopped() {
    let content = model_content();
    let half = MODEL_BYTES / 2;
    let server = FileServer::start(content.clone(), Some(half));
    let dir = tempfile::tempdir().unwrap();
    let store = ModelStore::new(dir.path(), spec_for(&content));

    let first = store.download(&server.url, &mut no_progress);

    assert!(matches!(first, Err(Error::Incomplete { .. })), "{first:?}");
    assert_eq!(store.check().unwrap(), ModelState::Partial(half as u64));
    let path = store.download(&server.url, &mut no_progress).unwrap();
    assert_eq!(std::fs::read(path).unwrap(), content);
    assert_eq!(
        server.ranges(),
        vec![Some("0".into()), Some(half.to_string())]
    );
}

#[test]
// specky: crit 01M4EQH9E1KX4BSAYNQK95VWD8
fn corrupted_download_is_deleted_and_can_be_retried() {
    let content = model_content();
    let mut corrupted = content.clone();
    corrupted[1000] ^= 0xff;
    let bad_server = FileServer::start(corrupted, None);
    let dir = tempfile::tempdir().unwrap();
    let store = ModelStore::new(dir.path(), spec_for(&content));

    let result = store.download(&bad_server.url, &mut no_progress);

    assert!(matches!(result, Err(Error::ChecksumMismatch)), "{result:?}");
    assert_eq!(store.check().unwrap(), ModelState::Missing);
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 0);
    let good_server = FileServer::start(content.clone(), None);
    let path = store.download(&good_server.url, &mut no_progress).unwrap();
    assert_eq!(std::fs::read(path).unwrap(), content);
}

#[test]
fn valid_model_on_disk_is_not_downloaded_again() {
    let content = model_content();
    let server = FileServer::start(content.clone(), None);
    let dir = tempfile::tempdir().unwrap();
    let store = ModelStore::new(dir.path(), spec_for(&content));
    std::fs::write(store.model_path(), &content).unwrap();

    store.download(&server.url, &mut no_progress).unwrap();

    assert!(
        server.ranges().is_empty(),
        "niepotrzebne żądanie do serwera"
    );
}

#[test]
fn model_with_wrong_checksum_on_disk_is_removed_at_check() {
    let content = model_content();
    let dir = tempfile::tempdir().unwrap();
    let store = ModelStore::new(dir.path(), spec_for(&content));
    std::fs::write(store.model_path(), b"to nie jest model").unwrap();

    assert_eq!(store.check().unwrap(), ModelState::Missing);
    assert!(!store.model_path().exists());
}

#[test]
fn server_error_status_is_reported() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}/model.bin", listener.local_addr().unwrap());
    std::thread::spawn(move || {
        let mut stream = listener.incoming().next().unwrap().unwrap();
        let mut request = String::new();
        BufReader::new(stream.try_clone().unwrap())
            .read_line(&mut request)
            .unwrap();
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    });
    let dir = tempfile::tempdir().unwrap();
    let store = ModelStore::new(dir.path(), spec_for(&model_content()));

    let result = store.download(&url, &mut no_progress);

    assert!(matches!(result, Err(Error::Status(404))), "{result:?}");
}
