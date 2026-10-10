//! Nagrywanie z mikrofonu do bufora w pamięci (nic nie trafia na dysk).

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use crate::convert::normalize;
use crate::{Error, Result};

/// Wołane raz, z wątku audio, gdy bufor osiągnie limit długości (VA-REC-6) — dalsze próbki
/// są odrzucane, więc odbiorca powinien zakończyć nagranie.
pub type LimitNotifier = Box<dyn FnOnce() + Send>;

/// Nagranie z mikrofonu: start, potem stop zwracający 16 kHz mono f32.
pub trait Recorder: Send {
    fn start(&mut self, device_name: &str, on_limit: LimitNotifier) -> Result<()>;
    fn stop(&mut self) -> Result<Vec<f32>>;
    /// Limit długości dla następnych nagrań (VA-SET-1); trwające nagranie go nie zmienia.
    fn set_limit(&mut self, max_seconds: u32);
}

/// Surowe próbki w formacie urządzenia, przed normalizacją.
#[derive(Default)]
struct Captured {
    interleaved: Vec<f32>,
    channels: u16,
    sample_rate: u32,
    truncated: bool,
    on_limit: Option<LimitNotifier>,
}

struct ActiveRecording {
    stop: mpsc::Sender<()>,
    worker: JoinHandle<Result<Captured>>,
}

/// Nagrywanie przez CoreAudio. Strumień cpal nie jest `Send`, więc żyje we własnym wątku.
pub struct CpalRecorder {
    max_seconds: u32,
    active: Option<ActiveRecording>,
}

impl CpalRecorder {
    pub fn new(max_seconds: u32) -> Self {
        Self {
            max_seconds,
            active: None,
        }
    }
}

impl Recorder for CpalRecorder {
    fn set_limit(&mut self, max_seconds: u32) {
        tracing::info!(max_seconds, "limit nagrania zmieniony");
        self.max_seconds = max_seconds;
    }

    fn start(&mut self, device_name: &str, on_limit: LimitNotifier) -> Result<()> {
        if self.active.is_some() {
            return Err(Error::AlreadyRecording);
        }
        let (stop, stop_signal) = mpsc::channel();
        let (ready, started) = mpsc::channel();
        let device_name = device_name.to_owned();
        let max_seconds = self.max_seconds;
        let worker = std::thread::Builder::new()
            .name("va-audio-capture".into())
            .spawn(move || capture(&device_name, max_seconds, on_limit, &ready, &stop_signal))
            .map_err(|error| Error::Backend(error.to_string()))?;
        match started.recv() {
            Ok(Ok(())) => {
                self.active = Some(ActiveRecording { stop, worker });
                Ok(())
            }
            Ok(Err(error)) => Err(error),
            Err(_) => Err(join_error(worker)),
        }
    }

    fn stop(&mut self) -> Result<Vec<f32>> {
        let active = self.active.take().ok_or(Error::NotRecording)?;
        let _ = active.stop.send(());
        let captured = active
            .worker
            .join()
            .map_err(|_| Error::Backend("wątek nagrywania zakończył się błędem".into()))??;
        if captured.truncated {
            tracing::warn!(
                max_seconds = self.max_seconds,
                "nagranie przycięte do limitu długości"
            );
        }
        normalize(
            &captured.interleaved,
            captured.channels,
            captured.sample_rate,
        )
    }
}

fn join_error(worker: JoinHandle<Result<Captured>>) -> Error {
    match worker.join() {
        Ok(Err(error)) => error,
        _ => Error::Backend("wątek nagrywania zakończył się przed startem".into()),
    }
}

fn capture(
    device_name: &str,
    max_seconds: u32,
    on_limit: LimitNotifier,
    ready: &mpsc::Sender<Result<()>>,
    stop_signal: &mpsc::Receiver<()>,
) -> Result<Captured> {
    let buffer = Arc::new(Mutex::new(Captured {
        on_limit: Some(on_limit),
        ..Captured::default()
    }));
    let stream = match open_stream(device_name, max_seconds, &buffer) {
        Ok(stream) => stream,
        Err(error) => {
            let _ = ready.send(Err(error));
            return Ok(Captured::default());
        }
    };
    let _ = ready.send(Ok(()));
    let _ = stop_signal.recv();
    drop(stream);
    let mut captured = buffer.lock().expect("bufor nagrania");
    Ok(std::mem::take(&mut *captured))
}

fn open_stream(
    device_name: &str,
    max_seconds: u32,
    buffer: &Arc<Mutex<Captured>>,
) -> Result<cpal::Stream> {
    let host = cpal::default_host();
    let device = host
        .input_devices()
        .map_err(backend)?
        .find(|device| device.to_string() == device_name)
        .ok_or_else(|| Error::DeviceUnavailable(device_name.to_owned()))?;
    let supported = device.default_input_config().map_err(backend)?;
    let config = supported.config();
    {
        let mut captured = buffer.lock().expect("bufor nagrania");
        captured.channels = config.channels;
        captured.sample_rate = config.sample_rate;
    }
    let limit = max_seconds as usize * config.sample_rate as usize * config.channels as usize;
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, &config, limit, buffer),
        SampleFormat::I16 => build::<i16>(&device, &config, limit, buffer),
        SampleFormat::I32 => build::<i32>(&device, &config, limit, buffer),
        SampleFormat::U16 => build::<u16>(&device, &config, limit, buffer),
        other => {
            return Err(Error::Backend(format!(
                "nieobsługiwany format próbek {other}"
            )));
        }
    }?;
    stream.play().map_err(backend)?;
    tracing::info!(
        device = device_name,
        channels = config.channels,
        sample_rate = config.sample_rate,
        "nagrywanie rozpoczęte"
    );
    Ok(stream)
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    limit: usize,
    buffer: &Arc<Mutex<Captured>>,
) -> Result<cpal::Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let buffer = Arc::clone(buffer);
    device
        .build_input_stream(
            *config,
            move |data: &[T], _| append_limited(&buffer, data, limit),
            |error| tracing::error!(%error, "błąd strumienia audio"),
            None,
        )
        .map_err(backend)
}

fn append_limited<T>(buffer: &Mutex<Captured>, data: &[T], limit: usize)
where
    T: SizedSample,
    f32: FromSample<T>,
{
    let notify_limit = {
        let Ok(mut captured) = buffer.lock() else {
            return;
        };
        let room = limit.saturating_sub(captured.interleaved.len());
        let overflow = data.len() > room;
        if overflow {
            captured.truncated = true;
        }
        captured.interleaved.extend(
            data.iter()
                .take(room)
                .map(|sample| sample.to_sample::<f32>()),
        );
        if overflow {
            captured.on_limit.take()
        } else {
            None
        }
    };
    if let Some(notify) = notify_limit {
        notify();
    }
}

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_beyond_limit_are_dropped_and_flagged_and_limit_is_reported_once() {
        let notifications = Arc::new(Mutex::new(0_u32));
        let counter = Arc::clone(&notifications);
        let buffer = Mutex::new(Captured {
            on_limit: Some(Box::new(move || *counter.lock().unwrap() += 1)),
            ..Captured::default()
        });

        append_limited(&buffer, &[0.1_f32, 0.2, 0.3], 4);
        assert_eq!(*notifications.lock().unwrap(), 0);
        append_limited(&buffer, &[0.4_f32, 0.5, 0.6], 4);
        append_limited(&buffer, &[0.7_f32], 4);

        let captured = buffer.into_inner().unwrap();
        assert_eq!(captured.interleaved, vec![0.1, 0.2, 0.3, 0.4]);
        assert!(captured.truncated);
        assert_eq!(*notifications.lock().unwrap(), 1);
    }

    #[test]
    fn integer_samples_are_converted_to_float() {
        let buffer = Mutex::new(Captured::default());

        append_limited(&buffer, &[i16::MAX, 0, i16::MIN], 10);

        let captured = buffer.into_inner().unwrap();
        assert!((captured.interleaved[0] - 1.0).abs() < 0.001);
        assert_eq!(captured.interleaved[1], 0.0);
        assert_eq!(captured.interleaved[2], -1.0);
        assert!(!captured.truncated);
    }

    #[test]
    fn stop_without_start_is_an_error() {
        let mut recorder = CpalRecorder::new(5);

        assert!(matches!(recorder.stop(), Err(Error::NotRecording)));
    }

    #[test]
    fn unknown_device_fails_at_start() {
        let mut recorder = CpalRecorder::new(5);

        let error = recorder
            .start("Nie ma takiego mikrofonu", Box::new(|| {}))
            .unwrap_err();

        assert!(matches!(error, Error::DeviceUnavailable(_)), "{error:?}");
        assert!(matches!(recorder.stop(), Err(Error::NotRecording)));
    }

    #[test]
    #[ignore = "wymaga mikrofonu i zgody na dostęp do niego"]
    fn real_microphone_reports_limit_from_audio_thread() {
        use crate::{AudioHost, CpalHost, choose_device};
        let devices = CpalHost::new().input_devices().unwrap();
        let device = choose_device(&devices, None).unwrap();
        let (limit_tx, limit_rx) = mpsc::channel();
        let mut recorder = CpalRecorder::new(1);

        recorder
            .start(&device.name, Box::new(move || limit_tx.send(()).unwrap()))
            .unwrap();
        let reported = limit_rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .is_ok();
        let samples = recorder.stop().unwrap();

        assert!(reported, "wątek audio nie zgłosił limitu 1 s");
        assert!(samples.len() <= 16_000 + 1_600, "{} próbek", samples.len());
    }

    #[test]
    #[ignore = "wymaga mikrofonu i zgody na dostęp do niego"]
    fn records_from_default_microphone_as_16khz_mono() {
        use crate::{AudioHost, CpalHost, choose_device};
        let devices = CpalHost::new().input_devices().unwrap();
        let device = choose_device(&devices, None).unwrap();
        let mut recorder = CpalRecorder::new(5);

        recorder.start(&device.name, Box::new(|| {})).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(500));
        let samples = recorder.stop().unwrap();

        assert!(
            (6_000..=10_000).contains(&samples.len()),
            "{} próbek",
            samples.len()
        );
    }
}
