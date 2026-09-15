#!/usr/bin/env python
"""Bounded real-player netplay check; needs a desktop/GPU, not hosted services.

Use a development cartridge/save identity. Output must be a new directory so
failed receipts are not overwritten. No game/library source is modified.
"""
import argparse
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import time


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("player", type=Path)
    parser.add_argument("cartridge", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--frames", type=int, default=120)
    parser.add_argument("--timeout", type=float, default=45)
    parser.add_argument("--host-join", action="store_true")
    args = parser.parse_args()
    if args.frames <= 0 or args.timeout <= 0:
        parser.error("frames and timeout must be positive")
    player, cartridge = args.player.resolve(strict=True), args.cartridge.resolve(strict=True)
    args.output.mkdir(parents=True, exist_ok=False)
    sockets = [socket.socket(socket.AF_INET, socket.SOCK_DGRAM) for _ in range(2)]
    try:
        for sock in sockets:
            sock.bind(("127.0.0.1", 0))
        ports = [sock.getsockname()[1] for sock in sockets]
    finally:
        for sock in sockets:
            sock.close()
    children = []
    rows = []
    deadline = time.monotonic() + args.timeout
    try:
        for i in range(2):
            mode = (["--host", str(ports[0]), "--players", "2"] if i == 0 else
                    ["--join", f"127.0.0.1:{ports[0]}", "--bind", str(ports[1]), "--players", "2"])
            if not args.host_join:
                mode = ["--p2p", "--bind", str(ports[i]), "--peer", str(ports[1-i]), "--local-player", str(i)]
            command = [str(player), str(cartridge), "--scale", "1", "--exit-after-frames", str(args.frames), *mode]
            log = args.output / f"player-{i}.log"
            stream = log.open("w", encoding="utf-8")
            try:
                child = subprocess.Popen(command, cwd=args.output, stdout=stream, stderr=subprocess.STDOUT)
            except Exception:
                stream.close()
                raise
            children.append((child, stream, log, command))
        for child, _, _, _ in children:
            child.wait(timeout=max(.01, deadline-time.monotonic()))
    finally:
        for child, stream, log, command in children:
            forced = child.poll() is None
            if forced:
                child.kill()
            child.wait()
            stream.close()
            output = log.read_text(encoding="utf-8", errors="replace")
            rows.append({"command": command, "exit_code": child.returncode, "forced_stop": forced,
                         "completed_frames": f"Frame limit reached after {args.frames} advanced input frames" in output,
                         "log": str(log.resolve())})
        passed = len(rows) == 2 and all(r["exit_code"] == 0 and not r["forced_stop"] and r["completed_frames"] for r in rows)
        receipt = {"status": "PASS" if passed else "FAIL", "rows": rows,
                   "player_sha256": hashlib.sha256(player.read_bytes()).hexdigest(),
                   "cartridge_sha256": hashlib.sha256(cartridge.read_bytes()).hexdigest()}
        (args.output / "receipt.json").write_text(json.dumps(receipt, indent=2), encoding="utf-8")
    if not passed:
        raise SystemExit("Netplay check failed; inspect the retained player logs")
    print(f"Both players completed {args.frames} frames without forced termination")


if __name__ == "__main__":
    main()
