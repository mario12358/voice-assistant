#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

use va_config::{Config, Paths};

fn main() -> anyhow::Result<()> {
    let paths = Paths::for_current_user()?;
    let (_config, config_error) = Config::load_or_default(&paths.config_file);
    if let Some(error) = config_error {
        eprintln!("{error} — używam ustawień domyślnych");
    }
    Ok(())
}
