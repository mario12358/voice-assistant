"""Generuje dialogue_me.wav i dialogue_others.wav: sztuczną rozmowę na dwie ścieżki (16 kHz, mono).

Fixture dla VA-CONF-4 (kolejność wypowiedzi w transkrypcji spotkania): sześć naprzemiennych
wypowiedzi z syntezatora macOS (`say`, głos Zosia); każda trafia na ścieżkę swojego mówcy, a na
drugiej w tym czasie jest cisza. Przerwa między wypowiedziami 0,7 s. Momenty wypowiedzi
drukuje skrypt — test sprawdza, że transkrypcja zachowuje tę kolejność.
Uruchomienie (z katalogu repozytorium): python3 tests/fixtures/generate_dialogue_fixtures.py
"""

import array
import subprocess
import tempfile
import wave
from pathlib import Path

SAMPLE_RATE = 16_000
GAP_S = 0.7
TURNS = [
    ("me", "Dzień dobry, w czym mogę pomóc?"),
    ("others", "Chciałabym zapytać o obiad."),
    ("me", "Na obiad będzie zupa pomidorowa."),
    ("others", "A co będzie na deser?"),
    ("me", "Na deser będą lody waniliowe."),
    ("others", "Dziękuję bardzo, do widzenia."),
]
OUT = Path(__file__).parent


def synthesize(text: str, work: Path, index: int) -> array.array:
    aiff = work / f"turn{index}.aiff"
    wav = work / f"turn{index}.wav"
    subprocess.run(["say", "-v", "Zosia", "-o", str(aiff), text], check=True)
    subprocess.run(
        ["afconvert", "-f", "WAVE", "-d", f"LEI16@{SAMPLE_RATE}", "-c", "1", str(aiff), str(wav)],
        check=True,
    )
    with wave.open(str(wav)) as reader:
        return array.array("h", reader.readframes(reader.getnframes()))


def main() -> None:
    tracks = {"me": array.array("h"), "others": array.array("h")}
    cursor = 0
    with tempfile.TemporaryDirectory() as work:
        for index, (speaker, text) in enumerate(TURNS):
            samples = synthesize(text, Path(work), index)
            for track in tracks.values():
                track.extend([0] * (cursor - len(track)))
            tracks[speaker].extend(samples)
            print(f"{cursor / SAMPLE_RATE:5.1f} s  {speaker:6}  {text}")
            cursor += len(samples) + int(GAP_S * SAMPLE_RATE)
    for speaker, track in tracks.items():
        track.extend([0] * (cursor - len(track)))
        with wave.open(str(OUT / f"dialogue_{speaker}.wav"), "wb") as writer:
            writer.setnchannels(1)
            writer.setsampwidth(2)
            writer.setframerate(SAMPLE_RATE)
            writer.writeframes(track.tobytes())


if __name__ == "__main__":
    main()
