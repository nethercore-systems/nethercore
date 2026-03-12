from __future__ import annotations

import argparse
import json
import subprocess
import sys
import time
from datetime import datetime, timedelta
from pathlib import Path

import run_epu_loop_queue as queue_runner


REPO_ROOT = Path(__file__).resolve().parents[2]
DEFAULT_QUEUE = REPO_ROOT / "agent" / "queue" / "epu-loop.yaml"
DEFAULT_RUNNER = REPO_ROOT / "tools" / "tmp" / "run_epu_loop_queue.py"
DEFAULT_OVERNIGHT_DIR = REPO_ROOT / "agent" / "runs" / "overnight"


def summarize_queue(queue_path: Path) -> dict[str, int]:
    jobs = queue_runner.load_queue(queue_path)
    counts: dict[str, int] = {
        "queued": 0,
        "running": 0,
        "awaiting_review": 0,
        "blocked": 0,
        "done": 0,
        "runnable": 0,
    }
    job_index = {job.job_id: job for job in jobs}
    for job in jobs:
        counts[job.status] = counts.get(job.status, 0) + 1
        if job.status != "queued":
            continue
        if job.depends_on:
            dependency = job_index.get(job.depends_on)
            if dependency is None or dependency.status != "done":
                continue
        counts["runnable"] += 1
    return counts


def write_json(path: Path, payload: dict) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2) + "\n", encoding="utf-8")


def append_log(path: Path, line: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("a", encoding="utf-8") as handle:
        handle.write(line.rstrip() + "\n")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Supervise the EPU queue runner for long unattended sessions."
    )
    parser.add_argument("--queue", type=Path, default=DEFAULT_QUEUE)
    parser.add_argument("--runner", type=Path, default=DEFAULT_RUNNER)
    parser.add_argument("--cwd", type=Path, default=REPO_ROOT)
    parser.add_argument("--artifacts-dir", type=Path, default=DEFAULT_OVERNIGHT_DIR)
    parser.add_argument("--hours", type=float, default=24.0, help="Maximum supervisor duration in hours.")
    parser.add_argument("--poll-seconds", type=int, default=300, help="Sleep interval when the queue is idle.")
    parser.add_argument(
        "--idle-seconds",
        type=int,
        default=3600,
        help="Stop after this many idle seconds with no runnable jobs.",
    )
    parser.add_argument(
        "--max-jobs-per-pass",
        type=int,
        default=2,
        help="Maximum runnable jobs to drain per queue-runner pass.",
    )
    parser.add_argument(
        "--stop-file",
        type=Path,
        default=None,
        help="Optional file whose presence tells the supervisor to stop cleanly.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    queue_path = args.queue.resolve()
    runner_path = args.runner.resolve()
    cwd = args.cwd.resolve()
    started_at = datetime.now()
    deadline = started_at + timedelta(hours=max(args.hours, 0.0))
    artifacts_dir = args.artifacts_dir.resolve() / started_at.strftime("%Y%m%d-%H%M%S")
    artifacts_dir.mkdir(parents=True, exist_ok=False)
    heartbeat_path = artifacts_dir / "heartbeat.json"
    log_path = artifacts_dir / "supervisor.log"
    runner_output_path = artifacts_dir / "last-runner-output.txt"
    stop_file = args.stop_file.resolve() if args.stop_file else None
    idle_started_at: datetime | None = None
    passes = 0

    append_log(log_path, f"[start] {started_at.isoformat()} queue={queue_path}")

    while True:
        now = datetime.now()
        queue_counts = summarize_queue(queue_path)
        heartbeat = {
            "started_at": started_at.isoformat(),
            "now": now.isoformat(),
            "deadline": deadline.isoformat(),
            "passes": passes,
            "queue": queue_counts,
            "idle_started_at": idle_started_at.isoformat() if idle_started_at else None,
            "stop_file": str(stop_file) if stop_file else None,
            "runner": str(runner_path),
            "queue_path": str(queue_path),
        }
        write_json(heartbeat_path, heartbeat)

        if stop_file and stop_file.exists():
            append_log(log_path, f"[stop-file] {stop_file}")
            return 0

        if now >= deadline:
            append_log(log_path, "[deadline] max duration reached")
            return 0

        if queue_counts["runnable"] <= 0:
            if idle_started_at is None:
                idle_started_at = now
            idle_seconds = (now - idle_started_at).total_seconds()
            append_log(
                log_path,
                f"[idle] queued={queue_counts['queued']} awaiting_review={queue_counts['awaiting_review']} blocked={queue_counts['blocked']} idle_seconds={int(idle_seconds)}",
            )
            if idle_seconds >= max(args.idle_seconds, 0):
                append_log(log_path, "[idle-stop] idle timeout reached")
                return 0
            time.sleep(max(args.poll_seconds, 1))
            continue

        idle_started_at = None
        command = [
            sys.executable,
            str(runner_path),
            "--queue",
            str(queue_path),
            "--cwd",
            str(cwd),
            "--until-idle",
            "--max-jobs",
            str(max(args.max_jobs_per_pass, 1)),
        ]
        append_log(log_path, f"[run] {' '.join(command)}")
        result = subprocess.run(command, cwd=str(cwd), capture_output=True, text=True)
        runner_output_path.write_text(
            (result.stdout or "") + (result.stderr or ""),
            encoding="utf-8",
        )
        append_log(log_path, f"[run-exit] code={result.returncode}")
        passes += 1

        if result.returncode != 0:
            append_log(log_path, "[error] queue runner failed; stopping overnight supervisor")
            return result.returncode


if __name__ == "__main__":
    raise SystemExit(main())
