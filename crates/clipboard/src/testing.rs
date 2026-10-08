//! Schowek w pamięci do testów: stan wspólny dla klonów, licznik zapisów, tryb awarii.

use std::sync::{Arc, Mutex};

use crate::{Clipboard, Error, Result};

#[derive(Debug, Default)]
struct State {
    contents: Option<String>,
    writes: usize,
    failing: bool,
}

#[derive(Clone, Default)]
pub struct MemoryClipboard(Arc<Mutex<State>>);

impl MemoryClipboard {
    pub fn containing(text: &str) -> Self {
        let clipboard = Self::default();
        clipboard.state().contents = Some(text.to_owned());
        clipboard
    }

    pub fn failing() -> Self {
        let clipboard = Self::default();
        clipboard.state().failing = true;
        clipboard
    }

    pub fn contents(&self) -> Option<String> {
        self.state().contents.clone()
    }

    pub fn writes(&self) -> usize {
        self.state().writes
    }

    fn state(&self) -> std::sync::MutexGuard<'_, State> {
        self.0.lock().expect("stan schowka")
    }
}

impl Clipboard for MemoryClipboard {
    fn set_text(&mut self, text: &str) -> Result<()> {
        let mut state = self.state();
        if state.failing {
            return Err(Error::Clipboard("atrapa w trybie awarii".into()));
        }
        state.contents = Some(text.to_owned());
        state.writes += 1;
        Ok(())
    }
}
