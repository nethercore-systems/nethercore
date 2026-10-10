#!/usr/bin/env python
"""Build/relocate lessons; check replay negatives and optional native output.

Python 3.11+. --render also requires Pillow and a native display. This does not
establish human acceptance, online netplay, a committed tree or remote delivery.
"""
import argparse
import hashlib
import json
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--nether', required=True, type=Path)
parser.add_argument('--player', required=True, type=Path)
parser.add_argument('--output', required=True, type=Path)
parser.add_argument('--render', action='store_true')
parser.add_argument('--projects', nargs='+', default=[
    '1-getting-started/triangle', '1-getting-started/input-lab',
    '1-getting-started/fixed-clock-lab', '1-getting-started/seeded-random-lab'],
    help='Paths relative to examples/; default: the four foundation lessons')
args = parser.parse_args()
nether, player = args.nether.resolve(strict=True), args.player.resolve(strict=True)
lesson = Path(__file__).resolve().parent
root = lesson.parent.parent
out = args.output.resolve()
assert not out.is_relative_to(root), 'Use an output directory outside the source checkout'
out.mkdir(parents=True, exist_ok=True)
assert not any(out.iterdir()), 'Use an empty output directory; preserve earlier receipts'
projects = [(root / 'examples' / name).resolve(strict=True) for name in args.projects]
assert all(p.is_relative_to(root / 'examples') for p in projects), 'Use repository examples'
assert len({p.name for p in projects}) == len(projects), 'Duplicate lesson/output name'


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


paths = [Path(__file__).resolve(), root / 'examples/examples-common/Cargo.toml']
for directory in [root / 'include', root / 'examples/examples-common/src', *projects]:
    paths.extend(p for p in directory.rglob('*') if p.is_file() and 'target' not in p.parts
                 and (p.suffix in {'.rs', '.toml', '.ncrs', '.py', '.md'} or p.name == 'Cargo.lock'))
sources = {p.relative_to(root).as_posix(): sha(p) for p in sorted(set(paths))}
commands, results = [], []


def run(label, command, expected=0, timeout=180):
    log = out / (label + '.log')
    with log.open('w', encoding='utf-8') as stream:
        result = subprocess.run([str(x) for x in command], cwd=out, stdout=stream,
                                stderr=subprocess.STDOUT, timeout=timeout)
    commands.append({'argv': [str(x) for x in command], 'exit_code': result.returncode,
                     'log': log.name, 'log_sha256': sha(log)})
    (out / 'commands.json').write_text(json.dumps(commands, indent=2), encoding='utf-8')
    assert result.returncode == expected, f'{label}: exit {result.returncode}; see {log}'
    return re.sub(r'\x1b\[[0-9;]*m', '', log.read_text(errors='replace'))


for project in projects:
    name = project.name
    rom, report = out / (name + '.nczx'), out / (name + '.json')
    run(name + '-build', [nether, 'build', '-p', project, '-o', rom], timeout=300)
    witness = (project / 'witness.ncrs').read_text()
    script = tomllib.loads(witness)
    run(name + '-state', [player, rom, '--headless', '--replay', project / 'witness.ncrs',
                         '--report', report, '--timeout', '30'])
    data = json.loads(report.read_text())
    assertions = sum('assert' in f for f in script['frames'])
    assert assertions > 0 and not data.get('error') and data['summary']['status'] == 'PASSED'
    assert data['frames_executed'] == data['total_frames'] == max(f['f'] for f in script['frames']) + 1
    assert data['summary']['assertions_passed'] == assertions and data['summary']['assertions_failed'] == 0
    original = next(f['assert'] for f in script['frames'] if 'assert' in f)
    variable = re.match(r'\$[a-zA-Z0-9_]+', original)[0]
    condition = variable + ' == 999999'
    negative = out / (name + '-negative.ncrs')
    negative.write_text(witness.replace('assert = ' + json.dumps(original),
                                        'assert = ' + json.dumps(condition), 1), encoding='utf-8')
    bad_report = out / (name + '-negative.json')
    run(name + '-negative', [player, rom, '--headless', '--replay', negative,
                            '--report', bad_report, '--timeout', '30'], expected=1)
    bad = json.loads(bad_report.read_text())
    assert not bad.get('error') and bad['summary']['status'] == 'FAILED'
    failures = [a for a in bad['assertions'] if not a['passed']]
    assert bad['summary']['assertions_failed'] == len(failures) == 1
    assert failures[0]['condition'] == condition and failures[0]['actual'] != 999999
    media = []
    sync_frames = 0
    if args.render:
        if name in ('fixed-clock-lab', 'seeded-random-lab'):
            command = [sys.executable, project / 'check.py', '--player', player,
                       '--rom', rom, '--output', out / name]
            if name == 'seeded-random-lab':
                command.append('--render')
            run(name + '-native-check', command, timeout=240)
            media = sorted((out / name).rglob('native-*.png'))
            assert len(media) == (12 if name == 'fixed-clock-lab' else 6)
        else:
            log = run(name + '-native', [player, rom, '--replay', project / 'capture.ncrs', '--scale', '1'])
            assert 'ERROR' not in log and 'can only be called during init()' not in log
            captures = list(dict.fromkeys(re.findall(r'Screenshot saved:\s*(.+\.png)', log)))
            expected = sum(bool(f.get('screenshot')) for f in tomllib.loads((project / 'capture.ncrs').read_text())['frames'])
            assert len(captures) == expected
            for i, path in enumerate(captures):
                dest = out / f'{name}-native-{i}.png'
                shutil.copyfile(path.strip().strip('"'), dest)
                media.append(dest)
            run(name + '-pixels', [sys.executable, project / 'check_pixels.py', *media])
        # The same scripted controls also execute through forced rollback, not only idle input.
        sync = out / (name + '-sync.ncrs')
        text = '\n'.join(line for line in (project / 'capture.ncrs').read_text().splitlines()
                         if not line.startswith('screenshot ='))
        if max(f['f'] for f in tomllib.loads(text)['frames']) < 119:
            text += '\n\n[[frames]]\nf = 119\n'
        sync_frames = max(f['f'] for f in tomllib.loads(text)['frames']) + 1
        sync.write_text(text, encoding='utf-8')
        log = run(name + '-sync', [player, rom, '--sync-test', '--check-distance', '2',
                                  '--players', script['players'], '--replay', sync,
                                  '--exit-after-frames', str(sync_frames), '--scale', '1'])
        assert f'Frame limit reached after {sync_frames} advanced input frames' in log
        assert 'desync' not in log.lower() and 'ERROR' not in log
    results.append({'id': name, 'frames': data['frames_executed'], 'assertions': assertions,
                    'negative_rejected': True, 'rom_sha256': sha(rom),
                    'native_and_scripted_rollback': args.render,
                    'sync_input_frames': sync_frames,
                    'captures': [{'path': p.relative_to(out).as_posix(), 'sha256': sha(p)} for p in media]})
    assert all(sha(root / p) == digest for p, digest in sources.items()), 'Source changed during qualification'
    receipt = {'source_sha256': sources, 'player_sha256': sha(player), 'nether_sha256': sha(nether),
               'results': results, 'commands': commands, 'human_review': 'unreviewed'}
    (out / 'receipt.json').write_text(json.dumps(receipt, indent=2), encoding='utf-8')
    print(f'PASS {name}: {data["frames_executed"]} ticks, {assertions} assertions, negative rejected; native/sync={args.render}', flush=True)
assert len(results) == len(projects)
print(f'PASS all {len(projects)} selected lessons; committed-tree/remote/CI and human review are separate gates.')
