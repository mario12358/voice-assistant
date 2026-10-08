#[cfg(not(target_os = "macos"))]
compile_error!("VoiceAsystent jest wspierany wyłącznie na macOS");

fn main() -> anyhow::Result<()> {
    Ok(())
}
