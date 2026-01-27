# EPU Preset & Showcase Fixes Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Fix broken EPU showcase presets and multi-reflections example by correcting RAMP threshold encoding, fixing opcode labels, and tuning parameters so all 24 environments render correctly with visible sky/wall/floor region differentiation.

**Architecture:** Every EPU environment starts with a RAMP layer that defines sky/wall/floor region boundaries via `param_d` (packed ceiling + floor Y thresholds). All 28 presets currently pass `param_d=0`, collapsing all regions to sky. Fix by setting correct thresholds per-preset, then fix secondary issues (opcode labels, wall colors in multi-reflections, intensity tuning).

**Tech Stack:** Rust (no_std WASM targets), WGSL compute shaders (read-only reference), EPU 128-bit instruction format.

---

## Background: EPU RAMP param_d Encoding

The RAMP opcode's `param_d` byte packs two 4-bit values:
- High nibble: `ceil_q` (0-15) -> ceiling Y threshold via `(q/15)*2 - 1`
- Low nibble: `floor_q` (0-15) -> floor Y threshold via `(q/15)*2 - 1`

Common threshold presets:
| Hex | ceil_q | floor_q | ceil_y | floor_y | Character |
|-----|--------|---------|--------|---------|-----------|
| `0xA5` | 10 | 5 | +0.33 | -0.33 | Balanced outdoor |
| `0xC3` | 12 | 3 | +0.60 | -0.60 | Interior/enclosed (more wall) |
| `0x96` | 9 | 6 | +0.20 | -0.20 | Open sky (more sky, less wall) |
| `0x87` | 8 | 7 | +0.07 | -0.07 | Vast open (almost all sky+floor) |
| `0xB4` | 11 | 4 | +0.47 | -0.47 | Semi-enclosed |

The `lo()` helper signature: `lo(intensity, param_a, param_b, param_c, param_d, direction, alpha_a, alpha_b)`

For RAMP: `param_a` = wall R, `param_b` = wall G, `param_c` = wall B, `param_d` = thresholds, `intensity` = softness.

---

## Task 1: Add Threshold Constant to constants.rs

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/constants.rs` (after line 352)

**Step 1: Add threshold constants**

Add at the end of the file, before the closing:

```rust
// =============================================================================
// RAMP Threshold Presets (param_d encoding)
// =============================================================================

/// Balanced outdoor: ceil_q=10 (y=+0.33), floor_q=5 (y=-0.33)
pub const THRESH_BALANCED: u64 = 0xA5;
/// Interior/enclosed: ceil_q=12 (y=+0.60), floor_q=3 (y=-0.60) — more wall region
pub const THRESH_INTERIOR: u64 = 0xC3;
/// Open sky: ceil_q=9 (y=+0.20), floor_q=6 (y=-0.20) — more sky region
pub const THRESH_OPEN: u64 = 0x96;
/// Vast open: ceil_q=8 (y=+0.07), floor_q=7 (y=-0.07) — almost all sky+floor
pub const THRESH_VAST: u64 = 0x87;
/// Semi-enclosed: ceil_q=11 (y=+0.47), floor_q=4 (y=-0.47)
pub const THRESH_SEMI: u64 = 0xB4;
```

**Step 2: Build to verify**

Run: `cargo build -p epu-showcase`
Expected: Compiles with no errors.

**Step 3: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/constants.rs
git commit -m "epu-showcase: add RAMP threshold constants for region encoding"
```

---

## Task 2: Fix set_01_04.rs (Presets 1-4)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_01_04.rs`

**Step 1: Fix Preset 1 "Neon Metropolis" RAMP layer**

Change line 27 from:
```rust
        lo(255, 0x1c, 0x1c, 0x1c, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(220, 0x1c, 0x1c, 0x1c, THRESH_BALANCED, DIR_UP, 15, 15),
```
(Also reduce intensity from 255 to 220 for slightly sharper region transitions in an urban scene.)

**Step 2: Fix Preset 2 "Crimson Hellscape" RAMP layer**

Change line 121 from:
```rust
        lo(255, 0x2a, 0x08, 0x08, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(230, 0x2a, 0x08, 0x08, THRESH_BALANCED, DIR_UP, 15, 15),
```

**Step 3: Fix Preset 3 "Frozen Tundra" RAMP layer**

Change line 223 from:
```rust
        lo(255, 0xa0, 0xc8, 0xe0, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(240, 0xa0, 0xc8, 0xe0, THRESH_BALANCED, DIR_UP, 15, 15),
```

**Step 4: Fix Preset 4 "Alien Jungle" RAMP layer**

Change line 325 from:
```rust
        lo(255, 0x00, 0x40, 0x40, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(230, 0x00, 0x40, 0x40, THRESH_BALANCED, DIR_UP, 15, 15),
```

**Step 5: Build to verify**

Run: `cargo build -p epu-showcase`
Expected: Compiles with no errors.

**Step 6: Commit**

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_01_04.rs
git commit -m "epu-showcase: fix RAMP thresholds for presets 1-4"
```

---

## Task 3: Fix set_05_08.rs (Presets 5-8)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_05_08.rs`

**Step 1: Fix Preset 5 "Gothic Cathedral" — interior**

Change the RAMP lo line from:
```rust
        lo(255, 0x20, 0x20, 0x20, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(180, 0x20, 0x20, 0x20, THRESH_INTERIOR, DIR_UP, 15, 15),
```
(Low softness=180 for sharp stone boundaries.)

**Step 2: Fix Preset 6 "Ocean Depths" — balanced**

Change from:
```rust
        lo(255, 0x00, 0x28, 0x48, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(230, 0x00, 0x28, 0x48, THRESH_BALANCED, DIR_UP, 15, 15),
```

**Step 3: Fix Preset 7 "Void Station" — interior**

Change from:
```rust
        lo(255, 0x18, 0x18, 0x20, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(180, 0x18, 0x18, 0x20, THRESH_INTERIOR, DIR_UP, 15, 15),
```

**Step 4: Fix Preset 8 "Desert Mirage" — open**

Change from:
```rust
        lo(255, 0xc8, 0xa8, 0x78, 0, DIR_UP, 15, 15),
```
to:
```rust
        lo(240, 0xc8, 0xa8, 0x78, THRESH_OPEN, DIR_UP, 15, 15),
```

**Step 5: Build and commit**

Run: `cargo build -p epu-showcase`

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_05_08.rs
git commit -m "epu-showcase: fix RAMP thresholds for presets 5-8"
```

---

## Task 4: Fix set_09_12.rs (Presets 9-12)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_09_12.rs`

**Step 1: Fix all four RAMP layers**

| Preset | Name | Old lo | New lo |
|--------|------|--------|--------|
| 9 | Neon Arcade | `lo(255, 0x08, 0x00, 0x18, 0, ...)` | `lo(220, 0x08, 0x00, 0x18, THRESH_BALANCED, ...)` |
| 10 | Storm Front | `lo(255, 0x30, 0x38, 0x40, 0, ...)` | `lo(230, 0x30, 0x38, 0x40, THRESH_BALANCED, ...)` |
| 11 | Crystal Cavern | `lo(255, 0x18, 0x00, 0x30, 0, ...)` | `lo(180, 0x18, 0x00, 0x30, THRESH_INTERIOR, ...)` |
| 12 | War Zone | `lo(255, 0x30, 0x28, 0x20, 0, ...)` | `lo(220, 0x30, 0x28, 0x20, THRESH_BALANCED, ...)` |

**Step 2: Build and commit**

Run: `cargo build -p epu-showcase`

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_09_12.rs
git commit -m "epu-showcase: fix RAMP thresholds for presets 9-12"
```

---

## Task 5: Fix set_13_16.rs (Presets 13-16)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_13_16.rs`

**Step 1: Fix all four RAMP layers**

| Preset | Name | Threshold | Softness |
|--------|------|-----------|----------|
| 13 | Enchanted Grove | `THRESH_OPEN` | 240 |
| 14 | Astral Void | `THRESH_VAST` | 250 |
| 15 | Toxic Wasteland | `THRESH_BALANCED` | 220 |
| 16 | Moonlit Graveyard | `THRESH_BALANCED` | 220 |

Apply to each RAMP lo line: replace 5th arg `0` with threshold constant, replace 1st arg `255` with softness value.

**Step 2: Build and commit**

Run: `cargo build -p epu-showcase`

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_13_16.rs
git commit -m "epu-showcase: fix RAMP thresholds for presets 13-16"
```

---

## Task 6: Fix set_17_20.rs (Presets 17-20)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_17_20.rs`

**Step 1: Fix all four RAMP layers**

| Preset | Name | Threshold | Softness |
|--------|------|-----------|----------|
| 17 | Volcanic Core | `THRESH_INTERIOR` | 180 |
| 18 | Digital Matrix | `THRESH_BALANCED` | 220 |
| 19 | Noir Detective | `THRESH_INTERIOR` | 180 |
| 20 | Steampunk Airship | `THRESH_INTERIOR` | 190 |

**Step 2: Build and commit**

Run: `cargo build -p epu-showcase`

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_17_20.rs
git commit -m "epu-showcase: fix RAMP thresholds for presets 17-20"
```

---

## Task 7: Fix set_21_24.rs (Presets 21-24)

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/presets/set_21_24.rs`

**Step 1: Fix all four RAMP layers**

| Preset | Name | Threshold | Softness |
|--------|------|-----------|----------|
| 21 | Stormy Shores | `THRESH_OPEN` | 240 |
| 22 | Polar Aurora | `THRESH_OPEN` | 240 |
| 23 | Sacred Geometry | `THRESH_SEMI` | 200 |
| 24 | Ritual Chamber | `THRESH_INTERIOR` | 180 |

**Step 2: Build and commit**

Run: `cargo build -p epu-showcase`

```bash
git add examples/3-inspectors/epu-showcase/src/presets/set_21_24.rs
git commit -m "epu-showcase: fix RAMP thresholds for presets 21-24"
```

---

## Task 8: Fix opcode_name() in showcase lib.rs

**Files:**
- Modify: `examples/3-inspectors/epu-showcase/src/lib.rs:217-240`

**Step 1: Fix the three misleading opcode names**

Change:
```rust
        0x02 => b"LOBE/SECTOR",
        0x03 => b"BAND/SILHOUETTE",
        0x04 => b"FOG/SPLIT",
```
to:
```rust
        0x02 => b"SECTOR",
        0x03 => b"SILHOUETTE",
        0x04 => b"SPLIT",
```

**Step 2: Build and commit**

Run: `cargo build -p epu-showcase`

```bash
git add examples/3-inspectors/epu-showcase/src/lib.rs
git commit -m "epu-showcase: fix misleading opcode names in UI overlay"
```

---

## Task 9: Fix epu-multi-reflections presets

**Files:**
- Modify: `examples/2-graphics/epu-multi-reflections/src/lib.rs:95-172`

**Step 1: Fix all 4 RAMP layers**

The multi-reflections example has its own `epu_lo()` helper with the same signature. Each RAMP currently has `param_c=0xA5, param_d=0`. The threshold must be in `param_d`, and `param_a/b/c` should be wall RGB.

Fix each RAMP lo line:

**Neon City** (line 99): wall color = gray `#404060`
```rust
// Before: epu_lo(200, 180, 160, 0xA5, 0, DIR_UP, 15, 15),
// After:
epu_lo(220, 0x40, 0x40, 0x60, 0xA5, DIR_UP, 15, 15),
```

**Ember Glow** (line 119): wall color = dark brown `#401800`
```rust
// Before: epu_lo(220, 180, 160, 0xA5, 0, DIR_UP, 15, 15),
// After:
epu_lo(220, 0x40, 0x18, 0x00, 0xA5, DIR_UP, 15, 15),
```

**Frozen** (line 139): wall color = pale blue-gray `#C8D8E8`
```rust
// Before: epu_lo(200, 200, 180, 0xA5, 0, DIR_UP, 15, 15),
// After:
epu_lo(230, 0xC8, 0xD8, 0xE8, 0xA5, DIR_UP, 15, 15),
```

**Void** (line 159): wall color = near-black blue `#080810`
```rust
// Before: epu_lo(180, 200, 180, 0xA5, 0, DIR_UP, 15, 15),
// After:
epu_lo(220, 0x08, 0x08, 0x10, 0xA5, DIR_UP, 15, 15),
```

Also update the comments above each preset to reflect the corrected wall colors.

**Step 2: Build and commit**

Run: `cargo build -p epu-multi-reflections`

```bash
git add examples/2-graphics/epu-multi-reflections/src/lib.rs
git commit -m "epu-multi-reflections: fix RAMP param_d thresholds and wall colors"
```

---

## Task 10: Full build and visual verification

**Step 1: Full workspace build**

Run: `cargo build`
Expected: All crates compile without errors or warnings.

**Step 2: Run cargo clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: No warnings.

**Step 3: Visual verification checklist**

Run: `cargo run` (launches the player)

Load epu-showcase and cycle through all 24 presets. For each, verify:
- [ ] Background is not uniform/flat — sky, wall, and floor regions are visibly distinct
- [ ] Wall-targeted layers (GRID, SILHOUETTE, CELL, APERTURE, TRACE, VEIL) render on the wall belt
- [ ] Floor-targeted layers (PLANE, FLOW on floor) render on the lower hemisphere
- [ ] Sky-targeted layers (CELESTIAL, SCATTER stars, BAND) render on the upper hemisphere
- [ ] Opcode labels in the UI overlay show correct single names (no slashes)
- [ ] The 3D shape shows environment-driven reflections and ambient lighting

Load epu-multi-reflections and verify:
- [ ] Left and right objects reflect different environments
- [ ] Environment backgrounds show distinct sky/wall/floor regions
- [ ] Wall colors are visible (not just sky gradient)

**Step 4: Final commit (if any tweaks needed)**

```bash
git add -A
git commit -m "epu: visual tuning pass after threshold fixes"
```
