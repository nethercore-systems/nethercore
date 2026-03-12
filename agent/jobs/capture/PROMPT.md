# Capture Pack

Use this pack for replay execution and screenshot batch collection only.

## Read Order

1. `agent/session-protocol.md`
2. `agent/shared/06-open-gaps.md`
3. `agent/shared/10-program-runbook.md`
4. `agent/shared/08-benchmark-suite.md`
5. `agent/shared/09-benchmark-log.md`
6. `agent/shared/04-capture-log.md`
7. `agent/shared/03-validation-playbook.md`
8. `agent/shared/12-live-workbench.md`
9. `agent/shared/00-charter.md`
10. Relevant `screenshot*.ncrs`

## Objective

- run the real player with the intended replay script
- collect the screenshot batch
- record what was run and what was produced
- treat `06-open-gaps.md` and the latest capture log as the current-state source for what needs to be captured next; use the charter as mission context only
- support prompt-fidelity, motion, and loopability verification with the right replay spacing instead of assuming the default sweep is always enough
- when the task is live local discovery instead of replay validation, use `agent/shared/12-live-workbench.md` and record the resulting workbench artifact path clearly

## Build Preconditions

- Confirm whether engine-side files changed. If yes, require a fresh `cargo dev` before capture.
- If the scope changed EPU runtime/opcode behavior, capture the benchmark replay before the full showcase replay.
- Confirm the target example ROM was rebuilt explicitly for the current source state before capture.
- Prefer an explicit player executable path for authoritative capture. Avoid ambiguous launcher discovery when logging final validation runs.
- If the scope includes visible-animation or loopability claims, ensure the selected replay script actually samples spaced frames that can prove or disprove those claims.
- If the executable or ROM freshness is unclear, log that the baseline is invalid and stop.

## Boundaries

- no preset design changes
- no code edits
- no visual pass/fail verdicts
- if the replay script is broken or stale, log it and stop

## Required Logging

Append a dated entry to `04-capture-log.md` with:

- run id
- executable path used
- script path
- scope
- output summary
- any anomalies

If the run is benchmark-first work, append the matching state to `09-benchmark-log.md` too.
