//! Zadanie 6.2: `scripts/build-dmg.sh` tworzy obraz .dmg z VoiceAsystent.app i skrótem
//! do /Applications, bez modelu. Bundle do testu powstaje z binarki debug (`--binary`),
//! obraz jest montowany `hdiutil attach -nobrowse` i odmontowywany po asercjach.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tempfile::TempDir;

const TEST_VERSION: &str = "v9.8.7";

fn scripts_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts")
}

fn build_test_app(out: &Path) -> PathBuf {
    let output = Command::new(scripts_dir().join("build-app.sh"))
        .arg("--binary")
        .arg(env!("CARGO_BIN_EXE_voice-asystent"))
        .arg("--out")
        .arg(out)
        .arg("--version")
        .arg(TEST_VERSION)
        .output()
        .expect("skrypt build-app.sh uruchamia się");
    assert!(output.status.success(), "build-app.sh: {output:?}");
    out.join("VoiceAsystent.app")
}

fn build_dmg(app: &Path, out: &Path) -> std::process::Output {
    Command::new(scripts_dir().join("build-dmg.sh"))
        .arg("--app")
        .arg(app)
        .arg("--out")
        .arg(out)
        .arg("--version")
        .arg(TEST_VERSION)
        .output()
        .expect("skrypt build-dmg.sh uruchamia się")
}

struct MountedImage {
    mountpoint: PathBuf,
}

impl MountedImage {
    fn attach(dmg: &Path, work: &TempDir) -> Self {
        let mountpoint = work.path().join("mnt");
        let output = Command::new("hdiutil")
            .args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
            .arg(&mountpoint)
            .arg(dmg)
            .output()
            .expect("hdiutil uruchamia się");
        assert!(output.status.success(), "hdiutil attach: {output:?}");
        Self { mountpoint }
    }
}

impl Drop for MountedImage {
    fn drop(&mut self) {
        let _ = Command::new("hdiutil")
            .args(["detach", "-quiet"])
            .arg(&self.mountpoint)
            .status();
    }
}

fn files_under(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("katalog czytelny") {
        let path = entry.expect("wpis katalogu").path();
        if path.is_dir() && !path.is_symlink() {
            files_under(&path, found);
        } else {
            found.push(path);
        }
    }
}

#[test]
// specky: crit 01M4EKHC2ZVHXRXRCVCK6TPATD
// specky: crit 01M4EQH9E1SE75YQPG89XWQV7B
fn dmg_contains_app_and_applications_link_without_model() {
    let work = tempfile::tempdir().expect("katalog tymczasowy");
    let app = build_test_app(work.path());

    let output = build_dmg(&app, work.path());
    assert!(output.status.success(), "build-dmg.sh: {output:?}");
    let dmg = work.path().join("VoiceAsystent-9.8.7.dmg");
    assert!(dmg.is_file(), "brak obrazu {dmg:?}");

    let mounted = MountedImage::attach(&dmg, &work);
    let volume = &mounted.mountpoint;
    assert!(
        volume
            .join("VoiceAsystent.app/Contents/Info.plist")
            .is_file()
    );
    assert!(
        volume
            .join("VoiceAsystent.app/Contents/MacOS/VoiceAsystent")
            .is_file()
    );
    assert_eq!(
        fs::read_link(volume.join("Applications")).expect("skrót do Aplikacji"),
        Path::new("/Applications")
    );

    let mut files = Vec::new();
    files_under(volume, &mut files);
    let model_like: Vec<_> = files
        .iter()
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
            name.ends_with(".bin") || name.contains("ggml")
        })
        .collect();
    assert!(model_like.is_empty(), "model w obrazie: {model_like:?}");
}

#[test]
fn dmg_build_fails_without_app_bundle() {
    let work = tempfile::tempdir().expect("katalog tymczasowy");

    let output = build_dmg(&work.path().join("Nieistniejaca.app"), work.path());

    assert!(!output.status.success());
    assert!(!work.path().join("VoiceAsystent-9.8.7.dmg").exists());
}
