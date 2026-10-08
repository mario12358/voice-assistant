//! `va-dev mic-test`: nagranie próbne z mikrofonu, poziom sygnału i opcjonalny zapis WAV.

use std::path::Path;
use std::time::Duration;

use anyhow::Context;
use va_audio::{
    AudioHost, CpalHost, CpalRecorder, Recorder, SilenceParams, TARGET_SAMPLE_RATE, choose_device,
    speech_bounds,
};
use va_config::{Config, Paths};

const SEGMENT_MS: usize = 250;
const BAR_WIDTH: usize = 40;
const FLOOR_DB: f32 = -60.0;

pub fn run(seconds: u32, save: Option<&Path>) -> anyhow::Result<()> {
    anyhow::ensure!(seconds > 0, "czas nagrania musi być większy od zera");
    let (config, _) = Config::load_or_default(&Paths::for_current_user()?.config_file);
    let devices = CpalHost::new().input_devices()?;
    for device in &devices {
        let default = if device.is_default { "*" } else { " " };
        println!("{default} {}", device.name);
    }
    let choice = choose_device(&devices, config.microphone.as_deref())?;
    println!("Nagrywam {seconds} s z: {}", choice.name);
    let mut recorder = CpalRecorder::new(config.max_recording_secs);
    recorder.start(&choice.name)?;
    std::thread::sleep(Duration::from_secs(u64::from(seconds)));
    let samples = recorder.stop()?;
    print_levels(&samples);
    print_speech_bounds(&samples, &config);
    if let Some(path) = save {
        write_wav(path, &samples)?;
        println!("Zapisano: {}", path.display());
    }
    Ok(())
}

fn print_levels(samples: &[f32]) {
    let segment = TARGET_SAMPLE_RATE as usize * SEGMENT_MS / 1000;
    for (index, chunk) in samples.chunks(segment).enumerate() {
        let start_s = (index * SEGMENT_MS) as f32 / 1000.0;
        println!("{start_s:6.2} s  {}", level_bar(chunk));
    }
}

fn print_speech_bounds(samples: &[f32], config: &Config) {
    let params = SilenceParams {
        threshold_rms: config.silence.threshold_rms,
        padding_ms: config.silence.padding_ms,
    };
    let rate = TARGET_SAMPLE_RATE as f32;
    match speech_bounds(samples, TARGET_SAMPLE_RATE, &params) {
        Some(bounds) => println!(
            "Mowa: {:.2}–{:.2} s (po przycięciu ciszy)",
            bounds.start as f32 / rate,
            bounds.end as f32 / rate
        ),
        None => println!("Sama cisza — nagranie nie trafiłoby do transkrypcji"),
    }
}

/// Pasek poziomu RMS w dBFS, od -60 dB (pusty) do 0 dB (pełny).
pub fn level_bar(samples: &[f32]) -> String {
    let db = rms_dbfs(samples);
    let fraction = ((db - FLOOR_DB) / -FLOOR_DB).clamp(0.0, 1.0);
    let filled = (fraction * BAR_WIDTH as f32).round() as usize;
    format!(
        "{}{} {db:6.1} dB",
        "#".repeat(filled),
        ".".repeat(BAR_WIDTH - filled)
    )
}

fn rms_dbfs(samples: &[f32]) -> f32 {
    if samples.is_empty() {
        return FLOOR_DB;
    }
    let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();
    (20.0 * rms.log10()).max(FLOOR_DB)
}

/// Zapis 16 kHz mono jako WAV PCM 16 bit — tylko na jawne żądanie (`--save`).
pub fn write_wav(path: &Path, samples: &[f32]) -> anyhow::Result<()> {
    let spec = hound::WavSpec {
        channels: 1,
        sample_rate: TARGET_SAMPLE_RATE,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };
    let context = || format!("zapis WAV {}", path.display());
    let mut writer = hound::WavWriter::create(path, spec).with_context(context)?;
    for sample in samples {
        let value = (sample.clamp(-1.0, 1.0) * f32::from(i16::MAX)).round() as i16;
        writer.write_sample(value).with_context(context)?;
    }
    writer.finalize().with_context(context)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_wav(path: &Path) -> (hound::WavSpec, Vec<f32>) {
        let mut reader = hound::WavReader::open(path).unwrap();
        let samples = reader
            .samples::<i16>()
            .map(|sample| f32::from(sample.unwrap()) / f32::from(i16::MAX))
            .collect();
        (reader.spec(), samples)
    }

    #[test]
    fn saved_wav_reads_back_as_16khz_mono_with_same_samples() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("nagranie.wav");
        let samples: Vec<f32> = (0..1_600)
            .map(|i| ((i % 100) as f32 / 50.0) - 1.0)
            .collect();

        write_wav(&path, &samples).unwrap();

        let (spec, read_back) = read_wav(&path);
        assert_eq!(spec.sample_rate, TARGET_SAMPLE_RATE);
        assert_eq!(spec.channels, 1);
        assert_eq!(read_back.len(), samples.len());
        for (written, read) in samples.iter().zip(&read_back) {
            assert!((written - read).abs() < 1e-4, "{written} → {read}");
        }
    }

    #[test]
    fn out_of_range_samples_are_clipped_not_wrapped() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("przester.wav");

        write_wav(&path, &[1.5, -1.5]).unwrap();

        assert_eq!(read_wav(&path).1, vec![1.0, -1.0]);
    }

    #[test]
    fn writing_to_missing_directory_is_an_error() {
        let dir = tempfile::tempdir().unwrap();

        assert!(write_wav(&dir.path().join("brak/plik.wav"), &[0.0]).is_err());
    }

    #[test]
    fn level_bar_is_empty_for_silence_and_full_for_full_scale() {
        assert!(level_bar(&[0.0; 100]).starts_with(&".".repeat(BAR_WIDTH)));
        assert!(level_bar(&[1.0, -1.0]).starts_with(&"#".repeat(BAR_WIDTH)));
    }
}
