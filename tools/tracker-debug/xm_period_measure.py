"""Measure original XM tones; no reference source/table inputs.
Usage: python tools/tracker-debug/xm_period_measure.py NEW_STAGE
These are observations, not automatically accepted runtime calibration.
"""
from pathlib import Path
import array, hashlib, json, math, os, struct, sys, wave
ROOT = Path(__file__).resolve().parents[2]
for key in ('TEMP','TMP','TMPDIR','temp','tmp','tmpdir'):
    os.environ[key] = 'C:/Users/rdave/AppData/Local/Temp'
os.environ['CARGO_NET_OFFLINE'] = 'true'
import compatibility as c
assert len(sys.argv) == 2 and Path(sys.argv[1]).name == sys.argv[1] and sys.argv[1] not in ('.','..')
stage = ROOT / 'target/tracker-compatibility' / sys.argv[1]
stage.mkdir(exist_ok=False)
assert (c.CACHE / 'reference-openmpt-0.8.9/openmpt123.exe').is_file(), 'Installed reference required; no downloads'
reference = c.filter_reference()
fines = list(range(-128, 128, 16))
cases = [(note, fine) for note in range(37,49) for fine in fines]
data = bytearray(336)
data[:17] = b'Extended Module: '
data[17:37] = b'Original period scan'.ljust(20,b' ')
data[37] = 26
data[38:58] = b'FastTracker v2.00'.ljust(20,b' ')
struct.pack_into('<HI8H', data, 58, 0x104, 276, 1,0,1,1,len(fines),0,20,125)
notes = bytes(v for note,fine in cases for v in (note, fines.index(fine)+1,0,0,0))
data.extend(struct.pack('<IBHH',9,0,len(cases),len(notes)) + notes)
for fine in fines:
    instrument = bytearray(263)
    struct.pack_into('<I',instrument,0,263)
    struct.pack_into('<HI',instrument,27,1,40)
    data.extend(instrument)
    sample = bytearray(40)
    struct.pack_into('<III',sample,0,64,0,64)
    sample[12]=32; sample[13]=fine&255; sample[14]=1; sample[15]=128
    data.extend(sample)
    previous=0
    for i in range(64):
        value=round(80*math.sin(2*math.pi*i/64))
        data.append((value-previous)&255); previous=value
module=stage/'original-tones.xm'
module.write_bytes(data)
settings=['--batch','--quiet','--no-float','--dither','0','--samplerate','44100','--channels','2','--gain','0','--stereo','100','--filter','8','--ramping','-1','--repeat','0','--subsong','-1']
c.run([reference,*settings,'--output',stage/'reference.wav','--',module])
with wave.open(str(stage/'reference.wav')) as w:
    assert w.getframerate()==44100 and w.getnchannels()==2 and w.getsampwidth()==2
    pcm=array.array('h',w.readframes(w.getnframes()))[::2]
measurements=[]
for index,(note,fine) in enumerate(cases):
    estimates=[]
    for start,end in ((.07,.21),(.23,.37)):
        block=pcm[round((index*.4+start)*44100):round((index*.4+end)*44100)]
        crossings=[i-block[i]/(block[i+1]-block[i]) for i in range(len(block)-1) if block[i]<=0<block[i+1]]
        assert len(crossings)>=5, (note,fine,'insufficient signal')
        estimates.append((len(crossings)-1)*44100/(crossings[-1]-crossings[0]))
    hz=sum(estimates)/len(estimates)
    measurements.append({'note':note,'finetune':fine,'frequency_hz':hz,'window_hz':estimates,'relative_window_spread':abs(estimates[0]/estimates[1]-1)})
assert len(measurements)==192
# Existing engine C-4 reference rate and our authored 64-frame sample cycle.
# Export inferred integer values, retaining the raw measurements and residuals.
calibration=[]
for item in measurements:
    if item['finetune'] >= 0:
        observed=8363*1712/(item['frequency_hz']*64)
        assert abs(observed-round(observed)) < .15, 'Integer inference ambiguous'
        calibration.append(round(observed))
assert len(calibration)==96 and all(a>b for a,b in zip(calibration,calibration[1:]))
(stage/'inferred-periods.json').write_text(json.dumps(calibration))
report={'status':'independent measured observations; not runtime acceptance','source_sha256':c.digest(module),'reference_sha256':c.digest(reference),'settings':settings,'sample_cycle_frames':64,'row_seconds':.4,'cases':measurements}
(stage/'report.json').write_text(json.dumps(report,indent=2))
print(json.dumps({'cases':len(measurements),'max_relative_window_spread':max(x['relative_window_spread'] for x in measurements),'report':str(stage/'report.json')}))
assert max(x['relative_window_spread'] for x in measurements)<.0001, 'Measurement precision unresolved; retain report'
