//! Zapis transkrypcji do schowka systemowego.
//!
//! Reguła VA-REC-3: tekst transkrypcji zastępuje zawartość schowka, a pusta transkrypcja
//! (sama cisza) zostawia schowek nietknięty. Reguła żyje w [`ClipboardSink`], więc obowiązuje
//! niezależnie od tego, czy pod spodem jest prawdziwy schowek, czy atrapa w testach.

#[cfg(any(test, feature = "testing"))]
pub mod testing;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("nie udało się zapisać tekstu do schowka: {0}")]
    Clipboard(String),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Co stało się z tekstem transkrypcji.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Delivery {
    Written,
    /// Pusta transkrypcja — schowek bez zmian.
    SkippedEmpty,
}

/// Odbiorca tekstu transkrypcji.
pub trait TextSink: Send {
    fn deliver(&mut self, text: &str) -> Result<Delivery>;
}

/// Surowy dostęp do schowka (prawdziwy albo atrapa).
pub trait Clipboard: Send {
    fn set_text(&mut self, text: &str) -> Result<()>;
}

/// [`TextSink`] zapisujący do schowka zgodnie z regułą VA-REC-3.
pub struct ClipboardSink<C: Clipboard> {
    clipboard: C,
}

impl<C: Clipboard> ClipboardSink<C> {
    pub fn new(clipboard: C) -> Self {
        Self { clipboard }
    }
}

impl<C: Clipboard> TextSink for ClipboardSink<C> {
    fn deliver(&mut self, text: &str) -> Result<Delivery> {
        if text.trim().is_empty() {
            tracing::info!("pusta transkrypcja — schowek bez zmian");
            return Ok(Delivery::SkippedEmpty);
        }
        self.clipboard.set_text(text)?;
        tracing::info!(chars = text.chars().count(), "transkrypcja w schowku");
        Ok(Delivery::Written)
    }
}

/// Schowek systemowy macOS (NSPasteboard przez arboard).
#[derive(Default)]
pub struct SystemClipboard;

impl Clipboard for SystemClipboard {
    fn set_text(&mut self, text: &str) -> Result<()> {
        arboard::Clipboard::new()
            .and_then(|mut clipboard| clipboard.set_text(text))
            .map_err(|error| Error::Clipboard(error.to_string()))
    }
}

/// Schowek systemowy z regułą VA-REC-3 — to wstrzykuje aplikacja.
pub fn system_sink() -> ClipboardSink<SystemClipboard> {
    ClipboardSink::new(SystemClipboard)
}

#[cfg(test)]
mod tests {
    use super::testing::MemoryClipboard;
    use super::*;

    #[test]
    // specky: crit 01M4EKHCXWFDWPC9PPCBFYR3XA
    fn transcript_lands_in_clipboard_exactly_with_polish_letters() {
        let clipboard = MemoryClipboard::containing("stara treść");
        let mut sink = ClipboardSink::new(clipboard.clone());

        let delivery = sink.deliver("Zażółć gęślą jaźń, 2026 r.").unwrap();

        assert_eq!(delivery, Delivery::Written);
        assert_eq!(
            clipboard.contents().as_deref(),
            Some("Zażółć gęślą jaźń, 2026 r.")
        );
    }

    #[test]
    // specky: crit 01M4EKHCXWKZQHG9CHNYWY9VX7
    fn next_transcript_replaces_previous_one() {
        let clipboard = MemoryClipboard::default();
        let mut sink = ClipboardSink::new(clipboard.clone());

        sink.deliver("pierwsza").unwrap();
        sink.deliver("druga").unwrap();

        assert_eq!(clipboard.contents().as_deref(), Some("druga"));
    }

    #[test]
    // specky: crit 01M4EKHCXW1S92G9Y6AS115MW9
    fn empty_transcript_leaves_clipboard_untouched() {
        let clipboard = MemoryClipboard::containing("to zostaje");
        let mut sink = ClipboardSink::new(clipboard.clone());

        assert_eq!(sink.deliver("").unwrap(), Delivery::SkippedEmpty);
        assert_eq!(sink.deliver("  \n").unwrap(), Delivery::SkippedEmpty);
        assert_eq!(clipboard.contents().as_deref(), Some("to zostaje"));
        assert_eq!(clipboard.writes(), 0);
    }

    #[test]
    fn clipboard_failure_is_reported() {
        let mut sink = ClipboardSink::new(MemoryClipboard::failing());

        let error = sink.deliver("tekst").unwrap_err();

        assert!(error.to_string().contains("schowka"), "{error}");
    }

    #[test]
    #[ignore = "wymaga sesji graficznej macOS (zmienia prawdziwy schowek)"]
    fn system_clipboard_holds_transcript_for_cmd_v() {
        let mut sink = system_sink();

        sink.deliver("VoiceAsystent: test schowka ąę").unwrap();

        let pasted = arboard::Clipboard::new().unwrap().get_text().unwrap();
        assert_eq!(pasted, "VoiceAsystent: test schowka ąę");
    }
}
