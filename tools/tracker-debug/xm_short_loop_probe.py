"""Original high-rate short-loop controls using existing packed playback helpers."""
from pathlib import Path
import array,json,os,struct,sys,wave
reference_filter=os.environ.get("XM_REFERENCE_FILTER", "8")
assert reference_filter in ("1", "2", "8")
probe=Path(__file__).with_name('xm_offset_probe.py')
exec(compile(probe.read_text().split('results=[]')[0],str(probe),'exec'))
template=(ROOT/'target/tracker-compatibility/xm-e9-target-standard-up-after-v10/1-fine-128-e5None/probe.xm').read_bytes()
header,packing,rows,size=struct.unpack_from('<IBHH',template,336)
assert (header,packing,rows,size)==(9,0,8,40)
ins=336+header+size;sh=ins+struct.unpack_from('<I',template,ins)[0]
results=[]
for stereo in (False,True):
 for kind in (1,2):
  for length in (1,2):
   for note in (49,85,96):
    stage=stage_root/f'{int(stereo)}-{kind}-{length}-{note}';stage.mkdir()
    data=bytearray(template[:sh+40]);data[345:385]=bytes([note,1,0,0,0]+[0]*35)
    values=[8192 if 4<=i<4+length else 0 for i in range(32)]
    def plane(sign):
     previous=0;encoded=bytearray()
     for value in values:
      v=value*sign;encoded.extend(struct.pack('<H',(v-previous)&65535));previous=v
     return encoded
    payload=plane(1)+(plane(-1) if stereo else b'');stride=2*(2 if stereo else 1)
    struct.pack_into('<III',data,sh,len(payload),4*stride,length*stride)
    data[sh+12:sh+18]=bytes([32,0,16|kind|(32 if stereo else 0),128,0,0]);data[sh+18:sh+40]=b' '*22
    # Audible tail control verifies a new low note revives a rate-stopped voice.
    data[380:385]=bytes([49,1,0,0,0])
    source=stage/'probe.xm';source.write_bytes(data+payload)
    capture=c.capture_case(native.capture,ROOT,renderer,stage,source,1.2,stage/'native.wav')
    c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter',reference_filter,'--ramping','-1','--repeat','0','--subsong','0','--output',stage/'reference.wav','--',source])
    item={'stereo':stereo,'kind':kind,'length':length,'note':note,'source_sha256':c.digest(source),'capture':capture}
    for player in ('native','reference'):
     with wave.open(str(stage/(player+'.wav'))) as w:pcm=array.array('h',w.readframes(w.getnframes()))
     if sys.byteorder!='little':pcm.byteswap()
     block=pcm[4410*2:8820*2]
     item[player]={'means':[sum(block[ch::2])/len(block[ch::2]) for ch in (0,1)],'clipped':sum(abs(x)>=32767 for x in pcm)}
    a,b=item['native']['means'],item['reference']['means']
    item['reference_audible']=abs(b[0])>10
    item['pass']=not capture['playing'] and all(abs(x-y)/max(1,abs(y))<.02 for x,y in zip(a,b)) and not item['native']['clipped']
    results.append(item);print(json.dumps(item),flush=True)
    (stage_root/'report.json').write_text(json.dumps({'renderer':identity,'reference_filter':reference_filter,'reference_sha256':c.digest(reference),'cases':results},indent=2))
assert len(results)==24 and all(x['pass'] for x in results),'Retained short-loop failures'
