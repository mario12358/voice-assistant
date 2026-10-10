//! Zadanie 6.2: `scripts/build-dmg.sh` tworzy obraz .dmg z VoiceAsystent.app i skrótem
//! do /Applications, bez modelu. Bundle do testu powstaje z binarki debug (`--binary`),
//! obraz jest montowany `hdiutil attach -nobrowse` i odmontowywany po asercjach.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, MutexGuard};

use tempfile::TempDir;

const TEST_VERSION: &str = "v9.8.7";

/// Testy obrazów jeden po drugim: równoległe `hdiutil create/attach` kończą się
/// „Resource temporarily unavailable”. Pod nextest to samo robi grupa `obrazy-dysku`
/// w `.config/nextest.toml` (każdy test w osobnym procesie).
fn one_image_at_a_time() -> MutexGuard<'static, ()> {
    static IMAGES: Mutex<()> = Mutex::new(());
    IMAGES
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

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
        // Montowanie bywa chwilowo odrzucane przy kilku obrazach naraz — do trzech prób.
        let mut output = None;
        for attempt in 1..=3u64 {
            let result = Command::new("hdiutil")
                .args(["attach", "-nobrowse", "-readonly", "-mountpoint"])
                .arg(&mountpoint)
                .arg(dmg)
                .output()
                .expect("hdiutil uruchamia się");
            if result.status.success() {
                return Self { mountpoint };
            }
            output = Some(result);
            std::thread::sleep(std::time::Duration::from_secs(attempt * 2));
        }
        panic!("hdiutil attach: {output:?}");
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
    let _images = one_image_at_a_time();
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
    let _images = one_image_at_a_time();
    let work = tempfile::tempdir().expect("katalog tymczasowy");

    let output = build_dmg(&work.path().join("Nieistniejaca.app"), work.path());

    assert!(!output.status.success());
    assert!(!work.path().join("VoiceAsystent-9.8.7.dmg").exists());
}

fn check_dmg(dmg: &Path, max_mb: &str) -> std::process::Output {
    Command::new(scripts_dir().join("check-dmg.sh"))
        .arg(dmg)
        .arg("--max-mb")
        .arg(max_mb)
        .output()
        .expect("skrypt check-dmg.sh uruchamia się")
}

#[test]
// specky: crit 01M4KD15NZCFGE5HQME8HXCGJX
fn release_check_accepts_small_image_without_model() {
    let _images = one_image_at_a_time();
    let work = tempfile::tempdir().expect("katalog tymczasowy");
    let app = build_test_app(work.path());
    assert!(build_dmg(&app, work.path()).status.success());

    let output = check_dmg(&work.path().join("VoiceAsystent-9.8.7.dmg"), "20");

    assert!(output.status.success(), "{output:?}");
}

#[test]
// specky: crit 01M4KD15NZYVAV8WJAC6D217TD
fn release_check_rejects_image_with_model_or_over_size_limit() {
    let _images = one_image_at_a_time();
    let work = tempfile::tempdir().expect("katalog tymczasowy");
    let app = build_test_app(work.path());
    fs::write(
        app.join("Contents/Resources/ggml-large-v3-turbo.bin"),
        b"to nie powinno trafic do obrazu",
    )
    .unwrap();
    assert!(build_dmg(&app, work.path()).status.success());
    let dmg = work.path().join("VoiceAsystent-9.8.7.dmg");

    let with_model = check_dmg(&dmg, "20");
    assert_eq!(with_model.status.code(), Some(1), "{with_model:?}");
    assert!(String::from_utf8_lossy(&with_model.stderr).contains("modelu"));

    let too_big = check_dmg(&dmg, "0");
    assert_eq!(too_big.status.code(), Some(1), "{too_big:?}");
    assert!(String::from_utf8_lossy(&too_big.stderr).contains("za duży"));
}

#[test]
fn release_check_fails_loudly_when_image_cannot_be_mounted() {
    let _images = one_image_at_a_time();
    let work = tempfile::tempdir().expect("katalog tymczasowy");
    let fake = work.path().join("VoiceAsystent-9.8.7.dmg");
    fs::write(&fake, b"to nie jest obraz dysku").unwrap();

    let output = check_dmg(&fake, "20");

    assert_eq!(output.status.code(), Some(1), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("próba 3/3"), "{stderr}");
    assert!(stderr.contains("nie udało się zamontować"), "{stderr}");
}
