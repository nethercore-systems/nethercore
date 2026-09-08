"""Original XM sample-offset timing controls through the existing packed pipeline."""
from pathlib import Path
import argparse, array, json, math, os, struct, sys, tempfile, wave
ROOT=Path(__file__).resolve().parents[2]
for key in ('TEMP','TMP','TMPDIR','temp','tmp','tmpdir'):
    os.environ[key]='C:/Users/rdave/AppData/Local/Temp'
os.environ['CARGO_NET_OFFLINE']='true'
import compatibility as c
sys.path.insert(0,str(ROOT.parent/'speccade/packs/tracker_starter_v1'))
import native, build
for env in (c.ENV,build.ENV):
    env.update({key:os.environ[key] for key in ('TEMP','TMP','TMPDIR','temp','tmp','tmpdir','CARGO_NET_OFFLINE')})
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument('stage')
parser.add_argument('volume',nargs='?',type=int,default=32)
parser.add_argument('--legacy',action='store_true',help='Use independently measured NUL-padded sample form')
parser.add_argument('--mix-mode',type=int,choices=(4,5))
args=parser.parse_args()
assert Path(args.stage).name==args.stage and args.stage not in ('.','..')
volume=args.volume
assert 1 <= volume <= 64
stage_root=ROOT/'target/tracker-compatibility'/args.stage
stage_root.mkdir(exist_ok=False)
class RetainedDirectory:
    def __init__(self,prefix='probe-',**kwargs):self.path=tempfile.mkdtemp(prefix=prefix,dir=stage_root)
    def __enter__(self):return self.path
    def __exit__(self,*args):return False
c.tempfile.TemporaryDirectory=RetainedDirectory
assert (c.CACHE/'reference-openmpt-0.8.9/openmpt123.exe').is_file(), 'No downloads'
reference=c.filter_reference()
c.run(['cargo','build','--offline','-p','nether-cli','-p','nethercore-zx'])
renderer=stage_root/'renderer.exe';identity=c.compile_compat_renderer(native,renderer)
results=[]
for bits,fine,relative,mapped in [(b,f,r,m) for m in (0,1,2) for b,f,r in [(8,0,0),(16,0,0),(8,64,0),(8,-64,0),(8,0,12)]]:
 for name,note,effect,param in [('plain',0,0,0),('no-note-offset',0,9,1),('note-offset',49,9,1),('note-restart',49,0,0),('empty-row-memory',0,9,1),('note-memory',49,9,1),('past-end',49,9,128),('porta-offset',61,9,1),('porta-zero',61,9,1),('porta-memory',61,9,1),('e93-after-offset',49,9,1),('r03-after-offset',49,9,1),('r93-after-offset',49,9,1),('r03-speed5-after-offset',49,9,1),('r03-gap5-after-offset',49,9,1),('e93-newnote-after-offset',49,9,1),('r03-newnote-after-offset',49,9,1),('r93-newnote-after-offset',49,9,1),('e93-envelope-after-offset',49,9,1),('r03-envelope-after-offset',49,9,1),('r93-envelope-after-offset',49,9,1),('plain-envelope',49,0,0),('r03-newnote-envelope-after-offset',49,9,1),('e93-panenv-after-offset',49,9,1),('r03-panenv-after-offset',49,9,1),('r93-panenv-after-offset',49,9,1),('r03-panenv64-after-offset',49,9,1),('r03-pan64-after-offset',49,9,1),('r03-panenv192-after-offset',49,9,1),('r03-pan192-after-offset',49,9,1)]+[(f'r{v:x}3-transform-after-offset',49,9,1) for v in range(16) if v not in (0,9)]:
    stage=stage_root/f'{bits}-{fine}-{relative}-{mapped}-{name}';stage.mkdir();module=stage/'probe.xm'
    data=bytearray(336);data[:17]=b'Extended Module: ';data[17:37]=b'Original offset test';data[37]=26;data[38:58]=b'FastTracker v2.00'.ljust(20,b' ')
    speed=5 if '5-after-offset' in name else 6
    struct.pack_into('<HI8H',data,58,0x104,276,1,0,1,1,1,1,speed,125)
    rows=[[49,1,0,0,0],[note,0,0,effect,param]]+[[0,0,0,0,0] for _ in range(6)]
    if mapped == 2: rows[0][0]=37
    if name.endswith('memory'): rows[2]=[49,0,0,9,0]
    if name.endswith('-after-offset'):
        rows[2]=[0,0,0,14 if name.startswith('e') else 27,0x93 if name.startswith(('e','r9')) else 3]
        rows[3]=[0,0,0,14 if name.startswith('e') else 27,0x90 if name.startswith('e') else 0]
    if 'transform' in name: rows[2][4]=(int(name[1],16)<<4)|3
    if 'newnote' in name: rows[2][0]=49; rows[2][1]=1
    if 'gap5' in name: rows[3]=[0,0,0,0,0]; rows[4]=[0,0,0,27,3]; rows[5]=[0,0,0,27,0]
    if name.startswith('porta-'): rows[1][2]=0xf0 if name=='porta-zero' else 0xf1
    cells=bytes(v for row in rows for v in row);data.extend(struct.pack('<IBHH',9,0,8,len(cells))+cells)
    ins=bytearray(263);struct.pack_into('<I',ins,0,263);struct.pack_into('<HI',ins,27,2 if mapped else 1,40)
    if mapped: ins[33:129]=bytes([1])*96
    if mapped == 2: ins[33:81]=bytes(48)
    if 'envelope' in name:
        for i,(tick,value) in enumerate(((0,64),(20,16),(40,48))): struct.pack_into('<HH',ins,129+4*i,tick,value)
        ins[225]=3; ins[233]=1
    if 'panenv' in name:
        for i,(tick,value) in enumerate(((0,32),(20,8),(40,48))): struct.pack_into('<HH',ins,177+4*i,tick,value)
        ins[226]=3; ins[234]=1
    data.extend(ins)
    # A distinct unselected first slot catches accidental instrument-default use.
    slots=[(0,0,4096),(fine,relative,2048)] if mapped else [(fine,relative,2048)]
    for slot_fine,slot_relative,cutoff in slots:
        sample=bytearray(40);sample[18:40]=bytes(22) if args.legacy else b' '*22;struct.pack_into('<III',sample,0,16384*(bits//8),0,0);sample[12]=volume;sample[13]=slot_fine&255;sample[14]=16 if bits==16 else 0;sample[15]=64 if ('pan64' in name or 'panenv64' in name) else 192 if ('pan192' in name or 'panenv192' in name) else 128;sample[16]=slot_relative&255;data.extend(sample)
    for slot_fine,slot_relative,cutoff in slots:
        previous=0
        for i in range(16384):
            value=round((80 if i<cutoff else 32)*(256 if bits==16 else 1)*math.sin(2*math.pi*i/32))
            data.extend(struct.pack('<H',(value-previous)&65535) if bits==16 else bytes([(value-previous)&255]));previous=value
    # Reuse the existing FT2 pan-law fixture's space-padded sample names.
    # No runtime creator inference is derived from this authoring convention.
    explicit_mix=args.mix_mode
    if explicit_mix is not None: data.extend(b"STPM.MMP\x04\x00"+struct.pack("<I",explicit_mix))
    module.write_bytes(data)
    capture=c.capture_case(native.capture,ROOT,renderer,stage,module,1.1,stage/'native.wav')
    c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','-1','--output',stage/'reference.wav','--',module])
    item={'legacy_sample_form':args.legacy,'explicit_mix':explicit_mix,'volume':volume,'mapped':mapped,'name':name,'bits':bits,'finetune':fine,'relative':relative,'source_sha256':c.digest(module),'capture':capture}
    waveforms={}
    baseline_energy={}
    for player in ('native','reference'):
        with wave.open(str(stage/(player+'.wav'))) as w:pcm=array.array('h',w.readframes(w.getnframes()))[::2]
        waveforms[player]=pcm
        energies=[]
        for tick in range(36):
            block=pcm[round((tick*.02+.006)*44100):round((tick*.02+.018)*44100)]
            energies.append(math.sqrt(sum(x*x for x in block)/len(block)))
        baseline_energy[player]=energies[0]
        item[player]=[x/energies[2] for x in energies]
    item['mismatched_ticks']=[i for i,(a,b) in enumerate(zip(item['native'],item['reference'])) if abs(a-b)>(.01 if 'transform' in name else .12)]
    item['phase_mismatched_ticks']=[]
    if name.endswith('-after-offset'):
        for tick in range(2*speed,(6 if 'gap5' in name else 4)*speed):
            # Phase is undefined for silence; amplitude gate still checks ramp tails.
            if max(item['native'][tick],item['reference'][tick]) < .002: continue
            a,b=[waveforms[p][round((tick*.02+.006)*44100):round((tick*.02+.018)*44100)] for p in ('native','reference')]
            norm=math.sqrt(sum(x*x for x in a)*sum(x*x for x in b))
            if (norm == 0 and (any(a) or any(b))) or (norm > 0 and sum(x*y for x,y in zip(a,b))/norm < .99): item['phase_mismatched_ticks'].append(tick)
    item['baseline_gain_ratio']=baseline_energy['native']/baseline_energy['reference']
    item['pass']=abs(item['baseline_gain_ratio']-1)<.01 and not item['mismatched_ticks'] and not item['phase_mismatched_ticks'] and not capture['playing']
    results.append(item)
    (stage_root/'report.json').write_text(json.dumps({'reference_sha256':c.digest(reference),'renderer':identity,'scope':'normalized sample-envelope timing, not gain parity','cases':results},indent=2))
    print(name,item['pass'],item['mismatched_ticks'],flush=True)
assert len(results)==660 and all(x['pass'] for x in results), 'Offset timing mismatch; retained evidence'
