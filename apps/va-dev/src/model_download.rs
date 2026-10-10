//! `va-dev model-download`: pobranie modelu large-v3-turbo z postępem w terminalu.

use std::io::Write;
use std::path::PathBuf;

use va_config::{ModelVariant, Paths};
use va_model::{ModelStore, Progress, spec_for};

pub fn run(url: Option<String>, dir: Option<PathBuf>, variant: ModelVariant) -> anyhow::Result<()> {
    let dir = match dir {
        Some(dir) => dir,
        None => Paths::for_current_user()?.models_dir,
    };
    let spec = spec_for(variant);
    let url = url.unwrap_or_else(|| spec.url.to_owned());
    let store = ModelStore::new(dir, spec);
    println!("Model: {}", store.model_path().display());
    let mut last_percent = None;
    let path = store.download(&url, &mut |progress| {
        print_progress(progress, &mut last_percent)
    })?;
    println!("\nGotowe: {}", path.display());
    Ok(())
}

fn print_progress(progress: Progress, last_percent: &mut Option<u8>) {
    let percent = progress.percent();
    if *last_percent == Some(percent) {
        return;
    }
    *last_percent = Some(percent);
    print!(
        "\rPobieranie modelu… {percent:3}% ({} / {} MB)",
        progress.downloaded / 1_000_000,
        progress.total / 1_000_000
    );
    let _ = std::io::stdout().flush();
}
