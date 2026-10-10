use std::process::Command;

#[test]
// specky: crit 01M4EKHCMW0ZW9K5GYM7D2NM9H
fn startup_reports_gpu_verdict() {
    let home = tempfile::tempdir().expect("katalog tymczasowy");

    let output = Command::new(env!("CARGO_BIN_EXE_voice-asystent"))
        .env("HOME", home.path())
        .arg("--self-check")
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

fn write_config(home: &std::path::Path, text: &str) {
    let config = home.join("Library/Application Support/VoiceAsystent/config.toml");
    std::fs::create_dir_all(config.parent().unwrap()).unwrap();
    std::fs::write(config, text).unwrap();
}

#[test]
// specky: crit 01M4K6M33C1BCHBQ7DEG4GER2W
fn startup_uses_custom_model_path_from_config() {
    let home = tempfile::tempdir().expect("katalog tymczasowy");
    let model = home.path().join("wlasny-model.bin");
    std::fs::write(&model, b"ggml").unwrap();
    write_config(
        home.path(),
        &format!("model_path = \"{}\"\n", model.display()),
    );

    let output = Command::new(env!("CARGO_BIN_EXE_voice-asystent"))
        .env("HOME", home.path())
        .arg("--self-check")
        .output()
        .expect("voice-asystent uruchamia się");

    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("wyjście w UTF-8");
    assert!(stderr.contains("własna ścieżka z konfiguracji"), "{stderr}");
    assert!(
        stderr.contains("Ready(") && stderr.contains("wlasny-model.bin"),
        "{stderr}"
    );
    assert!(!stderr.contains("Missing"), "{stderr}");
}

#[test]
// specky: crit 01M4K6M33CWKR54XAT4JVG3HWG
fn startup_reports_missing_custom_model_with_its_path() {
    let home = tempfile::tempdir().expect("katalog tymczasowy");
    write_config(home.path(), "model_path = \"/nie/ma/takiego/modelu.bin\"\n");

    let output = Command::new(env!("CARGO_BIN_EXE_voice-asystent"))
        .env("HOME", home.path())
        .arg("--self-check")
        .output()
        .expect("voice-asystent uruchamia się");

    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("wyjście w UTF-8");
    assert!(
        stderr.contains("/nie/ma/takiego/modelu.bin") && stderr.contains("Missing"),
        "{stderr}"
    );
}

#[test]
fn startup_reports_missing_model() {
    let home = tempfile::tempdir().expect("katalog tymczasowy");

    let output = Command::new(env!("CARGO_BIN_EXE_voice-asystent"))
        .env("HOME", home.path())
        .arg("--self-check")
        .output()
        .expect("voice-asystent uruchamia się");

    assert!(output.status.success(), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("wyjście w UTF-8");
    assert!(
        stderr.contains("model: stan przy starcie") && stderr.contains("Missing"),
        "{stderr}"
    );
}
