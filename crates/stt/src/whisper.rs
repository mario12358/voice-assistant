//! Whisper (whisper.cpp przez whisper-rs) z backendem Metal; model ładowany raz.

use std::path::Path;
use std::time::Instant;

use va_config::Language;
use whisper_rs::{
    FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState,
};

use crate::{Error, GpuReady, Result, SpeechToText, Transcript};

pub struct WhisperStt {
    // Kontekst trzyma wagi modelu; stan z niego utworzony musi go przeżyć.
    _context: WhisperContext,
    state: WhisperState,
    language: Language,
}

impl WhisperStt {
    /// Ładuje model na GPU. `GpuReady` pochodzi z [`crate::require_metal`] — bez niego nie ma CPU.
    pub fn load(model_path: &Path, gpu: &GpuReady, language: Language) -> Result<Self> {
        whisper_rs::install_logging_hooks();
        let mut parameters = WhisperContextParameters::default();
        parameters.use_gpu(true);
        let started = Instant::now();
        let context = WhisperContext::new_with_params(model_path, parameters)
            .map_err(|error| Error::ModelLoad(error.to_string()))?;
        let state = context
            .create_state()
            .map_err(|error| Error::ModelLoad(error.to_string()))?;
        tracing::info!(
            backend = "Metal",
            device = %gpu.device_name,
            model = %model_path.display(),
            load_ms = started.elapsed().as_millis() as u64,
            "model Whisper załadowany"
        );
        Ok(Self {
            _context: context,
            state,
            language,
        })
    }
}

/// Parametry dekodowania przeciw halucynacjom (VA-STT-3) — jedyne miejsce, gdzie się je ustawia.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DecodingSettings {
    /// Segment z prawdopodobieństwem „brak mowy” powyżej progu jest odrzucany po dekodowaniu.
    /// whisper.cpp sam stosuje ten próg tylko razem z fallbackiem temperatury, który wyłączamy,
    /// więc filtrujemy segmenty sami (szum bez mowy → pusty tekst zamiast wymyślonej frazy).
    pub no_speech_threshold: f32,
    /// Zakaz pustego tekstu na początku okna — model nie „dopełnia” ciszy białymi znakami.
    pub suppress_blank: bool,
    /// Zakaz tokenów niebędących mową (muzyka, „[oklaski]”, „♪”, napisy w nawiasach).
    pub suppress_non_speech_tokens: bool,
    /// Temperatura 0 = dekodowanie deterministyczne; fallback (inc 0) wyłączony, bo przy
    /// wyższych temperaturach model w oknach bez mowy chętnie zmyśla zdania.
    pub temperature: f32,
    pub temperature_increment: f32,
    /// Bez warunkowania na tekście poprzedniego okna — przerywa pętle powtórzeń.
    pub no_context: bool,
    /// Segment o średnim prawdopodobieństwie tokenów tekstu poniżej progu jest odrzucany.
    /// large-v3-turbo zgłasza „brak mowy” ≈ 0 także dla szumu, ale zmyślone słowo ma niską
    /// pewność: szum 6 s → „so” z 0,05, prawdziwa mowa w fixtures 0,90–1,00 (pomiar 2026-10-10).
    pub min_segment_confidence: f32,
}

pub const DECODING: DecodingSettings = DecodingSettings {
    no_speech_threshold: 0.6,
    suppress_blank: true,
    suppress_non_speech_tokens: true,
    temperature: 0.0,
    temperature_increment: 0.0,
    no_context: true,
    min_segment_confidence: 0.3,
};

/// Średnia pewność tokenów tekstu segmentu; tokeny specjalne (`[_…]`, `<|…|>`) pomijane.
/// Segment bez tokenów tekstu ma pewność 0 — i tak nie wnosi tekstu.
fn text_token_confidence(segment: &whisper_rs::WhisperSegment<'_>) -> f32 {
    let probabilities: Vec<f32> = (0..segment.n_tokens())
        .filter_map(|index| segment.get_token(index))
        .filter(|token| {
            token
                .to_str_lossy()
                .is_ok_and(|text| !text.starts_with("[_") && !text.starts_with("<|"))
        })
        .map(|token| token.token_probability())
        .collect();
    if probabilities.is_empty() {
        return 0.0;
    }
    probabilities.iter().sum::<f32>() / probabilities.len() as f32
}

fn full_params(language: Language, settings: &DecodingSettings) -> FullParams<'static, 'static> {
    let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
    params.set_language(Some(language_code(language)));
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_special(false);
    params.set_print_timestamps(false);
    params.set_suppress_blank(settings.suppress_blank);
    params.set_suppress_nst(settings.suppress_non_speech_tokens);
    params.set_temperature(settings.temperature);
    params.set_temperature_inc(settings.temperature_increment);
    params.set_no_speech_thold(settings.no_speech_threshold);
    params.set_no_context(settings.no_context);
    params
}

impl SpeechToText for WhisperStt {
    fn transcribe(&mut self, samples: &[f32]) -> Result<Transcript> {
        let params = full_params(self.language, &DECODING);
        let started = Instant::now();
        self.state
            .full(params, samples)
            .map_err(|error| Error::Inference(error.to_string()))?;
        let mut kept = Vec::new();
        let mut dropped = 0;
        for segment in self.state.as_iter() {
            let confidence = text_token_confidence(&segment);
            let no_speech = segment.no_speech_probability() > DECODING.no_speech_threshold;
            if no_speech || confidence < DECODING.min_segment_confidence {
                dropped += 1;
                continue;
            }
            let text = segment
                .to_str_lossy()
                .map_err(|error| Error::Inference(error.to_string()))?;
            kept.push(text.into_owned());
        }
        if dropped > 0 {
            tracing::info!(dropped, "odrzucone segmenty bez mowy");
        }
        Ok(Transcript {
            text: kept.concat().trim().to_owned(),
            inference: started.elapsed(),
        })
    }

    fn set_language(&mut self, language: Language) {
        tracing::info!(?language, "język transkrypcji zmieniony");
        self.language = language;
    }
}

fn language_code(language: Language) -> &'static str {
    match language {
        Language::Auto => "auto",
        Language::Pl => "pl",
        Language::En => "en",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // specky: crit 01M4KD15J5VKASK15NWEJCGJYS
    fn decoding_settings_limit_hallucinations() {
        let settings = std::hint::black_box(DECODING);

        assert!(settings.no_speech_threshold > 0.0 && settings.no_speech_threshold < 1.0);
        assert!(settings.suppress_blank && settings.suppress_non_speech_tokens);
        assert_eq!(settings.temperature, 0.0);
        assert_eq!(
            settings.temperature_increment, 0.0,
            "fallback temperatury wyłączony"
        );
        assert!(settings.no_context);
        assert!(settings.min_segment_confidence > 0.0 && settings.min_segment_confidence < 0.9);
    }

    #[test]
    fn language_codes_match_whisper() {
        assert_eq!(language_code(Language::Pl), "pl");
        assert_eq!(language_code(Language::En), "en");
        assert_eq!(language_code(Language::Auto), "auto");
    }
}
