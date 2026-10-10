#!/usr/bin/env python
"""Check the nine scale-1 native captures in capture.ncrs order (Pillow)."""
import argparse
import hashlib
from pathlib import Path
from PIL import Image


def check(paths):
    frames = [0, 5, 10, 15, 20, 25, 30, 35, 40]
    assert len(paths) == len(frames), 'Expected all nine captures, in replay order'
    coverage = {}
    hashes = {}
    for frame, path in zip(frames, paths):
        image = Image.open(path).convert('RGB')
        assert image.size == (960, 540), image.size
        # Contains the triangle, but none of the labels. Empty frames are required.
        coverage[frame] = sum(pixel != (79, 99, 121)
                              for pixel in image.crop((300, 100, 660, 430)).getdata())
        hashes[frame] = hashlib.sha256(image.tobytes()).hexdigest()
    assert all(coverage[f] == 0 for f in [5, 15, 35]), coverage
    assert all(coverage[f] > 1000 for f in [0, 10, 20, 25, 30, 40]), coverage
    assert 0 < coverage[30] < coverage[0], '45-degree view must narrow the triangle'
    assert hashes[0] == hashes[25] == hashes[40], 'Both resets must restore the initial frame'
    print('PASS winding, camera, culling, 45/90-degree rotation and reset:', coverage)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('images', nargs=9, type=Path)
    check(parser.parse_args().images)
