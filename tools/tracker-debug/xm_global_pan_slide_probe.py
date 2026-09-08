"""Original Hxy/Pxy controls; reuse the established offset probe's pack/reference bootstrap."""
from pathlib import Path
import sys
cross_channel="--cross-channel" in sys.argv
if cross_channel:sys.argv.remove("--cross-channel")
bootstrap=Path(__file__).with_name('xm_offset_probe.py').read_text().split('results=[]')[0]
exec(compile(bootstrap,'existing-offset-bootstrap','exec'))
results=[]
for linear in (0,1):
 for effect in (17,25):
  for parameter in (1,16,17,15,240,255):
   for column in (0,0x61,0x71,0xd1,0xe1):
    stage=stage_root/f'{linear}-{effect}-{parameter}-{column}';stage.mkdir();module=stage/'probe.xm'
    data=bytearray(336);data[:17]=b'Extended Module: ';data[17:37]=b'Original HP probe'.ljust(20,b' ');data[37]=26;data[38:58]=b'FastTracker v2.00'.ljust(20,b' ')
    struct.pack_into('<HI8H',data,58,0x104,276,1,0,2 if cross_channel else 1,1,1,linear,6,125)
    rows=[[49,1,0x30,16,32],[0,0,column,effect,parameter],[0,0,column,effect,0],[0,0,0,0,0],[0,0,column,effect,0],[49,1,column,effect,0],[0,0,0,effect,1 if parameter>=16 else 16],[0,0,0,effect,0]]
    if cross_channel:
     rows=[[row,[0,0,0,0,0]] for row in rows]
     for index in (1,4):
      rows[index][1][3:]=rows[index][0][3:];rows[index][0][3:]=[0,0]
     cells=bytes(v for row in rows for cell in row for v in cell)
    else:cells=bytes(v for row in rows for v in row)
    data.extend(struct.pack('<IBHH',9,0,8,len(cells))+cells)
    instrument=bytearray(263);struct.pack_into('<I',instrument,0,263);struct.pack_into('<HI',instrument,27,1,40);data.extend(instrument)
    sample=bytearray(40);struct.pack_into('<III',sample,0,32,0,32);sample[12]=32;sample[14]=1;sample[15]=128;sample[18:40]=bytes(22) if args.legacy else b' '*22;data.extend(sample)
    previous=0
    for i in range(32):
     value=80;data.append((value-previous)&255);previous=value
    if args.mix_mode is not None:data.extend(b'STPM.MMP\x04\x00'+struct.pack('<I',args.mix_mode))
    module.write_bytes(data)
    capture=c.capture_case(native.capture,ROOT,renderer,stage,module,1.2,stage/'native.wav')
    c.run([reference,'--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','0','--output',stage/'reference.wav','--',module])
    energies={}
    for player in ('native','reference'):
     with wave.open(str(stage/(player+'.wav'))) as w:pcm=array.array('h',w.readframes(w.getnframes()))
     energies[player]=[]
     for tick in range(48):
      # Constant authored PCM removes phase-window bias; late tick excludes reference volume ramp.
      end=round((tick+1)*.02*44100)
      a,b=end-8,end-2
      energies[player].append([math.sqrt(sum(v*v for v in pcm[2*a+ch:2*b:2])/(b-a)) for ch in (0,1)])
    baseline=max(energies['reference'][0]);assert baseline>20
    failures=[tick for tick in range(48) if max(abs(a-b) for a,b in zip(energies['native'][tick],energies['reference'][tick]))/baseline>.018]
    item={'linear':linear,'effect':effect,'parameter':parameter,'column':column,'source_sha256':c.digest(module),'capture':capture,'energies':energies,'failed_ticks':failures,'pass':not failures and not capture['playing']}
    results.append(item)
    (stage_root/'report.json').write_text(json.dumps({'reference_sha256':c.digest(reference),'renderer':identity,'cases':results},indent=2))
    print(linear,effect,parameter,column,item['pass'],failures,flush=True)
assert len(results)==120 and all(x['pass'] for x in results),'H/P mismatch; retained evidence'
