"""Original finite XM cross-channel flow controls, existing pack/native/reference path."""
from pathlib import Path
import array, json, math, os, statistics, struct, sys, tempfile, wave
ROOT=Path(__file__).resolve().parents[2]
for key in ('TEMP','TMP','TMPDIR','temp','tmp','tmpdir'):
    os.environ[key]='C:/Users/rdave/AppData/Local/Temp'
os.environ['CARGO_NET_OFFLINE']='true'
import compatibility as c
sys.path.insert(0,str(ROOT.parent/'speccade/packs/tracker_starter_v1'))
import native,build
for env in (c.ENV,build.ENV):
    env.update({k:os.environ[k] for k in ('TEMP','TMP','TMPDIR','temp','tmp','tmpdir','CARGO_NET_OFFLINE')})
legacy = len(sys.argv)==3 and sys.argv[2]=="--legacy"
assert (len(sys.argv)==2 or legacy) and Path(sys.argv[1]).name==sys.argv[1] and sys.argv[1] not in ('.','..')
stage_root=ROOT/'target/tracker-compatibility'/sys.argv[1];stage_root.mkdir(exist_ok=False)
class RetainedDirectory:
    def __init__(self,prefix='probe-',**kwargs):self.path=tempfile.mkdtemp(prefix=prefix,dir=stage_root)
    def __enter__(self):return self.path
    def __exit__(self,*args):return False
c.tempfile.TemporaryDirectory=RetainedDirectory
assert (c.CACHE/'reference-openmpt-0.8.9/openmpt123.exe').is_file(), 'No downloads'
reference=c.filter_reference()
c.run(['cargo','build','--offline','-p','nether-cli','-p','nethercore-zx'])
source=native.HELPER.read_text().replace('    for _ in 0..frames {','    let mut last = (u16::MAX,u16::MAX);\n    for frame in 0..frames {')
source=source.replace('        assert_eq!(buffer.len(), 1470);','''        assert_eq!(buffer.len(), 1470);
        if (state.order_position,state.row) != last {
            last=(state.order_position,state.row);
            println!("FLOW {} {} {} {}",frame+1,state.order_position,state.row,state.flags & tracker_flags::PLAYING != 0);
        }''')
helper=stage_root/'flow-renderer.rs';helper.write_text(source);native.HELPER=helper
renderer=stage_root/'renderer.exe';identity=c.compile_compat_renderer(native,renderer)
original_run=native.run
trace=[]
def retain(args):
    text=original_run(args)
    if str(args[0])==str(renderer):trace[:]=text.splitlines()
    return text
native.run=retain
# (row, channel, effect, parameter) in pattern zero; every row has a unique audible note.
cases={
 'plain':[],
 'marker-only':[(1,0,14,0x60)],
 'marker-break':[(1,0,14,0x60),(5,0,13,3)],
 'marker-jump':[(1,0,14,0x60),(5,0,11,2)],
 'loop-last-row':[(6,0,14,0x60),(7,0,14,0x61)],
 'loop-delay-two':[(1,0,14,0x60),(3,0,14,0x62),(3,1,14,0xe2)],
 'jump-left':[(2,0,11,2)],'jump-right':[(2,1,11,2)],
 'break-left':[(2,0,13,3)],'break-right':[(2,1,13,3)],
 'jump-break':[(2,0,11,2),(2,1,13,3)],
 'break-jump':[(2,0,13,3),(2,1,11,2)],
 'loop-left':[(1,0,14,0x60),(3,0,14,0x62)],
 'loop-right':[(1,1,14,0x60),(3,1,14,0x62)],
 'loop-competing':[(0,0,14,0x60),(1,1,14,0x60),(3,0,14,0x62),(3,1,14,0x61)],
 'delay-left':[(2,0,14,0xe1)],'delay-right':[(2,1,14,0xe1)],
 'delay-competing':[(2,0,14,0xe1),(2,1,14,0xe2)],
 'jump-delay':[(2,0,11,2),(2,1,14,0xe1)],
 'break-delay':[(2,0,13,3),(2,1,14,0xe1)],
 'loop-delay':[(1,0,14,0x60),(3,0,14,0x62),(3,1,14,0xe1)],
 'speed-competing':[(2,0,15,4),(2,1,15,5)],
 'tempo-competing':[(2,0,15,150),(2,1,15,180)],
}
results=[]
for linear in (0,1):
 for name,commands in cases.items():
    stage=stage_root/f'{linear}-{name}';stage.mkdir();module=stage/'probe.xm'
    data=bytearray(336);data[:17]=b'Extended Module: ';data[17:37]=b'Original flow probe'.ljust(20,b' ');data[37]=26;data[38:58]=b'FastTracker v2.00'.ljust(20,b' ')
    struct.pack_into('<HI8H',data,58,0x104,276,3,0,2,3,1,linear,3,125);data[80:83]=bytes([0,1,2])
    for pattern in range(3):
        rows=[[[61+pattern*8+row,1,0,0,0],[0,0,0,0,0]] for row in range(8)]
        if pattern==0:
            for row,ch,e,p in commands:rows[row][ch][3:]=[e,p]
        cells=bytes(v for row in rows for cell in row for v in cell)
        data.extend(struct.pack('<IBHH',9,0,8,len(cells))+cells)
    instrument=bytearray(263);struct.pack_into('<I',instrument,0,263);struct.pack_into('<HI',instrument,27,1,40);data.extend(instrument)
    sample=bytearray(40);sample[18:40]=bytes(22) if legacy else b' '*22;struct.pack_into('<III',sample,0,32,0,32);sample[12]=32;sample[14]=1;sample[15]=128;data.extend(sample)
    previous=0
    for i in range(32):
        value=round(64*math.sin(2*math.pi*i/32));data.append((value-previous)&255);previous=value
    module.write_bytes(data)
    capture=c.capture_case(native.capture,ROOT,renderer,stage,module,6,stage/'native.wav')
    (stage/'native-state.txt').write_text('\n'.join(trace))
    c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','0','--output',stage/'reference.wav','--',module])
    pcm={}
    for player in ('native','reference'):
        with wave.open(str(stage/(player+'.wav'))) as w:pcm[player]=array.array('h',w.readframes(w.getnframes()))[::2]
    windows=[];end=None
    for line in trace:
        if not line.startswith('FLOW '):continue
        _,frame,order,row,playing=line.split();time=int(frame)/60
        if playing=='false':end=time;continue
        start=round((time+.005)*44100);stop=round((time+.020)*44100)
        frequencies={}
        for player in pcm:
            block=pcm[player][start:stop]
            edges=[i for i in range(1,len(block)) if block[i-1]<=0<block[i]]
            frequencies[player]=44100*(len(edges)-1)/(edges[-1]-edges[0]) if len(edges)>1 else 0
        error=abs(frequencies['native']/frequencies['reference']-1) if frequencies['reference'] else 1
        windows.append({'time':time,'order':int(order),'row':int(row),'frequency':frequencies,'pass':error<.025})
    # End assertion tolerates the reference player's terminal fade, not missing rows.
    reference_end=max((i/44100 for i,v in enumerate(pcm['reference']) if abs(v)>8),default=0)
    item={'name':name,'linear':linear,'source_sha256':c.digest(module),'capture':capture,'rows':windows,'native_end':end,'reference_end':reference_end}
    item['pass']=bool(windows) and all(w['pass'] for w in windows) and end is not None and abs(end-reference_end)<.15 and not capture['playing'] and capture['wraps']==0
    results.append(item)
    (stage_root/'report.json').write_text(json.dumps({'reference_sha256':c.digest(reference),'renderer':identity,'cases':results},indent=2))
    print(linear,name,item['pass'],len(windows),end,reference_end,flush=True)
assert len(results)==len(cases)*2 and all(x['pass'] for x in results), 'Flow mismatch; evidence retained'
