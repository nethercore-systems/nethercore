"""Finite original IT main-column controls; retained real-cart/reference evidence.

Usage: python -B tools/tracker-debug/it_effect_probe.py NEW_STAGE
All cases use linear slides and instruments, with (old effects, compatible G)
explicitly (false,false) or (true,true). No other flag permutations are claimed.
"""
import array
import bisect
import json
import math
import os
from pathlib import Path
import re
import sys
import wave

ROOT = Path(__file__).resolve().parents[2]
for key in ('TEMP', 'TMP', 'TMPDIR', 'temp', 'tmp', 'tmpdir'):
    os.environ[key] = 'C:/Users/rdave/AppData/Local/Temp'
os.environ['CARGO_NET_OFFLINE'] = 'true'
import compatibility as c
sys.path.insert(0, str(ROOT.parent / 'speccade/packs/tracker_starter_v1'))
import build
import native
for env in (c.ENV, build.ENV):
    env.update({key: os.environ[key] for key in
                ('TEMP', 'TMP', 'TMPDIR', 'temp', 'tmp', 'tmpdir', 'CARGO_NET_OFFLINE')})


def timeline(name):
    """Authored row/tick times, including T memory and two SE1 rows."""
    tempo, time, ticks = 125, 0.0, []
    for row in range(8):
        count = 12 if name == 'S-pattern-delay' and row in (1, 2) else 6
        for tick in range(count):
            if name == 'T-set' and row == 1 and tick == 0:
                tempo = 150
            if name in ('T-up', 'T-down') and 1 <= row <= 5 and tick:
                tempo += 1 if name == 'T-up' else -1
            if name == 'T-gap' and row in (1, 3, 4, 5) and tick:
                tempo += 1
            if name == 'T-zero-up' and row == 1 and tick:
                tempo += 1
            if name == 'T-channel' and 1 <= row <= 5 and tick:
                tempo += -1 if row == 1 else 1
            if name == 'T-channel-isolation' and 1 <= row <= 5 and tick:
                tempo += -2 if row == 2 else 1
            if name == 'T-set-recall':
                if row == 1 and tick: tempo += 1
                if row == 2 and tick == 0: tempo = 150
            duration = 2.5 / tempo
            ticks.append({'row': row, 'tick': tick, 'bpm': tempo, 'time': time, 'duration': duration})
            time += duration
    return ticks, time


def check_native_trace(path, ticks, name):
    ends, elapsed = [], 0
    for tick in ticks:
        elapsed += 110250 // tick['bpm']
        ends.append(elapsed)
    errors, checked = [], 0
    for line in path.read_text().splitlines():
        frame = int(re.match(r'frame=(\d+)', line)[1])
        state = {key: int(value) for key, value in re.findall(
            r'\b(row|tick|bpm|tick_sample_pos|flags|order_position): (\d+)', line.split(' channel=')[0])}
        elapsed = (frame+1)*735
        index = bisect.bisect_right(ends, elapsed)
        if index == len(ticks):
            if state['flags'] & 1:
                errors.append({'check': 'exact native stop frame', 'frame': frame, 'state': state})
            continue
        expected = ticks[index]
        sample_pos = elapsed - (ends[index-1] if index else 0)
        # A row's tick-zero command is processed on its first rendered sample.
        if sample_pos == 0 and expected['tick'] == 0:
            continue
        wanted = {'row': expected['row'], 'tick': expected['tick'] % 6,
                  'bpm': expected['bpm'], 'tick_sample_pos': sample_pos, 'order_position': 0}
        if any(state[key] != value for key, value in wanted.items()):
            errors.append({'check': 'exact native frame clock', 'frame': frame, 'expected': wanted, 'actual': state})
        if name == 'S-cut':
            sounding = not (1 <= expected['row'] <= 5 and expected['tick'] >= 2)
            actual = re.search(r'\bnote_on: (true|false)', line)[1] == 'true'
            if actual != sounding:
                errors.append({'check': 'native SC2 cut tick', 'frame': frame, 'expected': sounding, 'actual': actual})
        if name.startswith('T-'):
            note = int(re.search(r'current_note: (\d+)', line)[1])
            if note != (73 if expected['row'] >= 6 else 61):
                errors.append({'check': 'native musical note state', 'frame': frame, 'note': note})
        checked += 1
    assert checked > 30, 'missing native trace'
    return errors, checked


def measurements(path, ticks):
    with wave.open(str(path)) as wav:
        assert (wav.getnchannels(), wav.getsampwidth(), wav.getframerate()) == (2, 2, 44100)
        pcm = array.array('h', wav.readframes(wav.getnframes()))
    assert max(map(abs, pcm)) < 32767, 'clipping'
    result = []
    for tick in ticks:
        start = round((tick['time'] + tick['duration'] * .35) * 44100)
        end = round((tick['time'] + tick['duration'] * .9) * 44100)
        lanes = [pcm[start * 2 + side:end * 2:2] for side in (0, 1)]
        assert all(lanes), (path, tick)
        block = lanes[0]
        crossings = [i - block[i] / (block[i+1] - block[i])
                     for i in range(len(block)-1) if block[i] <= 0 < block[i+1]]
        first, last = (math.ceil(crossings[0]), math.ceil(crossings[-1])) if len(crossings) > 2 else (0, len(block))
        rms = [math.sqrt(sum(v*v for v in lane[first:last]) / (last-first)) for lane in lanes]
        hz = (len(crossings)-1)*44100/(crossings[-1]-crossings[0]) if len(crossings) > 2 else 0
        result.append({**tick, 'rms': rms, 'hz': hz})
    return result, pcm


def check_cut_tails(native_pcm, reference_pcm):
    """SC2 stops oscillation at the cut; the oracle then discharges a DC tail.

    Full sample windows must show a sounding pre-cut tick, immediate native
    silence and a sign-preserving, monotonically decaying reference tail.
    Raw RMS measurements remain in the report; DC discharge is not a held note.
    """
    checks = []
    for row in range(1, 6):
        start = (row * 6 + 2) * 882
        for player, pcm in (('native', native_pcm), ('reference', reference_pcm)):
            for side in (0, 1):
                before = pcm[(start-882)*2+side:start*2:2]
                tail = pcm[start*2+side:(start+1764)*2:2]
                crossings = sum(a <= 0 < b or b <= 0 < a for a, b in zip(before, before[1:]))
                monotonic = all(abs(b) <= abs(a) + 1 and a*b >= 0 for a, b in zip(tail, tail[1:]))
                settled = max(map(abs, tail[-128:])) <= 1
                silent = max(map(abs, tail)) == 0
                checks.append({'row': row, 'player': player, 'side': side,
                    'pre_cut_crossings': crossings, 'tail_first': tail[0],
                    'tail_last': tail[-1], 'monotonic': monotonic,
                    'pass': crossings >= 20 and settled and (silent if player == 'native' else monotonic)})
    return checks


def main():
    assert len(sys.argv) == 2 and Path(sys.argv[1]).name == sys.argv[1] and sys.argv[1] not in ('.', '..')
    stage = c.CACHE / sys.argv[1]
    stage.mkdir(exist_ok=False)
    class RetainedDirectory:
        def __init__(self, prefix='renderer-', **kwargs):
            self.path = c.tempfile.mkdtemp(prefix=prefix, dir=stage)
        def __enter__(self):
            return self.path
        def __exit__(self, *args):
            return False
    c.tempfile.TemporaryDirectory = RetainedDirectory
    reference = c.CACHE / 'reference-openmpt-0.8.9/openmpt123.exe'
    assert reference.is_file() and c.digest(reference) == '9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c'
    assert c.filter_reference() == reference
    c.run(['cargo', 'build', '--offline', '--locked', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    c.run(['cargo', 'run', '--offline', '--locked', '-q', '-p', 'nether-it', '--example', 'effect_memory_probe', '--', stage / 'sources'])
    # Retain a per-frame native row/tick/channel trace without editing the shared helper.
    helper = stage / 'trace_renderer.rs'
    source = native.HELPER.read_text()
    source = source.replace('    for _ in 0..frames {',
        '    let mut trace = String::new();\n    for frame in 0..frames {')
    source = source.replace('        assert_eq!(buffer.len(), 1470);',
        '        trace.push_str(&format!("frame={} state={:?} channel={:?}\\n", frame, state, engine.snapshot().channels[0]));\n        assert_eq!(buffer.len(), 1470);')
    source = source.replace('    file.flush()?;', '    fs::write(format!("{}.state.txt", args[4]), trace)?;\n    file.flush()?;')
    helper.write_text(source)
    original = native.HELPER
    native.HELPER = helper
    try:
        identity = c.compile_compat_renderer(native, stage / 'renderer.exe')
    finally:
        native.HELPER = original
    identity['original_shared_helper_sha256'] = c.digest(original)
    report = {'renderer': identity, 'compile_count': 1, 'reference_sha256': c.digest(reference),
              'scope': '32 named controls x two explicit flag pairs; linear slides, mono sine, instrument mode, 44100 Hz',
              'thresholds': {'absolute_baseline_relative': .01, 'pitch_relative': .006, 'amplitude_baseline_units': .025},
              'cases': []}
    report['source_hashes'] = {str(path): c.digest(ROOT / path) for path in (
        Path('nether-it/examples/effect_memory_probe.rs'),
        Path('tools/tracker-debug/it_effect_probe.py'),
        Path('nethercore-zx/src/tracker/channels/mod.rs'),
        Path('nethercore-zx/src/tracker/engine/row_processing.rs'),
        Path('nethercore-zx/src/tracker/engine/effects.rs'),
        Path('nethercore-zx/src/tracker/engine/tick.rs'),
        Path('nethercore-zx/src/tracker/engine/render.rs'),
        Path('nethercore-zx/src/tracker/engine/mixing.rs'),
        Path('nethercore-zx/src/tracker/utils.rs'),
        Path('nethercore-zx/src/tracker/engine/rollback_tests.rs'))}
    for module in sorted((stage / 'sources').glob('*.it'), key=lambda p: (not p.stem.startswith('baseline'), p.name)):
        name = module.stem.rsplit('-old', 1)[0]
        case = stage / module.stem
        case.mkdir()
        ticks, end = timeline(name)
        item = {'name': module.stem, 'source_sha256': c.digest(module), 'authored_end_seconds': end}
        try:
            item['capture'] = c.capture_case(native.capture, ROOT, stage / 'renderer.exe', case, module, end + .3, case / 'native.wav')
            item['cart_sha256'] = c.digest(case / 'capture.nczx')
            trace_errors, item['exact_native_frames_checked'] = check_native_trace(case / 'native.wav.state.txt', ticks, name)
            args = [reference, '--batch', '--quiet', '--no-float', '--dither', '0', '--samplerate', '44100', '--channels', '2', '--gain', '0', '--stereo', '100', '--filter', '8', '--ramping', '-1', '--repeat', '0', '--subsong', '0', '--end-time', str(end + .3), '--output', case / 'reference.wav', '--', module]
            item['reference_command'] = [str(a) for a in args]
            c.run(args)
            native_ticks, native_pcm = measurements(case / 'native.wav', ticks)
            reference_ticks, reference_pcm = measurements(case / 'reference.wav', ticks)
            item['native'], item['reference'] = native_ticks, reference_ticks
            baseline = reference_ticks[0]['rms'][0]
            item['baseline_gain_ratio'] = native_ticks[0]['rms'][0] / baseline
            errors = trace_errors
            if name == 'S-cut':
                item['cut_tail_checks'] = check_cut_tails(native_pcm, reference_pcm)
                errors.extend({'check': 'cut tail', **check} for check in item['cut_tail_checks'] if not check['pass'])
            if abs(item['baseline_gain_ratio'] - 1) >= .01:
                errors.append({'check': 'absolute baseline', 'ratio': item['baseline_gain_ratio']})
            for a, b in zip(native_ticks, reference_ticks):
                detail = {'row': a['row'], 'tick': a['tick'], 'time': a['time']}
                cut_transition = name == 'S-cut' and 1 <= a['row'] <= 5 and a['tick'] == 2
                if not cut_transition and (a['rms'][0] > 10) != (b['rms'][0] > 10):
                    errors.append({**detail, 'check': 'gate', 'native': a['rms'][0], 'reference': b['rms'][0]})
                if a['hz'] and b['hz'] and abs(a['hz']/b['hz']-1) >= .006:
                    errors.append({**detail, 'check': 'pitch', 'native': a['hz'], 'reference': b['hz']})
                if not cut_transition and max(abs(x-y) for x, y in zip(a['rms'], b['rms'])) / baseline >= .025:
                    errors.append({**detail, 'check': 'absolute amplitude', 'native': a['rms'], 'reference': b['rms']})
                if any(abs(t['rms'][0]-t['rms'][1]) > 1 for t in (a,b)):
                    errors.append({**detail, 'check': 'center channel share'})
            # Reset marker and native stop are audible; CLI adds its known 100 ms tail.
            for player, values in (('native', native_ticks), ('reference', reference_ticks)):
                value = next(t for t in values if t['row'] == 6 and t['tick'] == 2)
                if abs(value['hz'] / 1378.125 - 1) >= .006:
                    errors.append({'check': 'row-six pitch marker', 'player': player, 'measurement': value})
            if max(map(abs, native_pcm[round((end+.002)*44100)*2:]), default=0) > 1:
                errors.append({'check': 'native authored stop'})
            if abs(len(reference_pcm)/2/44100 - end - .1) > .002:
                errors.append({'check': 'reference duration', 'actual': len(reference_pcm)/2/44100})
            item['errors'] = errors
            item['pass'] = not errors and not item['capture']['playing'] and item['capture']['wraps'] == 0
        except Exception as exc:
            item.update({'pass': False, 'error': repr(exc)})
        report['cases'].append(item)
        (stage / 'report.json').write_text(json.dumps(report, indent=2))
        print(item['name'], item['pass'], item.get('error', item.get('errors', [])[:2]), flush=True)
    report['passed'] = sum(item['pass'] for item in report['cases'])
    report['total'] = len(report['cases'])
    assert report['total'] == 64, 'finite inventory changed'
    tempo_names = ('T-up', 'T-down', 'T-gap', 'T-zero-up', 'T-set', 'T-channel-isolation', 'T-set-recall')
    selected = [item for item in report['cases'] if item['name'].rsplit('-old', 1)[0] in tempo_names]
    assert len(selected) == 14
    report['tempo_memory_slice'] = {'names': [item['name'] for item in selected],
        'passed': sum(item['pass'] for item in selected), 'total': len(selected),
        'simultaneous_tempo_cases': [item['name'] for item in report['cases'] if item['name'].startswith('T-channel-old')]}
    assert c.digest(ROOT / 'target/debug/deps' / identity['runtime_library']) == identity['runtime_sha256'], 'runtime changed during capture'
    assert all(c.digest(ROOT / path) == digest for path, digest in report['source_hashes'].items()), 'source changed during capture'
    (stage / 'report.json').write_text(json.dumps(report, indent=2))
    print(f"{report['passed']}/{report['total']} passing", flush=True)
    return 0 if report['passed'] == report['total'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
