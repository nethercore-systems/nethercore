#!/usr/bin/env python
"""Native clip selection, pose identity, pause and reset; Pillow, scale 1."""
import argparse
from pathlib import Path
from PIL import Image

# Exact boundary visits in capture.ncrs, including reverse ping-pong and restart.
POSES = [0,0,1,2,3,0,0,1,2,3,3,0,1,3,0,1,2,3,2,0,1,1,1,1,0]


def check(paths):
    assert len(paths) == len(POSES), 'Expected all 25 captures, in replay order'
    images = [Image.open(path).convert('RGB') for path in paths]
    assert all(image.size == (960, 540) for image in images)
    thumbs = [images[0].crop((264+i*108,412,328+i*108,476)).resize((16,16),Image.Resampling.NEAREST)
              for i in range(4)]
    assert len({thumb.tobytes() for thumb in thumbs}) == 4, 'Four distinct poses required'
    for thumb in thumbs:
        background = thumb.getpixel((0,15))
        assert 40 < sum(pixel != background for pixel in thumb.getdata()) < 160, 'Missing/solid pose'
    for index, (image, pose) in enumerate(zip(images, POSES)):
        main = image.crop((360,126,600,366)).resize((16,16),Image.Resampling.NEAREST)
        assert main.tobytes() == thumbs[pose].tobytes(), f'Capture {index}: expected pose {pose}'
        for tile in range(4):
            strip = image.crop((264+tile*108,412,328+tile*108,476)).resize((16,16),Image.Resampling.NEAREST)
            assert strip.tobytes() == thumbs[tile].tobytes(), 'Immutable atlas changed'
            r,g,b = image.getpixel((262+tile*108,410))
            assert (r > b+30 and g > b+30) == (tile == pose), 'Wrong selected-frame border'
    assert images[0].tobytes() == images[24].tobytes(), 'Reset differs from launch'
    assert images[21].tobytes() == images[22].tobytes(), 'Paused playback advanced'
    assert images[10].tobytes() == images[13].tobytes(), 'Once-mode restart did not reproduce its endpoint'
    print('PASS 25 native boundary views: four poses, atlas UVs, selection, reverse ping-pong, once/restart, pause, reset')


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('images', nargs=25, type=Path)
    check(parser.parse_args().images)
