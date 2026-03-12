# Session Protocol

This file defines how every future run must rehydrate context and limit blast radius.

## Fresh-Context Contract

- Start every run as if no prior session memory is trustworthy.
- Re-read the required files from disk at the start of the run.
- If a previous chat message conflicts with files on disk, trust the files.
- If a mutable planning file conflicts with current code, trust the code and log the drift.

## One-Pack Rule

- Use exactly one job pack per run.
- Exception: `jobs/orchestrate` may span the full loop because it is the explicit end-to-end pack.
- If you need to switch jobs mid-stream, stop, update logs, and re-read the next pack from the top.

## Edit Discipline

- Only edit files listed in the chosen job pack's `ALLOWED_FILES.md`.
- If a necessary fix falls outside that surface, stop and either:
  - hand off to the correct job pack, or
  - ask the user for approval to widen scope.

## Logging Discipline

- Treat `04-capture-log.md` and `05-review-log.md` as append-only.
- Do not silently rewrite history.
- If you need to correct an earlier mistake, add a new dated correction entry.
- Keep `06-open-gaps.md` current after each meaningful run.

## Locked Docs

Do not edit these without explicit user approval:

- `agent/session-protocol.md`
- `agent/shared/00-charter.md`

## Required Start Sequence

At the start of each run:

1. Read `agent/start-here.md`.
2. Read this file.
3. Read `agent/shared/06-open-gaps.md`.
4. Read `agent/shared/10-program-runbook.md`.
5. Read the chosen job pack and any additional shared docs it requires.
6. Read `agent/shared/00-charter.md` before making product-direction calls.
7. Read the exact code and replay files needed for the current task.

## Stop Conditions

Stop and surface the issue if:

- the worktree contains unexpected conflicting edits in the same files you need
- the chosen pack is too narrow for the actual task
- replay or screenshot validation cannot be performed but the task requires a pass/fail judgment
- a change would violate determinism or the repo hard rules
