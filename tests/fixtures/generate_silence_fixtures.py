"""Generuje fixtures WAV do testów przycinania ciszy (16 kHz, mono, 16 bit).

Mowa pochodzi z syntezatora macOS (`say`), przycięta do pierwszej/ostatniej
głośnej próbki, więc granice mowy w plikach są znane co do próbki.
Uruchomienie (z katalogu repozytorium): python3 tests/fixtures/generate_silence_fixtures.py
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
SPEECH_EDGE_AMPLITUDE = 0.02
LEADING_SILENCE_S = 1.0
TRAILING_SILENCE_S = 1.5
SILENCE_ONLY_S = 2.0
PHRASE = "Dzień dobry, to jest test przycinania ciszy."

FIXTURES = Path(__file__).resolve().parent


def synthesize_speech() -> list[float]:
    with tempfile.TemporaryDirectory() as directory:
        path = Path(directory) / "speech.wav"
        subprocess.run(
            ["say", "-o", str(path), f"--data-format=LEI16@{SAMPLE_RATE}", PHRASE],
            check=True,
        )
        samples = read_wav(path)
    loud = [index for index, sample in enumerate(samples) if abs(sample) >= SPEECH_EDGE_AMPLITUDE]
    return samples[loud[0] : loud[-1] + 1]


def read_wav(path: Path) -> list[float]:
    with wave.open(str(path), "rb") as wav:
        assert wav.getnchannels() == 1 and wav.getsampwidth() == 2
        frames = wav.readframes(wav.getnframes())
    return [value / 32768 for (value,) in struct.iter_unpack("<h", frames)]


def noise(seconds: float, rng: random.Random) -> list[float]:
    count = int(seconds * SAMPLE_RATE)
    return [rng.uniform(-NOISE_AMPLITUDE, NOISE_AMPLITUDE) for _ in range(count)]


def write_wav(name: str, samples: list[float]) -> None:
    data = b"".join(struct.pack("<h", round(max(-1.0, min(1.0, s)) * 32767)) for s in samples)
    with wave.open(str(FIXTURES / name), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(data)


def main() -> int:
    rng = random.Random(2026)
    speech = synthesize_speech()
    leading = noise(LEADING_SILENCE_S, rng)
    trailing = noise(TRAILING_SILENCE_S, rng)
    write_wav("silence_only.wav", noise(SILENCE_ONLY_S, rng))
    write_wav("speech_only.wav", speech)
    write_wav("speech_with_silence.wav", leading + speech + trailing)
    print(f"mowa: {len(speech)} próbek; start {len(leading)}, koniec {len(leading) + len(speech)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
