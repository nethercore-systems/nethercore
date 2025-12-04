# Implementation Plan: Unified Shading State

**Status:** Not Started (implement after matrix packing)
**Estimated Effort:** 4-6 days
**Priority:** High (bug fix - current implementation is wrong)
**Depends On:** Matrix index packing (required - uses second u32 in mvp_indices buffer)
**Related:** [proposed-render-architecture.md](./proposed-render-architecture.md), [rendering-architecture.md](./rendering-architecture.md)

---

## Overview

**This is a bug fix, not just an optimization.** The current implementation uses frame-wide uniforms for material properties (metallic, roughness, emissive, lights, sky), but users need to set these **per-draw**. This implementation fixes that by quantizing per-draw shading state and storing it in a GPU buffer.

Quantize all per-draw shading state into a hashable POD structure (`PackedUnifiedShadingState`), implement interning to deduplicate identical states, and enable per-draw material control.

**Benefits:**
- **FIX:** Per-draw material properties instead of incorrect frame-wide uniforms
- **HUGE SIMPLIFICATION:** All render modes use the SAME binding layout (0-5)
- Material state becomes hashable and comparable
- Same material used across draws = one GPU upload (deduplication)
- Better command sorting by material
- Reduced VRPCommand size (remove separate state fields)
- Eliminates off-by-N binding errors between modes

**Approach:** Storage buffer indexed via instance index (extends existing MVP indices buffer)

The MVP indices buffer is already `array<vec2<u32>>` where:
- `.x` = packed MVP indices (model: 16 bits, view: 8 bits, proj: 8 bits)
- `.y` = unified shading state index (reserved for this implementation)

**Complexity:** Medium-High - touches FFI, command recording, shaders, and GPU upload, but leverages existing infrastructure

---

## Critical Clarifications (December 2024)

### Current Codebase State vs Plan Assumptions

**Q: Are transform_stack, current_transform, and camera fields still in ZFFIState?**
**A: YES, but they should be REMOVED in this refactor.**
- These fields exist in current code (lines 252-256 in state.rs)
- FFI functions like `push_transform()`, `pop_transform()`, `translate()`, etc. use them
- **Decision:** Remove these fields and their associated FFI functions (they're not used in actual game logic)
- The matrix pool system (`model_matrices`, `view_matrices`, `proj_matrices`) fully replaces them

**Q: How are lights currently stored in ZFFIState?**
**A: As floating-point `[LightState; 4]`, NOT as `[PackedLight; 4]`**
- Current: `pub lights: [LightState; 4]` with `[f32; 3]` arrays for direction/color
- Plan expects: `pub lights: [PackedLight; 4]` (quantized)
- **Quantization point:** At FFI barrier - user passes f32, we quantize to snorm16/u8, store quantized, handle dirty flags
- This is consistent with how metallic/roughness/emissive work (unquantized in state, quantized when marked dirty)

**Q: What about mvp_indices buffer .y field?**
**A: Currently UNUSED - left as breadcrumbs for this refactor**
- Buffer is already allocated as `vec2<u32>` (line 213 in graphics/mod.rs)
- Currently only `.x` is populated with packed MVP indices
- `.y` is undefined/zero - this refactor will populate it with shading_state_index

**Q: How do shader binding layouts differ between modes?**
**A: See [binding-layout-migration.md](./binding-layout-migration.md) and [shader-gen-changes.md](./shader-gen-changes.md)**
- Current: Mode 0/1 use bindings 0-6, Mode 2/3 use bindings 0-8 (inconsistent)
- Target: All modes use bindings 0-5 (unified)
- Bones move from binding 6/8 → binding 5 (consistent across all modes)

**Q: What about DeferredCommand::SetSky?**
**A: EXISTS and should be REMOVED**
- Current: Sky is set via `DeferredCommand::SetSky` (frame-wide state)
- Target: Sky is per-draw state (part of UnifiedShadingState)
- Remove `SetSky` variant from `DeferredCommand` enum
- Remove FFI function `set_sky()` that creates this variant

**Q: Default sky value?**
**A: All zeros (black sky, no sun)**
- `PackedSky::default()` returns all zeros
- Games can set sky in init() via material setters (will be added)
- Sky is always per-draw, not frame-wide

**Q: matcap_blend_modes type - [u8; 4] or [MatcapBlendMode; 4]?**
**A: [MatcapBlendMode; 4] (enum array)**
- Current code has `[u8; 4]` in ZFFIState
- Should be `[MatcapBlendMode; 4]` for type safety
- Each enum variant maps to u8 value (0-2)
- Will be updated during this refactor

---

## Key Architectural Insight

**All render modes now use the SAME binding layout:**

| Binding | Contents | Used By |
|---------|----------|---------|
| 0 | `model_matrices: array<mat4x4<f32>>` | Vertex shader |
| 1 | `view_matrices: array<mat4x4<f32>>` | Vertex shader |
| 2 | `proj_matrices: array<mat4x4<f32>>` | Vertex shader |
| 3 | `shading_states: array<UnifiedShadingState>` | Fragment shader (vertex passes index) |
| 4 | `mvp_shading_indices: array<vec2<u32>>` | Vertex shader |
| 5 | `bones: array<mat4x4<f32>>` | Vertex shader (optional) |

**Logical grouping:**
- **Bindings 0-3:** Data buffers (matrices, shading states)
- **Bindings 4-5:** Indices/structural (per-draw indices, bones)

**What happened to sky, material, lights, camera?**
- ✅ **Sky** → Contained in `shading_states`
- ✅ **Material** → Contained in `shading_states`
- ✅ **Lights** → Contained in `shading_states`
- ✅ **Camera position** → Derivable from view matrix in `shading_states`

This eliminates 4 separate uniform bindings and makes all modes use the exact same shader interface!

---

## Phase 1: Define Packed State Structures

**Estimated Time:** 4-6 hours

### Files to Create
- `emberware-z/src/graphics/unified_shading_state.rs` (new)

### Changes

#### 1.1: Define Packed Structures

**New file:** `emberware-z/src/graphics/unified_shading_state.rs`

```rust
use bytemuck::{Pod, Zeroable};
use glam::{Vec3, Vec4};

/// Quantized sky data for GPU upload
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedSky {
    pub horizon_color: u32,              // RGBA8 packed
    pub zenith_color: u32,               // RGBA8 packed
    pub sun_direction: [i16; 4],         // snorm16x4 (w unused)
    pub sun_color_and_sharpness: u32,    // RGB8 + sharpness u8
}

/// One packed light
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedLight {
    pub direction: [i16; 4],             // snorm16x4 (w = enabled flag)
    pub color_and_intensity: u32,        // RGB8 + intensity u8
}

/// Unified per-draw shading state (~96 bytes, POD, hashable)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedUnifiedShadingState {
    // PBR params (4 bytes)
    pub metallic: u8,
    pub roughness: u8,
    pub emissive: u8,
    pub pad0: u8,

    pub color_rgba8: u32,                // Base color (4 bytes)
    pub blend_modes: u32,                // 4× u8 packed (4 bytes)

    pub sky: PackedSky,                  // 16 bytes
    pub lights: [PackedLight; 4],        // 64 bytes
}

/// Handle to interned shading state (newtype for clarity and type safety)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShadingStateIndex(pub u32);

impl ShadingStateIndex {
    pub const INVALID: Self = Self(0);
}
```

#### 1.2: Add Quantization Helpers

```rust
impl PackedUnifiedShadingState {
    /// Construct from unquantized render state
    pub fn from_render_state(
        color: u32,
        metallic: f32,
        roughness: f32,
        emissive: f32,
        matcap_blend_modes: &[MatcapBlendMode; 4],
        sky: &PackedSky,
        lights: &[PackedLight; 4],
    ) -> Self {
        Self {
            metallic: quantize_f32_to_u8(metallic),
            roughness: quantize_f32_to_u8(roughness),
            emissive: quantize_f32_to_u8(emissive),
            pad0: 0,

            color_rgba8: color,
            blend_modes: pack_blend_modes(matcap_blend_modes),

            sky: *sky,
            lights: *lights,
        }
    }
}

impl PackedSky {
    /// Create from unquantized float values (called from FFI setters)
    pub fn from_floats(
        horizon_color: Vec3,
        zenith_color: Vec3,
        sun_direction: Vec3,
        sun_color: Vec3,
        sun_sharpness: f32,
    ) -> Self {
        Self {
            horizon_color: pack_rgb8_to_u32(horizon_color),
            zenith_color: pack_rgb8_to_u32(zenith_color),
            sun_direction: quantize_vec3_to_snorm16(sun_direction),
            sun_color_and_sharpness: pack_color_and_scalar(sun_color, sun_sharpness),
        }
    }
}

impl PackedLight {
    /// Create from unquantized float values (called from FFI setters)
    pub fn from_floats(
        direction: Vec3,
        color: Vec3,
        intensity: f32,
        enabled: bool,
    ) -> Self {
        Self {
            direction: quantize_vec3_to_snorm16_with_flag(direction, enabled),
            color_and_intensity: pack_color_and_scalar(color, intensity),
        }
    }
}

// Helper functions
fn quantize_f32_to_u8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

fn quantize_vec3_to_snorm16(v: Vec3) -> [i16; 4] {
    let normalized = v.normalize_or_zero();
    [
        (normalized.x * 32767.0).round() as i16,
        (normalized.y * 32767.0).round() as i16,
        (normalized.z * 32767.0).round() as i16,
        0,
    ]
}

fn quantize_vec3_to_snorm16_with_flag(v: Vec3, enabled: bool) -> [i16; 4] {
    let normalized = v.normalize_or_zero();
    [
        (normalized.x * 32767.0).round() as i16,
        (normalized.y * 32767.0).round() as i16,
        (normalized.z * 32767.0).round() as i16,
        if enabled { 32767 } else { 0 },
    ]
}

fn pack_rgb8_to_u32(color: Vec3) -> u32 {
    let r = (color.x.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.y.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.z.clamp(0.0, 1.0) * 255.0).round() as u32;
    // Alpha is always 255 for sky colors
    (r << 24) | (g << 16) | (b << 8) | 0xFF
}

fn pack_color_and_scalar(color: Vec3, scalar: f32) -> u32 {
    let r = (color.x.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.y.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.z.clamp(0.0, 1.0) * 255.0).round() as u32;
    let s = (scalar.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | s
}

fn pack_blend_modes(modes: &[MatcapBlendMode; 4]) -> u32 {
    (modes[0] as u32) << 24
        | (modes[1] as u32) << 16
        | (modes[2] as u32) << 8
        | (modes[3] as u32)
}
```

---

## Phase 1.5: Remove Legacy Transform System

**Estimated Time:** 2-3 hours

### Files to Modify
- `emberware-z/src/state.rs`
- `emberware-z/src/ffi/mod.rs`

### Changes

#### 1.5.1: Remove Transform Stack Fields from ZFFIState

**File:** `emberware-z/src/state.rs`

Remove the following fields:
```rust
// REMOVE THESE:
pub camera: CameraState,           // ❌ Replaced by view/proj matrices
pub transform_stack: Vec<Mat4>,    // ❌ Replaced by model matrix pool
pub current_transform: Mat4,       // ❌ Replaced by model matrix pool
```

**Rationale:** The matrix pool system (`model_matrices`, `view_matrices`, `proj_matrices`) fully replaces these legacy fields. The transform stack was used for hierarchical transformations, but this is now handled by explicitly managing model matrices via FFI.

#### 1.5.2: Remove Transform Stack FFI Functions

**File:** `emberware-z/src/ffi/mod.rs`

Remove the following FFI functions (if they exist):
- `push_transform()` - No longer needed (use matrix pool instead)
- `pop_transform()` - No longer needed
- `translate()` - No longer needed (construct matrices explicitly)
- `rotate()` - No longer needed
- `scale()` - No longer needed
- Any other transform stack manipulation functions

**Note:** These functions are not used in actual game logic and were legacy from the old transform system.

#### 1.5.3: Remove Camera FFI Functions

Remove camera-related FFI functions that manipulate `CameraState`:
- `camera_set()` or similar
- Camera position/target setters

**Replacement:** Games use `view_set()` and `proj_set()` to set view and projection matrices directly.

#### 1.5.4: Remove DeferredCommand::SetSky Variant

**File:** `emberware-z/src/state.rs`

Remove the `SetSky` variant from the `DeferredCommand` enum:
```rust
pub enum DeferredCommand {
    DrawBillboard { /* ... */ },
    DrawSprite { /* ... */ },
    DrawRect { /* ... */ },
    DrawText { /* ... */ },
    // SetSky { /* ... */ },  ← REMOVE THIS VARIANT
}
```

**File:** `emberware-z/src/ffi/mod.rs`

Remove the `set_sky()` FFI function that creates this variant.

**Rationale:** Sky is now per-draw state (part of UnifiedShadingState), not frame-wide deferred state.

---

## Phase 2: Implement Shading State Cache

**Estimated Time:** 4-6 hours

### Files to Modify
- `emberware-z/src/graphics/unified_shading_state.rs` (extend)
- `emberware-z/src/graphics/mod.rs`
- `emberware-z/src/state.rs`

### Changes

#### 2.1: Add Shading State Pool to ZFFIState

**File:** `emberware-z/src/state.rs`

```rust
use hashbrown::HashMap;

pub struct ZFFIState {
    // Existing fields REMOVED (Phase 1.5):
    // - transform_stack: Vec<Mat4>  ❌ REMOVED (replaced by model matrix system)
    // - current_transform: Mat4      ❌ REMOVED (replaced by model matrix system)
    // - camera: CameraState          ❌ REMOVED (replaced by view/proj matrices)

    // Existing fields UPDATED:
    // - matcap_blend_modes: [u8; 4] → [MatcapBlendMode; 4]  (type safety)
    // - lights: [LightState; 4] → [PackedLight; 4]  (quantized storage)

    // Material properties (unquantized f32 for easy manipulation)
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
    pub matcap_blend_modes: [MatcapBlendMode; 4],

    // Sky and lights stored QUANTIZED (updated immediately at FFI barrier)
    pub sky: PackedSky,
    pub lights: [PackedLight; 4],

    // Current PACKED shading state (updated when FFI functions modify state)
    // This avoids re-quantizing on every draw!
    current_packed_shading_state: PackedUnifiedShadingState,

    // Shading state pool (reset each frame, similar to model_matrices)
    pub shading_states: Vec<PackedUnifiedShadingState>,
    shading_state_cache: HashMap<PackedUnifiedShadingState, u32>,  // For deduplication
}

impl Default for ZFFIState {
    fn default() -> Self {
        // Default sky: black (all zeros)
        let default_sky = PackedSky::default();

        // Default lights: all disabled
        let default_lights = [PackedLight::default(); 4];

        Self {
            // Existing initialization...
            metallic: 0.0,
            roughness: 1.0,
            emissive: 0.0,
            matcap_blend_modes: [MatcapBlendMode::Multiply; 4],
            sky: default_sky,
            lights: default_lights,
            current_packed_shading_state: PackedUnifiedShadingState::default(),
            shading_states: Vec::with_capacity(256),
            shading_state_cache: HashMap::new(),
        }
    }
}

impl ZFFIState {
    /// Mark current shading state as dirty (needs re-packing)
    /// Call this from FFI functions that modify material/sky/lights
    fn mark_shading_state_dirty(&mut self) {
        self.current_packed_shading_state = PackedUnifiedShadingState::from_render_state(
            self.color,
            self.metallic,
            self.roughness,
            self.emissive,
            &self.matcap_blend_modes,
            &self.sky,
            &self.lights,
        );
    }

    /// Add current shading state to pool (with deduplication)
    /// Returns the index into shading_states
    /// Uses the pre-packed state to avoid re-quantizing!
    pub fn add_shading_state(&mut self) -> u32 {
        let packed = self.current_packed_shading_state;

        // Check if already in pool (deduplication)
        if let Some(&idx) = self.shading_state_cache.get(&packed) {
            return idx;
        }

        // Add to pool
        let idx = self.shading_states.len() as u32;
        if idx >= 65536 {
            panic!("Shading state pool overflow! Maximum 65,536 unique states per frame.");
        }

        self.shading_states.push(packed);
        self.shading_state_cache.insert(packed, idx);
        idx
    }
}
```

**Key Architecture Decision:**

Both the shading state **pool** and the **current packed state** MUST live in `ZFFIState` because:

1. **Pool in ZFFIState:** The `shading_states` vector is reset each frame (like `model_matrices`)
2. **Current packed state in ZFFIState:** We quantize once when state changes, not on every draw
3. **Why necessary:** FFI functions need access to both unquantized (for manipulation) and packed (for pooling) state

This mirrors the matrix packing pattern exactly - ZFFIState owns both the pools and the current state.

#### 2.2: Update clear_frame to Reset Shading State Pool

**File:** `emberware-z/src/state.rs`

```rust
pub fn clear_frame(&mut self) {
    self.render_pass.reset();
    self.model_matrices.clear();
    self.model_matrices.push(Mat4::IDENTITY);
    self.deferred_commands.clear();
    self.audio_commands.clear();

    // NEW: Clear shading state pool
    self.shading_states.clear();
    self.shading_state_cache.clear();
}
```

**Note:** Everything lives in ZFFIState (not ZGraphics), just like matrices! This includes:
- ✅ `shading_states: Vec<PackedUnifiedShadingState>` - The per-frame pool
- ✅ `shading_state_cache: HashMap<...>` - Deduplication map
- ✅ `current_packed_shading_state: PackedUnifiedShadingState` - Pre-packed current state
- ✅ Unquantized fields (`metallic`, `roughness`, `sky`, `lights`, etc.) - Easy FFI access

This is **necessary** because FFI functions need direct access to both unquantized (for manipulation) and packed (for pooling) state.

---

## Phase 3: Update VRPCommand Structure

**Estimated Time:** 2-3 hours

### Files to Modify
- `emberware-z/src/graphics/command_buffer.rs`

### Changes

#### 3.1: Replace Individual State Fields

```rust
pub struct VRPCommand {
    pub format: u8,
    pub mvp_index: MvpIndex,                 // From matrix packing refactor
    pub vertex_count: u32,
    pub index_count: u32,
    pub base_vertex: u32,
    pub first_index: u32,
    pub buffer_source: BufferSource,
    pub texture_slots: [TextureHandle; 4],

    // NEW: Index into ZFFIState::shading_states (newtype for clarity)
    pub shading_state_index: ShadingStateIndex,

    // Keep these for pipeline selection (not in shading state)
    pub depth_test: bool,
    pub cull_mode: CullMode,

    // REMOVED (now in PackedUnifiedShadingState):
    // pub color: u32,
    // pub blend_mode: BlendMode,
    // pub matcap_blend_modes: [MatcapBlendMode; 4],
}
```

**Note:**
- `depth_test` and `cull_mode` affect pipeline selection, so they remain separate
- `shading_state_index` is a newtype for type safety (consistent with other handle types)

#### 3.2: Update VirtualRenderPass Methods

```rust
pub fn record_triangles(
    &mut self,
    format: u8,
    vertex_data: &[f32],
    mvp_index: MvpIndex,
    texture_slots: [TextureHandle; 4],
    shading_state_index: ShadingStateIndex,  // NEW: index into shading_states pool
    depth_test: bool,          // Keep for pipeline key
    cull_mode: CullMode,       // Keep for pipeline key
) {
    let format_idx = format as usize;
    let stride = vertex_stride(format) as usize;
    let vertex_count = (vertex_data.len() * 4) / stride;
    let base_vertex = self.vertex_counts[format_idx];

    // Write vertex data
    let byte_data = bytemuck::cast_slice(vertex_data);
    self.vertex_data[format_idx].extend_from_slice(byte_data);
    self.vertex_counts[format_idx] += vertex_count as u32;

    self.commands.push(VRPCommand {
        format,
        mvp_index,
        vertex_count: vertex_count as u32,
        index_count: 0,
        base_vertex,
        first_index: 0,
        buffer_source: BufferSource::Immediate,
        texture_slots,
        shading_state_index,
        depth_test,
        cull_mode,
    });
}

// Similar updates for record_triangles_indexed, record_mesh, etc.
```

---

## Phase 4: Update FFI Layer to Quantize State

**Estimated Time:** 6-8 hours (touches many FFI functions)

### Files to Modify
- `emberware-z/src/state.rs`
- `emberware-z/src/ffi/mod.rs`

### Changes

#### 4.1: Add `mark_shading_state_dirty()` Calls

**File:** `emberware-z/src/ffi/mod.rs`

**IMPORTANT: FFI functions quantize float inputs immediately and store quantized values in ZFFIState.**

```rust
// Material property setters (quantize on input)
fn material_set_metallic(caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, metallic: f32) {
    let state = &mut caller.data_mut().console;
    let quantized = (metallic.clamp(0.0, 1.0) * 255.0).round() as u8;

    // Only update if quantized value changed (avoids redundant packing)
    if (state.metallic * 255.0).round() as u8 != quantized {
        state.metallic = metallic;
        state.mark_shading_state_dirty();
    }
}

fn material_set_roughness(caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, roughness: f32) {
    let state = &mut caller.data_mut().console;
    let quantized = (roughness.clamp(0.0, 1.0) * 255.0).round() as u8;

    if (state.roughness * 255.0).round() as u8 != quantized {
        state.roughness = roughness;
        state.mark_shading_state_dirty();
    }
}

// Sky setters (quantize Vec3 inputs, store in PackedSky)
fn sky_set_colors(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    horizon_r: f32, horizon_g: f32, horizon_b: f32,
    zenith_r: f32, zenith_g: f32, zenith_b: f32,
) {
    let state = &mut caller.data_mut().console;

    let new_horizon = pack_rgb8_to_u32(Vec3::new(horizon_r, horizon_g, horizon_b));
    let new_zenith = pack_rgb8_to_u32(Vec3::new(zenith_r, zenith_g, zenith_b));

    // Only update if quantized values changed
    if state.sky.horizon_color != new_horizon || state.sky.zenith_color != new_zenith {
        state.sky.horizon_color = new_horizon;
        state.sky.zenith_color = new_zenith;
        state.mark_shading_state_dirty();
    }
}

// Light setters (quantize inputs, store in PackedLight)
fn light_set(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    index: u32,
    dir_x: f32, dir_y: f32, dir_z: f32,
    color_r: f32, color_g: f32, color_b: f32,
    intensity: f32,
    enabled: u32,
) {
    let state = &mut caller.data_mut().console;
    let idx = index as usize;

    let new_light = PackedLight::from_floats(
        Vec3::new(dir_x, dir_y, dir_z),
        Vec3::new(color_r, color_g, color_b),
        intensity,
        enabled != 0,
    );

    // Only update if quantized values changed
    if state.lights[idx] != new_light {
        state.lights[idx] = new_light;
        state.mark_shading_state_dirty();
    }
}

// Matcap blend mode setters (already discrete, just check equality)
fn matcap_set_blend_mode(caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, slot: u32, mode: u32) {
    let state = &mut caller.data_mut().console;
    let new_mode = MatcapBlendMode::from_u32(mode).unwrap_or(MatcapBlendMode::Multiply);

    if state.matcap_blend_modes[slot as usize] != new_mode {
        state.matcap_blend_modes[slot as usize] = new_mode;
        state.mark_shading_state_dirty();
    }
}
```

**Key Architecture Change:**

- **FFI functions receive floats** (developer-friendly API)
- **FFI functions quantize immediately** (store quantized in ZFFIState)
- **Only mark dirty if quantized values changed** (avoids redundant packing)
- **Remove `DeferredCommand::SetSky`** (sky is now per-draw state, not deferred)

**Functions that need `mark_shading_state_dirty()`:**
- All `material_*` setters (metallic, roughness, emissive)
- All `sky_*` setters (colors, sun direction/color/sharpness)
- All `light_*` setters (direction, color, intensity, enabled)
- All `matcap_*` blend mode setters

#### 4.2: Update FFI Draw Functions

**File:** `emberware-z/src/ffi/mod.rs`

The pattern is simple - just add current shading state to the pool before recording the draw command (uses the pre-packed state):

```rust
fn draw_triangles(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    format: u32,
    ptr: u32,
    vertex_count: u32,
) -> Result<(), Trap> {
    let state = &mut caller.data_mut().console;

    // ... existing vertex data copy

    // Pack current transform into model matrix pool
    let model_idx = state.add_model_matrix(state.current_transform)
        .expect("Model matrix pool overflow");

    let mvp_index = crate::graphics::MvpIndex::new(
        model_idx,
        state.current_view_idx,
        state.current_proj_idx,
    );

    // NEW: Pack current shading state into pool (with deduplication)
    let shading_state_idx = state.add_shading_state();

    state.render_pass.record_triangles(
        format as u8,
        &vertex_data,
        mvp_index,
        state.texture_slots,
        ShadingStateIndex(shading_state_idx),  // NEW: pass shading state index
        state.depth_test,
        state.cull_mode,
    );

    Ok(())
}
```

**Note:** This mirrors the matrix packing pattern exactly! No temporary storage or deferred processing needed.

---

## Phase 5: Update Render Pass Execution

**Estimated Time:** 3-4 hours

### Files to Modify
- `emberware-z/src/graphics/mod.rs`

### Changes

#### 5.1: Upload Shading States Before Rendering

**File:** `emberware-z/src/graphics/mod.rs`

```rust
pub fn render_frame(&mut self, view: &TextureView, z_state: &mut ZFFIState, clear_color: [f32; 4]) -> Result<()> {
    // 1. Upload matrices
    let matrix_data = bytemuck::cast_slice(&z_state.model_matrices);
    self.queue.write_buffer(&self.model_matrix_buffer, 0, matrix_data);

    let view_data = bytemuck::cast_slice(&z_state.view_matrices);
    self.queue.write_buffer(&self.view_matrix_buffer, 0, view_data);

    let proj_data = bytemuck::cast_slice(&z_state.proj_matrices);
    self.queue.write_buffer(&self.proj_matrix_buffer, 0, proj_data);

    // 2. Upload shading states (NEW)
    let shading_data = bytemuck::cast_slice(&z_state.shading_states);
    self.queue.write_buffer(&self.shading_state_buffer, 0, shading_data);

    // 3. Upload MVP + shading state indices
    let mut mvp_shading_indices = Vec::with_capacity(self.command_buffer.commands().len());
    for cmd in self.command_buffer.commands() {
        mvp_shading_indices.push([
            cmd.mvp_index.0,                // .x: packed MVP
            cmd.shading_state_index.0,      // .y: shading state index
        ]);
    }
    let indices_data = bytemuck::cast_slice(&mvp_shading_indices);
    self.queue.write_buffer(&self.mvp_indices_buffer, 0, indices_data);

    // 4. Upload immediate vertex/index data
    // ...

    // 5. Sort commands
    self.sort_commands();

    // 6. Execute render pass
    // ...
}
```

#### 5.2: Update Command Sorting

```rust
fn sort_commands(&mut self, z_state: &ZFFIState) {
    self.command_buffer.commands_mut().sort_unstable_by_key(|cmd| {
        // Extract blend mode from shading state
        let blend_mode = if let Some(state) = z_state.shading_states.get(cmd.shading_state_index.0 as usize) {
            (state.blend_modes & 0xFF) as u8
        } else {
            0
        };

        (
            self.render_mode,              // Mode (0-3)
            cmd.format,                    // Vertex format (0-15)
            blend_mode,                    // Blend mode (extracted from shading state)
            cmd.texture_slots[0].0,        // Primary texture
            cmd.shading_state_index.0,     // Material (NEW: sort by shading state index)
        )
    });
}
```

**Note:** Pass `z_state` to `sort_commands` to access the shading states pool.

#### 5.3: Upload MVP + Shading State Indices Buffer

**File:** `emberware-z/src/graphics/mod.rs` (in `render_frame`)

```rust
// Build combined MVP + shading state indices buffer
let mut mvp_shading_indices = Vec::with_capacity(self.command_buffer.commands().len());
for cmd in self.command_buffer.commands() {
    // Each entry is vec2<u32>: [packed_mvp, shading_state_index]
    mvp_shading_indices.push([
        cmd.mvp_index.0,                    // .x: packed MVP indices
        cmd.shading_state_index.0,          // .y: shading state index
    ]);
}

// Upload to GPU (replaces existing MVP indices upload)
let indices_data = bytemuck::cast_slice(&mvp_shading_indices);
self.queue.write_buffer(&self.mvp_indices_buffer, 0, indices_data);
```

**Note:** This replaces the existing MVP indices upload. The buffer size calculation remains the same since we were already using `vec2<u32>`.

---

## Phase 6: Update Shaders to Read Shading States

**Estimated Time:** 6-8 hours (all 4 shader templates)

### Files to Modify
- `emberware-z/shaders/mode0_unlit.wgsl`
- `emberware-z/shaders/mode1_matcap.wgsl`
- `emberware-z/shaders/mode2_pbr.wgsl`
- `emberware-z/shaders/mode3_hybrid.wgsl`

### Changes (Apply to All 4 Templates)

#### 6.1: Unified Binding Layout (ALL MODES)

All 4 render modes now use the **same binding layout**:

```wgsl
// Bindings 0-2: Matrix pools (per-frame arrays)
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;

// Binding 3: Shading states pool (per-frame array, contains sky/lights/material)
@group(0) @binding(3) var<storage, read> shading_states: array<UnifiedShadingState>;

// Binding 4: Per-draw indices (2 × u32: [packed_mvp, shading_state_index])
@group(0) @binding(4) var<storage, read> mvp_shading_indices: array<vec2<u32>>;

// Binding 5: Bone matrices pool (per-frame array, optional for skinning)
@group(0) @binding(5) var<storage, read> bones: array<mat4x4<f32>>;
```

**Logical grouping:**
- **Bindings 0-3:** Data buffers (matrices, shading states)
- **Bindings 4-5:** Indices/structural (per-draw indices, bones)

**Packed structures (must match Rust layout EXACTLY):**

```wgsl
struct UnifiedShadingState {
    // First 4 bytes: metallic, roughness, emissive, pad
    params_packed: u32,

    color_rgba8: u32,
    blend_modes: u32,
    _pad: u32,  // Alignment to 16 bytes

    // Sky (16 bytes)
    sky_horizon: u32,
    sky_zenith: u32,
    sky_sun_dir: vec2<i32>,
    sky_sun_color: u32,
    _pad_sky: u32,

    // Lights (64 bytes = 16 bytes × 4)
    light0_dir: vec2<i32>,
    light0_color: u32,
    _pad_l0: u32,

    light1_dir: vec2<i32>,
    light1_color: u32,
    _pad_l1: u32,

    light2_dir: vec2<i32>,
    light2_color: u32,
    _pad_l2: u32,

    light3_dir: vec2<i32>,
    light3_color: u32,
    _pad_l3: u32,
}
```

**Key benefits:**
- ✅ Same binding layout for ALL modes (0-3)
- ✅ No off-by-N errors between modes
- ✅ Bones always at binding 5 (consistent)
- ✅ Sky, material, lights, camera all contained in shading_states
- ✅ No redundant uniform bindings

#### 6.2: Add Unpacking Helpers

```wgsl
// Unpack RGBA8 from u32
fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32((packed >> 24u) & 0xFFu) / 255.0;
    let g = f32((packed >> 16u) & 0xFFu) / 255.0;
    let b = f32((packed >> 8u) & 0xFFu) / 255.0;
    let a = f32(packed & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

// Unpack RGB8 from u32 (alpha is something else)
fn unpack_rgb8(packed: u32) -> vec3<f32> {
    let r = f32((packed >> 24u) & 0xFFu) / 255.0;
    let g = f32((packed >> 16u) & 0xFFu) / 255.0;
    let b = f32((packed >> 8u) & 0xFFu) / 255.0;
    return vec3<f32>(r, g, b);
}

// Extract alpha channel (used for scalar values)
fn unpack_alpha(packed: u32) -> f32 {
    return f32(packed & 0xFFu) / 255.0;
}

// Convert snorm16 to f32 (-1 to 1)
fn unpack_snorm16(packed: i32, which: u32) -> f32 {
    let value = select(
        (packed & 0xFFFF),        // Low 16 bits (which == 0)
        (packed >> 16) & 0xFFFF,  // High 16 bits (which == 1)
        which == 1u
    );
    // Sign extend from 16 bits
    let signed = select(value, value | 0xFFFF0000, (value & 0x8000u) != 0u);
    return f32(signed) / 32767.0;
}

// Unpack vec3 from two i32s
fn unpack_snorm16_vec3(xy: i32, z_w: i32) -> vec3<f32> {
    return vec3<f32>(
        unpack_snorm16(xy, 0u),
        unpack_snorm16(xy, 1u),
        unpack_snorm16(z_w, 0u)
    );
}

// Check if light is enabled (w component of direction)
fn is_light_enabled(z_enabled: i32) -> bool {
    let enabled = unpack_snorm16(z_enabled, 1u);
    return enabled > 0.0;
}
```

#### 6.3: Update Vertex Shader to Pass Shading State Index

```wgsl
@vertex
fn vs(in: VertexIn, @builtin(instance_index) instance_index: u32) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Get packed MVP indices from storage buffer using instance index
    let indices = mvp_shading_indices[instance_index];
    let mvp_packed = indices.x;
    let shading_state_idx = indices.y;  // NEW: extract shading state index

    let model_idx = mvp_packed & 0xFFFFu;
    let view_idx = (mvp_packed >> 16u) & 0xFFu;
    let proj_idx = (mvp_packed >> 24u) & 0xFFu;

    // ... rest of vertex shader

    // Pass shading state index to fragment shader
    out.shading_state_index = shading_state_idx;  // NEW: add to VertexOut

    return out;
}
```

**Note:** Add `shading_state_index: u32` to the `VertexOut` struct (use `@location(N)` with appropriate N for each shader).

#### 6.4: Update Fragment Shader

```wgsl
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fetch shading state for this draw (via vertex shader)
    let state = shading_states[in.shading_state_index];

    // Unpack PBR params
    let params = state.params_packed;
    let metallic = f32((params >> 24u) & 0xFFu) / 255.0;
    let roughness = f32((params >> 16u) & 0xFFu) / 255.0;
    let emissive = f32((params >> 8u) & 0xFFu) / 255.0;

    // Unpack base color
    let base_color = unpack_rgba8(state.color_rgba8);

    // Sample texture (if UV present)
    //FS_SAMPLE_ALBEDO

    // Apply color tint
    var albedo = base_color;
    //FS_APPLY_TEXTURE

    // Unpack sky
    let sky_horizon = unpack_rgba8(state.sky_horizon);
    let sky_zenith = unpack_rgba8(state.sky_zenith);
    let sky_sun_dir = normalize(unpack_snorm16_vec3(state.sky_sun_dir.x, state.sky_sun_dir.y));
    let sky_sun_color = unpack_rgb8(state.sky_sun_color);
    let sky_sun_sharpness = unpack_alpha(state.sky_sun_color);

    // Unpack lights
    let light0_dir = unpack_snorm16_vec3(state.light0_dir.x, state.light0_dir.y);
    let light0_enabled = is_light_enabled(state.light0_dir.y);
    let light0_color = unpack_rgb8(state.light0_color);
    let light0_intensity = unpack_alpha(state.light0_color);

    // ... similar for light1, light2, light3

    // Lighting calculations (use unpacked values)
    //FS_LIGHTING

    return final_color;
}
```

**Note:** The exact unpacking logic must match the packing in Rust EXACTLY. Test thoroughly!

---

## Phase 7: Update Bind Group Layouts

**Estimated Time:** 2-3 hours

### Files to Modify
- `emberware-z/src/graphics/pipeline.rs`
- `emberware-z/src/graphics/mod.rs`

### Changes

#### 7.1: Unified Bind Group Layout (ALL MODES)

**File:** `emberware-z/src/graphics/pipeline.rs`

**Replace** the entire `create_frame_bind_group_layout` function with a unified layout that works for all modes:

```rust
/// Create bind group layout for per-frame uniforms (group 0)
/// This layout is now IDENTICAL for all render modes!
fn create_frame_bind_group_layout(device: &wgpu::Device, _render_mode: u8) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("Frame Bind Group Layout"),
        entries: &[
            // Binding 0: Model matrices storage buffer
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Binding 1: View matrices storage buffer
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Binding 2: Projection matrices storage buffer
            wgpu::BindGroupLayoutEntry {
                binding: 2,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Binding 3: Shading states storage buffer (contains sky, lights, material)
            wgpu::BindGroupLayoutEntry {
                binding: 3,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,  // Fragment reads, vertex passes index
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Binding 4: MVP + shading state indices storage buffer
            wgpu::BindGroupLayoutEntry {
                binding: 4,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            // Binding 5: Bone matrices storage buffer (for GPU skinning)
            wgpu::BindGroupLayoutEntry {
                binding: 5,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only: true },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}
```

**Key changes:**
- ✅ Removed render_mode switch - ALL modes use same layout now!
- ✅ Bindings 0-2: Matrix pools
- ✅ Binding 3: Shading states (NEW - replaces sky/material/lights/camera)
- ✅ Binding 4: MVP + shading indices (swapped from binding 3)
- ✅ Binding 5: Bones (consistent across all modes)
- ✅ Removed bindings 6-9 entirely (redundant)

#### 7.2: Create Unified Bind Group (ALL MODES)

**File:** `emberware-z/src/graphics/mod.rs`

Update the bind group creation in `render_frame` - now **identical for all modes**:

```rust
// Create bind group 0 (per-frame uniforms)
let bind_group_frame = device.create_bind_group(&wgpu::BindGroupDescriptor {
    label: Some("Frame Bind Group"),
    layout: &pipeline_entry.bind_group_layout_frame,
    entries: &[
        // Binding 0: Model matrices
        wgpu::BindGroupEntry {
            binding: 0,
            resource: self.model_matrix_buffer.as_entire_binding(),
        },
        // Binding 1: View matrices
        wgpu::BindGroupEntry {
            binding: 1,
            resource: self.view_matrix_buffer.as_entire_binding(),
        },
        // Binding 2: Projection matrices
        wgpu::BindGroupEntry {
            binding: 2,
            resource: self.proj_matrix_buffer.as_entire_binding(),
        },
        // Binding 3: Shading states (NEW)
        wgpu::BindGroupEntry {
            binding: 3,
            resource: self.shading_state_buffer.as_entire_binding(),
        },
        // Binding 4: MVP + shading state indices
        wgpu::BindGroupEntry {
            binding: 4,
            resource: self.mvp_indices_buffer.as_entire_binding(),
        },
        // Binding 5: Bones
        wgpu::BindGroupEntry {
            binding: 5,
            resource: self.bone_buffer.as_entire_binding(),
        },
    ],
});
```

**Key changes:**
- ✅ No render_mode switch needed!
- ✅ All 6 bindings present for all modes
- ✅ Binding 3 is the new shading_state_buffer
- ✅ Binding 4 is the mvp_indices_buffer (swapped)

---

## Phase 8: Update Pipeline Extraction

**Estimated Time:** 2-3 hours

### Files to Modify
- `emberware-z/src/graphics/pipeline.rs`
- `emberware-z/src/graphics/mod.rs`

### Changes

#### 8.1: Extract Pipeline State from Shading State

```rust
fn extract_pipeline_key(
    cmd: &VRPCommand,
    z_state: &ZFFIState,
    render_mode: u8,
) -> PipelineKey {
    // Get actual shading state from pool
    let shading_state = &z_state.shading_states[cmd.shading_state_index.0 as usize];

    // Extract blend mode from packed state
    let blend_mode = (shading_state.blend_modes & 0xFF) as u8;

    PipelineKey {
        render_mode,
        vertex_format: cmd.format,
        blend_mode,
        depth_test: cmd.depth_test,
        cull_mode: cmd.cull_mode,
    }
}
```

---

## Phase 9: Testing and Validation

**Estimated Time:** 6-8 hours

### Test Cases

1. **Same Material, Multiple Draws**
   - Set material once, draw 100 triangles
   - Verify only 1 shading state interned
   - Visual: All triangles have same material

2. **Different Materials**
   - Draw triangles with varying metallic/roughness
   - Verify cache grows with unique states
   - Visual: Materials differ correctly

3. **Sky Changes**
   - Change sky colors/sun direction
   - Verify quantization doesn't lose quality (tolerance: 1/255)
   - Visual: Sky looks correct

4. **Dynamic Lights**
   - Animate light positions/colors
   - Verify packed lights work correctly
   - Visual: Lighting updates smoothly

5. **All 4 Render Modes**
   - Test Unlit, Matcap, PBR, Hybrid
   - Verify shaders access state correctly
   - Visual: Each mode renders correctly

6. **Cache Efficiency**
   - Draw same materials across multiple frames
   - Verify cache clears/rebuilds each frame
   - Verify high hit rate for repeated materials

### Validation Checklist

- [ ] Visual: All test cases match pre-refactor renderer (within quantization tolerance)
- [ ] Performance: Measure reduction in state changes
- [ ] Memory: VRPCommand size reduced significantly
- [ ] Cache efficiency: High hit rate for repeated materials
- [ ] Quantization: No visible artifacts from u8/snorm16 precision
- [ ] Per-draw materials: Verify different draws can have different material properties
- [ ] MVP + shading indices buffer: Verify correct packing and upload

### Performance Metrics

```rust
// Log shading state stats (in ZGraphics or app.rs)
let state_count = z_state.shading_states.len();
let state_bytes = state_count * std::mem::size_of::<PackedUnifiedShadingState>();
tracing::debug!(
    "Shading states: {} unique states ({} KB)",
    state_count,
    state_bytes / 1024
);
```

**Expected improvements:**
- VRPCommand size: ~120 bytes → ~40 bytes (67% reduction)
- Material uploads: Reduced by deduplication ratio (depends on game)
- Command sorting: Better batching by material handle

---

## Rollout Strategy

### 1. Incremental Deployment

1. **Day 1-2:** Phases 1-3 (structures, cache, VRPCommand)
2. **Day 3-4:** Phase 4 (FFI layer, state quantization)
3. **Day 5:** Phases 5, 7-8 (render execution, bind groups, pipeline)
4. **Day 6:** Phase 6 (shaders - most complex)
5. **Day 7:** Phase 9 (testing and validation)

### 2. Breaking Changes

This refactor includes breaking changes:
- VRPCommand structure changes (depends on matrix packing)
- Shader binding layout changes (unified bindings 0-5 for all modes)
- ZFFIState structure changes (removal of transform_stack, current_transform, camera)
- Removal of `DeferredCommand::SetSky` (sky is now per-draw state)
- FFI function signatures (sky/light setters quantize immediately)

**Impact:** Must implement after matrix packing. All pipelines regenerated. Acceptable pre-release.

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Quantization artifacts | Medium | Medium | Visual testing, adjust precision if needed |
| Shader unpacking bugs | High | High | Thorough testing, exact Rust/WGSL layout matching |
| Cache bloat | Low | Medium | Monitor size, typical games have few unique materials |
| Complex debugging | High | Medium | Add debug views, unpacking validation tests |
| Performance regression | Low | High | Benchmark before/after, profile |

---

## Success Criteria

- ✅ Visual quality matches pre-refactor (within quantization tolerance)
- ✅ VRPCommand size reduced by ~67%
- ✅ Material state uploads reduced (measure deduplication ratio)
- ✅ Better command batching (fewer state changes)
- ✅ No crashes or glitches
- ✅ Shading state cache is efficient (high hit rate)

---

## Follow-Up Work

1. **Cache persistence** - Consider keeping cache across frames for static materials
2. **Quantization tuning** - Adjust precision based on visual testing (u16/f16 if needed)
3. **Shader optimization** - Profile GPU performance after refactor
4. **WebGL fallback** - TODO: Per-draw uniforms if storage buffers unsupported

---

## Performance Considerations

### Why Packed GPU Unpacking?

The current plan unpacks quantized data on the GPU. Alternative approaches considered:

**Option 1: Packed + GPU Unpack (CURRENT PLAN)**
- ✅ Small GPU buffer (96 bytes/state vs ~200 bytes)
- ✅ Better deduplication (quantized → more exact matches)
- ✅ Less GPU memory bandwidth
- ✅ Unpacking is **once per draw** (not per fragment!)
- ✅ Modern GPUs cache unpacked values in registers
- ⚠️ Unpacking overhead (minimal - simple bit shifts)

**Option 2: Unpack on CPU, Send Full Floats**
- ✅ No GPU unpacking overhead
- ✅ Simpler shader code
- ❌ 2× larger GPU buffer (~200 bytes/state)
- ❌ Less deduplication (f32 precision → fewer matches)
- ❌ More GPU memory bandwidth
- ❌ Still need to upload every frame anyway

**Option 3: Compute Shader Pre-Unpack**
- ✅ Best of both worlds (compact storage + pre-unpacked)
- ❌ Extra GPU pass complexity
- ❌ Overkill for once-per-draw unpacking
- ❌ Harder to debug

**Decision:** Option 1 (packed) is optimal because:
1. Deduplication is critical (many draws share materials)
2. GPU unpacking is **once per draw**, cached for all fragments
3. Memory bandwidth savings are significant
4. Simpler architecture (no extra passes)

### Unpacking Frequency Clarification

**Q: Does unpacking happen per-fragment?**
**A: No! It happens once per draw.**

Here's why:
1. Vertex shader passes `shading_state_index` with `@interpolate(flat)`
2. This means the index is **constant** across all fragments in a triangle
3. Fragment shader reads `shading_states[in.shading_state_index]`
4. GPU detects constant index → caches fetched data in registers
5. Unpacked values are reused for all fragments in the draw

**Effective cost:** One storage buffer read + unpacking per draw (negligible).

### Future Optimizations (if profiling shows need)

1. **Hybrid approach:** Keep common states unpacked in a separate buffer
2. **Texture-based storage:** Use texture fetch instead of storage buffer
3. **Compute pre-pass:** Unpack in compute shader to separate buffer (overkill)
4. **CPU-side unpacking:** Trade deduplication for simpler shaders (not recommended)

---

## Integration with Matrix Packing

This refactor **requires** matrix packing to be implemented first, as it:
- Uses the second u32 in the `mvp_indices` buffer (already allocated as `vec2<u32>`)
- Depends on VRPCommand having `mvp_index` instead of `transform`
- Leverages the same instance index indirection infrastructure

**Storage buffer structure (shared):**
```wgsl
// Per-frame storage buffer - packed MVP + shading state indices
@group(0) @binding(3) var<storage, read> mvp_shading_indices: array<vec2<u32>>;

// In vertex shader:
let indices = mvp_shading_indices[instance_index];
let mvp_packed = indices.x;          // Matrix packing uses .x
let shading_state_idx = indices.y;   // Unified shading state uses .y
```

**Key implementation details:**
- ✅ No push constants required (GPU doesn't support them)
- ✅ Uses existing instance index indirection
- ✅ Single storage buffer for all per-draw indices
- ✅ Shading state index passed from vertex → fragment shader via interpolator

---

**Last Updated:** December 2024 (Major revision - simplified approach)
**Status:** Ready for implementation (after matrix packing)

---

## Implementation Summary

This plan was significantly simplified from the original by:
1. **No push constants** - Uses instance index indirection via existing `vec2<u32>` storage buffer
2. **Shading state pool in ZFFIState** - Mirrors matrix packing approach, no separate cache in ZGraphics
3. **Automatic deduplication** - Hash-based deduplication happens in `add_shading_state()`
4. **Consistent with matrix packing** - Same pattern, same infrastructure, same cleanup
5. **Quantized storage in ZFFIState** - Sky and lights stored as `PackedSky`/`PackedLight` directly
6. **FFI quantization** - FFI functions quantize float inputs immediately, check for changes before marking dirty

### Key Architecture Decisions

1. **Why store PackedSky/PackedLight in ZFFIState?**
   - FFI functions quantize once on input (not on every draw)
   - Only mark dirty if quantized values actually changed
   - Avoids redundant quantization when state doesn't change
   - Simpler than maintaining parallel unquantized + quantized state
   - **Quantization point:** At FFI barrier (user passes f32, we store snorm16/u8)

2. **Why keep metallic/roughness/emissive as f32 but lights/sky as packed?**
   - **Material scalars (f32):** Small, easy to compare, quantized only during `mark_dirty`
   - **Lights/sky (packed):** Complex structures, quantize at FFI barrier for consistency
   - Both approaches avoid redundant packing - just at different points

3. **Why remove transform_stack and camera?**
   - Replaced by matrix pool system (model_matrices, view_matrices, proj_matrices)
   - Cleaner separation: transformations via matrices, not separate camera struct
   - Consistent with matrix packing refactor
   - **Not used in actual game logic** - legacy from old system

4. **Why remove DeferredCommand::SetSky?**
   - Sky is now per-draw state (part of UnifiedShadingState)
   - Each draw command can have different sky parameters
   - Eliminates frame-wide sky state (was a bug - should be per-draw)
   - Default sky is all zeros (black, no sun) - games set in init via FFI

5. **Why use ShadingStateIndex newtype?**
   - Type safety: prevents mixing up shading state indices with other u32 values
   - Consistent with other handle types (TextureHandle, MvpIndex)
   - Makes code more self-documenting

6. **Why unified binding layout (0-5) for all modes?**
   - Eliminates off-by-N errors (bones at 6 vs 8 in different modes)
   - Simpler maintenance (one bind group layout instead of mode-specific)
   - See [binding-layout-migration.md](./binding-layout-migration.md) for details

The key insight: the `mvp_indices` buffer was always `vec2<u32>` with the second u32 reserved for exactly this purpose (left as breadcrumbs). This implementation simply populates `.y` with shading_state_index.
