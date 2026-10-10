#!/usr/bin/env python
"""Independent integer glyph/layout oracle for every native capture and snapshot.

Contract: 960x540, size-16/24 nearest glyph pixels, exact coverage (zero tolerance).
Build first. Pillow is needed for the pixel check, not to generate/build the ROM.
"""
import argparse
import json
import tomllib
from pathlib import Path
from PIL import Image

PROJECT = Path(__file__).resolve().parent
SAMPLE = 'WWW ill'
PARAGRAPH = 'Wide WWW and tiny ill wrap into a menu.'


def states():
    frames = tomllib.loads((PROJECT / 'capture.ncrs').read_text())['frames']
    indexed = {row['f']: row for row in frames}
    state, previous, result = dict(variable=0, alignment=0, narrow=0, focus=0), set(), {}
    for tick in range(max(indexed) + 1):
        row = indexed.get(tick, {})
        buttons = set(row.get('p1', '').split('+')) - {''}
        pressed = buttons - previous
        previous = buttons
        if 'y' in pressed:
            state = dict(variable=0, alignment=0, narrow=0, focus=0)
        else:
            if 'a' in pressed: state['variable'] ^= 1
            if 'b' in pressed: state['alignment'] = (state['alignment'] + 1) % 3
            if 'x' in pressed: state['narrow'] ^= 1
            if 'down' in pressed: state['focus'] = (state['focus'] + 1) % 3
            if 'up' in pressed: state['focus'] = (state['focus'] - 1) % 3
        if row.get('screenshot'): result[tick] = dict(state)
    return result


def width(text, variable, advances, size=16):
    return sum((advances[ord(ch)-32] if variable else 8) * size // 8
               for ch in text if 32 <= ord(ch) <= 126)


def wrapped(variable, narrow, advances):
    # Whole-word greedy reference, distinct from the guest's prefix/UTF-8 loop.
    result, current = [], ''
    for word in PARAGRAPH.split():
        trial = (current + ' ' + word).strip()
        if current and width(trial, variable, advances) > (160 if narrow else 280):
            result.append(current)
            current = word
        else: current = trial
    return result + [current]


def check_state(path):
    advances = (PROJECT / 'assets/widths.bin').read_bytes()
    expected = states()
    data = json.loads(path.read_text())
    assert data['summary']['status'] == 'PASSED'
    assert [s['frame'] for s in data['snapshots']] == list(expected)
    checked = 0
    for snap in data['snapshots']:
        state = expected[snap['frame']]
        measured = width(SAMPLE, state['variable'], advances)
        values = {**state, 'fonts_ok': 1, 'sample_width': measured,
                  'sample_x': 48 + (528 - measured) * state['alignment'] / 2,
                  'raw_width': width('AB', state['variable'], advances, 24),
                  'fallback_width': width('A?B', state['variable'], advances, 24),
                  'lines': len(wrapped(state['variable'], state['narrow'], advances))}
        for key, wanted in values.items():
            actual = next(iter(snap['post']['$'+key].values()))
            assert actual == wanted, (snap['frame'], key, actual, wanted)
            checked += 1
    print('PASS independent snapshot fields:', checked)


def check_pixels(paths):
    expected = list(states().values())
    assert len(paths) == len(expected)
    images = [Image.open(path).convert('RGB') for path in paths]
    atlas = Image.open(PROJECT / 'assets/font.png').getchannel('A')
    advances = (PROJECT / 'assets/widths.bin').read_bytes()
    assert atlas.size == (128, 48) and len(advances) == 95 and max(advances) == 8
    assert all(image.size == (960, 540) for image in images)
    assert images[0].tobytes() == images[-1].tobytes(), 'Reset differs'
    glyph_pixels = 0
    for index, (image, state) in enumerate(zip(images, expected)):
        variable = state['variable']
        def glyphs(text, x, y, size=16):
            result = set()
            scale = size // 8
            for ch in text:
                if not 32 <= ord(ch) <= 126: continue
                cell = ord(ch) - 32
                advance = advances[cell] if variable else 8
                for gy in range(8):
                    for gx in range(advance):
                        if atlas.getpixel((cell % 16 * 8 + gx, cell // 16 * 8 + gy)):
                            result.update((int(x + gx * scale + dx), y + gy * scale + dy)
                                          for dy in range(scale) for dx in range(scale))
                x += advance * scale
            return result
        measure = width(SAMPLE, variable, advances)
        x = 48 + (528-measure) * state['alignment'] / 2
        cyan = glyphs(SAMPLE, x, 230)
        cyan |= glyphs('>', 636, 136 + state['focus'] * 36)
        cyan |= glyphs('AB', 700, 326, 24) | glyphs('A?B', 700, 366, 24)
        for line, text in enumerate(wrapped(variable, state['narrow'], advances)):
            cyan |= glyphs(text, 48, 322 + line * 22)
        actual = {(x,y) for y in range(120,448) for x in range(32,928)
                  if (lambda c: c[2] > 240 and c[1] > c[0] + 20 and c[0] < 230)(image.getpixel((x,y)))}
        assert cyan == actual, (index, 'cyan glyph coverage', len(cyan-actual), len(actual-cyan))
        white = glyphs('ABCDEFGHIJKLMNOPQRSTUVWXYZ',48,132)
        white |= glyphs('abcdefghijklmnopqrstuvwxyz',48,162)
        white |= glyphs('0123456789 !?.,:[]',48,192)
        white |= set().union(*(glyphs(text,658,136+i*36) for i,text in enumerate(['New game','Options','Credits'])))
        actual = {(x,y) for left,top,right,bottom in [(40,126,588,212),(628,126,920,246)]
                  for y in range(top,bottom) for x in range(left,right)
                  if min(image.getpixel((x,y))) > 240}
        assert white == actual, (index,'white glyph coverage',len(white-actual),len(actual-white))
        colors = [image.getpixel((914,142+i*36)) for i in range(3)]
        assert colors[state['focus']] != colors[(state['focus']+1)%3]
        assert colors[(state['focus']+1)%3] == colors[(state['focus']+2)%3]
        glyph_pixels += len(cyan) + len(white)
    print('PASS exact glyph/layout pixels:', glyph_pixels, 'across', len(images),
          'native frames; fixed/variable, three alignments, two wraps, missing/fallback, focus and reset')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('images', nargs='*', type=Path)
    parser.add_argument('--state', type=Path)
    args = parser.parse_args()
    assert args.state or args.images, 'Supply the report or all capture images'
    if args.state: check_state(args.state)
    if args.images: check_pixels(args.images)
