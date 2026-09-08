"""Original E5x override controls; reuse the established packed-playback pipeline."""
from pathlib import Path
import array, json, math, os, struct, sys, tempfile, wave
ROOT = Path(__file__).resolve().parents[2]
for key in ('TEMP', 'TMP', 'TMPDIR', 'temp', 'tmp', 'tmpdir'):
    os.environ[key] = 'C:/Users/rdave/AppData/Local/Temp'
os.environ['CARGO_NET_OFFLINE'] = 'true'
import compatibility as c
sys.path.insert(0, str(ROOT.parent / 'speccade/packs/tracker_starter_v1'))
import native, build
for env in (c.ENV, build.ENV):
    env.update({key: os.environ[key] for key in ('TEMP', 'TMP', 'TMPDIR', 'temp', 'tmp', 'tmpdir', 'CARGO_NET_OFFLINE')})
near_porta = '--near-porta' in sys.argv
if near_porta: sys.argv.remove('--near-porta')
porta_down = '--porta-down' in sys.argv
if porta_down: sys.argv.remove('--porta-down')
slow_porta = '--slow-porta' in sys.argv
if slow_porta: sys.argv.remove('--slow-porta')
quantization = '--quantization' in sys.argv
if quantization: sys.argv.remove('--quantization')
semitone = '--semitone' in sys.argv
if semitone: sys.argv.remove('--semitone')
extremes = '--extremes' in sys.argv
if extremes: sys.argv.remove('--extremes')
legacy = '--legacy' in sys.argv
if legacy: sys.argv.remove('--legacy')
assert len(sys.argv) in (2, 3) and Path(sys.argv[1]).name == sys.argv[1] and sys.argv[1] not in ('.', '..')
interaction = sys.argv[2] if len(sys.argv) == 3 else 'scope'
assert interaction in ('scope', 'gliss', 'delay', 'retrigger', 'selection', 'retrigger-porta')
stage = ROOT / 'target/tracker-compatibility' / sys.argv[1]
stage.mkdir(exist_ok=False)
class RetainedDirectory:
    def __init__(self, prefix='probe-', **kwargs):
        self.path = tempfile.mkdtemp(prefix=prefix, dir=stage)
    def __enter__(self): return self.path
    def __exit__(self, *args): return False
c.tempfile.TemporaryDirectory = RetainedDirectory
assert (c.CACHE / 'reference-openmpt-0.8.9/openmpt123.exe').is_file(), 'No reference downloads permitted'
reference = c.filter_reference()
c.run(['cargo', 'build', '--offline', '-p', 'nether-cli', '-p', 'nethercore-zx'])
renderer = stage / 'renderer.exe'
identity = c.compile_compat_renderer(native, renderer)
results = []
fines = (-9,-8,-7,-1,0,1,7,8,9,119,120,121,127) if quantization else (-128,-1,127) if extremes else (-64,0,64)
overrides = (None,) if quantization else (None,0,8,15)
for linear in (0, 1):
    # Reuse our original authored waveform/header, never a reference-player asset.
    original = ROOT / 'target/tracker-compatibility/xm-parity-review-interactions-v5' / f'{linear}-plain/probe.xm'
    template = original.read_bytes()
    header, packing, rows, size = struct.unpack_from('<IBHH', template, 336)
    assert (header, packing, rows, size) == (9, 0, 8, 40)
    instrument = 336 + header + size
    instrument_size = struct.unpack_from('<I', template, instrument)[0]
    sample_header = instrument + instrument_size
    for source_fine in fines:
        for override in overrides:
            name = f'{linear}-fine{source_fine}-e5{override}'
            case = stage / name
            case.mkdir()
            data = bytearray(template)
            data[336 + header:336 + header + size] = bytes([49, 1, 0, 14 if override is not None else 0, 0x50 + override if override is not None else 0] + [0] * 35)
            # Follow the overridden note with E58 without a note, then fresh notes
            # without/with an instrument, to observe scope and reset semantics.
            for row, cell in ((2, [0, 0, 0, 14, 0x58]), (4, [49, 0, 0, 0, 0]), (6, [49, 1, 0, 0, 0])):
                start = 336 + header + row * 5
                data[start:start + 5] = bytes(cell)
            if interaction != 'scope':
                if interaction == 'gliss':
                    cells = ((1, [0,0,0,14,0x31]), (2, [50,0,0,3,255]), (4, [49,0,0,3,0]), (6, [49,1,0,0,0]))
                elif interaction == 'retrigger-porta':
                    cells = ((2, [(48 if porta_down else 50) if near_porta else (37 if porta_down else 61),0,0xf1 if slow_porta else 0xff,14,0x92]), (4, [0,0,0xf0,0,0]), (6, [49,1,0,0,0]))
                elif interaction == 'selection':
                    cells = ((2, [0,1,0,0,0]), (4, [49,0,0,0,0]), (6, [49,1,0,0,0]))
                elif interaction == 'delay':
                    cells = ((2, [49,0,0,14,0xd3]), (4, [49,1,0,14,0xd3]), (6, [49,1,0,0,0]))
                else:
                    cells = ((2, [0,0,0,14,0x92]), (4, [49,0,0,27,0x93]), (6, [49,1,0,0,0]))
                for row, cell in cells:
                    start = 336 + header + row * 5
                    data[start:start + 5] = bytes(cell)
            data[sample_header + 13] = source_fine & 255
            # Exercise baked octave transposition independently of E5x finetune.
            transpose = 1 if semitone else 12
            data[sample_header + 16] = (-transpose if source_fine < 0 else transpose if source_fine > 0 else 0) & 255
            if legacy: data[sample_header + 18:sample_header + 40] = bytes(22)
            source = case / 'probe.xm'
            source.write_bytes(data)
            capture = c.capture_case(native.capture, ROOT, renderer, case, source, 1.1, case / 'native.wav')
            c.run([reference, '--batch', '--quiet', '--no-float', '--dither', '0', '--samplerate', '44100', '--channels', '2', '--gain', '0', '--stereo', '100', '--filter', '8', '--ramping', '-1', '--repeat', '0', '--subsong', '0', '--end-time', '1.1', '--output', case / 'reference.wav', '--', source])
            item = {'case': name, 'interaction': interaction, 'linear': linear, 'source_finetune': source_fine, 'override': override, 'source_sha256': c.digest(source), 'capture': capture}
            for player in ('native', 'reference'):
                with wave.open(str(case / (player + '.wav'))) as wav:
                    pcm = array.array('h', wav.readframes(wav.getnframes()))
                if sys.byteorder != 'little': pcm.byteswap()
                pitches = []
                rows = (0,2,4,6) if interaction == 'scope' else (1,2,3,4,5,7) if interaction == 'gliss' else (1,3,5,7)
                windows = [(row*.12+.04, row*.12+.10) for row in rows]
                if interaction == 'retrigger-porta':
                    # Never estimate one carrier across sample-restart discontinuities.
                    windows = [(.12+.04,.12+.10)] + [(.24+tick*.02+.005,.24+tick*.02+.019) for tick in range(6)] + [(row*.12+.04,row*.12+.10) for row in (3,4,5,7)]
                for row, (start, end) in enumerate(windows):
                    block = pcm[round(start*44100)*2:round(end*44100)*2:2]
                    crossings = [i - block[i] / (block[i + 1] - block[i]) for i in range(len(block) - 1) if block[i] <= 0 < block[i + 1]]
                    assert len(crossings) > 2, (name, player, row, 'no sustained pitch')
                    pitches.append((len(crossings) - 1) * 44100 / (crossings[-1] - crossings[0]))
                item[player + '_hz'] = pitches
            item['relative_pitch_error'] = max(abs(a / b - 1) for a, b in zip(item['native_hz'], item['reference_hz']))
            item['pass'] = item['relative_pitch_error'] < .006 and not capture['playing']
            results.append(item)
            (stage / 'report.json').write_text(json.dumps({'status': 'bounded E5x comparison, not comprehensive parity', 'renderer': identity, 'reference_sha256': c.digest(reference), 'cases': results}, indent=2))
            print(name, item['pass'], item['native_hz'], item['reference_hz'], flush=True)
assert len(results) == 2 * len(fines) * len(overrides) and all(item['pass'] for item in results), 'E5x parity failure; see retained report.json'
print(f'PASS: {len(results)} source-finetune/E5x controls in both period modes')
