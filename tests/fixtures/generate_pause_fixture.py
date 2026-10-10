"""Generuje speech_pl_long_pause.wav: dwa polskie zdania rozdzielone 40 s ciszy (16 kHz, mono, 16 bit).

Fixture dla VA-REC-5 (skracanie pauz): zdania z syntezatora macOS (`say`, głos Zosia),
między nimi szum o amplitudzie poniżej progu ciszy, jak w generate_silence_fixtures.py.
Osobny skrypt, żeby nie regenerować pozostałych fixtures (ich granice mowy są skalibrowane w testach).
Uruchomienie (z katalogu repozytorium): python3 tests/fixtures/generate_pause_fixture.py
"""

import random
import struct
import subprocess
import sys
import tempfile
import wave
from pathlib import Path

SAMPLE_RATE = 16_000
NOISE_AMPLITUDE = 0.002
PAUSE_S = 40.0
FIRST = "Dzisiaj jest piękna pogoda, idę na spacer do parku."
SECOND = "Jutro rano mam spotkanie w biurze o dziewiątej."
TARGET = Path(__file__).resolve().parent / "speech_pl_long_pause.wav"


def synthesize(phrase: str) -> list[float]:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "speech.wav"
        subprocess.run(
            ["say", "-v", "Zosia", "-o", str(path), f"--data-format=LEI16@{SAMPLE_RATE}", phrase],
            check=True,
        )
        with wave.open(str(path), "rb") as wav:
            frames = wav.readframes(wav.getnframes())
    return [value / 32768 for (value,) in struct.iter_unpack("<h", frames)]


def noise(seconds: float, rng: random.Random) -> list[float]:
    return [rng.uniform(-NOISE_AMPLITUDE, NOISE_AMPLITUDE) for _ in range(int(seconds * SAMPLE_RATE))]


def main() -> int:
    rng = random.Random(2026)
    samples = synthesize(FIRST) + noise(PAUSE_S, rng) + synthesize(SECOND)
    data = b"".join(struct.pack("<h", round(max(-1.0, min(1.0, s)) * 32767)) for s in samples)
    with wave.open(str(TARGET), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(data)
    print(f"{TARGET.name}: {len(samples)} próbek ({len(samples) / SAMPLE_RATE:.1f} s)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
