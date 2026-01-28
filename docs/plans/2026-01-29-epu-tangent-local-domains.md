# EPU Tangent-Local Domains Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Make precipitation (especially rain) render cleanly by adding `DOMAIN_TANGENT_LOCAL` support to the `VEIL` opcode and fixing `VEIL_RAIN_WALL` so it does not produce spherical band/ring artifacts.

**Architecture:** Extend `meta5.domain_id` support in the WGSL shader for `VEIL` (opcode `0x0D`) to include `domain_id=3` (tangent-local / gnomonic projection anchored at `direction`). Rework `VEIL_RAIN_WALL` to distribute streak segments along v using deterministic per-bar offsets plus `param_d` phase, avoiding fixed-v “rings”. Update EPU showcase presets to use the new domain for storm presets and verify via automated screenshot replay.

**Tech Stack:** WGSL (EPU compute shaders), Rust (showcase presets + packing helpers), Nethercore player + replay scripts.

---

## Pre-Flight

### Task 0: Create an isolated worktree

**Why:** This touches shader code + showcase presets and will likely need several iterations.

**Steps**

1. Create worktree (use superpowers:using-git-worktrees)

2. Verify the new worktree is clean

Run: `git status`

Expected: clean working tree.

---

## Phase 1 (High Impact): VEIL Tangent-Local + Rain Wall Fix

### Task 1: Add `VEIL` tangent-local domain (domain_id=3)

**Files:**
- Modify: `nethercore-zx/shaders/epu/features/05_veil.wgsl`
- (Docs) Modify: `examples/3-inspectors/epu-showcase/src/constants.rs`

**Step 1: Add a new domain constant**

In `nethercore-zx/shaders/epu/features/05_veil.wgsl`, add:

```wgsl
const VEIL_DOMAIN_AXIS_CYL: u32 = 1u;
const VEIL_DOMAIN_AXIS_POLAR: u32 = 2u;
const VEIL_DOMAIN_TANGENT_LOCAL: u32 = 3u;
```

**Step 2: Add tangent-local UV helper**

Add a helper modeled after `trace_tangent_uv` (same gnomonic projection):

```wgsl
fn veil_tangent_uv(dir: vec3f, center: vec3f) -> vec3f {
    // Returns (u, v, visibility_weight)
    let d = dot(dir, center);
    if d <= 0.0 {
        return vec3f(0.0, 0.0, 0.0);
    }

    let up = select(vec3f(0.0, 1.0, 0.0), vec3f(1.0, 0.0, 0.0), abs(center.y) > 0.9);
    let t = normalize(cross(up, center));
    let b = normalize(cross(center, t));

    let proj = dir - center * d;
    let u = dot(proj, t) / d;
    let v = dot(proj, b) / d;

    // Grazing fade (prevents hard edge artifacts near 90 degrees)
    let grazing_w = smoothstep(0.1, 0.3, d);
    return vec3f(u, v, grazing_w);
}
```

**Step 3: Wire domain 3 into `eval_veil`**

In the `switch domain_id` inside `eval_veil`, add a new case:

```wgsl
case VEIL_DOMAIN_TANGENT_LOCAL: {
    let result = veil_tangent_uv(dir, axis);
    let tuv = result.xy;
    domain_w = result.z;

    // Map tangent plane coords to VEIL’s expected ranges:
    // - u in [0,1] (ribbon centers are laid out over u)
    // - v in [-1,1] (matches cyl height conventions)
    uv = vec2f(tuv.x * 0.5 + 0.5, clamp(tuv.y, -1.0, 1.0));
}
```

**Step 4: Update Rust-side docs for VEIL domains**

In `examples/3-inspectors/epu-showcase/src/constants.rs`, update the VEIL comment to:

```rust
/// VEIL - Curtain/ribbon effects
/// Domains: 1=AXIS_CYL, 2=AXIS_POLAR, 3=TANGENT_LOCAL
/// Variants: 0=CURTAINS, 1=PILLARS, 2=LASER_BARS, 3=RAIN_WALL, 4=SHARDS
pub const OP_VEIL: u64 = 0x0D;
```

**Step 5: Build + smoke test shader compilation**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

Expected: game boots; no WGSL validation errors.

**Step 6: Commit**

```bash
git add nethercore-zx/shaders/epu/features/05_veil.wgsl examples/3-inspectors/epu-showcase/src/constants.rs
git commit -m "feat(epu): add tangent-local domain for VEIL"
```

---

### Task 2: Fix `VEIL_RAIN_WALL` to avoid ring artifacts

**Files:**
- Modify: `nethercore-zx/shaders/epu/features/05_veil.wgsl`
- Modify (docs): `docs/book/src/api/epu.md`

**Intent:** The current implementation gates visibility with a fixed v-band (creates a ring). Also `fall_speed` is computed but unused.

**Step 1: Use `param_d` as a deterministic phase**

Inside `eval_veil`, decode:

```wgsl
let phase = u8_to_01(instr_d(instr));
```

**Step 2: Thread `phase` into the rain wall variant**

Change `eval_veil_rain_wall(...)` to accept a `phase: f32` parameter, and update the call site.

Example signature:

```wgsl
fn eval_veil_rain_wall(
    u: f32,
    v: f32,
    ribbon_count: u32,
    thickness: f32,
    curvature: f32,
    phase: f32,
) -> vec3f {
    // ...
}
```

**Step 3: Replace the fixed-v band logic**

Replace the current block that builds `v_offset`, `v_adjusted`, and `bar_visible` with a true per-bar segment position:

```wgsl
let v01 = v * 0.5 + 0.5; // v is expected to be [-1,1]

// Each bar has a slightly different fall speed and a deterministic starting offset.
let fall_speed = 0.3 + h.x * 1.4;
let drop_pos = fract(h.y + phase * fall_speed);

// Short streak segments (length in 0..1 v-space)
let half_len = 0.02 + h.z * 0.06;
let dv = abs(v01 - drop_pos);
let bar_visible = 1.0 - smoothstep(half_len, half_len * 1.6, dv);
```

Keep the “nearest bar” behavior (using min u-distance) but only allow bars with `bar_visible` above a small threshold (e.g. `> 0.05`).

**Step 4: (Optional) Use `curvature` to add wind slant**

For extra quality, apply a small diagonal offset so rain isn’t perfectly vertical:

```wgsl
let wind = (curvature - 0.5) * 0.15;
let d = ribbon_dist_wrapped(u_scrolled + v * wind, center_u);
```

**Step 5: Update docs to mention `VEIL.param_d`**

In `docs/book/src/api/epu.md`, add a short note near `meta5` or the opcode map:

- `VEIL` currently uses `param_d` as a deterministic phase for the `RAIN_WALL` variant.

**Step 6: Verify via player run**

Run: `cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx`

Expected: no shader compile errors.

**Step 7: Commit**

```bash
git add nethercore-zx/shaders/epu/features/05_veil.wgsl docs/book/src/api/epu.md
git commit -m "fix(epu): remove VEIL_RAIN_WALL ring artifact and add phase"
```

---

### Task 3: Use the new rain mode in the EPU showcase storm presets

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_09_12.rs`
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_21_24.rs`

**Step 1: Update Storm Front**

Replace the current “rain” layer with VEIL rain-wall in tangent-local domain.

Example layer (tune params during verification):

```rust
[
    hi_meta(
        OP_VEIL,
        REGION_SKY | REGION_WALLS,
        BLEND_SCREEN,
        DOMAIN_TANGENT_LOCAL,
        VEIL_RAIN_WALL,
        0x90a8c0,
        0x000000,
    ),
    // intensity, ribbon_count, thickness, curvature, phase, direction, alpha_a, alpha_b
    lo(180, 220, 80, 128, 64, DIR_FORWARD, 12, 6),
],
```

**Step 2: Update Stormy Shores**

Apply the same pattern; use a slightly brighter color and higher intensity.

**Step 3: Rebuild + regenerate screenshots**

Run:

```bash
nether build
```

Workdir: `examples/3-inspectors/epu-showcase`

Then from repo root:

```bash
cargo run -- examples/3-inspectors/epu-showcase/epu-showcase.nczx --replay examples/3-inspectors/epu-showcase/screenshot-all.ncrs
```

Expected: a new 24-screenshot set in `%APPDATA%/Nethercore/data/screenshots/` where presets 10 and 21 do not show large curved “bands/rings” for rain.

**Step 4: Iterate parameters until rain reads as rain**

Use `examples/3-inspectors/epu-showcase/screenshot-single.ncrs` for faster iteration on one preset at a time.

**Step 5: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_09_12.rs examples/3-inspectors/epu-showcase/src/presets/set_21_24.rs
git commit -m "chore(epu-showcase): use tangent-local VEIL rain for storm presets"
```

---

## Phase 2 (Optional): Apply tangent-local domains to more opcodes

### Task 4 (Optional): Add domain support to GRID

**Why:** `GRID` is currently hard-coded to world cylindrical UV, which can also produce wrap artifacts. Tangent-local grid enables local “monitor/HUD panels” cleanly.

**Files:**
- Modify: `nethercore-zx/shaders/epu/features/01_grid.wgsl`
- Modify: `docs/book/src/api/epu.md` (brief doc note)

**Implementation sketch:**
- Read `domain_id = instr_domain_id(instr)` and `center = decode_dir16(instr_dir16(instr))`.
- Add a `GRID_DOMAIN_TANGENT_LOCAL = 3` path using gnomonic UV (same as `trace_tangent_uv`).
- Keep `domain_id=0` behavior identical for backward compatibility.

**Verification:** run the player; ensure no visual regression on existing presets.

**Commit:** `feat(epu): add domain selection for GRID`

---

### Task 5 (Optional): Add domain 3 to PATCHES

**Warning:** `PATCHES` is a bounds opcode; changing its domain behavior impacts region weights for all subsequent feature ops. Only do this if we have a clear use-case (localized clouds, localized streak walls, etc.).

**Files:**
- Modify: `nethercore-zx/shaders/epu/bounds/05_patches.wgsl`

**Implementation sketch:**
- Add `case 3u` in the domain switch to compute tangent-local UV around `axis`.
- Multiply the output alphas/weights by a domain fade factor so the effect is localized and doesn’t hard-cut region weights.

**Verification:** build + run showcase, watch for region weight weirdness.

---

## Done Criteria

- `VEIL` supports `domain_id=3` without WGSL validation issues.
- `VEIL_RAIN_WALL` no longer produces a fixed-v ring/band artifact in axis domains.
- Showcase presets (at minimum Storm Front and Stormy Shores) can render rain that is unmistakably rain in a still screenshot without visible projection artifacts.

---

## Execution Handoff

Plan complete and saved to `docs/plans/2026-01-29-epu-tangent-local-domains.md`.

Two execution options:

1. Subagent-Driven (this session) - dispatch a fresh subagent per task, review between tasks
2. Parallel Session (separate) - open a new session with superpowers:executing-plans and batch execution with checkpoints

Which approach?
