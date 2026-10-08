use std::process::Command;

#[test]
// specky: crit 01M4EKHCMW0ZW9K5GYM7D2NM9H
fn startup_reports_gpu_verdict() {
    let home = tempfile::tempdir().expect("katalog tymczasowy");

    let output = Command::new(env!("CARGO_BIN_EXE_voice-asystent"))
        .env("HOME", home.path())
        .output()
        .expect("voice-asystent uruchamia się");

    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("wyjście w UTF-8");
    let metal_ok = stderr.contains("GPU Metal dostępne");
    let blocked = stderr.contains("Wymagane GPU (Metal) — praca na CPU nie jest wspierana");
    assert!(
        metal_ok ^ blocked,
        "brak werdyktu GPU w logu startu:\n{stderr}"
    );
}
