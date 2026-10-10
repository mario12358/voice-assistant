//! `va-dev transcribe <plik.wav>`: WAV → 16 kHz mono → przycięcie ciszy i skrócenie pauz →
//! Whisper → stdout (ta sama ścieżka co w kontrolerze aplikacji).

use std::path::Path;

use anyhow::Context;
use va_audio::{
    SilenceParams, TARGET_SAMPLE_RATE, compress_pauses, convert::normalize, trim_silence,
};
use va_config::{Config, Paths};
use va_model::spec_for;
use va_stt::{METAL_BUILT, MetalProbe, SpeechToText, Transcript, WhisperStt, require_metal};

pub fn run(wav: &Path) -> anyhow::Result<()> {
    let paths = Paths::for_current_user()?;
    let (config, _) = Config::load_or_default(&paths.config_file);
    let gpu = require_metal(&MetalProbe, METAL_BUILT)?;
    let model = config.model_path.clone().unwrap_or_else(|| {
        paths
            .models_dir
            .join(spec_for(config.model_variant).file_name)
    });
    anyhow::ensure!(
        model.exists(),
        "brak modelu {} — uruchom `va-dev model-download`",
        model.display()
    );
    let mut stt = WhisperStt::load(&model, &gpu, config.language)?;
    match transcribe_file(wav, &mut stt, &silence_params(&config))? {
        Some(transcript) => {
            println!("{}", transcript.text);
            eprintln!("Czas inferencji: {} ms", transcript.inference.as_millis());
        }
        None => eprintln!("Sama cisza — nic do transkrypcji"),
    }
    Ok(())
}

fn silence_params(config: &Config) -> SilenceParams {
    SilenceParams {
        threshold_rms: config.silence.threshold_rms,
        padding_ms: config.silence.padding_ms,
    }
}

/// Transkrypcja pliku WAV; sama cisza nie trafia do STT i daje `None`.
pub fn transcribe_file(
    wav: &Path,
    stt: &mut dyn SpeechToText,
    silence: &SilenceParams,
) -> anyhow::Result<Option<Transcript>> {
    let (interleaved, channels, sample_rate) = read_wav(wav)?;
    let samples = normalize(&interleaved, channels, sample_rate)?;
    let speech = trim_silence(&samples, TARGET_SAMPLE_RATE, silence);
    if speech.is_empty() {
        return Ok(None);
    }
    let speech = compress_pauses(speech, TARGET_SAMPLE_RATE, silence);
    Ok(Some(stt.transcribe(&speech)?))
}

/// Próbki przeplatane jako f32 (-1.0..1.0), liczba kanałów, częstotliwość.
fn read_wav(path: &Path) -> anyhow::Result<(Vec<f32>, u16, u32)> {
    let context = || format!("odczyt WAV {}", path.display());
    let mut reader = hound::WavReader::open(path).with_context(context)?;
    let spec = reader.spec();
    let samples = match spec.sample_format {
        hound::SampleFormat::Float => reader
            .samples::<f32>()
            .collect::<Result<Vec<_>, _>>()
            .with_context(context)?,
        hound::SampleFormat::Int => {
            let scale = (1_i64 << (spec.bits_per_sample - 1)) as f32;
            reader
                .samples::<i32>()
                .map(|sample| sample.map(|value| value as f32 / scale))
                .collect::<Result<Vec<_>, _>>()
                .with_context(context)?
        }
    };
    Ok((samples, spec.channels, spec.sample_rate))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use va_stt::testing::ScriptedStt;

    use super::*;

    const DEFAULT_SILENCE: SilenceParams = SilenceParams {
        threshold_rms: 0.01,
        padding_ms: 200,
    };

    fn fixture(name: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures")
            .join(name)
    }

    #[test]
    fn stereo_44khz_wav_reaches_stt_as_16khz_mono() {
        let mut stt = ScriptedStt::answering([Ok("tekst".into())]);
        let observer = stt.clone();

        let transcript = transcribe_file(
            &fixture("speech_en_44k_stereo.wav"),
            &mut stt,
            &DEFAULT_SILENCE,
        )
        .unwrap()
        .unwrap();

        assert_eq!(transcript.text, "tekst");
        let received = observer.received();
        assert_eq!(received.len(), 1);
        let (original, ..) = read_wav(&fixture("speech_en_44k_stereo.wav")).unwrap();
        // Po downmixie i resamplingu nagranie ma najwyżej tyle próbek, ile 16 kHz mono tej długości
        // (mniej o przyciętą ciszę), a na pewno nie tyle, co surowe 44,1 kHz stereo.
        let mono_16k_len = original.len() / 2 * 16_000 / 44_100;
        assert!(
            received[0].len() <= mono_16k_len + 16 && received[0].len() > mono_16k_len / 2,
            "{} próbek, oczekiwane najwyżej {mono_16k_len} (mono 16 kHz)",
            received[0].len()
        );
    }

    #[test]
    fn same_16khz_mono_fixture_gives_same_length_as_stereo_44khz() {
        let mut stereo_stt = ScriptedStt::answering([Ok(String::new())]);
        let mut mono_stt = ScriptedStt::answering([Ok(String::new())]);
        let stereo_seen = stereo_stt.clone();
        let mono_seen = mono_stt.clone();

        transcribe_file(
            &fixture("speech_en_44k_stereo.wav"),
            &mut stereo_stt,
            &DEFAULT_SILENCE,
        )
        .unwrap();
        transcribe_file(&fixture("speech_en.wav"), &mut mono_stt, &DEFAULT_SILENCE).unwrap();

        let stereo_len = stereo_seen.received()[0].len();
        let mono_len = mono_seen.received()[0].len();
        assert!(
            stereo_len.abs_diff(mono_len) <= 800,
            "44,1 kHz stereo → {stereo_len}, 16 kHz mono → {mono_len}"
        );
    }

    #[test]
    fn silence_only_wav_never_reaches_stt() {
        let mut stt = ScriptedStt::answering([]);
        let observer = stt.clone();

        let result =
            transcribe_file(&fixture("silence_only.wav"), &mut stt, &DEFAULT_SILENCE).unwrap();

        assert!(result.is_none());
        assert!(observer.received().is_empty());
    }

    #[test]
    fn long_pause_is_shortened_before_stt() {
        let mut stt = ScriptedStt::answering([Ok("dwa zdania".into())]);
        let observer = stt.clone();
        let (original, ..) = read_wav(&fixture("speech_pl_long_pause.wav")).unwrap();

        transcribe_file(
            &fixture("speech_pl_long_pause.wav"),
            &mut stt,
            &DEFAULT_SILENCE,
        )
        .unwrap();

        let received = observer.received();
        assert_eq!(received.len(), 1);
        let removed_seconds = (original.len() - received[0].len()) / 16_000;
        assert!(
            removed_seconds >= 39,
            "STT dostał {} z {} próbek",
            received[0].len(),
            original.len()
        );
    }

    #[test]
    fn missing_file_is_a_readable_error() {
        let mut stt = ScriptedStt::answering([]);

        let error = transcribe_file(&fixture("nie-ma.wav"), &mut stt, &DEFAULT_SILENCE)
            .unwrap_err()
            .to_string();

        assert!(error.contains("odczyt WAV"), "{error}");
    }
}
