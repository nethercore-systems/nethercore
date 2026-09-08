"""Repeatable packed-playback probes using the existing SpecCade capture helper.
Results are diagnostic evidence, NOT blanket XM/IT certification.
"""
import argparse
import array
import hashlib
import json
import math
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import urllib.request
import wave

ROOT = Path(__file__).resolve().parents[2]
CACHE = ROOT / 'target/tracker-compatibility'
PIN = '6ec0ba21b1b28f91e22b68a51d59207c6bbf6139'
BASE = f'https://raw.githubusercontent.com/libxmp/libxmp/{PIN}/'
ENV = {k: v for k, v in os.environ.items() if k.upper() not in ('TEMP', 'TMP', 'TMPDIR')}
_temporary_root = str(Path.home() / 'AppData/Local/Temp') if os.name == 'nt' else '/tmp'
for _key in ('TEMP', 'TMP', 'TMPDIR', 'temp', 'tmp', 'tmpdir'):
    ENV[_key] = _temporary_root

def run(args):
    result = subprocess.run([str(a) for a in args], cwd=ROOT, env=ENV, capture_output=True, text=True, timeout=900)
    if result.returncode:
        raise RuntimeError(result.stdout[-3000:] + result.stderr[-3000:])
    return result.stdout

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def fetch():
    url = f'https://api.github.com/repos/libxmp/libxmp/git/trees/{PIN}?recursive=1'
    with urllib.request.urlopen(url, timeout=60) as response:
        tree = json.load(response)['tree']
    records = []
    for item in tree:
        name = item['path']
        if not name.startswith(('test-dev/openmpt/xm/', 'test-dev/openmpt/it/')):
            continue
        if not (name.lower().endswith(('.xm', '.it')) or name.endswith('00_README')):
            continue
        path = CACHE / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.exists():
            with urllib.request.urlopen(BASE + name, timeout=60) as response:
                path.write_bytes(response.read())
        data = path.read_bytes()
        blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        assert blob == item['sha'], f'Wrong pinned source bytes: {name}'
        records.append({'path': name, 'url': BASE + name, 'sha256': digest(path)})
    records.sort(key=lambda x: x['path'])
    (CACHE / 'sources.json').write_text(json.dumps({'commit': PIN, 'purpose': 'local playback compatibility testing; suite README explicitly invites other-player tests; not game assets or a redistribution license', 'files': records}, indent=2))
    print(f'Fetched and verified {len(records)} source files')

def features(path):
    with wave.open(str(path), 'rb') as wav:
        assert wav.getnchannels() == 2 and wav.getsampwidth() == 2
        rate = wav.getframerate()
        pcm = array.array('h', wav.readframes(wav.getnframes()))
    if sys.byteorder != 'little':
        pcm.byteswap()
    hop = rate // 50
    windows = []
    for start in range(0, len(pcm), hop * 2):
        block = pcm[start:start + hop * 2]
        left, right = block[::2], block[1::2]
        el, er = sum(x*x for x in left), sum(x*x for x in right)
        windows.append({'rms': math.sqrt((el + er) / max(1, len(block))) / 32768,
                        'pan': (er - el) / max(1, er + el),
                        'crossings': sum((a < 0) != (b < 0) for a, b in zip(left, left[1:]))})
    return {'rate': rate, 'frames': len(pcm)//2, 'windows_20ms': windows}

def capture_case(capture, root, renderer, stage, module, seconds, output):
    try:
        return capture(root, renderer, stage, module, seconds, output)
    except AssertionError as exc:
        # The shared music-pack helper rejects quiet WAVs after verifying finite
        # playback. Compatibility fixtures can intentionally be quieter than songs.
        detail = exc.args[0] if exc.args else None
        if not (isinstance(detail, tuple) and len(detail) == 3
                and detail[0] == output and detail[1] == 'silent or clipped'):
            raise
        with wave.open(str(output), 'rb') as wav:
            assert wav.getnchannels() == 2 and wav.getsampwidth() == 2
            pcm = array.array('h', wav.readframes(wav.getnframes()))
        if sys.byteorder != 'little':
            pcm.byteswap()
        peak = max(map(abs, pcm), default=0)
        if not 0 < peak <= 100:
            raise
        return {'quiet_fixture_peak_pcm16': peak,
                'playing': False, 'warning': 'Below song-pack loudness gate; not silence or a compatibility pass'}


def compile_compat_renderer(native, destination):
    # Reuse the shared renderer without modifying the sibling repository.
    # Empty packed IDs are intentional silent slots (handle zero in production).
    original = native.HELPER
    source = original.read_text()
    anchor = '    for id in &tracker.sample_ids {\n'
    assert source.count(anchor) == 1, 'Shared renderer changed: review silent-slot adaptation'
    source = source.replace(anchor, anchor + '        if id.is_empty() { handles.push(0); continue; }\n')
    anchor = '    let frames = (seconds * 60.0).ceil() as u32;'
    assert source.count(anchor) == 1, 'Shared renderer changed: review restart probe'
    source = source.replace(anchor, r'''    // Verify restarting this freshly packed module reproduces its initial frame.
    let initial_state = state;
    let mut first = Vec::new();
    generate_audio_frame_with_tracker(&mut playback, &mut state, &mut engine,
        &sounds, 60, 44100, &mut first);
    let mut resumed = Vec::new();
    for _ in 0..17 {
        generate_audio_frame_with_tracker(&mut playback, &mut state, &mut engine,
            &sounds, 60, 44100, &mut resumed);
    }
    engine.reset();
    state = initial_state;
    playback = AudioPlaybackState::default();
    generate_audio_frame_with_tracker(&mut playback, &mut state, &mut engine,
        &sounds, 60, 44100, &mut resumed);
    assert_eq!(first, resumed, "restart changed the first packed playback frame");
    engine.reset();
    state = initial_state;
    playback = AudioPlaybackState::default();
''' + anchor)
    with tempfile.TemporaryDirectory(prefix='tracker-renderer-') as temporary:
        adapted = Path(temporary) / 'render_tracker.rs'
        adapted.write_text(source)
        try:
            native.HELPER = adapted
            identity = native.compile_renderer(ROOT, destination)
        finally:
            native.HELPER = original
    identity['shared_helper_sha256'] = digest(original)
    identity['adaptation'] = 'empty sample ID maps to silent handle zero; assert exact first-frame PCM after restart'
    return identity


def volume_porta_probe(helper_root):
    """Compare original volume/main-column pairs within each player, not across PCM engines."""
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='volume-porta-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'volume_porta_probe', '--', stage])
        for digit in range(10):
            native_pcm, reference_pcm = {}, {}
            for column in ('main', 'volume'):
                module, output = stage / f'g{digit}-{column}.it', stage / f'{column}.wav'
                capture_case(native.capture, ROOT, renderer, stage, module, 1.2, output)
                with wave.open(str(output), 'rb') as wav:
                    native_pcm[column] = wav.readframes(wav.getnframes())
                run(['ffmpeg', '-v', 'error', '-y', '-f', 'libopenmpt', '-i', module,
                     '-t', '1.2', '-ar', '44100', '-ac', '2', output])
                with wave.open(str(output), 'rb') as wav:
                    reference_pcm[column] = wav.readframes(wav.getnframes())
            assert native_pcm['main'] == native_pcm['volume'], f'Native G{digit} mismatch'
            assert reference_pcm['main'] == reference_pcm['volume'], f'Reference G{digit} mismatch'
        print('PASS: all 10 volume/main portamento pairs match within each player; packed termination/restart verified')


def xm_global_volume_probe(helper_root, cut=False, keyoff=False, noteoff=False, fade=False, delay=False, delay_porta=False, empty_delay=False, delay_slide=False, sample_volume=False, zero_slide=False, sample_pan=False, zero_pan=False, sample_porta=False, high_patterns=False, loop_bits=False, multi_sample=False, multi_empty=False, env_escape=False, pan_freeze=False, env_setpos=False, pan_law=False, invalid_selection=False, selection=99, selection_envelope=False, selection_k00=False):
    """Original one-channel fixtures for saturated XM Gxx values."""
    import struct
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='xm-global-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        actual, reference = [], []
        for value in ((0, 64, 128, 192, 255) if pan_law else (0, 1) if multi_sample or env_escape or pan_freeze or env_setpos else (1, 0x11, 2, 0x12) if loop_bits else (0, 254, 255) if high_patterns else (0xd0, 0xe0) if zero_pan else (0, 128, 255) if sample_pan else (0, 0x60, 0x70) if zero_slide else (0, 32, 64) if sample_volume else (0, 1) if delay_porta else (1, 3, 6) if delay else (256,) if fade else (0, 256, 512, 768, 1024) if noteoff else (0, 1, 3, 256, 512) if keyoff else (0, 1, 3) if cut else (64, 65, 127, 128, 255, 16, 32)):
            data = bytearray(336)
            data[:17] = b'Extended Module: '
            data[37] = 26
            if zero_pan or env_escape or pan_freeze or pan_law:
                # Match FT2's space-padded title, tracker and sample names;
                # OpenMPT otherwise selects different compatibility semantics.
                data[17:37] = b'Pan zero probe'.ljust(20, b' ')
                data[38:58] = b'FastTracker v2.00'.ljust(20, b' ')
            if env_escape or pan_freeze:
                # These original fixtures exercise FT2 semantics, not ambiguous
                # legacy creator inference. The explicit FT2 clone control
                # selects those semantics in the external oracle.
                data[38:58] = b'Fasttracker II clone'
            struct.pack_into('<HI8H', data, 58, 0x104, 276, 1, 0, 1, 1, 1, 1, 6, 125)
            notes = (bytes([49, 1, 0x30, 0, 0, 0, 1 if keyoff and value == 256 else 0, 0xc8 if keyoff and value == 512 else 0, 0x14 if keyoff else 0x0e, (value & 255) if keyoff else 0xc0 | value,
                            0, 0, 0, 0x0c, 32, 0, 0, 0, 0, 0]) if cut else
                     bytes([49, 1, 0x30, 0x10, value] + [0, 0, 0, 0, 0] * 3))
            if env_escape or pan_freeze:
                notes = bytes([49, 1, 0x30, 0, 0, 97, 0, 0, 0, 0] + [0, 0, 0, 0, 0] * 2)
            if multi_sample:
                notes = bytes([49, 1, 0, 0, 0, 61, 1 if value or multi_empty else 2, 0, 0, 0,
                               49, 1, 0, 0, 0, 0, 0, 0, 0, 0])
            if loop_bits:
                notes = bytes([49, 1, 0x30, 0, 0] + [0, 0, 0, 0, 0] * 3)
            if delay:
                notes = bytes([49, 1, 0x30, 0x0e, 0xd0 | value,
                               49, 1, 0x30, 0, 0] + [0, 0, 0, 0, 0] * 2)
            if delay_porta:
                notes = bytes([49, 1, 0x30, 0, 0, 61, 1, 0xf1 if value else 0, 0x0e, 0xd3] + [0, 0, 0, 0, 0] * 2)
            if empty_delay:
                notes = bytes([49, 1, 0x30, 0, 0, 0 if value else 49, 0, 0, 0x0e, 0xd3] + [0, 0, 0, 0, 0] * 2)
            if delay_slide:
                notes = bytes([49, 1, 0x30, 0, 0, 61, 1, 0x6f if value else 0, 0x0e, 0xd3] + [0, 0, 0, 0, 0] * 2)
            if zero_slide:
                notes = bytes([49, 1, 0x30, 0, 0, 0, 0, 0x61, 0, 0,
                               0, 0, value, 0, 0, 0, 0, 0, 0, 0])
            if zero_pan:
                notes = bytes([49, 1, 0x30, 0, 0, 0, 0, value, 0, 0,
                               0, 0, 0xc8, 0, 0, 0, 0, 0, 0, 0])
            if sample_pan or pan_law:
                notes = bytes([49, 1, 0x30, 0, 0, 49, 1, 0x30, 8, 128] + [0, 0, 0, 0, 0] * 2)
            if sample_porta:
                notes = bytes([49, 1, 0x30, 0, 0, 61, 2, 0x30, 3, 16] + [0, 0, 0, 0, 0] * 2)
            if sample_volume:
                notes = bytes([49, 1, 0, 0, 0, 49, 1, 0x30, 0, 0] + [0, 0, 0, 0, 0] * 2)
            if noteoff:
                notes = bytearray(notes)
                notes[5:10] = bytes([97, 1 if value == 256 else 0,
                    0xc8 if value == 512 else 0x30 if value == 768 else 0,
                    0x0c if value == 1024 else 0, 32 if value == 1024 else 0])
            if env_setpos:
                notes = bytes([49, 1, 0, 0, 0, 0, 0, 0, 0x15, 24] + [0, 0, 0, 0, 0] * 2)
            if high_patterns:
                struct.pack_into('<H', data, 70, value + 1)
                data[80] = value
                data.extend(struct.pack('<IBHH', 9, 0, 4, 0) * value)
                notes = bytes([49, 1, 0x30, 0, 0] + [0, 0, 0, 0, 0] * 3)
            data.extend(struct.pack('<IBHH', 9, 0, 4, len(notes)) + notes)
            instrument_size = 263 if pan_law else 243
            instrument = bytearray(instrument_size)
            struct.pack_into('<I', instrument, 0, instrument_size)
            instrument[4:8] = b'tone'
            if fade:
                struct.pack_into('<H', instrument, 239, 4096)
            struct.pack_into('<HI', instrument, 27, 1, 40)
            if env_escape:
                for index, (tick, volume) in enumerate(((0, 64), (4, 32), (10, 0))):
                    struct.pack_into('<HH', instrument, 129 + index*4, tick, volume)
                instrument[225] = 3  # Volume envelope point count.
                instrument[227:230] = bytes([value, 0, 1])  # Sustain, loop start/end nodes.
                instrument[233] = 7  # Enabled + sustain + loop.
            if env_setpos:
                struct.pack_into('<HHHH', instrument, 129, 0, 64, 32, 64)
                instrument[225], instrument[227], instrument[233] = 2, 0, 1 | (2 if value else 0)
                struct.pack_into('<HHHHHH', instrument, 177, 0, 32, 16, 64, 24, 0)
                instrument[226], instrument[234] = 3, 1
            if pan_freeze:
                struct.pack_into('<HHHH', instrument, 129, 0, 64, 20, 64)
                instrument[225], instrument[233] = 2, 1
                for index, (tick, pan) in enumerate(((0, 32), (2, 64), (8, 0))):
                    struct.pack_into('<HH', instrument, 177 + index*4, tick, pan)
                instrument[226], instrument[230], instrument[234] = 3, 1, 1 | (2 if value else 0)
            if selection_envelope:
                struct.pack_into('<HHHH', instrument, 129, 0, 64, 10, 0)
                instrument[225], instrument[233] = 2, 1
            data.extend(instrument)
            sample = bytearray(40)
            struct.pack_into('<III', sample, 0, 64, 0, 64)
            sample[12], sample[14], sample[15] = value if sample_volume else 64 if delay_slide else 32, 1, 128
            if zero_pan or pan_law:
                sample[18:40] = b' ' * 22  # FT2 space-padded empty sample name.
            if sample_pan or pan_law:
                sample[15] = value
            if loop_bits:
                width = 2 if value & 0x10 else 1
                struct.pack_into('<III', sample, 0, 64 * width, 16 * width, 32 * width)
                sample[14] = value
            data.extend(sample)
            previous = 0
            for amplitude in ([16] * 64 if pan_law else [16] * 32 + [-16] * 32):
                if loop_bits and value & 0x10:
                    data.extend(struct.pack('<H', ((amplitude - previous) * 256) & 65535))
                else:
                    data.append((amplitude - previous) & 255)
                previous = amplitude
            module, output = stage / f'g{value}.xm', stage / 'audio.wav'
            if sample_porta:
                struct.pack_into('<H', data, 72, 2)
                second = bytearray(data[-347:])  # 243-byte instrument + 40-byte sample header + 64 PCM
                second[4:8] = b'two!'
                second[243 + 15] = 255 - value
                data.extend(second)
            if invalid_selection:
                data[-347 + 243 + 15] = 0
            if multi_sample:
                base = len(data) - 347
                first = bytearray(data[base:])
                second = bytearray(first)
                if selection_envelope:
                    struct.pack_into('<HHHH', second, 129, 0, 64, 1, 64)
                    second[225], second[233] = 2, 1
                second[243 + 12], second[243 + 15], second[243 + 16] = 16, 255, 12
                previous = 0
                for n in range(64):
                    amplitude = 16 if n % 16 < 8 else -16
                    second[283 + n] = (amplitude - previous) & 255
                    previous = amplitude
                if multi_empty:
                    struct.pack_into('<III', first, 243, 0, 0, 0)
                    first = first[:283]
                    first[33:129] = bytes([1] * 96)
                if value:
                    struct.pack_into('<H', first, 27, 2)
                    first[33 + 60] = 1
                    data[base:] = first[:243] + first[243:283] + second[243:283] + first[283:] + second[283:]
                elif multi_empty:
                    data[base:] = second
                else:
                    struct.pack_into('<H', data, 72, 2)
                    data.extend(second)
            if invalid_selection:
                if selection == 2 and value:
                    struct.pack_into('<H', data, 72, 2)
                    data.extend(second)
                struct.pack_into('<HH', data, 341, 5, 25)
                data[345:365] = bytes([49, 1, 0x30, 0, 0, 0, 0, 0, 8, 128,
                                       0, selection, 0x30 if selection_k00 else 0,
                                       0x0c if selection_k00 else 0, 0,
                                       49, 0, 0, 0, 0, 49, 1, 0x30, 8, 255])
            module.write_bytes(data)
            capture_case(native.capture, ROOT, renderer, stage, module, 0.7, output)
            with wave.open(str(output), 'rb') as wav:
                actual.append(wav.readframes(wav.getnframes()))
            if pan_law or invalid_selection:
                run([filter_reference(), '--batch', '--quiet', '--no-float', '--dither', '0',
                     '--samplerate', '44100', '--channels', '2', '--gain', '0',
                     '--stereo', '100', '--filter', '8', '--ramping', '-1',
                     '--repeat', '0', '--subsong', '-1', '--end-time', '0.7',
                     '--force', '--output', output, '--', module])
            else:
                run(['ffmpeg', '-v', 'error', '-y', '-f', 'libopenmpt', '-i', module,
                 '-t', '0.7', '-ar', '44100', '-ac', '2', output])
            with wave.open(str(output), 'rb') as wav:
                reference.append(wav.readframes(wav.getnframes()))
        if invalid_selection:
            valid_selection = selection == 2
            observations = []
            semantic_metrics = []
            for player, outputs in [('reference', reference), ('native', actual)]:
                modes = []
                metrics = []
                for pcm in outputs:
                    decoded = array.array('h', pcm)
                    if sys.byteorder != 'little': decoded.byteswap()
                    states = []
                    row_metrics = []
                    for start in (.06, .18, .30, .42, .54):
                        segment = decoded[int(start*44100)*2:int((start+.025)*44100)*2]
                        energy = [sum(abs(v) for v in segment[side::2]) for side in (0, 1)]
                        total = sum(energy)
                        states.append(None if total < len(segment) else energy[0]/total)
                        mono = segment[::2]
                        signs = [sample >= 0 for sample in mono if sample]
                        transitions = sum(a != b for a, b in zip(signs, signs[1:]))
                        row_metrics.append((total, transitions))
                    modes.append(states)
                    metrics.append(row_metrics)
                print('XM', 'valid' if valid_selection else 'invalid',
                      'selection pan/activity', player, modes)
                if valid_selection:
                    print('XM valid selection voice metrics', player, metrics)
                    semantic_metrics.append(metrics)
                observations.append(modes)
            for reference_modes, native_modes in zip(*observations):
                for expected, actual_pan in zip(reference_modes, native_modes):
                    assert (expected is None) == (actual_pan is None), (observations, 'activity')
                    if expected is not None:
                        assert abs(expected-actual_pan) < .02, (observations, 'pan')
            if valid_selection:
                for reference_metrics, native_metrics in zip(*semantic_metrics):
                    # Row 2 is selection-only; row 3 is the first selected pitched note.
                    if selection_envelope:
                        for metrics, player in ((reference_metrics, 'reference'), (native_metrics, 'native')):
                            row_index = 2 if selection_k00 else 1
                            ratio = metrics[row_index][0] / metrics[0][0]
                            if selection_k00:
                                assert ratio < .10, (semantic_metrics, player, 'K00 selection must not retrigger envelope')
                            else:
                                assert ratio > .10, (semantic_metrics, player, 'selection envelope retrigger')
                    if selection_k00:
                        assert reference_metrics[2][0] == 0, (semantic_metrics, 'reference K00 selection activity')
                        assert native_metrics[2][0] == 0, (semantic_metrics, 'native K00 selection activity')
                        continue
                    assert reference_metrics[2][1] < reference_metrics[3][1], (semantic_metrics, 'reference sample timing')
                    assert native_metrics[2][1] < native_metrics[3][1], (semantic_metrics, 'native sample timing')
                    assert native_metrics[3][1] >= native_metrics[2][1] * 2, (semantic_metrics, 'sample identity/pitch')
                    # Compare equal-pan rows against each player's initial voice:
                    # absolute XM gain is a separate known failing pan-law probe.
                    for row_index in (2, 3):
                        reference_gain = reference_metrics[row_index][0] / reference_metrics[0][0]
                        native_gain = native_metrics[row_index][0] / native_metrics[0][0]
                        assert abs(native_gain / reference_gain - 1) < .10, (semantic_metrics, 'relative volume', row_index)
                    for row_index in (2, 3):
                        assert abs(native_metrics[row_index][1] - reference_metrics[row_index][1]) <= 1, (semantic_metrics, 'sample transition timing', row_index)
                print('PASS: XM valid-selection sample identity/pitch, default-volume retention, timing, pan; fresh official reference')
            else:
                print('PASS: XM invalid-selection activity/pan and recovery, legacy and mapped; fresh official reference')
            return
        for player, outputs in [('reference', reference), ('native', actual)]:
            if pan_law:
                levels = []
                for pcm in outputs:
                    data = array.array('h', pcm)
                    if sys.byteorder != 'little': data.byteswap()
                    levels.append([max(map(abs, data[int(.06*44100)*2 + side:int(.09*44100)*2:2])) for side in (0,1)])
                print('XM constant pan law', player, levels)
                unity = levels[0][0]
                for pan, (left, right) in zip((0,64,128,192,255), levels):
                    assert abs(left/unity - math.sqrt(1-pan/256)) < .01, (player, pan, left, unity)
                    assert abs(right/unity - math.sqrt(pan/256)) < .01, (player, pan, right, unity)
                if player == 'reference':
                    reference_levels = levels
                else:
                    for pan, expected, got in zip((0, 64, 128, 192, 255), reference_levels, levels):
                        for side in (0, 1):
                            assert abs(got[side] - expected[side]) <= max(2, expected[side] * .02), (pan, side, expected, got, 'absolute XM gain')
                    print('PASS: original XM pan shape and absolute steady gain; fresh packed native/official reference')
                continue
            if env_setpos:
                for sustain, pcm in enumerate(outputs):
                    decoded = [v for (v,) in struct.iter_unpack('<h', pcm)]
                    part = decoded[int(.145*44100)*2:int(.175*44100)*2]
                    left, right = [max(map(abs, part[side::2])) for side in (0,1)]
                    assert left + right > 100
                    assert (right < 5) if sustain else (right/(left+right) > .6), (player, sustain, left, right)
                continue
            if pan_freeze:
                for held, pcm in enumerate(outputs):
                    samples = [v for (v,) in struct.iter_unpack('<h', pcm)]
                    late = samples[int(.30*44100)*2:int(.40*44100)*2]
                    left, right = [max(map(abs, late[side::2])) for side in (0,1)]
                    assert (right > 100 and right/(left+right) > .9) if held else (right < 5 and left > 100), (player, held, left, right)
                continue
            if env_escape:
                for escaped, pcm in enumerate(outputs):
                    samples = [v for (v,) in struct.iter_unpack('<h', pcm)]
                    early = max(map(abs, samples[int(.02*44100)*2:int(.08*44100)*2]))
                    late = max(map(abs, samples[int(.34*44100)*2:int(.42*44100)*2]))
                    assert early > 100, (player, escaped, 'silent')
                    assert late < 5 if escaped else late > 100, (player, escaped, early, late)
                continue
            if multi_sample:
                assert any(outputs[0]), (player, 'silent multisample control')
                assert outputs[0] == outputs[1], (player, 'keymapped samples differ from separate instruments')
                continue
            if loop_bits:
                for first in (0, 2):
                    assert any(outputs[first]), (player, 'silent loop control')
                    assert outputs[first] == outputs[first + 1], (player, first, '8/16-bit loop differs')
                continue
            if high_patterns:
                assert any(outputs[0]), (player, 'silent pattern control')
                assert outputs[0] == outputs[1] == outputs[2], (player, 'high pattern index changes playback')
                continue
            if sample_pan or zero_pan:
                for value, pcm in zip((0xd0,0xe0) if zero_pan else (0, 128, 255), outputs):
                    decoded = [v for (v,) in struct.iter_unpack('<h', pcm)]
                    def left_share(a, b):
                        segment = decoded[int(a*44100)*2:int(b*44100)*2]
                        left, right = sum(abs(v) for v in segment[::2]), sum(abs(v) for v in segment[1::2])
                        assert left + right > 0
                        return left/(left+right)
                    if zero_pan:
                        assert abs(left_share(.125,.135)-.5) < .02, (player,value,'tick0')
                        # FT2-compatible reference mixing ramps pan after the tick-1
                        # state change; verify onset and settled pan separately.
                        if value == 0xd0:
                            assert left_share(.15,.17) > .75, (player,value,'tick1 onset')
                            assert left_share(.18,.19) > .98, (player,value,'settled left')
                        else:
                            assert abs(left_share(.15,.19)-.5) < .02, (player,value,'E0 no-op')
                        assert abs(left_share(.30,.33)-.5) < .02, (player,value,'reset')
                        continue
                    assert abs(left_share(.06,.09) - (1-value/256)) < .02, (player,value,'default pan')
                    assert abs(left_share(.20,.23) - (1-value/256 if sample_porta else .5)) < .02, (player,value,'explicit instrument/row pan')
                continue
            if zero_slide:
                assert outputs[0] == outputs[1] == outputs[2], (player, 'zero-volume slide recalled memory')
                continue
            if sample_volume:
                decoded = [[v for (v,) in struct.iter_unpack('<h', pcm)] for pcm in outputs]
                first = [max(map(abs, pcm[int(.07*88200):int(.10*88200)])) for pcm in decoded]
                assert first[0] < 10 and first[2] > 100, (player, first)
                assert abs(first[1]/first[2] - .5) < .06, (player, first)
                override = [max(map(abs, pcm[int(.20*88200):int(.23*88200)])) for pcm in decoded]
                assert max(override)-min(override) < max(override)*.06, (player, override)
                continue
            if delay_slide:
                outputs = [[v for (v,) in struct.iter_unpack('<h', pcm)] for pcm in outputs]
                for a, b, expected in ((.158, .1595, 17/32), (.198, .1995, 1), (.218, .2195, 49/64)):
                    peaks = [max(abs(x) for x in pcm[int(a*88200):int(b*88200)]) for pcm in outputs]
                    assert abs(peaks[1]/peaks[0]-expected) < .06, (player, a, peaks, expected)
                continue
            if delay_porta:
                assert outputs[0] == outputs[1], (player, 'ED3 plus F1 differs from fresh-note control')
                continue
            if delay:
                for value, pcm in zip((1, 3, 6), outputs):
                    values = [v for (v,) in struct.iter_unpack('<h', pcm)]
                    def peak(a, b):
                        return max(map(abs, values[int(a*44100)*2:int(b*44100)*2]))
                    assert peak(.005, .015) < 10, (player, value, 'premature EDx')
                    assert (peak(.085, .105) > 100 if value < 6 else peak(.085, .105) < 10), (player, value)
                    assert peak(.15, .18) > 100, (player, value, 'next row')
                continue
            if fade:
                values = [v for (v,) in struct.iter_unpack('<h', outputs[0])]
                def peak(a, b):
                    return max(map(abs, values[int(a*44100)*2:int(b*44100)*2]))
                assert peak(.15, .17) > 100, (player, 'release unexpectedly silent')
                assert peak(.30, .33) < 10, (player, 'XM fade lasts too long')
                continue
            if cut:
                for tick, pcm in zip((0, 256, 512, 768, 1024) if noteoff else (0, 1, 3, 256, 512) if keyoff else (0, 1, 3), outputs):
                    values = [v for (v,) in struct.iter_unpack('<h', pcm)]
                    def peak(a, b):
                        return max(map(abs, values[int(a*44100)*2:int(b*44100)*2]))
                    assert peak(.06, .09) > 100, (player, tick)
                    assert (peak(.21, .23) > 100 if (tick in (256, 768, 1024) if noteoff else tick >= 256) else peak(.21, .23) < 10), (player, tick)
                    assert peak(.30, .33) > 100, (player, tick, 'volume did not restore')
                    if tick and not noteoff:
                        assert peak(.125, .135) > 100, (player, tick)
                continue
            assert all(pcm == outputs[0] for pcm in outputs[:5]), f'{player} Gxx saturation mismatch'
            energies = [sum(v * v for (v,) in struct.iter_unpack('<h', pcm)) for pcm in outputs]
            assert energies[0] > 0, f'{player} silent full-volume fixture'
            for index, gain in [(5, 0.25), (6, 0.5)]:
                ratio = math.sqrt(energies[index] / energies[0])
                assert abs(ratio - gain) < 0.01, (player, gain, ratio)
        if env_setpos:
            print('PASS: XM Lxx updates pan only with volume-sustain flag, both players')
            return
        if pan_freeze:
            print('PASS: XM reached pan sustain stays held after release; non-sustain control continues, both players')
            return
        if env_escape:
            print('PASS: XM key-off escapes only a loop sharing its sustain-end node in both players')
            return
        print('PASS: five XM pan positions verified; Gxx saturation is a separate probe' if pan_law else 'PASS: XM empty first sample preserves second-slot playback independently in both players' if multi_empty else 'PASS: XM two-sample keymap matches separate-instrument control independently in both players; fresh packing verified' if multi_sample else 'PASS: XM 8/16-bit forward and pingpong loops match independently in both players; fresh packing verified' if loop_bits else 'PASS: XM patterns 0/254/255 match independently in both players; fresh packing/restart/termination verified' if high_patterns else 'PASS: XM volume D0 pans left after tick0; E0 does nothing; row reset verified' if zero_pan else 'PASS: XM portamento retains old sample pan across explicit instrument changes' if sample_porta else 'PASS: XM sample pans0/128/255 and explicit override survive fresh packing' if sample_pan else 'PASS: XM volume-column 60/70 match no-command after a slide in both players' if zero_slide else 'PASS: XM sample defaults zero/half/full and explicit volume override survive packing' if sample_volume else 'PASS: XM ED3 volume slide timing matches reference before/at/after trigger' if delay_slide else 'PASS: XM empty ED3 matches last-note retrigger independently in both players' if empty_delay else 'PASS: XM ED3 plus F1 matches fresh-note control independently in both players' if delay_porta else 'PASS: XM ED1/ED3 defer and ED6 ignores pitched notes; packed/reference termination/restart verified' if delay else 'PASS: XM authored fade4096 has reference-aligned release duration; packed termination/restart verified' if fade else 'PASS: XM note97 volume/instrument exceptions; packed/reference termination/restart verified' if noteoff else 'PASS: XM K00 mute/context-fade and K01/K03 delayed release; packed/reference termination/restart verified' if keyoff else 'PASS: XM EC0/EC1/EC3 mute then volume restores playback; packed/reference termination/restart verified' if cut else 'PASS: seven Gxx sources preserve saturation and quarter/half gain in both players; packed termination/restart verified')


def panning_defaults_probe(helper_root):
    """Original IT fixtures exercise note defaults, surround and effect ordering."""
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='pan-defaults-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'panning_defaults_probe', '--', stage])
        for mode in ('sample', 'instrument', 'both'):
            module, output = stage / f'pan-{mode}.it', stage / 'audio.wav'
            capture_case(native.capture, ROOT, renderer, stage, module, 0.9, output)
            for player in ('native', 'reference'):
                if player == 'reference':
                    run(['ffmpeg', '-v', 'error', '-y', '-f', 'libopenmpt', '-i', module,
                         '-t', '0.9', '-ar', '44100', '-ac', '2', output])
                with wave.open(str(output), 'rb') as wav:
                    pcm = array.array('h', wav.readframes(wav.getnframes()))
                if sys.byteorder != 'little':
                    pcm.byteswap()
                for row, expectation in [(0, 'surround'), (1, 'right'), (2, 'center'), (3, 'right'), (4, 'center')]:
                    start, end = int((row * 0.12 + 0.04) * 44100), int((row * 0.12 + 0.09) * 44100)
                    left, right = pcm[start*2:end*2:2], pcm[start*2+1:end*2:2]
                    peaks = [max(map(abs, side), default=0) for side in (left, right)]
                    assert max(peaks) > 100, (mode, player, row, 'silent')
                    if expectation == 'right':
                        assert peaks[0] == 0, (mode, player, row, peaks)
                    elif expectation == 'center':
                        assert left == right, (mode, player, row, 'not centered')
                    else:
                        assert all(a == -b for a, b in zip(left, right)), (mode, player, row, 'not surround')
        print('PASS: three fresh IT fixtures preserve note pan defaults, sample precedence, surround override and row effect ordering in both players; termination/restart verified')


def filter_reference():
    """Official reference pinned independently of FFmpeg's embedded library."""
    path = CACHE / 'reference-openmpt-0.8.9/openmpt123.exe'
    if not path.exists():
        import io
        import zipfile
        url = 'https://lib.openmpt.org/files/libopenmpt/bin/libopenmpt-0.8.9+release.bin.windows.zip'
        with urllib.request.urlopen(url, timeout=60) as response:
            data = response.read()
        assert hashlib.sha256(data).hexdigest() == 'd20c7589b323013ce55d2198fd55f1a0586e66b98c7cede59dea7ee97460eb79', 'Reference archive changed'
        with zipfile.ZipFile(io.BytesIO(data)) as archive:
            for name in archive.namelist():
                if name.startswith('openmpt123/amd64/') or name == 'LICENSE.txt' or (name.startswith('Licenses/') and not name.endswith('/')):
                    target = path.parent / (Path(name).name if name.startswith('openmpt123/amd64/') else name)
                    target.parent.mkdir(parents=True, exist_ok=True)
                    target.write_bytes(archive.read(name))
    assert digest(path) == '9d809056e40e3d004b1ab9275081ee8ddb9d1874999369189d0ba172e7c3131c', 'Reference binary changed'
    version = run([path, '--version'])
    assert 'libopenmpt 0.8.9+r25651' in version, version
    return path


def envelope_enable_probe(helper_root):
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='envelope-enable-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'envelope_enable_probe', '--', stage])
        filter_player = filter_reference()
        start_filter_pcm = {}
        full_filter_pcm = {}
        delay_pcm = {}
        for mode in ('pan', 'volume', 'pitch', 'pitch_down', 'pitch_control', 'volume_sustain', 'volume_pause', 'pan_pause', 'pitch_pause', 'filter_delay', 'filter_delay_zero', 'filter_delay_sample', 'filter_delay_volume', 'filter_delay_cut', 'filter_delay_fine', 'filter_delay_memory', 'filter_delay_memory_control', 'filter_delay_memory_fine', 'filter_delay_memory_fine_control', 'filter_delay_memory_row', 'filter_delay_memory_row_control', 'filter_delay_memory_shared', 'filter_delay_memory_shared_control', 'filter_delay_repeat', 'filter_delay_repeat_control', 'filter_delay_outside', 'filter_delay_env', 'filter_delay_env_control', 'filter_full_hold', 'filter_full_control', 'filter_resonance', 'filter_resonance_open', 'filter_cutoff', 'filter_defaults', 'filter_env', 'filter_env_pause', 'filter_env_start_disabled', 'filter_env_midpoint'):
            module, output = stage / f'{mode}.it', stage / 'audio.wav'
            capture_case(native.capture, ROOT, renderer, stage, module, 0.7, output)
            cutoff_levels = {}
            for player in ('reference', 'native') if mode.startswith('filter_') else ('native', 'reference'):
                if player == 'native' and mode.startswith('filter_'):
                    capture_case(native.capture, ROOT, renderer, stage, module, 0.7, output)
                if player == 'reference' and mode.startswith('filter_'):
                    run([filter_player, '--batch', '--quiet', '--no-float', '--dither', '0',
                         '--samplerate', '44100', '--channels', '2', '--gain', '0',
                         '--stereo', '100', '--filter', '8', '--ramping', '-1',
                         '--repeat', '0', '--subsong', '-1', '--end-time', '0.7',
                         '--force', '--output', output, '--', module])
                elif player == 'reference':
                    run(['ffmpeg', '-v', 'error', '-y', '-f', 'libopenmpt', '-i', module,
                         '-t', '0.7', '-ar', '44100', '-ac', '2', output])
                with wave.open(str(output), 'rb') as wav:
                    pcm = array.array('h', wav.readframes(wav.getnframes()))
                if sys.byteorder != 'little':
                    pcm.byteswap()
                peaks = []
                for row in range(3):
                    a, b = int((row*.12+.04)*44100)*2, int((row*.12+.09)*44100)*2
                    peaks.append([max(map(abs, pcm[a+side:b:2])) for side in (0, 1)])
                if mode.startswith('filter_delay_memory'):
                    if not mode.endswith('control'):
                        delay_pcm[(player,'memory')] = pcm
                    else:
                        assert pcm == delay_pcm[(player,'memory')], (player,'S00 did not recall SD3')
                        print('S00 recall',mode,player,'PASS')
                    continue
                if mode.startswith('filter_delay_repeat'):
                    if mode == 'filter_delay_repeat':
                        delay_pcm[(player,'repeat')] = pcm
                    else:
                        a = delay_pcm[(player,'repeat')][:int(.22*44100)*2]
                        b = pcm[:int(.22*44100)*2]
                        repeat_peak = max(map(abs,a[int(.145*44100)*2:int(.17*44100)*2]))
                        print('Delayed row repetition',player,repeat_peak)
                        assert repeat_peak > 100 and a == b, (player,'SEx delayed note differs from expanded rows')
                    continue
                if mode == 'filter_delay_fine':
                    early = max(map(abs, pcm[:int(.115*44100)*2]))
                    late = max(map(abs, pcm[int(.125*44100)*2:int(.15*44100)*2]))
                    print('SD6 plus S62 boundary', player, early, late)
                    assert early == 0 and late > 100, (player, 'fine-delay extended note timing',early,late)
                    continue
                if mode in ('filter_delay_volume', 'filter_delay_cut'):
                    early = max(map(abs, pcm[int(.13*44100)*2:int(.17*44100)*2]))
                    late = max(map(abs, pcm[int(.2*44100)*2:int(.3*44100)*2]))
                    print('Delayed cell', mode, player, early, late)
                    assert early > 100 and late == 0, (player, 'volume-only SD3 timing', early, late)
                    continue
                if mode.startswith('filter_delay_env'):
                    if mode == 'filter_delay_env':
                        delay_pcm[player + '_env'] = pcm
                    else:
                        delayed = delay_pcm[player + '_env']
                        # Compare envelope onset with a no-delay control shifted one tick.
                        a = delayed[882*2:1764*2]
                        b = pcm[:882*2]
                        print('Delayed filter envelope onset', player, max(map(abs,a)), max(map(abs,b)))
                        assert a == b, (player, 'delayed envelope onset differs from shifted control')
                    continue
                if mode.startswith('filter_delay'):
                    early = max(map(abs, pcm[:int(.019*44100)*2]))
                    late = max(map(abs, pcm[int(.025*44100)*2:int(.1*44100)*2]))
                    print('Delayed filtered onset', mode, player, early, late)
                    assert early == 0, (mode, player, 'note sounded before delayed tick', early)
                    assert (late > 100) == (mode != 'filter_delay_outside'), (mode, player, late)
                    if mode == 'filter_delay':
                        delay_pcm[player] = pcm
                    elif mode == 'filter_delay_zero':
                        assert pcm == delay_pcm[player], (player, 'SD0 differs from SD1')
                    if mode == 'filter_delay_outside':
                        assert max(map(abs, pcm[:int(.35*44100)*2])) == 0, (mode, player, 'out-of-row delay leaked')
                        assert max(map(abs, pcm[int(.38*44100)*2:int(.45*44100)*2])) > 100, (mode, player, 'later control note missing')
                    continue
                if mode in ('filter_full_hold','filter_full_control'):
                    levels = [sum(x*x for x in pcm[int((r*.12+.05)*44100)*2:int((r*.12+.09)*44100)*2:2])**.5 for r in range(3)]
                    ratios=[x/levels[0] for x in levels]
                    print('Full-cutoff transition',mode,player,ratios)
                    assert abs(ratios[1]-1)<.03 and ratios[2]>4, (mode,player,ratios)
                    if mode == 'filter_full_hold':
                        full_filter_pcm[player] = pcm
                    else:
                        assert pcm == full_filter_pcm[player], (player, 'no-note Z7F changed retained filter output')
                    continue
                if mode.startswith('filter_resonance'):
                    levels = [sum(x*x for x in pcm[int((r*.12+.05)*44100)*2:int((r*.12+.09)*44100)*2:2])**.5 for r in range(3)]
                    cutoff_levels[player] = [level/levels[0] for level in levels]
                    print('Resonance response',player,cutoff_levels[player])
                    if player == 'native':
                        assert all(abs(a-b)<.03 for a,b in zip(cutoff_levels['native'],cutoff_levels['reference'])), cutoff_levels
                    continue
                if mode in ('filter_env_start_disabled', 'filter_env_midpoint'):
                    if mode == 'filter_env_start_disabled':
                        start_filter_pcm[player] = pcm
                    else:
                        assert pcm == start_filter_pcm[player], (mode, player, 'S7B-at-start differs from midpoint control')
                    continue
                if mode in ('filter_cutoff', 'filter_defaults', 'filter_env'):
                    # Default Zxx cutoff sweep: spectral attenuation, not PCM identity.
                    levels = [sum(x*x for x in pcm[int((r*.12+.05)*44100)*2:int((r*.12+.09)*44100)*2:2])**.5 for r in range(3)]
                    assert levels[0] < levels[1]*.5 and levels[1] > levels[2]*.5, (mode, player, levels)
                    cutoff_levels[player] = [level/levels[2] for level in levels]
                    print('Filter cutoff attenuation', player, cutoff_levels[player])
                    continue
                if mode.endswith('_pause'):
                    parts = [pcm[int((row*.12+.04)*44100)*2:int((row*.12+.09)*44100)*2] for row in range(3)]
                    levels = [[max(map(abs, part[side::2])) for side in (0,1)] for part in parts]
                    assert levels[0] == levels[1] == levels[2], (mode, player, levels)
                    if mode == 'pitch_pause':
                        rates = []
                        for part in parts:
                            side = part[::2];edges = [i for i,(a,b) in enumerate(zip(side,side[1:])) if (a<0)!=(b<0)]
                            rates.append((len(edges)-1)/(edges[-1]-edges[0]))
                        assert max(rates)/min(rates) < 1.02, (mode,player,rates)
                    continue
                if mode == 'volume_sustain':
                    gains = []
                    for tick in range(21):
                        a, b = int((tick*.02+.012)*44100)*2, int((tick*.02+.018)*44100)*2
                        gains.append(max(map(abs, pcm[a:b:2])))
                    # Held IT sustain visits both ends repeatedly, rather than
                    # freezing at its start; key-off resumes through normal loop end.
                    for start in (4, 7, 10):
                        low, middle, high = gains[start:start+3]
                        assert low < middle < high and high > low*1.4, (player, gains)
                    assert gains[12] < gains[13] < gains[14], (player, gains)
                    assert abs(gains[15]/gains[14]-1) < .1, (player, gains)
                    assert gains[16] < gains[15]*.7 and gains[18] > gains[16]*1.2, (player, gains)
                    continue
                if mode.startswith('pitch'):
                    # Constant endpoint windows: pitch rises then returns to base.
                    crossings = []
                    for start, end in ((.01, .019), (.121, .139), (.26, .30)):
                        side = pcm[int(start*44100)*2:int(end*44100)*2:2]
                        edges = [i for i, (a, b) in enumerate(zip(side, side[1:])) if (a < 0) != (b < 0)]
                        assert len(edges) >= 2, (mode, player, 'insufficient pitch edges')
                        crossings.append((len(edges)-1)*44100/(edges[-1]-edges[0]))
                    expected_ratio = .5 if mode == 'pitch_down' else 2.0
                    assert abs(crossings[1]/crossings[2] - expected_ratio) < .22, (mode, player, crossings)
                    assert abs(crossings[0]/crossings[2] - 1) < .25, (mode, player, crossings)
                    continue
                assert peaks[0][0] > 100 and peaks[0][0] == peaks[0][1], (mode, player, peaks)
                assert peaks[2] == peaks[0], (mode, player, peaks)
                if mode == 'pan':
                    assert peaks[1][0] == 0 and peaks[1][1] > 100, (mode, player, peaks)
                else:
                    assert abs(peaks[1][0]/peaks[0][0] - .5) < .01, (mode, player, peaks)
                    assert peaks[1][0] == peaks[1][1], (mode, player, peaks)
            if mode in ('filter_cutoff', 'filter_defaults', 'filter_env'):
                assert abs(cutoff_levels['native'][0] - cutoff_levels['reference'][0]) < .015, cutoff_levels
                assert abs(cutoff_levels['native'][1] - cutoff_levels['reference'][1]) < .15, cutoff_levels
        print('PASS: authored-off S78/S7A enable/disable and IT pitch rise/fall/return plus S7B/S7C, authored-on pause/resume and sustain-range/release loops plus bounded SD0/SD1 onset and out-of-range silence, Z7F note/no-note transitions, normal-range resonance, Zxx/filter-envelope sweep, pause and S7B-at-start match fresh packed/reference playback; termination/restart verified')


def nna_switch_probe(helper_root, override=False, fade=False):
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='nna-switch-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'nna_switch_probe', '--', stage] + (['--override'] if override else []) + (['--fade'] if fade else []))
        for mode in ('continue', 'cut'):
            module, output = stage / f'{mode}.it', stage / 'audio.wav'
            capture_case(native.capture, ROOT, renderer, stage, module, .7, output)
            for player in ('native', 'reference'):
                if player == 'reference':
                    run(['ffmpeg', '-v', 'error', '-y', '-f', 'libopenmpt', '-i', module, '-t', '0.7', '-ar', '44100', '-ac', '2', output])
                with wave.open(str(output), 'rb') as wav:
                    pcm = array.array('h', wav.readframes(wav.getnframes()))
                if sys.byteorder != 'little':
                    pcm.byteswap()
                for row in (0, 1):
                    a, b = int((row*.12+.04)*44100)*2, int((row*.12+.09)*44100)*2
                    left, right = [max(map(abs, pcm[a+side:b:2])) for side in (0, 1)]
                    assert (left > 100) == (row == 0 or mode == 'continue'), (mode, player, row, left, right)
                    assert (right > 100) == (row == 1), (mode, player, row, left, right)
        print(f'PASS: NNA {"zero-rate NoteFade" if fade else "S73/S74 overrides" if override else "instrument defaults"} preserve old-voice action in both switch directions; fresh packed/reference stereo, termination/restart verified')


def dct_parent_probe(helper_root, instruments=False, sample=False, dedup=False, past=False):
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='dct-parent-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        module, output = stage / 'parent.it', stage / 'audio.wav'
        run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'dct_parent_probe', '--', module] + (['--instruments'] if instruments else []) + (['--sample'] if sample else []) + (['--dedup'] if dedup else []) + (['--past'] if past else []))
        capture_case(native.capture, ROOT, renderer, stage, module, .8, output)
        for player in ('native', 'reference'):
            if player == 'reference':
                run(['ffmpeg', '-v', 'error', '-y', '-f', 'libopenmpt', '-i', module, '-t', '0.8', '-ar', '44100', '-ac', '2', output])
            with wave.open(str(output), 'rb') as wav:
                pcm = array.array('h', wav.readframes(wav.getnframes()))
            if sys.byteorder != 'little':
                pcm.byteswap()
            a, b = int(.28*44100)*2, int(.33*44100)*2
            peaks = [max(map(abs, pcm[a+side:b:2])) for side in (0, 1)]
            assert min(peaks) > 100, (player, peaks)
            if past:
                a, b = int(.41*44100)*2, int(.45*44100)*2
                left, right = [max(map(abs, pcm[a+side:b:2])) for side in (0, 1)]
                assert left > 100 and right < 30, (player, left, right)
        print(f'PASS: DCT {"sample" if sample else "note"} preserves other-{"source-sample" if dedup else "instrument" if instruments else "parent"} background voice; fresh packed/reference stereo, termination/restart verified')


def note_fade_probe(helper_root, empty=False, cut=False, porta=False, compat=False, terminal=False, held=False, off=False, constant=False):
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='note-fade-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        module, output = stage / 'fade.it', stage / 'audio.wav'
        run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'note_fade_probe', '--', module] + (['--empty'] if empty else []) + (['--cut'] if cut else []) + (['--porta'] if porta else []) + (['--compat'] if compat else []) + (['--terminal-envelope'] if terminal else []) + (['--held'] if held else []) + (['--special-off'] if off else []) + (['--constant'] if constant else []))
        if not terminal:
            capture_case(native.capture, ROOT, renderer, stage, module, 2.1, output)
        for player in (('reference', 'native') if terminal else ('native', 'reference')):
            if player == 'reference':
                if terminal:
                    run([filter_reference(), '--batch', '--quiet', '--no-float', '--dither', '0',
                         '--samplerate', '44100', '--channels', '2', '--gain', '0', '--stereo', '100',
                         '--filter', '8', '--ramping', '-1', '--repeat', '0', '--subsong', '-1',
                         '--end-time', '2.1', '--force', '--output', output, '--', module])
                else:
                    run(['ffmpeg', '-v', 'error', '-y', '-f', 'libopenmpt', '-i', module, '-t', '2.1', '-ar', '44100', '-ac', '2', output])
            elif terminal:
                capture_case(native.capture, ROOT, renderer, stage, module, 2.1, output)
            with wave.open(str(output), 'rb') as wav:
                pcm = array.array('h', wav.readframes(wav.getnframes()))
            if sys.byteorder != 'little':
                pcm.byteswap()
            peaks = [max(map(abs, pcm[int(t*44100)*2:int((t+.03)*44100)*2])) for t in (.06, .5, 1.0, 1.6)]
            if terminal:
                if constant:
                    assert abs(peaks[0] - 375) <= 1, (player, "steady DC amplitude", peaks[0])
                times = (.06, .19, .235, .275, .5, 1.0)
                levels = [max(map(abs, pcm[int(t*44100)*2:int((t+.01)*44100)*2])) for t in times]
                ratios = [level / levels[0] for level in levels]
                print('Terminal envelope', player, 'held', held, 'off', off, ratios)
                assert all(abs(a-b) < .025 for a,b in zip(ratios,
                    [1, 1, 1, .984, .812, .422] if held or off else [1, .945, .914, .883, .711, .32])), (player, ratios)
            elif cut:
                before = pcm[int(.125*44100)*2:int(.135*44100)*2]
                onset = pcm[int(.145*44100)*2:int(.155*44100)*2]
                after = pcm[int(.18*44100)*2:int(.20*44100)*2]
                # SC0 cuts at tick one (.14s); the official player's output ramp
                # decays beyond .16s. Check onset AND settled silence, not its tail.
                before_peak = max(map(abs, before))
                assert before_peak > 100, player
                assert max(map(abs, onset)) < before_peak * .5, (player, 'late cut')
                assert max(map(abs, after)) < 10, (player, 'cut did not settle')
            elif empty:
                crossings = []
                for t in (.06, .5, 1.0, 1.6):
                    window = pcm[int(t*44100)*2:int((t+.03)*44100)*2:2]
                    crossings.append(sum((a < 0) != (b < 0) for a, b in zip(window, window[1:])))
                assert max(peaks) - min(peaks) < 5, (player, peaks)
                assert max(crossings) - min(crossings) <= 2, (player, crossings)
            else:
                assert peaks[0] > peaks[1] > peaks[2] > peaks[3], (player, peaks)
                assert peaks[3] < peaks[0] * .05, (player, peaks)
            print('Direct fade', player, peaks)
        print(f'PASS: {"terminal envelope holds before endpoint then fades; explicit fade starts early" if terminal else "SC0 cuts at tick one" if cut else "empty slot preserves gain/pitch" if empty else "direct pattern NoteFade gain decay"}, fresh packed/reference, termination/restart')


def sample_special_probe(helper_root):
    """Original distinct-waveform samples expose unintended swaps on IT Off/Fade."""
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    reference = filter_reference()
    with tempfile.TemporaryDirectory(prefix='sample-special-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        for off in (False, True):
            module, output = stage / 'special.it', stage / 'audio.wav'
            run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'note_fade_probe', '--', module,
                 '--invalid-sample', '--valid-swap', '--special-selection'] + (['--special-off'] if off else []))
            for player in ('reference', 'native'):
                if player == 'reference':
                    run([reference, '--batch', '--quiet', '--no-float', '--dither', '0',
                         '--samplerate', '44100', '--channels', '2', '--gain', '0',
                         '--stereo', '100', '--filter', '8', '--ramping', '-1',
                         '--repeat', '0', '--subsong', '-1', '--end-time', '2.1',
                         '--force', '--output', output, '--', module])
                else:
                    capture_case(native.capture, ROOT, renderer, stage, module, 2.1, output)
                with wave.open(str(output), 'rb') as wav:
                    pcm = array.array('h', wav.readframes(wav.getnframes()))
                if sys.byteorder != 'little': pcm.byteswap()
                crossings = []
                for t in (.05, .18):
                    window = pcm[int(t*44100)*2:int((t+.03)*44100)*2:2]
                    assert max(map(abs, window)) > 100, (player, off, 'unexpected silence')
                    crossings.append(sum((a < 0) != (b < 0) for a,b in zip(window, window[1:])))
                print('Sample special', player, 'off', off, crossings)
                assert abs(crossings[1] - crossings[0]) <= 1, (player, off, crossings)
        print('PASS: IT sample-mode Off/Fade preserve the active waveform, fresh source/cart/reference')


def sample_restart_probe(helper_root):
    """Original one-shot: same sample number after end equals explicit retrigger."""
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    reference = filter_reference()
    with tempfile.TemporaryDirectory(prefix='sample-restart-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        results = {}
        for explicit in (False, True):
            module = stage / ('explicit.it' if explicit else 'implicit.it')
            run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'note_fade_probe',
                 '--', module, '--sample-restart'] + (['--explicit'] if explicit else []))
            for player in ('native', 'reference'):
                output = stage / f'{player}-{explicit}.wav'
                if player == 'native':
                    capture_case(native.capture, ROOT, renderer, stage, module, 2.1, output)
                else:
                    run([reference, '--batch', '--quiet', '--no-float', '--dither', '0',
                         '--samplerate', '44100', '--channels', '2', '--gain', '0',
                         '--stereo', '100', '--filter', '8', '--ramping', '-1',
                         '--repeat', '0', '--subsong', '-1', '--end-time', '2.1',
                         '--force', '--output', output, '--', module])
                with wave.open(str(output), 'rb') as wav:
                    pcm = array.array('h', wav.readframes(wav.getnframes()))
                if sys.byteorder != 'little':
                    pcm.byteswap()
                # Compare within each player, not reference/native PCM identity.
                results[player, explicit] = pcm[:int(.3 * 44100) * 2]
                assert max(map(abs, pcm[int(.12 * 44100)*2:int(.14 * 44100)*2])) > 100, (player, explicit)
        for player in ('native', 'reference'):
            assert results[player, False] == results[player, True], player
        print('PASS: stopped IT sample instrument-only restart equals explicit note in fresh native/cart and official reference')


def invalid_sample_probe(helper_root, valid=False, instrument_mode=False, selection_delay=None, empty_selection=False, selection_porta=True, selection_volume=False, special_selection=False, special_volume=False, special_off=False, old_effects=False, special_pan=False, special_ramp=False):
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    reference = filter_reference()
    with tempfile.TemporaryDirectory(prefix='invalid-sample-') as temporary:
        stage = Path(temporary)
        renderer = stage / ('renderer.exe' if os.name == 'nt' else 'renderer')
        compile_compat_renderer(native, renderer)
        for compat in (False, True):
            for pitched in (False, True):
                module = stage / 'invalid.it'
                run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'note_fade_probe',
                     '--', module, '--invalid-sample', '--recover']
                    + (['--porta'] if selection_porta else [])
                    + ([f'--selection-delay={selection_delay}'] if selection_delay is not None else [])
                    + (['--empty-selection'] if empty_selection else [])
                    + (['--selection-volume'] if selection_volume else [])
                    + (['--special-selection'] if special_selection else [])
                    + (['--special-volume'] if special_volume else [])
                    + (['--special-off'] if special_off else [])
                    + (['--old-effects'] if old_effects else [])
                    + (['--special-pan'] if special_pan else [])
                    + (['--special-ramp'] if special_ramp else [])
                    + (['--compat'] if compat else []) + (['--explicit'] if pitched else [])
                    + (['--valid-swap'] if valid else []) + (['--instrument-mode', '--memory-pitch'] if instrument_mode else []))
                for player in ('reference', 'native'):
                    output = stage / f'{player}.wav'
                    if player == 'native':
                        capture_case(native.capture, ROOT, renderer, stage, module, 2.1, output)
                    else:
                        run([reference, '--batch', '--quiet', '--no-float', '--dither', '0',
                             '--samplerate', '44100', '--channels', '2', '--gain', '0',
                             '--stereo', '100', '--filter', '8', '--ramping', '-1',
                             '--repeat', '0', '--subsong', '-1', '--end-time', '2.1',
                             '--force', '--output', output, '--', module])
                    with wave.open(str(output), 'rb') as wav:
                        pcm = array.array('h', wav.readframes(wav.getnframes()))
                    if sys.byteorder != 'little':
                        pcm.byteswap()
                    peak = max(map(abs, pcm[int(.2*44100)*2:int(.23*44100)*2]))
                    assert (peak > (10 if special_selection or old_effects or selection_volume else 100)) == (compat or valid or instrument_mode), (player, compat, pitched, peak)
                    if special_selection:
                        early = max(map(abs, pcm[int(.05*44100)*2:int(.08*44100)*2]))
                        expected_gain = (1.0 if special_off else .936) * (.25 if special_volume else 1.0)
                        if old_effects:
                            expected_gain *= .25
                        if special_ramp:
                            expected_gain = .5625
                            print("Same-sample envelope reset", player, peak/early)
                        if special_pan:
                            left = max(map(abs, pcm[int(.2*44100)*2:int(.23*44100)*2:2]))
                            right = max(map(abs, pcm[int(.2*44100)*2+1:int(.23*44100)*2:2]))
                            assert left < 2 and right > 10, (player, left, right, "selected sample pan")
                            expected_gain *= 2
                        if special_ramp:
                            # A restarted ramp is near its midpoint here; retaining the
                            # old clock would already be near the high endpoint.
                            assert .48 < peak/early < .59, (player, "envelope restart", peak/early)
                        else:
                            assert abs(peak/early - expected_gain) < .02, (player, compat, pitched, "special-note gain", peak/early)
                    if selection_volume:
                        early = max(map(abs, pcm[int(.05*44100)*2:int(.08*44100)*2]))
                        expected_gain = 1.0 if empty_selection and pitched else .25
                        assert abs(peak/early - expected_gain) < .02, (player, compat, pitched, 'selection volume', peak/early)
                    remembered = max(map(abs, pcm[int(.28*44100)*2:int(.31*44100)*2]))
                    recovered = max(map(abs, pcm[int(.4*44100)*2:int(.43*44100)*2]))
                    assert (remembered > (10 if special_selection or old_effects or selection_volume else 100)) == (compat or valid or instrument_mode), ('sample memory', player, compat, pitched, remembered)
                    if instrument_mode:
                        crossings = []
                        for t in (.05, .18, .3, .42):
                            window = pcm[int(t*44100)*2 + int(special_pan):int((t+.03)*44100)*2:2]
                            assert max(map(abs, window)) > (10 if special_selection or old_effects or selection_volume else 100), (player, 'invalid instrument silence')
                            crossings.append(sum((a < 0) != (b < 0) for a,b in zip(window, window[1:])))
                        expected = [8, 8, 16 if selection_delay is not None and selection_delay >= 6 else 8, 8]
                        assert all(abs(a-b) <= 1 for a,b in zip(crossings, expected)), (player, compat, pitched, selection_delay, crossings)
                    elif valid:
                        crossings = []
                        for t in (.06, .2, .28, .4):
                            window = pcm[int(t*44100)*2:int((t+.03)*44100)*2:2]
                            crossings.append(sum((a < 0) != (b < 0) for a, b in zip(window, window[1:])))
                        # Different source sample doubles frequency; porta may still be sliding.
                        expected = [8, (9 if pitched else 8) if compat else (19 if pitched else 16), 8 if compat else 16, 8]
                        assert all(abs(a-b) <= 1 for a, b in zip(crossings, expected)), (player, compat, pitched, crossings)
                    assert recovered > 100, ('valid sample recovery', player, compat, pitched, recovered)
        print(f'PASS: invalid instrument SD{selection_delay:X} selection memory/recovery; both compatibility modes and pitched/command-only' if selection_delay is not None else 'PASS: valid sample Gxx switching/pitch/memory/recovery' if valid else 'PASS: invalid sample Gxx activity/memory/recovery; both compatibility modes and pitched/command-only')


def historical_loop_probe(helper_root):
    """Brute VII authors B01 at pattern32/row63; this is not a finite song."""
    sys.path.insert(0, str(helper_root / 'packs/tracker_starter_v1'))
    import native
    module = CACHE / 'historical/Brute VII 2.it'
    assert digest(module) == '6466d54bb2f873170f7ef6f762a866a7221f531724393b1fe336ec5998a77d74'
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    with tempfile.TemporaryDirectory(prefix='authored-loop-') as temporary:
        stage = Path(temporary)
        renderer, actual, reference = stage/'renderer.exe', stage/'native.wav', stage/'reference.wav'
        compile_compat_renderer(native, renderer)
        try:
            capture_case(native.capture, ROOT, renderer, stage, module, 174, actual)
        except (RuntimeError, AssertionError) as exc:
            detail = str(exc)
            assert detail.startswith('Module did not terminate at authored duration:'), detail
            assert 'order=1 row=2 wraps=1 playing=true' in detail, detail
        else:
            raise AssertionError('Authored B01 loop unexpectedly terminated')
        run([filter_reference(), '--batch', '--quiet', '--no-float', '--dither', '0',
             '--samplerate', '44100', '--channels', '2', '--gain', '0', '--stereo', '100',
             '--filter', '8', '--ramping', '-1', '--repeat', '1', '--subsong', '-1',
             '--end-time', '174', '--force', '--output', reference, '--', module])
        for player, output in [('native', actual), ('reference', reference)]:
            with wave.open(str(output), 'rb') as wav:
                wav.setpos(int(173.9*44100))
                pcm = array.array('h', wav.readframes(int(.05*44100)))
            assert len(pcm) == int(.05*44100)*2, player
            assert max(map(abs, pcm)) > 100, (player, 'silent after authored loop')
        print('PASS: historical B01 returns to order1 and continues after first traversal; official reference repeat enabled; not whole-song fidelity acceptance')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--terminal-envelope-probe', action='store_true')
    parser.add_argument('--xm-pan-law-probe', action='store_true')
    parser.add_argument('--xm-invalid-selection-probe', action='store_true')
    parser.add_argument('--xm-valid-selection-probe', action='store_true')
    parser.add_argument('--sample-special-probe', action='store_true')
    parser.add_argument('--old-effects-off-probe', action='store_true')
    parser.add_argument('--historical-loop-probe', action='store_true')
    parser.add_argument('--fetch', action='store_true')
    parser.add_argument('--case', action='append', default=[])
    parser.add_argument('--sample-equivalence', action='store_true',
                        help='IT: compare original versus Nethercore-decoded PCM in both players')
    parser.add_argument('--helper-root', type=Path, default=ROOT.parent / 'speccade')
    parser.add_argument('--volume-porta-probe', action='store_true',
                        help='Fresh packed and reference volume/main G0-G9 equivalence check')
    parser.add_argument('--xm-global-volume-probe', action='store_true')
    parser.add_argument('--panning-defaults-probe', action='store_true')
    parser.add_argument('--envelope-enable-probe', action='store_true')
    parser.add_argument('--nna-switch-probe', action='store_true')
    parser.add_argument('--dct-parent-probe', action='store_true')
    parser.add_argument('--note-fade-probe', action='store_true')
    parser.add_argument('--sample-restart-probe', action='store_true')
    parser.add_argument('--invalid-sample-probe', action='store_true')
    parser.add_argument('--invalid-instrument-delay-probe', action='store_true')
    parser.add_argument('--empty-instrument-selection-probe', action='store_true')
    parser.add_argument('--special-instrument-selection-probe', action='store_true')
    args = parser.parse_args()
    if args.xm_pan_law_probe:
        xm_global_volume_probe(args.helper_root, pan_law=True)
    if args.xm_invalid_selection_probe:
        xm_global_volume_probe(args.helper_root, multi_sample=True, invalid_selection=True)
    if args.xm_valid_selection_probe:
        xm_global_volume_probe(args.helper_root, multi_sample=True, invalid_selection=True, selection=2, selection_envelope=True)
        xm_global_volume_probe(args.helper_root, multi_sample=True, invalid_selection=True, selection=2, selection_envelope=True, selection_k00=True)
    if args.sample_special_probe:
        sample_special_probe(args.helper_root)
    if args.old_effects_off_probe:
        for volume, pan, ramp in ((False, False, False), (True, True, False), (False, False, True)):
            invalid_sample_probe(args.helper_root, instrument_mode=True, empty_selection=True,
                                 selection_porta=False, special_selection=True, special_off=True, old_effects=True,
                                 special_volume=volume, special_pan=pan, special_ramp=ramp)
    if args.terminal_envelope_probe:
        note_fade_probe(args.helper_root, terminal=True, held=True, constant=True)
        for held, off in ((True, False), (False, True), (False, False)):
            note_fade_probe(args.helper_root, terminal=True, held=held, off=off)
    if args.historical_loop_probe:
        historical_loop_probe(args.helper_root)
    if args.special_instrument_selection_probe:
        for off in (False, True):
            for volume in (False, True):
                invalid_sample_probe(args.helper_root, instrument_mode=True, empty_selection=True, selection_porta=False, special_selection=True, special_volume=volume, special_off=off)
    if args.empty_instrument_selection_probe:
        for porta in (False, True):
            invalid_sample_probe(args.helper_root, instrument_mode=True, empty_selection=True, selection_porta=porta, selection_volume=True)
    if args.invalid_instrument_delay_probe:
        for delay in (0, 1, 5, 6, 15):
            invalid_sample_probe(args.helper_root, instrument_mode=True, selection_delay=delay)
    if args.invalid_sample_probe:
        invalid_sample_probe(args.helper_root)
        invalid_sample_probe(args.helper_root, valid=True)
        invalid_sample_probe(args.helper_root, instrument_mode=True)
    if args.sample_restart_probe:
        sample_restart_probe(args.helper_root)
    if args.note_fade_probe:
        note_fade_probe(args.helper_root)
        note_fade_probe(args.helper_root, empty=True)
        note_fade_probe(args.helper_root, cut=True)
        note_fade_probe(args.helper_root, empty=True, porta=True)
        note_fade_probe(args.helper_root, empty=True, porta=True, compat=True)
    if args.dct_parent_probe:
        dct_parent_probe(args.helper_root)
        dct_parent_probe(args.helper_root, instruments=True)
        dct_parent_probe(args.helper_root, instruments=True, sample=True)
        dct_parent_probe(args.helper_root, instruments=True, sample=True, dedup=True)
        dct_parent_probe(args.helper_root, instruments=True, past=True)
    if args.nna_switch_probe:
        nna_switch_probe(args.helper_root)
        nna_switch_probe(args.helper_root, override=True)
        nna_switch_probe(args.helper_root, fade=True)
    if args.envelope_enable_probe:
        envelope_enable_probe(args.helper_root)
    if args.panning_defaults_probe:
        panning_defaults_probe(args.helper_root)
    if args.xm_global_volume_probe:
        xm_global_volume_probe(args.helper_root)
        xm_global_volume_probe(args.helper_root, cut=True)
        xm_global_volume_probe(args.helper_root, cut=True, keyoff=True)
        xm_global_volume_probe(args.helper_root, cut=True, keyoff=True, noteoff=True)
        xm_global_volume_probe(args.helper_root, cut=True, keyoff=True, fade=True)
        xm_global_volume_probe(args.helper_root, delay=True)
        xm_global_volume_probe(args.helper_root, delay_porta=True)
        xm_global_volume_probe(args.helper_root, delay_porta=True, empty_delay=True)
        xm_global_volume_probe(args.helper_root, delay_porta=True, delay_slide=True)
        xm_global_volume_probe(args.helper_root, sample_volume=True)
        xm_global_volume_probe(args.helper_root, zero_slide=True)
        xm_global_volume_probe(args.helper_root, sample_pan=True)
        xm_global_volume_probe(args.helper_root, sample_pan=True, sample_porta=True)
        xm_global_volume_probe(args.helper_root, zero_pan=True)
        xm_global_volume_probe(args.helper_root, high_patterns=True)
        xm_global_volume_probe(args.helper_root, env_escape=True)
        xm_global_volume_probe(args.helper_root, pan_freeze=True)
        xm_global_volume_probe(args.helper_root, env_setpos=True)
        xm_global_volume_probe(args.helper_root, loop_bits=True)
        xm_global_volume_probe(args.helper_root, multi_sample=True)
        xm_global_volume_probe(args.helper_root, multi_sample=True, multi_empty=True)
    if args.volume_porta_probe:
        volume_porta_probe(args.helper_root)
    CACHE.mkdir(parents=True, exist_ok=True)
    if args.fetch:
        fetch()
    if not args.case:
        return
    sources = json.loads((CACHE/'sources.json').read_text())['files']
    if (CACHE/'historical.json').exists():
        sources += json.loads((CACHE/'historical.json').read_text())['files']
    modules = [entry for entry in sources if Path(entry['path']).name in args.case]
    assert {Path(entry['path']).name for entry in modules} == set(args.case), 'Unknown/unfetched case'
    sys.dont_write_bytecode = True
    sys.path.insert(0, str(args.helper_root / 'packs/tracker_starter_v1'))
    import native
    from native import capture
    run(['cargo', 'build', '-p', 'nether-cli', '-p', 'nethercore-zx'])
    renderer = CACHE / ('render_tracker.exe' if os.name == 'nt' else 'render_tracker')
    identity = compile_compat_renderer(native, renderer)
    ffmpeg = Path(shutil.which('ffmpeg'))
    report = {'status': 'diagnostic only; musical compatibility not inferred from aggregate metrics',
              'renderer': identity, 'ffmpeg_sha256': digest(ffmpeg),
              'ffmpeg_version': run([ffmpeg, '-version']).splitlines()[0], 'cases': {}}
    for entry in modules:
        module = CACHE / entry['path']
        assert digest(module) == entry['sha256']
        name = module.name
        reference, actual = CACHE/'reference.wav', CACHE/'actual.wav'
        if actual.exists():
            actual.unlink()
        result = {'source_sha256': entry['sha256']}
        try:
            info = json.loads(run(['ffprobe', '-v', 'error', '-f', 'libopenmpt', '-show_entries', 'format=duration', '-of', 'json', module]))
            duration = float(info['format']['duration'])
            assert 0 < duration < 599, 'Needs explicit long/infinite-song case'
            seconds = duration + .2
            run([ffmpeg, '-v', 'error', '-y', '-f', 'libopenmpt', '-sample_rate', '44100', '-layout', 'stereo', '-subsong', '0', '-i', module, '-t', str(seconds), '-ac', '2', '-ar', '44100', '-c:a', 'pcm_s16le', reference])
            result['reference_duration'] = duration
            try:
                result['capture'] = capture_case(capture, ROOT, renderer, CACHE, module, seconds, actual)
            except (RuntimeError, AssertionError) as exc:
                result['capture_failure'] = str(exc)
            if actual.exists():
                ref, got = features(reference), features(actual)
                pairs = list(zip(ref['windows_20ms'], got['windows_20ms']))
                audible = [(a, b) for a, b in pairs if a['rms'] > .001 or b['rms'] > .001]
                result['activity_disagreement_windows'] = sum((a['rms'] > .001) != (b['rms'] > .001) for a, b in audible)
                result['compared_audible_windows'] = len(audible)
                result['pan_mean_absolute_error'] = sum(abs(a['pan']-b['pan']) for a,b in audible) / max(1,len(audible))
                result['zero_crossing_mean_absolute_error'] = sum(abs(a['crossings']-b['crossings']) for a,b in audible) / max(1,len(audible))
                result['acceptance'] = 'UNREVIEWED: inspect named feature expectations; these metrics are not a pass'
                if args.sample_equivalence:
                    assert module.suffix.lower() == '.it', 'Sample-equivalence probe requires IT'
                    converted = CACHE / 'decoded-samples.it'
                    converted_ref, converted_actual = CACHE/'decoded-reference.wav', CACHE/'decoded-actual.wav'
                    try:
                        run(['cargo', 'run', '-q', '-p', 'nether-it', '--example', 'decode_to_pcm', '--', module, converted])
                        run([ffmpeg, '-v', 'error', '-y', '-f', 'libopenmpt', '-sample_rate', '44100', '-layout', 'stereo', '-subsong', '0', '-i', converted, '-t', str(seconds), '-ac', '2', '-ar', '44100', '-c:a', 'pcm_s16le', converted_ref])
                        capture_case(capture, ROOT, renderer, CACHE, converted, seconds, converted_actual)
                        equality = {}
                        for player, original, decoded in [('reference', reference, converted_ref), ('native', actual, converted_actual)]:
                            with wave.open(str(original), 'rb') as a, wave.open(str(decoded), 'rb') as b:
                                equality[player] = (a.getparams() == b.getparams()
                                    and a.readframes(a.getnframes()) == b.readframes(b.getnframes()))
                        result['sample_equivalence'] = equality
                        assert all(equality.values()), 'Compressed/decoded sample playback differs'
                    finally:
                        for path in (converted, converted_ref, converted_actual):
                            if path.exists(): path.unlink()

        except Exception as exc:
            result['error'] = str(exc)
        finally:
            for path in (reference, actual):
                if path.exists():
                    path.unlink()
        if 'error' in result or 'capture_failure' in result:
            result['acceptance'] = 'FAILED: packed playback did not complete successfully'
        report['cases'][name] = result
        (CACHE/'results.json').write_text(json.dumps(report, indent=2))
        print(name, json.dumps({k:v for k,v in result.items() if k != 'capture'}), flush=True)
    assert len(report['cases']) == len(set(args.case))
    if any('error' in case or 'capture_failure' in case for case in report['cases'].values()):
        raise SystemExit('Packed playback failed; see target/tracker-compatibility/results.json')

if __name__ == '__main__':
    main()
