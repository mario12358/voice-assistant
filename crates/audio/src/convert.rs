//! Normalizacja nagrania do formatu wymaganego przez Whisper: 16 kHz, mono, f32.

use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

use crate::{Error, Result};

pub const TARGET_SAMPLE_RATE: u32 = 16_000;

const RESAMPLER_CHUNK_FRAMES: usize = 1024;

/// Uśrednia kanały próbek przeplatanych (L R L R …) do jednego kanału.
pub fn downmix_to_mono(interleaved: &[f32], channels: u16) -> Vec<f32> {
    let channels = usize::from(channels.max(1));
    interleaved
        .chunks_exact(channels)
        .map(|frame| frame.iter().sum::<f32>() / channels as f32)
        .collect()
}

/// Przelicza sygnał mono z `sample_rate` na 16 kHz.
pub fn resample_to_target(mono: &[f32], sample_rate: u32) -> Result<Vec<f32>> {
    if sample_rate == TARGET_SAMPLE_RATE || mono.is_empty() {
        return Ok(mono.to_vec());
    }
    let mut resampler = Fft::<f32>::new(
        sample_rate as usize,
        TARGET_SAMPLE_RATE as usize,
        RESAMPLER_CHUNK_FRAMES,
        1,
        FixedSync::Input,
    )
    .map_err(resampling)?;
    let input = InterleavedSlice::new(mono, 1, mono.len()).map_err(resampling)?;
    let output = resampler
        .process_all(&input, mono.len(), None)
        .map_err(resampling)?;
    Ok(output.take_data())
}

/// Downmix + resampling: dowolny format urządzenia → 16 kHz mono f32.
pub fn normalize(interleaved: &[f32], channels: u16, sample_rate: u32) -> Result<Vec<f32>> {
    resample_to_target(&downmix_to_mono(interleaved, channels), sample_rate)
}

fn resampling(error: impl std::fmt::Display) -> Error {
    Error::Resampling(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::*;

    fn sine(frequency: f32, sample_rate: u32, seconds: f32) -> Vec<f32> {
        let frames = (sample_rate as f32 * seconds) as usize;
        (0..frames)
            .map(|index| (TAU * frequency * index as f32 / sample_rate as f32).sin() * 0.5)
            .collect()
    }

    fn interleave_stereo(left: &[f32], right: &[f32]) -> Vec<f32> {
        left.iter()
            .zip(right)
            .flat_map(|(left, right)| [*left, *right])
            .collect()
    }

    /// Częstotliwość dominująca liczona z przejść przez zero (wystarcza dla czystego sinusa).
    fn dominant_frequency(samples: &[f32], sample_rate: u32) -> f32 {
        let crossings = samples
            .windows(2)
            .filter(|pair| pair[0] < 0.0 && pair[1] >= 0.0)
            .count();
        crossings as f32 * sample_rate as f32 / samples.len() as f32
    }

    #[test]
    fn downmix_averages_channels() {
        let stereo = [1.0, 0.0, 0.5, 0.5, -1.0, 1.0];

        assert_eq!(downmix_to_mono(&stereo, 2), vec![0.5, 0.5, 0.0]);
    }

    #[test]
    fn mono_input_passes_through_downmix() {
        let mono = [0.1, 0.2, 0.3];

        assert_eq!(downmix_to_mono(&mono, 1), mono.to_vec());
    }

    #[test]
    fn already_16khz_mono_is_unchanged() {
        let mono = sine(440.0, TARGET_SAMPLE_RATE, 0.1);

        assert_eq!(normalize(&mono, 1, TARGET_SAMPLE_RATE).unwrap(), mono);
    }

    #[test]
    fn stereo_48khz_becomes_16khz_mono_with_same_length_and_tone() {
        let tone = sine(440.0, 48_000, 2.0);
        let stereo = interleave_stereo(&tone, &tone);

        let normalized = normalize(&stereo, 2, 48_000).unwrap();

        let expected_frames = 2 * TARGET_SAMPLE_RATE as usize;
        assert!(
            normalized.len().abs_diff(expected_frames) <= 16,
            "długość {} zamiast {expected_frames}",
            normalized.len()
        );
        let frequency = dominant_frequency(&normalized, TARGET_SAMPLE_RATE);
        assert!((frequency - 440.0).abs() < 5.0, "ton {frequency} Hz");
    }

    #[test]
    fn mono_44_1khz_becomes_16khz_with_same_length_and_tone() {
        let tone = sine(1000.0, 44_100, 1.5);

        let normalized = normalize(&tone, 1, 44_100).unwrap();

        let expected_frames = (1.5 * TARGET_SAMPLE_RATE as f32) as usize;
        assert!(normalized.len().abs_diff(expected_frames) <= 16);
        let frequency = dominant_frequency(&normalized, TARGET_SAMPLE_RATE);
        assert!((frequency - 1000.0).abs() < 5.0, "ton {frequency} Hz");
    }

    #[test]
    fn empty_input_gives_empty_output() {
        assert!(normalize(&[], 2, 48_000).unwrap().is_empty());
    }
}
