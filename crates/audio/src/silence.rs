//! Przycinanie ciszy na początku i końcu nagrania (energetyczny VAD na ramkach 20 ms).

use std::ops::Range;

const FRAME_MS: u32 = 20;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SilenceParams {
    /// Próg RMS (0.0–1.0), poniżej którego ramka jest ciszą.
    pub threshold_rms: f32,
    /// Margines zostawiany przed pierwszą i po ostatniej ramce mowy.
    pub padding_ms: u32,
}

/// Zakres próbek od pierwszej do ostatniej ramki mowy, z marginesem; `None` = sama cisza.
pub fn speech_bounds(
    samples: &[f32],
    sample_rate: u32,
    params: &SilenceParams,
) -> Option<Range<usize>> {
    let frame_len = samples_per_ms(sample_rate, FRAME_MS).max(1);
    let is_speech = |frame: &[f32]| rms(frame) >= params.threshold_rms;
    let first_frame = samples.chunks(frame_len).position(is_speech)?;
    let last_frame = samples.chunks(frame_len).rposition(is_speech)?;
    let padding = samples_per_ms(sample_rate, params.padding_ms);
    let start = (first_frame * frame_len).saturating_sub(padding);
    let end = ((last_frame + 1) * frame_len + padding).min(samples.len());
    Some(start..end)
}

/// Nagranie bez ciszy na brzegach; sama cisza daje pusty wycinek.
pub fn trim_silence<'a>(samples: &'a [f32], sample_rate: u32, params: &SilenceParams) -> &'a [f32] {
    match speech_bounds(samples, sample_rate, params) {
        Some(bounds) => &samples[bounds],
        None => &[],
    }
}

fn rms(frame: &[f32]) -> f32 {
    (frame.iter().map(|sample| sample * sample).sum::<f32>() / frame.len() as f32).sqrt()
}

fn samples_per_ms(sample_rate: u32, milliseconds: u32) -> usize {
    (u64::from(sample_rate) * u64::from(milliseconds) / 1000) as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    const RATE: u32 = 16_000;
    const PARAMS: SilenceParams = SilenceParams {
        threshold_rms: 0.01,
        padding_ms: 100,
    };

    fn signal(silence_before: usize, loud: usize, silence_after: usize) -> Vec<f32> {
        let mut samples = vec![0.0; silence_before];
        samples.extend((0..loud).map(|index| if index % 2 == 0 { 0.3 } else { -0.3 }));
        samples.extend(vec![0.0; silence_after]);
        samples
    }

    #[test]
    fn pure_silence_has_no_speech() {
        assert_eq!(speech_bounds(&vec![0.0; 16_000], RATE, &PARAMS), None);
        assert!(trim_silence(&vec![0.0; 16_000], RATE, &PARAMS).is_empty());
    }

    #[test]
    fn empty_recording_has_no_speech() {
        assert_eq!(speech_bounds(&[], RATE, &PARAMS), None);
    }

    #[test]
    fn silence_on_both_ends_is_cut_leaving_padding() {
        let samples = signal(16_000, 8_000, 16_000);

        let bounds = speech_bounds(&samples, RATE, &PARAMS).unwrap();

        assert_eq!(bounds, 14_400..25_600);
    }

    #[test]
    fn padding_is_clamped_to_recording_edges() {
        let samples = signal(320, 3_200, 320);

        assert_eq!(speech_bounds(&samples, RATE, &PARAMS), Some(0..3_840));
    }

    #[test]
    fn signal_below_threshold_counts_as_silence() {
        let quiet = vec![0.005; 16_000];

        assert_eq!(speech_bounds(&quiet, RATE, &PARAMS), None);
    }
}
