# EPU Benchmark Suite

This file defines the benchmark-first gate for EPU work.

Use the benchmark suite to validate capability changes before spending a full 12-preset showcase pass.

## Why This Exists

The old loop was too expensive:

- change opcode or shader
- run the full showcase
- inspect screenshots
- infer system truth from final scenes

That mixes engine R&D with final content review and burns time on noisy conclusions.

The benchmark suite is the cheaper first gate.

These benchmarks are not trying to prove that EPU can paint literal film-quality backdrops. They exist to prove that the current surface is shippable for its real job:

- metaphor-first place read
- ambient/reflection/IBL usefulness
- world-integrated motion and structure
- fantasy-console constrained authoring that developers can actually ship

The benchmark screenshots include a reflective hero probe in the foreground while the EPU renders in the background. Review the pair together:

- the direct background proves world-space place read
- the probe proves ambient/reflection usefulness and backside coverage
- a benchmark is not healthy if the important read only survives on the probe when direct view is part of the contract

## Source Of Truth

- Runtime scenes live in `examples/3-inspectors/epu-showcase/src/benchmarks.rs`
- Benchmark replay lives in `examples/3-inspectors/epu-showcase/screenshot-benchmarks-anim3.ncrs`
- Full showcase replay still lives in `examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs`

## Suite

### 1. `Open Horizon`

Purpose:
- validate outdoor openness, readable sky/wall/floor separation, and anti-chamber structure without demanding literal landscape modeling

Must read:
- open-air horizon metaphor
- distinct lower ground bed or cold field support
- no enclosing amphitheater collapse

Hard fail:
- scene collapses into one pale bowl, chamber, or wall belt

### 2. `Region Isolation`

Purpose:
- validate that floor carriers stay on the floor under mixed bounds

Must read:
- floor-only detail remains grounded enough to support shipping ambient/reflection use
- wall detail does not repaint the floor into a generic wash

Hard fail:
- floor/wall separation smears into a generic full-scene wash

### 3. `Projection Bay`

Purpose:
- validate world-integrated local projection architecture

Must read:
- one clear rectangular bay or projection volume
- room-integrated energy, not HUD/UI cards

Hard fail:
- overlay-like panels, unreadable bay shape, or static frame with no world context

### 4. `Transport Sweep`

Purpose:
- validate broad transport motion as motion

Must read:
- obvious direct-view movement across the reviewed frames
- motion readable as transported world energy, not just support shimmer

Hard fail:
- motion is too subtle, seed-shimmery, or collapses into static bars/slabs

### 5. `Front Mass`

Purpose:
- validate one dominant front body, not just transport

Must read:
- a coherent scene-owning wall/front mass in direct view
- enough body ownership that the benchmark would contribute real weather/front mood in reflections and ambient lighting
- the body must own the frame without relying on lightning or another accent layer to carry the scene

Hard fail:
- pale shell, thin slab, weak sheet, or synthetic panel read

### 6. `Frozen Bed`

Purpose:
- validate frozen surface identity distinct from `PLANE/WATER`

Must read:
- icy bed or crusted frozen floor
- not a generic wet reflective water floor
- enough frozen identity to justify shipping it as an abstract ice/tundra carrier instead of a water stand-in

Hard fail:
- surface still primarily reads as water

## Promotion Rule

If the change touches:

- EPU shaders
- EPU builder/packing/runtime
- opcode metadata or capability docs
- showcase benchmark scenes
- replay scripts used for capability validation

then the next loop must be:

1. benchmark replay
2. benchmark review
3. only then full showcase replay, if the relevant benchmark direction is healthy

Do not jump directly to the full showcase unless the benchmark step is unchanged or already closed for that exact surface.

## Agent Rule

When an engine/opcode change is proposed:

- identify the target benchmark first
- state which benchmark must improve
- run the benchmark sweep before the showcase sweep
- if the benchmark still clearly fails in the same way, stop reopening full-roster churn

## Current Interpretation

- `Projection Bay` is already the strongest positive proof-of-life class
- `Front Mass` is the current hardest blocker
- `Frozen Bed` is the secondary outdoor/material blocker
