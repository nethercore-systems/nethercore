# Orchestrate Pack

Use this pack when the instruction is to own the entire EPU showcase effort end to end.

## Read Order

1. `agent/session-protocol.md`
2. `agent/shared/06-open-gaps.md`
3. `agent/shared/10-program-runbook.md`
4. `agent/shared/08-benchmark-suite.md`
5. `agent/shared/09-benchmark-log.md`
6. `agent/shared/07-preset-briefs.md`
7. `agent/shared/04-capture-log.md`
8. `agent/shared/05-review-log.md`
9. `agent/shared/02-coverage-matrix.md`
10. `agent/shared/01-roster.md`
11. `agent/shared/03-validation-playbook.md`
12. `agent/shared/12-live-workbench.md`
13. `agent/shared/00-charter.md`
14. `docs/book/src/guides/epu-environments.md`
15. `docs/architecture/zx/rendering.md`
16. `examples/3-inspectors/epu-showcase/src/constants.rs`
17. Current code and replay files needed for the batch

## Objective

Run the full loop:

- design
- implement
- capture
- review
- fix
- repeat

Do not stop at code changes. The work is not done until replay screenshots have been reviewed and the logs show why the set passes.

## Working Rules

- Work in small batches, usually 2 to 4 presets at a time.
- If the change touches EPU surface/runtime behavior, run the benchmark suite before the full showcase sweep.
- If the next task is rapid local discovery, high-frequency tuning, or sweep exploration, use the live workbench before spending replay captures.
- Keep mutable planning docs current as you go.
- Use the other job packs as discipline guides even though you are running end to end.
- Use fresh-context reviewer agents for adversarial screenshot passes. Do not let the implementer or a context-bloated agent grade its own work.
- Separate engine freshness from ROM freshness. A rebuilt example ROM does not prove the player binary is current.
- `cargo ba` is not a showcase validation step.
- Use `cargo xtask build-examples` only when the task is to validate installed library examples; do not substitute it for the single-project replay loop.
- Before authoritative capture, know exactly which player executable will launch and rebuild it if needed.
- Do not promote an opcode/runtime change to a full showcase loop until the relevant benchmark direction is at least directionally healthy in `agent/shared/08-benchmark-suite.md`.
- Treat `07-preset-briefs.md` as the scene contract for both authoring and review. Do not accept a preset that scores well on mood but misses its named structural reads.
- Treat each preset's `Text prompt:` line in `07-preset-briefs.md` as the canonical one-paragraph spec for implementers and reviewers.
- Treat roster-level distinctness as part of the contract. If a preset feels like a recolor, remix, or shape-language duplicate of another showcase scene, fail it.
- Treat every showcase preset as failing unless motion is obvious across the reviewed frames.
- Treat visible artifacts or obvious rendering errors as automatic fails.
- When a preset can use clean phase-looped motion, prefer that over noisy or ambiguous motion, but treat loopability as a known defect area that must be validated rather than assumed.
- Treat phase support as variant-specific. Do not let workers assume an opcode moves just because some other variant in that opcode family uses `param_d` as phase.
- Treat looping or repeated patterning as a first-class defect until proven intentional.
- If an artifact looks engine-driven or survives preset, opcode, or domain changes, log a suspected EPU/rendering bug and stop content-only churn.
- If a blocker is really missing opcode-surface behavior or tooling support, log it as an opcode-surface/tooling gap and decide explicitly whether the next step is preset tuning or engine/surface work. Do not recycle it as another blind content loop.
- Any worker doing targeted EPU fixes must read the EPU guide, rendering architecture note, and showcase constants before changing code.
- Do not widen scope casually.
- Do not confuse a live-workbench win with an authoritative replay pass. Export the candidate, then revalidate it through replay before claiming promoted progress.
- Do not edit locked docs.

## Required Outputs

- updated code and replay assets
- updated benchmark suite state when the surface changed
- updated roster and coverage state
- real capture log entries
- adversarial review log entries
- honest open gaps

## Hard Stop

If replay screenshots cannot be reviewed, do not claim completion.
