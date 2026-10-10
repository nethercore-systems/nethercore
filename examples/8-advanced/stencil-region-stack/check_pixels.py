#!/usr/bin/env python
"""Frozen independent region-stack pixel and pre/post update state contract.

All integer-quad interior pixels are classified without an ignored edge band.
Popped child color stays visible; white parent-test strokes prove restoration;
green sibling reuse must not recolor released parents unless cleanup is disabled.
"""
import argparse
import json
import re
import tomllib
from pathlib import Path
from PIL import Image


def cases():
    script=tomllib.loads(Path(__file__).with_name('witness.ncrs').read_text())
    rows={r['f']:r for r in script['frames']}
    reset=dict(deep=1,cleanup=1,paused=1,tick=0)
    state=reset.copy();previous=set();result={}
    for frame in range(max(rows)+1):
        row=rows.get(frame,{})
        buttons=set(row.get('p1','').replace('+',' ').split())
        pressed=buttons-previous;previous=buttons
        pre=state.copy()
        if 'y' in pressed:state=reset.copy()
        else:
            for button,key in [('a','deep'),('b','cleanup'),('x','paused')]:
                if button in pressed:state[key]^=1
            if not state['paused']:state['tick']=(state['tick']+1)%2**32
        if row.get('snap'):result[frame]={'pre':pre,'post':state.copy()}
    return result,max(rows)+1


def check_state(path):
    expected,frames=cases()
    report=json.loads(path.read_text())
    assert report['summary']['status']=='PASSED' and report['frames_executed']==frames
    assert [s['frame'] for s in report['snapshots']]==list(expected)
    count=0
    for snapshot in report['snapshots']:
        for phase in ['pre','post']:
            for key,value in expected[snapshot['frame']][phase].items():
                assert snapshot[phase]['$'+key]=={'U32':value},(snapshot['frame'],phase,key,snapshot[phase]['$'+key],value)
                count+=1
    print('PASS independent stack pre/post fields:',count)


def regions(state,panel):
    move=((state['tick']+panel)&1)*12
    result=[(24,24,268,280),(72,64,172,196),(112+move,112,80,112),(328+move,104,88,196)]
    return [(456-x-w,y,w,h) for x,y,w,h in result] if panel else result


def inside(x,y,r):
    return r[0]<=x+0.5<r[0]+r[2] and r[1]<=y+0.5<r[1]+r[3]


def color(pixel,kind):
    r,g,b=pixel
    if kind==-1:return max(pixel)<140
    return [b>r+70 and g>r+60,r>b+70 and g>b+50,
            r>g+40 and b>g+60,g>r+40 and g>b+30,min(pixel)>220][kind]


def check_pixels(paths):
    expected,_=cases()
    assert len(paths)==len(expected)
    images=[Image.open(p).convert('RGB') for p in paths]
    assert all(i.size==(960,540) for i in images)
    frames=dict(zip(expected,images))
    for box in [(0,0,960,68),(0,484,960,540)]:
        base=images[0].crop(box)
        assert sum(min(p)>220 for p in base.getdata())>100,('missing HUD',box)
        assert all(i.crop(box).tobytes()==base.tobytes() for i in images),('HUD spill',box)
    for group in [(0,16,40,52,53),(4,44),(8,48),(36,37)]:
        assert all(frames[f].tobytes()==frames[group[0]].tobytes() for f in group),('reset/hold/pause',group)
    for a,b in [(20,21),(21,22),(22,23),(26,27),(30,31)]:
        first=frames[a].crop((0,120,960,472));second=frames[b].crop((0,120,960,472))
        assert sum(x!=y for x,y in zip(first.getdata(),second.getdata()))>100,('adjacent-frame motion missing',a,b)
    by_state={};checked=border=restored=deliberate_leak=0
    for frame,snapshot in expected.items():
        state=snapshot['post'];image=frames[frame];pixels=image.load()
        key=(state['deep'],state['cleanup'],state['tick']&1)
        body=image.crop((0,90,960,472)).tobytes()
        if key in by_state:assert body==by_state[key],('history-dependent stencil leak',frame,key)
        by_state[key]=body
        for panel in range(2):
            ox=16+472*panel;oy=120
            a,b,c,sibling=regions(state,panel)
            vertical=(a[0]+a[2]/2-3,0,6,352)
            boundary=[(x,y) for y in [0,1,350,351] for x in range(456)]
            boundary += [(x,y) for x in [0,1,454,455] for y in range(2,350)]
            for x,y in boundary:
                p=pixels[ox+x,oy+y]
                assert min(p)>190 and max(p)-min(p)<5,('border',frame,panel,x,y,p)
                border+=1
            for y in range(2,350):
                for x in range(2,454):
                    outer=inside(x,y,a);middle=inside(x,y,b)
                    wanted=-1
                    if outer:wanted=0
                    if middle:wanted=1
                    if state['deep'] and inside(x,y,c):wanted=2
                    if state['deep'] and middle and 168<=y<174:wanted=4
                    if outer and inside(x,y,vertical):wanted=4
                    if inside(x,y,sibling) or (not state['cleanup'] and outer):wanted=3
                    assert color(pixels[ox+x,oy+y],wanted),('stack/reuse pixel',frame,panel,x,y,wanted,pixels[ox+x,oy+y])
                    checked+=1
                    restored+=wanted==4
                    deliberate_leak+=not state['cleanup'] and outer
            assert sum(min(p)>220 for p in image.crop((ox,98,ox+440,112)).getdata())>50,('local label missing',frame,panel)
    assert len(by_state)>=6 and restored>1000 and deliberate_leak>1000
    print('PASS stack native images:',len(images),'interior classifications:',checked,'border:',border,'restored-parent pixels:',restored,'counterexample pixels:',deliberate_leak)


if __name__=='__main__':
    source=(Path(__file__).parent/'src/lib.rs').read_text()
    template=re.search(r'"(CLEAN=[^"\n]+)"',source).group(1)
    assert 16+len(template.format(1,1,4294967295,3,255))*11<=944,'Maximum-tick status clips'
    for prefix in ['Push 1..N;', 'Fixed 3-level recipe;']:
        text=re.search(r'"('+re.escape(prefix)+r'[^"\n]+)"',source).group(1)
        assert 16+len(text)*10<=944,('Footer clips',text)
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state',type=Path)
    parser.add_argument('images',type=Path,nargs='*')
    args=parser.parse_args()
    if args.state:check_state(args.state)
    else:check_pixels(args.images)
