# Implement Pack

Use this pack for preset authoring, wiring, and replay-script alignment.

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
15. `src/presets.rs`
16. Relevant `src/presets/*.rs`
17. Relevant `screenshot*.ncrs`

## Objective

- implement a small preset batch or targeted preset changes
- keep names, counts, and animation tables aligned
- treat `06-open-gaps.md` and the latest reviews as the live priority source; do not chase old assumptions or coverage checkboxes ahead of the current blockers
- treat each preset's `Text prompt:` line in `agent/shared/07-preset-briefs.md` as the literal target to author toward
- preserve the brief's required visual reads before chasing secondary flourish
- every preset changed in this workflow must move visibly across review frames
- if the task is micro-tuning or candidate discovery, use the live workbench to patch, capture, and export quickly instead of burning replay runs for every small adjustment
- when changing animated effects, bias toward motion that will read clearly in review frames
- prefer clean loopable phase-driven motion where the opcode family supports it, but treat looping as a known risk that must be validated
- treat phase support as variant-specific, not opcode-wide. Verify the exact authored variant and shader path before assuming an `ANIM_SPEEDS` slot will produce smooth motion
- if a desired read depends on behavior the current opcode surface cannot supply, surface an opcode-surface/tooling gap instead of forcing it into preset-only authoring
- if the change alters EPU runtime/opcode behavior, identify the benchmark scene that should move before touching the full showcase sweep
- preserve roster-level distinctness. If the preset converges toward its nearest neighbor in palette, silhouette, floor language, or overall scene structure, stop and choose a different approach
- keep replay scripts aligned with the real preset count and capture needs

## Required Verification

- run `cargo fmt`
- build the example
- validate replay script syntax

## Boundaries

- do not mark presets as visually passing
- do not treat a live-workbench capture as a final pass; replay promotion is still required
- do not assume looping or repeated patterning is content-authored when review flags it
- if an artifact looks engine-driven or survives preset, opcode, or domain changes, stop blind preset tuning and surface a suspected engine bug
- if the blocker is an opcode-surface/tooling limitation rather than a renderer defect, log that explicitly and stop pretending another content-only pass will solve it
- do not make targeted EPU fixes without first reading the EPU guide, rendering architecture note, and showcase constants
- do not rewrite planning history casually
- do not edit locked docs
