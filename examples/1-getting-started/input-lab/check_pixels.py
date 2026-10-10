#!/usr/bin/env python
"""Check all eight stick markers in an actual scale-1 input-lab capture.

Requires Pillow. This rejects the real missing-panel regression; it does not
certify physical controllers, network inputs, readable text or human taste.
"""
import argparse
from pathlib import Path
from PIL import Image


def check(path: Path):
    image = Image.open(path).convert("RGB")
    assert image.size == (960, 540), image.size
    pixels = image.load()
    counts = []
    for player in range(4):
        x = 24 + (player % 2) * 468
        y = 82 + (player // 2) * 208
        for stick in (54, 174):
            # Covers the entire [-1, 1] component range, including diagonal input.
            colors = [pixels[px, py]
                      for py in range(y + 102, y + 182)
                      for px in range(x + stick - 40, x + stick + 40)]
            blue = colors.count((170, 235, 247))
            gold = colors.count((255, 232, 170))
            assert blue >= 15 and gold >= 40, (
                f"P{player + 1} stick {stick}: missing markers, blue={blue}, gold={gold}"
            )
            counts.append((blue, gold))
    assert len(counts) == 8
    print(f"PASS {path.name}: eight visible raw/filtered stick markers {counts}")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("images", nargs="+", type=Path)
    for capture in parser.parse_args().images:
        check(capture)
