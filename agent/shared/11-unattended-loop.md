# Unattended EPU Loop

This file defines the durable unattended-loop model for long EPU work.

Use it when the goal is to keep benchmark-first capture/compare work running for hours without relying on chat memory or manual timestamp bookkeeping.

## Purpose

The unattended loop exists to automate the mechanical parts of the process:

- build freshness
- replay-pair capture
- deterministic batch comparison
- queue progression
- durable artifact creation

It does not replace adversarial visual review or the human-owned markdown logs.

## Scope

The unattended loop is for:

- benchmark-first engine/surface work
- deterministic full-showcase promotion pairs after a benchmark direction is healthy
- overnight capture/compare throughput

It is not for:

- auto-approving benchmark quality
- auto-passing showcase quality
- auto-editing append-only review history

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

This helper:

- runs the explicit player replay twice sequentially
- detects the exact new screenshot windows
- compares hashes pairwise
- prints the first/last filenames for both batches
- fails if a run does not append the expected screenshot count
- fails if the two batches overlap

This solves the main drift source from earlier loops: invalid determinism reads caused by overlapping or interleaved batches.

The queue runner:

- selects the next runnable queue item
- runs the listed build steps
- invokes the pair runner with the local player / ROM / screenshot defaults
- writes durable artifacts under `agent/runs/<timestamp>-<job_id>/`
- copies the exact compared screenshot windows into the run bundle
- updates queue status to `awaiting_review` on a clean pair or `blocked` on infrastructure failure
- can run one targeted job by `--job-id` or drain multiple runnable jobs with `--until-idle` / `--max-jobs`

When one replay covers multiple benchmark targets:

- do not pay for duplicate captures just because the queue models the targets separately
- run one fresh deterministic pair
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

The markdown logs remain the canonical human-readable history:

- `agent/shared/04-capture-log.md`
- `agent/shared/05-review-log.md`
- `agent/shared/09-benchmark-log.md`
- `agent/shared/06-open-gaps.md`

The unattended loop should prepare paste-ready summaries for those files, but it should not auto-edit them.

## Current Operating Rule

For now, use the unattended loop for deterministic capture/compare only.

Meaning:

- the runner may build, replay, and compare
- the queue runner may update queue state and create `agent/runs/*` bundles
- a fresh reviewer agent must still inspect screenshots directly
- only after review should the durable markdown logs be updated
- benchmark review may reuse one shared benchmark-sweep artifact for multiple targets when the replay and content state are identical

Typical commands:

```bash
python tools/tmp/run_epu_loop_queue.py --job-id front-mass-body-ownership
python tools/tmp/run_epu_loop_queue.py --until-idle --max-jobs 2
```

## Restart Rule

On any restart:

1. Read `agent/start-here.md`
2. Read `agent/shared/10-program-runbook.md`
3. Read this file
4. Read `agent/queue/epu-loop.yaml`
5. Resume the oldest meaningful `queued` job that still matches current `06-open-gaps.md`
