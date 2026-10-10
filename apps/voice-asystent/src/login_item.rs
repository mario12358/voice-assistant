//! „Uruchamiaj przy logowaniu” (VA-SET-2): plik LaunchAgent w `~/Library/LaunchAgents`
//! wskazujący binarkę z bundla `VoiceAsystent.app` (bez `unsafe` i bez nowych zależności).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const AGENT_LABEL: &str = "io.github.mario12358.voiceasystent";
pub const LOGIN_ITEM_ID: &str = "login-item";

/// Wpis startowy dla tej aplikacji; `executable` = binarka z bundla albo `None` poza bundlem.
pub struct LoginItem {
    launch_agents_dir: PathBuf,
    executable: Option<PathBuf>,
}

impl LoginItem {
    pub fn new(launch_agents_dir: PathBuf, current_exe: &Path) -> Self {
        Self {
            launch_agents_dir,
            executable: bundle_executable(current_exe),
        }
    }

    pub fn plist_path(&self) -> PathBuf {
        self.launch_agents_dir.join(format!("{AGENT_LABEL}.plist"))
    }

    /// Czy aplikacja działa z bundla — poza nim (np. `cargo run`) wpisu nie tworzymy.
    pub fn available(&self) -> bool {
        self.executable.is_some()
    }

    /// Wpis istnieje i wskazuje dokładnie tę binarkę (inna kopia aplikacji to nie „ta”).
    pub fn is_enabled(&self) -> bool {
        let Some(executable) = &self.executable else {
            return false;
        };
        fs::read_to_string(self.plist_path()).is_ok_and(|text| {
            text.contains(&format!(
                "<string>{}</string>",
                xml_escape(&executable.to_string_lossy())
            ))
        })
    }

    pub fn enable(&self) -> io::Result<()> {
        let Some(executable) = &self.executable else {
            return Err(io::Error::other(
                "uruchamianie przy logowaniu działa tylko z VoiceAsystent.app",
            ));
        };
        fs::create_dir_all(&self.launch_agents_dir)?;
        fs::write(self.plist_path(), plist_text(executable))?;
        tracing::info!(path = %self.plist_path().display(), "uruchamianie przy logowaniu włączone");
        Ok(())
    }

    pub fn disable(&self) -> io::Result<()> {
        match fs::remove_file(self.plist_path()) {
            Ok(()) => {
                tracing::info!("uruchamianie przy logowaniu wyłączone");
                Ok(())
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    /// Przełącza stan; zwraca stan po zmianie.
    pub fn toggle(&self) -> io::Result<bool> {
        if self.is_enabled() {
            self.disable()?;
        } else {
            self.enable()?;
        }
        Ok(self.is_enabled())
    }

    /// Etykieta pozycji w menu — poza bundlem z wyjaśnieniem, dlaczego jest nieaktywna.
    pub fn menu_label(&self) -> &'static str {
        if self.available() {
            "Uruchamiaj przy logowaniu"
        } else {
            "Uruchamiaj przy logowaniu (tylko z VoiceAsystent.app)"
        }
    }
}

/// `…/VoiceAsystent.app/Contents/MacOS/VoiceAsystent` → ta sama ścieżka; inna → `None`.
fn bundle_executable(current_exe: &Path) -> Option<PathBuf> {
    let macos = current_exe.parent()?;
    let contents = macos.parent()?;
    let bundle = contents.parent()?;
    let is_bundle = macos.file_name()? == "MacOS"
        && contents.file_name()? == "Contents"
        && bundle.extension()? == "app";
    is_bundle.then(|| current_exe.to_path_buf())
}

fn plist_text(executable: &Path) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{AGENT_LABEL}</string>
	<key>ProgramArguments</key>
	<array>
		<string>{}</string>
	</array>
	<key>RunAtLoad</key>
	<true/>
	<key>ProcessType</key>
	<string>Interactive</string>
</dict>
</plist>
"#,
        xml_escape(&executable.to_string_lossy())
    )
}

fn xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use super::*;

    const BUNDLED: &str = "/Applications/VoiceAsystent.app/Contents/MacOS/VoiceAsystent";

    #[test]
    // specky: crit 01M4KD15DB3BM58PTSR1008YN8
    fn enable_writes_valid_agent_pointing_at_bundle_and_disable_removes_it() {
        let dir = tempfile::tempdir().unwrap();
        let item = LoginItem::new(dir.path().join("LaunchAgents"), Path::new(BUNDLED));
        assert!(!item.is_enabled());

        assert!(item.toggle().unwrap());

        let lint = Command::new("plutil")
            .arg("-lint")
            .arg(item.plist_path())
            .output()
            .unwrap();
        assert!(lint.status.success(), "{lint:?}");
        let text = fs::read_to_string(item.plist_path()).unwrap();
        assert!(
            text.contains(BUNDLED) && text.contains("<key>RunAtLoad</key>"),
            "{text}"
        );

        assert!(!item.toggle().unwrap());
        assert!(!item.plist_path().exists());
    }

    #[test]
    // specky: crit 01M4KD15DB7W591JT9P5FYP7Q9
    fn agent_of_another_copy_does_not_count_as_enabled() {
        let dir = tempfile::tempdir().unwrap();
        let other = LoginItem::new(
            dir.path().to_path_buf(),
            Path::new("/Users/test/Stara/VoiceAsystent.app/Contents/MacOS/VoiceAsystent"),
        );
        other.enable().unwrap();

        let current = LoginItem::new(dir.path().to_path_buf(), Path::new(BUNDLED));

        assert!(other.is_enabled());
        assert!(!current.is_enabled(), "plik wskazuje inną kopię aplikacji");
    }

    #[test]
    // specky: crit 01M4KD15DBAF4A3ATNKW62GVV1
    fn outside_bundle_the_item_is_unavailable_and_explains_why() {
        let dir = tempfile::tempdir().unwrap();
        let item = LoginItem::new(
            dir.path().to_path_buf(),
            Path::new("/Users/test/projekt/target/debug/voice-asystent"),
        );

        assert!(!item.available());
        assert!(item.menu_label().contains("tylko z VoiceAsystent.app"));
        assert!(item.enable().is_err());
        assert!(!item.plist_path().exists());
    }

    #[test]
    fn paths_with_xml_characters_are_escaped() {
        let dir = tempfile::tempdir().unwrap();
        let exe = "/Users/a&b/VoiceAsystent.app/Contents/MacOS/VoiceAsystent";
        let item = LoginItem::new(dir.path().to_path_buf(), Path::new(exe));

        item.enable().unwrap();

        assert!(item.is_enabled());
        let text = fs::read_to_string(item.plist_path()).unwrap();
        assert!(text.contains("/Users/a&amp;b/"), "{text}");
    }
}
