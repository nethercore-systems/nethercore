# Multi-Environment System Specification

**Status:** Pending
**Author:** Zerve
**Version:** 1.0
**Last Updated:** December 2024

---

## Summary

Add support for 7 procedural environment states via a new storage buffer. Each environment is identical to the current `PackedSky` (16 bytes). The embedded sky in `PackedUnifiedShadingState` is replaced with an environment index.

---

## Current Architecture

**PackedSky (16 bytes):**
```rust
pub struct PackedSky {
    pub horizon_color: u32,           // RGBA8
    pub zenith_color: u32,            // RGBA8
    pub sun_direction_oct: u32,       // Octahedral (snorm16x2)
    pub sun_color_and_sharpness: u32, // RGB8 + sharpness u8
}
```

**Current @group(0) bindings:**
- @binding(0): unified_transforms
- @binding(1): mvp_shading_indices
- @binding(2): shading_states
- @binding(3): unified_animation
- @binding(4): quad_instances (2D only)

---

## Proposed Changes

### 1. PackedEnvironmentState (16 bytes)

Same as PackedSky, just renamed and moved to separate buffer:

```rust
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedEnvironmentState {
    pub horizon_color: u32,           // RGBA8
    pub zenith_color: u32,            // RGBA8
    pub sun_direction_oct: u32,       // Octahedral (snorm16x2)
    pub sun_color_and_sharpness: u32, // RGB8 + sharpness u8
}
```

Can be aliased: `pub type PackedEnvironmentState = PackedSky;`

### 2. New Storage Buffer

**@group(0) @binding(4): environments**

```wgsl
@group(0) @binding(4) var<storage, read> environments: array<PackedEnvironmentState>;
```

Buffer size: 7 × 16 = 112 bytes

### 3. Renumber quad_instances

**@binding(4) → @binding(5)** in quad_template.wgsl

### 4. PackedUnifiedShadingState Changes

Remove embedded `sky: PackedSky` (16 bytes), repurpose `_animation_reserved` as `environment_index`:

```rust
#[repr(C)]
pub struct PackedUnifiedShadingState {
    // Header (16 bytes) - unchanged
    pub color_rgba8: u32,
    pub uniform_set_0: u32,
    pub uniform_set_1: u32,
    pub flags: u32,

    // Lights (48 bytes) - unchanged
    pub lights: [PackedLight; 4],

    // Animation + Environment (16 bytes)
    pub keyframe_base: u32,
    pub inverse_bind_base: u32,
    pub animation_flags: u32,
    pub environment_index: u32,    // Replaces _animation_reserved
}
// Total: 16 + 48 + 16 = 80 bytes (16-byte aligned) ✓
```

**Size reduction:** 96 → 80 bytes (-16 bytes per shading state)

### 5. Shader Updates

**common.wgsl:**
```wgsl
struct PackedEnvironmentState {
    horizon_color: u32,
    zenith_color: u32,
    sun_direction_oct: u32,
    sun_color_and_sharpness: u32,
}

// New binding
@group(0) @binding(4) var<storage, read> environments: array<PackedEnvironmentState>;

// Update PackedUnifiedShadingState struct (80 bytes, down from 96)
struct PackedUnifiedShadingState {
    color_rgba8: u32,
    uniform_set_0: u32,
    uniform_set_1: u32,
    flags: u32,

    lights: array<PackedLight, 4>,

    keyframe_base: u32,
    inverse_bind_base: u32,
    animation_flags: u32,
    environment_index: u32,      // Replaces _animation_reserved
}

// Helper function
fn get_sky(shading: PackedUnifiedShadingState) -> PackedEnvironmentState {
    let idx = shading.environment_index & 0x7u;
    return environments[idx];
}
```

**quad_template.wgsl:**
```wgsl
// Renumber
@group(0) @binding(5) var<storage, read> quad_instances: array<QuadInstance>;
```

### 6. FFI - Backwards Compatible

Existing functions update `environments[0]`:

```rust
// sky_set_colors() → updates environments[0].horizon_color, zenith_color
// sky_set_sun() → updates environments[0].sun_direction_oct, sun_color_and_sharpness
```

New functions for multiple environments:

```rust
/// Set environment sky colors (index 0-6)
fn environment_set_colors(index: u32, horizon: u32, zenith: u32);

/// Set environment sun (index 0-6)
fn environment_set_sun(index: u32, dir_x: f32, dir_y: f32, dir_z: f32, color: u32, sharpness: f32);

/// Select environment for subsequent draws (persists until changed)
fn environment_select(index: u32);
```

### 7. CPU State

**ffi_state.rs:**
```rust
pub environments: [PackedEnvironmentState; 7],
pub environments_dirty: bool,
pub current_environment_index: u32,
```

**Initialization:** All 7 environments get the same default sky values.

**Upload:** Once per frame if `environments_dirty`, 112 bytes.

---

## Implementation Plan

### Phase 1: Rust Structs
1. Add `PackedEnvironmentState` type alias in `unified_shading_state.rs`
2. Modify `PackedUnifiedShadingState`: remove `sky: PackedSky`, rename `_animation_reserved` to `environment_index`
3. Update size assertions (96 → 80 bytes)

### Phase 2: GPU Buffer
1. Add `environments_buffer: wgpu::Buffer` to ZGraphics (`graphics/mod.rs`)
2. Create buffer in `graphics/init.rs` (112 bytes, storage)
3. Add @binding(4) to frame bind group layout (`graphics/pipeline.rs`)
4. Upload buffer in `graphics/frame.rs` when dirty

### Phase 3: Shader Updates
1. Add `PackedEnvironmentState` struct to `common.wgsl`
2. Add @binding(4) environments buffer to `common.wgsl`
3. Update `PackedUnifiedShadingState` in WGSL
4. Replace `shading.sky` reads with `get_sky(shading)` helper
5. Renumber @binding(4) → @binding(5) in `quad_template.wgsl`

### Phase 4: State Management
1. Add `environments`, `environments_dirty`, `current_environment_index` to `ffi_state.rs`
2. Initialize all 7 environments with default sky
3. Update `clear_frame()` to reset dirty flag after upload

### Phase 5: FFI Functions
1. Update `sky_set_colors()` to write to `environments[0]` and set dirty
2. Update `sky_set_sun()` to write to `environments[0]` and set dirty
3. Add `environment_set_colors(index, ...)`
4. Add `environment_set_sun(index, ...)`
5. Add `environment_select(index)` - sets `current_environment_index`
6. Update shading state sync to use `current_environment_index`

### Phase 6: Update Bind Group
1. Update `create_frame_bind_group()` to include environments buffer
2. Update `create_frame_bind_group_layout()` for new binding

---

## Files to Modify

| File | Changes |
|------|---------|
| `emberware-z/src/graphics/unified_shading_state.rs` | Add type alias, modify struct |
| `emberware-z/src/graphics/mod.rs` | Add `environments_buffer` field |
| `emberware-z/src/graphics/init.rs` | Create environments buffer |
| `emberware-z/src/graphics/pipeline.rs` | Add @binding(4) to layout |
| `emberware-z/src/graphics/frame.rs` | Upload environments, update bind group |
| `emberware-z/src/state/ffi_state.rs` | Add environments state |
| `emberware-z/src/ffi/sky.rs` | Update existing + add new FFI |
| `emberware-z/shaders/common.wgsl` | Add struct, binding, update reads |
| `emberware-z/shaders/quad_template.wgsl` | Renumber @binding(4)→@binding(5) |

---

## Memory Impact

**Buffer:** 112 bytes (7 × 16) - tiny, uploaded once per frame if dirty

**Per shading state:** 96 → 80 bytes (-16 bytes, -17%)
- Removed: 16-byte embedded `PackedSky`
- Added: 4-byte `environment_index` (repurposed `_animation_reserved`)

**Benefit:**
- 16 bytes saved per unique shading state
- Multiple draws can reference different environments without duplicating sky data
