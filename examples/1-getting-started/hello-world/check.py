#!/usr/bin/env python
"""Build all three first cartridges and verify relocated guests, never skip a language.

Python 3.11+. Optional --render captures native pixels and checks rollback; the
headless state check alone does not certify presentation or online netplay.
"""
import argparse
import hashlib
import json
import re
import shutil
import subprocess
from pathlib import Path

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("--nether", required=True, type=Path)
parser.add_argument("--player", required=True, type=Path)
parser.add_argument("--output", required=True, type=Path)
parser.add_argument("--render", action="store_true")
args = parser.parse_args()
nether, player = args.nether.resolve(strict=True), args.player.resolve(strict=True)
output = args.output.resolve()
output.mkdir(parents=True, exist_ok=True)
lesson = Path(__file__).resolve().parent
replay = lesson / "movement.ncrs"
capture = lesson / "capture.ncrs"
negative = output / "wrong-expectation.ncrs"
negative.write_text(replay.read_text().replace('$square_y == 240', '$square_y == 241', 1), encoding="utf-8")


def sha(path):
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


commands = []


def run(cmd, log, expected=0, timeout=120):
    with (output / log).open("w", encoding="utf-8") as stream:
        result = subprocess.run([str(x) for x in cmd], cwd=output,
                                stdout=stream, stderr=subprocess.STDOUT, timeout=timeout)
    commands.append({"argv": [str(x) for x in cmd], "log": log, "exit_code": result.returncode})
    assert result.returncode == expected, f"Unexpected exit {result.returncode}: {output / log}"
    return re.sub(r"\x1b\[[0-9;]*m", "", (output / log).read_text(errors="replace"))


results = []
for name in ("hello-world", "hello-world-c", "hello-world-zig"):
    project = lesson.parent / name
    built = output / (name + ".built.nczx")
    rom = output / (name + ".nczx")
    run([nether, "build", "-p", project, "-o", built], name + "-build.log", timeout=240)
    shutil.copyfile(built, rom)
    report = output / (name + ".json")
    report.unlink(missing_ok=True)
    run([player, rom, "--headless", "--replay", replay, "--report", report, "--timeout", "30"], name + "-state.log", timeout=60)
    data = json.loads(report.read_text())
    assert data.get("error") is None and data["summary"]["status"] == "PASSED", data
    assert data["frames_executed"] == data["total_frames"] == 91, data
    assert data["summary"]["assertions_passed"] == 15 and data["summary"]["assertions_failed"] == 0, data
    assert len(data["snapshots"]) == 15 and all(x["post"] for x in data["snapshots"]), data
    bad_report = output / (name + "-negative.json")
    bad_report.unlink(missing_ok=True)
    run([player, rom, "--headless", "--replay", negative, "--report", bad_report, "--timeout", "30"], name + "-negative.log", expected=1, timeout=60)
    bad = json.loads(bad_report.read_text())
    assert bad.get("error") is None and bad["summary"]["status"] == "FAILED", bad
    assert bad["summary"]["assertions_failed"] == 1 and bad["assertions"][0]["actual"] == 240, bad
    media = []
    if args.render:
        log = run([player, rom, "--replay", capture, "--scale", "1"], name + "-render.log", timeout=180)
        assert "Replay script loaded: 91 frames, 5 screenshots" in log, name + "-render.log"
        assert "can only be called during init()" not in log and "ERROR" not in log, name + "-render.log"
        # Paths originate from the actual player; retain each capture in the receipt directory.
        paths = dict.fromkeys(re.findall(r"Screenshot saved:\s*(.+\.png)", log))
        for i, path in enumerate(paths):
            source = Path(path.strip().strip('"'))
            dest = output / f"{name}-frame-{i}.png"
            shutil.copyfile(source, dest)
            media.append({"path": dest.name, "sha256": sha(dest)})
        assert len(media) == 5, f"Expected five fresh native captures; found {len(media)}"
        log = run([player, rom, "--sync-test", "--check-distance", "2", "--players", "1", "--exit-after-frames", "120", "--scale", "1"], name + "-sync.log", timeout=180)
        assert "Frame limit reached after 120 advanced input frames" in log, name + "-sync.log"
        assert "desync" not in log.lower() and "ERROR" not in log, name + "-sync.log"
    results.append({"id": name, "rom_sha256": sha(rom), "assertions": 15,
                    "negative_rejected": True, "captures": media,
                    "rendered_and_sync": args.render})
    receipt = {"player_sha256": sha(player), "nether_sha256": sha(nether),
               "results": results, "commands": commands}
    (output / "receipt.json").write_text(json.dumps(receipt, indent=2) + "\n", encoding="utf-8")
    print(f"PASS {name}: 91 ticks, 15 state assertions, wrong expectation rejected", flush=True)
assert len(results) == 3
if args.render:
    reference = [frame["sha256"] for frame in results[0]["captures"]]
    assert reference[0] == reference[-1] and len(set(reference)) == 4, "Move, clamps and reset must affect pixels"
    assert all([frame["sha256"] for frame in row["captures"]] == reference for row in results), "Language outputs differ"
print("PASS: all three compiled, packed and ran after relocation. Human review remains separate.")
