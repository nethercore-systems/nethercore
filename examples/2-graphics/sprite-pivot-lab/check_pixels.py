#!/usr/bin/env python
"""Check actual scale-1 anchor/rotation/UV captures with independent trig (Pillow)."""
import argparse
import math
from pathlib import Path
from PIL import Image

STATES = [(i,0,0) for i in range(8)] + [(0,0,0),(0,0,1),(0,1,1),(1,1,1),(2,1,1),(0,0,0)]


def colored(pixel, region):
    r,g,b = pixel
    return (g-r > 60 and b-r > 60) if region == 0 else (r-b > 90 and g-b > 90)


def check(paths):
    assert len(paths) == len(STATES), 'Expected all fourteen captures in replay order'
    images = [Image.open(path).convert('RGB') for path in paths]
    assert all(image.size == (960,540) for image in images)
    areas = [[],[]]
    for index,(image,(step,flip,region)) in enumerate(zip(images,STATES)):
        angle = math.radians(step*45)
        child = angle + math.radians(-90 if flip else 90)
        def point(origin,distance,rotation):
            return (origin[0]+distance*math.cos(rotation),origin[1]+distance*math.sin(rotation))
        elbow = point((790,270),80,angle)
        tip = point(elbow,60,child)
        probes = [point((176,270),-50,angle),point((176,270),60,angle),
                  point((480,270),40,angle),point((480,270),135,angle),
                  point((790,270),40,angle),point(elbow,30,child)]
        for x,y in probes:
            # Predeclared 3-pixel raster tolerance; much smaller than a wrong pivot offset.
            patch = image.crop((round(x)-3,round(y)-3,round(x)+4,round(y)+4))
            assert sum(colored(p,region) for p in patch.getdata()) >= 10, (index,'missing rotated shaft',x,y)
        for x,y in [(176,270),(480,270),(790,270),elbow,tip]:
            patch = image.crop((round(x)-2,round(y)-2,round(x)+3,round(y)+3))
            assert sum(p==(255,255,255) for p in patch.getdata()) >= 5, (index,'missing anchor',x,y)
        for area,box in zip(areas,[(24,108,312,448),(320,108,648,448)]):
            coverage = sum(colored(p,region) for p in image.crop(box).getdata())
            assert 3000 < coverage < 9000, (index,'missing/clipped sprite',coverage)
            area.append(coverage)
    for area in areas:
        assert max(area)-min(area) < max(area)*0.08, ('rotation changed visible coverage',area)
    assert images[0].tobytes()==images[8].tobytes()==images[13].tobytes(), '360-degree wrap/reset changed the initial frame'
    box=(24,108,648,448)
    assert images[9].crop(box).tobytes()==images[10].crop(box).tobytes(), 'Child-relative toggle leaked into other sprites'
    assert images[9].crop((656,108,936,448)).tobytes()!=images[10].crop((656,108,936,448)).tobytes(), 'Child-relative angle did not change'
    print('PASS center/edge pivots, 8 rotations, UV selection, relative child, anchor positions, wrap and reset;',areas)


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('images',nargs=14,type=Path)
    check(parser.parse_args().images)
