# Emberware Z Dither Transparency Specification

**Status:** Ready for Implementation
**Author:** Zerve
**Last Updated:** December 2025

---

## Overview

Emberware Z uses **always-on dither transparency** (screen-door / ordered dithering) with no separate "transparency mode" flag. Dithering is the only transparency method for 3D geometry—alpha blending is reserved for screen-space UI/text only.

**Key principles:**
- No alpha blending for 3D (ever)
- No transparency mode flag—dithering is always active
- `uniform_alpha = 15` = fully opaque (default), `0` = fully transparent
- 4-bit dither offset prevents stacked mesh cancellation
- Compile-time 4×4 Bayer pattern (era-authentic Saturn/PS1)

---

## Why Always-On Dither?

| Aspect | Alpha Blending | Always-On Dither |
|--------|----------------|-------------------|
| Sorting | Required (expensive) | **Never** |
| Overdraw | High | **Low** |
| Depth write | Complex | **Always** (discard = no write) |
| Stacking | Order-dependent artifacts | **Order-independent** |
| Branching | N/A | **Uniform-coherent** (fast) |
| Era-authentic | No | **Yes** (Saturn/PS1) |

With `uniform_alpha = 15` (the default), the dither check is essentially free because all fragments coherently skip the discard (uniform branching, no divergence).

---

## PackedUnifiedShadingState.flags Layout

```
flags (32 bits):
├─ Bit 0:      skinning_mode         (existing: 0=raw, 1=inverse bind)
├─ Bit 1:      texture_filter        (existing: 0=nearest, 1=linear)
├─ Bits 2-7:   [reserved for material overrides - see material-overrides-spec.md]
├─ Bits 8-11:  uniform_alpha         (4 bits: 0-15, default 15 = opaque)
├─ Bits 12-13: dither_offset_x       (2 bits: 0-3 pixel shift)
├─ Bits 14-15: dither_offset_y       (2 bits: 0-3 pixel shift)
└─ Bits 16-31: reserved
```

### uniform_alpha (4 bits, 0-15)

Maps directly to 4×4 Bayer matrix levels:
- `0` = fully transparent (all pixels discarded)
- `8` = ~50% transparency
- `15` = fully opaque (no pixels discarded, default)

**Default: 15** — Existing code renders opaque without changes.

### dither_offset (4 bits total: 2-bit X + 2-bit Y)

Shifts the Bayer pattern lookup to prevent stacked mesh cancellation:
- Values 0-3 for each axis
- Different offsets = different patterns = visible overlap

---

## Dither Offset: Solving Stacked Mesh Cancellation

### The Problem

When two dithered meshes overlap with the same alpha and offset, their Bayer patterns align:

```
Mesh A (alpha=8, offset=0,0):  █░█░    Both use same pattern
Mesh B (alpha=8, offset=0,0):  ░█░█    → Pixels cancel out!
Result:                        ░░░░    (invisible)
```

### The Solution

Assign different `dither_offset` values to stacked transparent objects:

```
Mesh A (alpha=8, offset=0,0):  █░█░    Standard pattern
Mesh B (alpha=8, offset=2,1):  ░░█░    Shifted pattern
Result:                        █░██    (both visible!)
```

### Usage Pattern

```c
// Glass windows that might overlap
uniform_alpha(8);
dither_offset(0, 0);  // Window 1
draw_mesh(window_front);

dither_offset(2, 2);  // Window 2 (offset to avoid cancellation)
draw_mesh(window_back);
```

---

## 4×4 Bayer Matrix

The classic Saturn/PS1 dither pattern with 16 alpha levels:

```wgsl
const BAYER_4X4: array<f32, 16> = array(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
);
```

This is a compile-time constant. To use a different pattern (2×2, 8×8, 16×16), modify the shader constant and adjust the modulo operations.

---

## Shader Implementation

### Constants (common.wgsl)

```wgsl
// Dither field masks in PackedUnifiedShadingState.flags
const FLAG_UNIFORM_ALPHA_MASK: u32 = 0xF00u;      // Bits 8-11
const FLAG_UNIFORM_ALPHA_SHIFT: u32 = 8u;
const FLAG_DITHER_OFFSET_X_MASK: u32 = 0x3000u;   // Bits 12-13
const FLAG_DITHER_OFFSET_X_SHIFT: u32 = 12u;
const FLAG_DITHER_OFFSET_Y_MASK: u32 = 0xC000u;   // Bits 14-15
const FLAG_DITHER_OFFSET_Y_SHIFT: u32 = 14u;

// 4x4 Bayer matrix - classic Saturn dither (16 alpha levels)
const BAYER_4X4: array<f32, 16> = array(
     0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
    12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
     3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
    15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
);
```

### Helper Functions (common.wgsl)

```wgsl
// Extract uniform alpha (0-15 → 0.0-1.0)
fn get_uniform_alpha(flags: u32) -> f32 {
    let level = (flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
    return f32(level) / 15.0;
}

// Extract dither offset
fn get_dither_offset(flags: u32) -> vec2<u32> {
    let x = (flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT;
    return vec2<u32>(x, y);
}

// Always-on dither transparency
fn should_discard_dither(frag_coord: vec2<f32>, flags: u32) -> bool {
    let uniform_alpha = get_uniform_alpha(flags);
    let offset = get_dither_offset(flags);

    // Apply offset to break pattern alignment for stacked meshes
    let x = (u32(frag_coord.x) + offset.x) % 4u;
    let y = (u32(frag_coord.y) + offset.y) % 4u;

    let threshold = BAYER_4X4[y * 4u + x];
    return uniform_alpha < threshold;
}
```

### Fragment Shader Usage

Every mode shader (0-3) adds this before the final return:

```wgsl
@fragment
fn fs(in: VertexOut) -> @location(0) vec4<f32> {
    let shading = shading_states[in.shading_state_index];
    // ... existing shading code ...

    // Dither transparency (always active)
    if should_discard_dither(in.clip_position.xy, shading.flags) {
        discard;
    }

    return vec4<f32>(color, 1.0);  // Alpha channel unused for 3D
}
```

---

## Rust Implementation

### Flag Constants (unified_shading_state.rs)

```rust
// Dither transparency fields
pub const FLAG_UNIFORM_ALPHA_MASK: u32 = 0xF << 8;     // Bits 8-11
pub const FLAG_UNIFORM_ALPHA_SHIFT: u32 = 8;
pub const FLAG_DITHER_OFFSET_X_MASK: u32 = 0x3 << 12;  // Bits 12-13
pub const FLAG_DITHER_OFFSET_X_SHIFT: u32 = 12;
pub const FLAG_DITHER_OFFSET_Y_MASK: u32 = 0x3 << 14;  // Bits 14-15
pub const FLAG_DITHER_OFFSET_Y_SHIFT: u32 = 14;

// Default flags should have uniform_alpha = 15 (opaque)
pub const DEFAULT_FLAGS: u32 = 0xF << 8;  // uniform_alpha = 15
```

### State Update Methods (ffi_state.rs)

```rust
/// Update uniform alpha level in current shading state
/// - 0: fully transparent (all pixels discarded)
/// - 15: fully opaque (no pixels discarded, default)
pub fn update_uniform_alpha(&mut self, alpha: u8) {
    use crate::graphics::{FLAG_UNIFORM_ALPHA_MASK, FLAG_UNIFORM_ALPHA_SHIFT};

    let alpha = alpha.min(15) as u32;  // Clamp to 4 bits
    let new_flags = (self.current_shading_state.flags & !FLAG_UNIFORM_ALPHA_MASK)
        | (alpha << FLAG_UNIFORM_ALPHA_SHIFT);

    if self.current_shading_state.flags != new_flags {
        self.current_shading_state.flags = new_flags;
        self.shading_state_dirty = true;
    }
}

/// Update dither offset in current shading state
/// - x: 0-3 pixel shift in X
/// - y: 0-3 pixel shift in Y
/// Use different offsets for stacked transparent objects to prevent cancellation
pub fn update_dither_offset(&mut self, x: u8, y: u8) {
    use crate::graphics::{
        FLAG_DITHER_OFFSET_X_MASK, FLAG_DITHER_OFFSET_X_SHIFT,
        FLAG_DITHER_OFFSET_Y_MASK, FLAG_DITHER_OFFSET_Y_SHIFT,
    };

    let x = (x.min(3) as u32) << FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (y.min(3) as u32) << FLAG_DITHER_OFFSET_Y_SHIFT;
    let new_flags = (self.current_shading_state.flags
        & !FLAG_DITHER_OFFSET_X_MASK
        & !FLAG_DITHER_OFFSET_Y_MASK)
        | x | y;

    if self.current_shading_state.flags != new_flags {
        self.current_shading_state.flags = new_flags;
        self.shading_state_dirty = true;
    }
}
```

### FFI Functions (ffi/render_state.rs)

```rust
/// Set uniform alpha level for dither transparency
///
/// # Arguments
/// * `level` — 0-15 (0=transparent, 15=opaque, default=15)
fn uniform_alpha(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, level: u32) {
    let state = &mut caller.data_mut().console;

    if level > 15 {
        warn!("uniform_alpha({}) invalid - must be 0-15, clamping to 15", level);
    }

    state.update_uniform_alpha(level.min(15) as u8);
}

/// Set dither offset for dither transparency
///
/// # Arguments
/// * `x` — 0-3 pixel shift in X axis
/// * `y` — 0-3 pixel shift in Y axis
///
/// Use different offsets for stacked dithered meshes to prevent pattern cancellation.
fn dither_offset(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, x: u32, y: u32) {
    let state = &mut caller.data_mut().console;

    if x > 3 || y > 3 {
        warn!("dither_offset({}, {}) invalid - values must be 0-3", x, y);
    }

    state.update_dither_offset(x.min(3) as u8, y.min(3) as u8);
}
```

---

## FFI API Summary

```c
// Set dither alpha level (0-15, default 15 = opaque)
uniform_alpha(8);   // 50% transparency

// Set dither offset to prevent stacking cancellation (0-3 each axis)
dither_offset(0, 0);  // Default
dither_offset(2, 1);  // Offset for second layer
```

---

## Default Values

| Field | Default | Bits | Meaning |
|-------|---------|------|---------|
| uniform_alpha | 15 (0xF) | 8-11 | Fully opaque |
| dither_offset_x | 0 | 12-13 | No X shift |
| dither_offset_y | 0 | 14-15 | No Y shift |

**Critical:** Default `uniform_alpha = 15` ensures existing code renders opaque without changes.

---

## Performance Notes

### Why Is This Fast?

**Uniform branching:** `uniform_alpha` comes from the shading state uniform—same value for all fragments in a draw call.

- When `uniform_alpha = 15`, ALL fragments coherently skip the discard
- GPU SIMD lanes take the same path (no divergence)
- Branch prediction is perfect
- Essentially zero overhead for opaque objects

This is fundamentally different from texture-based alpha where different pixels may take different paths.

### Depth Write Behavior

`discard` automatically prevents depth write for discarded pixels. This is why dither transparency is order-independent—each mesh writes depth for its surviving pixels, subsequent meshes are properly depth-tested.

---

## Visual Reference

### Dither Levels (4×4 pattern)

```
α=0 (0/15):    α=4 (4/15):    α=8 (8/15):    α=12 (12/15):  α=15 (15/15):
░░░░           █░░░           █░█░           █░██           ████
░░░░           ░░░░           ░█░█           ██░█           ████
░░░░           ░░█░           █░█░           █░██           ████
░░░░           ░░░░           ░█░█           ░███           ████
(invisible)    (sparse)       (50%)          (dense)        (solid)
```

### Shimmer Behavior

Screen-space dithering produces intentional shimmer when objects/camera move. **This is the authentic 5th-gen aesthetic.**

---

## Unit Test Specifications

### Rust Tests

```rust
#[test]
fn test_uniform_alpha_packing() {
    // Test all 16 values pack/unpack correctly
    for alpha in 0..=15 {
        let flags = (alpha as u32) << FLAG_UNIFORM_ALPHA_SHIFT;
        let unpacked = (flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(unpacked, alpha as u32);
    }
}

#[test]
fn test_dither_offset_packing() {
    // Test all 16 offset combinations
    for x in 0..=3 {
        for y in 0..=3 {
            let flags = ((x as u32) << FLAG_DITHER_OFFSET_X_SHIFT)
                      | ((y as u32) << FLAG_DITHER_OFFSET_Y_SHIFT);
            let unpacked_x = (flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT;
            let unpacked_y = (flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT;
            assert_eq!(unpacked_x, x as u32);
            assert_eq!(unpacked_y, y as u32);
        }
    }
}

#[test]
fn test_default_flags_are_opaque() {
    let state = PackedUnifiedShadingState::default();
    let alpha = (state.flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 15, "Default uniform_alpha must be 15 (opaque)");
}

#[test]
fn test_uniform_alpha_update() {
    let mut ffi_state = ZFFIState::default();

    // Default should be opaque
    let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
                >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 15);

    // Update to 50%
    ffi_state.update_uniform_alpha(8);
    let alpha = (ffi_state.current_shading_state.flags & FLAG_UNIFORM_ALPHA_MASK)
                >> FLAG_UNIFORM_ALPHA_SHIFT;
    assert_eq!(alpha, 8);
    assert!(ffi_state.shading_state_dirty);
}

#[test]
fn test_dither_offset_update() {
    let mut ffi_state = ZFFIState::default();

    ffi_state.update_dither_offset(2, 3);

    let x = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_X_MASK)
            >> FLAG_DITHER_OFFSET_X_SHIFT;
    let y = (ffi_state.current_shading_state.flags & FLAG_DITHER_OFFSET_Y_MASK)
            >> FLAG_DITHER_OFFSET_Y_SHIFT;

    assert_eq!(x, 2);
    assert_eq!(y, 3);
}

#[test]
fn test_bayer_threshold_values() {
    // Verify Bayer matrix produces values in expected range
    const BAYER_4X4: [f32; 16] = [
         0.0/16.0,  8.0/16.0,  2.0/16.0, 10.0/16.0,
        12.0/16.0,  4.0/16.0, 14.0/16.0,  6.0/16.0,
         3.0/16.0, 11.0/16.0,  1.0/16.0,  9.0/16.0,
        15.0/16.0,  7.0/16.0, 13.0/16.0,  5.0/16.0,
    ];

    for (i, &threshold) in BAYER_4X4.iter().enumerate() {
        assert!(threshold >= 0.0, "Threshold {} is negative", i);
        assert!(threshold < 1.0, "Threshold {} >= 1.0", i);
    }

    // Verify we have 16 unique values
    let mut sorted = BAYER_4X4.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    for i in 0..15 {
        assert_ne!(sorted[i], sorted[i+1], "Duplicate threshold values");
    }
}
```

---

## Example Games

### 1. Dither Showcase Demo

**Purpose:** Visual demonstration of all dither features

**Scene contents:**
- Row of 16 cubes with `uniform_alpha` 0-15
- Two overlapping glass panels demonstrating `dither_offset`
- Animated character fading in/out
- Moving camera to show shimmer effect

**Code snippet:**
```c
void render() {
    // Alpha level showcase
    for (int i = 0; i <= 15; i++) {
        uniform_alpha(i);
        transform_set(cube_positions[i]);
        draw_mesh(cube);
    }

    // Stacked glass demo
    uniform_alpha(8);
    dither_offset(0, 0);
    draw_mesh(glass_front);

    dither_offset(2, 2);
    draw_mesh(glass_back);

    // Fading character
    uniform_alpha(fade_level);  // Animated 0-15
    dither_offset(0, 0);
    draw_mesh(character);
}
```

### 2. Ghost Game Prototype

**Purpose:** Gameplay integration of dither transparency

**Mechanics:**
- Player toggles "ghost mode" with dither fade
- Ghost enemies pulse between alpha 8-12
- Walls become semi-transparent when occluding player
- Multiple ghosts use different `dither_offset` values

**Code snippet:**
```c
void render_ghosts() {
    for (int i = 0; i < ghost_count; i++) {
        // Pulsing alpha 8-12
        int pulse = 8 + (int)(sin(time + i) * 2.0);
        uniform_alpha(pulse);

        // Different offset per ghost to prevent cancellation
        dither_offset(i % 4, (i / 4) % 4);

        transform_set(ghost_transforms[i]);
        draw_mesh(ghost_mesh);
    }
}
```

### 3. Window Reflection Demo

**Purpose:** Show correct depth handling with dithering

**Scene:**
- Building with multiple glass windows at different depths
- Character walking behind windows
- Demonstrates order-independent transparency

---

## Files to Create/Modify

| File | Changes |
|------|---------|
| `emberware-z/src/graphics/unified_shading_state.rs` | Add `FLAG_UNIFORM_ALPHA_*`, `FLAG_DITHER_OFFSET_*` constants; update `Default` impl to set `uniform_alpha = 15` |
| `emberware-z/src/state/ffi_state.rs` | Add `update_uniform_alpha()`, `update_dither_offset()` methods |
| `emberware-z/src/ffi/render_state.rs` | Add `uniform_alpha`, `dither_offset` FFI functions |
| `emberware-z/shaders/common.wgsl` | Add `FLAG_UNIFORM_ALPHA_*` constants, `BAYER_4X4` array, `should_discard_dither()` helper |
| `emberware-z/shaders/mode0_unlit.wgsl` | Add dither discard check in fragment shader |
| `emberware-z/shaders/mode1_matcap.wgsl` | Add dither discard check in fragment shader |
| `emberware-z/shaders/blinnphong_common.wgsl` | Add dither discard check (shared by Mode 2 + 3) |
| `emberware-z/shaders/quad_template.wgsl` | Add dither discard check for world-space quads/billboards |

**Note:** All 3D rendering paths (meshes, quads, billboards) use the same dither transparency. Screen-space UI/text is the only exception (uses alpha blending).

---

## Default Initialization

The `PackedUnifiedShadingState::default()` impl MUST set `uniform_alpha = 15` (opaque):

```rust
impl Default for PackedUnifiedShadingState {
    fn default() -> Self {
        Self {
            color_rgba8: 0xFFFFFFFF,
            uniform_set_0: pack_uniform_set_0(0, 128, 0, 0),
            uniform_set_1: pack_uniform_set_1(255, 255, 255, 0),
            // Critical: uniform_alpha = 15 (bits 8-11) for opaque default
            flags: 0xF << 8,  // uniform_alpha = 15, all other flags = 0
            sky: PackedSky::default(),
            lights: [PackedLight::default(); 4],
        }
    }
}
```

---

## Future Work

- **Texture alpha multiplication:** `final_alpha = uniform_alpha * texture.a` (allows per-pixel detail within dithering)
- **Alternative patterns:** 2×2, 8×8, 16×16 Bayer matrices via compile-time flag
- **Auto-offset assignment:** Runtime assignment of unique offsets to batched transparent objects

---

## Related Documents

- [material-overrides-spec.md](material-overrides-spec.md) — Uniform vs texture sourcing flags (bits 2-7) — **Ready for implementation**
- [texture-spec.md](texture-spec.md) — Texture format specification
- [dither-patterns.md](../reference/dither-patterns.md) — Bayer matrix reference (2x2, 4x4, 8x8, 16x16)

---

## References

- [Ordered Dithering (Wikipedia)](https://en.wikipedia.org/wiki/Ordered_dithering)
- [Bayer Matrix Generation](https://www.anisopteragames.com/how-to-fix-color-banding-with-dithering/)
- [Saturn Mesh Transparency](https://segaretro.org/Sega_Saturn/Hardware_features)
