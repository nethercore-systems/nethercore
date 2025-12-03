# Implementation Plan: Unified Shading State

**Status:** Phase 1-4 Complete (Core infrastructure implemented)
**Estimated Effort:** 5-7 days
**Priority:** Medium (implement second)
**Depends On:** Matrix index packing (recommended)
**Related:** [proposed-render-architecture.md](./proposed-render-architecture.md), [rendering-architecture.md](./rendering-architecture.md)

---

## Implementation Progress

### ✅ Completed Phases

**Phase 1: Define Packed State Structures** ✅
- Created `unified_shading_state.rs` module with all packed structures
- Implemented `PackedUnifiedShadingState`, `PackedSky`, `PackedLight`
- Added quantization helpers for f32→u8, Vec3→snorm16, color packing
- All structures are POD, hashable, and GPU-ready

**Phase 2: Implement Shading State Cache** ✅
- Implemented `ShadingStateCache` with HashMap-based interning
- Added GPU buffer management with automatic growth
- Implemented upload logic with dirty tracking
- Integrated cache into `ZGraphics` struct

**Phase 3: Update VRPCommand Structure** ✅
- Added `shading_state_handle: Option<UnifiedShadingStateHandle>` to `VRPCommand`
- Updated all command recording methods to initialize handle as `None`
- Kept legacy fields for backward compatibility during transition

**Phase 4: Update FFI Layer to Track State** ✅
- Added sky state fields to `ZFFIState` (horizon, zenith, sun direction, color, sharpness)
- Updated `set_sky()` FFI function to store state in `ZFFIState`
- Implemented `pack_current_shading_state()` helper method
- Added shading state interning in command processing (after swap)
- All commands now receive interned shading state handles

**Phase 5: Upload Shading States Before Rendering** ✅
- Added `shading_state_cache.upload()` call in `render_frame()`
- Upload happens after matrix uploads, before rendering
- Dirty tracking ensures efficient uploads

**Phase 6: Update Shaders to Read Shading States** ✅
- Added shading state buffer binding (@binding(7) for modes 0-1, @binding(9) for modes 2-3)
- Updated all 4 shader templates (mode0-3) with `UnifiedShadingState` struct
- Added unpacking helper functions (unpack_rgba8, unpack_u8_to_f32, unpack_snorm16)
- Updated vertex shaders to pass `shading_state_index` via @location(20)
- Updated fragment shaders to fetch and use shading state
- Mode 0 (Unlit): Uses base color and sky from shading state
- Mode 1 (Matcap): Uses base color and blend modes from shading state
- Mode 2 (PBR): Uses material properties and lights from shading state
- Mode 3 (Hybrid): Uses material properties and first light from shading state
- **GPU Integration**: Added shading state buffer to bind group layouts and bind groups
  - Updated `create_frame_bind_group_layout()` in pipeline.rs
  - Updated bind group creation in render loop for all modes
  - Shading state buffer now properly bound and accessible to shaders

**Phase 7: Update Pipeline Extraction and Command Sorting** ✅
- Created shading_indices_buffer to map instance_index → shading_state_index after sorting
- Commands are sorted by pipeline key (render_mode, format, blend_mode, depth_test, textures)
- Shading indices buffer populated after sorting and uploaded to GPU
- Shaders read: `shading_states[shading_indices[instance_index]]` for correct indirection
- All bind group layouts updated to include shading_indices buffer
- Verified data flow: commands → sort → collect indices → upload → shader reads correctly

### 🚧 Remaining Work

**Phase 8: Testing and Validation** (In Progress)
- Visual testing across all 4 render modes
- Performance benchmarking
- Cache efficiency metrics
- Verify all billboard/sprite rendering works correctly

---

## Overview

Quantize all per-draw material state into a hashable POD structure (`PackedUnifiedShadingState`), implement interning to deduplicate identical materials, and enable better batching/sorting.

**Benefits:**
- Material state becomes hashable and comparable
- Same material used across draws = one GPU upload
- Better command sorting by material
- Reduced VRPCommand size (remove separate state fields)
- All per-draw state packaged together (self-contained)

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

    /// Clear cache (optional, for per-frame reset)
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

    // REMOVED (now in PackedUnifiedShadingState):
    // pub color: u32,
    // pub depth_test: bool,
    // pub cull_mode: CullMode,
    // pub blend_mode: BlendMode,
    // pub matcap_blend_modes: [MatcapBlendMode; 4],
}
```

**Note:** `depth_test` and `cull_mode` may still be needed for pipeline selection. Options:
1. Keep them separate (for pipeline cache key)
2. Extract from shading state during render
3. Add to pipeline key extraction logic

#### 3.2: Update VirtualRenderPass Methods

```rust
pub fn record_triangles(
    &mut self,
    format: u8,
    vertex_data: &[f32],
    mvp_index: MvpIndex,
    texture_slots: [TextureHandle; 4],
    shading_state_handle: UnifiedShadingStateHandle,  // NEW
    depth_test: bool,    // Keep for pipeline key (or extract later)
    cull_mode: CullMode, // Keep for pipeline key (or extract later)
) {
    // ... implementation
}
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

    // Sky and lights (existing, already tracked)
    pub sky: Sky,
    pub lights: [Light; 4],

    // NEW: Shading state cache reference (or access via ZGraphics)
    // (Will access via ZGraphics during render, not stored in ZFFIState)
}

impl ZFFIState {
    pub fn new(...) -> Self {
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

#### 4.3: Update Draw Commands to Intern State

**Challenge:** FFI functions don't have direct access to `ZGraphics::shading_state_cache`.

**Solution:** Pass shading state to command recording, intern during `process_draw_commands()`.

**Updated approach:**

1. **Record commands with unquantized state** (temporary structure)
2. **Intern during `process_draw_commands()`** when we have access to ZGraphics

**File:** `emberware-z/src/graphics/command_buffer.rs`

```rust
// Temporary: Store unquantized state in VRPCommand during recording
pub struct VRPCommand {
    // ... existing fields

    // Temporary: Store unquantized state during FFI recording
    pub temp_shading_state: Option<UnquantizedShadingState>,

    // Final: Interned handle (set during process_draw_commands)
    pub shading_state_handle: Option<UnifiedShadingStateHandle>,
}

pub struct UnquantizedShadingState {
    pub color: u32,
    pub metallic: f32,
    pub roughness: f32,
    pub emissive: f32,
    pub matcap_blend_modes: [MatcapBlendMode; 4],
    pub sky: Sky,
    pub lights: [Light; 4],
}
```

**File:** `emberware-z/src/ffi/mod.rs`

```rust
fn draw_triangles(...) -> Result<(), Trap> {
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
            cmd.shading_state_handle = Some(self.shading_state_cache.intern(packed));
        }
    }

    // Process deferred commands (billboards, sprites)
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
        let shading_state = cmd.shading_state_handle.unwrap_or(UnifiedShadingStateHandle::INVALID);
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

#### 5.3: Track Bound Shading State

```rust
let mut bound_shading_state: Option<UnifiedShadingStateHandle> = None;

for cmd in self.command_buffer.commands() {
    // Extract shading state
    let shading_state_handle = cmd.shading_state_handle.unwrap();

    // Bind shading state buffer (only if changed)
    if bound_shading_state != Some(shading_state_handle) {
        // Note: Shading state is in storage buffer at binding X
        // Actual binding happens via bind group 0
        // (No per-draw binding needed if using storage buffer indexing)

        bound_shading_state = Some(shading_state_handle);
    }

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
// Packed structures (must match Rust layout)
struct PackedSky {
    horizon_color: vec4<f32>,           // RGBA8 unpacked by GPU
    zenith_color: vec4<f32>,
    sun_direction: vec4<f32>,           // snorm16 unpacked
    sun_color_and_sharpness: vec4<f32>,
}

struct PackedLight {
    direction: vec4<f32>,               // snorm16 unpacked
    color_and_intensity: vec4<f32>,
}

struct UnifiedShadingState {
    metallic: f32,          // u8 unpacked to f32 (0-1)
    roughness: f32,
    emissive: f32,
    _pad0: f32,

    color: vec4<f32>,       // RGBA8 unpacked
    blend_modes: vec4<u32>, // 4× u8 as u32 components

    sky: PackedSky,
    lights: array<PackedLight, 4>,
}

// Shading state buffer (group 0, binding X)
@group(0) @binding(X) var<storage, read> shading_states: array<UnifiedShadingState>;
```

#### 6.2: Update Vertex Shader

```wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    // ... other vertex attributes
    @location(10) mvp_index: u32,              // From matrix packing
    @location(11) shading_state_index: u32,    // NEW
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) @interpolate(flat) shading_state_index: u32,  // Pass to fragment
    // ... other outputs
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    // Unpack MVP (from matrix packing refactor)
    let indices = unpack_mvp(in.mvp_index);
    let model = model_matrices[indices.x];
    let view = view_matrices[indices.y];
    let proj = proj_matrices[indices.z];

    // Transform vertex
    let world_pos = model * vec4(in.position, 1.0);
    let clip_pos = proj * view * world_pos;

    var out: VertexOutput;
    out.position = clip_pos;
    out.shading_state_index = in.shading_state_index;  // Pass through
    // ... other outputs

    return out;
}
```

#### 6.3: Update Fragment Shader

```wgsl
@fragment
fn fs(in: VertexOutput) -> @location(0) vec4<f32> {
    // Fetch shading state for this draw
    let state = shading_states[in.shading_state_index];

    // Use state values
    let base_color = state.color;
    let metallic = state.metallic;
    let roughness = state.roughness;
    let emissive = state.emissive;

    // Sample texture (if UV present)
    //FS_SAMPLE_ALBEDO

    // Apply color tint
    var albedo = base_color;
    //FS_APPLY_TEXTURE

    // Lighting (use state.lights, state.sky)
    let light0 = state.lights[0];
    let light_dir = normalize(light0.direction.xyz);
    let light_color = light0.color_and_intensity.rgb;
    let light_intensity = light0.color_and_intensity.a;

    // ... rest of fragment shader (lighting, PBR, etc.)

    return final_color;
}
```

#### 6.4: Add Unpacking Helpers

```wgsl
// Convert u8 (0-255) to f32 (0-1)
fn unpack_u8_to_f32(value: u32) -> f32 {
    return f32(value) / 255.0;
}

// Convert snorm16 to f32 (-1 to 1)
fn unpack_snorm16(packed: vec4<i32>) -> vec3<f32> {
    return vec3<f32>(
        f32(packed.x) / 32767.0,
        f32(packed.y) / 32767.0,
        f32(packed.z) / 32767.0,
    );
}

// Unpack RGBA8 from u32
fn unpack_rgba8(packed: u32) -> vec4<f32> {
    let r = f32((packed >> 24u) & 0xFFu) / 255.0;
    let g = f32((packed >> 16u) & 0xFFu) / 255.0;
    let b = f32((packed >> 8u) & 0xFFu) / 255.0;
    let a = f32(packed & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}
```

---

## Phase 7: Update Pipeline Extraction

**Estimated Time:** 2-3 hours

### Files to Modify
- `emberware-z/src/graphics/pipeline.rs`

### Changes

#### 7.1: Extract Pipeline State from Shading State

```rust
fn extract_pipeline_key(
    cmd: &VRPCommand,
    shading_cache: &ShadingStateCache,
    render_mode: u8,
) -> PipelineKey {
    // Get actual shading state from cache
    let shading_state = shading_cache
        .get(cmd.shading_state_handle.unwrap())
        .expect("Shading state handle not found in cache");

    // Extract blend mode from packed state
    let blend_mode = (shading_state.blend_modes & 0xFF) as u8;

    PipelineKey {
        render_mode,
        vertex_format: cmd.format,
        blend_mode,
        depth_test: cmd.depth_test,  // Still separate, or extract from state
        cull_mode: cmd.cull_mode,    // Still separate, or extract from state
    }
}
```

---

## Phase 8: Testing and Validation

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

6. **Cache Persistence**
   - Draw same materials across multiple frames
   - Verify cache doesn't clear unnecessarily (or does, if designed that way)
   - Performance: No redundant uploads

### Validation Checklist

- [ ] Visual: All test cases match old renderer (within quantization tolerance)
- [ ] Performance: Measure reduction in uniform uploads
- [ ] Memory: Shading state cache size is reasonable (<10KB typical game)
- [ ] Cache efficiency: High hit rate for repeated materials
- [ ] Quantization: No visible artifacts from u8/snorm16 precision

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
- VRPCommand size: ~120 bytes → ~60 bytes (50% reduction)
- Material uploads: Reduced by deduplication ratio (depends on game)
- Command sorting: Better batching by material handle

---

## Rollout Strategy

### 1. Feature Flag

```rust
pub struct ZGraphics {
    use_unified_shading_state: bool,  // Toggle during testing
}
```

### 2. Incremental Deployment

1. **Day 1-2:** Phases 1-3 (structures, cache, VRPCommand)
2. **Day 3-4:** Phase 4 (FFI layer, state quantization)
3. **Day 5:** Phase 5 (render pass execution)
4. **Day 6:** Phase 6 (shaders)
5. **Day 7:** Phases 7-8 (pipeline extraction, testing)

### 3. Fallback Plan

If quantization causes visual issues:
- Use `u16` instead of `u8` for critical params (metallic, roughness)
- Use `f16` instead of `snorm16` for directions (if supported)
- Keep high-precision fallback path for quality-sensitive draws

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Quantization artifacts | Medium | Medium | Visual testing, adjust precision (u16/f16) |
| Cache bloat | Low | Medium | Monitor size, implement LRU eviction |
| Complex debugging | High | Medium | Add debug views, unpacking helpers |
| Shader complexity | Medium | Low | Thorough review, extensive testing |
| Performance regression | Low | High | Benchmark before/after, profile |

---

## Success Criteria

- ✅ Visual quality matches old renderer (within quantization tolerance)
- ✅ VRPCommand size reduced by ~50%
- ✅ Material uploads reduced (measure actual reduction in real games)
- ✅ Better command batching (fewer pipeline/texture changes)
- ✅ No crashes or glitches
- ✅ Shading state cache is efficient (high hit rate)

---

## Follow-Up Work

1. **WebGL fallback** - If storage buffers unsupported, use per-draw uniforms
2. **Cache eviction** - LRU policy if cache grows too large
3. **Shader optimization** - Profile GPU performance after refactor
4. **Quantization tuning** - Adjust precision based on visual testing

---

**Last Updated:** December 2024
**Status:** Ready for implementation (after matrix packing)
