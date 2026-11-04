//! Zgoda „Nagrywanie dźwięku systemowego” (usługa TCC `kTCCServiceAudioCapture`). macOS nie ma
//! publicznego API do jej sprawdzenia, więc — jak inne aplikacje z process tap — korzystamy
//! z funkcji `TCCAccessPreflight`/`TCCAccessRequest` frameworka TCC, ładowanych dynamicznie.
//! Gdy ich zabraknie (zmiana w systemie), stan jest nieustalony, a pytanie pokaże sam system
//! przy pierwszym odczycie tapu.
#![allow(unsafe_code)] // FFI TCC — jedyne miejsce z unsafe obok `tap` (PROJECT_CONTEXT, 2026-11-04)

use std::ffi::{CStr, c_int, c_void};
use std::sync::mpsc;
use std::time::Duration;

use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::Bool;
use objc2_foundation::NSString;

use crate::Authorization;

const FRAMEWORK: &CStr = c"/System/Library/PrivateFrameworks/TCC.framework/Versions/A/TCC";
const SERVICE: &str = "kTCCServiceAudioCapture";
/// Tyle czekamy na odpowiedź użytkownika w systemowym oknie zgody.
const ANSWER_TIMEOUT: Duration = Duration::from_secs(120);

type PreflightFn = unsafe extern "C" fn(service: *const c_void, options: *const c_void) -> c_int;
type RequestFn = unsafe extern "C" fn(
    service: *const c_void,
    options: *const c_void,
    completion: *const block2::DynBlock<dyn Fn(Bool)>,
);

pub fn audio_capture_status() -> Authorization {
    let Some(preflight) = symbol::<PreflightFn>(c"TCCAccessPreflight") else {
        return Authorization::Undetermined;
    };
    let service = service_name();
    // SAFETY: `preflight` ma sygnaturę funkcji TCC; usługa to NSString (mostkowany z CFString)
    // żyjący przez wywołanie; opcje NULL.
    let status = unsafe { preflight(Retained::as_ptr(&service).cast(), std::ptr::null()) };
    match status {
        0 => Authorization::Granted,
        1 => Authorization::Denied,
        _ => Authorization::Undetermined,
    }
}

pub fn request_audio_capture() -> Authorization {
    let Some(request) = symbol::<RequestFn>(c"TCCAccessRequest") else {
        return Authorization::Undetermined;
    };
    let (answer, answered) = mpsc::channel();
    let completion = RcBlock::new(move |granted: Bool| {
        let _ = answer.send(granted.as_bool());
    });
    let service = service_name();
    // SAFETY: sygnatura funkcji TCC; usługa żyje przez wywołanie, a blok jest kopiowany przez
    // TCC przed powrotem (wywołanie asynchroniczne) — nasza referencja żyje do końca funkcji.
    unsafe {
        request(
            Retained::as_ptr(&service).cast(),
            std::ptr::null(),
            &*completion,
        );
    }
    match answered.recv_timeout(ANSWER_TIMEOUT) {
        Ok(true) => Authorization::Granted,
        Ok(false) => Authorization::Denied,
        Err(_) => audio_capture_status(),
    }
}

fn service_name() -> Retained<NSString> {
    NSString::from_str(SERVICE)
}

/// Funkcja `name` z frameworka TCC albo `None`, gdy framework lub symbol nie istnieje.
fn symbol<F: Copy>(name: &CStr) -> Option<F> {
    // SAFETY: `dlopen` z poprawną ścieżką C; uchwyt nie jest zamykany (framework systemowy
    // zostaje załadowany do końca procesu).
    let handle = unsafe { libc::dlopen(FRAMEWORK.as_ptr(), libc::RTLD_NOW) };
    if handle.is_null() {
        tracing::warn!("brak frameworka TCC — stan zgody na dźwięk systemowy nieznany");
        return None;
    }
    // SAFETY: `handle` z udanego `dlopen`, nazwa to poprawny ciąg C.
    let pointer = unsafe { libc::dlsym(handle, name.as_ptr()) };
    if pointer.is_null() {
        tracing::warn!(symbol = ?name, "brak funkcji TCC — stan zgody na dźwięk systemowy nieznany");
        return None;
    }
    // SAFETY: `F` to wskaźnik funkcji o rozmiarze wskaźnika i sygnaturze tej funkcji TCC
    // (wywołujący podaje typ zgodny z deklaracją w TCC.framework).
    Some(unsafe { std::mem::transmute_copy::<*mut c_void, F>(&pointer) })
}
