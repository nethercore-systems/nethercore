"""Original XM effect-gap fixtures using the existing native/reference pipeline."""
from pathlib import Path
import sys,os,json,struct,math,wave,array,tempfile
ROOT=Path(__file__).resolve().parents[2]
for key in ('TEMP','TMP','TMPDIR','temp','tmp','tmpdir'):
    os.environ[key]='C:/Users/rdave/AppData/Local/Temp'
os.environ['CARGO_NET_OFFLINE']='true'
import compatibility as c
sys.path.insert(0,str(ROOT.parent/'speccade/packs/tracker_starter_v1'))
import native,build
for env in (c.ENV,build.ENV):
    env['CARGO_NET_OFFLINE']='true'
    for key in ('TEMP','TMP','TMPDIR','temp','tmp','tmpdir'):env[key]=os.environ[key]
assert len(sys.argv)==2, 'usage: python tools/tracker-debug/xm_effect_probe.py NEW_STAGE_NAME'
assert Path(sys.argv[1]).name==sys.argv[1] and sys.argv[1] not in ('.','..')
stage_root=ROOT/'target/tracker-compatibility'/sys.argv[1]
stage_root.mkdir(exist_ok=False)
class RetainedDirectory:
    def __init__(self,prefix='probe-',**kwargs):self.path=tempfile.mkdtemp(prefix=prefix,dir=stage_root)
    def __enter__(self):return self.path
    def __exit__(self,*args):return False
c.tempfile.TemporaryDirectory=RetainedDirectory
assert (c.CACHE/'reference-openmpt-0.8.9/openmpt123.exe').is_file(), 'installed reference required; downloads prohibited'
reference=c.filter_reference()
original_capture=c.capture_case
c.run(['cargo','build','--offline','-p','nether-cli','-p','nethercore-zx'])
renderer=stage_root/'renderer.exe';identity=c.compile_compat_renderer(native,renderer)
results=[]
gliss_controls={
 'gliss-fine':(49,61,5,64),
 'gliss-fine-pos-fast':(49,61,3,64),
 'gliss-fine-neg-up':(49,61,5,-64),
 'gliss-fine-pos-down':(61,49,3,64),
 'gliss-fine-neg-down':(61,49,5,-64),
 'gliss-octave-down-slow':(61,49,1,64),
 'gliss-holdout-d-up':(51,63,4,32),
 'gliss-holdout-a-down':(58,46,2,96),
 'gliss-holdout-c-down':(61,49,2,80),
 'gliss-holdout-e-up':(53,65,3,-32),
}
# Paired continuous slides isolate accumulator errors from output snapping.
gliss_controls.update({name.replace('gliss-', 'porta-'): value for name,value in list(gliss_controls.items())})
combined_controls = [kind+'-volume-'+case for kind in ('porta','vibrato') for case in ('down','memory','up','shared','both','column','column-down','column-zero')]
delay_controls=[f'delay-{d}-{direction}-{instrument}' for d in (1,3,5,6) for direction in ('up','down') for instrument in (0,1)]
combined_controls += delay_controls
tremolo_controls=['tremolo','tremolo-memory','tremolo-gap','tremolo-fast','tremolo-deep','tremolo-square','tremolo-ramp','tremolo-reset','tremolo-noreset','tremolo-ramp-vibrato','tremolo-zero','tremolo-random']
vibrato_controls=['vibrato-square','vibrato-ramp','vibrato-reset','vibrato-noreset','vibrato-random']
vibrato_controls += ['vibrato-random-holdout-up','vibrato-random-holdout-down']
vibrato_controls += [f'vibrato-random-{speed}' for speed in (1,3,5,7,9,11,13,15)]
auto_controls={'auto-sine':(0,0,4,3),'auto-sweep':(0,16,4,3),'auto-square':(1,0,4,3),'auto-up':(2,0,4,3),'auto-down':(3,0,4,3)}
auto_controls.update({f'auto-holdout-{wave}':(wave,7,11,13) for wave in range(4)})
auto_controls.update({f'auto-{kind}':(0,24,11,13) for kind in ('release','retrigger','selection','delay','pattern-delay','effect')})
auto_controls['auto-pattern-delay']=(1,0,15,32)
auto_controls['auto-delay-vibrato']=(1,0,15,32)
auto_controls['auto-delay-vibrato-zero']=(1,0,0,32)
auto_controls['auto-release-full']=(0,4,11,13)
for linear in [1,0]:
 for effect in ['plain','gliss','tremor','tremor-00','tremor-01','tremor-10','tremor-21-gap','up','down','fine-up','fine-down','extra-up','extra-down','vibrato','arpeggio','up-vibrato','fine-vibrato','arpeggio-gap','vibrato-gap','gliss-fine','gliss-fine-pos-fast','gliss-fine-neg-up','gliss-fine-pos-down','gliss-fine-neg-down','gliss-octave-down-slow','gliss-holdout-d-up','gliss-holdout-a-down','gliss-holdout-c-down','gliss-holdout-e-up','porta-vibrato','fine-volume-vibrato'] + [name for name in gliss_controls if name.startswith('porta-')] + combined_controls + tremolo_controls + vibrato_controls + list(auto_controls):
  stage=stage_root/f'{linear}-{effect}';stage.mkdir();module=stage/'probe.xm'
  data=bytearray(336);data[:17]=b'Extended Module: ';data[17:37]=b'Effect gap probe'.ljust(20,b' ');data[37]=26;data[38:58]=b'FastTracker v2.00'.ljust(20,b' ')
  struct.pack_into('<HI8H',data,58,0x104,276,1,0,1,1,1,linear,6,125)
  start_note=gliss_controls.get(effect,(49,0,0,0))[0]
  rows=[[start_note,1,0,14,0x31 if effect.startswith('gliss') else 0x30]]
  if effect.startswith('tremor'):
   parameter=int(effect.split('-')[1],16) if '-' in effect else 0x21
   rows=[[49,1,0,0x1d,parameter]]+[[0,0,0,0x1d,0] for _ in range(7)]
   if effect.endswith('gap'):rows[2]=[0,0,0,0,0];rows[4]=[49,1,0,0x1d,0]
  elif effect in ('plain','gliss') or effect in gliss_controls:
   _,target,speed,_=gliss_controls.get(effect,(49,61,5,0))
   rows += [[target if i==0 else 0,0,0,3,speed if i==0 else 0] for i in range(7)]
  elif effect in tremolo_controls:
   if effect.endswith('random'):rows[0][4]=0x73
   if effect.endswith(('square','ramp')):rows[0][4]=0x72 if effect.endswith('square') else 0x71
   tremolo_parameter=0x53 if effect.endswith('fast') else 0x27 if effect.endswith('deep') else 0x34
   rows += [[0,0,0,7,tremolo_parameter]]
   rows += [[0,0,0,0 if effect.endswith('gap') and i>=2 else 7,0 if effect.endswith('memory') else tremolo_parameter if not (effect.endswith('gap') and i>=2) else 0] for i in range(6)]
   if effect.endswith('zero'):rows[1]=[0,0,0,12,0]
   if effect.endswith(('reset','noreset')):
    rows[0][4]=0x74 if effect.endswith('noreset') else 0x70
    rows[4]=[49,1,0,7,tremolo_parameter]
   if effect.endswith('ramp-vibrato'):
    rows[0][4]=0x71
    rows[1:4]=[[0,0,0,4,0x34] for _ in range(3)]
  elif effect in auto_controls:
   rows += [[0,0,0,0,0] for _ in range(7)]
   if effect in ('auto-release','auto-release-full'):rows[2]=[97,0,0,0,0]
   if effect=='auto-retrigger':rows[2]=[49,1,0,0,0]
   if effect=='auto-selection':rows[2]=[0,1,0,0,0]
   if effect=='auto-delay':rows[2]=[49,1,0,14,0xd3]
   if effect=='auto-pattern-delay':rows[2]=[0,0,0,14,0xe1]
   if effect.startswith('auto-delay-vibrato'):rows[1]=[0,0,0,4,0x3f];rows[2]=[0,0,0xbf,14,0xe1]
   if effect=='auto-effect':rows[1:]=[[0,0,0,4,0x34] for _ in range(7)]
  elif effect in vibrato_controls:
   rows[0][4]=0x43 if 'random' in effect else 0x42 if effect.endswith('square') else 0x41 if effect.endswith('ramp') else 0x44 if effect.endswith('noreset') else 0x40
   if 'holdout' in effect:rows[0][0]=53 if effect.endswith('up') else 61
   parameter=(int(effect.rsplit('-',1)[1])<<4)|15 if effect.rsplit('-',1)[-1].isdigit() else 0x34
   if 'holdout' in effect:parameter=0x27 if effect.endswith('up') else 0x85
   rows += [[0,0,0,4,parameter] for _ in range(7)]
   if effect.endswith(('reset','noreset')):rows[4]=[49,1,0,4,0x34]
  elif effect in delay_controls:
   _,delay,direction,instrument=effect.split('-')
   rows += [[49,int(instrument),0x72 if direction=='up' else 0x62,14,0xd0+int(delay)]]
   rows += [[0,0,0,0,0] for _ in range(6)]
  elif effect in combined_controls:
   porta=effect.startswith('porta-'); command=5 if porta else 6
   rows += [[61 if porta else 0,0,0,3 if porta else 4,5 if porta else 0x34]]
   parameter=0x10 if effect.endswith(('up','column-down')) else 0x12 if effect.endswith('both') else 1
   rows += [[0,0,0,0x0a if effect.endswith('shared') else command,parameter]]
   rows += [[0,0,0x71 if effect.endswith('column') else 0x61 if effect.endswith('column-down') else 0x70 if effect.endswith('column-zero') else 0,command,0 if effect.endswith(('memory','shared')) else parameter] for _ in range(5)]
  elif effect=='porta-vibrato':
   rows += [[61,0,0,3,5]]+[[0,0,0,4,0x34] for _ in range(2)]+[[0,0,0,0,0] for _ in range(4)]
  elif effect=='fine-volume-vibrato':
   rows += [[0,0,0xa3,0,0],[0,0,0xb4,14,0x13]]+[[0,0,0xb4,0,0] for _ in range(2)]+[[0,0,0,0,0] for _ in range(3)]
  elif effect in ('up-vibrato','fine-vibrato'):
   rows += [[0,0,0,1,5] if effect=='up-vibrato' else [0,0,0,14,0x13]]
   rows += [[0,0,0,4,0x34] for _ in range(2)]+[[0,0,0,0,0] for _ in range(4)]
  elif effect in ('arpeggio-gap','vibrato-gap'):
   command,parameter=(0,0x37) if effect=='arpeggio-gap' else (4,0x34)
   rows += [[0,0,0,command,parameter] for _ in range(2)]+[[0,0,0,0,0] for _ in range(5)]
  else:
   command,parameter={'up':(1,5),'down':(2,5),'fine-up':(14,0x13),'fine-down':(14,0x23),
     'extra-up':(0x21,0x13),'extra-down':(0x21,0x23),'vibrato':(4,0x34),'arpeggio':(0,0x37)}[effect]
   rows += [[0,0,0,command,parameter] for _ in range(7)]
  notes=bytes(v for row in rows for v in row);data.extend(struct.pack('<IBHH',9,0,len(rows),len(notes))+notes)
  ins=bytearray(263);struct.pack_into('<I',ins,0,263);struct.pack_into('<HI',ins,27,1,40)
  if effect in auto_controls:
   ins[235:239]=bytes(auto_controls[effect])
   if effect in ('auto-release','auto-release-full'):
    struct.pack_into('<4H',ins,129,0,64,100,64);ins[225]=2;ins[233]=1
  data.extend(ins)
  sample=bytearray(40);struct.pack_into('<III',sample,0,64,0,64);sample[12]=32;sample[13]=gliss_controls.get(effect,(0,0,0,0))[3]&255;sample[14]=1;sample[15]=128;sample[18:40]=b' '*22
  if effect in tremolo_controls + delay_controls:
   struct.pack_into('<III',sample,0,128,0,128);sample[14]=17
  data.extend(sample)
  previous=0
  for i in range(64):
   value=round((12000 if effect in tremolo_controls + delay_controls else 48)*math.sin(2*math.pi*i/(32 if effect in combined_controls + tremolo_controls else 16)))
   if effect in tremolo_controls + delay_controls:data.extend(struct.pack('<H',(value-previous)&65535))
   else:data.append((value-previous)&255)
   previous=value
  module.write_bytes(data)
  capture=original_capture(native.capture,ROOT,renderer,stage,module,1.1,stage/'native.wav')
  c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','-1','--end-time','1.1','--output',stage/'reference.wav','--',module])
  item={'linear':linear,'effect':effect,'capture':capture,'source_sha256':c.digest(module)}
  for player in ['native','reference']:
   with wave.open(str(stage/(player+'.wav'))) as w:pcm=array.array('h',w.readframes(w.getnframes()))[::2]
   ticks=[]
   for tick in range(48):
    block=pcm[round((tick*.02+.006)*44100):round((tick*.02+.018)*44100)]
    crossings=[i-block[i]/(block[i+1]-block[i]) for i in range(len(block)-1) if block[i]<=0<block[i+1]]
    # Whole cycles avoid phase-dependent RMS bias in short pitch windows.
    energy=block[math.ceil(crossings[0]):math.ceil(crossings[-1])] if len(crossings)>2 else block
    ticks.append({'rms':math.sqrt(sum(v*v for v in energy)/len(energy)), 'hz':(len(crossings)-1)*44100/(crossings[-1]-crossings[0]) if len(crossings)>2 else 0})
   if effect in tremolo_controls + delay_controls and player == 'reference':
    # FT2 ramps modulation over the tick even at reference ramping0.
    # Fit the known sinusoid in the settled tail, not its transition average.
    for tick,measurement in enumerate(ticks):
     if effect.endswith('ramp-vibrato') and 6<=tick<24:continue
     hz=ticks[0]['hz']; assert hz>0, 'unmeasurable stationary carrier'
     first=round((tick*.02+.012)*44100);last=round((tick*.02+.0198)*44100)
     block=pcm[first:last]
     basis=[]
     for i in range(len(block)):
      phase=2*math.pi*hz*i/44100;time=(first+i)/44100-(tick+1)*.02
      sn=math.sin(phase);cs=math.cos(phase)
      basis.append((sn,cs,sn*time,cs*time))
     matrix=[[sum(v[a]*v[b] for v in basis) for b in range(4)]+[sum(y*v[a] for y,v in zip(block,basis))] for a in range(4)]
     for col in range(4):
      pivot=max(range(col,4),key=lambda row:abs(matrix[row][col]));matrix[col],matrix[pivot]=matrix[pivot],matrix[col]
      scale=matrix[col][col];assert abs(scale)>1e-10
      matrix[col]=[v/scale for v in matrix[col]]
      for row in range(4):
       if row!=col:
        scale=matrix[row][col];matrix[row]=[a-scale*b for a,b in zip(matrix[row],matrix[col])]
     coefficients=[row[-1] for row in matrix]
     residual=math.sqrt(sum((y-sum(a*b for a,b in zip(v,coefficients)))**2 for y,v in zip(block,basis))/len(block))
     measurement['rms']=math.hypot(*coefficients[:2])/math.sqrt(2)
     measurement['fit_residual']=residual
     assert residual < ticks[0]['rms']*.015, 'carrier/ramp fit is not explanatory'
   item[player]=ticks
  item['gate_mismatches']=[i for i,(a,b) in enumerate(zip(item['native'],item['reference'])) if (a['rms']>100)!=(b['rms']>100)]
  item['maximum_pitch_error']=max(abs(a['hz']/b['hz']-1) for a,b in zip(item['native'],item['reference']) if a['hz'] and b['hz'])
  item['baseline_gain_ratio']=item['native'][0]['rms']/item['reference'][0]['rms']
  item['maximum_volume_error']=max(abs(a['rms']/item['native'][0]['rms']-b['rms']/item['reference'][0]['rms']) for a,b in zip(item['native'],item['reference']))
  item['pass']= (effect not in combined_controls + tremolo_controls or (abs(item['baseline_gain_ratio']-1)<.01 and item['maximum_volume_error']<.025)) and not item['gate_mismatches'] and item['maximum_pitch_error']<.006 and not capture['playing']
  results.append(item)
  print(linear,effect,'native hz',[round(t['hz'],1) for t in item['native'][6:18]],'ref hz',[round(t['hz'],1) for t in item['reference'][6:18]],flush=True)
  if effect=='tremor':print('gates native',[int(t['rms']>100) for t in item['native'][:24]],'ref',[int(t['rms']>100) for t in item['reference'][:24]],flush=True)
  (stage_root/'report.json').write_text(json.dumps({'status':'bounded effect comparison; not comprehensive parity','renderer':identity,'reference_sha256':c.digest(reference),'cases':results},indent=2))

assert len(results)==82+2*len(combined_controls + tremolo_controls + vibrato_controls + list(auto_controls)) and all(case['pass'] for case in results), 'XM effect parity failure; see retained report.json'
print(f'PASS: {len(results)} XM effect/period-mode cases, 48 tick gates/pitches each, finite end')
