# EPU Program Entrypoint

This is the single restart entrypoint for the entire EPU improvement program.

If a session pauses and resumes later, start here again. Do not trust chat memory over the files listed here.

## Mission

Drive the EPU system and the EPU showcase to S+ quality with a benchmark-first workflow:

- improve the underlying EPU surface when the current vocabulary is insufficient
- validate those improvements in small benchmark scenes first
- only then promote healthy changes into the 12-preset showcase loop
- keep the prompt pack, logs, and docs aligned so future agents do not re-derive stale assumptions

Judge success against EPU's real product role:

- a shippable cubemap-replacement workflow
- strong ambient lighting and reflection/IBL contribution
- metaphor-first world-place reads
- fantasy-console constraints, not literal feature-film scene rendering

Remember the showcase/benchmark camera rig:

- a reflective hero probe sits in the foreground during authoritative captures
- the EPU renders into the background behind that probe
- review must judge both direct background readability and reflection contribution

This is not just a showcase-art task. It is a combined:

- EPU surface capability program
- benchmark/gating program
- showcase validation program
- durable process/documentation program

## Non-Negotiable Rules

- Process authority: `agent/session-protocol.md`
- Mission authority: `agent/shared/00-charter.md`
- Current mutable truth: `agent/shared/06-open-gaps.md`
- Benchmark gate truth: `agent/shared/08-benchmark-suite.md` and `agent/shared/09-benchmark-log.md`
- Visual contract truth: `agent/shared/07-preset-briefs.md`
- Validation history: `agent/shared/04-capture-log.md` and `agent/shared/05-review-log.md`
- Runtime truth lives in code, replay scripts, and screenshots, not planning prose
- If code/docs/prompts drift, fix the drift immediately before continuing broad loops

## Cold-Start Read Order

Read these in order before choosing work:

1. `agent/session-protocol.md`
2. `agent/shared/06-open-gaps.md`
3. `agent/shared/10-program-runbook.md`
4. `agent/shared/08-benchmark-suite.md`
5. `agent/shared/09-benchmark-log.md`
6. `agent/shared/05-review-log.md`
7. `agent/shared/04-capture-log.md`
8. `agent/shared/07-preset-briefs.md`
9. `agent/shared/03-validation-playbook.md`
10. `agent/shared/12-live-workbench.md`
11. `agent/shared/00-charter.md`

For code-writing or shader-writing runs, also re-read:

- `docs/book/src/guides/epu-environments.md`
- `docs/architecture/epu-surface-expansion-plan.md`
- `nethercore-zx/src/debug/epu_capabilities.rs`
- `docs/architecture/zx/rendering.md`
- `examples/3-inspectors/epu-showcase/src/constants.rs`

## Durable Artifacts

These files carry long-term program state and must stay current:

- `agent/shared/10-program-runbook.md`
  Canonical program phases, decision tree, and restart procedure.
- `agent/shared/06-open-gaps.md`
  Current blockers, next batch, and current-state truth.
- `agent/shared/08-benchmark-suite.md`
  Benchmark definitions and promotion rules.
- `agent/shared/09-benchmark-log.md`
  Benchmark capture/review history.
- `agent/shared/11-unattended-loop.md`
  Long-running capture/compare workflow and restart rules for unattended loops.
- `agent/shared/12-live-workbench.md`
  Restartable live authoring workflow for machine-driven local EPU tuning.
- `agent/shared/04-capture-log.md`
  Showcase and targeted capture history.
- `agent/shared/05-review-log.md`
  Review verdicts and failure classes.
- `docs/architecture/epu-surface-expansion-plan.md`
  System-level EPU capability plan and opcode-surface roadmap.
- `examples/3-inspectors/epu-showcase/src/benchmarks.rs`
  Runtime benchmark scenes.
- `examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
  Benchmark replay gate.
- `examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs`
  Full 12-preset showcase replay.
- `agent/queue/epu-loop.yaml`
  Restartable unattended queue for benchmark/showcase jobs.
- `tools/tmp/run_epu_loop_queue.py`
  Queue executor that turns the unattended queue into durable `agent/runs/*` bundles.
- `tools/epu_workbench.py`
  Thin local CLI for the live EPU workbench control surface.

## Program Phases

The long-term plan is:

1. Keep process/docs/prompts coherent so agents restart cleanly.
2. Build and maintain benchmark-first capability gates.
3. Expand or correct the EPU surface only when benchmarks prove a real gap.
4. Promote healthy capability changes into the showcase.
5. Push the current 12 presets to pass quality.
6. Only after proof-of-life across the current set, expand toward the full 20-preset roster.
7. Finish with clean docs, accurate logs, and a showcase that can usually pass in 1 to 2 focused loops.
8. Keep unattended queue artifacts current so future agents can resume overnight-style loops from disk rather than chat memory.

## Work Selection Decision Tree

Choose the next loop by scope, not by habit:

- If the blocker is doc drift, prompt drift, or terminology drift:
  fix the docs/prompt pack first
- If the change touches EPU runtime, shader logic, opcode semantics, builder/packing, benchmark scenes, or capability metadata:
  run the benchmark loop first
- If the next task is rapid local tuning, scripted sweeps, or candidate discovery on one machine:
  use the live workbench first, then replay-promote only the winning candidates
- If the benchmark still clearly fails in the same way:
  stay in engine/surface work and do not burn a full showcase sweep
- If the benchmark direction is healthy:
  promote to the showcase loop
- If the change is preset-only and does not alter the EPU surface:
  use targeted showcase capture/review directly
- If repeated deterministic loops keep failing in the same way:
  log a real surface gap or engine bug instead of reopening blind preset churn

## Job Packs

After rehydrating, choose exactly one job pack for the current worker role:

- `agent/jobs/orchestrate/PROMPT.md`
- `agent/jobs/design/PROMPT.md`
- `agent/jobs/implement/PROMPT.md`
- `agent/jobs/capture/PROMPT.md`
- `agent/jobs/review/PROMPT.md`
- `agent/jobs/fix/PROMPT.md`

Use `orchestrate` when owning the full loop end to end.

## Stop Condition

Do not stop at a code patch. Continue until one of these is true:

- the benchmark or showcase batch has been captured, reviewed, and logged
- the current line of work is blocked by a real missing capability, engine bug, or user intervention
- the work reached a clean pass and the logs/docs reflect that pass

## Restart Instruction

If resuming after any pause:

1. Re-read this file and the cold-start files above.
2. Confirm the current phase and blocker in `agent/shared/06-open-gaps.md`.
3. Confirm whether the next loop should start in the live workbench, benchmark-first replay, or showcase-first replay from `agent/shared/10-program-runbook.md` and `agent/shared/12-live-workbench.md`.
4. Run the next smallest truthful loop.
5. Update the durable artifacts before handing off.

