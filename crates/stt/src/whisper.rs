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

impl SpeechToText for WhisperStt {
    fn transcribe(&mut self, samples: &[f32]) -> Result<Transcript> {
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_language(Some(language_code(self.language)));
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_special(false);
        params.set_print_timestamps(false);
        let started = Instant::now();
        self.state
            .full(params, samples)
            .map_err(|error| Error::Inference(error.to_string()))?;
        let text = self
            .state
            .as_iter()
            .map(|segment| segment.to_str_lossy().map(|text| text.into_owned()))
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|error| Error::Inference(error.to_string()))?
            .concat();
        Ok(Transcript {
            text: text.trim().to_owned(),
            inference: started.elapsed(),
        })
    }
}

fn language_code(language: Language) -> &'static str {
    match language {
        Language::Auto => "auto",
        Language::Pl => "pl",
        Language::En => "en",
    }
}
