#!/usr/bin/env python3
"""Generuje AppIcon.iconset dla VoiceAsystent.app: szare kółko z czerwoną kropką.

Ikona rysowana w kodzie (jak ikona paska menu w apps/voice-asystent/src/indicator.rs),
żeby nie trzymać w repozytorium binarnych zasobów. Tylko biblioteka standardowa —
bez PIL, uruchamiane na maszynie deweloperskiej i w CI (macOS).

Użycie: app-icon.py <katalog .iconset>   (potem: iconutil -c icns <katalog>)
"""

import struct
import sys
import zlib
from pathlib import Path

GRAY = (142, 142, 147)
RED = (255, 59, 48)
ICONSET_SIZES = (16, 32, 128, 256, 512)


def coverage(distance: float, radius: float) -> float:
    return min(1.0, max(0.0, radius + 0.5 - distance))


def row_pixels(y: int, size: int) -> bytes:
    center = size / 2
    row = bytearray()
    for x in range(size):
        distance = ((x + 0.5 - center) ** 2 + (y + 0.5 - center) ** 2) ** 0.5
        outer = coverage(distance, size * 0.46)
        inner = coverage(distance, size * 0.18)
        color = (round(RED[i] * inner + GRAY[i] * (1 - inner)) for i in range(3))
        row.extend(color)
        row.append(round(outer * 255))
    return bytes(row)


def png_chunk(tag: bytes, data: bytes) -> bytes:
    crc = zlib.crc32(tag + data) & 0xFFFFFFFF
    return struct.pack(">I", len(data)) + tag + data + struct.pack(">I", crc)


def png(size: int) -> bytes:
    raw = b"".join(b"\x00" + row_pixels(y, size) for y in range(size))
    header = struct.pack(">IIBBBBB", size, size, 8, 6, 0, 0, 0)
    return (
        b"\x89PNG\r\n\x1a\n"
        + png_chunk(b"IHDR", header)
        + png_chunk(b"IDAT", zlib.compress(raw, 9))
        + png_chunk(b"IEND", b"")
    )


def write_iconset(directory: Path) -> None:
    directory.mkdir(parents=True, exist_ok=True)
    rendered: dict[int, bytes] = {}
    for base in ICONSET_SIZES:
        for scale in (1, 2):
            pixels = base * scale
            rendered.setdefault(pixels, png(pixels))
            suffix = "@2x" if scale == 2 else ""
            (directory / f"icon_{base}x{base}{suffix}.png").write_bytes(rendered[pixels])


def main(argv: list[str]) -> int:
    if len(argv) != 2:
        print("użycie: app-icon.py <katalog .iconset>", file=sys.stderr)
        return 2
    write_iconset(Path(argv[1]))
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
