//! Nagranie spotkania (VA-CONF-1, VA-CONF-3): mikrofon i dźwięk systemowy jednocześnie, każda
//! ścieżka normalizowana na bieżąco do 16 kHz mono i dopisywana do własnego pliku tymczasowego.

use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use va_audio::{
    LimitNotifier, StreamNormalizer, StreamSink, StreamingRecorder, TARGET_SAMPLE_RATE,
};
use va_system_audio::{Authorization, SystemAudioSource};

use crate::spool::{SpoolWriter, SpooledAudio};

/// Elementy trybu spotkania wstrzykiwane do kontrolera.
pub struct MeetingParts {
    pub microphone: Box<dyn StreamingRecorder>,
    pub system: Box<dyn SystemAudioSource>,
    /// Katalog plików tymczasowych (katalog danych aplikacji).
    pub spool_dir: PathBuf,
    /// Limit długości nagrania spotkania w sekundach (`max_meeting_secs`).
    pub max_seconds: u32,
    /// Etykiety „Ja:” / „Rozmówcy:” w wyniku (`meeting_speaker_labels`).
    pub speaker_labels: bool,
}

impl MeetingParts {
    /// Zgoda na dźwięk systemowy; przy pierwszym użyciu pokazuje pytanie systemu.
    pub fn authorize(&mut self) -> Authorization {
        self.system.request_authorization()
    }
}

/// Obie ścieżki zamknięte po Stop, gotowe do transkrypcji (pliki znikają po zniszczeniu).
pub struct MeetingAudio {
    pub microphone: SpooledAudio,
    pub system: SpooledAudio,
}

/// Trwające nagranie spotkania.
pub struct MeetingRecording {
    microphone: SharedTrack,
    system: SharedTrack,
}

impl MeetingRecording {
    /// Otwiera pliki obu ścieżek i uruchamia mikrofon oraz dźwięk systemowy. `on_limit` jest
    /// wołane raz, gdy ścieżka mikrofonu (zegar nagrania) dojdzie do limitu.
    pub fn start(
        parts: &mut MeetingParts,
        device: &str,
        on_limit: LimitNotifier,
    ) -> Result<Self, String> {
        let limit = u64::from(parts.max_seconds) * u64::from(TARGET_SAMPLE_RATE);
        let microphone = Track::shared(&parts.spool_dir, "mic", limit, Some(on_limit))?;
        let system = Track::shared(&parts.spool_dir, "system", limit, None)?;
        let format = parts
            .microphone
            .start(device, sink(&microphone))
            .map_err(|error| error.to_string())?;
        lock(&microphone).set_format(format.channels, format.sample_rate);
        match parts.system.start(sink(&system)) {
            Ok(format) => lock(&system).set_format(format.channels, format.sample_rate),
            Err(error) => {
                parts.microphone.stop();
                return Err(error.to_string());
            }
        }
        tracing::info!(
            max_seconds = parts.max_seconds,
            "nagrywanie spotkania rozpoczęte"
        );
        Ok(Self { microphone, system })
    }

    /// Zatrzymuje oba źródła i domyka pliki.
    pub fn stop(self, parts: &mut MeetingParts) -> Result<MeetingAudio, String> {
        parts.microphone.stop();
        parts.system.stop();
        let microphone = lock(&self.microphone).finish();
        let system = lock(&self.system).finish();
        Ok(MeetingAudio {
            microphone: microphone?,
            system: system?,
        })
    }
}

type SharedTrack = Arc<Mutex<Track>>;

fn lock(track: &SharedTrack) -> std::sync::MutexGuard<'_, Track> {
    track
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

fn sink(track: &SharedTrack) -> StreamSink {
    let track = Arc::clone(track);
    Box::new(move |samples| lock(&track).push(samples))
}

/// Jedna ścieżka: porcje z urządzenia → normalizacja → plik, do limitu długości.
struct Track {
    normalizer: Option<StreamNormalizer>,
    /// Porcje, które przyszły, zanim źródło podało swój format.
    pending: Vec<f32>,
    writer: Option<SpoolWriter>,
    limit: u64,
    on_limit: Option<LimitNotifier>,
    error: Option<String>,
}

impl Track {
    fn shared(
        dir: &std::path::Path,
        name: &str,
        limit: u64,
        on_limit: Option<LimitNotifier>,
    ) -> Result<SharedTrack, String> {
        let writer = SpoolWriter::create(dir, name).map_err(|error| error.to_string())?;
        Ok(Arc::new(Mutex::new(Self {
            normalizer: None,
            pending: Vec::new(),
            writer: Some(writer),
            limit,
            on_limit,
            error: None,
        })))
    }

    fn set_format(&mut self, channels: u16, sample_rate: u32) {
        match StreamNormalizer::new(channels, sample_rate) {
            Ok(normalizer) => self.normalizer = Some(normalizer),
            Err(error) => self.error = Some(error.to_string()),
        }
        let pending = std::mem::take(&mut self.pending);
        self.push(&pending);
    }

    fn push(&mut self, samples: &[f32]) {
        if self.error.is_some() || samples.is_empty() {
            return;
        }
        let Some(normalizer) = self.normalizer.as_mut() else {
            self.pending.extend_from_slice(samples);
            return;
        };
        let normalized = normalizer.push(samples);
        self.write(normalized);
    }

    fn write(&mut self, normalized: va_audio::Result<Vec<f32>>) {
        let samples = match normalized {
            Ok(samples) => samples,
            Err(error) => {
                self.error = Some(error.to_string());
                return;
            }
        };
        let Some(writer) = self.writer.as_mut() else {
            return;
        };
        let room =
            usize::try_from(self.limit.saturating_sub(writer.samples())).unwrap_or(usize::MAX);
        let kept = &samples[..samples.len().min(room)];
        if let Err(error) = writer.append(kept) {
            self.error = Some(error.to_string());
            return;
        }
        if samples.len() >= room
            && let Some(notify) = self.on_limit.take()
        {
            notify();
        }
    }

    fn finish(&mut self) -> Result<SpooledAudio, String> {
        if let Some(mut normalizer) = self.normalizer.take() {
            let rest = normalizer.finish();
            self.write(rest);
        }
        if let Some(error) = self.error.take() {
            return Err(error);
        }
        self.writer
            .take()
            .ok_or_else(|| "ścieżka spotkania już zamknięta".to_owned())?
            .finish()
            .map_err(|error| error.to_string())
    }
}
