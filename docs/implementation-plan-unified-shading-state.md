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
- Material state becomes hashable and comparable
- Same material used across draws = one GPU upload (deduplication)
- Better command sorting by material
- Reduced VRPCommand size (remove separate state fields)

**Approach:** Storage buffer indexed via instance index (extends existing MVP indices buffer)

The MVP indices buffer is already `array<vec2<u32>>` where:
- `.x` = packed MVP indices (model: 16 bits, view: 8 bits, proj: 8 bits)
- `.y` = unified shading state index (reserved for this implementation)

**Complexity:** Medium-High - touches FFI, command recording, shaders, and GPU upload, but leverages existing infrastructure

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

/// Handle to interned shading state
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct UnifiedShadingStateHandle(pub u32);

impl UnifiedShadingStateHandle {
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
        sky: &Sky,
        lights: &[Light; 4],
    ) -> Self {
        Self {
            metallic: quantize_f32_to_u8(metallic),
            roughness: quantize_f32_to_u8(roughness),
            emissive: quantize_f32_to_u8(emissive),
            pad0: 0,

            color_rgba8: color,
            blend_modes: pack_blend_modes(matcap_blend_modes),

            sky: PackedSky::from_sky(sky),
            lights: [
                PackedLight::from_light(&lights[0]),
                PackedLight::from_light(&lights[1]),
                PackedLight::from_light(&lights[2]),
                PackedLight::from_light(&lights[3]),
            ],
        }
    }
}

impl PackedSky {
    pub fn from_sky(sky: &Sky) -> Self {
        Self {
            horizon_color: pack_rgba8_from_vec4(sky.horizon_color),
            zenith_color: pack_rgba8_from_vec4(sky.zenith_color),
            sun_direction: quantize_vec3_to_snorm16(sky.sun_direction),
            sun_color_and_sharpness: pack_color_and_scalar(
                sky.sun_color,
                sky.sun_sharpness,
            ),
        }
    }
}

impl PackedLight {
    pub fn from_light(light: &Light) -> Self {
        Self {
            direction: quantize_vec3_to_snorm16_with_flag(
                light.direction,
                light.enabled,
            ),
            color_and_intensity: pack_color_and_scalar(
                light.color,
                light.intensity,
            ),
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

fn pack_rgba8_from_vec4(color: Vec4) -> u32 {
    let r = (color.x.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g = (color.y.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b = (color.z.clamp(0.0, 1.0) * 255.0).round() as u32;
    let a = (color.w.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r << 24) | (g << 16) | (b << 8) | a
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

## Phase 2: Implement Shading State Cache

**Estimated Time:** 4-6 hours

### Files to Modify
- `emberware-z/src/graphics/unified_shading_state.rs` (extend)
- `emberware-z/src/graphics/mod.rs`

### Changes

#### 2.1: Add Shading State Pool to ZFFIState

**File:** `emberware-z/src/state.rs`

```rust
use hashbrown::HashMap;

pub struct ZFFIState {
    // Existing fields...

    // Current unquantized shading state (set by FFI functions)
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
    pub matcap_blend_modes: [MatcapBlendMode; 4],
    // sky, lights already exist

    // Shading state pool (reset each frame, similar to model_matrices)
    pub shading_states: Vec<PackedUnifiedShadingState>,
    shading_state_cache: HashMap<PackedUnifiedShadingState, u32>,  // For deduplication
}

impl Default for ZFFIState {
    fn default() -> Self {
        Self {
            // Existing initialization...
            metallic: 0.0,
            roughness: 1.0,
            emissive: 0.0,
            matcap_blend_modes: [MatcapBlendMode::Multiply; 4],
            shading_states: Vec::with_capacity(256),
            shading_state_cache: HashMap::new(),
        }
    }
}

impl ZFFIState {
    /// Pack current shading state and add to pool (with deduplication)
    /// Returns the index into shading_states
    pub fn add_shading_state(&mut self) -> u32 {
        let packed = PackedUnifiedShadingState::from_render_state(
            self.color,
            self.metallic,
            self.roughness,
            self.emissive,
            &self.matcap_blend_modes,
            &self.sky,
            &self.lights,
        );

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

**Note:** No separate cache in ZGraphics needed - everything is in ZFFIState, just like matrices!

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

    // NEW: Index into ZFFIState::shading_states
    pub shading_state_index: u32,

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
- `shading_state_index` is just a u32 index, not a handle type (simpler, consistent with matrix approach)

#### 3.2: Update VirtualRenderPass Methods

```rust
pub fn record_triangles(
    &mut self,
    format: u8,
    vertex_data: &[f32],
    mvp_index: MvpIndex,
    texture_slots: [TextureHandle; 4],
    shading_state_index: u32,  // NEW: index into shading_states pool
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

#### 4.1: Update FFI Draw Functions

**File:** `emberware-z/src/ffi/mod.rs`

The pattern is simple - just add current shading state to the pool before recording the draw command:

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
        shading_state_idx,  // NEW: pass shading state index
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
            cmd.mvp_index.0,           // .x: packed MVP
            cmd.shading_state_index,   // .y: shading state index
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
        let blend_mode = if let Some(state) = z_state.shading_states.get(cmd.shading_state_index as usize) {
            (state.blend_modes & 0xFF) as u8
        } else {
            0
        };

        (
            self.render_mode,           // Mode (0-3)
            cmd.format,                 // Vertex format (0-15)
            blend_mode,                 // Blend mode (extracted from shading state)
            cmd.texture_slots[0].0,     // Primary texture
            cmd.shading_state_index,    // Material (NEW: sort by shading state index)
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
        cmd.shading_state_handle.0,         // .y: shading state index
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

#### 6.1: Rename MVP Indices Buffer and Add Shading State Buffer

```wgsl
// Per-frame storage buffer - packed MVP + shading state indices
// Each entry is 2 × u32: [packed_mvp, shading_state_index]
@group(0) @binding(3) var<storage, read> mvp_shading_indices: array<vec2<u32>>;

// Packed structures (must match Rust layout EXACTLY)
struct PackedSky {
    horizon_color: u32,           // Will be unpacked to vec4<f32>
    zenith_color: u32,
    sun_direction_x: i32,         // snorm16 (low 16 bits)
    sun_direction_yz: i32,        // snorm16 (x: y, y: z)
    sun_color_and_sharpness: u32,
}

struct PackedLight {
    direction_xy: i32,            // snorm16 (x: x, y: y)
    direction_z_enabled: i32,     // snorm16 (x: z, y: enabled)
    color_and_intensity: u32,
}

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

// Shading state buffer (binding varies by mode)
// Mode 0/1: @group(0) @binding(7) (after bones at 6)
// Mode 2/3: @group(0) @binding(9) (after bones at 8)
@group(0) @binding(BINDING_SHADING_STATE) var<storage, read> shading_states: array<UnifiedShadingState>;
```

**Note:** Use the correct binding number for each shader template:
- `mode0_unlit.wgsl` and `mode1_matcap.wgsl`: binding 7
- `mode2_pbr.wgsl` and `mode3_hybrid.wgsl`: binding 9

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

#### 7.1: Add Shading State Buffer to Bind Group 0

**File:** `emberware-z/src/graphics/pipeline.rs`

Update `create_frame_bind_group_layout` for each render mode:

**Mode 0/1 (Unlit, Matcap):**
```rust
// Existing bindings 0-6 (model, view, proj, mvp_indices, sky, material, bones)

// Binding 7: Shading states (NEW)
BindGroupLayoutEntry {
    binding: 7,
    visibility: ShaderStages::FRAGMENT,  // Fragment shader reads this
    ty: BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only: true },
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
},
```

**Mode 2/3 (PBR, Hybrid):**
```rust
// Existing bindings 0-8 (model, view, proj, mvp_indices, sky, material, lights, camera, bones)

// Binding 9: Shading states (NEW)
BindGroupLayoutEntry {
    binding: 9,
    visibility: ShaderStages::FRAGMENT,  // Fragment shader reads this
    ty: BindingType::Buffer {
        ty: BufferBindingType::Storage { read_only: true },
        has_dynamic_offset: false,
        min_binding_size: None,
    },
    count: None,
},
```

#### 7.2: Create Bind Group with Shading State Buffer

**File:** `emberware-z/src/graphics/mod.rs`

Update the bind group creation in `render_frame` to include the shading state buffer:

**Mode 0/1:**
```rust
// ... existing bindings 0-6

// Binding 7: Shading states
BindGroupEntry {
    binding: 7,
    resource: self.shading_state_cache.buffer().as_entire_binding(),
},
```

**Mode 2/3:**
```rust
// ... existing bindings 0-8

// Binding 9: Shading states
BindGroupEntry {
    binding: 9,
    resource: self.shading_state_cache.buffer().as_entire_binding(),
},
```

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
    let shading_state = &z_state.shading_states[cmd.shading_state_index as usize];

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
- Shader binding layout changes (new binding 6)
- Push constants usage (VERTEX + FRAGMENT stages)

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

The key insight: the `mvp_indices` buffer was always `vec2<u32>` with the second u32 reserved for exactly this purpose. This implementation simply uses that reserved slot.
