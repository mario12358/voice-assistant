//! Przycinanie ciszy na początku i końcu nagrania oraz skracanie długich pauz w środku
//! (energetyczny VAD na ramkach 20 ms).

use std::ops::Range;

const FRAME_MS: u32 = 20;
/// Pauza wewnątrz nagrania dłuższa niż tyle jest skracana (VA-REC-5).
pub const MAX_PAUSE_MS: u32 = 1_500;
/// Tyle pauzy zostaje po skróceniu — Whisper dostaje naturalną przerwę między zdaniami.
pub const KEPT_PAUSE_MS: u32 = 500;

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

/// Kopia nagrania, w której każda pauza między fragmentami mowy dłuższa niż [`MAX_PAUSE_MS`]
/// jest skrócona do [`KEPT_PAUSE_MS`]. Mowa, krótsze pauzy i cisza na brzegach zostają
/// bajt w bajt (brzegi to robota [`trim_silence`]).
pub fn compress_pauses(samples: &[f32], sample_rate: u32, params: &SilenceParams) -> Vec<f32> {
    let frame_len = samples_per_ms(sample_rate, FRAME_MS).max(1);
    let frames: Vec<&[f32]> = samples.chunks(frame_len).collect();
    let is_speech = |frame: &&[f32]| rms(frame) >= params.threshold_rms;
    let (Some(first_speech), Some(last_speech)) = (
        frames.iter().position(is_speech),
        frames.iter().rposition(is_speech),
    ) else {
        return samples.to_vec();
    };
    let max_pause_frames = (MAX_PAUSE_MS / FRAME_MS) as usize;
    let kept_pause_frames = (KEPT_PAUSE_MS / FRAME_MS) as usize;

    let mut output = Vec::with_capacity(samples.len());
    let mut index = 0;
    while index < frames.len() {
        let inside_speech = index > first_speech && index < last_speech;
        if !inside_speech || is_speech(&frames[index]) {
            output.extend_from_slice(frames[index]);
            index += 1;
            continue;
        }
        let pause_end = (index..last_speech)
            .find(|candidate| is_speech(&frames[*candidate]))
            .unwrap_or(last_speech);
        let pause_frames = pause_end - index;
        let kept = if pause_frames > max_pause_frames {
            kept_pause_frames
        } else {
            pause_frames
        };
        for frame in &frames[index..index + kept] {
            output.extend_from_slice(frame);
        }
        index = pause_end;
    }
    output
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

    fn speech(samples: usize) -> Vec<f32> {
        (0..samples)
            .map(|index| if index % 2 == 0 { 0.3 } else { -0.3 })
            .collect()
    }

    fn seconds(value: f32) -> usize {
        (value * RATE as f32) as usize
    }

    fn with_pause(pause_s: f32) -> (Vec<f32>, Vec<f32>, Vec<f32>) {
        let first = speech(seconds(1.0));
        let second = speech(seconds(0.7));
        let mut samples = first.clone();
        samples.extend(vec![0.0; seconds(pause_s)]);
        samples.extend(&second);
        (samples, first, second)
    }

    #[test]
    // specky: crit 01M4K06AR8A386FPF759VJJ9M2
    fn pause_longer_than_limit_is_shortened_to_half_second() {
        let (samples, first, second) = with_pause(40.0);

        let compressed = compress_pauses(&samples, RATE, &PARAMS);

        assert_eq!(compressed.len(), first.len() + seconds(0.5) + second.len());
        assert_eq!(&compressed[..first.len()], &first[..]);
        assert_eq!(&compressed[compressed.len() - second.len()..], &second[..]);
        assert!(
            compressed[first.len()..first.len() + seconds(0.5)]
                .iter()
                .all(|sample| *sample == 0.0)
        );
    }

    #[test]
    // specky: crit 01M4K06AR8A4107QK83WR0K6YC
    fn pause_within_limit_and_speech_stay_untouched() {
        let (samples, _, _) = with_pause(1.0);

        assert_eq!(compress_pauses(&samples, RATE, &PARAMS), samples);
    }

    #[test]
    fn pause_exactly_at_limit_stays_untouched() {
        let (samples, _, _) = with_pause(1.5);

        assert_eq!(compress_pauses(&samples, RATE, &PARAMS), samples);
    }

    #[test]
    fn every_long_pause_is_shortened() {
        let (mut samples, first, second) = with_pause(3.0);
        let third = speech(seconds(0.4));
        samples.extend(vec![0.0; seconds(10.0)]);
        samples.extend(&third);

        let compressed = compress_pauses(&samples, RATE, &PARAMS);

        assert_eq!(
            compressed.len(),
            first.len() + seconds(0.5) + second.len() + seconds(0.5) + third.len()
        );
    }

    #[test]
    // specky: crit 01M4K06AR8VG4GQ4HY90VTX40G
    fn edges_are_left_for_trim_silence() {
        let mut samples = vec![0.0; seconds(5.0)];
        let (middle, _, _) = with_pause(0.5);
        samples.extend(&middle);
        samples.extend(vec![0.0; seconds(5.0)]);

        assert_eq!(compress_pauses(&samples, RATE, &PARAMS), samples);
        assert_eq!(
            trim_silence(&samples, RATE, &PARAMS).len(),
            middle.len() + 2 * samples_per_ms(RATE, PARAMS.padding_ms)
        );
    }

    #[test]
    fn pure_silence_is_returned_unchanged() {
        let silence = vec![0.0; seconds(3.0)];

        assert_eq!(compress_pauses(&silence, RATE, &PARAMS), silence);
        assert!(compress_pauses(&[], RATE, &PARAMS).is_empty());
    }
}
