#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

use std::path::PathBuf;

use anyhow::Context;
use clap::{Parser, Subcommand};
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
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let _log_guard = logging::init(&LogOptions {
        logs_dir: None,
        verbosity: cli.verbose,
    });
    match cli.command {
        Some(Command::Config { file }) => show_config(file),
        None => Ok(()),
    }
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
