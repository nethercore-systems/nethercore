#!/usr/bin/env python
"""Frozen native contract: 960x540, 2P/4P rectangles, local UI and stencil scissor.

One-pixel analytic mask-boundary band only; fixed color classes, no fitted colors.
All non-UI ink pixels and every viewport border pixel are tested. Camera effects
must stay in their own viewport; unchanged regions/reset/freeze are exact.
"""
import argparse
import json
import math
from functools import lru_cache
from pathlib import Path
from PIL import Image

CASES = [{'frame': 0, 'quad': 0, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 6, 'quad': 0, 'mask': 0, 'ink': 1, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 10, 'quad': 0, 'mask': 1, 'ink': 1, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 14, 'quad': 1, 'mask': 1, 'ink': 1, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 18, 'quad': 1, 'mask': 0, 'ink': 1, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 22, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 26, 'quad': 1, 'mask': 1, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 30, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 34, 'quad': 0, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 38, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 47, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [12.0, 84.0, 180.0, 274.0]}, {'frame': 52, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [12.0, 84.0, 178.0, 274.0]}, {'frame': 63, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 6, 'angles': [12.0, 84.0, 178.0, 274.0]}, {'frame': 64, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 6, 'angles': [12.0, 84.0, 178.0, 274.0]}, {'frame': 70, 'quad': 0, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 74, 'quad': 1, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 78, 'quad': 1, 'mask': 1, 'ink': 1, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}, {'frame': 82, 'quad': 0, 'mask': 0, 'ink': 0, 'paused': 1, 'tick': 0, 'angles': [0.0, 90.0, 180.0, 270.0]}]

def rects(state):
    return [(0,64,480,428),(480,64,480,428)] if not state['quad'] else [
        (0,64,480,214),(480,64,480,214),(0,278,480,214),(480,278,480,214)]

def ink(pixel, player):
    r,g,b=pixel
    return [b>r+70 and g>r+60, r>b+70 and g>b+50,
            r>g+40 and b>g+60, g>r+40 and g>b+30][player]

@lru_cache(maxsize=2)
def mask_margin(h):
    radius=min(h*0.75,270)
    half=radius*math.sin(math.pi/16)
    axes=[(math.cos(i*math.tau/16),math.sin(i*math.tau/16)) for i in range(16)]
    return [max(min((x+0.5-120)*c+(y+0.5-h/2)*s,
                   radius-(x+0.5-120)*c-(y+0.5-h/2)*s,
                   half-abs(-(x+0.5-120)*s+(y+0.5-h/2)*c)) for c,s in axes)
            for y in range(h) for x in range(480)]

def check_state(path):
    data=json.loads(path.read_text())
    assert data['summary']['status']=='PASSED' and data['frames_executed']==83
    assert [s['frame'] for s in data['snapshots']]==[c['frame'] for c in CASES]
    count=0
    for snap,wanted in zip(data['snapshots'],CASES):
        for phase in ['pre','post']:
            for key in ['quad','mask','ink','paused','tick']:
                assert snap[phase]['$'+key]=={'U32':wanted[key]},(snap['frame'],phase,key,snap[phase]['$'+key],wanted[key])
                count+=1
            for i,angle in enumerate(wanted['angles']):
                value=snap[phase]['$cam'+str(i+1)]
                assert set(value)=={'F32'} and abs(value['F32']-angle)<=0.001,(snap['frame'],phase,i,value,angle)
                count+=1
    print('PASS typed viewport state fields:',count)

def check_pixels(paths):
    assert len(paths)==len(CASES)
    imgs=[Image.open(path).convert('RGB') for path in paths]
    assert all(i.size==(960,540) for i in imgs)
    by_frame={c['frame']:img for c,img in zip(CASES,imgs)}
    for box in [(0,0,460,64),(0,492,960,540)]:
        reference=imgs[0].crop(box)
        assert sum(min(p)>220 for p in reference.getdata())>200,('global HUD absent',box)
        assert all(img.crop(box).tobytes()==reference.tobytes() for img in imgs),('global HUD leak',box)
    for group in [(0,34,70,82),(22,30,38,74),(14,78),(63,64)]:
        assert all(by_frame[f].tobytes()==by_frame[group[0]].tobytes() for f in group),('reset/freeze/layout',group)
    rectangles=rects(CASES[5])
    for before,after,changed_views in [(38,47,{0,1,3}),(47,52,{2}),(52,63,{0,1,2,3})]:
        for p,(x,y,w,h) in enumerate(rectangles):
            box=(x,y,x+w,y+h)
            a=by_frame[before].crop(box);b=by_frame[after].crop(box)
            changed=sum(a!=b for a,b in zip(a.getdata(),b.getdata()))
            assert (changed>200 if p in changed_views else changed==0),('camera ownership/animation',before,after,p,changed)
    checked=0
    for state,img in zip(CASES,imgs):
        allpix=img.load()
        for p,(ox,oy,w,h) in enumerate(rects(state)):
            view=img.crop((ox,oy,ox+w,oy+h));pix=view.load()
            # Every border is exactly local to this viewport, including all corners.
            border=[(x,y) for y in [0,1,h-2,h-1] for x in range(w)]
            border += [(x,y) for x in [0,1,w-2,w-1] for y in range(2,h-2)]
            for x,y in border: assert ink(pix[x,y],p),('border/viewport remap',state['frame'],p,x,y,pix[x,y])
            for box in [(10,8,310,26),(w-34,h-18,w-4,h-8)]:
                assert sum(min(v)>220 for v in view.crop(box).getdata())>15,('local label absent',state['frame'],p,box)
            margins=mask_margin(h)
            inside=outside=0
            for y in range(36,h-2):
                for x in range(2,w-2):
                    if x>=w-44 and y>=h-24:continue  # Declared BR HUD panel.
                    margin=margins[y*w+x] if state['mask'] else 2
                    if abs(margin)<=1:continue
                    value=pix[x,y]
                    if margin<0:
                        assert max(value)<140,('mask spill',state['frame'],p,x,y,value)
                        outside+=1
                    elif state['ink']:
                        assert ink(value,p),('wrong/missing ink',state['frame'],p,x,y,value)
                        inside+=1
                    checked+=1
            if state['ink']:
                assert inside>1000 and (not state['mask'] or outside>1000),(state['frame'],p,inside,outside)
            else:
                body=view.crop((4,36,w-4,h-24))
                assert sum(max(v)>140 and max(v)-min(v)>40 for v in body.getdata())>200,('missing 3D geometry',state['frame'],p)
    print('PASS viewport pixel classifications:',checked,'native images:',len(imgs))

if __name__=='__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--state',type=Path)
    parser.add_argument('images',type=Path,nargs='*')
    args=parser.parse_args()
    if args.state:check_state(args.state)
    else:check_pixels(args.images)
