//! Zadanie 6.1: `scripts/build-app.sh` składa poprawny bundle VoiceAsystent.app
//! (Info.plist, ikona, podpis ad-hoc, bez modelu). Bundle do testów powstaje z binarki
//! debug przez `--binary`, więc test nie potrzebuje builda release.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use tempfile::TempDir;

const TEST_VERSION: &str = "v9.8.7-3-gabcdef0";

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn build_script() -> PathBuf {
    workspace_root().join("scripts/build-app.sh")
}

fn bundled_app() -> &'static Path {
    static BUNDLE: OnceLock<(TempDir, PathBuf)> = OnceLock::new();
    let (_, app) = BUNDLE.get_or_init(|| {
        let out = tempfile::tempdir().expect("katalog tymczasowy");
        let output = Command::new(build_script())
            .arg("--binary")
            .arg(env!("CARGO_BIN_EXE_voice-asystent"))
            .arg("--out")
            .arg(out.path())
            .arg("--version")
            .arg(TEST_VERSION)
            .output()
            .expect("skrypt build-app.sh uruchamia się");
        assert!(output.status.success(), "build-app.sh: {output:?}");
        let app = out.path().join("VoiceAsystent.app");
        (out, app)
    });
    app
}

fn plist_value(key: &str) -> String {
    let plist = bundled_app().join("Contents/Info.plist");
    let output = Command::new("plutil")
        .args(["-extract", key, "raw", "-o", "-"])
        .arg(&plist)
        .output()
        .expect("plutil uruchamia się");
    assert!(output.status.success(), "brak klucza {key}: {output:?}");
    String::from_utf8(output.stdout)
        .expect("wartość w UTF-8")
        .trim()
        .to_string()
}

fn files_under(dir: &Path, found: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(dir).expect("katalog bundla czytelny") {
        let path = entry.expect("wpis katalogu").path();
        if path.is_dir() {
            files_under(&path, found);
        } else {
            found.push(path);
        }
    }
}

#[test]
fn bundle_has_app_structure_icon_and_valid_signature() {
    let app = bundled_app();

    assert!(app.join("Contents/MacOS/VoiceAsystent").is_file());
    assert!(app.join("Contents/Resources/AppIcon.icns").is_file());
    assert_eq!(
        fs::read_to_string(app.join("Contents/PkgInfo")).unwrap(),
        "APPL????"
    );

    let lint = Command::new("plutil")
        .arg("-lint")
        .arg(app.join("Contents/Info.plist"))
        .output()
        .unwrap();
    assert!(lint.status.success(), "{lint:?}");

    let signature = Command::new("codesign")
        .args(["--verify", "--strict"])
        .arg(app)
        .output()
        .unwrap();
    assert!(signature.status.success(), "{signature:?}");
}

#[test]
// specky: crit 01M4EKHC2ZXTQ2VB5D4XWJD64K
fn info_plist_describes_menu_bar_app_with_microphone_usage() {
    assert_eq!(plist_value("CFBundleExecutable"), "VoiceAsystent");
    assert_eq!(
        plist_value("CFBundleIdentifier"),
        "io.github.mario12358.voiceasystent"
    );
    assert_eq!(plist_value("LSUIElement"), "true");
    assert_eq!(plist_value("LSMinimumSystemVersion"), "13.0");
    assert_eq!(plist_value("CFBundleShortVersionString"), "9.8.7");
    assert_eq!(plist_value("CFBundleVersion"), TEST_VERSION);

    let microphone = plist_value("NSMicrophoneUsageDescription");
    assert!(
        microphone.contains("mikrofon") && microphone.contains("schowka"),
        "opis uprawnienia po polsku: {microphone}"
    );
}

#[test]
fn bundle_contains_no_model_file() {
    let mut files = Vec::new();
    files_under(bundled_app(), &mut files);

    let model_like: Vec<_> = files
        .iter()
        .filter(|path| {
            let name = path.file_name().unwrap().to_string_lossy().to_lowercase();
            name.ends_with(".bin") || name.contains("ggml") || name.contains("whisper")
        })
        .collect();
    assert!(model_like.is_empty(), "model w bundlu: {model_like:?}");
}

#[test]
fn bundled_binary_passes_self_check() {
    let home = tempfile::tempdir().expect("katalog tymczasowy");

    let output = Command::new(bundled_app().join("Contents/MacOS/VoiceAsystent"))
        .env("HOME", home.path())
        .arg("--self-check")
        .output()
        .expect("binarka z bundla uruchamia się");

    assert!(output.status.success(), "{output:?}");
}

#[test]
#[ignore = "wymaga builda release z Metal (kilka minut) — uruchamiane z --include-ignored"]
fn release_build_creates_dist_app() {
    let output = Command::new(build_script())
        .output()
        .expect("skrypt build-app.sh uruchamia się");
    assert!(output.status.success(), "build-app.sh: {output:?}");

    let app = workspace_root().join("dist/VoiceAsystent.app");
    let home = tempfile::tempdir().expect("katalog tymczasowy");
    let self_check = Command::new(app.join("Contents/MacOS/VoiceAsystent"))
        .env("HOME", home.path())
        .arg("--self-check")
        .output()
        .expect("binarka release uruchamia się");
    assert!(self_check.status.success(), "{self_check:?}");
}
