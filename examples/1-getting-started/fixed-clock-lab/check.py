#!/usr/bin/env python
"""Compare zero-render replay with native one/four-draw runs by host tick.

Requires Python 3.11+, Pillow and a native display. Build the cartridge first.
"""
import argparse
import hashlib
import json
import re
import shutil
import subprocess
import tomllib
from pathlib import Path
from PIL import Image

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--player', required=True, type=Path)
parser.add_argument('--rom', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
args = parser.parse_args()
player, rom = args.player.resolve(strict=True), args.rom.resolve(strict=True)
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
lesson = Path(__file__).resolve().parent
witness = (lesson / 'witness.ncrs').read_text()
capture = (lesson / 'capture.ncrs').read_text()
assert witness.count('p1 = "b"') == capture.count('p1 = "b"') == 1
rows = []
endpoints = []
for draws in [1, 4]:
    folder = out / f'draws-{draws}'
    folder.mkdir(exist_ok=True)
    headless, native = witness, capture
    if draws == 1:
        headless = headless.replace('p1 = "b"', '# Keep one draw').replace('$copies == 4', '$copies == 1')
        native = native.replace('p1 = "b"', '# Keep one draw')
    control = lambda text: {f['f']: {k: v for k, v in f.items() if k.startswith('p') and k[1:].isdigit()} for f in tomllib.loads(text)['frames']}
    assert control(headless) == control(native), 'Headless/native input sequences differ'
    scripts = [folder / 'state.ncrs', folder / 'native.ncrs']
    for path, text in zip(scripts, [headless, native]):
        path.write_text(text)
    report = folder / 'state.json'
    report.unlink(missing_ok=True)
    with (folder / 'state.log').open('w') as log:
        result = subprocess.run([str(player), str(rom), '--headless', '--replay', str(scripts[0]), '--report', str(report), '--timeout', '30'], cwd=folder, stdout=log, stderr=subprocess.STDOUT, timeout=60)
    assert result.returncode == 0
    data = json.loads(report.read_text())
    assert data['frames_executed'] == data['total_frames'] == 181
    assert not data.get('error') and data['summary']['assertions_passed'] == 10 and data['summary']['assertions_failed'] == 0
    ticks = {}
    for snap in data['snapshots']:
        for phase in ['pre', 'post']:
            values = {k: next(iter(v.values())) for k, v in snap[phase].items()}
            ticks[values['$host_tick']] = (values['$sim_ticks'], values['$position'])
    with (folder / 'native.log').open('w') as log:
        result = subprocess.run([str(player), str(rom), '--replay', str(scripts[1]), '--scale', '1'], cwd=folder, stdout=log, stderr=subprocess.STDOUT, timeout=180)
    text = re.sub(r'\x1b\[[0-9;]*m', '', (folder / 'native.log').read_text(errors='replace'))
    assert result.returncode == 0 and 'ERROR' not in text
    native_ticks = {int(t): (int(s), int(p)) for t, s, p in re.findall(r'CLOCK tick=(\d+) sim=(\d+) position=(\d+)', text)}
    assert set(native_ticks) == {60, 120, 180}, native_ticks
    assert all(native_ticks[t] == ticks[t] for t in native_ticks), (native_ticks, ticks)
    paths = list(dict.fromkeys(re.findall(r'Screenshot saved:\s*(.+\.png)', text)))
    assert len(paths) == 6, paths
    captures = []
    for i, path in enumerate(paths):
        target = folder / f'native-{i}.png'
        shutil.copyfile(path.strip().strip('"'), target)
        captures.append(str(target))
    image = Image.open(captures[-1]).convert('RGB')
    assert image.size == (960, 540)
    marker = image.crop((64, 280, 896, 352))
    assert sum(pixel == (255, 232, 170) for pixel in marker.getdata()) > 1000, 'No rendered marker'
    endpoints.append(hashlib.sha256(marker.tobytes()).hexdigest())
    rows.append({'draws_per_callback': draws, 'zero_render_and_native_ticks': native_ticks,
                 'endpoint_marker_sha256': endpoints[-1], 'captures': captures})
    (out / 'progress.json').write_text(json.dumps(rows, indent=2))
assert rows[0]['zero_render_and_native_ticks'] == rows[1]['zero_render_and_native_ticks']
assert endpoints[0] == endpoints[1], 'Repeated draws changed endpoint pixels'
(out / 'receipt.json').write_text(json.dumps({'runs': rows, 'human_review': 'unreviewed'}, indent=2))
print('PASS: zero-render/native states agree at ticks 60/120/180; one/four draws give identical marker pixels')
