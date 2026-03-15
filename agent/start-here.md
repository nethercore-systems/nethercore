# EPU Program Entrypoint

This is the single restart entrypoint for the entire EPU improvement program.

If a session pauses and resumes later, start here again. Do not trust chat memory over the files listed here.

Treat this as a long-running orchestration program, not a one-shot task. Keep running in validated waves across sessions until the full 20-scene program is complete, or a real blocker/user intervention stops progress. Do not short-circuit benchmark-first, replay review, or logging discipline just because one local patch looks promising.

## Mission

Drive the EPU system and the EPU showcase to S+ quality with a benchmark-first workflow:

- improve the underlying EPU surface when the current vocabulary is insufficient
- validate those improvements in small benchmark scenes first
- only then promote healthy changes into the full 20-scene showcase loop
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
- Do not pair every showcase capture by default. Use one authoritative replay capture for routine showcase polish; reserve paired replay for benchmark work, engine/shader/runtime changes, replay-script changes, suspicious capture drift, or milestone proof.

## Hard Gate Order

Do not treat these as optional review notes. The program-level build and validation flow must clear them in this order:

1. screenshot determinism for the current capture path
2. seamless looping around 3D positions and full spatial wrap
3. seamless looping around animation value or phase
4. opcode quality through the benchmark gate
5. full 20-scene showcase pass quality

Operational meaning:

- Treat screenshot determinism as currently solved, but reopen it immediately if a fresh regression appears.
- Do not spend benchmark or showcase waves while a known spatial seam or phase-loop flash still exists in shared runtime behavior.
- Spatial seam sign-off requires a full 360-degree direct-background orbit, not one hero view.
- Animation-loop sign-off requires phase-cycle wrap checks, not just "motion exists" on a few nearby frames.

## Subagent Mode

When the current worker owns the loop end to end, operate with this default workflow:

- Work in subagent orchestration mode.
- Split the task into parallel workers with disjoint file ownership.
- Do not implement, capture, or review locally unless integration or emergency repair is required.
- Treat the current worker as the orchestrator only:
  - implementation belongs to implement/fix workers
  - capture belongs to capture workers or unattended queue machinery
  - visual judgment belongs to fresh-context review workers
- Keep a live worker roster in every progress update: active workers, scope, completed slices, next queued lane.
- Run work in waves and validate after each wave.
- If a worker stalls, rescope it rather than silently waiting forever.
- When using the live workbench, audit existing `nethercore-zx` workbench sessions first and attach/reuse a healthy session before launching another HTTP server.
- Do not spawn a fresh workbench instance per worker by default. Keep one persistent session per active lane, or serialize workers through one shared session when that is sufficient.

## Stop-Loss Rules

Do not let one lane consume the whole program.

- Cap active polishing at `3` validated waves per lane before a mandatory decision.
- After `3` waves, classify the lane as exactly one of:
  - `pass`
  - `banked near-pass`
  - `parked fail`
  - `benchmark-blocked`
- Treat `banked near-pass` as done for now. Do not reopen it immediately just because one more micro-lever exists.
- Do not spend more than `2` consecutive active waves in the same source preset file unless the second wave already produced a material review gain.
- Do not mix `engine/surface R&D` and `showcase finishing` in the same active batch. If the blocker is capability-bound, move it to benchmarks and unblock other showcase lanes in parallel.
- Always keep at least `3` live showcase lanes in motion:
  - one active survivor lane
  - one active rebuild lane
  - one parked-or-banked lane being protected from unnecessary churn
- If review indexing, screenshot mapping, or artifact selection is suspect, stop and repair that immediately before spending another authoring wave.

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
  Full 20-scene showcase replay.
- `agent/queue/epu-loop.yaml`
  Restartable unattended queue for benchmark/showcase jobs.
- `tools/tmp/run_epu_loop_queue.py`
  Queue executor that turns the unattended queue into durable `agent/runs/*` bundles.
- `tools/tmp/run_epu_overnight_loop.py`
  Overnight supervisor that keeps the unattended queue alive for long sessions with heartbeat and idle-stop behavior.
- `tools/epu_workbench.py`
  Thin local CLI for the live EPU workbench control surface.

## Program Phases

The long-term plan is:

1. Keep process/docs/prompts coherent so agents restart cleanly.
2. Keep screenshot determinism closed for the active capture path.
3. Keep spatial wrap and 3D-position looping seam-free.
4. Keep animation/phase looping seam-free.
5. Build and maintain benchmark-first capability gates.
6. Expand or correct the EPU surface only when benchmarks prove a real gap.
7. Promote healthy capability changes into the showcase.
8. Push the current 20-scene roster to pass quality.
9. Expand or reopen the roster only when the current 20-scene set is either passing or explicitly banked/parked with honest blocker classification.
10. Finish with clean docs, accurate logs, and a showcase that can usually pass in 1 to 2 focused loops.
11. Keep unattended queue artifacts current so future agents can resume overnight-style loops from disk rather than chat memory.

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

