//! Strażnik prywatności: biblioteka audio trzyma nagranie wyłącznie w pamięci.

const SOURCES: [(&str, &str); 5] = [
    ("lib.rs", include_str!("../src/lib.rs")),
    ("devices.rs", include_str!("../src/devices.rs")),
    ("convert.rs", include_str!("../src/convert.rs")),
    ("recorder.rs", include_str!("../src/recorder.rs")),
    ("silence.rs", include_str!("../src/silence.rs")),
];

const FORBIDDEN: [&str; 5] = [
    "std::fs",
    "File::create",
    "OpenOptions",
    "hound",
    "tempfile",
];

#[test]
fn audio_library_never_touches_the_filesystem() {
    for (file, source) in SOURCES {
        for pattern in FORBIDDEN {
            assert!(
                !source.contains(pattern),
                "{file} zawiera {pattern:?} — nagranie nie może trafiać na dysk"
            );
        }
    }
}
