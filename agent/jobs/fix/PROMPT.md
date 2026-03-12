# Fix Pack

Use this pack only for targeted fixes driven by concrete review failures.

## Read Order

1. `agent/session-protocol.md`
2. `agent/shared/06-open-gaps.md`
3. `agent/shared/10-program-runbook.md`
4. `agent/shared/08-benchmark-suite.md`
5. `agent/shared/09-benchmark-log.md`
6. `agent/shared/07-preset-briefs.md`
7. `agent/shared/05-review-log.md`
8. `agent/shared/02-coverage-matrix.md`
9. `agent/shared/01-roster.md`
10. `agent/shared/12-live-workbench.md`
11. `agent/shared/00-charter.md`
12. `docs/book/src/guides/epu-environments.md`
13. `docs/architecture/zx/rendering.md`
14. `src/constants.rs`
15. Relevant preset code and replay files

## Objective

- fix named failures from the review log
- keep blast radius narrow
- use EPU-aware hypotheses instead of blind parameter churn
- treat `06-open-gaps.md` and the latest reviews as the live state; use coverage as inventory, not as the default next-target source
- treat each preset's `Text prompt:` line in `agent/shared/07-preset-briefs.md` as the literal repair target
- restore the brief's missing required reads before tuning mood polish
- every preset changed in this workflow must move visibly across review frames
- if the next step is high-frequency tuning on one machine, use the live workbench first and replay-promote the winning candidate instead of burning full replay runs on every micro-fix
- when repairing animation, make the reviewed frames show obvious motion
- when feasible, shift animated layers toward clean phase-looped motion rather than noisy pseudo-random drift, but treat loopability as a known risk that must be validated
- treat phase support as variant-specific, not opcode-wide. Verify the exact authored variant and shader path before assuming an `ANIM_SPEEDS` slot will produce smooth motion
- if the missing read depends on behavior the current opcode surface cannot provide, log an opcode-surface/tooling gap instead of inventing more parameter churn
- if the fix changes EPU runtime/opcode behavior, name the target benchmark up front and do not skip the benchmark gate
- preserve roster-level distinctness. If a repair makes two presets read like siblings or recolors, fail that direction and pick a more separated one
- prefer minimal, high-leverage changes over broad rewrites

## Required Verification

- run `cargo fmt`
- build the example
- validate replay scripts if they changed

## Boundaries

- do not mark a preset as passing
- do not treat a live-workbench capture as a final pass; replay promotion is still required
- do not assume looping or repeated patterning is authored intent
- if an artifact looks engine-driven or survives preset, opcode, or domain changes, stop content-only churn and log a suspected engine bug
- if the blocker is a capability mismatch rather than a renderer defect, log an opcode-surface/tooling gap and stop pretending another preset-only pass will solve it
- do not make targeted EPU fixes without first reading the EPU guide, rendering architecture note, and showcase constants
- do not redesign the entire roster unless the user explicitly approves it
- do not edit locked docs
