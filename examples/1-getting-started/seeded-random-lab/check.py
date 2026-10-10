#!/usr/bin/env python
"""Verify named RNG state across repeated/different seeds; optionally render.

Build first, then pass a relocated ROM and matching player. Python 3.11+.
"""
import argparse
import json
import re
import shutil
import subprocess
import tomllib
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--player', required=True, type=Path)
parser.add_argument('--rom', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--render', action='store_true')
args = parser.parse_args()
player, rom = args.player.resolve(strict=True), args.rom.resolve(strict=True)
out = args.output.resolve()
out.mkdir(parents=True, exist_ok=True)
lesson = Path(__file__).resolve().parent
source = (lesson / 'witness.ncrs').read_text()
assert source.count('seed = 4919') == 1
rows = []
states = []
for index, seed in enumerate([4919, 4919, 4920]):
    stem = f'seed-{seed}-{index}'
    script, report = out / (stem + '.ncrs'), out / (stem + '.json')
    script.write_text(source.replace('seed = 4919', f'seed = {seed}'))
    report.unlink(missing_ok=True)
    command = [str(player), str(rom), '--headless', '--replay', str(script), '--report', str(report), '--timeout', '30']
    with (out / (stem + '.log')).open('w') as log:
        result = subprocess.run(command, cwd=out, stdout=log, stderr=subprocess.STDOUT, timeout=60)
    assert result.returncode == 0, stem
    data = json.loads(report.read_text())
    assert data['seed'] == seed and data['frames_executed'] == data['total_frames'] == 91
    assert data['summary']['assertions_passed'] == 10 and data['summary']['assertions_failed'] == 0
    assert not data.get('error') and len(data['snapshots']) == 10
    named = [{k: snap[k] for k in ['frame', 'input', 'pre', 'post']} for snap in data['snapshots']]
    for snap in named:
        for phase in ['pre', 'post']:
            values = {k: v['U32'] for k, v in snap[phase].items()}
            assert values['$points'] == 32 and values['$max_lane'] <= 3
            assert 0 <= values['$cursor'] <= 31 and values['$auto'] in [0, 1]
    states.append(named)
    rows.append({'seed': seed, 'argv': command, 'report': str(report)})
    (out / 'progress.json').write_text(json.dumps(rows, indent=2))
assert states[0] == states[1], 'Identical replay seed/inputs changed named guest state'
route_hashes = [[snap['post']['$route_hash']['U32'] for snap in state] for state in states]
assert route_hashes[0] != route_hashes[2], 'Different seed did not change the route'
assert route_hashes[0][0] != route_hashes[0][-1], 'B must advance the stream, not reset it'
captures = []
if args.render:
    log_path = out / 'native.log'
    with log_path.open('w') as log:
        result = subprocess.run([str(player), str(rom), '--replay', str(lesson / 'capture.ncrs'), '--scale', '1'], cwd=out, stdout=log, stderr=subprocess.STDOUT, timeout=180)
    text = re.sub(r'\x1b\[[0-9;]*m', '', log_path.read_text(errors='replace'))
    assert result.returncode == 0 and 'ERROR' not in text
    actual = {int(g): int(h) for g, h in re.findall(r'ROUTE generation=(\d+) points=32 hash=(\d+)', text)}
    expected = {snap['post']['$generation']['U32']: snap['post']['$route_hash']['U32'] for snap in states[0]}
    assert actual == expected, (actual, expected)
    paths = list(dict.fromkeys(re.findall(r'Screenshot saved:\s*(.+\.png)', text)))
    expected_count = sum(bool(f.get('screenshot')) for f in tomllib.loads((lesson / 'capture.ncrs').read_text())['frames'])
    assert len(paths) == expected_count
    for i, path in enumerate(paths):
        target = out / f'native-{i}.png'
        shutil.copyfile(path.strip().strip('"'), target)
        captures.append(str(target))
receipt = {'same_seed_equal': True, 'different_seed_changed_route': True,
           'named_route_hashes': route_hashes, 'runs': rows, 'native_checked': args.render,
           'captures': captures, 'human_review': 'unreviewed'}
(out / 'receipt.json').write_text(json.dumps(receipt, indent=2))
print('PASS: equal named state for repeated seed; changed bounded route for different seed; native checked:', args.render)
