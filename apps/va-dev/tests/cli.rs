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
