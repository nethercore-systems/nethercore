# Emberware Z Rendering Architecture

**Last Updated:** December 2024
**Status:** Current Implementation (before matrix packing/unified state refactors)

This document describes the current rendering architecture of Emberware Z, a fantasy console with PS1/N64 aesthetics. It serves as both reference documentation and a foundation for understanding upcoming architectural improvements.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Command Recording and Replay](#command-recording-and-replay)
3. [Immediate vs Retained Mode](#immediate-vs-retained-mode)
4. [Vertex Format System](#vertex-format-system)
5. [Shader Permutation System](#shader-permutation-system)
6. [Resource Management](#resource-management)
7. [Render Loop Flow](#render-loop-flow)
8. [Performance Characteristics](#performance-characteristics)
9. [Critical Files Reference](#critical-files-reference)

---

## System Overview

Emberware Z uses a **Virtual Render Pass (VRP)** architecture that decouples FFI command recording (CPU-side, WASM-facing) from GPU execution (renderer-side). This enables:

- **Deferred rendering:** Commands recorded during game's `render()`, executed later
- **Command sorting:** Minimize GPU state changes via batching
- **Hybrid mode support:** Both immediate-mode (transient) and retained-mode (persistent) geometry
- **Resource safety:** WASM can't directly access GPU resources

### Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────┐
│                        Game WASM Code                            │
│              (calls FFI: draw_triangles, draw_mesh, etc)         │
└────────────────────┬────────────────────────────────────────────┘
                     │ FFI calls
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                     ZFFIState (CPU-side)                         │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ VirtualRenderPass (command_buffer.rs)                    │   │
│  │  • Commands: Vec<VRPCommand>                             │   │
│  │  • Vertex data: [Vec<u8>; 16] (per format)              │   │
│  │  • Index data: [Vec<u16>; 16] (per format)              │   │
│  └──────────────────────────────────────────────────────────┘   │
│  • Render state: color, depth_test, cull_mode, blend_mode       │
│  • Transform: current_transform, transform_stack                 │
│  • Deferred commands: billboards, sprites, text                  │
└────────────────────┬────────────────────────────────────────────┘
                     │ Swapped & consumed each frame
                     ▼
┌─────────────────────────────────────────────────────────────────┐
│                    ZGraphics (GPU execution)                     │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ BufferManager (buffer.rs)                                │   │
│  │  • Immediate buffers: [GrowableBuffer; 16] (V+I)        │   │
│  │  • Retained buffers: [GrowableBuffer; 16] (V+I)         │   │
│  │  • Retained meshes: HashMap<u32, RetainedMesh>          │   │
│  └──────────────────────────────────────────────────────────┘   │
│  • Pipeline cache: HashMap by (mode, format, state)             │
│  • Render loop: Execute VRPCommands → GPU draw calls            │
└─────────────────────────────────────────────────────────────────┘
```

---

## Command Recording and Replay

### Recording Phase

**Location:** [emberware-z/src/ffi/mod.rs](../emberware-z/src/ffi/mod.rs)

When a WASM game calls `draw_triangles()`, `draw_mesh()`, or similar:

1. **Read data from WASM memory** - Vertex/index data copied to CPU-side buffers
2. **Capture render state** - Transform, color, textures, depth test, cull mode, blend mode
3. **Record VRPCommand** - Single command with all necessary metadata
4. **Accumulate in VirtualRenderPass** - Commands buffered until frame end

#### VRPCommand Structure

**Location:** [graphics/command_buffer.rs:26-51](../emberware-z/src/graphics/command_buffer.rs)

```rust
pub struct VRPCommand {
    pub format: u8,                              // Vertex format (0-15)
    pub transform: Mat4,                         // Model matrix (64 bytes)
    pub vertex_count: u32,
    pub index_count: u32,
    pub base_vertex: u32,                        // Offset into vertex buffer
    pub first_index: u32,                        // Offset into index buffer
    pub buffer_source: BufferSource,             // Immediate or Retained
    pub texture_slots: [TextureHandle; 4],       // Bound textures
    pub color: u32,                              // Tint color (RGBA8)
    pub depth_test: bool,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
    pub matcap_blend_modes: [MatcapBlendMode; 4],
}
```

**Note:** Each command stores a full 64-byte Mat4 transform. This is a known inefficiency addressed in future refactors (see matrix packing proposal).

### Replay Phase

**Location:** [graphics/mod.rs:1988-2636](../emberware-z/src/graphics/mod.rs)

After the game's `render()` function returns:

1. **Upload vertex/index data** - Copy from CPU buffers to GPU (per-format buffers)
2. **Sort commands** - Group by `(pipeline, texture_slots, ...)` to minimize state changes
3. **Execute render pass:**
   - Get/create pipeline from cache
   - Bind textures (only if changed since last draw)
   - Set vertex/index buffers (only if format/source changed)
   - Upload material uniforms (currently per-draw)
   - Issue GPU draw call

#### State Change Minimization

The renderer tracks bound state and skips redundant GPU calls:

```rust
let mut bound_pipeline: Option<PipelineKey> = None;
let mut bound_texture_slots: Option<[TextureHandle; 4]> = None;
let mut bound_vertex_format: Option<(u8, BufferSource)> = None;

for cmd in commands {
    if bound_pipeline != Some(pipeline_key) {
        render_pass.set_pipeline(&pipeline);
        bound_pipeline = Some(pipeline_key);
    }
    // Similar for textures, buffers, materials...
}
```

---

## Immediate vs Retained Mode

The renderer supports two geometry submission modes:

### Immediate Mode

**API:** `draw_triangles()`, `draw_sprite()`, `draw_billboard()`

**Flow:**
1. FFI copies vertex data from WASM memory
2. Data appended to `VirtualRenderPass::vertex_data[format]`
3. Command recorded with `BufferSource::Immediate`
4. At render time, uploaded to `BufferManager::vertex_buffers[format]`
5. **Buffers reset each frame** via `reset_command_buffer()`

**Use cases:**
- Dynamic/animated geometry
- Procedural generation
- Particle systems
- Debug visualizations
- UI elements

**Memory overhead:** Data copied twice (WASM → CPU buffer, CPU buffer → GPU)

### Retained Mode

**API:** `load_mesh()`, `draw_mesh()`

**Flow:**
1. `load_mesh()` uploads vertex data to GPU **once** (during `init()`)
2. Data stored in `retained_vertex_buffers[format]`
3. Returns `MeshHandle` with metadata (format, counts, offsets)
4. `draw_mesh()` records command with `BufferSource::Retained`
5. **Data persists across frames** (no re-upload)

**Use cases:**
- Static geometry (level meshes, props)
- Character models
- Reusable assets

**Memory overhead:** One-time upload, references persist

#### RetainedMesh Metadata

**Location:** [graphics/buffer.rs:246-299](../emberware-z/src/graphics/buffer.rs)

```rust
pub struct RetainedMesh {
    pub format: u8,
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertex_offset: u64,   // Byte offset in retained buffer
    pub index_offset: u64,    // Byte offset in index buffer (if indexed)
}
```

---

## Vertex Format System

**16 vertex formats** based on 4 bitflags (2^4 = 16 combinations):

### Format Flags

**Location:** [graphics/vertex.rs:6-16](../emberware-z/src/graphics/vertex.rs)

```rust
pub const FORMAT_UV: u8 = 1;        // Has UV coordinates (2 f32)
pub const FORMAT_COLOR: u8 = 2;     // Has per-vertex color (3 f32)
pub const FORMAT_NORMAL: u8 = 4;    // Has normals (3 f32)
pub const FORMAT_SKINNED: u8 = 8;   // Has bone indices/weights (4 u8 + 4 f32)
```

### All 16 Formats

| Format | Flags | Name | Stride | Components |
|--------|-------|------|--------|------------|
| 0 | `0000` | POS | 12 bytes | Position only |
| 1 | `0001` | POS_UV | 20 bytes | Position + UV |
| 2 | `0010` | POS_COLOR | 24 bytes | Position + Color |
| 3 | `0011` | POS_UV_COLOR | 32 bytes | Position + UV + Color |
| 4 | `0100` | POS_NORMAL | 24 bytes | Position + Normal |
| 5 | `0101` | POS_UV_NORMAL | 32 bytes | Position + UV + Normal |
| 6 | `0110` | POS_COLOR_NORMAL | 36 bytes | Position + Color + Normal |
| 7 | `0111` | POS_UV_COLOR_NORMAL | 44 bytes | Position + UV + Color + Normal |
| 8 | `1000` | POS_SKINNED | 32 bytes | Position + Skinning |
| 9 | `1001` | POS_UV_SKINNED | 40 bytes | Position + UV + Skinning |
| 10 | `1010` | POS_COLOR_SKINNED | 44 bytes | Position + Color + Skinning |
| 11 | `1011` | POS_UV_COLOR_SKINNED | 52 bytes | Position + UV + Color + Skinning |
| 12 | `1100` | POS_NORMAL_SKINNED | 44 bytes | Position + Normal + Skinning |
| 13 | `1101` | POS_UV_NORMAL_SKINNED | 52 bytes | Position + UV + Normal + Skinning |
| 14 | `1110` | POS_COLOR_NORMAL_SKINNED | 56 bytes | Position + Color + Normal + Skinning |
| 15 | `1111` | ALL | 64 bytes | All attributes |

### Key Design Decisions

#### One Buffer Per Format

**Rationale:** Avoid padding waste

Instead of a single unified vertex buffer with max stride (64 bytes), the system uses 16 separate buffers:

```rust
pub struct BufferManager {
    vertex_buffers: [GrowableBuffer; 16],           // Immediate mode
    index_buffers: [GrowableBuffer; 16],
    retained_vertex_buffers: [GrowableBuffer; 16],  // Retained mode
    retained_index_buffers: [GrowableBuffer; 16],
}
```

**Example:** A mesh with format 1 (POS_UV, 20 bytes stride) uses `vertex_buffers[1]`, avoiding 44 bytes of padding per vertex that would occur in a unified 64-byte buffer.

#### Attribute Layout Order

Fixed order ensures predictable shader attribute locations:

```
position (always) → uv → color → normal → bone_indices → bone_weights
```

**Shader locations:**
- Location 0: Position (vec3<f32>)
- Location 1: UV (vec2<f32>, if FORMAT_UV)
- Location 2: Color (vec3<f32>, if FORMAT_COLOR)
- Location 3: Normal (vec3<f32>, if FORMAT_NORMAL)
- Location 4: Bone indices (vec4<u8>, if FORMAT_SKINNED)
- Location 5: Bone weights (vec4<f32>, if FORMAT_SKINNED)

---

## Shader Permutation System

**40 shader variants** generated at compile time from 4 mode templates.

### Permutation Breakdown

| Mode | Name | Formats Supported | Shader Count |
|------|------|-------------------|--------------|
| 0 | Unlit / Simple Lambert | All 16 formats | 16 |
| 1 | Matcap | Formats 4-15 (requires NORMAL) | 8 |
| 2 | PBR-lite | Formats 4-15 (requires NORMAL) | 8 |
| 3 | Hybrid | Formats 4-15 (requires NORMAL) | 8 |

**Total:** 40 shaders (16 + 8 + 8 + 8)

### Shader Generation

**Location:** [src/shader_gen.rs](../emberware-z/src/shader_gen.rs)

At compile time, `build.rs` or `shader_gen.rs` generates WGSL source by:

1. Loading the WGSL template for each mode (e.g., `mode0_unlit.wgsl`)
2. Parsing vertex format flags (UV, COLOR, NORMAL, SKINNED)
3. Replacing placeholders with format-specific code:
   - `//VIN_*` → Vertex input struct fields
   - `//VOUT_*` → Vertex output struct fields
   - `//VS_*` → Vertex shader code
   - `//FS_*` → Fragment shader code
4. Validating generated code with `naga` (compile-time check)

**Example placeholder replacement:**

```wgsl
// Template
struct VertexInput {
    @location(0) position: vec3<f32>,
    //VIN_UV
    //VIN_COLOR
    //VIN_NORMAL
}

// Generated (format 5 = POS_UV_NORMAL)
struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) uv: vec2<f32>,
    @location(3) normal: vec3<f32>,
}
```

### Pipeline Caching

**Location:** [graphics/pipeline.rs](../emberware-z/src/graphics/pipeline.rs)

Pipelines are cached by a composite key:

```rust
struct PipelineKey {
    render_mode: u8,      // 0-3
    vertex_format: u8,    // 0-15
    blend_mode: u8,       // None, Alpha, Additive, Multiply
    depth_test: bool,
    cull_mode: u8,        // None, Back, Front
}
```

- Created lazily on first use
- Persists for entire session (until app exit)
- HashMap lookup on each draw (fast)

---

## Resource Management

### Handle System

**Two-level indirection:**

1. **Game handles** - Opaque `u32` values returned to WASM
2. **Graphics handles** - Internal GPU resource references (TextureHandle, MeshHandle)

**Mapping** (`app.rs:78` in `GameSession`):

```rust
pub struct GameSession {
    texture_map: HashMap<u32, TextureHandle>,  // Game → GPU
    mesh_map: HashMap<u32, MeshHandle>,        // Game → GPU
}
```

**Why?** WASM cannot directly hold GPU resource references (lifetime issues, safety). Game handles are stable across frames and independent of GPU resource lifecycle.

### Deferred Resource Loading

**Lifecycle:**

```
┌─ init() phase ────────────────────────────────────┐
│                                                    │
│  load_texture(pixels) → game_handle               │
│    ├─ Allocate game handle (next_texture_handle++)│
│    ├─ Copy pixels from WASM memory                │
│    └─ Push PendingTexture { handle, data }        │
│                                                    │
│  load_mesh(vertices) → game_handle                │
│    ├─ Allocate game handle                        │
│    ├─ Copy vertices from WASM memory              │
│    └─ Push PendingMesh { handle, data }           │
│                                                    │
└────────────────────────────────────────────────────┘
            ↓
┌─ After init() returns ────────────────────────────┐
│                                                    │
│  App::process_pending_resources()                 │
│    For each pending texture:                      │
│      ├─ Upload to GPU (TextureManager)            │
│      ├─ Get graphics_handle                       │
│      └─ Map game_handle → graphics_handle         │
│                                                    │
│    For each pending mesh:                         │
│      ├─ Upload to GPU (BufferManager)             │
│      ├─ Get graphics_handle                       │
│      └─ Map game_handle → graphics_handle         │
│                                                    │
└────────────────────────────────────────────────────┘
            ↓
┌─ render() phase ──────────────────────────────────┐
│                                                    │
│  Commands reference game handles                  │
│    ↓                                               │
│  ZGraphics translates via texture_map/mesh_map    │
│    ↓                                               │
│  Bind actual GPU textures/buffers                 │
│                                                    │
└────────────────────────────────────────────────────┘
```

**Why deferred?**
- Allows batched GPU uploads (more efficient)
- Avoids blocking WASM execution during `init()`
- FFI functions don't need device/queue references

### Texture Management

**Key Files:**
- FFI: [ffi/mod.rs:575](../emberware-z/src/ffi/mod.rs)
- GPU upload: [graphics/texture_manager.rs:124](../emberware-z/src/graphics/texture_manager.rs)
- Pending: [state.rs](../emberware-z/src/state.rs) (`PendingTexture`)

**Specs:**
- Format: `Rgba8UnormSrgb`
- Usage: `TEXTURE_BINDING | COPY_DST`
- VRAM budget: 4MB (tracked, enforced)

**Fallback textures:**
- **Checkerboard** (8×8 magenta/black) - Missing texture indicator for debugging
- **White** (1×1) - Untextured draws
- **Font atlas** - Built-in 8×8 monospace font (generated procedurally)

### Mesh Management

**Key Files:**
- FFI: [ffi/mod.rs:729](../emberware-z/src/ffi/mod.rs)
- GPU upload: [graphics/buffer.rs:247](../emberware-z/src/graphics/buffer.rs)
- Pending: [state.rs](../emberware-z/src/state.rs) (`PendingMesh`)

Meshes are stored in **per-format retained buffers**. Each of the 16 formats has its own buffer, and meshes are appended sequentially:

```
retained_vertex_buffers[5]:  [Mesh A][Mesh B][Mesh C]...
                              ↑       ↑       ↑
                           offset=0  offset=800  offset=2400
```

Metadata tracks offsets:

```rust
pub struct RetainedMesh {
    pub format: u8,
    pub vertex_count: u32,
    pub index_count: u32,
    pub vertex_offset: u64,   // Byte offset in buffer
    pub index_offset: u64,
}
```

---

## Render Loop Flow

**Frame lifecycle** (`graphics/mod.rs:2657-2721`):

### 1. Begin Frame

```rust
fn begin_frame(&mut self) {
    self.command_buffer.reset();           // Clear commands/vertex data
    self.current_transform = Mat4::IDENTITY;
    self.transform_stack.clear();

    // Acquire swapchain image
    let frame = self.surface.get_current_texture()?;
    self.current_frame = Some(frame);
}
```

### 2. Game Render Phase

WASM game's `render()` function executes, calling FFI:

- `draw_triangles()` → Record command, append vertex data
- `draw_mesh()` → Record command (references retained mesh)
- `draw_sprite()` → Push deferred command
- `draw_billboard()` → Push deferred command

Commands accumulate in `ZFFIState::render_pass`.

### 3. Process Draw Commands

```rust
fn process_draw_commands(&mut self, z_state, texture_map) {
    // Swap VirtualRenderPass from ZFFIState → ZGraphics
    std::mem::swap(&mut self.command_buffer, &mut z_state.render_pass);

    // Expand deferred commands (billboards/sprites → triangles)
    for cmd in z_state.deferred_commands.drain(..) {
        match cmd {
            DeferredCommand::DrawBillboard { .. } => {
                // Generate camera-facing quad geometry
                self.command_buffer.record_triangles_indexed(...);
            }
            DeferredCommand::DrawSprite { .. } => {
                // Generate screen-space quad
                self.command_buffer.record_triangles_indexed(...);
            }
        }
    }
}
```

### 4. Render Frame

```rust
fn render_frame(&mut self, view_matrix, projection_matrix, clear_color) {
    // 1. Upload immediate vertex/index data to GPU
    for format in 0..16 {
        let vertex_data = self.command_buffer.vertex_data(format);
        if !vertex_data.is_empty() {
            self.buffer_manager.vertex_buffer_mut(format)
                .write_at(&self.queue, 0, vertex_data);
        }
    }

    // 2. Sort commands to minimize state changes
    self.command_buffer.commands_mut().sort_unstable_by_key(|cmd| {
        (
            pipeline_key.render_mode,
            pipeline_key.vertex_format,
            pipeline_key.blend_mode,
            cmd.texture_slots[0].0,
        )
    });

    // 3. Execute render pass
    let mut render_pass = encoder.begin_render_pass(...);

    for cmd in self.command_buffer.commands() {
        // Get/create pipeline
        let pipeline = self.pipeline_cache.get_or_create(...);

        // Set pipeline (if changed)
        if bound_pipeline != Some(pipeline_key) {
            render_pass.set_pipeline(&pipeline);
        }

        // Bind textures (if changed)
        if bound_textures != Some(cmd.texture_slots) {
            render_pass.set_bind_group(1, &texture_bind_group, &[]);
        }

        // Set buffers (if format/source changed)
        // Upload material uniforms

        // Draw
        if cmd.index_count > 0 {
            render_pass.draw_indexed(...);
        } else {
            render_pass.draw(...);
        }
    }
}
```

### 5. Blit to Window

Offscreen render target → window surface via fullscreen triangle.

### 6. End Frame

```rust
fn end_frame(&mut self) {
    if let Some(frame) = self.current_frame.take() {
        frame.present();
    }
}
```

---

## Performance Characteristics

### Strengths

1. **Command sorting** - Groups draws by pipeline/texture to minimize GPU state changes
2. **Per-format buffers** - Eliminates padding waste (up to 3× memory savings for small formats)
3. **Immediate mode flexibility** - Easy to use for dynamic geometry
4. **Retained mode efficiency** - Static meshes uploaded once
5. **State change tracking** - Skips redundant GPU calls

### Identified Weaknesses

1. **Per-draw matrix storage** - Each command stores full 64-byte Mat4
   - For 10,000 draws: 640KB just for transforms
   - **Proposed fix:** Matrix index packing (4 bytes per draw)

2. **Per-draw material uploads** - Color, blend modes, sky, lights uploaded every draw
   - Redundant uploads when same material used multiple times
   - **Proposed fix:** Unified shading state with interning

3. **Large VRPCommand size** - ~120+ bytes per command
   - Transform (64) + state (32) + metadata (24)
   - Impacts cache locality and memory usage

4. **No material deduplication** - Same material properties uploaded repeatedly

---

## Critical Files Reference

### Core Rendering

| File | Purpose | Key Structures |
|------|---------|----------------|
| [graphics/mod.rs](../emberware-z/src/graphics/mod.rs) | Main graphics backend | `ZGraphics`, `render_frame()` |
| [graphics/command_buffer.rs](../emberware-z/src/graphics/command_buffer.rs) | Command recording | `VirtualRenderPass`, `VRPCommand` |
| [graphics/buffer.rs](../emberware-z/src/graphics/buffer.rs) | Buffer management | `BufferManager`, `GrowableBuffer`, `RetainedMesh` |
| [graphics/pipeline.rs](../emberware-z/src/graphics/pipeline.rs) | Pipeline caching | `PipelineCache`, `PipelineKey` |
| [graphics/render_state.rs](../emberware-z/src/graphics/render_state.rs) | Render state types | `RenderState`, `CullMode`, `BlendMode`, `SkyUniforms` |
| [graphics/vertex.rs](../emberware-z/src/graphics/vertex.rs) | Vertex format system | `FORMAT_*` constants, `vertex_stride()` |
| [graphics/texture_manager.rs](../emberware-z/src/graphics/texture_manager.rs) | Texture storage | `TextureManager`, `TextureEntry` |

### FFI and State

| File | Purpose | Key Functions |
|------|---------|---------------|
| [ffi/mod.rs](../emberware-z/src/ffi/mod.rs) | FFI functions | `draw_triangles()`, `draw_mesh()`, `load_texture()` |
| [state.rs](../emberware-z/src/state.rs) | FFI state | `ZFFIState`, `PendingTexture`, `PendingMesh` |

### Shaders

| File | Purpose | Variants |
|------|---------|----------|
| [shaders/mode0_unlit.wgsl](../emberware-z/shaders/mode0_unlit.wgsl) | Unlit/simple Lambert | 16 (all formats) |
| [shaders/mode1_matcap.wgsl](../emberware-z/shaders/mode1_matcap.wgsl) | Matcap shading | 8 (requires normals) |
| [shaders/mode2_pbr.wgsl](../emberware-z/shaders/mode2_pbr.wgsl) | PBR-lite | 8 (requires normals) |
| [shaders/mode3_hybrid.wgsl](../emberware-z/shaders/mode3_hybrid.wgsl) | Hybrid PBR+matcap | 8 (requires normals) |

---

## Future Improvements

See the implementation plan file for detailed proposals:

1. **Matrix Index Packing** - Replace 64-byte Mat4 with 4-byte packed index
2. **Unified Shading State** - Quantize and intern material state for better batching

Both improvements are documented in the plan file and ready for implementation.

---

**Maintainers:** Update this document when making architectural changes to the rendering system.
