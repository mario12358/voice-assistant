#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

mod app;
mod click;
mod download;
mod history_menu;
mod hotkeys;
mod indicator;
mod messages;
mod microphones;
mod model_menu;
mod startup;
mod tray_menu;

use va_config::{Config, Paths};
use va_core::logging::{self, LogOptions};

/// `--self-check`: same sprawdzenia startowe (GPU, model) bez paska menu — do testów i diagnozy.
const SELF_CHECK_FLAG: &str = "--self-check";

fn main() -> anyhow::Result<()> {
    let paths = Paths::for_current_user()?;
    let _log_guard = logging::init(&LogOptions {
        logs_dir: Some(paths.logs_dir.clone()),
        verbosity: 0,
    });
    tracing::info!(version = env!("CARGO_PKG_VERSION"), "start VoiceAsystent");
    let (config, config_error) = Config::load_or_default(&paths.config_file);
    if let Some(error) = config_error {
        tracing::warn!(%error, "używam ustawień domyślnych");
    }
    let gpu = startup::check_gpu();
    let source = startup::ModelSource::from_config(&config, &paths);
    let model = startup::check_model(&source);
    if std::env::args().any(|arg| arg == SELF_CHECK_FLAG) {
        return Ok(());
    }
    let controller_parts =
        startup::controller_parts(&config, &paths, &source, gpu.as_ref(), model.as_ref());
    let download =
        startup::ModelDownload::needed(&config, &paths, &source, gpu.as_ref(), model.as_ref());
    app::run(
        controller_parts,
        download,
        source.info(),
        paths.config_file,
        config.max_recording_secs,
    )
}
