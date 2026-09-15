#!/usr/bin/env python
"""Check every advertised preview ROM with the packaged native player.

This is an init/update/import gate, not GPU, audio or controller acceptance.
The native subprocess deadline also covers instantiation and startup failures.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument("packet", type=Path)
args = parser.parse_args()
packet = args.packet.resolve(strict=True)
manifest = json.loads((packet / "manifest.json").read_text(encoding="utf-8-sig"))
suffix = ".exe" if os.name == "nt" else ""
for name in ("nether", "nethercore", "nethercore-zx"):
    if not (packet / "tools" / (name + suffix)).is_file():
        raise SystemExit(f"Missing required packaged tool: {name}{suffix}")
player = packet / "tools" / ("nethercore-zx" + suffix)
output = packet / "runtime-checks"
output.mkdir(exist_ok=True)
script = output / "load.ncrs"
script.write_text('console = "zx"\nseed = 4919\nplayers = 1\n[[frames]]\nf = 29\np1 = "idle"\n', encoding="utf-8")
results = []
for example in manifest["examples"]:
    identity = example["id"]
    if not isinstance(identity, str) or not identity or any(c not in "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_" for c in identity):
        raise SystemExit(f"Invalid example identity: {identity!r}")
    rom = packet / "roms" / identity / f"{identity}.nczx"
    report = output / f"{identity}.json"
    report.unlink(missing_ok=True)
    row = {"id": identity, "passed": False}
    if rom.is_file():
        row["cartridge_sha256"] = hashlib.sha256(rom.read_bytes()).hexdigest()
        with (output / f"{identity}.log").open("w", encoding="utf-8") as log:
            try:
                process = subprocess.run([str(player), str(rom), "--headless", "--replay", str(script), "--report", str(report), "--timeout", "10"], cwd=output, stdout=log, stderr=subprocess.STDOUT, timeout=20)
                row["exit_code"] = process.returncode
                if report.is_file():
                    data = json.loads(report.read_text(encoding="utf-8"))
                    row["frames_executed"] = data.get("frames_executed")
                    row["status"] = data.get("summary", {}).get("status")
                    row["passed"] = process.returncode == 0 and data.get("error") is None and data.get("total_frames") == 30 and row["frames_executed"] == 30 and row["status"] == "PASSED"
            except subprocess.TimeoutExpired:
                row["error"] = "Native process exceeded the 20-second deadline"
    else:
        row["error"] = "Advertised cartridge is missing"
    results.append(row)
    (output / "receipt.json").write_text(json.dumps({"expected": len(manifest["examples"]), "results": results}, indent=2), encoding="utf-8")
    if not row["passed"]:
        raise SystemExit(f"Runtime-load check failed for {identity}; see {output}")
print(f"Runtime-load checks passed: {len(results)} advertised ROM(s), 30 frames each")
