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

/// Normalizacja na bieżąco dla długich nagrań (VA-CONF-3): kolejne porcje z urządzenia
/// wychodzą jako 16 kHz mono bez trzymania całego nagrania w pamięci.
pub struct StreamNormalizer {
    channels: u16,
    resampler: Option<Fft<f32>>,
    pending: Vec<f32>,
    delay_to_skip: usize,
    input_frames: u64,
    output_frames: u64,
    ratio: f64,
}

impl StreamNormalizer {
    pub fn new(channels: u16, sample_rate: u32) -> Result<Self> {
        let resampler = if sample_rate == TARGET_SAMPLE_RATE {
            None
        } else {
            Some(
                Fft::<f32>::new(
                    sample_rate as usize,
                    TARGET_SAMPLE_RATE as usize,
                    RESAMPLER_CHUNK_FRAMES,
                    1,
                    FixedSync::Input,
                )
                .map_err(resampling)?,
            )
        };
        let delay_to_skip = resampler.as_ref().map_or(0, Resampler::output_delay);
        Ok(Self {
            channels,
            resampler,
            pending: Vec::new(),
            delay_to_skip,
            input_frames: 0,
            output_frames: 0,
            ratio: f64::from(TARGET_SAMPLE_RATE) / f64::from(sample_rate),
        })
    }

    /// Porcja próbek przeplatanych z urządzenia → gotowe próbki 16 kHz mono.
    pub fn push(&mut self, interleaved: &[f32]) -> Result<Vec<f32>> {
        let mono = downmix_to_mono(interleaved, self.channels);
        self.input_frames += mono.len() as u64;
        if self.resampler.is_none() {
            self.output_frames += mono.len() as u64;
            return Ok(mono);
        }
        self.pending.extend_from_slice(&mono);
        self.drain_full_chunks()
    }

    /// Reszta sygnału po ostatniej porcji; długość całości = wejście × proporcja częstotliwości.
    pub fn finish(&mut self) -> Result<Vec<f32>> {
        let expected = (self.input_frames as f64 * self.ratio).round() as u64;
        let mut output = Vec::new();
        while self.resampler.is_some() && self.output_frames < expected {
            let needed = self.next_chunk_len();
            self.pending.resize(needed.max(self.pending.len()), 0.0);
            output.extend(self.drain_full_chunks()?);
        }
        let excess = (self.output_frames.saturating_sub(expected)) as usize;
        output.truncate(output.len().saturating_sub(excess));
        self.output_frames = expected;
        Ok(output)
    }

    fn next_chunk_len(&self) -> usize {
        self.resampler
            .as_ref()
            .map_or(RESAMPLER_CHUNK_FRAMES, Resampler::input_frames_next)
    }

    fn drain_full_chunks(&mut self) -> Result<Vec<f32>> {
        let mut output = Vec::new();
        let Some(resampler) = self.resampler.as_mut() else {
            return Ok(output);
        };
        while self.pending.len() >= resampler.input_frames_next() {
            let needed = resampler.input_frames_next();
            let chunk: Vec<f32> = self.pending.drain(..needed).collect();
            let input = InterleavedSlice::new(&chunk, 1, chunk.len()).map_err(resampling)?;
            let mut produced = resampler
                .process(&input, None)
                .map_err(resampling)?
                .take_data();
            let skipped = self.delay_to_skip.min(produced.len());
            produced.drain(..skipped);
            self.delay_to_skip -= skipped;
            self.output_frames += produced.len() as u64;
            output.extend(produced);
        }
        Ok(output)
    }
}

fn resampling(error: impl std::fmt::Display) -> Error {
    Error::Resampling(error.to_string())
}

#[cfg(test)]
mod tests {
    use std::f32::consts::TAU;

    use super::*;

    #[test]
    fn streaming_normalizer_matches_whole_recording_length_and_level() {
        let stereo: Vec<f32> = sine(440.0, 44_100, 3.0)
            .into_iter()
            .flat_map(|value| [value, value])
            .collect();
        let mut normalizer = StreamNormalizer::new(2, 44_100).unwrap();

        let mut output = Vec::new();
        for chunk in stereo.chunks(2 * 441) {
            output.extend(normalizer.push(chunk).unwrap());
        }
        output.extend(normalizer.finish().unwrap());

        assert_eq!(output.len(), 48_000);
        let peak = output[2_000..46_000]
            .iter()
            .fold(0.0f32, |a, b| a.max(b.abs()));
        assert!((peak - 0.5).abs() < 0.05, "amplituda {peak}");
    }

    #[test]
    fn streaming_normalizer_at_target_rate_only_downmixes() {
        let mut normalizer = StreamNormalizer::new(2, TARGET_SAMPLE_RATE).unwrap();

        let output = normalizer.push(&[0.2, 0.4, -0.2, -0.4]).unwrap();

        assert_eq!(output.len(), 2);
        assert!((output[0] - 0.3).abs() < 1e-6 && (output[1] + 0.3).abs() < 1e-6);
        assert!(normalizer.finish().unwrap().is_empty());
    }

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
