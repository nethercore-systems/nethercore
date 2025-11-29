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

Shaders are generated from WGSL templates using placeholder replacement. Two compile-time dimensions:

1. **Render mode** (0-3) — Set once in `init()`, never changes. Each mode has its own template with different fragment shader logic.
2. **Vertex format** (0-15) — Flags determine which vertex attributes exist.

**Render mode templates (compile-time, not runtime):**

Each mode is a separate shader template because the fragment logic differs fundamentally:

| Mode | Fragment Logic | Why Separate |
|------|---------------|--------------|
| 0 (Unlit) | `albedo × vertex_color` | No lighting calculations at all |
| 1 (Matcap) | `albedo × vertex_color × matcap_blend` | Matcaps multiply together |
| 2 (PBR) | Full PBR with 4 lights | GGX specular, Fresnel, MRE texture |
| 3 (Hybrid) | PBR direct + matcap ambient | Combines both approaches |

Mode 0 skips all lighting for maximum performance. Modes 1-3 require `FORMAT_NORMAL`.

**Vertex format flags (compile-time permutations):**

```rust
const FORMAT_UV: u32 = 1;       // Has UV coordinates
const FORMAT_COLOR: u32 = 2;    // Has per-vertex color (RGB, 3 floats)
const FORMAT_NORMAL: u32 = 4;   // Has normals
const FORMAT_SKINNED: u32 = 8;  // Has bone indices/weights
```

**Shader count:**
- Mode 0: 16 permutations (all vertex formats)
- Modes 1-3: 8 permutations each (only formats with NORMAL flag)
- Total: 16 + 8 + 8 + 8 = **40 shaders**

Formats without NORMAL in modes 1-3 fall back to Mode 0 at runtime (warning logged).

**Template placeholder replacement:**

```rust
pub fn generate_shader(template: &str, mode: u8, format: u8) -> String {
    let has_uv = format & FORMAT_UV != 0;
    let has_color = format & FORMAT_COLOR != 0;
    let has_normal = format & FORMAT_NORMAL != 0;
    let has_skinned = format & FORMAT_SKINNED != 0;

    // Replace placeholders with WGSL code or empty string
    shader.replace("//VIN_UV", if has_uv { VIN_UV } else { "" });
    shader.replace("//VIN_COLOR", if has_color { VIN_COLOR } else { "" });
    shader.replace("//VIN_NORMAL", if has_normal { VIN_NORMAL } else { "" });
    shader.replace("//VIN_SKINNED", if has_skinned { VIN_SKINNED } else { "" });
    // ... same pattern for VOUT_*, VS_*, FS_*
}
```

**Placeholder categories:**
- `//VIN_*` — Vertex input struct fields
- `//VOUT_*` — Vertex output to fragment (interpolated)
- `//VS_*` — Vertex shader code (attribute passing, skinning, normal transform)
- `//FS_*` — Fragment shader code (mode-specific, see below)

**Procedural sky (shared by all modes):**

```wgsl
fn sample_sky(direction: vec3<f32>) -> vec3<f32> {
    let up_factor = direction.y * 0.5 + 0.5;
    let gradient = mix(sky.horizon_color, sky.zenith_color, up_factor);
    let sun_dot = max(0.0, dot(direction, sky.sun_direction));
    let sun = sky.sun_color * pow(sun_dot, sky.sun_sharpness);
    return gradient + sun;
}
```

**Default:** All zeros (black sky, no sun, no lighting). Call `set_sky()` in `init()` to enable lighting.

Used for: background rendering, environment reflections, ambient lighting.

**Mode-specific fragment shader logic:**

```wgsl
// Mode 0 (Unlit) — Flat or simple Lambert if normals present
fn fs_mode0(in: VertexOut) -> vec4<f32> {
    var color = get_base_color(in);  // vertex_color × uniform_color
    //FS_UV      → color *= textureSample(slot0, sampler, in.uv).rgb;
    //FS_NORMAL  → Simple Lambert: color *= sky_lambert(in.world_normal);
    return vec4(color, 1.0);
}

// Sky lambert for Mode 0 with normals (cheap directional + ambient)
fn sky_lambert(normal: vec3<f32>) -> vec3<f32> {
    let n_dot_l = max(0.0, dot(normal, sky.sun_direction));
    let direct = sky.sun_color * n_dot_l;
    let ambient = sample_sky(normal) * 0.3;
    return direct + ambient;
}

// Mode 1 (Matcap) — Matcaps in slots 1-3 multiply together
fn fs_mode1(in: VertexOut) -> vec4<f32> {
    var color = get_base_color(in);
    //FS_UV   → color *= textureSample(slot0, sampler, in.uv).rgb;
    let matcap_uv = compute_matcap_uv(in.view_normal);
    color *= textureSample(slot1, sampler, matcap_uv).rgb;
    color *= textureSample(slot2, sampler, matcap_uv).rgb;
    color *= textureSample(slot3, sampler, matcap_uv).rgb;
    return vec4(color, 1.0);
}

// Mode 2 (PBR-lite) — GGX specular, Schlick fresnel, up to 4 lights
// Reference: emberware-z/pbr-lite.wgsl
fn fs_mode2(in: VertexOut) -> vec4<f32> {
    let albedo = get_albedo(in);
    let mre = textureSample(slot1, sampler, in.uv);  // R=Metallic, G=Roughness, B=Emissive

    var final_color = vec3(0.0);
    for (var i = 0u; i < 4u; i++) {
        if (lights[i].enabled) {
            final_color += pbr_lite(
                in.world_normal, view_dir, lights[i].direction,
                albedo, mre.r, mre.g, mre.b,
                lights[i].color, sample_sky(in.world_normal) * 0.3
            );
        }
    }

    // Environment reflection: sky × env matcap (slot 2)
    let reflection_dir = reflect(-view_dir, in.world_normal);
    let env_matcap_uv = compute_matcap_uv(in.view_normal);
    let env_matcap = textureSample(slot2, sampler, env_matcap_uv).rgb;  // White if unbound
    let env_reflection = sample_sky(reflection_dir) * env_matcap * mre.r;

    return vec4(final_color + env_reflection, 1.0);
}

// Mode 3 (Hybrid) — PBR direct + matcap for ambient/reflections
fn fs_mode3(in: VertexOut) -> vec4<f32> {
    let albedo = get_albedo(in);
    let mre = textureSample(slot1, sampler, in.uv);
    let matcap_uv = compute_matcap_uv(in.view_normal);
    let env_matcap = textureSample(slot2, sampler, matcap_uv).rgb;  // Env reflection tint
    let matcap = textureSample(slot3, sampler, matcap_uv).rgb;      // Ambient/stylized

    // Single directional light (sun) for direct lighting
    let direct = pbr_lite(
        in.world_normal, view_dir, sky.sun_direction,
        albedo, mre.r, mre.g, mre.b,
        sky.sun_color, vec3(0.0)  // No ambient here, matcaps provide it
    );

    // Environment reflection: sky × env matcap (slot 2)
    let reflection_dir = reflect(-view_dir, in.world_normal);
    let env_reflection = sample_sky(reflection_dir) * env_matcap * mre.r;

    // Ambient from matcap (slot 3) × sky
    let ambient = matcap * sample_sky(in.world_normal) * albedo * (1.0 - mre.r);

    return vec4(direct + env_reflection + ambient, 1.0);
}
```

**Bind groups are identical across all shaders:**

All 4 texture slots always bound (simplifies bind group management):

```wgsl
@group(1) @binding(0) var slot0: texture_2d<f32>;  // Albedo
@group(1) @binding(1) var slot1: texture_2d<f32>;  // MRE or Matcap
@group(1) @binding(2) var slot2: texture_2d<f32>;  // Reflection or Matcap
@group(1) @binding(3) var slot3: texture_2d<f32>;  // Matcap (modes 1, 3)
```

Unused slots bound to fallback texture (8×8 magenta/black or 1×1 white depending on context).

**Key design principle:** One vertex buffer per stride. A mesh with pos+color uses a different buffer than one with pos+uv+color+normal. This avoids vertex padding waste while keeping bind groups uniform.

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
| 2 (PBR) | Albedo (UV) | MRE (UV) | Env Matcap (N) | — |
| 3 (Hybrid) | Albedo (UV) | MRE (UV) | Env Matcap (N) | Matcap (N) |

**(N) = Normal-sampled, (UV) = UV-sampled**

Slot 2 "Env Matcap" in Modes 2/3 multiplies with procedural sky reflections (defaults to white).

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

#### 2D Drawing (Screen Space)

For UI/HUD, renders in screen pixel coordinates (not affected by 3D transforms):
- `draw_sprite`, `draw_sprite_region`, `draw_sprite_ex` — textured quads
- `draw_rect` — solid color rectangles
- `draw_text` — built-in font rendering (UTF-8)

#### Billboarding (3D Sprites)

Camera-facing quads in 3D world space (uses current transform):

```rust
fn draw_billboard(w: f32, h: f32, mode: u32, color: u32)
fn draw_billboard_region(w, h, src_x, src_y, src_w, src_h, mode, color)
// mode: 1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z
```

Use for particles, foliage, sprite-based characters.

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
