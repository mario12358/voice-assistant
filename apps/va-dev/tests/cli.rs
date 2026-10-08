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
