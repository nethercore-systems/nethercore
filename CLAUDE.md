# Emberware - Claude Code Instructions

## Project Overview

Emberware is a 5th-generation fantasy console platform designed to support multiple fantasy consoles with shared rollback netcode infrastructure.

- **Core** — Shared console framework (WASM runtime, GGRS rollback, game loop)
- **Emberware Z** — PS1/N64 aesthetic fantasy console (first implementation)
- **Shared** — API types shared with the platform backend
- **Docs** — FFI documentation for game developers
- **Examples** — Example games

See [TASKS.md](./TASKS.md) for current development status and implementation plan.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    emberware-z                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ ZGraphics   │  │ ZAudio      │  │ Z-specific FFI  │ │
│  │ (wgpu)      │  │ (rodio)     │  │ (draw_*, etc)   │ │
│  └──────┬──────┘  └──────┬──────┘  └────────┬────────┘ │
│         │                │                   │          │
│         └────────────────┼───────────────────┘          │
│                          │ implements Console trait     │
├──────────────────────────┼──────────────────────────────┤
│                    emberware-core                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │ Console     │  │ Runtime<C>  │  │ Common FFI      │ │
│  │ trait       │  │ game loop   │  │ (input, save,   │ │
│  │             │  │ GGRS        │  │  random, etc)   │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
│  ┌─────────────┐  ┌─────────────┐                      │
│  │ WasmEngine  │  │ Rollback    │                      │
│  │ (wasmtime)  │  │ state mgmt  │                      │
│  └─────────────┘  └─────────────┘                      │
└─────────────────────────────────────────────────────────┘
```

### Console Trait

Each fantasy console implements the `Console` trait:

```rust
pub trait Console: Send + 'static {
    type Graphics: Graphics;
    type Audio: Audio;
    type Input: ConsoleInput;  // Console-specific input layout

    fn name(&self) -> &'static str;
    fn specs(&self) -> &ConsoleSpecs;
    fn register_ffi(&self, linker: &mut Linker<GameState>) -> Result<()>;
    fn create_graphics(&self, window: Arc<Window>) -> Result<Self::Graphics>;
    fn create_audio(&self) -> Result<Self::Audio>;
    fn map_input(&self, raw: &RawInput) -> Self::Input;
}

// Must be POD for GGRS serialization
pub trait ConsoleInput: Clone + Copy + Default + bytemuck::Pod + bytemuck::Zeroable {}
```

This allows:
- Shared WASM execution, rollback netcode, and game loop
- Console-specific rendering, audio, FFI functions, and input layouts
- Easy addition of future consoles (Emberware Y, X, etc.)

### Input Abstraction

Each console defines its own input struct:

```rust
// Emberware Z (PS2/Xbox style)
#[repr(C)]
pub struct ZInput {
    pub buttons: u16,        // D-pad + face + shoulders + start/select
    pub left_stick_x: i8,    // -128..127
    pub left_stick_y: i8,
    pub right_stick_x: i8,
    pub right_stick_y: i8,
    pub left_trigger: u8,    // 0..255 analog
    pub right_trigger: u8,
}

// Emberware Classic (6-button retro)
#[repr(C)]
pub struct ClassicInput {
    pub buttons: u16,  // D-pad + A/B/C/X/Y/Z + L/R + start/select
}
```

The core handles GGRS serialization of whatever input type the console uses.

## Tech Stack

### Core
- wasmtime (WASM execution)
- GGRS (rollback netcode)
- matchbox_socket (WebRTC P2P networking)
- winit (windowing)

### Emberware Z
- wgpu (graphics with PS1/N64 aesthetic)
- glam (math: vectors, matrices, quaternions)
- rodio (audio)
- egui (library UI)
- reqwest (ROM downloads)

### Shared
- serde for serialization

## Project Structure

- `/core` — `emberware-core` crate with Console trait, WASM runtime, GGRS integration
- `/emberware-z` — `emberware-z` binary implementing Console for PS1/N64 aesthetic
- `/shared` — `emberware-shared` crate with API types
- `/docs/ffi.md` — FFI reference for game developers
- `/examples/hello-world` — Minimal example game

## Conventions

### FFI Functions
- No prefix (e.g., `clear`, `draw_triangle`)
- Use C ABI: `extern "C" fn`

### Game Lifecycle
Games export: `init()`, `update()`, `render()`

- `init()` — Called once at startup
- `update()` — Called every tick (deterministic, used for rollback)
- `render()` — Called every frame (skipped during rollback replay)

### Rollback Netcode (GGRS)
The console uses GGRS for deterministic rollback netcode. This means:
- `update()` MUST be deterministic (same inputs → same state)
- Game state must be serializable for save/load during rollback
- No external randomness — use seeded RNG from host
- Tick rate is separate from frame rate (update can run multiple times per frame during catchup)

### Math Conventions
- **glam** for all math (vectors, matrices, quaternions)
- **Column-major** matrix storage (compatible with WGSL/wgpu)
- **Column vectors**: `v' = M * v`
- **Y-up**, right-handed coordinate system
- FFI angles in **degrees** (convert to radians internally)
- `transform_set()` takes 16 floats in column-major order: `[col0, col1, col2, col3]`

### Resource Management
- All graphics resources (textures, palettes, tilemaps) created in `init()`
- No `*_free` functions — resources auto-cleaned on game shutdown
- Vertex buffers: one buffer per stride, grows dynamically during init
- Immediate-mode draws buffered on CPU, flushed once per frame

### Rendering Architecture

#### Shader Generation System

Shaders are generated from templates using flag-based permutations. Each flag combination produces a unique shader variant with only the needed vertex attributes and fragment operations.

```rust
// Vertex format flags (bitmask)
pub mod flags {
    pub const VERTEX_COLOR: u8 = 1;  // Include per-vertex color
    pub const UV_TEXTURE: u8 = 2;    // Include UV coordinates
    pub const CUBEMAP: u8 = 4;       // Include cubemap reflection (implies NORMAL)
    pub const MATCAP: u8 = 8;        // Include matcap sampling (implies NORMAL)
    // NORMAL is derived: has_normal = has_cubemap || has_matcap
}
```

The shader generator replaces placeholders in a template:
```rust
pub fn generate_shader(template: &str, flags: u8) -> String {
    let has_color = flags & flags::VERTEX_COLOR != 0;
    let has_uv = flags & flags::UV_TEXTURE != 0;
    let has_cubemap = flags & flags::CUBEMAP != 0;
    let has_matcap = flags & flags::MATCAP != 0;
    let has_normal = has_cubemap || has_matcap;

    // Replace placeholders like //VIN_COLOR, //VS_COLOR, //FS_COLOR
    // with actual WGSL code or empty string
}
```

Template placeholders:
- `//VIN_*` - Vertex input attributes
- `//VOUT_*` - Vertex output (to fragment)
- `//VS_*` - Vertex shader code
- `//FS_*` - Fragment shader code

#### Pipeline Entry Structure

Each vertex format gets its own pipeline with dedicated buffers:

```rust
pub struct RenderPipelineEntry {
    pub pipeline: wgpu::RenderPipeline,
    pub bind_group_layout: wgpu::BindGroupLayout,
    pub vertex_stride: u64,           // Bytes per vertex for this format

    vertex_buffer: GrowableBuffer,    // Auto-growing GPU buffer
    index_buffer: GrowableBuffer,     // Auto-growing GPU buffer
    vertex_count: u32,                // For base_vertex in add_mesh
    index_count: u32,                 // For index_start tracking
}
```

#### Texture Slots

Fixed layout per render mode (set in `init()`):

| Mode | Slot 0 | Slot 1 | Slot 2 | Slot 3 |
|------|--------|--------|--------|--------|
| 0 (Unlit) | Albedo (UV) | — | — | — |
| 1 (Matcap) | Albedo (UV) | Matcap (N) | Matcap (N) | Matcap (N) |
| 2 (PBR) | Albedo (UV) | MRE (UV) | Reflection (N) | — |
| 3 (Hybrid) | Albedo (UV) | MRE (UV) | Reflection (N) | Matcap (N) |

**(N) = Normal-sampled, (UV) = UV-sampled**

Fallbacks:
- UV slots without UVs/texture → `set_color()` / `material_*()` uniforms
- Normal slots without texture → ambient color
- Modes 1-3 without normals → behaves like Mode 0
- Missing required textures → 8×8 magenta/black checkerboard (debug visibility)

#### Vertex Color Handling

- If per-vertex color exists in vertex data → use it
- If not → use uniform color (set via `set_color()`)
- Both can be combined (uniform tints per-vertex color)

#### Vertex Formats

Vertices are packed `[f32]` arrays. Color is RGB32f (3 floats). Format is a 3-bit bitmask:

```rust
const FORMAT_UV: u32 = 1;      // Has UV coordinates
const FORMAT_COLOR: u32 = 2;   // Has per-vertex color (RGB, 3 floats)
const FORMAT_NORMAL: u32 = 4;  // Has normals
```

All 8 combinations:

| Format | Value | Components | Stride |
|--------|-------|------------|--------|
| POS | 0 | pos(3) | 12 bytes |
| POS_UV | 1 | pos(3) + uv(2) | 20 bytes |
| POS_COLOR | 2 | pos(3) + color(3) | 24 bytes |
| POS_UV_COLOR | 3 | pos(3) + uv(2) + color(3) | 32 bytes |
| POS_NORMAL | 4 | pos(3) + normal(3) | 24 bytes |
| POS_UV_NORMAL | 5 | pos(3) + uv(2) + normal(3) | 32 bytes |
| POS_COLOR_NORMAL | 6 | pos(3) + color(3) + normal(3) | 36 bytes |
| POS_UV_COLOR_NORMAL | 7 | pos(3) + uv(2) + color(3) + normal(3) | 44 bytes |

#### GPU Skinning

Emberware Z supports GPU-based skeletal animation. Developers animate bones on CPU (update transforms each frame), and the GPU handles skinning (vertex deformation based on bone weights).

**Skinned vertex format flag:**
```rust
const FORMAT_SKINNED: u32 = 8;  // Has bone indices (4 u8) + bone weights (4 f32)
```

When FORMAT_SKINNED is set, each vertex includes:
- `bone_indices`: 4 × u8 (4 bytes) — which bones affect this vertex
- `bone_weights`: 4 × f32 (16 bytes) — weight of each bone's influence

This adds 20 bytes to the vertex stride. Can combine with other flags (e.g., FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED).

**Bone transform upload:**
```rust
fn set_bones(matrices: *const f32, count: u32)  // 16 floats per bone (4x4 matrix)
```

Called before `draw_mesh()` or `draw_triangles()` to set the current bone transforms (max 256 bones). The vertex shader multiplies position/normal by the weighted sum of bone matrices.

**Workflow:**
1. In `init()`: Load skinned mesh with bone indices/weights baked into vertices
2. Each `update()`: Animate skeleton on CPU (update bone transforms)
3. Each `render()`: Call `set_bones()` then `draw_mesh()`

### Local Storage
```
~/.emberware/
├── config.toml
├── games/{game_id}/
│   ├── manifest.json
│   ├── rom.wasm
│   └── saves/
```

## Deep Links
`emberware://play/{game_id}` — Download if needed, then play

## Related
- `emberware-platform` (private) — Backend API, web frontend
