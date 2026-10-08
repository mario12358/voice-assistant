use std::process::Command;

fn va_dev() -> Command {
    Command::new(env!("CARGO_BIN_EXE_va-dev"))
}

// specky: crit 01M4EKHC6ETDFR6N95SHE3Y9QZ
#[test]
fn version_flag_prints_package_version() {
    let output = va_dev()
        .arg("--version")
        .output()
        .expect("va-dev uruchamia się");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).expect("wyjście w UTF-8");
    assert_eq!(
        stdout.trim(),
        format!("va-dev {}", env!("CARGO_PKG_VERSION"))
    );
}

#[test]
fn unknown_argument_fails_with_nonzero_exit() {
    let output = va_dev()
        .arg("--nie-ma-takiej-flagi")
        .output()
        .expect("va-dev uruchamia się");

    assert!(!output.status.success());
}

#[test]
fn mic_test_rejects_zero_seconds_before_touching_microphone() {
    let output = va_dev()
        .args(["mic-test", "--seconds", "0"])
        .output()
        .expect("va-dev uruchamia się");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("wyjście w UTF-8");
    assert!(stderr.contains("większy od zera"), "{stderr}");
}

#[test]
#[ignore = "wymaga mikrofonu i zgody na nagrywanie"]
fn mic_test_saves_recording_only_when_asked() {
    let dir = tempfile::tempdir().expect("katalog tymczasowy");
    let wav = dir.path().join("proba.wav");

    let output = va_dev()
        .args(["mic-test", "--seconds", "1", "--save"])
        .arg(&wav)
        .output()
        .expect("va-dev uruchamia się");

    assert!(output.status.success(), "{output:?}");
    assert!(wav.exists());
    let without_save = va_dev()
        .current_dir(dir.path())
        .args(["mic-test", "--seconds", "1"])
        .output()
        .expect("va-dev uruchamia się");
    assert!(without_save.status.success());
    assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn model_download_reports_server_error_and_leaves_no_model() {
    let listener = std::net::TcpListener::bind("127.0.0.1:0").expect("port");
    let url = format!("http://{}/model.bin", listener.local_addr().expect("adres"));
    std::thread::spawn(move || {
        use std::io::{BufRead, Write};
        let mut stream = listener
            .incoming()
            .next()
            .expect("połączenie")
            .expect("strumień");
        let mut line = String::new();
        std::io::BufReader::new(stream.try_clone().expect("klon"))
            .read_line(&mut line)
            .expect("żądanie");
        let _ = stream
            .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
    });
    let dir = tempfile::tempdir().expect("katalog tymczasowy");

    let output = va_dev()
        .args(["model-download", "--url", &url, "--dir"])
        .arg(dir.path())
        .output()
        .expect("va-dev uruchamia się");

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).expect("wyjście w UTF-8");
    assert!(stderr.contains("kodem 404"), "{stderr}");
    assert!(!dir.path().join("ggml-large-v3-turbo.bin").exists());
}
