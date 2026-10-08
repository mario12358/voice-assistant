//! Przycinanie ciszy na nagraniach mowy (fixtures z tests/fixtures/generate_silence_fixtures.py).

use std::ops::Range;
use std::path::PathBuf;

use va_audio::{SilenceParams, TARGET_SAMPLE_RATE, speech_bounds, trim_silence};

/// Wartości domyślne z `va_config::SilenceConfig`.
const DEFAULT_PARAMS: SilenceParams = SilenceParams {
    threshold_rms: 0.01,
    padding_ms: 200,
};
const PADDING_SAMPLES: usize = 3_200;
const SPEECH_START: usize = 16_000;
const SPEECH_END: usize = 61_078;
/// Tolerancja granicy: ramka 20 ms plus łagodne narastanie głosu syntezatora.
const TOLERANCE: usize = 800;

fn load_fixture(name: &str) -> Vec<f32> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures")
        .join(name);
    let mut reader = hound::WavReader::open(&path).expect("brak fixture WAV");
    assert_eq!(reader.spec().sample_rate, TARGET_SAMPLE_RATE);
    assert_eq!(reader.spec().channels, 1);
    reader
        .samples::<i16>()
        .map(|sample| f32::from(sample.unwrap()) / 32_768.0)
        .collect()
}

fn assert_near(actual: Range<usize>, expected: Range<usize>) {
    assert!(
        actual.start.abs_diff(expected.start) <= TOLERANCE
            && actual.end.abs_diff(expected.end) <= TOLERANCE,
        "granice {actual:?}, oczekiwane około {expected:?}"
    );
}

#[test]
fn silence_only_recording_yields_empty_result() {
    let samples = load_fixture("silence_only.wav");

    assert_eq!(
        speech_bounds(&samples, TARGET_SAMPLE_RATE, &DEFAULT_PARAMS),
        None
    );
    assert!(trim_silence(&samples, TARGET_SAMPLE_RATE, &DEFAULT_PARAMS).is_empty());
}

#[test]
fn speech_surrounded_by_silence_is_cut_to_speech_with_padding() {
    let samples = load_fixture("speech_with_silence.wav");

    let bounds = speech_bounds(&samples, TARGET_SAMPLE_RATE, &DEFAULT_PARAMS).unwrap();

    assert_near(
        bounds,
        SPEECH_START - PADDING_SAMPLES..SPEECH_END + PADDING_SAMPLES,
    );
}

#[test]
fn speech_without_silence_is_kept_whole() {
    let samples = load_fixture("speech_only.wav");

    let trimmed = trim_silence(&samples, TARGET_SAMPLE_RATE, &DEFAULT_PARAMS);

    assert_eq!(trimmed.len(), samples.len());
}
