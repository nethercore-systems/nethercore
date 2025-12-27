# Nethercore ZX Rendering Architecture

This document describes the rendering architecture of Nethercore ZX, covering the GPU pipeline, render modes, vertex formats, shader generation, and material system.

---

## Overview

Nethercore ZX uses a **wgpu-based** forward renderer with a command buffer pattern. The architecture separates staging (FFI state) from GPU execution:

```
┌──────────────────┐    ┌────────────────┐    ┌─────────────┐
│  FFI Functions   │───▶│   ZFFIState    │───▶│  ZGraphics  │
│  (WASM calls)    │    │   (staging)    │    │  (GPU exec) │
└──────────────────┘    └────────────────┘    └─────────────┘
```

**Key characteristics:**

- **Offscreen render target**: Game renders at fixed resolution (360p/540p/720p/1080p), then blits to window
- **Command buffer pattern**: Draw commands buffered on CPU, sorted by state, flushed to GPU once per frame
- **One vertex buffer per stride**: Avoids padding waste, efficient GPU memory usage
- **40 shader permutations**: Pregenerated at compile time, validated with naga
- **Per-draw shading state**: 96-byte packed state with deduplication

---

## Coordinate System

Nethercore ZX uses wgpu's standard coordinate conventions throughout the rendering pipeline.

### Screen Space

2D drawing functions use screen-space pixels:

| Property | Value |
|----------|-------|
| Resolution | 960×540 (fixed, 16:9) |
| Origin | Top-left (0, 0) |
| X-axis | 0 (left) → 960 (right) |
| Y-axis | 0 (top) → 540 (bottom) |

**Screen to NDC transformation** (in `quad_template.wgsl`):

```wgsl
let ndc_x = (screen_pos.x / screen_dims.x) * 2.0 - 1.0;  // [0, 960] → [-1, 1]
let ndc_y = 1.0 - (screen_pos.y / screen_dims.y) * 2.0;  // [0, 540] → [1, -1] (Y flip)
```

The Y-flip (`1.0 - ...`) converts screen space (Y-down) to NDC space (Y-up).

### World Space (3D)

3D rendering uses a right-handed, Y-up coordinate system:

| Property | Value |
|----------|-------|
| Coordinate system | Right-handed, Y-up |
| +X | Right |
| +Y | Up |
| +Z | Toward viewer (out of screen) |

**Camera setup** (in `camera.rs`):

```rust
let view = Mat4::look_at_rh(position, target, Vec3::Y);  // Right-handed
let proj = Mat4::perspective_rh(fov, aspect, near, far); // Right-handed
```

### NDC (Normalized Device Coordinates)

wgpu uses the following NDC conventions:

| Axis | Range |
|------|-------|
| X | -1.0 (left) to +1.0 (right) |
| Y | -1.0 (bottom) to +1.0 (top) |
| Z | 0.0 (near) to 1.0 (far) |

This is the "Vulkan" NDC convention (Z: 0→1) rather than OpenGL's (-1→1).

### Texture Coordinates (UV)

| Property | Value |
|----------|-------|
| Origin | Top-left (0, 0) |
| U | 0 (left) → 1 (right) |
| V | 0 (top) → 1 (bottom) |

**UV handling in shaders:**

- Screen-space sprites: UVs used as-is (Y already flipped in NDC calculation)
- World-space/billboards: V is flipped (`1.0 - v`) to match image file convention

### Matrix Storage

All matrices use **column-major** order (glam/WGSL standard):

| Format | Layout | Usage |
|--------|--------|-------|
| 4×4 (16 floats) | `[col0.xyzw, col1.xyzw, col2.xyzw, col3.xyzw]` | View, projection, model |
| 3×4 (12 floats) | `[col0.xyz, col1.xyz, col2.xyz, col3.xyz]` | Bone matrices (implicit row `[0,0,0,1]`) |

**Transform order:** `clip_pos = projection × view × model × vertex_pos`

### Projection Defaults

| Property | Value |
|----------|-------|
| Type | Perspective (right-handed) |
| Default FOV | 60° (vertical) |
| Aspect ratio | 16:9 (960÷540) |
| Near plane | 0.1 units |
| Far plane | 1000 units |

---

## 2D Layer Ordering

Nethercore ZX uses **CPU-side layer sorting** for 2D elements (sprites, text, UI), following industry standards (Unity, Unreal, Bevy).

### Layer System

```rust
fn layer(n: u32)  // Set current layer (0 = back, higher = front)
```

**Key concepts:**

- **Layer 0** = Background (default, resets each frame)
- **Higher layers** = Render on top (layer 2 appears above layer 1)
- **No depth testing for 2D**: Screen-space quads rely entirely on layer sorting
- **Separate from 3D depth**: Billboards and 3D meshes use depth buffer, not layers

### Render Order

Commands are sorted by layer (ascending), so rendering happens in this order:

```
Layer 0 (background) → Layer 1 → Layer 2 → ... (foreground)
```

**Example:**
```rust
// Background sprite
layer(0);
draw_sprite(bg_x, bg_y, bg_w, bg_h, bg_color);

// Character (layer 1)
layer(1);
draw_sprite(char_x, char_y, char_w, char_h, char_color);

// UI text (layer 2 - on top)
layer(2);
draw_text("Score: 100", 10.0, 10.0, 16.0, WHITE);
```

### Batching Within Layers

Quads at the **same layer** are batched by texture for performance:

- Same layer + same texture → Single draw call
- Same layer + different textures → Multiple draw calls (sorted by texture ID)
- Different layers → **Never batched** (ensures correct ordering)

### 3D vs 2D Rendering

| Type | Ordering Method | Depth Test | Depth Write | Layer Used |
|------|----------------|------------|-------------|------------|
| Screen-space quads (2D) | Layer sorting (CPU) | Always pass | Enabled (0.0) | Current layer |
| World-space quads (billboards) | Depth buffer (GPU) | Enabled | Enabled | 0 (fixed) |
| 3D meshes | Depth buffer (GPU) | Enabled | Enabled | 0 (fixed) |
| Sky | Depth buffer (GPU) | GreaterOrEqual | Disabled | 0 (fixed) |

**Early-Z Optimization:**

Screen-space quads (2D UI) render **first** with depth writes enabled at depth=0.0 (near plane). This allows:
- 3D geometry behind opaque UI elements to be culled via early depth testing
- Significant fragment shader savings (e.g., 15% fewer invocations if UI covers 15% of screen)
- Transparent dithered pixels use `discard`, preventing depth writes and allowing 3D to show through

**Render order:**
1. **2D UI** (render_type=0): Sorted by layers, all render at depth=0.0
2. **3D Meshes** (render_type=1): Culled where 2D wrote depth
3. **Sky** (render_type=2): Only renders where depth==1.0 (background fill)

**Design rationale:**

- Layer sorting is deterministic and matches game dev expectations
- Early-z reduces fragment shader cost for 3D behind UI
- Sky renders last to avoid wasting shader invocations on covered pixels
- Dithering enables order-independent transparency without alpha blending

---

## Rendering Pipeline

### Frame Lifecycle

```
begin_frame()
    │
    ▼
┌────────────────────────────────────────────────────┐
│  Game's render() function                          │
│  - FFI calls write to ZFFIState                    │
│  - Commands buffered in VirtualRenderPass          │
│  - Shading states interned for deduplication       │
└────────────────────────────────────────────────────┘
    │
    ▼
render_frame()
    │
    ├──▶ Upload vertex/index data to GPU buffers
    │
    ├──▶ Sort commands by (viewport, layer, stencil, pipeline, textures)
    │
    ├──▶ Upload transforms, shading states, animation data
    │
    ├──▶ Execute render pass on offscreen target
    │
    └──▶ Clear command buffer for next frame
    │
    ▼
blit_to_window()
    │
    └──▶ Scale render target to window (stretch/fit/pixel-perfect)
    │
    ▼
end_frame()
```

### GPU Buffer Architecture

The renderer uses a unified buffer layout to minimize binding changes:

| Binding | Buffer | Contents |
|---------|--------|----------|
| @binding(0) | unified_transforms | [models \| views \| projections] — all mat4x4 |
| @binding(1) | mvp_indices | Pre-computed absolute indices per draw |
| @binding(2) | shading_states | Array of 96-byte PackedUnifiedShadingState |
| @binding(3) | unified_animation | [inverse_bind \| keyframes \| immediate] — all mat3x4 |

**Why unified buffers?**

- Reduces storage buffer count from 9 to 4
- Single bind group for all frame data
- Cached bind group avoids wasteful GPU descriptor set creation

---

## Render Modes

Nethercore ZX supports 4 forward rendering modes, set once in `init()`:

```rust
fn render_mode(mode: u32)  // 0-3, init-only
```

| Mode | Name | Shaders | Description |
|------|------|---------|-------------|
| 0 | **Lambert** | 16 | Texture × vertex color. Simple Lambert if normals present. |
| 1 | **Matcap** | 8 | View-space normal matcap sampling. Stylized, cheap. |
| 2 | **MR-Blinn-Phong** | 8 | Metallic-roughness Blinn-Phong. Energy-conserving. |
| 3 | **Blinn-Phong** | 8 | Classic specular-shininess with rim lighting. |

### Mode 0: Lambert

The simplest mode — no lighting calculations for formats without normals.

**Without normals:**
```
final_color = texture_sample * vertex_color
```

**With normals (automatic Lambert):**
```
n_dot_l = max(0, dot(normal, sky.sun_direction))
direct = albedo * sky.sun_color * n_dot_l
ambient = albedo * sample_sky(normal) * 0.3
final_color = direct + ambient
```

**Use cases:** UI, sprites, flat-shaded retro graphics, performance-critical scenes

### Mode 1: Matcap

Adds view-space normal sampling from up to 3 matcap textures. Lighting is "baked" into the matcap texture.

```
view_normal = transform_normal_to_view_space(surface_normal)
matcap_uv = view_normal.xy * 0.5 + 0.5
final_color = albedo * vertex_color * matcap1 * matcap2 * matcap3
```

**Texture slots:**
- Slot 0: Albedo (base color)
- Slots 1-3: Matcap textures (multiply together by default)

**Blend modes per slot:**
- `0` = Multiply (default)
- `1` = Add (glow effects)
- `2` = HSV Modulate (iridescence)

**Use cases:** Stylized/toon rendering, metallic surfaces, consistent look regardless of scene lighting

### Mode 2: Metallic-Roughness Blinn-Phong

Normalized Blinn-Phong with energy conservation (Gotanda 2010). Uses metallic-roughness workflow for physically-motivated materials.

**Lighting model:**
```
// Roughness → Shininess mapping
shininess = pow(256.0, 1.0 - roughness)  // Range: 256 (smooth) → 1 (rough)

// Specular color (F0 calculation)
specular_color = mix(vec3(0.04), albedo, metallic)

// Gotanda normalization
normalization = shininess × 0.0397436 + 0.0856832

// Blinn-Phong specular
H = normalize(L + V)
spec = normalization × pow(max(0, dot(N, H)), shininess)
```

**Texture slots:**
- Slot 0: Albedo (RGB: diffuse color)
- Slot 1: MRE (R: Metallic, G: Roughness, B: Emissive)
- Slot 2: Unused

**Lights:** 4 dynamic lights + sun from procedural sky

**Use cases:** PBR-inspired materials, realistic surfaces, games needing physical material properties

### Mode 3: Specular-Shininess Blinn-Phong

Classic Blinn-Phong with explicit specular control and rim lighting. Era-authentic for PS1/N64 aesthetic.

**Features:**
- Normalized Blinn-Phong specular
- Explicit specular color (not derived from metallic)
- Rim lighting with adjustable intensity and power
- Lambert diffuse

**Texture slots:**
- Slot 0: Albedo (RGB: diffuse color)
- Slot 1: SSE (R: Specular damping, G: Shininess, B: Emissive)
- Slot 2: Specular (RGB: specular highlight color)

**Use cases:** Retro 3D aesthetic, artistic control over specular, character shaders with rim lighting

---

## Vertex Format System

### Format Flags

Vertex formats are defined by combining flags:

```rust
const FORMAT_UV: u8 = 0x01;       // Bit 0
const FORMAT_COLOR: u8 = 0x02;    // Bit 1
const FORMAT_NORMAL: u8 = 0x04;   // Bit 2
const FORMAT_SKINNED: u8 = 0x08;  // Bit 3
```

### 16 Vertex Formats

| Format | Name | Flags | Packed Stride |
|--------|------|-------|---------------|
| 0 | POS | - | 8 bytes |
| 1 | POS_UV | UV | 12 bytes |
| 2 | POS_COLOR | COLOR | 12 bytes |
| 3 | POS_UV_COLOR | UV+COLOR | 16 bytes |
| 4 | POS_NORMAL | NORMAL | 12 bytes |
| 5 | POS_UV_NORMAL | UV+NORMAL | 16 bytes |
| 6 | POS_COLOR_NORMAL | COLOR+NORMAL | 16 bytes |
| 7 | POS_UV_COLOR_NORMAL | UV+COLOR+NORMAL | 20 bytes |
| 8 | POS_SKINNED | SKINNED | 16 bytes |
| 9 | POS_UV_SKINNED | UV+SKINNED | 20 bytes |
| 10 | POS_COLOR_SKINNED | COLOR+SKINNED | 20 bytes |
| 11 | POS_UV_COLOR_SKINNED | UV+COLOR+SKINNED | 24 bytes |
| 12 | POS_NORMAL_SKINNED | NORMAL+SKINNED | 20 bytes |
| 13 | POS_UV_NORMAL_SKINNED | UV+NORMAL+SKINNED | 24 bytes |
| 14 | POS_COLOR_NORMAL_SKINNED | COLOR+NORMAL+SKINNED | 24 bytes |
| 15 | POS_UV_COLOR_NORMAL_SKINNED | ALL | 28 bytes |

### Packed Attribute Layout

GPU uses packed formats for memory efficiency:

| Attribute | Location | Format | Size |
|-----------|----------|--------|------|
| Position | 0 | Float16x4 | 8 bytes |
| UV | 1 | Unorm16x2 | 4 bytes |
| Color | 2 | Unorm8x4 | 4 bytes |
| Normal | 3 | Uint32 (octahedral) | 4 bytes |
| Bone Indices | 4 | Uint8x4 | 4 bytes |
| Bone Weights | 5 | Unorm8x4 | 4 bytes |

**Packing functions:**
- Position: `pack_position_f16()` — 3 floats → 4 half-floats (padded)
- UV: `pack_uv_unorm16()` — 2 floats [0,1] → 2 unorm16
- Color: `pack_color_rgba_unorm8()` — 3 floats → 4 unorm8
- Normal: `pack_octahedral_u32()` — 3 floats → octahedral encoding in u32
- Skinning: `pack_bone_weights_unorm8()` — 4 floats → 4 unorm8

### Mode Requirements

Render modes 1-3 require normals (FORMAT_NORMAL flag):

| Mode | Valid Formats |
|------|---------------|
| 0 (Unlit) | All 16 formats |
| 1 (Matcap) | 4, 5, 6, 7, 12, 13, 14, 15 |
| 2 (MR-Blinn-Phong) | 4, 5, 6, 7, 12, 13, 14, 15 |
| 3 (Blinn-Phong) | 4, 5, 6, 7, 12, 13, 14, 15 |

---

## Shader Generation

### Build-Time Generation

All 40 shader permutations are generated at **compile time** by `build.rs`:

```
Total: 40 shaders
- Mode 0: 16 shaders (all vertex formats)
- Mode 1: 8 shaders (formats with NORMAL)
- Mode 2: 8 shaders (formats with NORMAL)
- Mode 3: 8 shaders (formats with NORMAL)
```

**Why compile-time?**
- Validated with naga before shipping
- Zero runtime shader compilation cost
- Guaranteed working shaders

### Template System

Shaders are generated from WGSL templates with placeholder replacement:

**Template files:**
- `shaders/mode0_lambert.wgsl` — Mode 0 template
- `shaders/mode1_matcap.wgsl` — Mode 1 template
- `shaders/blinnphong_common.wgsl` — Modes 2-3 common code
- `shaders/common.wgsl` — Shared utilities

**Placeholders replaced:**

Vertex inputs:
- `//VIN_UV` → `@location(1) uv: vec2<f32>,`
- `//VIN_COLOR` → `@location(2) color: vec4<f32>,`
- `//VIN_NORMAL` → `@location(3) normal: u32,`
- `//VIN_SKINNED` → bone indices/weights

Vertex shader body:
- `//VS_UV` → UV passthrough
- `//VS_COLOR` → Color passthrough
- `//VS_WORLD_NORMAL` → Normal transform
- `//VS_SKINNED` → Skinning calculations

Fragment shader:
- `//FS_COLOR` → Vertex color sampling
- `//FS_UV` → Texture coordinate usage
- `//FS_NORMAL` → Normal unpacking
- `//FS_MRE` → Metallic/roughness/emissive sampling

### Additional Shaders

Beyond the 40 main permutations:

- **SKY_SHADER**: Procedural sky rendering (gradient + sun)
- **QUAD_SHADER**: GPU-instanced billboards and sprites

---

## Material System

### Texture Slots by Mode

| Slot | Mode 0 | Mode 1 | Mode 2 | Mode 3 |
|------|--------|--------|--------|--------|
| 0 | Albedo | Albedo | Albedo | Albedo |
| 1 | - | Matcap 1 | MRE | SSE |
| 2 | - | Matcap 2 | Unused | Specular Color |
| 3 | - | Matcap 3 | - | - |

**Acronyms:**
- **MRE**: Metallic (R), Roughness (G), Emissive (B)
- **SSE**: Specular Damping (R), Shininess (G), Emissive (B)

### Material Properties (Uniform Fallbacks)

When textures aren't bound, uniform values are used:

```rust
fn material_metallic(value: f32)   // Mode 2: 0.0 (dielectric) to 1.0 (metal)
fn material_roughness(value: f32)  // Mode 2: 0.0 (smooth) to 1.0 (rough)
fn material_emissive(value: f32)   // All modes: glow intensity
fn material_shininess(value: f32)  // Mode 3: 0.0-1.0 → shininess 1-256
fn material_rim(intensity: f32, power: f32)  // Mode 3: rim lighting
```

### Packed Shading State

Per-draw state is packed into a 96-byte structure for GPU upload:

```rust
struct PackedUnifiedShadingState {
    color_rgba8: u32,        // Material color
    uniform_set_0: u32,      // Mode-specific (4 × u8)
    uniform_set_1: u32,      // Mode-specific (4 × u8)
    flags: u32,              // Skinning mode, texture filter
    sky: PackedSky,          // 16 bytes
    lights: [PackedLight; 4], // 48 bytes
    // Animation fields: 16 bytes
}
```

**Mode-specific uniform_set_0:**

| Mode | Byte 0 | Byte 1 | Byte 2 | Byte 3 |
|------|--------|--------|--------|--------|
| 0 | - | - | - | Rim Intensity |
| 1 | Blend 0 | Blend 1 | Blend 2 | Blend 3 |
| 2 | Metallic | Roughness | Emissive | Rim Intensity |
| 3 | Spec Damping* | Shininess | Emissive | Rim Intensity |

*Spec Damping is inverted: 0 = full specular (default), 255 = no specular

---

## Lighting System

### Light Types

Two light types supported:

**Directional lights:**
```rust
fn light_set(index: u32, x: f32, y: f32, z: f32)  // Direction rays travel
```

**Point lights:**
```rust
fn light_set_point(index: u32, x: f32, y: f32, z: f32)  // World position
fn light_range(index: u32, range: f32)                   // Falloff distance
```

### Light Properties

```rust
fn light_color(index: u32, color: u32)           // 0xRRGGBBAA
fn light_intensity(index: u32, intensity: f32)   // 0.0-8.0 (HDR range)
fn light_enable(index: u32)
fn light_disable(index: u32)
```

**Intensity range**: 0.0-8.0 for HDR support. Values >1.0 useful for point light falloff.

### Procedural Environment

The environment provides ambient lighting for all modes via the Environment Processing Unit (EPU).

**Dual-Layer Architecture:**

The EPU supports two independent environment layers (base and overlay) that can be blended together:

```rust
// Configure base layer (layer 0)
fn env_gradient(layer: u32, zenith: u32, sky_horizon: u32, ground_horizon: u32,
                nadir: u32, rotation: f32, shift: f32)

// Configure overlay layer (layer 1) with a different mode
fn env_scatter(layer: u32, variant: u32, density: u32, ...)

// Set blend mode for overlay compositing
fn env_blend(mode: u32)  // 0=alpha, 1=add, 2=multiply, 3=screen

// Render the configured environment
fn draw_env()
```

**Layer System:**
- Layer 0 (base): Primary environment layer
- Layer 1 (overlay): Secondary layer composited on top
- Same mode can be used on both layers with different parameters
- Example: Stars (layer 0) + rain (layer 1), both using scatter mode

**Ambient calculation:**
```
ambient = sample_sky(normal) * albedo * ambient_factor
```

**Lighting integration:**
- Mode 0 with normals: Simple Lambert from directional lights
- Modes 2-3: Dynamic lights + environment ambient
- See [Environment (EPU) API](../../book/src/api/epu.md) for all 8 procedural modes

### Packed Light Format

Lights are packed into 12 bytes for GPU upload:

```rust
struct PackedLight {
    data0: u32,  // Direction (oct) or position XY (f16x2)
    data1: u32,  // RGB8 + type (1 bit) + intensity (7 bits)
    data2: u32,  // Point: Z (f16) + range (f16)
}
```

---

## Animation System

### Bone Matrix Storage

Animation uses 3×4 matrices (48 bytes each) with implicit `[0, 0, 0, 1]` row:

```
unified_animation buffer layout:
[inverse_bind matrices | keyframe matrices | immediate matrices]
     └─ static ─┘         └─ static ─┘       └─ per-frame ─┘
```

**Indices in shading state:**
- `inverse_bind_base`: Start of skeleton's inverse bind matrices
- `keyframe_base`: Start of animation keyframe matrices
- `animation_flags`: Bit 0 = use static keyframes vs immediate bones

### GPU Skinning

Vertex shader applies up to 4-bone skinning:

```wgsl
var skin_matrix = mat4x4<f32>(0.0);
for (var i = 0u; i < 4u; i++) {
    let bone_index = bone_indices[i];
    let weight = bone_weights[i];
    skin_matrix += get_bone_matrix(bone_index) * weight;
}
position = skin_matrix * position;
```

---

## Performance Considerations

### VRAM Limits

- **Total VRAM**: 4 MB
- Tracked via `TextureManager.vram_used()`
- Textures + mesh buffers count toward limit

### Draw Call Optimization

Commands are sorted by a multi-level key to minimize state changes and ensure correct rendering order:

```
Sort order (ascending):
1. Viewport region (split-screen)
2. Layer (2D ordering - higher = on top)
3. Stencil mode (masking groups)
4. Render type (Quad=0, Mesh=1, Sky=2)
5. Vertex format (meshes only)
6. Depth test & cull mode
7. Texture bindings
```

**Render type ordering (optimized for performance):**

- **Quad=0**: Renders first, writes depth=0.0 for early-z culling of 3D behind UI
- **Mesh=1**: Renders second, culled where quads wrote depth
- **Sky=2**: Renders last, only fills gaps where depth==1.0 (background)

**Key principles:**

- **Layer-first sorting for 2D**: Quads at different layers never batch together, ensuring correct 2D ordering
- **Screen-space quads**: Render first with depth writes (early-z optimization)
- **World-space quads/billboards**: Use depth testing for 3D occlusion (layer 0)
- **Within same layer**: Batch by texture for performance
- **Sky last**: Depth test skips pixels covered by geometry, saving shader invocations

**Batching benefits:**
- Early-z culling reduces 3D fragment shader cost behind UI
- Correct 2D layer ordering via CPU sorting
- Sky optimization avoids unnecessary shader invocations
- Fewer pipeline switches
- Fewer texture bind group changes
- Better GPU parallelism

### Immediate vs Retained Mode

**Immediate mode** (`draw_triangles`):
- Vertices buffered per-frame
- Good for dynamic geometry
- CPU overhead for packing

**Retained mode** (`draw_mesh`):
- Persistent GPU buffers
- Upload once, draw many times
- Better for static geometry

### Vertex Buffer Architecture

One buffer per stride (16 buffers total):

**Why separate buffers?**
- No padding waste
- Format 0 (8 bytes) doesn't pad to Format 15 (28 bytes)
- Efficient memory usage

### Bind Group Caching

Frame bind group is cached and only recreated when buffers change:

```rust
cached_frame_bind_group: Option<wgpu::BindGroup>
cached_frame_bind_group_hash: u64
```

Invalidate on:
- Buffer resize/recreation
- Game resource clear

---

## Source File Reference

| File | Description |
|------|-------------|
| `nethercore-zx/src/graphics/mod.rs` | ZGraphics main implementation |
| `nethercore-zx/src/graphics/frame.rs` | Frame rendering and command execution |
| `nethercore-zx/src/graphics/pipeline.rs` | Pipeline cache and creation |
| `nethercore-zx/src/graphics/vertex.rs` | Vertex format definitions |
| `nethercore-zx/src/graphics/unified_shading_state.rs` | Shading state packing |
| `nethercore-zx/src/graphics/buffer.rs` | Buffer management |
| `nethercore-zx/src/graphics/command_buffer.rs` | Virtual render pass |
| `nethercore-zx/src/graphics/texture_manager.rs` | Texture loading and VRAM tracking |
| `nethercore-zx/src/shader_gen.rs` | Shader permutation system |
| `nethercore-zx/shaders/*.wgsl` | Shader templates |

---

## Common Issues

### "Render mode X requires NORMAL flag"

Modes 1-3 need normals for lighting calculations. Use formats 4-7 or 12-15.

### Black screen after init

Ensure `render_mode()` is called in `init()`, not `update()` or `render()`.

### Textures appear as checkerboard

The checkerboard is the fallback texture. Check:
- Texture handle is valid
- `bind_texture()` called before drawing
- Texture loaded successfully in `init()`

### Skinned mesh not animating

Verify:
- Skeleton bound with `bind_skeleton()`
- Animation loaded and playing
- Vertex format includes SKINNED flag (formats 8-15)
