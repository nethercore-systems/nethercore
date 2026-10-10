#!/usr/bin/env python
"""Independent orthographic ray/coverage oracle for the fixed blob-shadow scene.

Freeze before GPU output: a one-pixel band around analytic opaque/inlay geometry
edges and 2e-5 texel tie tolerance only. No alpha/dither-edge error budget or RGB
fitting. Every other scene-interior pixel is classified, including the floor edge.
"""
import argparse
import json
import re
import tomllib
from pathlib import Path
from PIL import Image

RESET=dict(lane=0,bad_depth=0,write_depth=0,shadow=1,paused=1,tick=30)
BAYER=((0,8,2,10),(12,4,14,6),(3,11,1,9),(15,7,13,5))
BACKGROUND,FLOOR,ACTOR,PROP,SHADOW,INLAY=range(6)
GEOMETRY_EDGE_PX=1.0
TEXEL_TIE=2e-5
VIEW_Y=.75
VIEW_Z=.5
DETERMINANT=VIEW_Y**2+VIEW_Z**2
SHADOW_Y=1/1024
INLAY_Y=1/4096


def cases():
    script=tomllib.loads(Path(__file__).with_name('witness.ncrs').read_text())
    rows={r['f']:r for r in script['frames']};state=RESET.copy();previous=set();result={}
    for frame in range(max(rows)+1):
        row=rows.get(frame,{})
        held=set(row.get('p1','').replace('+',' ').split());pressed=held-previous
        previous=held;pre=state.copy()
        if 'y' in pressed:state=RESET.copy()
        else:
            if 'a' in pressed:state['lane']=(state['lane']+1)%3
            for b,key in [('b','bad_depth'),('r','write_depth'),('l','shadow'),('x','paused')]:
                if b in pressed:state[key]^=1
            if not state['paused']:state['tick']=(state['tick']+1)%2**32
        if row.get('snap'):result[frame]={'pre':pre,'post':state.copy()}
    return result,max(rows)+1


def check_state(path):
    expected,frames=cases();report=json.loads(path.read_text())
    assert report['summary']['status']=='PASSED' and report['frames_executed']==frames
    assert [s['frame'] for s in report['snapshots']]==list(expected)
    count=0
    for s in report['snapshots']:
        for phase in ['pre','post']:
            for key,value in expected[s['frame']][phase].items():
                assert s[phase]['$'+key]=={'U32':value},(s['frame'],phase,key,s[phase]['$'+key],value)
                count+=1
    print('PASS independent blob pre/post fields:',count)


def project_box(box):
    x0,x1,y0,y1,z0,z1=box
    return (480+64*x0,480+64*x1,296-64*(VIEW_Y*y1-VIEW_Z*z0),296-64*(VIEW_Y*y0-VIEW_Z*z1))


def scene(state):
    t=state['tick']%64;b=t if t<=32 else 64-t
    x=(-1,4,5.5)[state['lane']]+b/32;h=b/16;r=1+b/64;level=12-b//8
    actor=(x-.35,x+.35,h,h+.75,-.75,0)
    prop=(-.2,.8,0,1.15,.3,1.1)
    marker=(max(-5,x-.5),min(5,x-.4375),INLAY_Y,INLAY_Y,-1.4,1.2)
    boxes=[project_box(v) for v in [(-5,5,0,0,-3,3),actor,prop]]
    if marker[0]<marker[1]:boxes.append(project_box(marker))
    return dict(state=state,x=x,r=r,level=level,actor=actor,prop=prop,marker=marker,edges=boxes)


def near_edge(px,py,boxes):
    for x0,x1,y0,y1 in boxes:
        if x0-1<=px<=x1+1 and y0-1<=py<=y1+1:
            if min(abs(px-x0),abs(px-x1),abs(py-y0),abs(py-y1))<GEOMETRY_EDGE_PX:return True
    return False


def box_hit(x,q,box):
    x0,x1,y0,y1,z0,z1=box
    if not x0<x<x1:return None
    low=max((DETERMINANT*y0-VIEW_Y*q)/VIEW_Z,(DETERMINANT*z0+VIEW_Z*q)/VIEW_Y)
    high=min((DETERMINANT*y1-VIEW_Y*q)/VIEW_Z,(DETERMINANT*z1+VIEW_Z*q)/VIEW_Y)
    return high if low<=high else None


def expected_pixel(px,py,s):
    cx=px+.5;cy=py+.5
    if near_edge(cx,cy,s['edges']):return None
    x=(cx-480)/64;q=(296-cy)/64
    z=-q/VIEW_Z;depth=-float('inf');kind=BACKGROUND
    if -5<x<5 and -3<z<3:depth=VIEW_Y*z;kind=FLOOR
    for box,color in [(s['actor'],ACTOR),(s['prop'],PROP)]:
        hit=box_hit(x,q,box)
        if hit is not None and hit>=depth:depth=hit;kind=color
    if s['state']['shadow']:
        z=(VIEW_Y*SHADOW_Y-q)/VIEW_Z
        if -5<x<5 and -3<z<3 and abs(x-s['x'])<s['r'] and abs(z+.375)<s['r']:
            u=(x-s['x']+s['r'])*16/s['r'];v=(z+.375+s['r'])*16/s['r']
            if min(abs(u-round(u)),abs(v-round(v)))<TEXEL_TIE:return None
            dx=2*int(u)-31;dz=2*int(v)-31
            a=max(0,900-dx*dx-dz*dz)*255//900
            coverage=a*s['level']*16>BAYER[py%4][px%4]*255*15
            sd=VIEW_Z*SHADOW_Y+VIEW_Y*z
            if coverage and (s['state']['bad_depth'] or sd>=depth):
                kind=SHADOW
                if s['state']['write_depth']:depth=sd
    x0,x1,_,_,z0,z1=s['marker'];z=(VIEW_Y*INLAY_Y-q)/VIEW_Z
    if x0<x<x1 and z0<z<z1:
        md=VIEW_Z*INLAY_Y+VIEW_Y*z
        if md>=depth:kind=INLAY
    return kind


def color(p,kind):
    r,g,b=p
    return [70<max(p)<140,155<min(p) and max(p)<220 and max(p)-min(p)<45,
            b>r+30 and g>r+35 and b>g+10,g>r+40 and g>b+30,
            max(p)<70,min(p)>240][kind]


def check_frame(image,state,frame):
    assert image.size==(960,540)
    pixels=image.load();s=scene(state);checked=skipped=border=0;counts=[0]*6
    for y in range(120,472):
        for x in range(32,928):
            p=pixels[x,y]
            if x<34 or x>=926 or y<122 or y>=470:
                assert min(p)>190 and max(p)-min(p)<5,('border',frame,x,y,p)
                border+=1;continue
            wanted=expected_pixel(x,y,s)
            if wanted is None:skipped+=1;continue
            assert color(p,wanted),('world/coverage',frame,x,y,wanted,p)
            checked+=1;counts[wanted]+=1
    assert checked/(checked+skipped)>.95,('Too much excluded geometry',frame,checked,skipped)
    return checked,skipped,border,counts


def check_pixels(paths):
    expected,_=cases();assert len(paths)==len(expected)
    images=[Image.open(p).convert('RGB') for p in paths];frames=dict(zip(expected,images))
    for box in [(0,0,960,68),(0,484,960,540)]:
        base=images[0].crop(box)
        assert sum(min(p)>220 for p in base.getdata())>100,('missing HUD',box)
        assert all(i.crop(box).tobytes()==base.tobytes() for i in images),('HUD pollution',box)
    for group in [(0,8,16,24,36,78,79),(74,75)]:
        assert all(frames[f].tobytes()==frames[group[0]].tobytes() for f in group),('reset/hold',group)
    for a,b in [(40,41),(41,42),(52,62)]:
        roi=(32,120,928,472)
        assert sum(x!=y for x,y in zip(frames[a].crop(roi).getdata(),frames[b].crop(roi).getdata()))>100,('motion absent',a,b)
    roi=(32,120,928,472)
    first=list(frames[0].crop(roi).getdata())
    erased_green=sum(color(a,PROP) and color(b,SHADOW) for a,b in zip(first,frames[4].crop(roi).getdata()))
    erased_inlay=sum(color(a,INLAY) and color(b,SHADOW) for a,b in zip(first,frames[12].crop(roi).getdata()))
    assert erased_green>50 and erased_inlay>20,('Counterexamples not discriminating',erased_green,erased_inlay)
    totals=[0]*3;counts=[0]*6
    for (frame,snap),image in zip(expected.items(),images):
        result=check_frame(image,snap['post'],frame)
        for i in range(3):totals[i]+=result[i]
        for i in range(6):counts[i]+=result[3][i]
    assert all(c>100 for c in counts),counts
    print('PASS blob native images:',len(images),'classified/edge-excluded/border:',totals,
          'classes:',counts,'depth-test counterexample:',erased_green,'depth-write counterexample:',erased_inlay)


def preflight():
    assert 896/(2*7)==352/(2*2.75)==64
    scenarios=[scene(RESET),scene(dict(RESET,bad_depth=1)),scene(dict(RESET,write_depth=1))]
    differences=[0,0]
    for y in range(220,355):
        for x in range(400,560):
            a,b,c=[expected_pixel(x,y,s) for s in scenarios]
            differences[0]+=a==PROP and b==SHADOW
            differences[1]+=a==INLAY and c==SHADOW
    assert differences[0]>50 and differences[1]>20,differences
    margins=[]
    for tick in range(64):
        s=scene(dict(RESET,tick=tick));x=s['x'];r=s['r']
        bounds=project_box((x-r,x+r,SHADOW_Y,SHADOW_Y,-.375-r,-.375+r))
        for lo,hi in [(bounds[0],bounds[1]),(bounds[2],bounds[3])]:
            assert abs(lo*256-round(lo*256))<1e-8 and abs(hi*256-round(hi*256))<1e-8
            for pixel in range(int(lo),int(hi)+1):
                t=(pixel+.5-lo)*32/(hi-lo)
                if not 0<t<32:continue
                distance=abs(t-round(t))
                if distance>TEXEL_TIE:margins.append(distance)
    assert min(margins)>1/512,('Near-boundary non-tie sample',min(margins))
    print('All motion phases: projected edges on 1/256 grid; minimum non-tie texel margin:',min(margins))
    expected,n=cases()
    assert {v['post']['lane'] for v in expected.values()}=={0,1,2}
    assert any(v['pre']['tick']!=v['post']['tick'] for v in expected.values())
    print('Frozen CPU contract:',len(expected),'captures,',n,'frames; counterexample samples:',differences)


if __name__=='__main__':
    source=(Path(__file__).parent/'src/lib.rs').read_text()
    for prefix in ['Blue actor /','Known flat receiver;']:
        text=re.search(r'"('+re.escape(prefix)+r'[^"\n]+)"',source).group(1)
        assert 16+len(text)*10<=944,('Footer clips',text)
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state',type=Path)
    parser.add_argument('--clean-depth',type=Path)
    parser.add_argument('--preflight',action='store_true')
    parser.add_argument('images',type=Path,nargs='*')
    args=parser.parse_args()
    if args.state:check_state(args.state)
    elif args.clean_depth:check_frame(Image.open(args.clean_depth).convert('RGB'),RESET,'negative')
    elif args.preflight:preflight()
    else:check_pixels(args.images)
