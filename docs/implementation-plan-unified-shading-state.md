# Implementation Plan: Unified Shading State

**Status:** Not Started (implement after matrix packing)
**Estimated Effort:** 4-6 days
**Priority:** Medium (implement second)
**Depends On:** Matrix index packing (required - uses push constants slot)
**Related:** [proposed-render-architecture.md](./proposed-render-architecture.md), [rendering-architecture.md](./rendering-architecture.md)

---

## Overview

Quantize all per-draw material state into a hashable POD structure (`PackedUnifiedShadingState`), implement interning to deduplicate identical materials, and enable better batching/sorting.

**Benefits:**
- Material state becomes hashable and comparable
- Same material used across draws = one GPU upload
- Better command sorting by material
- Reduced VRPCommand size (remove separate state fields)
- All per-draw state packaged together (self-contained)

**Approach:** Storage buffer + push constants (integrates with matrix packing's existing push constant infrastructure)

**Complexity:** High - touches FFI, command recording, shaders, and GPU upload

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

#### 2.1: Add Cache Structure

**File:** `emberware-z/src/graphics/unified_shading_state.rs`

```rust
use hashbrown::HashMap;
use wgpu::{Buffer, BufferDescriptor, BufferUsages, Device, Queue};

pub struct ShadingStateCache {
    // Map: packed state → handle
    cache: HashMap<PackedUnifiedShadingState, UnifiedShadingStateHandle>,

    // All unique states (indexed by handle)
    states: Vec<PackedUnifiedShadingState>,

    // GPU buffer
    states_buffer: Buffer,
    buffer_capacity: usize,
    dirty: bool,
}

impl ShadingStateCache {
    pub fn new(device: &Device) -> Self {
        let initial_capacity = 256;
        let states_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("Shading States"),
            size: (initial_capacity * std::mem::size_of::<PackedUnifiedShadingState>()) as u64,
            usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            cache: HashMap::new(),
            states: Vec::new(),
            states_buffer,
            buffer_capacity: initial_capacity,
            dirty: false,
        }
    }

    /// Intern a shading state, returning its handle
    pub fn intern(&mut self, state: PackedUnifiedShadingState) -> UnifiedShadingStateHandle {
        // Check if already cached
        if let Some(&handle) = self.cache.get(&state) {
            return handle;
        }

        // Allocate new handle
        let handle = UnifiedShadingStateHandle(self.states.len() as u32);
        if handle.0 >= 65536 {
            panic!("Shading state cache overflow! Maximum 65,536 unique states per frame.");
        }

        self.states.push(state);
        self.cache.insert(state, handle);
        self.dirty = true;

        tracing::trace!("Interned new shading state: {:?}", handle);
        handle
    }

    /// Upload dirty states to GPU
    pub fn upload(&mut self, device: &Device, queue: &Queue) {
        if !self.dirty {
            return;
        }

        // Grow buffer if needed
        if self.states.len() > self.buffer_capacity {
            let new_capacity = (self.states.len() * 2).next_power_of_two();
            tracing::debug!(
                "Growing shading state buffer: {} → {} states",
                self.buffer_capacity,
                new_capacity
            );

            self.states_buffer = device.create_buffer(&BufferDescriptor {
                label: Some("Shading States"),
                size: (new_capacity * std::mem::size_of::<PackedUnifiedShadingState>()) as u64,
                usage: BufferUsages::STORAGE | BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.buffer_capacity = new_capacity;
        }

        // Upload all states
        let data = bytemuck::cast_slice(&self.states);
        queue.write_buffer(&self.states_buffer, 0, data);

        self.dirty = false;
        tracing::debug!("Uploaded {} shading states to GPU", self.states.len());
    }

    /// Get GPU buffer
    pub fn buffer(&self) -> &Buffer {
        &self.states_buffer
    }

    /// Get state by handle (for pipeline extraction)
    pub fn get(&self, handle: UnifiedShadingStateHandle) -> Option<&PackedUnifiedShadingState> {
        self.states.get(handle.0 as usize)
    }

    /// Clear cache (called per frame)
    pub fn clear(&mut self) {
        self.cache.clear();
        self.states.clear();
        self.dirty = true;
    }

    /// Get stats for debugging
    pub fn stats(&self) -> (usize, usize) {
        (self.states.len(), self.buffer_capacity)
    }
}
```

#### 2.2: Add to ZGraphics

**File:** `emberware-z/src/graphics/mod.rs`

```rust
pub struct ZGraphics {
    // Existing fields...
    shading_state_cache: ShadingStateCache,
}

impl ZGraphics {
    pub fn new(...) -> Result<Self> {
        // Existing initialization...

        let shading_state_cache = ShadingStateCache::new(&device);

        Ok(Self {
            // Existing fields...
            shading_state_cache,
        })
    }
}
```

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

    // NEW: Single handle to interned shading state
    pub shading_state_handle: UnifiedShadingStateHandle,

    // Keep these for pipeline selection (not in shading state)
    pub depth_test: bool,
    pub cull_mode: CullMode,

    // REMOVED (now in PackedUnifiedShadingState):
    // pub color: u32,
    // pub blend_mode: BlendMode,
    // pub matcap_blend_modes: [MatcapBlendMode; 4],
}
```

**Note:** `depth_test` and `cull_mode` affect pipeline selection, so they remain separate.

#### 3.2: Update VirtualRenderPass Methods

```rust
pub fn record_triangles(
    &mut self,
    format: u8,
    vertex_data: &[f32],
    mvp_index: MvpIndex,
    texture_slots: [TextureHandle; 4],
    shading_state_handle: UnifiedShadingStateHandle,  // NEW
    depth_test: bool,    // Keep for pipeline key
    cull_mode: CullMode, // Keep for pipeline key
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
        shading_state_handle,
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

#### 4.1: Add Current Shading State to ZFFIState

**File:** `emberware-z/src/state.rs`

```rust
pub struct ZFFIState {
    // Existing fields...

    // NEW: Current shading state (built incrementally by FFI setters)
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
    pub color: u32,
    pub matcap_blend_modes: [MatcapBlendMode; 4],

    // Sky and lights (already exist, no changes needed)
    pub sky: Sky,
    pub lights: [Light; 4],
}

impl Default for ZFFIState {
    fn default() -> Self {
        Self {
            // Existing initialization...
            metallic: 0.0,
            roughness: 1.0,
            emissive: 0.0,
            color: 0xFFFFFFFF,
            matcap_blend_modes: [MatcapBlendMode::Multiply; 4],
            // sky, lights already initialized
        }
    }
}

impl ZFFIState {
    /// Pack current shading state for interning
    pub fn pack_current_shading_state(&self) -> PackedUnifiedShadingState {
        PackedUnifiedShadingState::from_render_state(
            self.color,
            self.metallic,
            self.roughness,
            self.emissive,
            &self.matcap_blend_modes,
            &self.sky,
            &self.lights,
        )
    }
}
```

#### 4.2: Update FFI Setters

**File:** `emberware-z/src/ffi/mod.rs`

```rust
fn set_color(mut caller: Caller, color: u32) {
    let state = &mut caller.data_mut().console;
    state.color = color;
}

fn material_metallic(mut caller: Caller, value: f32) {
    let state = &mut caller.data_mut().console;
    state.metallic = value.clamp(0.0, 1.0);
}

fn material_roughness(mut caller: Caller, value: f32) {
    let state = &mut caller.data_mut().console;
    state.roughness = value.clamp(0.0, 1.0);
}

fn material_emissive(mut caller: Caller, value: f32) {
    let state = &mut caller.data_mut().console;
    state.emissive = value.clamp(0.0, 1.0);
}

// Sky and light setters update state.sky, state.lights (no changes needed)
```

#### 4.3: Update Draw Commands to Defer Shading State Packing

**Challenge:** FFI functions don't have access to `ZGraphics::shading_state_cache` for interning.

**Solution:** Store unquantized state in VRPCommand temporarily, intern during `process_draw_commands()`.

**File:** `emberware-z/src/graphics/command_buffer.rs`

```rust
/// Temporary storage for unquantized shading state during FFI recording
pub struct UnquantizedShadingState {
    pub color: u32,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
    pub matcap_blend_modes: [MatcapBlendMode; 4],
    pub sky: Sky,
    pub lights: [Light; 4],
}

pub struct VRPCommand {
    // ... existing fields

    // Temporary: Store unquantized state during FFI recording
    pub temp_shading_state: Option<UnquantizedShadingState>,

    // Final: Interned handle (set during process_draw_commands)
    pub shading_state_handle: UnifiedShadingStateHandle,
}
```

**File:** `emberware-z/src/ffi/mod.rs`

```rust
fn draw_triangles(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    format: u32,
    ptr: u32,
    vertex_count: u32,
) -> Result<(), Trap> {
    let state = &mut caller.data_mut().console;

    // ... existing vertex data copy, matrix index packing

    // Pack unquantized shading state
    let temp_shading_state = UnquantizedShadingState {
        color: state.color,
        metallic: state.metallic,
        roughness: state.roughness,
        emissive: state.emissive,
        matcap_blend_modes: state.matcap_blend_modes,
        sky: state.sky.clone(),
        lights: state.lights.clone(),
    };

    state.render_pass.record_triangles_with_temp_state(
        format as u8,
        &vertex_data,
        mvp_index,
        state.texture_slots,
        temp_shading_state,
        state.depth_test,
        state.cull_mode,
    );

    Ok(())
}
```

**File:** `emberware-z/src/graphics/mod.rs` (in `process_draw_commands`)

```rust
pub fn process_draw_commands(&mut self, z_state: &mut ZFFIState) {
    // Swap command buffer from FFI state
    std::mem::swap(&mut self.command_buffer, &mut z_state.render_pass);

    // Intern all temporary shading states
    for cmd in self.command_buffer.commands_mut() {
        if let Some(temp_state) = cmd.temp_shading_state.take() {
            let packed = PackedUnifiedShadingState::from_render_state(
                temp_state.color,
                temp_state.metallic,
                temp_state.roughness,
                temp_state.emissive,
                &temp_state.matcap_blend_modes,
                &temp_state.sky,
                &temp_state.lights,
            );
            cmd.shading_state_handle = self.shading_state_cache.intern(packed);
        }
    }

    // Process deferred commands (billboards, sprites, text)
    // ... (also intern shading states for these)
}
```

---

## Phase 5: Update Render Pass Execution

**Estimated Time:** 3-4 hours

### Files to Modify
- `emberware-z/src/graphics/mod.rs`

### Changes

#### 5.1: Upload Shading States Before Rendering

```rust
pub fn render_frame(&mut self, ...) -> Result<()> {
    // 1. Upload matrices (from matrix packing refactor)
    // ...

    // 2. Upload shading state buffer
    self.shading_state_cache.upload(&self.device, &self.queue);

    // 3. Upload immediate vertex/index data
    // ...

    // 4. Sort commands
    self.sort_commands();

    // 5. Execute render pass
    // ...
}
```

#### 5.2: Update Command Sorting

```rust
fn sort_commands(&mut self) {
    self.command_buffer.commands_mut().sort_unstable_by_key(|cmd| {
        let shading_state = cmd.shading_state_handle;
        let packed_state = self.shading_state_cache.get(shading_state);

        // Extract blend mode for pipeline key
        let blend_mode = if let Some(state) = packed_state {
            (state.blend_modes & 0xFF) as u8
        } else {
            0
        };

        (
            self.render_mode,           // Mode (0-3)
            cmd.format,                 // Vertex format (0-15)
            blend_mode,                 // Blend mode (extracted from shading state)
            cmd.texture_slots[0].0,     // Primary texture
            shading_state.0,            // Material (NEW: sort by shading state handle)
        )
    });
}
```

#### 5.3: Update Push Constants to Include Shading State Index

**File:** `emberware-z/src/graphics/mod.rs` (in render pass loop)

```rust
for cmd in self.command_buffer.commands() {
    // ... pipeline and bind group setup

    // Set push constants with matrix + shading indices
    let (model_idx, view_idx, proj_idx) = cmd.mvp_index.unpack();
    let push_constants = [
        model_idx,
        view_idx,
        proj_idx,
        cmd.shading_state_handle.0,  // NEW: shading state index
    ];
    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,  // NOTE: Fragment needs it too!
        0,
        bytemuck::cast_slice(&push_constants),
    );

    // ... rest of render pass execution
}
```

---

## Phase 6: Update Shaders to Read Shading States

**Estimated Time:** 6-8 hours (all 4 shader templates)

### Files to Modify
- `emberware-z/shaders/mode0_unlit.wgsl`
- `emberware-z/shaders/mode1_matcap.wgsl`
- `emberware-z/shaders/mode2_pbr.wgsl`
- `emberware-z/shaders/mode3_hybrid.wgsl`

### Changes (Apply to All 4 Templates)

#### 6.1: Add Shading State Buffer Binding

```wgsl
// Push constants (updated from matrix packing)
struct PushConstants {
    model_index: u32,
    view_index: u32,
    proj_index: u32,
    shading_state_index: u32,  // NEW: for unified shading state
}

var<push_constant> pc: PushConstants;

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

// Shading state buffer (group 0, binding 6 - after bones at binding 5)
@group(0) @binding(6) var<storage, read> shading_states: array<UnifiedShadingState>;
```

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

#### 6.3: Update Fragment Shader

```wgsl
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fetch shading state for this draw (via push constants)
    let state = shading_states[pc.shading_state_index];

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

```rust
let bind_group_layout_0 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
    label: Some("Frame Uniforms"),
    entries: &[
        // Bindings 0-2: Matrix storage buffers (from matrix packing)
        // Binding 3: Sky uniforms
        // Binding 4: Material uniforms
        // Binding 5: Bone buffer

        // Binding 6: Shading states (NEW)
        BindGroupLayoutEntry {
            binding: 6,
            visibility: ShaderStages::FRAGMENT,  // Fragment shader reads this
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
    ],
});
```

#### 7.2: Create Bind Group with Shading State Buffer

**File:** `emberware-z/src/graphics/mod.rs`

```rust
fn create_frame_bind_group(&self) -> wgpu::BindGroup {
    self.device.create_bind_group(&BindGroupDescriptor {
        label: Some("Frame Uniforms"),
        layout: &self.frame_bind_group_layout,
        entries: &[
            // Bindings 0-5: Existing (matrices, sky, material, lights, bones)
            // ...

            // Binding 6: Shading states
            BindGroupEntry {
                binding: 6,
                resource: self.shading_state_cache.buffer().as_entire_binding(),
            },
        ],
    })
}
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
    shading_cache: &ShadingStateCache,
    render_mode: u8,
) -> PipelineKey {
    // Get actual shading state from cache
    let shading_state = shading_cache
        .get(cmd.shading_state_handle)
        .expect("Shading state handle not found in cache");

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
- [ ] Push constants: Verify 16-byte total (4 × u32)

### Performance Metrics

```rust
// Log cache stats
let (state_count, capacity) = self.shading_state_cache.stats();
tracing::debug!(
    "Shading state cache: {} states, {} capacity ({} KB)",
    state_count,
    capacity,
    (state_count * std::mem::size_of::<PackedUnifiedShadingState>()) / 1024
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
- Uses the 4th push constant slot reserved by matrix packing
- Depends on VRPCommand having `mvp_index` instead of `transform`
- Leverages the same push constant infrastructure

**Push constants structure (shared):**
```wgsl
struct PushConstants {
    model_index: u32,           // Matrix packing
    view_index: u32,            // Matrix packing
    proj_index: u32,            // Matrix packing
    shading_state_index: u32,   // Unified shading state (THIS refactor)
}
```

**Key difference from original plan:**
- ✅ Uses push constants (simpler than vertex attributes/instance buffers)
- ✅ No per-draw instance buffer management
- ✅ All per-draw indices in one place (push constants)

---

**Last Updated:** December 2024
**Status:** Ready for implementation (after matrix packing)
