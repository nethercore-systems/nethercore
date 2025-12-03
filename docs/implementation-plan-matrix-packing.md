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
- Cross-backend compatible

**Approach:** Push constants for matrix indices (4 bytes per draw, reserves space for future unified shading state index)

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

impl Default for ZFFIState {
    fn default() -> Self {
        let mut view_matrices = Vec::with_capacity(4);
        let mut proj_matrices = Vec::with_capacity(4);

        // Default view: camera at (0, 0, 5) looking at origin
        view_matrices.push(Mat4::look_at_rh(Vec3::new(0.0, 0.0, 5.0), Vec3::ZERO, Vec3::Y));

        // Default projection: 60° FOV, 16:9 aspect
        proj_matrices.push(Mat4::perspective_rh(60f32.to_radians(), 16.0/9.0, 0.1, 1000.0));

        Self {
            // Existing initialization...

            model_matrices: Vec::with_capacity(256),
            view_matrices,
            proj_matrices,
            current_model_idx: 0,
            current_view_idx: 0,
            current_proj_idx: 0,
        }
    }
}

impl ZFFIState {
    /// Add a model matrix to the pool and return its index
    pub fn add_model_matrix(&mut self, matrix: Mat4) -> Option<u32> {
        let idx = self.model_matrices.len() as u32;
        if idx >= 65536 {
            // Panic in all builds - this is a programming error
            panic!("Model matrix pool overflow! Maximum 65,536 matrices per frame.");
        }
        self.model_matrices.push(matrix);
        Some(idx)
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
    pub fn clear_frame(&mut self) {
        self.render_pass.reset();
        self.model_matrices.clear();
        // View and proj typically have defaults and persist, but we keep them
        // They will be repopulated by camera_set or push_*_matrix calls
        self.deferred_commands.clear();
        self.pending_textures.clear();
        self.pending_meshes.clear();
        self.audio_commands.clear();
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

    /// Ensure view matrix buffer has sufficient capacity
    fn ensure_view_buffer_capacity(&mut self, count: usize) {
        if count <= self.view_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!("Growing view matrix buffer: {} → {}", self.view_matrix_capacity, new_capacity);

        self.view_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("View Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.view_matrix_capacity = new_capacity;
    }

    /// Ensure projection matrix buffer has sufficient capacity
    fn ensure_proj_buffer_capacity(&mut self, count: usize) {
        if count <= self.proj_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!("Growing projection matrix buffer: {} → {}", self.proj_matrix_capacity, new_capacity);

        self.proj_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.proj_matrix_capacity = new_capacity;
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
    pub mvp_index: MvpIndex,         // New: 4 bytes (packed indices)
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

    // Update view matrix pool (always index 0 for convenience)
    if state.view_matrices.is_empty() {
        state.view_matrices.push(view);
    } else {
        state.view_matrices[0] = view;
    }
    state.current_view_idx = 0;

    // Build projection matrix
    let aspect = state.viewport_width as f32 / state.viewport_height as f32;
    let proj = Mat4::perspective_rh(
        fov.to_radians(),
        aspect,
        near,
        far,
    );

    // Update projection matrix pool (always index 0 for convenience)
    if state.proj_matrices.is_empty() {
        state.proj_matrices.push(proj);
    } else {
        state.proj_matrices[0] = proj;
    }
    state.current_proj_idx = 0;
}
```

#### 3.2: Add Advanced Matrix Functions (NEW)

**For advanced users who want direct control over view/projection matrices:**

```rust
/// Push a custom view matrix to the pool, returning its index
///
/// For advanced rendering techniques (multiple cameras, render-to-texture, etc.)
/// Most users should use camera_set() instead.
fn push_view_matrix(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    m0: f32, m1: f32, m2: f32, m3: f32,
    m4: f32, m5: f32, m6: f32, m7: f32,
    m8: f32, m9: f32, m10: f32, m11: f32,
    m12: f32, m13: f32, m14: f32, m15: f32,
) -> u32 {
    let state = &mut caller.data_mut().console;

    let matrix = Mat4::from_cols_array(&[
        m0, m1, m2, m3,
        m4, m5, m6, m7,
        m8, m9, m10, m11,
        m12, m13, m14, m15,
    ]);

    let idx = state.view_matrices.len() as u32;
    if idx >= 256 {
        panic!("View matrix pool overflow! Maximum 256 view matrices per frame.");
    }

    state.view_matrices.push(matrix);
    state.current_view_idx = idx;
    idx
}

/// Push a custom projection matrix to the pool, returning its index
///
/// For advanced rendering techniques (custom projections, orthographic, etc.)
/// Most users should use camera_set() instead.
fn push_projection_matrix(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    m0: f32, m1: f32, m2: f32, m3: f32,
    m4: f32, m5: f32, m6: f32, m7: f32,
    m8: f32, m9: f32, m10: f32, m11: f32,
    m12: f32, m13: f32, m14: f32, m15: f32,
) -> u32 {
    let state = &mut caller.data_mut().console;

    let matrix = Mat4::from_cols_array(&[
        m0, m1, m2, m3,
        m4, m5, m6, m7,
        m8, m9, m10, m11,
        m12, m13, m14, m15,
    ]);

    let idx = state.proj_matrices.len() as u32;
    if idx >= 256 {
        panic!("Projection matrix pool overflow! Maximum 256 projection matrices per frame.");
    }

    state.proj_matrices.push(matrix);
    state.current_proj_idx = idx;
    idx
}
```

**Register new functions:**

```rust
pub fn register_z_ffi(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    // ... existing registrations

    // Advanced matrix functions
    linker.func_wrap("env", "push_view_matrix", push_view_matrix)?;
    linker.func_wrap("env", "push_projection_matrix", push_projection_matrix)?;

    // ... rest
}
```

#### 3.3: Update Draw Commands

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
    let model_idx = state.add_model_matrix(state.current_transform)
        .expect("Model matrix pool overflow");

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
        self.ensure_view_buffer_capacity(z_state.view_matrices.len());
        let data = bytemuck::cast_slice(&z_state.view_matrices);
        self.queue.write_buffer(&self.view_matrix_buffer, 0, data);
    }

    // 3. Upload projection matrices
    if !z_state.proj_matrices.is_empty() {
        self.ensure_proj_buffer_capacity(z_state.proj_matrices.len());
        let data = bytemuck::cast_slice(&z_state.proj_matrices);
        self.queue.write_buffer(&self.proj_matrix_buffer, 0, data);
    }

    // 4. Upload immediate vertex/index data
    // ... (existing code)

    // 5. Sort and execute commands
    // ... (existing code)
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

#### 5.1: Replace Uniforms with Storage Buffers

```wgsl
// OLD: Single uniform matrices per draw
// @group(0) @binding(1) var<uniform> view: mat4x4<f32>;
// @group(0) @binding(2) var<uniform> projection: mat4x4<f32>;

// NEW: All matrices in storage buffers
@group(0) @binding(0) var<storage, read> model_matrices: array<mat4x4<f32>>;
@group(0) @binding(1) var<storage, read> view_matrices: array<mat4x4<f32>>;
@group(0) @binding(2) var<storage, read> proj_matrices: array<mat4x4<f32>>;
```

#### 5.2: Add Push Constants

```wgsl
/// Per-draw push constants (16 bytes)
/// Contains indices into the matrix storage buffers
struct PushConstants {
    model_index: u32,
    view_index: u32,
    proj_index: u32,
    shading_state_index: u32,  // Reserved for unified shading state (Phase 2)
}

var<push_constant> pc: PushConstants;
```

#### 5.3: Update Vertex Shader

```wgsl
struct VertexIn {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    //VIN_NORMAL
    //VIN_SKINNED
}

@vertex
fn vs(in: VertexIn) -> VertexOut {
    var out: VertexOut;

    //VS_SKINNED

    // Fetch matrices using push constant indices
    let model = model_matrices[pc.model_index];
    let view = view_matrices[pc.view_index];
    let proj = proj_matrices[pc.proj_index];

    // Apply transforms
    //VS_POSITION
    let model_pos = model * world_pos;
    out.world_position = model_pos.xyz;

    // View-projection transform
    out.clip_position = proj * view * model_pos;

    //VS_UV
    //VS_COLOR
    //VS_NORMAL

    return out;
}
```

**Note:** Update ALL 4 shader templates (mode0, mode1, mode2, mode3) with these changes.

---

## Phase 6: Update Bind Group Layouts and Push Constants

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

#### 6.2: Add Push Constant Range to Pipeline Layout

**File:** `emberware-z/src/graphics/pipeline.rs`

```rust
let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
    label: Some("Render Pipeline Layout"),
    bind_group_layouts: &[&bind_group_layout_0, &bind_group_layout_1],
    push_constant_ranges: &[
        PushConstantRange {
            stages: ShaderStages::VERTEX,
            range: 0..16,  // 4 × u32 = 16 bytes (model, view, proj, shading indices)
        }
    ],
});
```

#### 6.3: Create Bind Group with Matrix Buffers

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

#### 6.4: Set Push Constants Per Draw

**File:** `emberware-z/src/graphics/mod.rs` (in render pass loop)

```rust
for cmd in self.command_buffer.commands() {
    // ... pipeline and bind group setup

    // Set push constants with matrix indices
    let (model_idx, view_idx, proj_idx) = cmd.mvp_index.unpack();
    let push_constants = [
        model_idx,
        view_idx,
        proj_idx,
        0u32,  // Reserved for shading_state_index (unified shading state refactor)
    ];
    render_pass.set_push_constants(
        wgpu::ShaderStages::VERTEX,
        0,
        bytemuck::cast_slice(&push_constants),
    );

    // Set vertex buffer
    let vertex_buffer = match cmd.buffer_source {
        BufferSource::Immediate => self.buffer_manager.vertex_buffer(cmd.format),
        BufferSource::Retained => self.buffer_manager.retained_vertex_buffer(cmd.format),
    };
    render_pass.set_vertex_buffer(0, vertex_buffer.slice(..));

    // Draw
    if cmd.index_count > 0 {
        let index_buffer = match cmd.buffer_source {
            BufferSource::Immediate => self.buffer_manager.index_buffer(cmd.format),
            BufferSource::Retained => self.buffer_manager.retained_index_buffer(cmd.format),
        };
        if let Some(buffer) = index_buffer.buffer() {
            render_pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
            render_pass.draw_indexed(
                cmd.first_index..cmd.first_index + cmd.index_count,
                cmd.base_vertex as i32,
                0..1,  // Single instance
            );
        }
    } else {
        render_pass.draw(
            cmd.base_vertex..cmd.base_vertex + cmd.vertex_count,
            0..1,  // Single instance
        );
    }
}
```

---

## Phase 7: Testing and Validation

**Estimated Time:** 4-6 hours

### Test Cases

1. **Single Draw**
   - Render 1 triangle
   - Verify 1 model matrix uploaded
   - Visual: Matches old renderer

2. **Multiple Draws, Same Transform**
   - Render 100 triangles with identity transform
   - Verify 100 model matrices (no deduplication yet)
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

7. **Multiple Cameras (Advanced)**
   - Use `push_view_matrix()` and `push_projection_matrix()`
   - Verify multiple views per frame work
   - Visual: Correct rendering with different cameras

### Validation Checklist

- [ ] Visual: All test cases match old renderer pixel-for-pixel
- [ ] Performance: Measure draw call overhead (should decrease)
- [ ] Memory: VRPCommand size reduced (64 bytes saved per command)
- [ ] Capacity: Panic gracefully at 65,536 model matrices
- [ ] Push constants: Verify 16-byte limit is sufficient

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
- VRPCommand size: ~120 bytes → ~60 bytes (50% reduction)
- Matrix uploads: Per-draw → Once per frame
- Memory bandwidth: Reduced by ~16× for transforms

---

## Rollout Strategy

### 1. Incremental Deployment

1. **Day 1-2:** Implement phases 1-3 (infrastructure + FFI)
2. **Day 3:** Implement phases 4-5 (GPU upload + shaders)
3. **Day 4:** Implement phase 6 (push constants + bind groups)
4. **Day 5:** Testing and validation

### 2. Breaking Changes

This refactor includes breaking changes:
- Shader binding layout changes (bindings 0-2)
- Pipeline layout changes (push constants)
- VRPCommand structure changes

**Impact:** All pipelines must be regenerated. This is acceptable pre-release.

---

## Risk Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Push constant size limit | Low | Medium | 16 bytes well within 128-byte minimum guarantee |
| Storage buffer bugs | Medium | High | Thorough testing, visual validation |
| Matrix pool overflow | Low | Low | Panic on overflow (programming error) |
| Performance regression | Low | Medium | Benchmark before/after, profile bottlenecks |

---

## Success Criteria

- ✅ All visual tests pass (pixel-perfect match)
- ✅ VRPCommand memory reduced by ~50%
- ✅ Matrix uploads reduced to once per frame
- ✅ No performance regressions
- ✅ No crashes or rendering glitches
- ✅ Handles 10,000+ draws without issues
- ✅ Advanced matrix functions work (push_view_matrix, push_projection_matrix)

---

## Follow-Up Work

After completion:
1. **Unified Shading State** - Use 4th push constant slot for shading state index
2. **WebGL fallback** - TODO: If storage buffers unsupported, use uniform buffers with capacity limits
3. **Matrix deduplication** - Optional: Hash and dedupe identical transforms
4. **Profiling** - Measure actual performance gains in real games

---

## Integration with Unified Shading State

This refactor reserves the 4th u32 in push constants for the unified shading state index:

```wgsl
struct PushConstants {
    model_index: u32,
    view_index: u32,
    proj_index: u32,
    shading_state_index: u32,  // ← Used by unified shading state refactor
}
```

When implementing unified shading state:
- Update push constant write to include `shading_state_index`
- Shader reads from `shading_states[pc.shading_state_index]`
- Total push constant size remains 16 bytes

---

**Last Updated:** December 2024
**Status:** Ready for implementation
