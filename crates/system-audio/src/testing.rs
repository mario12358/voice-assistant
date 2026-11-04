//! Atrapa źródła dźwięku systemowego: zadana zgoda, a po starcie wysyła zadane próbki.

use std::sync::{Arc, Mutex};

use crate::{Authorization, CaptureFormat, Result, SampleSink, SystemAudioSource};

#[derive(Default)]
struct Shared {
    starts: usize,
    stops: usize,
    sink: Option<SampleSink>,
}

/// Klony dzielą stan: test trzyma jeden, kontroler dostaje drugi.
#[derive(Clone)]
pub struct FakeSystemAudio {
    authorization: Authorization,
    answer_on_request: Authorization,
    format: CaptureFormat,
    shared: Arc<Mutex<Shared>>,
}

impl FakeSystemAudio {
    /// Zgoda udzielona, format 16 kHz mono (próbki trafiają do ścieżki bez przeliczania).
    pub fn granted() -> Self {
        Self::with_authorization(Authorization::Granted, Authorization::Granted)
    }

    pub fn with_authorization(current: Authorization, answer_on_request: Authorization) -> Self {
        Self {
            authorization: current,
            answer_on_request,
            format: CaptureFormat {
                sample_rate: 16_000,
                channels: 1,
            },
            shared: Arc::default(),
        }
    }

    /// Przekazuje próbki do odbiorcy działającego przechwytywania (jak wątek Core Audio).
    pub fn play(&self, samples: &[f32]) {
        if let Some(sink) = self.shared.lock().expect("stan atrapy").sink.as_mut() {
            sink(samples);
        }
    }

    pub fn starts(&self) -> usize {
        self.shared.lock().expect("stan atrapy").starts
    }

    pub fn stops(&self) -> usize {
        self.shared.lock().expect("stan atrapy").stops
    }
}

impl SystemAudioSource for FakeSystemAudio {
    fn authorization(&self) -> Authorization {
        self.authorization
    }

    fn request_authorization(&mut self) -> Authorization {
        if self.authorization == Authorization::Undetermined {
            self.authorization = self.answer_on_request;
        }
        self.authorization
    }

    fn start(&mut self, sink: SampleSink) -> Result<CaptureFormat> {
        let mut shared = self.shared.lock().expect("stan atrapy");
        shared.starts += 1;
        shared.sink = Some(sink);
        Ok(self.format)
    }

    fn stop(&mut self) {
        let mut shared = self.shared.lock().expect("stan atrapy");
        shared.stops += 1;
        shared.sink = None;
    }
}
