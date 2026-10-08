#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

mod mic_test;
mod model_download;

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
use va_audio::{AudioHost, CpalHost, choose_device};
use va_config::{Config, Paths};
use va_core::logging::{self, LogOptions};

/// Narzędzia deweloperskie VoiceAsystent.
#[derive(Parser)]
#[command(name = "va-dev", version, about)]
struct Cli {
    /// Więcej logów na stderr (-v debug, -vv trace).
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Wypisuje obowiązującą konfigurację (plik + wartości domyślne).
    Config {
        /// Plik konfiguracji; domyślnie ten, którego używa aplikacja.
        #[arg(long)]
        file: Option<PathBuf>,
    },
    /// Wypisuje dostępne mikrofony (* = domyślny systemu, > = używany wg konfiguracji).
    Devices,
    /// Nagrywa próbnie z mikrofonu i pokazuje poziom sygnału; WAV tylko z --save.
    MicTest {
        /// Długość nagrania w sekundach.
        #[arg(long, default_value_t = 5)]
        seconds: u32,
        /// Zapisz nagranie (16 kHz mono) do pliku WAV.
        #[arg(long)]
        save: Option<PathBuf>,
    },
    /// Pobiera model large-v3-turbo (wznawia przerwane pobieranie, sprawdza SHA-256).
    ModelDownload {
        /// Adres pliku modelu; domyślnie Hugging Face.
        #[arg(long)]
        url: Option<String>,
        /// Katalog docelowy; domyślnie katalog modeli aplikacji.
        #[arg(long)]
        dir: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _log_guard = logging::init(&LogOptions {
        logs_dir: None,
        verbosity: cli.verbose,
    });
    match cli.command {
        Some(Command::Config { file }) => show_config(file),
        Some(Command::Devices) => list_devices(),
        Some(Command::MicTest { seconds, save }) => mic_test::run(seconds, save.as_deref()),
        Some(Command::ModelDownload { url, dir }) => model_download::run(url, dir),
        None => Ok(()),
    }
}

fn list_devices() -> anyhow::Result<()> {
    let (config, _) = Config::load_or_default(&Paths::for_current_user()?.config_file);
    let devices = CpalHost::new().input_devices()?;
    let choice = choose_device(&devices, config.microphone.as_deref())?;
    for device in &devices {
        let used = if device.name == choice.name { ">" } else { " " };
        let default = if device.is_default { "*" } else { " " };
        println!("{used}{default} {}", device.name);
    }
    Ok(())
}

fn show_config(file: Option<PathBuf>) -> anyhow::Result<()> {
    let path = match file {
        Some(path) => path,
        None => Paths::for_current_user()?.config_file,
    };
    let config = Config::load(&path)?;
    let text = toml::to_string_pretty(&config).context("serializacja konfiguracji")?;
    println!("# {}\n{text}", path.display());
    Ok(())
}
