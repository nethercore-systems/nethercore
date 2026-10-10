#!/usr/bin/env python
"""Frozen per-pixel sprite/stencil coverage oracle, with exact Bayer phases.

All panel pixels, including texture/transparent/dither boundaries, are checked.
Coverage is binary; no image-fitted color, ignored boundary or error band exists.
"""
import argparse
import json
import re
import tomllib
from pathlib import Path
from PIL import Image

BAYER=((0,8,2,10),(12,4,14,6),(3,11,1,9),(15,7,13,5))


def cases():
    script=tomllib.loads(Path(__file__).with_name('witness.ncrs').read_text())
    rows={r['f']:r for r in script['frames']}
    state=dict(alpha=0,offset=0,box_mask=0);previous=set();result={}
    for frame in range(max(rows)+1):
        row=rows.get(frame,{})
        held=set(row.get('p1','').replace('+',' ').split())
        pressed=held-previous;previous=held;pre=state.copy()
        if 'y' in pressed:state=dict(alpha=0,offset=0,box_mask=0)
        else:
            if 'a' in pressed:state['alpha']=(state['alpha']+1)%3
            if 'b' in pressed:state['offset']=(state['offset']+1)%4
            if 'x' in pressed:state['box_mask']^=1
        if row.get('snap'):result[frame]={'pre':pre,'post':state.copy()}
    assert {tuple(s['post'].values()) for s in result.values()}=={(a,o,b) for a in range(3) for o in range(4) for b in range(2)}
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
    print('PASS independent texture stencil pre/post fields:',count)


def alpha(x,y):
    if 12<=x<20 and 12<=y<20:return 0
    if 8<=x<11 and 14<=y<17:return 0
    distance=abs(x-15.5)+abs(y-15.5)
    if distance<=10:return 255
    if distance<=12:return 192
    if distance<=14:return 128
    if distance<=15:return 64
    return 0


def survives(x,y,panel,state,box=False):
    if not (68<=x<388 and 16<=y<336):return False
    a=255 if box else alpha((x-68)//10,(y-16)//10)
    ox=16+472*panel;oy=120;phase=state['offset']
    threshold=BAYER[(oy+y+2*phase)%4][(ox+x+phase)%4]
    # Exact rational comparison against threshold/16; alpha is RGBA8, level/15.
    return a*[15,8,0][state['alpha']]*16>threshold*255*15


def ink(p,kind):
    r,g,b=p
    if kind==-1:return max(p)<140
    if kind==0:return b>r+70 and g>r+60
    if kind==1:return r>b+70 and g>b+50
    return r>g+40 and b>g+60


def check_pixels(paths):
    expected,_=cases()
    assert len(paths)==len(expected)
    images=[Image.open(p).convert('RGB') for p in paths]
    assert all(i.size==(960,540) for i in images)
    for box in [(0,0,960,68),(0,484,960,540),(0,96,960,116)]:
        base=images[0].crop(box)
        assert sum(min(p)>220 for p in base.getdata())>100,('missing HUD',box)
        assert all(i.crop(box).tobytes()==base.tobytes() for i in images),('dither leaked into HUD',box)
    assert images[0].tobytes()==images[-2].tobytes()==images[-1].tobytes(),'reset/hold mismatch'
    checked=border=transparent=fringe=box_difference=0
    for (frame,snapshot),image in zip(expected.items(),images):
        state=snapshot['post'];pixels=image.load()
        for panel in range(2):
            ox=16+472*panel;oy=120
            for y in range(352):
                for x in range(456):
                    p=pixels[ox+x,oy+y]
                    if x<2 or x>=454 or y<2 or y>=350:
                        assert min(p)>190 and max(p)-min(p)<5,('border',frame,panel,x,y,p)
                        border+=1;continue
                    coverage=survives(x,y,panel,state,panel==1 and state['box_mask']==1)
                    wanted=(0 if panel==0 else (2 if x%64<32 else 1)) if coverage else -1
                    assert ink(p,wanted),('coverage/content',frame,panel,x,y,wanted,p)
                    checked+=1
                    if 68<=x<388 and 16<=y<336:
                        a=alpha((x-68)//10,(y-16)//10)
                        transparent+=a==0 and not coverage
                        fringe+=0<a<255
                    if panel==1 and state['box_mask']:
                        box_difference+=coverage!=survives(x,y,panel,state)
    assert transparent>1000 and fringe>1000 and box_difference>1000
    print('PASS texture stencil native images:',len(images),'interior classifications:',checked,
          'border:',border,'transparent:',transparent,'fractional-edge samples:',fringe,'box differences:',box_difference)


if __name__=='__main__':
    source=(Path(__file__).parent/'src/lib.rs').read_text()
    for prefix in ['Cyan sprite writes', 'Holes stay empty.']:
        text=re.search(r'"('+re.escape(prefix)+r'[^"\n]+)"',source).group(1)
        assert 16+len(text)*10<=944,('Footer clips',text)
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state',type=Path)
    parser.add_argument('images',type=Path,nargs='*')
    args=parser.parse_args()
    if args.state:check_state(args.state)
    else:check_pixels(args.images)
