//! Fałszywe STT do testów: zwraca zadane teksty i zapamiętuje otrzymane nagrania.

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::{Error, Result, SpeechToText, Transcript};

/// Kolejne wywołania zwracają kolejne odpowiedzi; `Err(())` udaje błąd inferencji.
#[derive(Clone, Default)]
pub struct ScriptedStt {
    answers: Arc<Mutex<VecDeque<std::result::Result<String, ()>>>>,
    received: Arc<Mutex<Vec<Vec<f32>>>>,
}

impl ScriptedStt {
    pub fn answering(answers: impl IntoIterator<Item = std::result::Result<String, ()>>) -> Self {
        Self {
            answers: Arc::new(Mutex::new(answers.into_iter().collect())),
            received: Arc::default(),
        }
    }

    /// Nagrania, które trafiły do transkrypcji (wspólne dla klonów).
    pub fn received(&self) -> Vec<Vec<f32>> {
        self.received.lock().expect("lista nagrań").clone()
    }
}

impl SpeechToText for ScriptedStt {
    fn transcribe(&mut self, samples: &[f32]) -> Result<Transcript> {
        self.received
            .lock()
            .expect("lista nagrań")
            .push(samples.to_vec());
        match self.answers.lock().expect("odpowiedzi").pop_front() {
            Some(Ok(text)) => Ok(Transcript {
                text,
                inference: Duration::ZERO,
            }),
            Some(Err(())) => Err(Error::Inference("błąd testowy".into())),
            None => Err(Error::Inference("brak zaplanowanej odpowiedzi".into())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scripted_stt_returns_answers_in_order_and_records_input() {
        let mut stt = ScriptedStt::answering([Ok("raz".into()), Err(())]);
        let observer = stt.clone();

        assert_eq!(stt.transcribe(&[0.5]).unwrap().text, "raz");
        assert!(stt.transcribe(&[0.25]).is_err());
        assert_eq!(observer.received(), vec![vec![0.5], vec![0.25]]);
    }
}
