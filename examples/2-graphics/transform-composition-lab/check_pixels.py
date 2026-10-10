#!/usr/bin/env python
"""Independent scalar projection of real native box bounds (Python + Pillow)."""
import argparse
import itertools
import json
import math
import tomllib
from pathlib import Path
from PIL import Image

ANGLES = [1,2,3,4,5,6,7,0,1]
STATES = [(s,0) for s in ANGLES]*2 + [(1,1),(2,1),(3,1),(1,0)]


def joints(step, flip):
    angle = math.radians(step*45)
    child = angle + math.radians(-45 if flip else 45)
    elbow = (7+1.5*math.cos(angle),1.5*math.sin(angle))
    tip = (elbow[0]+math.cos(child),elbow[1]+math.sin(child))
    return angle,child,elbow,tip


def projected_box(center, angle, size):
    c,s = math.cos(angle),math.sin(angle)
    points=[]
    for signs in itertools.product((-0.5,0.5),repeat=3):
        u,v,z = [side*extent for side,extent in zip(signs,size)]
        x,y = center[0]+c*u-s*v,center[1]+s*u+c*v
        points.append((480+270*x/(8-z),270-270*y/(8-z)))
    return (min(p[0] for p in points),min(p[1] for p in points),
            max(p[0] for p in points),max(p[1] for p in points))


def colored(pixel, kind):
    r,g,b = pixel
    if kind=='cyan': return g-r>30 and b-r>30
    if kind=='gold': return r-b>30 and g-b>30
    return g-r>25 and g-b>10


def check(paths):
    assert len(paths)==len(STATES), 'Expected 22 captures in replay order'
    images=[Image.open(path).convert('RGB') for path in paths]
    assert all(i.size==(960,540) for i in images)
    for index,(image,(step,flip)) in enumerate(zip(images,STATES)):
        a,b,elbow,tip=joints(step,flip)
        c,s=math.cos(a),math.sin(a)
        orbit=(2*c,2*s)
        upper=projected_box((7+0.75*c,0.75*s),a,(1.5,0.25,0.35))
        lower=projected_box((elbow[0]+0.5*math.cos(b),elbow[1]+0.5*math.sin(b)),b,(1,0.22,0.3))
        green=tuple(min(x,y) if n<2 else max(x,y) for n,(x,y) in enumerate(zip(upper,lower)))
        expected={'cyan':projected_box((-5,0),a,(1.5,0.4,0.4)),
                  'gold':projected_box(orbit,a,(1.5,0.4,0.4)), 'green':green}
        # Separate labeled panels: lit gray crane surfaces can also look cyan.
        # Keep the geometric tolerance unchanged; classify only the named object.
        panels={'cyan':(100,142,352,378),'gold':(368,142,616,378),'green':(624,142,860,378)}
        for kind,bounds in expected.items():
            box=panels[kind]
            scene=image.crop(box)
            pixels=scene.load()
            hit=[(x+box[0],y+box[1]) for y in range(scene.height) for x in range(scene.width)
                 if colored(pixels[x,y],kind)]
            assert 100<len(hit)<4000,(index,kind,'visible coverage',len(hit))
            actual=(min(x for x,y in hit),min(y for x,y in hit),max(x for x,y in hit),max(y for x,y in hit))
            assert all(abs(a-e)<=3 for a,e in zip(actual,bounds)),(index,kind,actual,bounds)
        for x,y in [(-7,0),(-5,0),(0,0),orbit,(7,0),elbow,tip]:
            px,py=round(480+270*x/8),round(270-270*y/8)
            crop=image.crop((px-2,py-2,px+3,py+3))
            assert sum(p==(255,255,255) for p in crop.getdata())>=5,(index,'joint cross',x,y)
    scene=(100,142,860,378)
    for api,explicit in zip(images[:9],images[9:18]):
        a,b=list(api.crop(scene).getdata()),list(explicit.crop(scene).getdata())
        differences=sum(x!=y for x,y in zip(a,b))
        assert differences<=len(a)*0.005,('API/matrix disagreement',differences,len(a))
    assert images[0].tobytes()==images[8].tobytes()==images[-1].tobytes(),'Wrap/reset must be exact'
    assert images[9].tobytes()==images[17].tobytes(),'Matrix-mode wrap must be exact'
    left=(100,142,620,378)
    assert images[17].crop(left).tobytes()==images[18].crop(left).tobytes(),'Child toggle moved comparison bars'
    assert images[17].crop(scene).tobytes()!=images[18].crop(scene).tobytes(),'Child toggle did nothing'
    print('PASS 22 native frames: both orders, 8 angles in both paths, scalar box projections, joints, independent child, wrap/reset')


def check_state(report):
    data=json.loads(report.read_text())
    script=tomllib.loads(Path(__file__).with_name('witness.ncrs').read_text())
    frames={f['f']:f for f in script['frames']}
    snapshots={s['frame']:s['post'] for s in data['snapshots']}
    assert set(snapshots)=={f['f'] for f in script['frames'] if f.get('snap')}
    step,flip,explicit=1,0,0
    previous=set()
    checked=0
    for frame in range(max(frames)+1):
        held=set(frames.get(frame,{}).get('p1','').split())
        pressed=held-previous
        if 'a' in pressed: step=(step+1)%8
        if 'b' in pressed: flip^=1
        if 'x' in pressed: explicit^=1
        if 'y' in pressed: step,flip,explicit=1,0,0
        previous=held
        if frame not in snapshots: continue
        a,_,elbow,tip=joints(step,flip)
        expected={'step':step,'flip':flip,'explicit':explicit,
                  'right_x':2*math.cos(a),'right_y':2*math.sin(a),
                  'elbow_x':elbow[0],'elbow_y':elbow[1],'tip_x':tip[0],'tip_y':tip[1]}
        for name,value in expected.items():
            actual=next(iter(snapshots[frame]['$'+name].values()))
            assert abs(actual-value)<0.0001,(frame,name,actual,value)
            checked+=1
    print('PASS independent replay-input/trigonometric oracle:',checked,'snapshot fields')


if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('images',nargs=22,type=Path)
    parser.add_argument('--report',type=Path,help='Defaults to the shared runner report beside the images')
    args=parser.parse_args()
    check_state(args.report or args.images[0].with_name('transform-composition-lab.json'))
    check(args.images)
