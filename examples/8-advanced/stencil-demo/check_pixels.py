#!/usr/bin/env python
"""960x540 native stencil contract, frozen before capture.

All scene-area pixels are checked against a scalar 16-rectangle union/half-plane.
Only a one-pixel analytic raster-boundary band is indeterminate. Color classes
identify masks, not shader RGB parity. Common HUD pixels and repeat/reset frames
are exact. Depth-clear differences must stay inside the portal aperture.
"""
import argparse
import json
import math
import re
from functools import lru_cache
from pathlib import Path
from PIL import Image

CASES = [(0, 0, 0, 1, 0, 0), (7, 0, 1, 1, 0, 0), (12, 1, 1, 1, 0, 0), (17, 1, 0, 1, 0, 0), (22, 2, 0, 1, 0, 0), (27, 2, 1, 1, 0, 0), (32, 3, 1, 1, 0, 0), (37, 3, 1, 0, 0, 0), (42, 3, 0, 0, 0, 0), (47, 3, 0, 1, 0, 0), (52, 0, 0, 1, 0, 0), (57, 0, 0, 1, 0, 30), (62, 0, 0, 1, 0, 0), (90, 3, 0, 1, 8, 8), (91, 3, 0, 1, 8, 8), (97, 3, 1, 1, 8, 8), (102, 0, 0, 1, 0, 0)]
WORLD = (0, 104, 960, 476)
DIRECTIONS = [(math.cos(i * math.tau / 16), math.sin(i * math.tau / 16)) for i in range(16)]

@lru_cache(maxsize=8)
def circle(radius):
    # Independent point-in-rectangle union, matching the documented circle primitive.
    half = radius * math.sin(math.pi / 16)
    result = []
    for y in range(WORLD[1], WORLD[3]):
        dy = y + 0.5 - 270
        for x in range(960):
            dx = x + 0.5 - 480
            if abs(dx) > radius + half or abs(dy) > radius + half:
                result.append(-2.0)
                continue
            result.append(max(min(dx*c+dy*s, radius-dx*c-dy*s, half-abs(-dx*s+dy*c)) for c,s in DIRECTIONS))
    return result

def color(pixel, kind):
    r,g,b = pixel
    if kind == 'green': return g > r+40 and g > b+20
    if kind == 'dark': return max(pixel) < 140
    if kind == 'warm': return r > b+30 and g > 140
    if kind == 'cool': return b > r+30 and g > 140
    if kind == 'ink': return r > 170 and g > 120 and b > 220
    if kind == 'ring': return r > 80 and g < 80 and b > 160
    raise AssertionError(kind)

def check_state(path):
    data = json.loads(path.read_text())
    assert data['summary']['status'] == 'PASSED' and data['frames_executed'] == 103
    assert [row['frame'] for row in data['snapshots']] == [row[0] for row in CASES]
    checked = 0
    for snap, (frame, mode, mask, clear, tick, rotation) in zip(data['snapshots'], CASES):
        wanted = dict(mode=mode, mask=mask, clear_depth=clear, paused=1, tick=tick, rotation=rotation)
        for phase in ['pre', 'post']:
            for name, value in wanted.items():
                actual = snap[phase]['$'+name]
                assert actual == {'U32': value}, (frame, phase, name, actual, value)
                checked += 1
    print('PASS exact typed snapshot fields:', checked)

def check_pixels(paths):
    assert len(paths) == len(CASES)
    images = [Image.open(path).convert('RGB') for path in paths]
    assert all(image.size == (960,540) for image in images)
    # Header and controls must be present and identical after every stencil mode.
    for box in [(0,0,960,35), (0,476,960,540)]:
        reference = images[0].crop(box)
        assert sum(min(p)>220 for p in reference.getdata()) > 200, ('missing HUD',box)
        assert all(image.crop(box).tobytes() == reference.tobytes() for image in images), ('HUD leak',box)
    for i in [10,12,16]: assert images[i].tobytes() == images[0].tobytes(), ('reset/wrap',i)
    assert images[13].tobytes() == images[14].tobytes(), 'Pause did not freeze'
    worlds = [list(image.crop(WORLD).getdata()) for image in images]
    assert worlds[6] == worlds[7], '2D ink uses Always depth compare, not 3D occlusion'
    for a,b in [(0,11),(9,13),(6,15)]:
        changed=sum(x!=y for x,y in zip(worlds[a],worlds[b]))
        assert changed>1000, ('rotation/animation inactive',a,b,changed)
    inner = circle(147.0)
    changed = [i for i,(a,b) in enumerate(zip(worlds[8],worlds[9])) if a!=b]
    assert len(changed)>1000, ('3D depth-clear toggle has no visible witness',len(changed))
    assert all(inner[i]>=-1 for i in changed), 'Depth clear changed color outside aperture'
    for i in [0,3,4,8,9,11,13]:
        assert sum(max(p)>140 and max(p)-min(p)>40 for p in worlds[i])>1000, ('missing 3D scene',i)
    for i,p in enumerate(worlds[0]):
        if circle(200.0)[i]<-1: assert color(p,'dark'), ('scene outside circle',i,p)
    checked=0
    for index,(frame,mode,mask,clear,tick,rotation) in enumerate(CASES):
        if not mask: continue
        radius=150.0+math.sin(tick/60*3)*30
        small=circle(200.0 if mode==0 else 250.0 if mode==1 else radius-3)
        large=circle(radius+3) if mode==3 else None
        counts={}
        for i,pixel in enumerate(worlds[index]):
            margin=small[i]
            if mode==2:
                x=i%960+0.5; y=i//960+104.5
                margin=(1-x/960-y/540)/math.hypot(1/960,1/540)
                kind='warm' if margin>0 else 'cool'
            elif mode==3:
                if abs(margin)<=1 or abs(large[i])<=1: continue
                kind='ink' if margin>0 else 'ring' if large[i]>0 else 'green'
            else:
                kind='green' if margin>0 else 'dark'
            if abs(margin)<=1: continue
            assert color(pixel,kind), (frame,i%960,i//960+104,kind,pixel,margin)
            counts[kind]=counts.get(kind,0)+1
            checked+=1
        assert min(counts.values())>1000, (frame,counts)
        print('MASK',frame,counts)
    print('PASS native mask classifications:',checked,'3D depth-change pixels:',len(changed))

if __name__ == '__main__':
    source=(Path(__file__).parent/'src/lib.rs').read_text()
    template=re.search(r'"(View=[^"\n]+)"',source).group(1)
    widest=template.format('mask ink',1,4294967295,1)
    assert 16+len(widest)*10<=944, 'Status line clips at maximum tick'
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state',type=Path)
    parser.add_argument('images',nargs='*',type=Path)
    args=parser.parse_args()
    if args.state: check_state(args.state)
    else: check_pixels(args.images)
