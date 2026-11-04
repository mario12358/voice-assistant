//! Nagrywanie z mikrofonu porcjami do odbiorcy (tryb spotkania, VA-CONF-3): nagranie nie jest
//! trzymane w pamięci, odbiorca dostaje próbki w formacie urządzenia, gdy tylko nadejdą.

use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample};

use crate::{Error, Result};

/// Odbiorca porcji: próbki f32 przeplatane w formacie urządzenia, wołany z wątku audio.
pub type StreamSink = Box<dyn FnMut(&[f32]) + Send>;

/// Format strumienia mikrofonu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StreamFormat {
    pub sample_rate: u32,
    pub channels: u16,
}

/// Mikrofon strumieniowo: start z odbiorcą porcji, stop kończy strumień.
pub trait StreamingRecorder: Send {
    fn start(&mut self, device_name: &str, sink: StreamSink) -> Result<StreamFormat>;
    fn stop(&mut self);
}

struct ActiveStream {
    stop: mpsc::Sender<()>,
    worker: JoinHandle<()>,
}

/// Strumień cpal nie jest `Send`, więc żyje we własnym wątku do sygnału stopu.
#[derive(Default)]
pub struct CpalStreamRecorder {
    active: Option<ActiveStream>,
}

impl StreamingRecorder for CpalStreamRecorder {
    fn start(&mut self, device_name: &str, sink: StreamSink) -> Result<StreamFormat> {
        if self.active.is_some() {
            return Err(Error::Backend("nagrywanie mikrofonu już trwa".into()));
        }
        let (ready, started) = mpsc::channel();
        let (stop, stop_signal) = mpsc::channel::<()>();
        let device_name = device_name.to_owned();
        let worker = std::thread::Builder::new()
            .name("va-meeting-mic".into())
            .spawn(move || match open_stream(&device_name, sink) {
                Ok((stream, format)) => {
                    let _ = ready.send(Ok(format));
                    let _ = stop_signal.recv();
                    drop(stream);
                }
                Err(error) => {
                    let _ = ready.send(Err(error));
                }
            })
            .map_err(backend)?;
        match started.recv() {
            Ok(Ok(format)) => {
                self.active = Some(ActiveStream { stop, worker });
                Ok(format)
            }
            Ok(Err(error)) => {
                let _ = worker.join();
                Err(error)
            }
            Err(_) => Err(Error::Backend(
                "wątek nagrywania zakończył się przed startem".into(),
            )),
        }
    }

    fn stop(&mut self) {
        if let Some(active) = self.active.take() {
            let _ = active.stop.send(());
            let _ = active.worker.join();
        }
    }
}

fn open_stream(device_name: &str, sink: StreamSink) -> Result<(cpal::Stream, StreamFormat)> {
    let device = cpal::default_host()
        .input_devices()
        .map_err(backend)?
        .find(|device| device.to_string() == device_name)
        .ok_or_else(|| Error::DeviceUnavailable(device_name.to_owned()))?;
    let supported = device.default_input_config().map_err(backend)?;
    let config = supported.config();
    let sink = Arc::new(Mutex::new(sink));
    let stream = match supported.sample_format() {
        SampleFormat::F32 => build::<f32>(&device, &config, sink),
        SampleFormat::I16 => build::<i16>(&device, &config, sink),
        SampleFormat::I32 => build::<i32>(&device, &config, sink),
        SampleFormat::U16 => build::<u16>(&device, &config, sink),
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
        "nagrywanie mikrofonu (spotkanie) rozpoczęte"
    );
    let format = StreamFormat {
        sample_rate: config.sample_rate,
        channels: config.channels,
    };
    Ok((stream, format))
}

fn build<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sink: Arc<Mutex<StreamSink>>,
) -> Result<cpal::Stream>
where
    T: SizedSample,
    f32: FromSample<T>,
{
    device
        .build_input_stream(
            *config,
            move |data: &[T], _| {
                let samples: Vec<f32> = data.iter().map(|sample| sample.to_sample()).collect();
                if let Ok(mut sink) = sink.lock() {
                    sink(&samples);
                }
            },
            |error| tracing::error!(%error, "błąd strumienia audio"),
            None,
        )
        .map_err(backend)
}

fn backend(error: impl std::fmt::Display) -> Error {
    Error::Backend(error.to_string())
}
