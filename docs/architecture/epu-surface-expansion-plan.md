# EPU Capability Audit And Next-Surface Spec

## Purpose

This document replaces the earlier speculative expansion draft with an audit grounded in current code and the deterministic showcase loops completed through `2026-03-11-epu-showcase-12preset-replay-41`.

Goal:
- describe the current EPU surface as it actually exists now
- separate showcase content misses from real opcode-surface gaps
- define the smallest next implementation surface that can move outdoor proof-of-life quality

This is not a preset design document. Presets are validation targets for the surface.

## Design Guardrail

EPU should stay metaphor-first and fantasy-console-like.

New surface work should:
- add reusable structural, motion, or material-response carriers
- avoid literal scene nouns as opcode identity
- keep `bounds` focused on scene organization
- keep readable motion and motif behavior concentrated in `features`

Bad direction:
- `RAIN`
- `GLACIER`
- `STORM`

Good direction:
- `ADVECT`
- `SHEET`
- `SURFACE`
- `DRIFT`

## Current Runtime Truth

### Opcode surface

Current runtime code implements:
- `0x01..0x07` bounds
- `0x08..0x17` features
- `0x18..0x1F` reserved

Implemented bounds:
- `RAMP`
- `SECTOR`
- `SILHOUETTE`
- `SPLIT`
- `CELL`
- `PATCHES`
- `APERTURE`

Implemented features:
- `DECAL`
- `GRID`
- `SCATTER`
- `FLOW`
- `TRACE`
- `VEIL`
- `ATMOSPHERE`
- `PLANE`
- `CELESTIAL`
- `PORTAL`
- `LOBE`
- `BAND`
- `MOTTLE`
- `ADVECT`
- `SURFACE`
- `MASS`

Important already-landed expansions:
- `SPLIT` now includes `TIER` and `FACE`
- `MOTTLE` now exists at `0x14`
- the builder/runtime authoring model is sequential 8-slot authored order, not legacy `0..3 bounds / 4..7 features`

### Current capability summary

Strengths:
- ambient and reflection shaping
- indoor projection architecture
- local luminous frames and bays
- stylized world-integrated mood fields
- support breakup for flat fills

Reliable primary movers today:
- `FLOW`
- `GRID`
- `LOBE`
- `DECAL`
- `VEIL/RAIN_WALL`
- `PLANE/WATER`
- `PORTAL/VORTEX`
- `ADVECT`

Important practical limits today:
- `PORTAL/RECT` is a static local frame
- `TRACE/LIGHTNING` is a static strike silhouette, not cadence
- `SCATTER` is seed-driven shimmer, not smooth transport
- `APERTURE` is a bounds remapper, not a feature carrier
- `BAND` phase is accent modulation, not a general travel carrier
- `MOTTLE` is anti-flat-fill support, not a hero structure or motion carrier

### Domain reality

`DIRECT3D`
- strongest default starting point for world-space carriers

`AXIS_CYL`
- strong for shafts, curtains, wrapped sweeps
- weaker for clean far-field planar weather fronts

`AXIS_POLAR`
- strong for radial/orbital reads
- weaker for planar travel and horizon-led scenes

`TANGENT_LOCAL`
- strong for local anchored frames and projectors
- weak for far-field ocean, skyline, and weather reads

## Evidence From The Showcase Loops

Determinism is no longer the blocker.

Confirmed:
- paired real-player runs reached `0` mismatches on the latest authoritative baseline
- `Combat Lab` passed on run29
- `Frozen Tundra` and `Storm Front` still fail for stable, code-matching reasons

This matters because it changes the interpretation of repeated failures:
- we are not looking at random capture drift
- we are not looking at an obviously hidden already-implemented opcode family
- we are mostly looking at honest capability ceilings plus some remaining authoring misses

### Proven positive signal

`Combat Lab` proves EPU is not generally broken.

What it proves:
- indoor direct-view scenes can pass
- room-integrated projection architecture is within the current surface
- current motion carriers can survive review when matched to the right scene class

### Proven outdoor failure classes

`Frozen Tundra`
- improved slightly after `MOTTLE`
- still reads as a pale ice basin / amphitheater
- the first `ADVECT/SPINDRIFT` pass largely disappears into the pale scene rather than surfacing a clear mover
- still lacks a hard glacier ridge and convincing open-air place read
- still lacks review-visible snow or sheen motion

`Storm Front`
- the old `VEIL/RAIN_WALL` experiments produced three coherent failure modes across domains:
- `DIRECT3D`: giant side-arc fans
- `TANGENT_LOCAL`: too hidden in direct view
- `AXIS_CYL`: visible, but as giant geometric pillar slabs
- the first `ADVECT/SQUALL` pass is materially better than all three, but it still resolves as large translucent slabs / ceiling panels rather than a natural squall front

Meaning:
- this is no longer just preset churn
- adding a broad transport family was the right direction
- the remaining gap is now refinement of that transport family rather than rediscovery of whether it is needed

## Capability Audit

### 1. Outdoor far-field structure

Status:
- `partial`

What exists:
- `SILHOUETTE`
- `SPLIT`
- new `SPLIT/TIER`
- new `SPLIT/FACE`

What improved:
- `TIER` and `FACE` reduce some bowl-like generic structure

What is still missing:
- a stronger open-air far-field organizer that can hold ridge, shelf, or storm-mass structure without collapsing into the generic wall belt
- richer structural separation than one ridge plus one dominant wall read

Conclusion:
- structure is better than before, but still not fully sufficient for outdoor proof-of-life on its own

### 2. Broad directional advection / weather-mass motion

Status:
- `partial`

What exists:
- `FLOW`
- `VEIL/RAIN_WALL`
- `SCATTER` support particles
- `ADVECT`
- `MASS`

Why that is insufficient:
- `FLOW` is pattern/advection, not a broad density slab
- `VEIL` is still fundamentally ribbon/bar vocabulary
- `SCATTER` is not a smooth primary mover
- the first `ADVECT` pass proves the new family can carry direct-view storm motion, but its shaping still reads too architectural in the hard-weather case and too weak in the pale spindrift case

Conclusion:
- the missing weather-body family is no longer hypothetical; it now exists as `MASS`
- the active weather architecture should be `MASS` for body plus `ADVECT` for subordinate motion
- the remaining work is improving occupancy, contrast, and scene ownership in that combined path instead of pretending transport alone is enough

### 3. Anti-flat-fill / macro breakup

Status:
- `present`, but still support-scoped

What exists:
- local warp and noise in several existing carriers
- `MOTTLE` as explicit broad breakup support

Evidence:
- `Frozen Tundra` improved slightly once `MOTTLE` was added
- the flat shelf problem weakened, but did not become a complete scene solution

Conclusion:
- the system now has a real support answer for broad flat color
- current evidence does not justify another generic noise opcode
- the remaining misses are weather shaping and frozen-surface identity, not absence of breakup itself

### 4. Ice-specific material response

Status:
- `partial`

What exists:
- `PLANE/WATER`
- other `PLANE` material variants
- `SURFACE`

Problem:
- `PLANE` is already full on variant slots
- authors still have to fake ice with `WATER`
- `Frozen Tundra` keeps showing why that is a compromise rather than a real answer

Conclusion:
- the separate material-response family is now landed
- the remaining work is proving that `SURFACE` actually closes the frozen-material benchmark/showcase gap at shipping quality

### 5. Capability semantics in tooling

Status:
- `partial`

What exists:
- runtime-facing docs are now aligned to `bounds` and `features`
- `nethercore-zx/src/debug/epu_capabilities.rs` captures practical authoring truth

What still drifts:
- generated metadata still uses internal `Radiance` naming
- some older cross-repo catalog sections still carry legacy terminology
- capability truth still lives partly in docs, partly in Rust tables, and only indirectly in shader metadata

Conclusion:
- tooling is better, but the capability contract is still not first-class enough

## Gap Classification

### Real surface gaps

These are the current evidence-backed missing behaviors:

1. Better shaping for the combined `MASS + ADVECT` weather-body surface
2. Better outdoor far-field structural control
3. Ice or frozen-sheet material response distinct from water
4. Stronger first-class capability semantics in metadata/tooling

### Content problems that still exist

These are not automatically engine or surface gaps:
- some presets still depend on weak carrier choice
- some animation amplitudes are too subtle even when the carrier is valid
- some scenes still collapse because structure and motion responsibilities are assigned to the wrong layers

### Things that do not currently look like hidden bugs

The following failures match code behavior closely enough that they should not be treated as primary bug suspects:
- `VEIL/RAIN_WALL` domain failure modes in `Storm Front`
- `SCATTER` failing as a primary mover
- `TRACE/LIGHTNING` failing as cadence
- `PORTAL/RECT` failing as a mover
- `BAND` failing as a general horizon scroller

## Next-Surface Spec

## Priority 1: Refine the combined weather-body path

Type:
- existing feature opcodes at `0x15` and `0x17`

Working names:
- `ADVECT`
- `MASS`

Reason:
- `ADVECT` alone was not enough
- `MASS` alone was not enough
- deterministic benchmark and showcase evidence now show the correct architecture is `MASS` for scene-owning body plus `ADVECT` for subordinate motion
- the remaining miss is body occupancy, contrast, and scene ownership in direct view

Behavior contract:
- `MASS` provides the coherent scene-owning body or shelf
- `ADVECT` provides readable scene-scale motion through or across that body
- together they must support fog banks, squall fronts, spindrift, ash drift, magical veils, and other metaphorical weather/front reads
- they must not degrade into thin bars, arc fans, local projector slabs, or pale non-owning shells

Active variants:
- `ADVECT/SHEET`
- `ADVECT/SPINDRIFT`
- `ADVECT/SQUALL`
- `ADVECT/MIST`
- `ADVECT/BANK`
- `ADVECT/FRONT`
- `MASS/BANK`
- `MASS/SHELF`
- `MASS/PLUME`
- `MASS/VEIL`

Proposed domains:
- `DIRECT3D`
- `AXIS_CYL`
- optional `AXIS_POLAR`

Do not require `TANGENT_LOCAL` in the current refinement pass.

Proposed fields:
- `intensity = brightness / density`
- `param_a = scale`
- `param_b = coverage`
- `param_c = breakup`
- `param_d = phase`
- `direction = prevailing travel axis`

Design rules:
- prioritize large low-frequency transport over micro detail
- make `param_c` the first-class macro-breakup control
- preserve deterministic explicit motion

Current validation targets:
- `Frozen Tundra` via `SURFACE` plus outdoor structure/motion support
- `Storm Front` via `MASS + ADVECT`

Acceptance target:
- `Storm Front`: three review frames must show one scene-owning storm/front body with obvious subordinate motion, without slab, pillar, or ceiling-panel reads
- `Frozen Tundra`: the frozen-scene proof must survive as an outdoor open-air read with non-water floor identity and clear support motion

## Priority 2: Prove the landed material-response family

Type:
- existing feature opcode at `0x16`

Working name:
- `SURFACE`

Reason:
- `PLANE` is full
- `Frozen Tundra` still needs proof that frozen-sheet reads no longer collapse into water behavior

Behavior contract:
- material-response carrier for broad ground/surface identity
- should support frozen sheen and crust behavior without reading as liquid water

Current variants:
- `ICE`
- `CRUST`
- `GLAZE`
- `FLAT`

Proposed fields:
- `intensity = contrast`
- `param_a = scale`
- `param_b = fracture`
- `param_c = sheen`
- `param_d = phase`
- `direction = surface normal / preferred response axis`

Primary validation target:
- `Frozen Tundra`

Acceptance target:
- a direct-view frozen floor that does not read as water and does not collapse into tiled scratch bands

## Priority 3: Improve capability semantics

Type:
- tooling and metadata work

Do next:
- keep `nethercore-zx/src/debug/epu_capabilities.rs` as the current canonical capability table
- add entries for the new feature families when they land
- continue exposing warnings in debug UI and inspector

Do later:
- normalize generated metadata terminology from internal `Radiance` naming toward `Feature`
- push capability tags closer to shader metadata if the surface stabilizes

## Priority 4: Re-evaluate whether a new bounds carrier is still needed

Do not add a new bounds opcode first.

Reason:
- `TIER` and `FACE` just landed
- the biggest currently demonstrated miss is transport, not bounds dispatch itself
- a new bounds namespace change is more invasive than a new feature opcode

Revisit only after:
- the `MASS + ADVECT` weather path stabilizes
- `SURFACE` is re-reviewed on the frozen-material lane
- outdoor proof-of-life is re-reviewed

## Implementation Surface

Likely refinement surface for `ADVECT` and `MASS`:
- `nethercore-zx/shaders/epu/epu_common.wgsl`
- `nethercore-zx/shaders/epu/epu_dispatch.wgsl`
- `nethercore-zx/shaders/epu/features/13_advect.wgsl`
- `nethercore-zx/src/graphics/epu/layer.rs`
- `nethercore-zx/src/graphics/epu/shaders.rs`
- `nethercore-zx/src/graphics/epu/params.rs`
- `nethercore-zx/src/graphics/epu/builder.rs`
- `nethercore-zx/src/debug/epu_capabilities.rs`
- generated metadata surfaces
- showcase constants and presets 11-12

Current `SURFACE` surface:
- same general surface, with a second WGSL feature file at `0x16`

Likely showcase files:
- `examples/3-inspectors/epu-showcase/src/constants.rs`
- `examples/3-inspectors/epu-showcase/src/lib.rs`
- `examples/3-inspectors/epu-showcase/src/presets.rs`
- `examples/3-inspectors/epu-showcase/src/presets/set_11_12.rs`

## Validation Plan

Compile and tooling checks:
- `cargo test -p nethercore-zx --lib`
- `cargo test -p nethercore-zx epu_capabilities --lib`
- `cargo check --target wasm32-unknown-unknown` in `examples/3-inspectors/epu-inspector`
- `cargo run -p nether-cli -- build --project examples/3-inspectors/epu-showcase`
- `cargo run -p nether-cli -- replay validate examples/3-inspectors/epu-showcase/screenshot-all-anim3.ncrs`

Deterministic proof:
- paired real-player capture run
- `0` hash mismatches

Review targets:
- `Frozen Tundra`
- `Storm Front`

Success criteria:
- `Frozen Tundra` shows a convincing open-air frozen field with obvious direct-view support motion and non-water floor identity
- `Storm Front` shows a convincing world-space storm/front body with subordinate motion, without arc fans, hidden local sheets, cylindrical pillar bars, or broad slab/panel artifacts

## Recommended Build Order

1. Refine the `MASS + ADVECT` path so `Storm Front` stops reading as a pale or synthetic non-owning body.
2. Re-run the `Front Mass` benchmark and only then re-promote to `Storm Front`.
3. Re-run the `Frozen Bed` benchmark to prove whether `SURFACE` materially closes frozen-material identity.
4. Update capability metadata/tooling alongside each landing.
5. Re-run deterministic capture and adversarial review.

Status after the later body/material follow-up:
- `ADVECT/FRONT` exists as the narrower front-motion path inside `ADVECT`.
- `SURFACE` exists at `0x16` and is now the active frozen-material lane.
- `MASS` exists at `0x17` and is now the active body-carrier lane.
- The current benchmark/showcase evidence says `MASS + ADVECT` is the correct architecture for `Storm Front`, but the preset still fails because the body stays too pale/soft and does not fully own the scene.
- `Frozen Tundra` remains blocked even after `SURFACE` landed, which means material response alone is not enough; outdoor structure and open-air readability still need more proof.
- The newer outdoor rig-profile rerun closes one more process theory: even with a fairer pullback and slightly smaller reflective probe, both `Front Mass` and `Frozen Bed` still fail. That means the remaining blocker is still primarily surface / scene-vocabulary quality, not just the shared showcase camera rig.

Latest implication:
- the next credible work may need to revisit stronger outdoor far-field structure or segmentation sooner than earlier drafts assumed
- `MASS + ADVECT` remains the right weather-body architecture, but it is no longer enough by itself
- `SURFACE` remains necessary for frozen material identity, but it is no longer enough by itself either
- decoupling bounds paint from bounds structural authority materially improved both active outdoor benchmarks by reducing giant flat organizer bands without weakening region retagging

## Bottom Line

The current audit says:
- EPU is validated enough to trust the loop
- the system is not worthless
- the next blocker is no longer vague

The highest-value missing surface work is:
- refining the combined `MASS + ADVECT` weather-body path until it reads cleanly and forcefully in direct view

The second highest-value missing surface is:
- proving whether the landed `SURFACE` family plus current outdoor structure can actually close frozen-scene proof-of-life

That is the smallest credible path from "strong indoor/reflection mood system" to "real outdoor cubemap replacement candidate."
