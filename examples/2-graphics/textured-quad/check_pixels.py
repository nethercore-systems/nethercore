#!/usr/bin/env python
"""Check ten scale-1 native capture.ncrs images; requires Pillow."""
import argparse
from pathlib import Path
from PIL import Image


def check(paths):
    assert len(paths) == 10, 'Expected all ten captures in replay order'
    images = [Image.open(path).convert('RGB') for path in paths]
    assert all(image.size == (960, 540) for image in images)
    refs = [image.crop((64, 154, 384, 474)) for image in images]
    subjects = [image.crop((520, 154, 840, 474)) for image in images]
    assert all(ref.tobytes() == refs[0].tobytes() for ref in refs), 'Per-draw filtering leaked into the reference'
    assert refs[0].tobytes() == subjects[0].tobytes(), 'Full UV region differs from draw_sprite'
    assert images[0].tobytes() == images[-1].tobytes(), 'Reset did not restore the exact frame'

    def color(pixel, tile):
        r, g, b = pixel
        return [r > g + 30 and r > b + 30,
                g > r + 30 and g > b + 30,
                b > r + 30 and b > g + 30,
                r > b + 50 and g > b + 50][tile]

    for tile, point in enumerate([(8, 8), (168, 8), (8, 168), (168, 168)]):
        assert color(refs[0].getpixel(point), tile), f'Wrong atlas orientation: corner {tile}'
    # Each nearest tile capture contains exactly two checker colors and white ink.
    for index, tile in [(4, 0), (5, 1), (6, 2), (7, 3)]:
        palette = subjects[index].getcolors(200000)
        assert len(palette) == 3, (index, len(palette))
        assert any(pixel == (255, 255, 255) and count > 1000 for count, pixel in palette), 'Missing numeral ink'
        assert all(pixel == (255, 255, 255) or color(pixel, tile) for _, pixel in palette), 'Wrong UV tile or neighboring color'
    nearest_colors = len(subjects[0].getcolors(200000))
    linear_colors = len(subjects[1].getcolors(200000))
    assert nearest_colors == 9 and linear_colors > 100, 'Nearest/linear toggle is not rendered'
    # Tile 1's right edge crosses into tile 2 without the inset; green must disappear.
    raw, inset = subjects[2].getpixel((319, 8)), subjects[3].getpixel((319, 8))
    assert raw[1] > raw[0] and inset[0] > inset[1] + 30 and raw[1] > inset[1] + 30, (raw, inset)
    assert subjects[8].tobytes() != subjects[0].tobytes(), 'Full-atlas inset should visibly narrow the UV range'
    print('PASS atlas corners, four labeled UV tiles, per-draw filtering, edge bleed/inset and reset;',
          'palette sizes:', nearest_colors, linear_colors, 'edge:', raw, inset)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('images', nargs=10, type=Path)
    check(parser.parse_args().images)
