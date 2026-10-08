"""Generuje nagrania mowy do testów transkrypcji (16 kHz, mono, 16 bit).

speech_pl.wav — głos Zosia (pl_PL), speech_en.wav — głos Samantha (en_US),
speech_en_44k_stereo.wav — to samo zdanie EN w 44,1 kHz stereo (wejście do normalizacji).
Uruchomienie (z katalogu repozytorium): python3 tests/fixtures/generate_speech_fixtures.py
"""

import subprocess
import sys
from pathlib import Path

FIXTURES = Path(__file__).resolve().parent

RECORDINGS = [
    ("speech_pl.wav", "Zosia", "LEI16@16000", "Dzisiaj jest piękna pogoda, idę na spacer do parku."),
    ("speech_en.wav", "Samantha", "LEI16@16000", "The weather is beautiful today, I am going for a walk in the park."),
]


def main() -> int:
    for name, voice, data_format, phrase in RECORDINGS:
        target = FIXTURES / name
        subprocess.run(
            ["say", "-v", voice, "-o", str(target), f"--data-format={data_format}", phrase],
            check=True,
        )
        print(f"{name}: {target.stat().st_size} B")
    stereo = FIXTURES / "speech_en_44k_stereo.wav"
    subprocess.run(
        ["afconvert", "-f", "WAVE", "-d", "LEI16@44100", "-c", "2",
         str(FIXTURES / "speech_en.wav"), str(stereo)],
        check=True,
    )
    print(f"{stereo.name}: {stereo.stat().st_size} B")
    return 0


if __name__ == "__main__":
    sys.exit(main())
