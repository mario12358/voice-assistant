//! Core Audio process tap: globalny tap stereo (bez wyciszania odtwarzania) w prywatnym
//! urządzeniu zagregowanym, czytany blokiem IO. Prywatne urządzenie nie pojawia się na liście
//! urządzeń audio, a wyjście dźwięku użytkownika zostaje bez zmian.
#![allow(unsafe_code)] // FFI Core Audio — jedyne miejsce z unsafe (PROJECT_CONTEXT, 2026-11-04)

use std::ffi::CStr;
use std::ptr::{self, NonNull};
use std::sync::Mutex;

use block2::RcBlock;
use objc2::AnyThread;
use objc2::rc::Retained;
use objc2_core_audio::{
    AudioDeviceCreateIOProcIDWithBlock, AudioDeviceDestroyIOProcID, AudioDeviceIOProcID,
    AudioDeviceStart, AudioDeviceStop, AudioHardwareCreateAggregateDevice,
    AudioHardwareCreateProcessTap, AudioHardwareDestroyAggregateDevice,
    AudioHardwareDestroyProcessTap, AudioObjectGetPropertyData, AudioObjectID,
    AudioObjectPropertyAddress, CATapDescription, CATapMuteBehavior,
    kAudioAggregateDeviceIsPrivateKey, kAudioAggregateDeviceIsStackedKey,
    kAudioAggregateDeviceMainSubDeviceKey, kAudioAggregateDeviceNameKey,
    kAudioAggregateDeviceSubDeviceListKey, kAudioAggregateDeviceTapAutoStartKey,
    kAudioAggregateDeviceTapListKey, kAudioAggregateDeviceUIDKey, kAudioDevicePropertyDeviceUID,
    kAudioHardwarePropertyDefaultSystemOutputDevice, kAudioObjectPropertyElementMain,
    kAudioObjectPropertyScopeGlobal, kAudioObjectSystemObject, kAudioSubDeviceUIDKey,
    kAudioSubTapDriftCompensationKey, kAudioSubTapUIDKey, kAudioTapPropertyFormat,
};
use objc2_core_audio_types::{
    AudioBuffer, AudioBufferList, AudioStreamBasicDescription, AudioTimeStamp,
    kAudioFormatFlagIsFloat, kAudioFormatFlagIsNonInterleaved, kAudioFormatLinearPCM,
};
use objc2_core_foundation::{CFDictionary, CFRetained, CFString};
use objc2_foundation::{NSArray, NSDictionary, NSNumber, NSObject, NSString, NSUUID};

use crate::{CaptureFormat, Error, Result, SampleSink};

const NO_ERROR: i32 = 0;

/// Działające przechwytywanie; zniszczenie zatrzymuje IO i usuwa tap oraz urządzenie.
pub struct ProcessTap {
    tap: AudioObjectID,
    device: AudioObjectID,
    io_proc: AudioDeviceIOProcID,
    format: CaptureFormat,
    /// Blok IO trzymany do końca przechwytywania (Core Audio woła własną kopię).
    _block: RcBlock<IoFn>,
}

type IoFn = dyn Fn(
    NonNull<AudioTimeStamp>,
    NonNull<AudioBufferList>,
    NonNull<AudioTimeStamp>,
    NonNull<AudioBufferList>,
    NonNull<AudioTimeStamp>,
);

// SAFETY: identyfikatory obiektów Core Audio to liczby, a `io_proc` to wskaźnik funkcji
// zwrócony przez system — używane tylko do zatrzymania i sprzątania z dowolnego wątku. Blok
// jest tylko trzymany (nie wołany ani klonowany) do końca życia `ProcessTap`; Core Audio woła
// własną kopię na swoim wątku, a stan bloku to `Mutex<SampleSink>` z `SampleSink: Send`.
unsafe impl Send for ProcessTap {}

impl ProcessTap {
    pub fn start(sink: SampleSink) -> Result<Self> {
        let description = global_tap_description();
        let tap = create_tap(&description)?;
        match Self::start_with_tap(tap, &description, sink) {
            Ok(running) => Ok(running),
            Err(error) => {
                destroy_tap(tap);
                Err(error)
            }
        }
    }

    pub fn format(&self) -> CaptureFormat {
        self.format
    }

    fn start_with_tap(
        tap: AudioObjectID,
        description: &CATapDescription,
        sink: SampleSink,
    ) -> Result<Self> {
        let stream = tap_stream_format(tap)?;
        let format = capture_format(&stream)?;
        let non_interleaved = stream.mFormatFlags & kAudioFormatFlagIsNonInterleaved != 0;
        let device = create_private_device(description)?;
        let block = io_block(sink, format.channels, non_interleaved);
        let io_proc = match create_io_proc(device, &block) {
            Ok(io_proc) => io_proc,
            Err(error) => {
                destroy_device(device);
                return Err(error);
            }
        };
        // SAFETY: `device` i `io_proc` pochodzą z udanych wywołań powyżej.
        let status = unsafe { AudioDeviceStart(device, io_proc) };
        if status != NO_ERROR {
            // SAFETY: jak wyżej; IO nie wystartowało, więc nic go nie woła.
            unsafe { AudioDeviceDestroyIOProcID(device, io_proc) };
            destroy_device(device);
            return Err(core_audio("AudioDeviceStart", status));
        }
        tracing::info!(
            sample_rate = format.sample_rate,
            channels = format.channels,
            "przechwytywanie dźwięku systemowego"
        );
        Ok(Self {
            tap,
            device,
            io_proc,
            format,
            _block: block,
        })
    }
}

impl Drop for ProcessTap {
    fn drop(&mut self) {
        // SAFETY: obiekty utworzone w `start`; kolejność: zatrzymanie IO, usunięcie bloku,
        // urządzenia i tapu — po `AudioDeviceStop` Core Audio nie woła już bloku.
        unsafe {
            AudioDeviceStop(self.device, self.io_proc);
            AudioDeviceDestroyIOProcID(self.device, self.io_proc);
        }
        destroy_device(self.device);
        destroy_tap(self.tap);
        tracing::info!("przechwytywanie dźwięku systemowego zakończone");
    }
}

fn global_tap_description() -> Retained<CATapDescription> {
    let exclude: Retained<NSArray<NSNumber>> = NSArray::new();
    // SAFETY: inicjalizator z pustą tablicą wykluczeń (wszystkie procesy) — poprawny argument.
    let description = unsafe {
        CATapDescription::initStereoGlobalTapButExcludeProcesses(
            CATapDescription::alloc(),
            &exclude,
        )
    };
    // SAFETY: zwykłe settery właściwości obiektu, którego jesteśmy jedynym właścicielem.
    unsafe {
        description.setPrivate(true);
        description.setMuteBehavior(CATapMuteBehavior::Unmuted);
        description.setName(&NSString::from_str("VoiceAsystent — spotkanie"));
    }
    description
}

fn create_tap(description: &CATapDescription) -> Result<AudioObjectID> {
    let mut tap: AudioObjectID = 0;
    // SAFETY: `description` jest poprawnym obiektem, `tap` żyje przez cały czas wywołania.
    let status = unsafe { AudioHardwareCreateProcessTap(Some(description), &mut tap) };
    if status != NO_ERROR {
        return Err(core_audio("AudioHardwareCreateProcessTap", status));
    }
    Ok(tap)
}

fn destroy_tap(tap: AudioObjectID) {
    // SAFETY: `tap` pochodzi z udanego `AudioHardwareCreateProcessTap`.
    let status = unsafe { AudioHardwareDestroyProcessTap(tap) };
    if status != NO_ERROR {
        tracing::warn!(status, "nie udało się usunąć tapu dźwięku systemowego");
    }
}

fn destroy_device(device: AudioObjectID) {
    // SAFETY: `device` pochodzi z udanego `AudioHardwareCreateAggregateDevice`.
    let status = unsafe { AudioHardwareDestroyAggregateDevice(device) };
    if status != NO_ERROR {
        tracing::warn!(status, "nie udało się usunąć prywatnego urządzenia audio");
    }
}

fn address(selector: u32) -> AudioObjectPropertyAddress {
    AudioObjectPropertyAddress {
        mSelector: selector,
        mScope: kAudioObjectPropertyScopeGlobal,
        mElement: kAudioObjectPropertyElementMain,
    }
}

/// Odczyt właściwości o stałym rozmiarze `T`.
fn property<T: Copy>(object: AudioObjectID, selector: u32, empty: T) -> Result<T> {
    let mut value = empty;
    let mut size = u32::try_from(size_of::<T>()).unwrap_or(u32::MAX);
    let mut address = address(selector);
    // SAFETY: bufor wyjściowy to `value` o rozmiarze `size`; adres właściwości żyje do końca
    // wywołania; bez danych kwalifikujących.
    let status = unsafe {
        AudioObjectGetPropertyData(
            object,
            NonNull::from(&mut address),
            0,
            ptr::null(),
            NonNull::from(&mut size),
            NonNull::from(&mut value).cast(),
        )
    };
    if status != NO_ERROR {
        return Err(core_audio("AudioObjectGetPropertyData", status));
    }
    Ok(value)
}

fn tap_stream_format(tap: AudioObjectID) -> Result<AudioStreamBasicDescription> {
    let empty = AudioStreamBasicDescription {
        mSampleRate: 0.0,
        mFormatID: 0,
        mFormatFlags: 0,
        mBytesPerPacket: 0,
        mFramesPerPacket: 0,
        mBytesPerFrame: 0,
        mChannelsPerFrame: 0,
        mBitsPerChannel: 0,
        mReserved: 0,
    };
    property(tap, kAudioTapPropertyFormat, empty)
}

fn capture_format(stream: &AudioStreamBasicDescription) -> Result<CaptureFormat> {
    let is_float32 = stream.mFormatID == kAudioFormatLinearPCM
        && stream.mFormatFlags & kAudioFormatFlagIsFloat != 0
        && stream.mBitsPerChannel == 32;
    if !is_float32 {
        return Err(Error::Format(format!(
            "format {:#x}, flagi {:#x}, {} bitów",
            stream.mFormatID, stream.mFormatFlags, stream.mBitsPerChannel
        )));
    }
    let channels = u16::try_from(stream.mChannelsPerFrame)
        .ok()
        .filter(|channels| *channels > 0)
        .ok_or_else(|| Error::Format(format!("{} kanałów", stream.mChannelsPerFrame)))?;
    Ok(CaptureFormat {
        sample_rate: stream.mSampleRate.round() as u32,
        channels,
    })
}

/// UID bieżącego wyjścia systemowego — zegar prywatnego urządzenia; wyjście nie jest zmieniane.
fn default_output_uid() -> Result<Retained<NSString>> {
    let device: AudioObjectID = property(
        kAudioObjectSystemObject as AudioObjectID,
        kAudioHardwarePropertyDefaultSystemOutputDevice,
        0,
    )?;
    let raw: *const CFString = property(device, kAudioDevicePropertyDeviceUID, ptr::null())?;
    let raw = NonNull::new(raw.cast_mut())
        .ok_or_else(|| Error::Format("brak UID wyjścia dźwięku".into()))?;
    // SAFETY: właściwość UID zwraca CFString z licznikiem +1 (reguła „Copy/Create” Core Audio),
    // więc przejmujemy tę referencję.
    let uid: CFRetained<CFString> = unsafe { CFRetained::from_raw(raw) };
    Ok(NSString::from_str(&uid.to_string()))
}

fn key(name: &CStr) -> Retained<NSString> {
    NSString::from_str(&name.to_string_lossy())
}

fn dictionary(
    entries: Vec<(Retained<NSString>, Retained<NSObject>)>,
) -> Retained<NSDictionary<NSString, NSObject>> {
    let keys: Vec<&NSString> = entries.iter().map(|(key, _)| &**key).collect();
    let values: Vec<Retained<NSObject>> = entries.iter().map(|(_, value)| value.clone()).collect();
    NSDictionary::from_retained_objects(&keys, &values)
}

fn number(value: bool) -> Retained<NSObject> {
    Retained::into_super(Retained::into_super(NSNumber::new_bool(value)))
}

fn string(value: Retained<NSString>) -> Retained<NSObject> {
    Retained::into_super(value)
}

fn create_private_device(description: &CATapDescription) -> Result<AudioObjectID> {
    let output_uid = default_output_uid()?;
    // SAFETY: getter UUID opisu tapu, którego jesteśmy właścicielem.
    let tap_uid = unsafe { description.UUID() }.UUIDString();
    let sub_tap = dictionary(vec![
        (key(kAudioSubTapUIDKey), string(tap_uid)),
        (key(kAudioSubTapDriftCompensationKey), number(true)),
    ]);
    let sub_device = dictionary(vec![(
        key(kAudioSubDeviceUIDKey),
        string(output_uid.clone()),
    )]);
    let device = dictionary(vec![
        (
            key(kAudioAggregateDeviceNameKey),
            string(NSString::from_str("VoiceAsystent — spotkanie")),
        ),
        (
            key(kAudioAggregateDeviceUIDKey),
            string(NSUUID::new().UUIDString()),
        ),
        (
            key(kAudioAggregateDeviceMainSubDeviceKey),
            string(output_uid),
        ),
        (key(kAudioAggregateDeviceIsPrivateKey), number(true)),
        (key(kAudioAggregateDeviceIsStackedKey), number(false)),
        (key(kAudioAggregateDeviceTapAutoStartKey), number(true)),
        (
            key(kAudioAggregateDeviceSubDeviceListKey),
            Retained::into_super(NSArray::from_retained_slice(&[sub_device])),
        ),
        (
            key(kAudioAggregateDeviceTapListKey),
            Retained::into_super(NSArray::from_retained_slice(&[sub_tap])),
        ),
    ]);
    // SAFETY: NSDictionary i CFDictionary są mostkowane bez kopiowania (toll-free bridged);
    // referencja żyje do końca wywołania.
    let device_description: &CFDictionary =
        unsafe { &*(Retained::as_ptr(&device).cast::<CFDictionary>()) };
    let mut device_id: AudioObjectID = 0;
    // SAFETY: poprawny słownik opisu i bufor wyjściowy żyjący przez wywołanie.
    let status = unsafe {
        AudioHardwareCreateAggregateDevice(device_description, NonNull::from(&mut device_id))
    };
    if status != NO_ERROR {
        return Err(core_audio("AudioHardwareCreateAggregateDevice", status));
    }
    Ok(device_id)
}

fn io_block(sink: SampleSink, channels: u16, non_interleaved: bool) -> RcBlock<IoFn> {
    let sink = Mutex::new(sink);
    let scratch = Mutex::new(Vec::new());
    RcBlock::new(
        move |_now: NonNull<AudioTimeStamp>,
              input: NonNull<AudioBufferList>,
              _input_time: NonNull<AudioTimeStamp>,
              _output: NonNull<AudioBufferList>,
              _output_time: NonNull<AudioTimeStamp>| {
            // SAFETY: Core Audio przekazuje poprawną listę buforów na czas wywołania bloku.
            let buffers = unsafe { input_buffers(input) };
            let (Ok(mut sink), Ok(mut interleaved)) = (sink.lock(), scratch.lock()) else {
                return;
            };
            interleaved.clear();
            if non_interleaved {
                interleave(&buffers, usize::from(channels), &mut interleaved);
            } else if let Some(first) = buffers.first() {
                interleaved.extend_from_slice(first);
            }
            if !interleaved.is_empty() {
                sink(&interleaved);
            }
        },
    )
}

/// Bufory listy jako wycinki f32.
///
/// # Safety
/// `list` musi wskazywać poprawną `AudioBufferList` z buforami f32, żyjącą przez czas użycia.
unsafe fn input_buffers<'a>(list: NonNull<AudioBufferList>) -> Vec<&'a [f32]> {
    // SAFETY: wymagania funkcji; `mBuffers` ma `mNumberBuffers` elementów (tablica zmiennej
    // długości z C), a każdy `mData` to `mDataByteSize` bajtów próbek f32.
    unsafe {
        let list = list.as_ref();
        let count = list.mNumberBuffers as usize;
        let first: *const AudioBuffer = list.mBuffers.as_ptr();
        std::slice::from_raw_parts(first, count)
            .iter()
            .filter(|buffer| !buffer.mData.is_null())
            .map(|buffer| {
                let samples = buffer.mDataByteSize as usize / size_of::<f32>();
                std::slice::from_raw_parts(buffer.mData.cast::<f32>().cast_const(), samples)
            })
            .collect()
    }
}

/// Kanały w osobnych buforach (L L L…, R R R…) → przeplot L R L R.
fn interleave(planes: &[&[f32]], channels: usize, output: &mut Vec<f32>) {
    let frames = planes.iter().map(|plane| plane.len()).min().unwrap_or(0);
    for frame in 0..frames {
        for channel in 0..channels {
            output.push(planes.get(channel).map_or(0.0, |plane| plane[frame]));
        }
    }
}

fn create_io_proc(device: AudioObjectID, block: &RcBlock<IoFn>) -> Result<AudioDeviceIOProcID> {
    let mut io_proc: AudioDeviceIOProcID = None;
    let block_ptr: *const block2::DynBlock<IoFn> = &**block;
    // SAFETY: urządzenie z udanego utworzenia; blok żyje w `ProcessTap` dłużej niż IO
    // (Core Audio i tak kopiuje blok); kolejka NULL = wątek IO Core Audio.
    let status = unsafe {
        AudioDeviceCreateIOProcIDWithBlock(
            NonNull::from(&mut io_proc),
            device,
            None,
            block_ptr.cast_mut(),
        )
    };
    if status != NO_ERROR || io_proc.is_none() {
        return Err(core_audio("AudioDeviceCreateIOProcIDWithBlock", status));
    }
    Ok(io_proc)
}

fn core_audio(call: &'static str, status: i32) -> Error {
    Error::CoreAudio { call, status }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn planar_channels_are_interleaved() {
        let left = [0.1, 0.2, 0.3];
        let right = [-0.1, -0.2, -0.3];
        let mut output = Vec::new();

        interleave(&[&left, &right], 2, &mut output);

        assert_eq!(output, vec![0.1, -0.1, 0.2, -0.2, 0.3, -0.3]);
    }

    #[test]
    fn only_float32_pcm_is_accepted() {
        let mut stream = AudioStreamBasicDescription {
            mSampleRate: 48_000.0,
            mFormatID: kAudioFormatLinearPCM,
            mFormatFlags: kAudioFormatFlagIsFloat,
            mBytesPerPacket: 8,
            mFramesPerPacket: 1,
            mBytesPerFrame: 8,
            mChannelsPerFrame: 2,
            mBitsPerChannel: 32,
            mReserved: 0,
        };
        assert_eq!(
            capture_format(&stream).unwrap(),
            CaptureFormat {
                sample_rate: 48_000,
                channels: 2
            }
        );

        stream.mFormatFlags = 0;
        stream.mBitsPerChannel = 16;
        assert!(matches!(capture_format(&stream), Err(Error::Format(_))));
    }
}
