# GPU-Instanced Quad Rendering — Clean Architecture Plan

**Status**: Ready for Implementation
**Last Updated**: 2025-12-05
**Approach**: Storage buffer quad instances, separate 2D pipeline, preserve MVP system

---

## Executive Summary

Replace the fragile `DeferredCommand` system with GPU-instanced quad rendering using **storage buffer lookups** (consistent with existing MVP pattern). This approach:

- **Eliminates `DeferredCommand` entirely** (no more deferred processing)
- **Adds `BufferSource::Quad` variant** (explicit rendering path)
- **Preserves MVP instancing system** (no conflicts)
- **Uses storage buffer for QuadInstance data** (clean, follows existing pattern)
- **Single static unit quad mesh** with `@builtin(instance_index)` lookup
- **GPU-driven billboard/sprite expansion** in dedicated shaders
- **Separate 2D pipeline** (matches Unity/Unreal/Godot architecture)

**Key Architectural Decision**: Use storage buffer lookup (like MVP matrices) instead of per-instance vertex attributes. Cleaner, more consistent, easier to extend.

---

## Problem Analysis

### The DeferredCommand Issues

**Current flow**:
```
1. FFI: draw_billboard() → DeferredCommand { transform, color, ... }
2. End of frame: process_draw_commands() processes deferred queue
3. CPU generates vertices (needs camera, hard-codes ShadingStateIndex(0))
4. PANIC: State pool empty when accessing state[0]
```

**Root causes**:
1. Shading state captured during `render()`, but accessed later during processing
2. Hard-coded `ShadingStateIndex(0)` when state pool might be empty
3. Fragile state management across frame boundary
4. CPU vertex generation is complex and expensive

### Why Not "Fix" DeferredCommand?

We considered just fixing the bugs (store view_index, capture shading_state_index), but:
- Still requires separate deferred processing path
- Duplicate code for CPU billboard math
- Can't batch/sort with other commands
- Mixing two rendering paradigms unnecessarily

---

## Solution: BufferSource::Quad with Storage Buffer

### Architecture

Extend the existing `BufferSource` enum and use storage buffer for instance data:

```rust
pub enum BufferSource {
    Immediate,  // Per-frame dynamic geometry (draw_triangles)
    Retained,   // Static meshes (load_mesh)
    Quad,       // GPU-instanced quads (billboards, sprites) ← NEW
}
```

**Quad instances stored in GPU storage buffer** (like MVP matrices):
```wgsl
@group(0) @binding(4) var<storage, read> quad_instances: array<QuadInstance>;
```

**Flow**:
```
1. FFI: draw_billboard() → QuadInstance { position, size, mode, view_idx, ... }
2. Shading state captured IMMEDIATELY (no deferred access)
3. End of frame: upload instance buffer, create VRPCommand with BufferSource::Quad
4. Render: Bind instance storage buffer, GPU does instance_index lookup + billboard math
```

**Benefits**:
- ✅ No separate deferred processing
- ✅ Quads go through same VRPCommand pipeline (can be sorted)
- ✅ MVP system stays clean (Immediate/Retained unchanged)
- ✅ Consistent with existing storage buffer pattern
- ✅ Only 1 additional binding (not 8 vertex attribute locations)
- ✅ Easier to extend QuadInstance without pipeline changes
- ✅ Separate 2D pipeline (matches professional engines)

---

## Phase 1: Infrastructure (Already Complete ✅)

### 1.1: QuadInstance Struct ✅

**File**: `emberware-z/src/graphics/quad_instance.rs`

```rust
#[repr(u32)]
pub enum QuadMode {
    BillboardSpherical = 0,
    BillboardCylindricalY = 1,
    BillboardCylindricalX = 2,
    BillboardCylindricalZ = 3,
    ScreenSpace = 4,
    WorldSpace = 5,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadInstance {
    pub position: [f32; 3],         // 12 bytes
    pub size: [f32; 2],             // 8 bytes
    pub rotation: f32,              // 4 bytes
    pub mode: u32,                  // 4 bytes
    pub uv: [f32; 4],               // 16 bytes
    pub color: u32,                 // 4 bytes
    pub shading_state_index: u32,   // 4 bytes
    pub view_index: u32,            // 4 bytes
    // Total: 56 bytes (16-byte aligned)
}
```

### 1.2: BufferSource::Quad ✅

**File**: `emberware-z/src/graphics/command_buffer.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSource {
    Immediate,
    Retained,
    Quad,  // ✅ DONE
}
```

### 1.3: Unit Quad Mesh ✅

**File**: `emberware-z/src/graphics/mod.rs`

Unit quad created in `ZGraphics::new()`:
- Format: `FORMAT_UV | FORMAT_COLOR` (POS_UV_COLOR)
- 4 vertices, 6 indices
- Stored in retained buffer
- Fields: `unit_quad_format`, `unit_quad_base_vertex`, `unit_quad_first_index`, `unit_quad_index_count`

### 1.4: Instance Buffer ✅

**File**: `emberware-z/src/graphics/buffer.rs`

```rust
quad_instance_buffer: GrowableBuffer  // BufferUsages::STORAGE | COPY_DST
```

**NOTE**: Need to change buffer usage from `VERTEX` to `STORAGE` for storage buffer binding.

### 1.5: FFI State Tracking ✅

**File**: `emberware-z/src/state.rs`

```rust
pub quad_instances: Vec<QuadInstance>
```

### 1.6: Render Loop Updates ✅

**File**: `emberware-z/src/graphics/mod.rs`

Render loop matches on `BufferSource::Quad` to use separate path.

---

## Phase 2: Storage Buffer & Shader Updates

### 2.1: Change Instance Buffer to Storage Buffer

**File**: `emberware-z/src/graphics/buffer.rs`

**Change buffer usage**:
```rust
quad_instance_buffer: GrowableBuffer::new(
    device,
    "Quad Instance Storage Buffer",
    1024 * std::mem::size_of::<QuadInstance>() as u64,
    wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,  // ← Changed from VERTEX
),
```

**Update upload method** (if needed - should just work):
```rust
pub fn upload_quad_instances(
    &mut self,
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    instances: &[QuadInstance],
) -> Result<()> {
    let byte_data = bytemuck::cast_slice(instances);
    self.quad_instance_buffer.ensure_capacity(device, byte_data.len() as u64);
    self.quad_instance_buffer.reset();
    self.quad_instance_buffer.write(queue, byte_data);
    Ok(())
}
```

### 2.2: Add Storage Buffer Binding to Bind Group

**File**: `emberware-z/src/graphics/mod.rs` (or wherever bind groups are created)

**Current bind group layout** (estimated):
```rust
@group(0) @binding(0) var<uniform> ...          // Camera/frame data
@group(0) @binding(1) var<storage> mvp_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage> shading_states: array<PackedUnifiedShadingState>;
@group(0) @binding(4) var<storage> quad_instances: array<QuadInstance>;  // ← ADD THIS
```

**Add to bind group layout creation**:
```rust
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
```

**Add to bind group creation** (in render loop):
```rust
wgpu::BindGroupEntry {
    binding: 4,
    resource: self.buffer_manager.quad_instance_buffer()
        .expect("Quad instance buffer should exist")
        .as_entire_binding(),
},
```

### 2.3: Create Dedicated Quad Shaders

**New files**:
- `emberware-z/shaders/quad_unlit.wgsl`
- `emberware-z/shaders/quad_matcap.wgsl` (if needed)

**Shader structure**:

```wgsl
// ============================================================================
// Bindings
// ============================================================================

@group(0) @binding(0) var<uniform> frame_data: FrameData;
@group(0) @binding(1) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(3) var<storage, read> proj_matrices: array<mat4x4<f32>>;
@group(0) @binding(4) var<storage, read> shading_states: array<PackedUnifiedShadingState>;
@group(0) @binding(5) var<storage, read> quad_instances: array<QuadInstance>;  // ← NEW

@group(1) @binding(0) var texture0: texture_2d<f32>;
@group(1) @binding(1) var sampler0: sampler;
// ... other texture slots ...

// ============================================================================
// Structures
// ============================================================================

struct QuadInstance {
    position: vec3<f32>,      // 12 bytes
    size: vec2<f32>,          // 8 bytes
    rotation: f32,            // 4 bytes
    mode: u32,                // 4 bytes
    uv: vec4<f32>,            // 16 bytes
    color: u32,               // 4 bytes
    shading_state_index: u32, // 4 bytes
    view_index: u32,          // 4 bytes
}

struct VertexIn {
    @location(0) position: vec3<f32>,  // Unit quad vertex
    @location(1) uv: vec2<f32>,
    @location(2) color: vec3<f32>,
}

struct VertexOut {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(2) color: vec3<f32>,
    @location(3) @interpolate(flat) shading_state_index: u32,
}

// ============================================================================
// Billboard Math
// ============================================================================

fn apply_billboard(
    local_pos: vec2<f32>,
    size: vec2<f32>,
    mode: u32,
    view_matrix: mat4x4<f32>,
) -> vec3<f32> {
    let cam_right = normalize(vec3<f32>(view_matrix[0].x, view_matrix[0].y, view_matrix[0].z));
    let cam_up = normalize(vec3<f32>(view_matrix[1].x, view_matrix[1].y, view_matrix[1].z));
    let cam_fwd = -normalize(vec3<f32>(view_matrix[2].x, view_matrix[2].y, view_matrix[2].z));

    let scaled_x = local_pos.x * size.x;
    let scaled_y = local_pos.y * size.y;

    if (mode == 0u) { // Spherical
        return cam_right * scaled_x + cam_up * scaled_y;
    } else if (mode == 1u) { // Cylindrical Y
        let fwd_xz = normalize(vec3<f32>(cam_fwd.x, 0.0, cam_fwd.z));
        let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), fwd_xz));
        return right * scaled_x + vec3<f32>(0.0, 1.0, 0.0) * scaled_y;
    } else if (mode == 2u) { // Cylindrical X
        let fwd_yz = normalize(vec3<f32>(0.0, cam_fwd.y, cam_fwd.z));
        let up = normalize(cross(fwd_yz, vec3<f32>(1.0, 0.0, 0.0)));
        return vec3<f32>(1.0, 0.0, 0.0) * scaled_x + up * scaled_y;
    } else if (mode == 3u) { // Cylindrical Z
        let fwd_xy = normalize(vec3<f32>(cam_fwd.x, cam_fwd.y, 0.0));
        let right = normalize(cross(vec3<f32>(0.0, 0.0, 1.0), fwd_xy));
        return right * scaled_x + vec3<f32>(0.0, 0.0, 1.0) * scaled_y;
    }

    // Fallback: no billboard
    return vec3<f32>(scaled_x, scaled_y, 0.0);
}

fn apply_screen_space(
    local_pos: vec2<f32>,
    size: vec2<f32>,
    rotation: f32,
    screen_pos: vec2<f32>,
) -> vec2<f32> {
    // Apply rotation
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);
    let rotated = vec2<f32>(
        local_pos.x * cos_r - local_pos.y * sin_r,
        local_pos.x * sin_r + local_pos.y * cos_r,
    );

    // Scale and translate
    return screen_pos + rotated * size;
}

// ============================================================================
// Vertex Shader
// ============================================================================

@vertex
fn vs_main(
    in: VertexIn,
    @builtin(instance_index) instance_idx: u32,
) -> VertexOut {
    var out: VertexOut;

    // Look up instance data
    let instance = quad_instances[instance_idx];

    // Get view matrix for billboard math
    let view_matrix = view_matrices[instance.view_index];
    let proj_matrix = proj_matrices[0];  // Typically use proj index 0

    // Transform based on quad mode
    if (instance.mode < 4u) {
        // Billboard modes (0-3)
        let billboard_offset = apply_billboard(
            in.position.xy,
            instance.size,
            instance.mode,
            view_matrix,
        );
        let world_pos = instance.position + billboard_offset;
        out.world_position = world_pos;
        out.clip_position = proj_matrix * view_matrix * vec4<f32>(world_pos, 1.0);

    } else if (instance.mode == 4u) {
        // Screen-space sprite
        let screen_pos = apply_screen_space(
            in.position.xy,
            instance.size,
            instance.rotation,
            instance.position.xy,
        );
        out.world_position = vec3<f32>(screen_pos, 0.0);
        out.clip_position = vec4<f32>(screen_pos, 0.0, 1.0);

    } else {
        // World-space quad (5+)
        let local_offset = vec3<f32>(in.position.xy * instance.size, 0.0);
        let world_pos = instance.position + local_offset;
        out.world_position = world_pos;
        out.clip_position = proj_matrix * view_matrix * vec4<f32>(world_pos, 1.0);
    }

    // Interpolate UV between instance.uv (texture atlas rect)
    out.uv = mix(instance.uv.xy, instance.uv.zw, in.uv);

    // Unpack color and blend with instance color
    let instance_color = unpack4x8unorm(instance.color).rgb;
    out.color = in.color * instance_color;

    // Pass shading state index
    out.shading_state_index = instance.shading_state_index;

    return out;
}

// ============================================================================
// Fragment Shader
// ============================================================================

@fragment
fn fs_main(in: VertexOut) -> @location(0) vec4<f32> {
    // Sample texture
    let tex_color = textureSample(texture0, sampler0, in.uv);

    // Get shading state
    let state = shading_states[in.shading_state_index];
    let material_color = unpack4x8unorm(state.color).rgb;

    // Combine
    let final_color = tex_color.rgb * in.color * material_color;

    return vec4<f32>(final_color, tex_color.a);
}

// ============================================================================
// Helper Functions
// ============================================================================

fn unpack4x8unorm(packed: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((packed >> 0u) & 0xFFu) / 255.0,
        f32((packed >> 8u) & 0xFFu) / 255.0,
        f32((packed >> 16u) & 0xFFu) / 255.0,
        f32((packed >> 24u) & 0xFFu) / 255.0,
    );
}
```

### 2.4: Add Quad Pipelines to Pipeline Cache

**File**: `emberware-z/src/graphics/pipeline_cache.rs`

Add pipeline keys for quad rendering:
```rust
pub enum PipelineKey {
    // Existing keys...
    Standard { mode: u8, format: u8, depth_test: bool, cull_mode: CullMode },

    // New quad keys
    Quad { mode: u8, depth_test: bool, cull_mode: CullMode },  // ← ADD THIS
}
```

Generate quad pipelines:
- Input: Unit quad mesh layout (POS_UV_COLOR)
- Shader: quad_unlit.wgsl (or quad_matcap.wgsl)
- Bind groups: Standard bind group 0 + texture bind group 1

### 2.5: Update Render Loop to Bind Quad Instance Buffer

**File**: `emberware-z/src/graphics/mod.rs` (in render loop)

When `BufferSource::Quad` is detected, ensure:
1. Bind group with quad_instances buffer is bound
2. Use quad-specific pipeline (PipelineKey::Quad)
3. No need to set vertex buffer slot 1 (no per-instance vertex attributes)

---

## Phase 3: Testing & Validation

### 3.1: Test Billboard Example

```bash
cargo run --release --package emberware-z -- billboard
```

Expected: Billboards render correctly using GPU instancing.

### 3.2: Remove DeferredCommand

**File**: `emberware-z/src/state.rs`

Delete entire `DeferredCommand` enum and deferred processing code.

### 3.3: Performance Validation

- 1000+ billboards should use single draw call (instanced)
- CPU time reduced (no vertex generation)
- GPU profiler shows instanced draw

---

## Architecture Summary

### Two Separate Instancing Systems

**MVP Instancing** (existing, unchanged):
- BufferSource: `Immediate` | `Retained`
- Instance data: `@builtin(instance_index)` → storage buffer (model/view/proj matrices)
- Vertex buffer: Regular mesh data (various formats)
- Pipeline: 40 existing shader permutations

**Quad Instancing** (new, separate):
- BufferSource: `Quad`
- Instance data: `@builtin(instance_index)` → storage buffer (QuadInstance array)
- Vertex buffer: Unit quad mesh (POS_UV_COLOR, always)
- Pipeline: 1-2 dedicated quad shaders

**No Conflicts**: Completely separate rendering paths, unified via `BufferSource` enum.

---

## Critical Files Modified

1. **`emberware-z/src/graphics/quad_instance.rs`** ✅ - QuadInstance struct
2. **`emberware-z/src/graphics/command_buffer.rs`** ✅ - BufferSource::Quad
3. **`emberware-z/src/graphics/buffer.rs`** - Change VERTEX to STORAGE usage
4. **`emberware-z/src/graphics/mod.rs`** - Add binding(5), update bind group
5. **`emberware-z/shaders/quad_unlit.wgsl`** (NEW) - Dedicated quad shader
6. **`emberware-z/src/graphics/pipeline_cache.rs`** - Add Quad pipeline key
7. **`emberware-z/src/state.rs`** ✅ - quad_instances vec (remove DeferredCommand later)
8. **`emberware-z/src/ffi/mod.rs`** ✅ - draw_billboard creates QuadInstance

---

## Timeline Estimate

- **Phase 2.1-2.2**: Buffer changes + bind group (1-2 hours)
- **Phase 2.3**: Write quad shader (2-3 hours)
- **Phase 2.4**: Add pipeline cache support (1 hour)
- **Phase 2.5**: Update render loop (1 hour)
- **Phase 3**: Testing + cleanup (2 hours)

**Total**: 7-9 hours remaining (Phase 1 already complete)

---

## FAQ

**Q: Why storage buffer instead of per-instance vertex attributes?**

**A**: Consistency with existing MVP pattern, cleaner pipeline layout, easier to extend, only uses 1 binding instead of 8 attribute locations, follows modern GPU best practices.

**Q: Does this break the existing MVP instancing system?**

**A**: No! MVP system uses `BufferSource::Immediate/Retained` with existing instance_index lookup. Quad system uses `BufferSource::Quad` with separate instance buffer. No conflicts.

**Q: Why separate shaders for quads?**

**A**: Quads have different vertex layout (unit quad) and different math (billboard expansion). Mixing them would pollute existing shaders. Separation is cleaner and matches professional engine architecture.

**Q: Can we batch quads with regular draws?**

**A**: They go through the same VRPCommand pipeline, so they can be sorted together, but they're rendered with separate draw calls (different pipelines). This is expected and optimal.
