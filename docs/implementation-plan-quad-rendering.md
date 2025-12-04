# GPU-Instanced Quad Rendering — Implementation Plan (Post Unified-State Refactor)

**Status**: Ready for Implementation
**Last Updated**: 2025-12-04
**Context**: Replaces problematic `DeferredCommand` CPU vertex generation with GPU-driven instanced rendering

---

## Executive Summary

Replace the fragile `DeferredCommand` system (billboards, sprites, rects, text) with GPU-instanced quad rendering:

- **Single static unit quad mesh** (4 vertices, 6 indices)
- **Per-instance attributes** (position, size, rotation, UV, color, mode, shading state)
- **GPU vertex shader expansion** (billboards face camera, sprites in screen-space)
- **Batch instanced draws** by texture/material/mode
- **Eliminate CPU vertex generation** in `process_draw_commands()`

**Benefits**:
- Fixes shading state index bugs (root cause: deferred processing)
- Better performance (GPU-driven, fewer draw calls)
- Cleaner architecture (no delayed processing)
- Supports thousands of quads with minimal CPU overhead

---

## Background: Why This Refactor is Needed

### The Problem with DeferredCommand

**Current flow** (problematic):
```
1. Game render() → queue DeferredCommand { billboard/sprite/rect/text }
2. End of frame → process_draw_commands()
3. CPU generates quad vertices (needs camera info)
4. Hard-coded ShadingStateIndex(0) → PANIC if state pool empty
5. Upload vertices → VRPCommand → GPU draw
```

**Issues**:
- Shading state created during `render()`, but deferred commands processed later
- State pool may be empty when deferred commands are expanded
- CPU vertex generation is expensive and complex
- Billboard math duplicated on CPU instead of GPU
- Fragile state management across frame boundary

**Root cause of current panics**:
- `DeferredCommand` uses hard-coded `ShadingStateIndex(0)` (line 1068, 1193, 1262)
- If game only uses deferred drawing, state pool is empty
- Accessing state[0] during sort/render → out-of-bounds panic

### The Solution: GPU Instancing

**New flow** (robust):
```
1. Game render() → create QuadInstance { position, size, mode, UV, color, shading_state_index }
2. Shading state created IMMEDIATELY (while state is valid)
3. End of frame → upload instance buffer
4. GPU vertex shader expands quad based on camera and mode
5. Single instanced draw call → thousands of quads
```

**Benefits**:
- Shading state captured when created (no index mismatch)
- GPU does billboard math (camera basis, rotation)
- Batching reduces draw calls dramatically
- Simpler CPU code (no vertex generation)

---

## Current Architecture (Post Unified-State Refactor)

### Key Components

**VRPCommand** (`emberware-z/src/graphics/command_buffer.rs:26-48`):
```rust
pub struct VRPCommand {
    pub format: u8,                    // Vertex format (16 permutations)
    pub mvp_index: MvpIndex,           // Packed matrix indices
    pub vertex_count: u32,
    pub index_count: u32,
    pub base_vertex: u32,
    pub first_index: u32,
    pub buffer_source: BufferSource,   // Immediate or Retained
    pub texture_slots: [TextureHandle; 4],
    pub shading_state_index: ShadingStateIndex,  // NEW: unified shading state
    pub depth_test: bool,
    pub cull_mode: CullMode,
}
```

**BufferManager** (`emberware-z/src/graphics/buffer.rs:179-187`):
- Per-format vertex buffers (16 formats: POS, POS_UV, POS_UV_COLOR, POS_UV_NORMAL, etc.)
- Separate immediate (frame-transient) and retained (persistent) buffers
- GrowableBuffer type (dynamic GPU buffers)

**Shading States** (`emberware-z/src/graphics/unified_shading_state.rs`):
- `PackedUnifiedShadingState`: 64 bytes (color, blend, lights, sky, roughness, metalness, etc.)
- Per-frame GPU buffer: `shading_state_buffer`
- Deduplication via HashMap in ZFFIState

**DeferredCommand** (`emberware-z/src/state.rs:77-119`):
- Enum variants: DrawBillboard, DrawSprite, DrawRect, DrawText
- Processed in `graphics/mod.rs:process_draw_commands()` (line 922+)
- **THIS WILL BE REMOVED**

---

## Implementation Plan

### Phase 0: Quick Fix (Optional Interim)

**Purpose**: Get examples working immediately while implementing full solution

**Changes**:
1. **Fix Panic #1** - Line 2048 in `emberware-z/src/graphics/mod.rs`:
   ```rust
   // Change from:
   let pipeline_entry = self.pipeline_cache.get(...).unwrap();
   // To:
   let pipeline_entry = self.pipeline_cache.get_or_create(&self.device, ...);
   ```

2. **Fix Panic #2** - After line 918 in `process_draw_commands()`:
   ```rust
   // Add before deferred command processing:
   if z_state.shading_states.is_empty() {
       z_state.add_shading_state();
   }
   ```

**Time**: 15 minutes
**Result**: All examples work, ready for refactor

---

### Phase 1: Infrastructure

**Goal**: Add quad instancing without breaking existing code

#### Step 1.1: Define QuadInstance

**New file**: `emberware-z/src/graphics/quad_instance.rs`

```rust
/// Quad rendering mode
#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuadMode {
    BillboardSpherical = 0,      // Fully camera-facing
    BillboardCylindricalY = 1,   // Rotate around world Y
    BillboardCylindricalX = 2,   // Rotate around world X
    BillboardCylindricalZ = 3,   // Rotate around world Z
    ScreenSpace = 4,             // 2D sprite overlay
    WorldSpace = 5,              // Standard quad (uses model matrix)
}

/// Per-instance quad data (48 bytes, must be bytemuck::Pod)
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct QuadInstance {
    pub position: [f32; 3],            // 12 bytes - world or screen coords
    pub size: [f32; 2],                // 8 bytes - width, height
    pub rotation: f32,                 // 4 bytes - radians (sprites only)
    pub mode: u32,                     // 4 bytes - QuadMode enum
    pub uv: [f32; 4],                  // 16 bytes - (u0, v0, u1, v1)
    pub color: u32,                    // 4 bytes - packed RGBA8
    pub shading_state_index: u32,      // 4 bytes - index into shading_states buffer
    _padding: u32,                     // 4 bytes - pad to 48 (16-byte aligned)
}
```

Add module to `emberware-z/src/graphics/mod.rs`:
```rust
mod quad_instance;
pub use quad_instance::{QuadInstance, QuadMode};
```

#### Step 1.2: Create Static Unit Quad

**Location**: `emberware-z/src/graphics/mod.rs` - in `ZGraphics` struct

Add fields:
```rust
pub struct ZGraphics {
    // ... existing fields ...

    // Unit quad for instanced rendering
    unit_quad_format: u8,
    unit_quad_base_vertex: u32,
    unit_quad_first_index: u32,
    unit_quad_index_count: u32,
}
```

Add to `ZGraphics::new()` (before `Ok(Self { ... })`):
```rust
// Create static unit quad mesh
const QUAD_FORMAT: u8 = FORMAT_UV | FORMAT_COLOR; // POS_UV_COLOR = 0b011 = 3

let unit_quad_vertices: Vec<f32> = vec![
    // pos_x, pos_y, pos_z, uv_u, uv_v, color_r, color_g, color_b
    -0.5, -0.5, 0.0,  0.0, 0.0,  1.0, 1.0, 1.0,  // Bottom-left
     0.5, -0.5, 0.0,  1.0, 0.0,  1.0, 1.0, 1.0,  // Bottom-right
     0.5,  0.5, 0.0,  1.0, 1.0,  1.0, 1.0, 1.0,  // Top-right
    -0.5,  0.5, 0.0,  0.0, 1.0,  1.0, 1.0, 1.0,  // Top-left
];

let unit_quad_indices: Vec<u16> = vec![
    0, 1, 2,  // First triangle
    0, 2, 3,  // Second triangle
];

let (quad_base_vertex, quad_first_index) = buffer_manager.upload_retained_mesh(
    QUAD_FORMAT,
    &unit_quad_vertices,
    &unit_quad_indices,
    "Unit Quad Mesh",
)?;

// Store in ZGraphics initialization
unit_quad_format: QUAD_FORMAT,
unit_quad_base_vertex: quad_base_vertex,
unit_quad_first_index: quad_first_index,
unit_quad_index_count: 6,
```

#### Step 1.3: Add Instance Buffer

**Location**: `emberware-z/src/graphics/buffer.rs`

Add to `BufferManager`:
```rust
pub struct BufferManager {
    // ... existing fields ...

    /// Quad instance buffer (per-frame)
    quad_instance_buffer: GrowableBuffer,
}
```

Initialize in `BufferManager::new()`:
```rust
quad_instance_buffer: GrowableBuffer::new(
    device,
    "Quad Instance Buffer",
    1024 * std::mem::size_of::<QuadInstance>() as u64,
    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
),
```

Add methods:
```rust
pub fn upload_quad_instances(
    &mut self,
    queue: &wgpu::Queue,
    instances: &[QuadInstance],
) -> Result<()> {
    let byte_data = bytemuck::cast_slice(instances);
    self.quad_instance_buffer.upload(queue, byte_data)?;
    Ok(())
}

pub fn quad_instance_buffer(&self) -> &wgpu::Buffer {
    self.quad_instance_buffer.buffer()
}
```

#### Step 1.4: Track Instances in State

**Location**: `emberware-z/src/state.rs`

Add to `ZFFIState`:
```rust
pub struct ZFFIState {
    // ... existing fields ...

    /// Quad instances for GPU-driven rendering
    pub quad_instances: Vec<QuadInstance>,
}
```

Initialize:
```rust
impl Default for ZFFIState {
    fn default() -> Self {
        Self {
            // ... existing fields ...
            quad_instances: Vec::with_capacity(256),
        }
    }
}
```

Clear in `clear_frame()`:
```rust
pub fn clear_frame(&mut self) {
    // ... existing clears ...
    self.quad_instances.clear();
}
```

**Checkpoint**: Code compiles, examples still work.

---

### Phase 2: Migrate Billboards

**Goal**: Replace `DeferredCommand::DrawBillboard` with `QuadInstance`

#### Step 2.1: Update FFI Function

**Location**: `emberware-z/src/ffi/mod.rs` - `draw_billboard()` function

Find current code (around line 1440):
```rust
state.deferred_commands.push(DeferredCommand::DrawBillboard {
    width,
    height,
    mode,
    uv_rect,
    transform: state.current_transform,
    color: state.current_shading_state.color(),
    depth_test: state.depth_test,
    cull_mode,
    bound_textures: state.bound_textures,
});
```

**Replace with**:
```rust
use crate::graphics::{QuadInstance, QuadMode};

// Capture shading state IMMEDIATELY (critical!)
let shading_state_index = state.add_shading_state();

// Map mode to QuadMode enum
let quad_mode = match mode {
    1 => QuadMode::BillboardSpherical,
    2 => QuadMode::BillboardCylindricalY,
    3 => QuadMode::BillboardCylindricalX,
    4 => QuadMode::BillboardCylindricalZ,
    _ => QuadMode::BillboardSpherical,
};

// Extract position from current transform
let position = [
    state.current_transform.w_axis.x,
    state.current_transform.w_axis.y,
    state.current_transform.w_axis.z,
];

// Create instance
let instance = QuadInstance {
    position,
    size: [width, height],
    rotation: 0.0, // Unused for billboards
    mode: quad_mode as u32,
    uv: uv_rect.map_or([0.0, 0.0, 1.0, 1.0], |(u0, v0, u1, v1)| [u0, v0, u1, v1]),
    color: state.current_shading_state.color(),
    shading_state_index: shading_state_index.0,
    _padding: 0,
};

state.quad_instances.push(instance);
```

#### Step 2.2: Process Instances in Render

**Location**: `emberware-z/src/graphics/mod.rs` - `process_draw_commands()`

Find deferred command processing loop (line 922):
```rust
for cmd in z_state.deferred_commands.drain(..) {
    match cmd {
        DeferredCommand::DrawBillboard { ... } => {
            // OLD: CPU vertex generation
        }
    }
}
```

**Replace billboard case** with:
```rust
// Process quad instances (before or after deferred commands)
if !z_state.quad_instances.is_empty() {
    // Upload instances to GPU
    self.buffer_manager.upload_quad_instances(
        &self.queue,
        &z_state.quad_instances,
    )?;

    // Sort instances by texture/material for batching
    // For now, single draw call for all instances
    let instance_count = z_state.quad_instances.len() as u32;

    // Create instanced VRPCommand
    self.command_buffer.add_instanced_draw(
        self.unit_quad_format,
        self.unit_quad_base_vertex,
        self.unit_quad_first_index,
        self.unit_quad_index_count,
        instance_count,
        MvpIndex::IDENTITY, // Instances have their own transforms
        [TextureHandle::INVALID; 4], // TODO: Batch by texture
        true,  // depth_test
        CullMode::None,
    );
}
```

#### Step 2.3: Extend VRPCommand

**Location**: `emberware-z/src/graphics/command_buffer.rs`

**Option 1** (Simpler): Add instance_count field
```rust
pub struct VRPCommand {
    // ... existing fields ...

    /// Instance count for instanced draws (1 = non-instanced)
    pub instance_count: u32,
}
```

Update all `VRPCommand` construction sites to include `instance_count: 1`.

Add method to `VirtualRenderPass`:
```rust
pub fn add_instanced_draw(
    &mut self,
    format: u8,
    base_vertex: u32,
    first_index: u32,
    index_count: u32,
    instance_count: u32,
    mvp_index: MvpIndex,
    texture_slots: [TextureHandle; 4],
    depth_test: bool,
    cull_mode: CullMode,
) {
    self.commands.push(VRPCommand {
        format,
        mvp_index,
        vertex_count: 0, // Not used for indexed instanced draws
        index_count,
        base_vertex,
        first_index,
        buffer_source: BufferSource::Retained, // Unit quad is retained
        texture_slots,
        shading_state_index: ShadingStateIndex(0), // Comes from instance data
        depth_test,
        cull_mode,
        instance_count,
    });
}
```

#### Step 2.4: Update Render Loop

**Location**: `emberware-z/src/graphics/mod.rs` - `render_frame()` render pass

Find render loop (around line 2100+):
```rust
for cmd in self.command_buffer.commands() {
    // ... set pipeline, bind textures ...

    if cmd.index_count > 0 {
        render_pass.draw_indexed(
            cmd.first_index..(cmd.first_index + cmd.index_count),
            cmd.base_vertex as i32,
            0..1, // Single instance
        );
    }
}
```

**Update to handle instancing**:
```rust
if cmd.index_count > 0 {
    let instance_count = if cmd.instance_count == 0 { 1 } else { cmd.instance_count };

    render_pass.draw_indexed(
        cmd.first_index..(cmd.first_index + cmd.index_count),
        cmd.base_vertex as i32,
        0..instance_count, // Multiple instances
    );
}
```

#### Step 2.5: Update Shaders

**Location**: `emberware-z/src/shader_gen.rs`

**Challenge**: Current shaders generated from templates for 40 permutations (4 render modes × 10 vertex formats). Need to add instance attributes conditionally.

**Approach**: Add instance attributes to pipeline when `instance_count > 1`.

Add instance vertex attributes:
```wgsl
// Standard vertex attributes (@location 0-4 depending on format)
@location(0) position: vec3<f32>,
@location(1) uv: vec2<f32>,         // If FORMAT_UV
@location(2) color: vec3<f32>,      // If FORMAT_COLOR
@location(3) normal: vec3<f32>,     // If FORMAT_NORMAL

// Instance attributes (instanced = true in vertex buffer layout)
@location(5) instance_position: vec3<f32>,
@location(6) instance_size: vec2<f32>,
@location(7) instance_rotation: f32,
@location(8) instance_mode: u32,
@location(9) instance_uv: vec4<f32>,
@location(10) instance_color: u32,
@location(11) instance_shading_state_index: u32,
```

Add billboard helper function:
```wgsl
fn apply_billboard(
    local_pos: vec2<f32>,
    instance_size: vec2<f32>,
    billboard_mode: u32,
    view_matrix: mat4x4<f32>
) -> vec3<f32> {
    // Extract camera basis from view matrix
    let cam_right = vec3<f32>(view_matrix[0].x, view_matrix[0].y, view_matrix[0].z);
    let cam_up = vec3<f32>(view_matrix[1].x, view_matrix[1].y, view_matrix[1].z);
    let cam_forward = -vec3<f32>(view_matrix[2].x, view_matrix[2].y, view_matrix[2].z);

    let scaled_x = local_pos.x * instance_size.x;
    let scaled_y = local_pos.y * instance_size.y;

    // Billboard mode dispatch
    if (billboard_mode == 0u) { // Spherical
        return cam_right * scaled_x + cam_up * scaled_y;
    } else if (billboard_mode == 1u) { // Cylindrical Y
        let forward_xz = vec3<f32>(cam_forward.x, 0.0, cam_forward.z);
        let len_sq = dot(forward_xz, forward_xz);
        if (len_sq < 0.0001) {
            // Camera pointing straight up/down
            return cam_right * scaled_x + vec3<f32>(0.0, 1.0, 0.0) * scaled_y;
        }
        let forward_norm = normalize(forward_xz);
        let right = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), forward_norm));
        return right * scaled_x + vec3<f32>(0.0, 1.0, 0.0) * scaled_y;
    } else if (billboard_mode == 2u) { // Cylindrical X
        // TODO: Implement X-axis cylindrical
        return vec3<f32>(0.0, scaled_y, scaled_x);
    } else if (billboard_mode == 3u) { // Cylindrical Z
        // TODO: Implement Z-axis cylindrical
        return vec3<f32>(scaled_x, scaled_y, 0.0);
    }

    // Default: no billboarding
    return vec3<f32>(scaled_x, scaled_y, 0.0);
}
```

Update vertex shader main:
```wgsl
@vertex
fn vs_main(vertex: VertexInput, instance: InstanceInput) -> VertexOutput {
    // Determine if this is an instanced draw (instance_mode != 0xFFFFFFFF sentinel)
    let is_instanced = instance.instance_mode < 10u;

    var world_pos: vec3<f32>;
    var final_uv: vec2<f32>;
    var final_color: vec3<f32>;
    var shading_idx: u32;

    if (is_instanced) {
        // Instanced quad rendering
        if (instance.instance_mode < 4u) {
            // Billboard mode
            let billboard_offset = apply_billboard(
                vertex.position.xy,
                instance.instance_size,
                instance.instance_mode,
                view_matrices[0]
            );
            world_pos = instance.instance_position + billboard_offset;
        } else if (instance.instance_mode == 4u) {
            // Screen-space sprite
            // TODO: Transform to NDC
            world_pos = instance.instance_position;
        } else {
            // World-space quad (use model matrix)
            world_pos = instance.instance_position + vertex.position.xyz;
        }

        // Use instance UV and color
        final_uv = mix(instance.instance_uv.xy, instance.instance_uv.zw, vertex.uv);
        final_color = unpack_color(instance.instance_color);
        shading_idx = instance.instance_shading_state_index;
    } else {
        // Standard non-instanced draw
        world_pos = (model_matrices[mvp.model_idx] * vec4<f32>(vertex.position, 1.0)).xyz;
        final_uv = vertex.uv;
        final_color = vertex.color;
        shading_idx = draw_shading_state_index; // From push constant or uniform
    }

    // Rest of vertex shader (MVP transform, lighting, etc.)
    let clip_pos = proj_matrices[mvp.proj_idx] * view_matrices[mvp.view_idx] * vec4<f32>(world_pos, 1.0);

    return VertexOutput {
        @builtin(position) clip_position: clip_pos,
        @location(0) uv: final_uv,
        @location(1) color: final_color,
        // ...
    };
}
```

**Note**: This is a significant shader change. May require generating separate shaders for instanced vs non-instanced paths.

**Checkpoint**: Billboard example works with GPU instancing.

---

### Phase 3: Migrate Sprites

Similar process to billboards:

1. Update `draw_sprite()` FFI to create `QuadInstance` with `mode = QuadMode::ScreenSpace`
2. Handle screen-space transformation in shader:
   ```wgsl
   // Screen-space mode (4u)
   let screen_pos = instance.instance_position.xy;
   let screen_size = instance.instance_size;

   // Apply rotation if needed
   let rotated_local = rotate_2d(vertex.position.xy, instance.instance_rotation);
   let final_screen_pos = screen_pos + rotated_local * screen_size;

   // Convert to NDC (-1..1)
   let ndc_x = (final_screen_pos.x / render_width) * 2.0 - 1.0;
   let ndc_y = 1.0 - (final_screen_pos.y / render_height) * 2.0;
   ```

---

### Phase 4: Migrate Rects and Text

**Rects**: Simple - `QuadInstance` with solid color, default UV

**Text**: More complex - options:
1. Keep deferred (text layout requires CPU processing)
2. Generate glyph quads as instances
3. Use SDF font rendering

**Recommendation**: Keep text deferred for now, migrate rects to instances.

---

### Phase 5: Cleanup

1. Remove `DeferredCommand` enum from `state.rs`
2. Remove CPU vertex generation from `process_draw_commands()`
3. Remove `deferred_commands` field from `ZFFIState`
4. Update all examples
5. Performance benchmark
6. Documentation

---

## Testing & Validation

### Per-Phase Tests

**Phase 0**: Quick fix
```bash
cargo test
cargo run -- billboard  # Should not panic
cargo run -- cube       # Should not panic
```

**Phase 1**: Infrastructure
```bash
cargo build  # Must compile
cargo run -- cube  # Existing examples still work
```

**Phase 2**: Billboards
```bash
cargo run -- billboard
# Verify: No panics, billboards face camera correctly
```

**Phase 3-4**: Sprites, Rects
```bash
cargo run -- hello-world
cargo run -- platformer
# Verify: Sprites render, no regressions
```

**Phase 5**: Final
```bash
# Run all examples
for example in cube triangle billboard hello-world lighting platformer skinned-mesh textured-quad; do
    cargo run --release -- $example
done
```

### Performance Benchmarks

Compare before/after:
- Draw call count (use GPU profiler)
- CPU frame time
- 1000 quads stress test

Expected improvements:
- **Draw calls**: 1000 → 1 (batched)
- **CPU time**: -50% to -80% (no vertex generation)
- **GPU time**: Minimal change (same triangles)

---

## Migration Complexity

**Low Risk**:
- Phase 0 (quick fix): 2 line changes
- Phase 1 (infrastructure): Additive, no behavior change

**Medium Risk**:
- Phase 2 (billboards): Shader changes, new code paths
- Phase 3 (sprites): Screen-space transformation

**High Risk**:
- Shader generation (40 permutations, need instanced variants)
- Text rendering (may need rework)

**Mitigation**:
- Keep old code behind feature flag during migration
- Test after each phase
- Incremental rollout (billboard first, sprites later)

---

## Open Questions & Decisions

### Q1: Shader Generation Strategy

**Options**:
1. Generate separate instanced shaders for each permutation (80 shaders total)
2. Use dynamic branching in shaders (check instance_mode)
3. Hybrid: instanced-only shaders for quad rendering

**Recommendation**: Option 3 - separate shader for instanced quads, keeps existing shaders unchanged.

### Q2: Text Rendering

**Options**:
1. Keep `DeferredCommand::DrawText` (hybrid approach)
2. Pre-layout text, generate glyph instances
3. GPU-driven text rendering (advanced)

**Recommendation**: Option 1 for now - keep text deferred, migrate later.

### Q3: Texture Batching

Instances with different textures can't batch. Solutions:
1. Texture arrays (bind multiple textures, use index per instance)
2. Texture atlas (single texture, UV per instance)
3. Multiple draw calls (batch by texture)

**Recommendation**: Start with Option 3 (simple), upgrade to texture array later.

### Q4: Instance Buffer Size

**Current**: 1024 instances (48 KB)

**Considerations**:
- Platformer: ~100 sprites
- Particle effects: 1000+ quads
- UI: 50-200 elements

**Recommendation**: Start with 1024, grow dynamically if needed.

---

## Timeline

**Phase 0**: 30 minutes
**Phase 1**: 4-6 hours
**Phase 2**: 8-12 hours (shader work)
**Phase 3**: 4-6 hours
**Phase 4**: 2-4 hours
**Phase 5**: 2-4 hours

**Total**: 3-4 days

---

## Success Criteria

- [ ] All 8 examples render correctly
- [ ] No panics or crashes
- [ ] Billboard example uses GPU instancing
- [ ] Sprite examples use GPU instancing
- [ ] Draw call count reduced by 10x+ for 100+ quads
- [ ] CPU frame time improved
- [ ] Code is simpler and more maintainable
- [ ] `DeferredCommand` enum removed

---

## References

- Original plan: `docs/implementation-plan-quad-rendering.md` (outdated)
- Unified shading state: `docs/implementation-plan-unified-shading-state.md`
- Vertex formats: `emberware-z/src/graphics/vertex.rs`
- Command buffer: `emberware-z/src/graphics/command_buffer.rs`
- Buffer management: `emberware-z/src/graphics/buffer.rs`
