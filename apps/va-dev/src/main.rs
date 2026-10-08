#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

use clap::Parser;

/// Narzędzia deweloperskie VoiceAsystent.
#[derive(Parser)]
#[command(name = "va-dev", version, about)]
struct Cli {}

fn main() -> anyhow::Result<()> {
    let _cli = Cli::parse();
    Ok(())
}
