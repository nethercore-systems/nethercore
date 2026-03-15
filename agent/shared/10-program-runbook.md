# EPU Program Runbook

This file is the canonical long-term runbook for the EPU system and showcase effort.

Use it to restart the program after a pause without relying on chat history.

## Program Goal

Make EPU a reliable cubemap-replacement workflow for Nethercore:

- elegant
- flexible
- fast to author
- fantasy-console constrained but empowering
- strong enough for ambient lighting, IBL/reflection use, and direct-view world scenes

It is not a goal for EPU to behave like an unconstrained literal scene renderer. The target is a shippable, elegant, metaphor-first environment system that helps developers avoid hand-built skyboxes.

The showcase is the validation surface, not the only product.

## Program Scope

There are four linked workstreams:

1. EPU surface capability development
2. benchmark/gating development
3. showcase quality validation
4. prompt-pack and documentation coherence

The process should move from smaller truth-gates to larger art-validation loops, not the other way around.
Live tuning sits ahead of replay validation when the task is local discovery rather than final proof.

## Required Gate Ladder

The program must progress through these gates in order:

1. screenshot determinism
2. seamless 3D-position looping and spatial wrap
3. seamless animation-value or phase looping
4. opcode quality through benchmark validation
5. full 20-scene showcase quality

Current standing truth:

- determinism is considered solved until a fresh regression proves otherwise
- spatial seams are not considered solved without a 360-degree direct-background orbit check
- animation looping is not considered solved without checking the actual phase-wrap boundary
- benchmark and showcase work must not be used to hide unresolved shared loop defects

Default operating mode is persistent orchestration across waves. Each wave should choose the next smallest truthful slice, delegate disjoint ownership, validate through the required gate for that slice, log the result, and queue the next wave. The program is not complete until all 20 scenes are implemented, validated, reviewed, and logged, unless a real blocker is explicitly recorded.

When the current session is acting as the orchestration owner:

- do not personally implement, capture, or review if a worker can own that slice
- use implementation workers for code and preset edits
- use capture workers or unattended queue machinery for replay runs
- use fresh-context review workers for screenshot judgment
- keep local work limited to integration, queue control, and emergency repair

## Stop-Loss Policy

The loop must optimize for roster progress, not local perfection.

- Hard cap: `3` validated waves per active lane before a forced classification.
- Forced classifications:
  - `pass`
  - `banked near-pass`
  - `parked fail`
  - `benchmark-blocked`
- `banked near-pass` means stop active churn. It stays available for a later shipping-polish pass, but it is not allowed to keep consuming the next wave by default.
- If two consecutive waves on the same lane produce only micro-delta reviews, the third wave must be the last one before bank/park.
- Do not run more than `2` consecutive active waves in the same source file unless the previous wave materially changed the review class.
- If a lane is blocked by capability, move it to benchmark or engine work and immediately rotate a different showcase lane into the active batch.
- Keep parallel breadth:
  - one survivor lane
  - one rebuild lane
  - one benchmark or blocked lane tracked separately without stealing every wave
- Protect passes and banked near-passes from needless reopen churn.

## Source-Of-Truth Map

Read these files as the durable state model:

- `agent/session-protocol.md`
  Process rules and delegation discipline.
- `agent/shared/00-charter.md`
  Mission-level product intent.
- `agent/shared/06-open-gaps.md`
  Current blockers and next-batch truth.
- `agent/shared/08-benchmark-suite.md`
  Benchmark definitions and promotion requirements.
- `agent/shared/09-benchmark-log.md`
  Benchmark history and verdicts.
- `agent/shared/11-unattended-loop.md`
  Unattended capture/compare workflow and queue semantics.
- `agent/shared/12-live-workbench.md`
  Restartable local live-authoring workflow for high-iteration EPU tuning.
- `agent/shared/04-capture-log.md`
  Capture history.
- `agent/shared/05-review-log.md`
  Review history.
- `agent/shared/07-preset-briefs.md`
  Preset scene contracts.
- `agent/shared/03-validation-playbook.md`
  Build, replay, capture, and review procedure.
- `docs/architecture/epu-surface-expansion-plan.md`
  Current system-level capability roadmap.

Code-backed truth lives in:

- `nethercore-zx/src/graphics/epu/*`
- `nethercore-zx/shaders/epu/*`
- `nethercore-zx/src/debug/epu_capabilities.rs`
- `examples/3-inspectors/epu-showcase/src/*`
- replay scripts on disk
- newest screenshot batches

## Durable Program Phases

### Phase 1: Process Integrity

Keep the docs, prompts, and terminology aligned:

- `bounds` and `features` wording must stay correct
- sequencing rules must stay correct
- benchmark-first promotion rules must stay visible
- known capability limits must be updated as soon as they are proven

If process truth drifts, fix that before broad implementation churn.

### Phase 2: Capability Discovery Through Benchmarks

Use the benchmark suite for small, fast truth checks.

Do not enter this phase dishonestly. If spatial wrap or animation wrap is still visibly broken in shared runtime behavior, the benchmark phase is blocked until those technical gates are closed.

Judge benchmark health by whether the carrier class is shippable for EPU's actual role:

- readable enough in direct view
- useful for ambient/reflection contribution
- metaphorically strong enough to sell place
- still appropriately constrained for a fantasy-console surface

Current benchmark set:

- `Open Horizon`
- `Region Isolation`
- `Projection Bay`
- `Transport Sweep`
- `Front Mass`
- `Frozen Bed`

When changing shaders, opcodes, builder/packing, capability metadata, or benchmark scenes:

1. rebuild the relevant binaries/ROM
2. validate the benchmark replay script
3. run the benchmark replay in the real player
4. review the PNGs
5. log benchmark truth in `agent/shared/09-benchmark-log.md`

If one benchmark replay sweep covers multiple blocked targets in the exact same content state:

- capture once
- review all relevant targets from that shared artifact
- update every affected queue/log entry instead of rerunning the same replay redundantly

Do not promote a capability change to the full showcase until the relevant benchmark direction is healthier.

### Phase 2.5: Live Workbench Discovery

Use the live workbench for:

- rapid local parameter tuning
- candidate discovery before touching Rust presets
- layer-by-layer experimentation
- scripted sweeps and candidate export

The live workbench is for fast local discovery on one machine. It should teach the surface faster, then export winning candidates back into durable source and replay validation.

Live iteration does not waive artifact quality:

- obvious concentric rings are failures
- visible noise or tiling seams are failures
- visible phase-wrap flashes or end-of-loop pops are failures
- repeated technical pattern edges that read as shader seams are failures

If those survive across multiple presets or carrier swaps, treat the problem as engine/surface work rather than more blind content churn.

### Phase 3: EPU Surface Development

Only add or reshape surface vocabulary when repeated deterministic evidence shows the current surface is insufficient.

Allowed reasons to change the surface:

- repeated benchmark failure with the same failure class
- repeated showcase failure that maps cleanly to a benchmark gap
- proven engine/runtime bug
- proven authoring-surface mismatch that causes agents to make bad assumptions

Not sufficient by itself:

- one ugly preset
- vague dissatisfaction
- literal scene requests that violate EPU's metaphor-first intent

Surface additions must stay abstract and reusable. Do not add scene-literal opcodes like `rain` or `glacier`.

### Phase 4: Showcase Promotion

Once the relevant benchmark is healthy:

1. run targeted or full showcase replay
2. review the screenshots adversarially
3. log the result in capture/review logs
4. update `agent/shared/06-open-gaps.md`

The showcase is where the system proves it can create scene identity, not where a missing capability should first be discovered.

Scene identity here still means metaphor-first place read and shippable world-light utility, not literal prop-perfect rendering.

### Phase 5: Full-Roster Finish

The 20-scene roster already exists in code and the full replay sweep is the active validation surface.

Do not create additional roster churn beyond the current 20 scenes until:

- the current 20-scene set is honestly classified lane by lane
- benchmark-blocked outdoor lanes are either improved or explicitly parked with system-level follow-up
- the benchmark suite is stable enough that new scene additions would not just multiply the same uncertainty

Expansion without current-roster health just multiplies noise.

## Decision Tree

Use this exact triage:

### A. The problem looks like drift or confusion

Examples:

- wrong terminology
- stale prompt assumptions
- builder/runtime mismatch
- stale opcode map

Action:

- fix docs, prompts, metadata, or authoring surface first

### B. The problem looks like a capability gap

Examples:

- repeated deterministic failure
- same benchmark fails after multiple tuned passes
- current opcode families cannot produce the required abstract behavior

Action:

- document the gap
- update the surface plan
- implement or reshape the smallest abstract capability that closes the gap
- validate on the benchmark first

### C. The problem looks like a preset/content miss

Examples:

- benchmark is healthy but the showcase preset still misses the brief
- scene structure is wrong for that one preset
- motion is authored too subtly

Action:

- stay in preset authoring
- do targeted replay/review
- do not widen into engine work

### D. The problem looks like an engine bug

Examples:

- code and docs disagree
- behavior violates parameter contract
- different presets/opcodes fail in the same artifact pattern
- concentric rings, seam lines, or repeated technical edges survive across scenes or carrier swaps
- loop-wrap flashes or phase snaps survive across scenes or carrier swaps
- repeated deterministic captures show the same obviously wrong runtime behavior

Action:

- isolate the bug
- fix it
- run benchmark-first validation

## Required Artifacts Per Loop

Every meaningful loop should leave durable evidence:

### Surface/engine loop

- code changes
- technical loop validation for the relevant open gate:
  - determinism pair if capture truth is suspect
  - 360-degree orbit if spatial wrap is suspect
  - full phase-cycle wrap check if animation looping is suspect
- benchmark replay run
- benchmark review note
- updated `agent/shared/06-open-gaps.md`
- updated `docs/architecture/epu-surface-expansion-plan.md` if the roadmap changed
- prompt/doc updates if capability semantics changed

### Preset/showcase loop

- code changes
- targeted or full showcase replay run
- review note
- updated `agent/shared/06-open-gaps.md`

### Process/doc loop

- doc/prompt changes
- corrected source-of-truth links
- updated `agent/shared/06-open-gaps.md` if process state changed materially

## Validation Commands

Use `agent/shared/03-validation-playbook.md` as the command authority.

Canonical high-level paths:

- Live discovery:
  - start or attach to the live workbench
  - tune or sweep locally
  - capture workbench outputs
  - export a candidate
- Benchmark-first:
  - build
  - replay validate `screenshot-benchmarks-anim3.ncrs`
  - real player `--replay` on the benchmark script
- Showcase:
  - build
  - replay validate `screenshot-all-anim3.ncrs`
  - single real-player `--replay` on the showcase script by default
  - paired replay only for milestone proof, engine/shader/replay changes, or suspected capture drift

Final visual authority is always the real-player screenshot batch.
Fast local discovery authority lives in the live workbench captures and exports described in `agent/shared/12-live-workbench.md`.

For unattended deterministic pair capture, use:

- `tools/tmp/run_epu_replay_pair.py`

For queued unattended benchmark/showcase execution with durable artifacts, use:

- `tools/tmp/run_epu_loop_queue.py`

For long unattended supervision around that queue runner, use:

- `tools/tmp/run_epu_overnight_loop.py`

That helper runs the replay twice sequentially, records the exact batch windows, and compares the new screenshots pairwise.

## Current Strategic Focus

As of the current state in `agent/shared/06-open-gaps.md`:

- protect known proof-of-life wins
- use the benchmark harness as the first gate for EPU surface work
- move the showcase roster in broader parallel batches instead of endless micro-polish on one survivor
- bank near-passes aggressively and reopen them only for explicit final-shipping polish
- keep the prompt pack accurate as soon as new capabilities or limits are proven

## Handoff / Restart Checklist

Before ending a session:

1. Update the relevant logs.
2. Update `agent/shared/06-open-gaps.md`.
3. Update this runbook if the phase model or promotion logic changed.
4. Ensure the next likely loop is obvious from the files.

When restarting:

1. Read `agent/start-here.md`.
2. Re-read `agent/shared/06-open-gaps.md`.
3. Re-read this runbook.
4. Check the newest benchmark or showcase log entries.
5. Pick the next smallest loop that can produce truth.

## Success Condition

The program is healthy when:

- new agents can restart from files without context rot
- capability work is benchmarked before showcase promotion
- showcase passes come from established capability truth instead of guesswork
- final showcase quality can usually be recovered in 1 to 2 focused passes instead of multi-hour rediscovery
