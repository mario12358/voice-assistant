"""Generuje fixtures do VA-STT-3 (16 kHz, mono, 16 bit).

noise_only.wav      — 6 s szumu o RMS ok. 0,03, czyli powyżej progu ciszy 0,01 (przejdzie przez
                      przycinanie ciszy do modelu), bez żadnej mowy.
speech_pl_mixed.wav — polskie zdanie z angielskimi słowami (głos Zosia), do sprawdzenia, że
                      z językiem „pl” model nie przechodzi na angielski.
Uruchomienie (z katalogu repozytorium): python3 tests/fixtures/generate_stt_quality_fixtures.py
"""

import random
import struct
import subprocess
import sys
import wave
from pathlib import Path

SAMPLE_RATE = 16_000
NOISE_SECONDS = 6.0
NOISE_AMPLITUDE = 0.052  # rozkład jednostajny: RMS = A / sqrt(3) ≈ 0,03
MIXED_PHRASE = "Muszę zrobić deploy na produkcję i sprawdzić logi w dashboardzie."
FIXTURES = Path(__file__).resolve().parent


def write_wav(path: Path, samples: list[float]) -> None:
    data = b"".join(struct.pack("<h", round(max(-1.0, min(1.0, s)) * 32767)) for s in samples)
    with wave.open(str(path), "wb") as wav:
        wav.setnchannels(1)
        wav.setsampwidth(2)
        wav.setframerate(SAMPLE_RATE)
        wav.writeframes(data)


def main() -> int:
    rng = random.Random(2026)
    noise = [rng.uniform(-NOISE_AMPLITUDE, NOISE_AMPLITUDE) for _ in range(int(NOISE_SECONDS * SAMPLE_RATE))]
    write_wav(FIXTURES / "noise_only.wav", noise)
    mixed = FIXTURES / "speech_pl_mixed.wav"
    subprocess.run(
        ["say", "-v", "Zosia", "-o", str(mixed), f"--data-format=LEI16@{SAMPLE_RATE}", MIXED_PHRASE],
        check=True,
    )
    print(f"noise_only.wav: {len(noise)} próbek; speech_pl_mixed.wav: {mixed.stat().st_size} B")
    return 0


if __name__ == "__main__":
    sys.exit(main())
