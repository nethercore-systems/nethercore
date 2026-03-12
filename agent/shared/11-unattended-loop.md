# Unattended EPU Loop

This file defines the durable unattended-loop model for long EPU work.

Use it when the goal is to keep benchmark/showcase capture work running for hours without relying on chat memory or manual timestamp bookkeeping.

Treat paired determinism proving as background infrastructure, not the main purpose of ongoing beauty work. The unattended loop should primarily help advance benchmark/showcase authoring with durable capture bundles and clear queue state.

## Purpose

The unattended loop exists to automate the mechanical parts of the process:

- build freshness
- replay capture bundles
- optional paired comparison when diagnostics are needed
- queue progression
- durable artifact creation
- overnight supervision and restartability

It does not replace adversarial visual review, creative authoring, or the human-owned markdown logs.

## Scope

The unattended loop is for:

- benchmark-first engine/surface work
- routine benchmark/showcase capture bundles after a benchmark direction is healthy enough to run
- overnight queue supervision and state preservation

It is not for:

- auto-approving benchmark quality
- auto-passing showcase quality
- auto-editing append-only review history
- inventing new code changes while the user sleeps

## Queue Files

Prepared jobs live in:

- `agent/queue/epu-loop.yaml`

Each queue item should declare:

- `job_id`
- `status`
- `mode`
- `target`
- `replay`
- `batch_size`
- `build_steps`
- `promotion_gate`
- `notes`
- `depends_on`
- `attempts`
- `last_run_dir`

Allowed `mode` values:

- `benchmark`
- `showcase`

Allowed `status` values:

- `queued`
- `running`
- `awaiting_review`
- `blocked`
- `done`

## Runner Surface

Current low-level pair runner:

- `tools/tmp/run_epu_replay_pair.py`

Current queue runner:

- `tools/tmp/run_epu_loop_queue.py`

Current overnight supervisor:

- `tools/tmp/run_epu_overnight_loop.py`

The pair helper:

- runs the explicit player replay twice sequentially
- detects the exact new screenshot windows
- compares hashes pairwise
- prints the first/last filenames for both batches
- fails if a run does not append the expected screenshot count
- fails if the two batches overlap

Keep this available for capture-path diagnostics, but do not make paired comparison the center of every beauty wave now that determinism is considered solved.

The queue runner:

- selects the next runnable queue item
- runs the listed build steps
- invokes the pair runner with the local player / ROM / screenshot defaults
- writes durable artifacts under `agent/runs/<timestamp>-<job_id>/`
- copies the exact compared screenshot windows into the run bundle
- updates queue status to `awaiting_review` on a clean pair or `blocked` on infrastructure failure
- can run one targeted job by `--job-id` or drain multiple runnable jobs with `--until-idle` / `--max-jobs`

The overnight supervisor:

- repeatedly invokes the queue runner for a bounded duration
- writes heartbeat and summary logs under a durable supervisor directory
- records idle time instead of silently disappearing when the queue has no runnable jobs
- stops on hard runner failure, an idle timeout, or an explicit stop file
- resumes cleanly because queue state and `agent/runs/*` bundles remain the source of truth

When one replay covers multiple benchmark targets:

- do not pay for duplicate captures just because the queue models the targets separately
- run one fresh capture bundle
- review every relevant benchmark target from that same run bundle
- record in the queue/logs which other targets reused the artifact

## State Machine

Each unattended job should move through:

1. `queued`
2. `running`
3. `awaiting_review`
4. `blocked` or `done`

Benchmark jobs must not auto-promote themselves.

Showcase jobs must only be queued after the relevant benchmark direction is explicitly healthy enough to promote.

## Required Artifacts

Each unattended run should leave a repo-local bundle under:

- `agent/runs/<timestamp>-<job_id>/`

Recommended contents:

- `manifest.json`
- `summary.md`
- `commands.txt`
- `build.log`
- `pair-output.txt`
- `notes.md`
- `error.txt` when the run fails before review

If future automation copies PNGs out of `%APPDATA%`, store them under:

- `screenshots/a/`
- `screenshots/b/`

Each overnight supervisor session should leave a repo-local bundle under:

- `agent/runs/overnight/<timestamp>/`

Recommended contents:

- `heartbeat.json`
- `supervisor.log`
- `last-runner-output.txt`

The markdown logs remain the canonical human-readable history:

- `agent/shared/04-capture-log.md`
- `agent/shared/05-review-log.md`
- `agent/shared/09-benchmark-log.md`
- `agent/shared/06-open-gaps.md`

The unattended loop should prepare paste-ready summaries for those files, but it should not auto-edit them.

## Current Operating Rule

Use the unattended loop for routine benchmark/showcase capture progression.

Meaning:

- the runner may build, replay, and create durable `agent/runs/*` bundles
- paired comparison remains available and may run automatically, but it is infrastructure rather than the main review goal
- a fresh reviewer agent must still inspect screenshots directly
- only after review should the durable markdown logs be updated
- benchmark review may reuse one shared benchmark-sweep artifact for multiple targets when the replay and content state are identical

Typical commands:

```bash
python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership
python tools/tmp/run_epu_loop_queue.py --until-idle --max-jobs 2
python tools/tmp/run_epu_overnight_loop.py --hours 24 --poll-seconds 300 --idle-seconds 3600
```

## Overnight Launch Rule

Use the overnight supervisor when:

- you want the queue machinery to keep watching for runnable work for many hours
- you want durable heartbeat/state output while the user is away
- you want one clear restart surface instead of manual repeated invocations

Do not present it as a fully autonomous creative agent. It can only execute the queued build/capture machinery and preserve state until a reviewer or later agent session picks up the next wave.

## Restart Rule

On any restart:

1. Read `agent/start-here.md`
2. Read `agent/shared/10-program-runbook.md`
3. Read this file
4. Read `agent/queue/epu-loop.yaml`
5. Inspect the newest `agent/runs/*` or `agent/runs/overnight/*` artifacts
6. Resume the oldest meaningful `queued` job that still matches current `06-open-gaps.md`
