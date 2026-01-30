# EPU Bounds Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make every bounds opcode define sky/wall/floor regions and a direction without relying on a shared enclosure, so any bounds can be layer 0 and region semantics are consistent.

**Architecture:** Replace `EnclosureConfig` with a `bounds_dir` state and per-bounds region computation. RAMP decodes its thresholds locally and outputs regions directly. Other bounds compute a signed-distance triplet (sky/opening, wall/edge, floor/solid), blending any existing strength parameters by redistributing sky into wall or by lerping toward an all-sky baseline. The runtime tracks `bounds_dir` and `RegionWeights`, and features consume region weights plus the current `bounds_dir` (ATMOSPHERE uses it as up). Update docs, FFI comments, and builder/preset notes to shift terminology from "enclosure" to "bounds" and adjust presets for the new floor behavior.

**Tech Stack:** WGSL shaders, Rust (EPU builder + presets), mdBook docs, replay screenshot scripts.

---

## Validation Snapshot (Current Behavior)

- RAMP is the only bounds opcode that defines floor; other bounds preserve `base_regions.floor` and only split sky vs wall (see `nethercore-zx/shaders/epu/bounds/01_sector.wgsl`, `02_silhouette.wgsl`, `04_cell.wgsl`, `05_patches.wgsl`, `06_aperture.wgsl`).
- `enclosure_from_layer` keeps `ceil_y/floor_y/soft` for non-RAMP bounds and returns `prev_enc` for APERTURE (see `nethercore-zx/shaders/epu/epu_common.wgsl`).
- Bounds ignore the region mask bits; only feature opcodes apply `instr_region` (see `nethercore-zx/shaders/epu/epu_dispatch.wgsl`).
- Default regions are seeded from a synthetic RAMP in `epu_compute_env.wgsl` and `common/20_environment/90_sampling.wgsl`.

---

## Pre-Flight

### Task 0: Create an isolated worktree

**Why:** This touches WGSL + docs + Rust helpers/presets and will likely require several iterations.

**Steps**

1. Create worktree (use superpowers:using-git-worktrees)

2. Verify the new worktree is clean

Run: `git status`

Expected: clean working tree.

---

## Phase 1: Core Shader State (Remove EnclosureConfig)

### Task 1: Replace global enclosure with bounds direction + regions

**Files:**
- Modify: `nethercore-zx/shaders/epu/epu_common.wgsl`
- Modify: `nethercore-zx/shaders/epu/epu_dispatch.wgsl`
- Modify: `nethercore-zx/shaders/epu/epu_compute_env.wgsl`
- Modify: `nethercore-zx/shaders/common/20_environment/90_sampling.wgsl`
- Modify: `nethercore-zx/shaders/epu/features/06_atmosphere.wgsl`

**Step 1: Add a bounds-direction helper and remove EnclosureConfig**

In `nethercore-zx/shaders/epu/epu_common.wgsl`, replace `EnclosureConfig` + `enclosure_from_layer` with:

```wgsl
fn bounds_dir_from_layer(instr: vec4u, opcode: u32, prev_dir: vec3f) -> vec3f {
    switch opcode {
        case OP_RAMP, OP_SECTOR, OP_SILHOUETTE, OP_SPLIT, OP_CELL, OP_PATCHES, OP_APERTURE: {
            return decode_dir16(instr_dir16(instr));
        }
        default: { return prev_dir; }
    }
}
```

Remove `struct EnclosureConfig` and `enclosure_from_layer` from this file.

**Step 2: Seed default direction + regions**

In `nethercore-zx/shaders/epu/epu_compute_env.wgsl`, replace `enc` with `bounds_dir` and seed regions:

```wgsl
var bounds_dir = vec3f(0.0, 1.0, 0.0);
var regions = RegionWeights(1.0, 0.0, 0.0);
```

(If we want to preserve current fallback behavior, call a new `ramp_region_weights(dir, bounds_dir, 0.5, -0.5, 0.1)` instead.)

**Step 3: Update bounds evaluation signature**

In `epu_dispatch.wgsl`, update signatures and call sites:

```wgsl
fn evaluate_bounds_layer(
    dir: vec3f,
    instr: vec4u,
    opcode: u32,
    bounds_dir: vec3f,
    base_regions: RegionWeights
) -> BoundsResult
```

```wgsl
bounds_dir = bounds_dir_from_layer(instr, opcode, bounds_dir);
let bounds_result = evaluate_bounds_layer(dir, instr, opcode, bounds_dir, regions);
regions = bounds_result.regions;
```

**Step 4: Update epu_eval_hi**

Mirror Step 2 and Step 3 in `nethercore-zx/shaders/common/20_environment/90_sampling.wgsl` so procedural evaluation matches compute.

**Step 5: Update ATMOSPHERE**

In `nethercore-zx/shaders/epu/features/06_atmosphere.wgsl`, replace `enc` with `bounds_dir`:

```wgsl
fn eval_atmosphere(dir: vec3f, instr: vec4u, bounds_dir: vec3f, region_w: f32) -> LayerSample {
    let up = bounds_dir;
    ...
}
```

**Step 6: Run a smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

Expected: app boots; no WGSL validation errors.

**Step 7: Commit**

```bash
git add nethercore-zx/shaders/epu/epu_common.wgsl nethercore-zx/shaders/epu/epu_dispatch.wgsl nethercore-zx/shaders/epu/epu_compute_env.wgsl nethercore-zx/shaders/common/20_environment/90_sampling.wgsl nethercore-zx/shaders/epu/features/06_atmosphere.wgsl
git commit -m "refactor(epu): track bounds direction instead of enclosure"
```

---

## Phase 2: Bounds Define Full Regions

### Task 2: Add a shared signed-distance -> RegionWeights helper

**Files:**
- Modify: `nethercore-zx/shaders/epu/epu_common.wgsl`

**Step 1: Add helper**

```wgsl
fn regions_from_signed_distance(d: f32, bw: f32) -> RegionWeights {
    let w_sky = smoothstep(0.0, bw, -d);
    let w_floor = smoothstep(0.0, bw, d);
    let w_wall = max(0.0, 1.0 - w_sky - w_floor);
    return RegionWeights(w_sky, w_wall, w_floor);
}
```

**Step 2: Commit**

```bash
git add nethercore-zx/shaders/epu/epu_common.wgsl
git commit -m "feat(epu): add signed-distance region helper"
```

### Task 3: Update RAMP to compute regions internally

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/00_ramp.wgsl`
- Modify: `nethercore-zx/shaders/epu/epu_dispatch.wgsl` (RAMP branch)

**Step 1: Inline threshold decode + region computation**

Add this inside `eval_ramp`:

```wgsl
let up = decode_dir16(instr_dir16(instr));
let pd = instr_d(instr);
let ceil_q = (pd >> 4u) & 0xFu;
let floor_q = pd & 0xFu;
let soft = mix(0.01, 0.5, u8_to_01(instr_intensity(instr)));
var ceil_y = nibble_to_signed_1(ceil_q);
var floor_y = nibble_to_signed_1(floor_q);
if floor_y > ceil_y { let t = floor_y; floor_y = ceil_y; ceil_y = t; }

let y = dot(dir, up);
let w_sky = smoothstep(ceil_y - soft, ceil_y + soft, y);
let w_floor = smoothstep(floor_y + soft, floor_y - soft, y);
let w_wall = max(0.0, 1.0 - w_sky - w_floor);
let regions = RegionWeights(w_sky, w_wall, w_floor);
```

Return `regions` from the RAMP branch in `evaluate_bounds_layer`.

**Step 2: Smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

Expected: RAMP-only presets still show 3 regions.

**Step 3: Commit**

```bash
git add nethercore-zx/shaders/epu/bounds/00_ramp.wgsl nethercore-zx/shaders/epu/epu_dispatch.wgsl
git commit -m "feat(epu): make RAMP compute its own regions"
```

### Task 4: SECTOR outputs sky/wall/floor from wedge geometry

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/01_sector.wgsl`

**Step 1: Replace base_regions logic with signed-distance regions**

After computing `dist` and `half_width`, add:

```wgsl
let d = dist - half_width;
let bw = max(0.01, half_width * 0.25);
let regions_open = regions_from_signed_distance(d, bw);

// Blend "no opening" -> "full opening" using intensity.
let regions_closed = RegionWeights(0.0, 1.0 - regions_open.floor, regions_open.floor);
let regions = RegionWeights(
    mix(regions_closed.sky, regions_open.sky, intensity),
    mix(regions_closed.wall, regions_open.wall, intensity),
    regions_open.floor
);
```

Use `regions` for both output and color blend (keep floor color as darkened wall):

```wgsl
let floor_color = wall_color * 0.5;
let rgb = sky_color * regions.sky + wall_color * regions.wall + floor_color * regions.floor;
```

**Step 2: Smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

Expected: SECTOR-only presets show a 3-band sky/wall/floor split.

**Step 3: Commit**

```bash
git add nethercore-zx/shaders/epu/bounds/01_sector.wgsl
git commit -m "feat(epu): make SECTOR define full regions"
```

### Task 5: SILHOUETTE outputs sky/wall/floor from horizon geometry

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/02_silhouette.wgsl`

**Step 1: Replace base_regions mutation**

After computing `h` and `y_equiv`, add:

```wgsl
let d = h - y_equiv; // negative above horizon (sky), positive below (floor)
let regions_full = regions_from_signed_distance(d, softness);

// Strength blends from all-sky -> full silhouette
let regions = RegionWeights(
    mix(1.0, regions_full.sky, strength),
    mix(0.0, regions_full.wall, strength),
    mix(0.0, regions_full.floor, strength)
);
```

Use `regions` for output and color blend; keep existing `effect` for the visual mix or replace with `regions.wall`.

**Step 2: Smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

**Step 3: Commit**

```bash
git add nethercore-zx/shaders/epu/bounds/02_silhouette.wgsl
git commit -m "feat(epu): make SILHOUETTE define full regions"
```

### Task 6: SPLIT outputs 3 regions for all variants

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/03_split.wgsl`

**Step 1: Add a helper for planar bands**

```wgsl
fn split_band(d: f32, bw: f32) -> RegionWeights {
    return regions_from_signed_distance(-d, bw); // sky on positive side
}
```

**Step 2: Update HALF/WEDGE/BANDS/CROSS to use bands**

Example for HALF:

```wgsl
let d = dot(dir, n0);
let regions = split_band(d, bw);
```

For WEDGE:

```wgsl
let inside = min(dot(dir, n0), -dot(dir, n1));
let regions = regions_from_signed_distance(-inside, bw);
```

For BANDS:

```wgsl
let stripe = fract((dot(dir, n0) * 0.5 + 0.5 + band_offset) * band_count);
let d = stripe - 0.5;
let regions = regions_from_signed_distance(d, bw * band_count);
```

For CROSS:

```wgsl
let d = -dot(dir, n0) * dot(dir, basis[0]);
let regions = regions_from_signed_distance(d, bw);
```

Keep CORNER and PRISM using their existing 3-region logic.

**Step 3: Update color blend**

Use `regions` and keep floor color as `wall_color * 0.5` for variants without explicit floor color.

**Step 4: Smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

**Step 5: Commit**

```bash
git add nethercore-zx/shaders/epu/bounds/03_split.wgsl
git commit -m "feat(epu): make SPLIT variants output full regions"
```

### Task 7: CELL outputs sky/wall/floor from gaps/edges/interior

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/04_cell.wgsl`

**Step 1: Replace base_regions floor preservation**

After computing `d_edge` and `is_solid`, use the signed-distance helper:

```wgsl
if !is_solid {
    let regions = RegionWeights(1.0, 0.0, 0.0);
    ...
} else {
    let d = d_edge - gap_width; // negative in gap
    let regions = regions_from_signed_distance(d, max(0.005, gap_width * 0.5));
    ...
}
```

Use `regions` as output (no `base_regions`).

**Step 2: Keep radiance weights as-is**

Continue using `gap_alpha` and `outline` for visuals; use `regions` only for the bounds output.

**Step 3: Smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

**Step 4: Commit**

```bash
git add nethercore-zx/shaders/epu/bounds/04_cell.wgsl
git commit -m "feat(epu): make CELL define full regions"
```

### Task 8: PATCHES outputs sky/wall/floor from noise

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/05_patches.wgsl`

**Step 1: Compute regions from noise threshold**

After `noise_01` and `threshold`:

```wgsl
var d = noise_01 - threshold;
if variant == 3u { // MEMBRANE flips inside/outside
    d = -d;
}
let regions = regions_from_signed_distance(d, bw);
```

Use `regions` for output and keep radiance weights based on `sky_alpha` / `wall_alpha`.

**Step 2: Smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

**Step 3: Commit**

```bash
git add nethercore-zx/shaders/epu/bounds/05_patches.wgsl
git commit -m "feat(epu): make PATCHES define full regions"
```

### Task 9: APERTURE outputs sky/wall/floor from opening/frame/outside

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/06_aperture.wgsl`

**Step 1: Replace baseline mixing with direct regions**

Use the existing SDF bands:

```wgsl
let opening_w = smoothstep(softness, -softness, sdf);
let frame_inner = smoothstep(-softness, softness, sdf);
let frame_outer = smoothstep(softness, -softness, sdf - frame_thickness);
let wall_w = frame_inner * frame_outer;
let floor_w = clamp(1.0 - opening_w - wall_w, 0.0, 1.0);

let regions_front = RegionWeights(opening_w, wall_w, floor_w);
let regions = mix(RegionWeights(1.0, 0.0, 0.0), regions_front, front_w);
```

(If we want full replace even on the back hemisphere, drop the `mix` and clamp `front_w` into the SDF evaluation instead.)

**Step 2: Update output regions and sample**

Use `regions` for output; keep `zone_w` and `zone_rgb` logic for the sample.

**Step 3: Smoke test**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

**Step 4: Commit**

```bash
git add nethercore-zx/shaders/epu/bounds/06_aperture.wgsl
git commit -m "feat(epu): make APERTURE define full regions"
```

---

## Decision Gate: Composite Mode (Optional)

If we still want bounds to "carve into" prior regions:

1. Decide how to encode the composite flag for bounds (see Open Questions).
2. Add `bounds_flags = instr_region(instr)` and a helper in `epu_common.wgsl` to interpret flags.
3. For composite bounds, blend `regions` with `base_regions` (for example, `regions = normalize(regions * base_regions)` for intersection or a carve that preserves `base_regions` outside the bounds geometry).
4. Update Rust packing helpers to set the new bounds flags when desired.

---

## Phase 3: Docs + API Terminology ("Enclosure" -> "Bounds")

### Task 10: Update Rust docs and FFI comments

**Files:**
- Modify: `nethercore-zx/src/graphics/epu/mod.rs`
- Modify: `nethercore-zx/src/resource_manager.rs`
- Modify: `include/zx/epu.rs`
- Modify: `examples/3-inspectors/epu-showcase/src/constants.rs` (opcode comments)

**Step 1: Rename wording in docs**

Example in `nethercore-zx/src/graphics/epu/mod.rs`:

```rust
/// - 0x01..=0x07: Bounds ops
```

Update method docs to say "bounds" rather than "enclosure".

**Step 2: If renaming API, add new aliases**

If we want to keep API stable, add new methods that call the existing ones:

```rust
pub fn ramp_bounds(&mut self, p: RampParams) { self.ramp_enclosure(p); }
```

and mark `*_enclosure` as deprecated in doc comments.

**Step 3: Run Rust tests**

Run: `cargo test -p nethercore-zx --lib epu`

Expected: builder tests still pass.

**Step 4: Commit**

```bash
git add nethercore-zx/src/graphics/epu/mod.rs nethercore-zx/src/resource_manager.rs include/zx/epu.rs examples/3-inspectors/epu-showcase/src/constants.rs
git commit -m "docs(epu): shift terminology to bounds"
```

### Task 11: Update mdBook docs

**Files:**
- Modify: `docs/book/src/api/epu.md`
- Modify: `docs/book/src/architecture/epu-overview.md`
- Modify: `docs/book/src/guides/epu-environments.md`

**Step 1: Replace "enclosure" phrasing**

Examples:

- "Enclosure (bounds)" -> "Bounds"
- "RAMP + optional enclosure ops" -> "Bounds ops (0x01..0x07)"
- Explain that bounds output direction + region weights, and do not use the region mask.

**Step 2: Build docs (optional)**

Run: `cd docs/book; mdbook serve`

Expected: pages render without broken links.

**Step 3: Commit**

```bash
git add docs/book/src/api/epu.md docs/book/src/architecture/epu-overview.md docs/book/src/guides/epu-environments.md
git commit -m "docs(book): describe bounds-only architecture"
```

---

## Phase 4: Preset Retune + Visual Verification

### Task 12: Update showcase presets to align with new floor semantics

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/*.rs`
- Modify (if needed): `examples/3-inspectors/epu-showcase/src/constants.rs`

**Step 1: Rebuild presets that rely on RAMP floor**

Focus on presets that use SECTOR/SILHOUETTE/CELL/PATCHES/APERTURE after RAMP; verify floor region still matches the intended look. Adjust colors or order if necessary.

**Step 2: Run screenshot replay**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-all.ncrs`

Expected: 24 screenshots generated under `%APPDATA%/Nethercore/data/screenshots/` with consistent sky/wall/floor separation.

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets
git commit -m "chore(epu-showcase): retune presets for new bounds regions"
```

---

## Cross-Repo Follow-Ups

- Update `nethercore-design/specs/epu-feature-catalog.md` to describe the new bounds semantics (separate repo).
- If `nethercore-platform` references enclosure docs, update those after shader changes land.

---

## Open Questions

1. Composite mode encoding: Should we implement composite now, and if so which flag encoding should bounds use without breaking existing presets that set `REGION_ALL`?
2. Default regions before first bounds: Keep the current synthetic RAMP fallback or default to all-sky?
3. API naming: Should we rename Rust API methods (`*_enclosure`) or keep names and only update docs?
