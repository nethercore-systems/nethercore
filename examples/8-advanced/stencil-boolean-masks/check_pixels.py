#!/usr/bin/env python
"""Frozen boolean truth table against every interior pixel of real native panels.

Integer quad bounds, pixel-center membership; no fitted thresholds or boundary
exclusions. Exact reset/freeze/commutativity and independent typed snapshots.
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
    state={'mode':0,'layout':0,'swap':0};previous=set();result={}
    for f in range(max(rows)+1):
        row=rows.get(f,{})
        held=set(row.get('p1','').replace('+',' ').split())
        pressed=held-previous;previous=held
        if 'y' in pressed:state={'mode':0,'layout':0,'swap':0}
        else:
            if 'a' in pressed:state['mode']=(state['mode']+1)%4
            if 'b' in pressed:state['layout']=(state['layout']+1)%3
            if 'x' in pressed:state['swap']^=1
        if row.get('snap'):result[f]=state.copy()
    assert {(s['mode'],s['layout'],s['swap']) for s in result.values()}=={
        (m,l,s) for m in range(4) for l in range(3) for s in range(2)}
    return result,max(rows)+1


def check_state(path):
    expected,length=cases()
    data=json.loads(path.read_text())
    assert data['summary']['status']=='PASSED' and data['frames_executed']==length
    assert [s['frame'] for s in data['snapshots']]==list(expected)
    count=0
    for snap in data['snapshots']:
        for phase in ['pre','post']:
            for key,value in expected[snap['frame']].items():
                assert snap[phase]['$'+key]=={'U32':value},(snap['frame'],phase,key)
                count+=1
    print('PASS independent typed boolean state fields:',count)


def ink(pixel,panel):
    r,g,b=pixel
    return [b>r+70 and g>r+60,r>b+70 and g>b+50,r>g+40 and b>g+60][panel]


def contains(x,y,rect):
    ox,oy,w,h=rect
    return ox<=x+0.5<ox+w and oy<=y+0.5<oy+h


def check_pixels(paths):
    expected,_=cases()
    assert len(paths)==len(expected)
    images=[Image.open(p).convert('RGB') for p in paths]
    assert all(i.size==(960,540) for i in images)
    for box in [(0,0,960,78),(0,487,960,540)]:
        first=images[0].crop(box)
        assert sum(min(p)>220 for p in first.getdata())>100,('HUD absent',box)
        assert all(i.crop(box).tobytes()==first.tobytes() for i in images),('HUD leak',box)
    by_state={};checked=border=holes=0
    for state,image in zip(expected.values(),images):
        key=(state['mode'],state['layout'],state['swap'])
        if key in by_state:assert image.tobytes()==by_state[key].tobytes(),('reset/idle/hold',key)
        by_state[key]=image
        shapes=[(38,42,176,188),[(128,130,122,164),(86,96,80,80),(225,20,45,75)][state['layout']]]
        if state['swap']:shapes.reverse()
        pixels=image.load()
        for panel in range(3):
            ox=24+312*panel;oy=130
            boundary=[(x,y) for y in [0,1,318,319] for x in range(288)]
            boundary += [(x,y) for x in [0,1,286,287] for y in range(2,318)]
            for x,y in boundary:
                assert ink(pixels[ox+x,oy+y],panel),('panel border',key,panel,x,y)
                border+=1
            for y in range(2,318):
                for x in range(2,286):
                    a,b=[contains(x,y,r) for r in shapes]
                    wanted=[a,b,[a or b,a and b,a and not b,a!=b][state['mode']]][panel]
                    pixel=pixels[ox+x,oy+y]
                    if wanted:assert ink(pixel,panel),('missing ink',key,panel,x,y,pixel)
                    else:assert max(pixel)<140,('unexpected color/stencil leak',key,panel,x,y,pixel)
                    checked+=1
                    if panel==2 and state['layout']==1 and a and b and not wanted:holes+=1
        for panel in range(3):
            assert sum(ink(p,panel) for p in image.crop((24+312*panel,108,310+312*panel,122)).getdata())>30,('operand label',key,panel)
    result=(648,130,936,450)
    for layout in range(3):
        for mode in range(4):
            a=by_state[(mode,layout,0)].crop(result).tobytes()
            b=by_state[(mode,layout,1)].crop(result).tobytes()
            assert (a!=b if mode==2 else a==b),('operand order',mode,layout)
    assert holes>0,'Nested subtraction/parity holes not witnessed'
    print('PASS boolean images:',len(images),'interior classifications:',checked,'border pixels:',border,'hole pixels:',holes)


if __name__=='__main__':
    source=(Path(__file__).parent/'src/lib.rs').read_text()
    for prefix in ['Build masks', 'Single-quad shapes']:
        label=re.search(r'"('+re.escape(prefix)+r'[^"\n]+)"',source).group(1)
        assert 24+len(label)*11<=936,('Footer exceeds right margin',label)
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state',type=Path)
    parser.add_argument('images',type=Path,nargs='*')
    args=parser.parse_args()
    if args.state:check_state(args.state)
    else:check_pixels(args.images)
