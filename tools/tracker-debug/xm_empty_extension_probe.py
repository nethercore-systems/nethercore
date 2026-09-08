"""Test empty/opaque extension containers on retained original-authored controls."""
from pathlib import Path
import hashlib
bootstrap=Path(__file__).with_name('xm_offset_probe.py').read_text().split('results=[]')[0]
exec(compile(bootstrap,'existing-offset-bootstrap','exec'))
source_stage='xm-slide-integrated-legacy-v1' if args.legacy else 'xm-slide-integrated-standard-v1'
original=ROOT/'target/tracker-compatibility'/source_stage/'0-17-1-0/probe.xm'
data=original.read_bytes()
results=[]; baseline={}; baseline_endpoints={}
for name,tail in [('baseline',b''),('song-empty',b'STPM'),('instrument-empty',b'XTPM'),('instrument-song-empty',b'XTPMSTPM'),('song-instrument-empty',b'STPMXTPM'),('song-opaque',b'STPMzzzz\x01\x00\x12'),('instrument-opaque',b'XTPMzzzz\x01\x00\x12')]:
 stage=stage_root/name;stage.mkdir();module=stage/'probe.xm';module.write_bytes(data+tail)
 capture=c.capture_case(native.capture,ROOT,renderer,stage,module,1.2,stage/'native.wav')
 c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','0','--output',stage/'reference.wav','--',module])
 hashes={}; endpoints={}
 for player in ('native','reference'):
  with wave.open(str(stage/(player+'.wav'))) as w:pcm=w.readframes(w.getnframes())
  hashes[player]=hashlib.sha256(pcm).hexdigest()
  decoded=array.array("h",pcm)
  endpoints[player]=[decoded[2*((tick+1)*882-2)+ch] for tick in range(48) for ch in (0,1)]
 if not baseline:baseline=hashes;baseline_endpoints=endpoints
 item={'name':name,'source_sha256':c.digest(module),'capture':capture,'pcm_sha256':hashes,'whole_pcm_equal':hashes==baseline,'endpoints':endpoints,'pass':endpoints==baseline_endpoints and not capture['playing']}
 results.append(item)
 (stage_root/'report.json').write_text(json.dumps({'reference_sha256':c.digest(reference),'source_control_sha256':c.digest(original),'renderer':identity,'cases':results},indent=2))
 print(name,item['pass'],flush=True)
assert len(results)==7 and all(x['pass'] for x in results)
