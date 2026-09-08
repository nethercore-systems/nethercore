"""Original delayed-pan and zero-column memory review controls."""
from pathlib import Path
bootstrap=Path(__file__).with_name('xm_offset_probe.py').read_text().split('results=[]')[0]
exec(compile(bootstrap,'existing-offset-bootstrap','exec'))
results=[]
for linear in (0,1):
 source_stage='xm-slide-integrated-legacy-v1' if args.legacy else 'xm-slide-integrated-standard-v1'
 original=(ROOT/'target/tracker-compatibility'/source_stage/f'{linear}-17-1-0/probe.xm').read_bytes()
 assert struct.unpack_from('<IBHH',original,336)==(9,0,8,40)
 cases=[]
 for delay in (1,3,5):
  for direction in (0xd0,0xe0):
   for step in (0,1,15):
    rows=[[49,1,0x30,16,32],[49,1,direction+step,14,0xd0+delay]]+[[0,0,0,0,0] for _ in range(6)]
    cases.append((f'delay-{delay}-{direction}-{step}',rows))
 for pan in (1,16):
  rows=[[49,1,0x30,16,32],[0,0,0,25,pan],[0,0,0xd0,0,0],[0,0,0,25,0],[0,0,0xe0,0,0],[0,0,0,25,0],[0,0,0,0,0],[0,0,0,0,0]]
  cases.append((f'zero-memory-{pan}',rows))
 for name,rows in cases:
  stage=stage_root/f'{linear}-{name}';stage.mkdir();module=stage/'probe.xm'
  data=bytearray(original);data[345:385]=bytes(v for row in rows for v in row)
  if args.mix_mode is not None:data.extend(b'STPM.MMP\x04\x00'+struct.pack('<I',args.mix_mode))
  module.write_bytes(data)
  capture=c.capture_case(native.capture,ROOT,renderer,stage,module,1.2,stage/'native.wav')
  c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','0','--output',stage/'reference.wav','--',module])
  endpoints={}
  for player in ('native','reference'):
   with wave.open(str(stage/(player+'.wav'))) as w:pcm=array.array('h',w.readframes(w.getnframes()))
   endpoints[player]=[[sum(pcm[2*((tick+1)*882-8)+ch:2*((tick+1)*882-2)+ch:2])/6 for ch in (0,1)] for tick in range(48)]
  baseline=max(endpoints['reference'][0]);assert baseline>20
  # Reference pan ramps/quantization differ slightly; trigger reset is checked exactly below.
  tolerance=.018 if args.legacy else .006
  trigger_reset=True
  if name.startswith("delay-") and not args.legacy:
   trigger=6+int(name.split("-")[1])
   trigger_reset=endpoints["native"][trigger]==endpoints["native"][0]
  failures=[tick for tick in range(48) if max(abs(a-b) for a,b in zip(endpoints['native'][tick],endpoints['reference'][tick]))/baseline>tolerance]
  item={'name':name,'linear':linear,'source_sha256':c.digest(module),'capture':capture,'endpoints':endpoints,'tolerance':tolerance,'failed_ticks':failures,'trigger_reset_pass':trigger_reset,'pass':not failures and trigger_reset and not capture['playing']}
  results.append(item)
  (stage_root/'report.json').write_text(json.dumps({'reference_sha256':c.digest(reference),'renderer':identity,'cases':results},indent=2))
  print(linear,name,item['pass'],failures[:8],flush=True)
assert len(results)==40 and all(x['pass'] for x in results),'Review edge mismatch; retained evidence'
