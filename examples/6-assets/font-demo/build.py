#!/usr/bin/env python
"""Derive a cropped custom atlas from the repository font; Python stdlib only."""
import argparse
import ast
import re
import struct
import subprocess
import zlib
from pathlib import Path


def generate():
    project = Path(__file__).resolve().parent
    source = project.parents[2] / 'nethercore-zx/src/font.rs'
    glyphs = {}
    for char, rows in re.findall(r"b'((?:\\.|[^'])+)' => \[([^]]+)\]", source.read_text()):
        code = ast.literal_eval("b'" + char + "'")[0]
        glyphs[code] = [int(value.strip(), 16) for value in rows.split(',')]
    assert sorted(glyphs) == list(range(32, 127))
    pixels, advances = bytearray(128 * 48 * 4), bytearray()
    for code, rows in sorted(glyphs.items()):
        assert len(rows) == 8
        occupied = [x for x in range(8) if any(row & (128 >> x) for row in rows)]
        left = min(occupied, default=0)
        advance = min(8, max(occupied, default=2) - left + 2)
        advances.append(advance)
        col, row = (code - 32) % 16, (code - 32) // 16
        for y, bits in enumerate(rows):
            for x in range(8 - left):
                offset = ((row * 8 + y) * 128 + col * 8 + x) * 4
                pixels[offset:offset + 4] = bytes((255, 255, 255, 255 if bits & (128 >> (x + left)) else 0))
    assert len(advances) == 95 and max(advances) == 8
    assert advances[ord('i') - 32] < advances[ord('W') - 32]
    # PNG's unfiltered scanlines + zlib avoid a build-time image dependency.
    scanlines = b''.join(b'\0' + pixels[y * 512:(y + 1) * 512] for y in range(48))
    def chunk(kind, data):
        return struct.pack('>I', len(data)) + kind + data + struct.pack('>I', zlib.crc32(kind + data))
    png = b'\x89PNG\r\n\x1a\n'
    png += chunk(b'IHDR', struct.pack('>IIBBBBB', 128, 48, 8, 6, 0, 0, 0))
    png += chunk(b'IDAT', zlib.compress(scanlines, 9)) + chunk(b'IEND', b'')
    assets = project / 'assets'
    assets.mkdir(exist_ok=True)
    (assets / 'font.png').write_bytes(png)
    (assets / 'widths.bin').write_bytes(advances)
    return project


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--release', action='store_true')
    parser.add_argument('--assets-only', action='store_true')
    args = parser.parse_args()
    project = generate()
    if not args.assets_only:
        subprocess.run(['cargo', 'build', '--target', 'wasm32-unknown-unknown']
                       + (['--release'] if args.release else []), cwd=project, check=True)
