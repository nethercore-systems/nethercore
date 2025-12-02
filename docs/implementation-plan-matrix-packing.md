# Implementation Plan: Matrix Index Packing

**Status:** Not Started
**Estimated Effort:** 3-5 days
**Priority:** High (implement first)
**Related:** [matrix-index-packing.md](./matrix-index-packing.md), [rendering-architecture.md](./rendering-architecture.md)

---

## Overview

Replace per-draw 64-byte Mat4 storage with 4-byte packed matrix indices, uploading matrices in bulk once per frame instead of per-draw.

**Benefits:**
- **16× reduction** in transform storage (64 → 4 bytes per draw)
- Zero per-draw matrix uploads
- Scales to tens of thousands of instances
- Cross-backend compatible (no push constants needed)

**Approach:** Instance buffer (Option A) for maximum compatibility

---

## Phase 1: Add Matrix Pool Infrastructure

**Estimated Time:** 4-6 hours

### Files to Modify
- `emberware-z/src/state.rs`
- `emberware-z/src/graphics/mod.rs`
- `emberware-z/src/graphics/matrix_packing.rs` (new file)

### Changes

#### 1.1: Create MvpIndex Newtype

**New file:** `emberware-z/src/graphics/matrix_packing.rs`

```rust
/// Packed MVP matrix indices (model: 16 bits, view: 8 bits, proj: 8 bits)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MvpIndex(pub u32);

impl MvpIndex {
    pub const INVALID: Self = Self(0);

    /// Pack three matrix indices into a single u32
    pub fn new(model: u32, view: u32, proj: u32) -> Self {
        debug_assert!(model < 65536, "Model index must fit in 16 bits");
        debug_assert!(view < 256, "View index must fit in 8 bits");
        debug_assert!(proj < 256, "Projection index must fit in 8 bits");

        Self((model & 0xFFFF) | ((view & 0xFF) << 16) | ((proj & 0xFF) << 24))
    }

    /// Unpack into (model, view, proj) indices
    pub fn unpack(self) -> (u32, u32, u32) {
        let model = self.0 & 0xFFFF;
        let view = (self.0 >> 16) & 0xFF;
        let proj = (self.0 >> 24) & 0xFF;
        (model, view, proj)
    }

    pub fn model_index(self) -> u32 {
        self.0 & 0xFFFF
    }

    pub fn view_index(self) -> u32 {
        (self.0 >> 16) & 0xFF
    }

    pub fn proj_index(self) -> u32 {
        (self.0 >> 24) & 0xFF
    }
}
```

#### 1.2: Add Matrix Pools to ZFFIState

**File:** `emberware-z/src/state.rs`

```rust
pub struct ZFFIState {
    // Existing fields...

    // New: Matrix pools (reset each frame)
    pub model_matrices: Vec<Mat4>,
    pub view_matrices: Vec<Mat4>,
    pub proj_matrices: Vec<Mat4>,

    // New: Current matrix indices
    pub current_model_idx: u32,
    pub current_view_idx: u32,
    pub current_proj_idx: u32,
}

impl ZFFIState {
    pub fn new(...) -> Self {
        Self {
            // Existing initialization...

            model_matrices: Vec::with_capacity(256),
            view_matrices: Vec::with_capacity(4),
            proj_matrices: Vec::with_capacity(4),
            current_model_idx: 0,
            current_view_idx: 0,
            current_proj_idx: 0,
        }
    }

    /// Add a model matrix to the pool and return its index
    pub fn add_model_matrix(&mut self, matrix: Mat4) -> u32 {
        let idx = self.model_matrices.len() as u32;
        if idx >= 65536 {
            tracing::warn!("Model matrix pool overflow (max 65536)");
            return 65535; // Return max valid index
        }
        self.model_matrices.push(matrix);
        idx
    }

    /// Pack current matrix indices into MvpIndex
    pub fn pack_current_mvp(&self) -> MvpIndex {
        MvpIndex::new(
            self.current_model_idx,
            self.current_view_idx,
            self.current_proj_idx
        )
    }

    /// Reset matrix pools at start of frame
    pub fn reset_matrix_pools(&mut self) {
        self.model_matrices.clear();
        // View and proj matrices typically persist, but clear if needed
        // self.view_matrices.clear();
        // self.proj_matrices.clear();
    }
}
```

#### 1.3: Add Matrix Buffers to ZGraphics

**File:** `emberware-z/src/graphics/mod.rs`

```rust
pub struct ZGraphics {
    // Existing fields...

    // New: GPU matrix storage buffers
    model_matrix_buffer: wgpu::Buffer,
    view_matrix_buffer: wgpu::Buffer,
    proj_matrix_buffer: wgpu::Buffer,

    model_matrix_capacity: usize,
    view_matrix_capacity: usize,
    proj_matrix_capacity: usize,
}

impl ZGraphics {
    pub fn new(...) -> Result<Self> {
        // Existing initialization...

        // Create initial matrix buffers
        let model_matrix_capacity = 1024;
        let model_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Matrices"),
            size: (model_matrix_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let view_matrix_capacity = 16;
        let view_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("View Matrices"),
            size: (view_matrix_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let proj_matrix_capacity = 16;
        let proj_matrix_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection Matrices"),
            size: (proj_matrix_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            // Existing fields...
            model_matrix_buffer,
            view_matrix_buffer,
            proj_matrix_buffer,
            model_matrix_capacity,
            view_matrix_capacity,
            proj_matrix_capacity,
        })
    }

    /// Ensure model matrix buffer has sufficient capacity
    fn ensure_model_buffer_capacity(&mut self, count: usize) {
        if count <= self.model_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!("Growing model matrix buffer: {} → {}", self.model_matrix_capacity, new_capacity);

        self.model_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.model_matrix_capacity = new_capacity;
    }
}
```

---

## Phase 2: Modify VRPCommand Structure

**Estimated Time:** 2-3 hours

### Files to Modify
- `emberware-z/src/graphics/command_buffer.rs`

### Changes

#### 2.1: Update VRPCommand

**File:** `emberware-z/src/graphics/command_buffer.rs`

```rust
pub struct VRPCommand {
    pub format: u8,
    pub mvp_index: MvpIndex,         // New: 4 bytes
    // REMOVED: pub transform: Mat4, // Old: 64 bytes
    pub vertex_count: u32,
    pub index_count: u32,
    pub base_vertex: u32,
    pub first_index: u32,
    pub buffer_source: BufferSource,
    pub texture_slots: [TextureHandle; 4],
    pub color: u32,
    pub depth_test: bool,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
    pub matcap_blend_modes: [MatcapBlendMode; 4],
}
```

#### 2.2: Update VirtualRenderPass Methods

**File:** `emberware-z/src/graphics/command_buffer.rs`

Update all `record_*` method signatures to accept `mvp_index: MvpIndex` instead of `transform: Mat4`:

```rust
pub fn record_triangles(
    &mut self,
    format: u8,
    vertex_data: &[f32],
    mvp_index: MvpIndex,  // Changed from Mat4
    color: u32,
    depth_test: bool,
    cull_mode: CullMode,
    blend_mode: BlendMode,
    texture_slots: [TextureHandle; 4],
    matcap_blend_modes: [MatcapBlendMode; 4],
) {
    // ... implementation (use mvp_index instead of transform)
}

pub fn record_mesh(
    &mut self,
    format: u8,
    vertex_count: u32,
    index_count: u32,
    vertex_offset: u64,
    index_offset: u64,
    mvp_index: MvpIndex,  // Changed from Mat4
    color: u32,
    // ... rest of parameters
) {
    // ... implementation
}
```

---

## Phase 3: Update FFI Layer

**Estimated Time:** 4-6 hours

### Files to Modify
- `emberware-z/src/ffi/mod.rs`

### Changes

#### 3.1: Update Camera Functions

```rust
fn camera_set(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32, y: f32, z: f32,
    target_x: f32, target_y: f32, target_z: f32,
    fov: f32,
    near: f32,
    far: f32,
) {
    let state = &mut caller.data_mut().console;

    state.camera.position = Vec3::new(x, y, z);
    state.camera.target = Vec3::new(target_x, target_y, target_z);
    state.camera.fov = fov;
    state.camera.near = near;
    state.camera.far = far;

    // Build view matrix
    let view = Mat4::look_at_rh(
        state.camera.position,
        state.camera.target,
        Vec3::Y,
    );

    // Update view matrix pool
    if state.view_matrices.is_empty() {
        state.view_matrices.push(view);
        state.current_view_idx = 0;
    } else {
        state.view_matrices[0] = view;
    }

    // Build projection matrix
    let aspect = state.viewport_width as f32 / state.viewport_height as f32;
    let proj = Mat4::perspective_rh(
        fov.to_radians(),
        aspect,
        near,
        far,
    );

    // Update projection matrix pool
    if state.proj_matrices.is_empty() {
        state.proj_matrices.push(proj);
        state.current_proj_idx = 0;
    } else {
        state.proj_matrices[0] = proj;
    }
}
```

#### 3.2: Update Draw Commands

**Example: `draw_triangles`**

```rust
fn draw_triangles(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    format: u32,
    ptr: u32,
    vertex_count: u32,
) -> Result<(), Trap> {
    let state = &mut caller.data_mut().console;

    // Copy vertex data from WASM memory
    let vertex_data: Vec<f32> = { /* ... */ };

    // Add current transform to model matrix pool
    let model_idx = state.add_model_matrix(state.current_transform);

    // Pack matrix indices
    let mvp_index = MvpIndex::new(
        model_idx,
        state.current_view_idx,
        state.current_proj_idx,
    );

    // Record command with packed index
    state.render_pass.record_triangles(
        format as u8,
        &vertex_data,
        mvp_index,  // New: MvpIndex instead of Mat4
        state.color,
        state.depth_test,
        state.cull_mode,
        state.blend_mode,
        state.texture_slots,
        state.matcap_blend_modes,
    );

    Ok(())
}
```

**Apply similar changes to:**
- `draw_triangles_indexed`
- `draw_mesh`
- Deferred command expansion in `process_draw_commands()`:
  - `DeferredCommand::DrawBillboard`
  - `DeferredCommand::DrawSprite`
  - `DeferredCommand::DrawText`

---

## Phase 4: Upload Matrix Buffers to GPU

**Estimated Time:** 3-4 hours

### Files to Modify
- `emberware-z/src/graphics/mod.rs`

### Changes

#### 4.1: Upload Matrices Before Rendering

**File:** `emberware-z/src/graphics/mod.rs`

```rust
pub fn render_frame(
    &mut self,
    z_state: &ZFFIState,
    view_matrix: Mat4,
    projection_matrix: Mat4,
    clear_color: [f32; 4],
) -> Result<()> {
    // 1. Upload model matrices
    if !z_state.model_matrices.is_empty() {
        self.ensure_model_buffer_capacity(z_state.model_matrices.len());
        let data = bytemuck::cast_slice(&z_state.model_matrices);
        self.queue.write_buffer(&self.model_matrix_buffer, 0, data);
    }

    // 2. Upload view matrices
    if !z_state.view_matrices.is_empty() {
        let data = bytemuck::cast_slice(&z_state.view_matrices);
        self.queue.write_buffer(&self.view_matrix_buffer, 0, data);
    }

    // 3. Upload projection matrices
    if !z_state.proj_matrices.is_empty() {
        let data = bytemuck::cast_slice(&z_state.proj_matrices);
        self.queue.write_buffer(&self.proj_matrix_buffer, 0, data);
    }

    // 4. Upload immediate vertex/index data
    // ... (existing code)

    // 5. Sort and execute commands
    // ... (existing code)
}
```

#### 4.2: Reset Matrix Pools After Frame

**File:** `emberware-z/src/graphics/mod.rs` or `emberware-z/src/app.rs`

```rust
pub fn begin_frame(&mut self, z_state: &mut ZFFIState) {
    z_state.render_pass.reset();
    z_state.current_transform = Mat4::IDENTITY;
    z_state.transform_stack.clear();

    // Reset model matrix pool
    z_state.reset_matrix_pools();
}
```

---

## Phase 5: Update Shaders

**Estimated Time:** 4-6 hours

### Files to Modify
- `emberware-z/shaders/mode0_unlit.wgsl`
- `emberware-z/shaders/mode1_matcap.wgsl`
- `emberware-z/shaders/mode2_pbr.wgsl`
- `emberware-z/shaders/mode3_hybrid.wgsl`

### Changes (Apply to All 4 Templates)

#### 5.1: Replace Uniform Matrices with Storage Buffers

```wgsl
// OLD: Individual uniforms
@group(0) @binding(0) var<storage, read> model_matrices_old: array<mat4x4<f32>>;  // Was per-instance
@group(0) @binding(1) var<uniform> view: mat4x4<f32>;
@group(0) @binding(2) var<uniform> projection: mat4x4<f32>;

// NEW: All matrices in storage buffers
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;
```

#### 5.2: Add Unpacking Function

```wgsl
/// Unpack MvpIndex into (model, view, proj) indices
fn unpack_mvp(key: u32) -> vec3<u32> {
    let model = key & 0xFFFFu;
    let view = (key >> 16u) & 0xFFu;
    let proj = (key >> 24u) & 0xFFu;
    return vec3<u32>(model, view, proj);
}
```

#### 5.3: Update Vertex Shader

```wgsl
struct VertexInput {
    @location(0) position: vec3<f32>,
    // ... other vertex attributes
    @location(10) mvp_index: u32,  // From instance buffer
}

@vertex
fn vs(in: VertexInput) -> VertexOutput {
    // Unpack matrix indices
    let indices = unpack_mvp(in.mvp_index);

    // Fetch matrices
    let model = model_matrices[indices.x];
    let view = view_matrices[indices.y];
    let proj = proj_matrices[indices.z];

    // Transform vertex
    let world_pos = model * vec4(in.position, 1.0);
    let view_pos = view * world_pos;
    let clip_pos = proj * view_pos;

    var out: VertexOutput;
    out.position = clip_pos;

    // Transform normals (if FORMAT_NORMAL)
    //VIN_NORMAL
    //VS_TRANSFORM_NORMAL

    // Pass through other attributes
    //VS_PASSTHROUGH_UV
    //VS_PASSTHROUGH_COLOR

    return out;
}
```

---

## Phase 6: Update Bind Group Layouts

**Estimated Time:** 3-4 hours

### Files to Modify
- `emberware-z/src/graphics/pipeline.rs`
- `emberware-z/src/graphics/mod.rs`

### Changes

#### 6.1: Update Bind Group 0 Layout

**File:** `emberware-z/src/graphics/pipeline.rs`

```rust
let bind_group_layout_0 = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
    label: Some("Frame Uniforms"),
    entries: &[
        // Binding 0: Model matrices (storage buffer, read-only)
        BindGroupLayoutEntry {
            binding: 0,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 1: View matrices (storage buffer, read-only)
        BindGroupLayoutEntry {
            binding: 1,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // Binding 2: Projection matrices (storage buffer, read-only)
        BindGroupLayoutEntry {
            binding: 2,
            visibility: ShaderStages::VERTEX,
            ty: BindingType::Buffer {
                ty: BufferBindingType::Storage { read_only: true },
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        },
        // ... rest of bindings (sky, material, lights, bones)
    ],
});
```

#### 6.2: Create Bind Group with Matrix Buffers

**File:** `emberware-z/src/graphics/mod.rs`

```rust
fn create_frame_bind_group(&self) -> wgpu::BindGroup {
    self.device.create_bind_group(&BindGroupDescriptor {
        label: Some("Frame Uniforms"),
        layout: &self.frame_bind_group_layout,
        entries: &[
            BindGroupEntry {
                binding: 0,
                resource: self.model_matrix_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 1,
                resource: self.view_matrix_buffer.as_entire_binding(),
            },
            BindGroupEntry {
                binding: 2,
                resource: self.proj_matrix_buffer.as_entire_binding(),
            },
            // ... rest of bindings (sky, material, lights, bones)
        ],
    })
}
```

---

## Phase 7: Instance Data for MVP Keys

**Estimated Time:** 4-6 hours

**Selected Approach:** Instance buffer (ensures cross-backend compatibility)

### Files to Modify
- `emberware-z/src/graphics/buffer.rs`
- `emberware-z/src/graphics/mod.rs`
- `emberware-z/src/graphics/pipeline.rs`

### Changes

#### 7.1: Add Instance Buffer to BufferManager

**File:** `emberware-z/src/graphics/buffer.rs`

```rust
pub struct BufferManager {
    // Existing fields...

    // New: Instance buffer for MVP indices
    instance_buffer: GrowableBuffer,
}

impl BufferManager {
    pub fn new(device: &wgpu::Device) -> Self {
        let instance_buffer = GrowableBuffer::new(
            device,
            "Instance Buffer (MVP Indices)",
            4096,  // Initial capacity: 1024 instances
            wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        );

        Self {
            // Existing fields...
            instance_buffer,
        }
    }

    pub fn instance_buffer(&self) -> &wgpu::Buffer {
        self.instance_buffer.buffer()
    }
}
```

#### 7.2: Populate Instance Buffer During Render

**File:** `emberware-z/src/graphics/mod.rs`

```rust
pub fn render_frame(&mut self, ...) -> Result<()> {
    // 1. Upload matrices (from Phase 4)
    // ...

    // 2. Collect MVP indices from commands
    let mvp_indices: Vec<u32> = self.command_buffer.commands()
        .iter()
        .map(|cmd| cmd.mvp_index.0)  // Extract inner u32 from MvpIndex newtype
        .collect();

    // 3. Upload to instance buffer
    if !mvp_indices.is_empty() {
        let byte_size = (mvp_indices.len() * std::mem::size_of::<u32>()) as u64;
        self.buffer_manager.instance_buffer
            .ensure_capacity(&self.device, byte_size);

        self.buffer_manager.instance_buffer
            .write_at(&self.queue, 0, bytemuck::cast_slice(&mvp_indices));
    }

    // 4. Sort commands
    // ...

    // 5. Execute render pass
    // ...
}
```

#### 7.3: Set Instance Buffer in Render Pass

**File:** `emberware-z/src/graphics/mod.rs` (in render pass execution loop)

```rust
let mut render_pass = encoder.begin_render_pass(...);

for (instance_index, cmd) in self.command_buffer.commands().iter().enumerate() {
    // Set vertex buffer (vertex data)
    let vertex_buffer = match cmd.buffer_source {
        BufferSource::Immediate => self.buffer_manager.vertex_buffer(cmd.format),
        BufferSource::Retained => self.buffer_manager.retained_vertex_buffer(cmd.format),
    };
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

    // Set instance buffer (MVP indices)
    let instance_offset = (instance_index * std::mem::size_of::<u32>()) as u64;
    render_pass.set_vertex_buffer(
        1,
        self.buffer_manager.instance_buffer()
            .slice(instance_offset..instance_offset + 4)
    );

    // Set index buffer (if indexed)
    if cmd.index_count > 0 {
        // ...
    }

    // Draw with instancing (instance count = 1)
    if cmd.index_count > 0 {
        render_pass.draw_indexed(
            cmd.first_index..cmd.first_index + cmd.index_count,
            cmd.base_vertex as i32,
            0..1,  // Instance range
        );
    } else {
        render_pass.draw(
            cmd.base_vertex..cmd.base_vertex + cmd.vertex_count,
            0..1,  // Instance range
        );
    }
}
```

#### 7.4: Update Vertex Buffer Layout for Instance Data

**File:** `emberware-z/src/graphics/pipeline.rs`

```rust
fn create_vertex_buffer_layouts(format: u8) -> Vec<VertexBufferLayout<'static>> {
    vec![
        // Buffer 0: Per-vertex data (position, UV, color, normal, etc.)
        VertexBufferLayout {
            array_stride: vertex_stride(format) as u64,
            step_mode: VertexStepMode::Vertex,
            attributes: &vertex_attributes(format),  // Locations 0-5
        },

        // Buffer 1: Per-instance data (MVP index)
        VertexBufferLayout {
            array_stride: 4,  // sizeof(u32)
            step_mode: VertexStepMode::Instance,
            attributes: &[
                VertexAttribute {
                    format: VertexFormat::Uint32,
                    offset: 0,
                    shader_location: 10,  // High location to avoid conflicts
                },
            ],
        },
    ]
}
```

---

## Phase 8: Testing and Validation

**Estimated Time:** 4-6 hours

### Test Cases

1. **Single Draw**
   - Render 1 triangle
   - Verify 1 model matrix uploaded
   - Visual: Matches old renderer

2. **Multiple Draws, Same Transform**
   - Render 100 triangles with identity transform
   - Verify model matrices deduplicated (or 100 entries if no dedup)
   - Visual: All render correctly

3. **Stress Test (1000+ draws)**
   - Render 10,000 triangles with unique transforms
   - Verify no crashes or glitches
   - Measure memory usage (should be ~64KB for matrices vs 640KB before)

4. **Animated Transforms**
   - Rotate/translate/scale objects each frame
   - Verify matrix pool grows and resets correctly
   - Visual: Smooth animation

5. **Retained Meshes**
   - Load mesh in init(), draw in render()
   - Verify mesh transforms work correctly
   - Visual: Mesh renders at correct position

6. **Mixed Immediate + Retained**
   - Draw both immediate triangles and retained meshes
   - Verify both use matrix packing correctly
   - Visual: Both render correctly

### Validation Checklist

- [ ] Visual: All test cases match old renderer pixel-for-pixel
- [ ] Performance: Measure draw call overhead (should decrease)
- [ ] Memory: Matrix pool size is reasonable (<100KB typical game)
- [ ] Capacity: Warn/handle gracefully at 65,536 model matrices
- [ ] Cross-platform: Test on WebGPU, native, WebGL (if supported)

### Performance Metrics to Track

```rust
// Add instrumentation
tracing::debug!(
    "Matrix upload: {} model, {} view, {} proj ({} KB total)",
    model_count,
    view_count,
    proj_count,
    total_kb
);
```

**Expected improvements:**
- Command size: ~120 bytes → ~60 bytes (50% reduction)
- Matrix uploads: Per-draw → Once per frame
- Memory bandwidth: Reduced by ~16× for transforms

---

## Rollout Strategy

### 1. Feature Flag (Optional)

Add a runtime flag to switch between old and new systems during transition:

```rust
pub struct ZGraphics {
    use_matrix_packing: bool,  // Toggle during testing
}
```

### 2. Incremental Deployment

1. **Day 1-2:** Implement phases 1-3 (infrastructure + FFI)
2. **Day 3:** Implement phases 4-6 (GPU upload + shaders)
3. **Day 4:** Implement phase 7 (instance buffer)
4. **Day 5:** Testing and validation

### 3. Fallback Plan

If issues arise:
- Keep old `VRPCommand` structure with `transform: Mat4`
- Add new field `mvp_index: Option<MvpIndex>`
- Gradually migrate FFI calls to use packed indices
- Remove old field once fully migrated

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| WebGL storage buffer incompatibility | Medium | High | Use uniform buffer with smaller capacity (256 models) |
| Instance buffer overhead | Low | Low | 4 bytes per draw is negligible |
| Matrix pool overflow (>65,536) | Low | Medium | Warn and cap, unlikely in fantasy console |
| Shader unpacking bugs | Medium | High | Thorough testing, visual validation |
| Performance regression | Low | Medium | Benchmark before/after, profile bottlenecks |

---

## Success Criteria

- ✅ All visual tests pass (pixel-perfect match)
- ✅ Memory usage reduced by ~50% for commands
- ✅ Matrix uploads reduced to once per frame
- ✅ No performance regressions
- ✅ No crashes or rendering glitches
- ✅ Handles 10,000+ draws without issues

---

## Follow-Up Work

After completion:
1. **Unified Shading State** - Next refactor (see separate plan)
2. **WebGL fallback** - If storage buffers unsupported, use uniform buffers
3. **Matrix deduplication** - Optional: Hash and dedupe identical transforms
4. **Profiling** - Measure actual performance gains in real games

---

**Last Updated:** December 2024
**Status:** Ready for implementation
