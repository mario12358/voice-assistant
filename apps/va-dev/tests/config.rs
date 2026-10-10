use std::fs;
use std::process::{Command, Output};

fn config_command(file: &std::path::Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_va-dev"))
        .arg("config")
        .arg("--file")
        .arg(file)
        .output()
        .expect("va-dev uruchamia się")
}

#[test]
fn config_without_file_prints_defaults() {
    let dir = tempfile::tempdir().unwrap();

    let output = config_command(&dir.path().join("brak.toml"));

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("language = \"auto\""), "{stdout}");
    assert!(stdout.contains("max_recording_secs = 600"), "{stdout}");
}

#[test]
fn config_prints_values_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(
        &path,
        "microphone = \"USB Audio Device\"\nlanguage = \"pl\"\n",
    )
    .unwrap();

    let output = config_command(&path);

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("microphone = \"USB Audio Device\""),
        "{stdout}"
    );
    assert!(stdout.contains("language = \"pl\""), "{stdout}");
}

#[test]
fn config_with_invalid_toml_fails_and_names_line() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("config.toml");
    fs::write(&path, "language = \"pl\"\nmicrophone =\n").unwrap();

    let output = config_command(&path);

    assert!(!output.status.success());
    let stderr = String::from_utf8(output.stderr).unwrap();
    assert!(stderr.contains("linia 2"), "{stderr}");
}
