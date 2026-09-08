"""Original mono/stereo forward/ping-pong loop controls, existing packed pipeline."""
from pathlib import Path
import array, json, struct, sys, wave
probe=Path(__file__).with_name('xm_offset_probe.py')
ns={'__file__':str(probe),'__name__':'loop_probe'}
exec(compile(probe.read_text().split('results=[]')[0],str(probe),'exec'),ns)
c,ROOT,stage_root,renderer,native,reference=map(ns.get,('c','ROOT','stage_root','renderer','native','reference'))
template=(ROOT/'target/tracker-compatibility/xm-e9-target-standard-up-after-v10/1-fine-128-e5None/probe.xm').read_bytes()
header,packing,rows,size=struct.unpack_from('<IBHH',template,336)
assert (header,packing,rows,size)==(9,0,8,40)
ins=336+header+size;sh=ins+struct.unpack_from('<I',template,ins)[0]
def measure(path):
    with wave.open(str(path)) as w:
        assert w.getframerate()==44100 and w.getnchannels()==2
        pcm=array.array('h',w.readframes(w.getnframes()))
    if sys.byteorder!='little':pcm.byteswap()
    block=pcm[4410*2:17640*2];left=block[::2]
    import numpy as np
    signal=np.asarray(left[:4096],dtype=float);signal-=signal.mean()
    maximum=int(length*44100/8363*2.4)+2
    scores=[float(np.dot(signal[:-lag],signal[lag:])/np.sqrt(np.dot(signal[:-lag],signal[:-lag])*np.dot(signal[lag:],signal[lag:]))) for lag in range(1,maximum)]
    peaks=[i for i in range(max(1,int(length*44100/8363*.6)),len(scores)-1) if scores[i]>scores[i-1] and scores[i]>=scores[i+1]]
    assert peaks,(path,scores)
    best=max(scores[i] for i in peaks)
    i=next(i for i in peaks if scores[i]>=best*.99)
    period=i+1+(scores[i-1]-scores[i+1])/(2*(scores[i-1]-2*scores[i]+scores[i+1]))
    return {'means':[sum(block[ch::2])/len(block[ch::2]) for ch in (0,1)],'period':period,'period_correlation':scores[i],'max_step':max(abs(a-b) for a,b in zip(left,left[1:])),'clipped':sum(abs(x)>=32767 for x in pcm)}
results=[]
for bits in (8,16):
 for stereo in (False,True):
  for loop_type in (1,2):
   for length in (8,64):
    stage=stage_root/f'{bits}-{int(stereo)}-{loop_type}-{length}';stage.mkdir()
    data=bytearray(template[:sh+40]);data[345:385]=bytes([49,1,0,0,0]+[0]*35)
    def plane(sign):
        values=[sign*(i//2)*(256 if bits==16 else 1) for i in range(128)]
        previous=0;out=bytearray()
        for value in values:
            delta=value-previous;previous=value
            out.extend(bytes([delta&255]) if bits==8 else struct.pack('<H',delta&65535))
        return out
    payload=plane(1)+(plane(-1) if stereo else b'')
    stride=(bits//8)*(2 if stereo else 1)
    struct.pack_into('<III',data,sh,len(payload),4*stride,length*stride)
    data[sh+12:sh+18]=bytes([32,0,loop_type|(16 if bits==16 else 0)|(32 if stereo else 0),128,0,0]);data[sh+18:sh+40]=b' '*22
    source=stage/'probe.xm';source.write_bytes(data+payload)
    capture=c.capture_case(native.capture,ROOT,renderer,stage,source,1.2,stage/'native.wav')
    c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','0','--output',stage/'reference.wav','--',source])
    a,b=measure(stage/'native.wav'),measure(stage/'reference.wav')
    item={'bits':bits,'stereo':stereo,'loop_type':loop_type,'length':length,'source_sha256':c.digest(source),'capture':capture,'native':a,'reference':b}
    item['pass']=not capture['playing'] and not a['clipped'] and abs(a['period']/b['period']-1)<.006 and all(x*y>0 and abs(x/y-1)<.02 for x,y in zip(a['means'],b['means']))
    item['pass'] = item['pass'] and min(a['period_correlation'],b['period_correlation'])>.95 and (loop_type!=2 or a['max_step']<=b['max_step']+1)
    results.append(item);print(json.dumps(item),flush=True)
    (stage_root/'report.json').write_text(json.dumps({'renderer':ns['identity'],'reference_sha256':c.digest(reference),'scope':'authored mono/stereo forward and ping-pong loops; no universal scope','cases':results},indent=2))
assert len(results)==16 and all(x['pass'] for x in results),'Retained loop mismatch'
