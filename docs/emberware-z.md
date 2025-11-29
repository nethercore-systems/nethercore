# Emberware Z — Console Specification

Emberware Z is a 5th-generation fantasy console targeting PS1/N64/Saturn aesthetics with modern conveniences.

## Console Specs

| Spec | Value |
|------|-------|
| **Aesthetic** | PS1/N64/Saturn (5th gen) |
| **Resolution** | 360p, 540p (default), 720p, 1080p |
| **Color depth** | RGBA8 |
| **Tick rate** | 24, 30, 60 (default), 120 fps |
| **RAM** | 16MB |
| **VRAM** | 8MB |
| **CPU budget** | 4ms per tick (at 60fps) |
| **ROM size** | 32MB max |
| **Netcode** | Deterministic rollback via GGRS |
| **Max players** | 4 (any mix of local + remote) |

### Configuration (init-only)

These settings **must be called in `init()`** — they cannot be changed at runtime.

```rust
fn set_resolution(res: u32)             // 0=360p, 1=540p (default), 2=720p, 3=1080p
fn set_tick_rate(fps: u32)              // 24, 30, 60 (default), or 120
fn set_clear_color(color: u32)          // 0xRRGGBBAA, default: 0x000000FF (black)
fn render_mode(mode: u32)               // 0-3, see Rendering Modes below
```

If not set, defaults to 540p @ 60fps with render mode 0 (Unlit).

---

## Controller

Emberware Z uses a modern PS2/Xbox-style controller:

```
         [LB]                    [RB]
         [LT]                    [RT]
        ┌─────────────────────────────┐
       │  [^]              [Y]        │
       │ [<][>]    [☐][☐]  [X] [B]    │
       │  [v]              [A]        │
       │       [SELECT] [START]       │
       │        [L3]     [R3]         │
        └─────────────────────────────┘
           Left      Right
           Stick     Stick
```

- **D-Pad:** 4 directions
- **Face buttons:** A, B, X, Y
- **Shoulder bumpers:** LB, RB (digital)
- **Triggers:** LT, RT (analog 0.0-1.0)
- **Sticks:** Left + Right (analog -1.0 to 1.0, clickable L3/R3)
- **Menu:** Start, Select

### Button Constants

```rust
// D-Pad
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_LEFT: u32 = 2;
const BUTTON_RIGHT: u32 = 3;

// Face buttons
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;

// Shoulder bumpers
const BUTTON_LB: u32 = 8;
const BUTTON_RB: u32 = 9;

// Stick clicks
const BUTTON_L3: u32 = 10;
const BUTTON_R3: u32 = 11;

// Menu
const BUTTON_START: u32 = 12;
const BUTTON_SELECT: u32 = 13;
```

---

## Input FFI

### Individual Button Queries (Convenient)

```rust
fn button_held(player: u32, button: u32) -> u32     // 1 if held, 0 otherwise
fn button_pressed(player: u32, button: u32) -> u32  // 1 if just pressed this tick
fn button_released(player: u32, button: u32) -> u32 // 1 if just released this tick
```

### Bulk Button Queries (Efficient)

```rust
fn buttons_held(player: u32) -> u32     // Bitmask of all held buttons
fn buttons_pressed(player: u32) -> u32  // Bitmask of all just pressed
fn buttons_released(player: u32) -> u32 // Bitmask of all just released
```

Use bulk queries when checking multiple buttons to reduce FFI overhead:

```rust
let held = buttons_held(0);
if held & (1 << BUTTON_A) != 0 { /* A held */ }
if held & (1 << BUTTON_B) != 0 { /* B held */ }
```

### Analog Sticks

```rust
// Individual axis queries
fn left_stick_x(player: u32) -> f32   // -1.0 to 1.0
fn left_stick_y(player: u32) -> f32   // -1.0 to 1.0
fn right_stick_x(player: u32) -> f32  // -1.0 to 1.0
fn right_stick_y(player: u32) -> f32  // -1.0 to 1.0

// Bulk queries (one FFI call for both axes)
fn left_stick(player: u32, out_x: *mut f32, out_y: *mut f32)
fn right_stick(player: u32, out_x: *mut f32, out_y: *mut f32)
```

### Analog Triggers

```rust
fn trigger_left(player: u32) -> f32   // 0.0 to 1.0
fn trigger_right(player: u32) -> f32  // 0.0 to 1.0
```

---

## Graphics FFI

### Frame Handling

The runtime automatically:
- Clears the screen to `set_clear_color()` before each `render()` call
- Presents the frame after `render()` returns

No manual `frame_begin()`/`frame_end()` calls needed.

### Camera

```rust
fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32)
fn camera_fov(fov_degrees: f32)         // Default: 60
```

### Rendering Modes

Emberware Z supports 4 forward rendering modes.

**⚠️ Must be set in `init()` only.** Cannot be changed at runtime.

```rust
fn render_mode(mode: u32)               // 0-3, see below (init-only)
```

| Mode | Name | Lights | Description |
|------|------|--------|-------------|
| 0 | **Unlit** | None | Texture × vertex color. No lighting calculations. |
| 1 | **Matcap** | None (baked) | Adds view-space normal matcap sampling. Stylized, cheap. |
| 2 | **PBR-lite** | 4 lights | Physically-based rendering. Dynamic lighting, most realistic. |
| 3 | **Hybrid** | 1 dir + ambient | Matcap for reflections + PBR for direct lighting. |

Each mode builds on the previous — textures and vertex colors always work.

#### Mode 0: Unlit

No lighting calculations. Output = texture × vertex color.

```
final_color = texture_sample * vertex_color
```

#### Mode 1: Matcap

Adds view-space normal sampling from up to 4 blended matcap textures. Lighting is "baked" into the matcap — cheap stylized look.

```rust
fn matcap_set(slot: u32, texture: u32)      // slot 0-3
fn matcap_blend(m0: f32, m1: f32, m2: f32, m3: f32)  // Blend weights (normalized)
```

```
view_normal = transform_normal_to_view_space(surface_normal)
matcap_uv = view_normal.xy * 0.5 + 0.5
final_color = texture * vertex_color * matcap_sample(matcap_uv)
```

Good for:
- Stylized/toon rendering
- Metallic/shiny materials without environment maps
- Consistent look regardless of scene setup
- Fast performance

#### Mode 2: PBR-lite (4 Lights)

Full PBR lighting with up to 4 dynamic lights:
- GGX specular distribution
- Schlick fresnel approximation
- Energy-conserving Lambert diffuse
- Emissive support

```rust
fn light_set(index: u32, light_type: u32, x: f32, y: f32, z: f32)  // index 0-3
fn light_color(index: u32, r: f32, g: f32, b: f32)
fn light_intensity(index: u32, intensity: f32)
fn light_disable(index: u32)
```

Light types: 0 = ambient, 1 = directional, 2 = point, 3 = spot (TBD)

Material properties via MRE texture (R=Metallic, G=Roughness, B=Emissive):

```rust
fn material_mre(texture: u32)               // Metallic/Roughness/Emissive packed texture
fn material_albedo(texture: u32)            // Base color (linear RGB)
```

Or set directly:
```rust
fn material_metallic(value: f32)            // 0.0 = dielectric (default), 1.0 = metal
fn material_roughness(value: f32)           // 0.0 = mirror (default), 1.0 = rough
fn material_emissive(value: f32)            // Glow intensity (default: 0.0)
```

**Material defaults:** All material properties default to 0.0 (dielectric, mirror-smooth, no emissive).

```
// Per-light contribution
diffuse = (1 - F0) * (1 - metallic) * albedo / PI
specular = D_GGX * F_schlick
direct = (diffuse + specular) * light_color * NdotL

final_color = sum(direct) + ambient * albedo + emissive
```

#### Mode 3: Hybrid (Matcap + PBR)

Best of both worlds with constrained lighting:
- **Matcap** provides ambient reflections (replaces environment maps)
- **PBR** handles direct lighting from 1 directional light + ambient
- Good balance of quality and performance

```rust
// Matcap for reflections
fn matcap_set(slot: u32, texture: u32)
fn matcap_blend(m0: f32, m1: f32, m2: f32, m3: f32)

// Single directional light + ambient
fn light_direction(x: f32, y: f32, z: f32)  // Normalized direction TO light
fn light_color(r: f32, g: f32, b: f32)      // Linear RGB
fn ambient_color(r: f32, g: f32, b: f32)    // Linear RGB

// PBR material properties
fn material_metallic(value: f32)
fn material_roughness(value: f32)
fn material_emissive(value: f32)
```

```
// Matcap modulates the ambient/reflection term
matcap = matcap_sample(view_normal)
ambient_reflection = matcap * ambient_color * albedo

// PBR handles direct light
direct = pbr_direct(light_direction, light_color, material)

final_color = direct + ambient_reflection + emissive
```

**Note:** All lit modes output linear RGB. The runtime applies tonemapping and gamma correction.

### Vertex Formats

Vertex data is packed `[f32]` arrays. The format is a 3-bit bitmask determining which attributes are present.

```rust
// Vertex format flags (bitmask)
const FORMAT_UV: u32 = 1;      // Has UV coordinates
const FORMAT_COLOR: u32 = 2;   // Has per-vertex color (RGB, 3 floats)
const FORMAT_NORMAL: u32 = 4;  // Has normals

// All 8 possible combinations:
const FORMAT_POS: u32 = 0;                    // pos(3) = 12 bytes
const FORMAT_POS_UV: u32 = 1;                 // pos(3) + uv(2) = 20 bytes
const FORMAT_POS_COLOR: u32 = 2;              // pos(3) + color(3) = 24 bytes
const FORMAT_POS_UV_COLOR: u32 = 3;           // pos(3) + uv(2) + color(3) = 32 bytes
const FORMAT_POS_NORMAL: u32 = 4;             // pos(3) + normal(3) = 24 bytes
const FORMAT_POS_UV_NORMAL: u32 = 5;          // pos(3) + uv(2) + normal(3) = 32 bytes
const FORMAT_POS_COLOR_NORMAL: u32 = 6;       // pos(3) + color(3) + normal(3) = 36 bytes
const FORMAT_POS_UV_COLOR_NORMAL: u32 = 7;    // pos(3) + uv(2) + color(3) + normal(3) = 44 bytes
```

**Attribute order:** position → uv (if present) → color (if present) → normal (if present)

**Color format:** RGB as 3 floats (0.0-1.0 range)

| Format | Stride | Example Use Case |
|--------|--------|------------------|
| POS | 12 | Debug wireframes, solid color shapes |
| POS_UV | 20 | Textured geometry (unlit) |
| POS_COLOR | 24 | Vertex-colored geometry (no texture) |
| POS_UV_COLOR | 32 | Textured + per-vertex tint |
| POS_NORMAL | 24 | Lit geometry with matcap as albedo (slot 0) |
| POS_UV_NORMAL | 32 | Standard lit textured geometry |
| POS_COLOR_NORMAL | 36 | Lit vertex-colored geometry |
| POS_UV_COLOR_NORMAL | 44 | Full-featured: texture + vertex color + lighting |

**Notes:**
- Formats without UV can use matcap in slot 0 as base color
- Formats without COLOR use `set_color()` for uniform tint
- Formats without NORMAL only work correctly in Mode 0 (Unlit)

### GPU Skinning

Emberware Z supports GPU-based skeletal animation. Developers animate bones on CPU (calculate bone transforms each frame), and the GPU performs skinning (vertex deformation based on bone weights) in the vertex shader.

**Skinned vertex format flag:**

```rust
const FORMAT_SKINNED: u32 = 8;  // Has bone indices (4 × u8) + bone weights (4 × f32)
```

When `FORMAT_SKINNED` is set, each vertex includes:
- `bone_indices`: 4 × u8 (4 bytes packed as one u32) — which bones affect this vertex (indices 0-255)
- `bone_weights`: 4 × f32 (16 bytes) — weight of each bone's influence (should sum to 1.0)

This adds 20 bytes to the vertex stride. Maximum 256 bones per skeleton. Can combine with other flags:

```rust
// Skinned mesh with UVs and normals for lit rendering
const FORMAT_SKINNED_UV_NORMAL: u32 = FORMAT_SKINNED | FORMAT_UV | FORMAT_NORMAL;
// = 8 | 1 | 4 = 13 → stride = 32 + 20 = 52 bytes
```

**Attribute order for skinned vertices:**
position → uv (if present) → color (if present) → normal (if present) → bone_indices → bone_weights

**Bone transform upload:**

```rust
fn set_bones(matrices: *const f32, count: u32)  // 16 floats per bone (4×4 matrix, column-major)
```

Call `set_bones()` before `draw_mesh()` or `draw_triangles()` to upload the current bone transforms. Maximum 256 bones per skeleton.

**Workflow:**

1. **In `init()`:** Load skinned mesh with bone indices/weights baked into vertex data
2. **Each `update()`:** Animate skeleton on CPU (update bone transforms from keyframes/blend trees)
3. **Each `render()`:** Call `set_bones()` with current transforms, then `draw_mesh()`

**Example:**

```rust
static mut CHARACTER_MESH: u32 = 0;
static mut BONE_MATRICES: [f32; 256 * 16] = [0.0; 256 * 16];  // Up to 256 bones
static mut BONE_COUNT: u32 = 0;

fn init() {
    unsafe {
        // Load skinned mesh (pos + uv + normal + bones)
        CHARACTER_MESH = load_mesh(
            CHARACTER_VERTS.as_ptr() as *const u8,
            CHARACTER_VERT_COUNT,
            FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED
        );
        BONE_COUNT = 24;  // This character has 24 bones

        // Initialize bone matrices to identity
        for i in 0..BONE_COUNT as usize {
            // Column-major identity matrix
            BONE_MATRICES[i * 16 + 0] = 1.0;  // col0.x
            BONE_MATRICES[i * 16 + 5] = 1.0;  // col1.y
            BONE_MATRICES[i * 16 + 10] = 1.0; // col2.z
            BONE_MATRICES[i * 16 + 15] = 1.0; // col3.w
        }
    }
}

fn update() {
    unsafe {
        // Animate bones on CPU (your animation system)
        // Update BONE_MATRICES with new transforms for each bone
        animate_skeleton(&mut BONE_MATRICES, elapsed_time());
    }
}

fn render() {
    unsafe {
        texture_bind(CHARACTER_TEXTURE);
        set_bones(BONE_MATRICES.as_ptr(), BONE_COUNT);  // Upload bone transforms
        transform_translate(0.0, 0.0, -5.0);
        draw_mesh(CHARACTER_MESH);
    }
}
```

**Notes:**
- Bone matrices are world-space (or object-space if you prefer — just be consistent)
- The vertex shader computes: `skinned_pos = Σ(bone_weight[i] * bone_matrix[bone_index[i]] * vertex_pos)`
- Normals are also transformed using the inverse transpose of the bone matrix
- For best performance, limit to 4 bones per vertex with normalized weights
- CPU-side animation (keyframes, blend trees, IK) is left to the developer

### Textures

Games embed assets via `include_bytes!()` and pass raw pixels — no file-based loading. All resources are created in `init()` and automatically cleaned up on game shutdown.

```rust
fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32
fn texture_bind(handle: u32)                    // Bind to slot 0 (albedo)
fn texture_bind_slot(handle: u32, slot: u32)    // Bind to specific slot
```

**Texture slots per render mode:**

| Mode | Slot 0 | Slot 1 | Slot 2 | Slot 3 |
|------|--------|--------|--------|--------|
| **0 (Unlit)** | Albedo (UV) | — | — | — |
| **1 (Matcap)** | Albedo (UV) | Matcap (N) | Matcap (N) | Matcap (N) |
| **2 (PBR)** | Albedo (UV) | MRE (UV) | Reflection (N) | — |
| **3 (Hybrid)** | Albedo (UV) | MRE (UV) | Reflection (N) | Matcap (N) |

**(N) = Normal-sampled (requires `FORMAT_NORMAL`), (UV) = UV-sampled (requires `FORMAT_UV`)**

**Fallback rules:**
- UV-sampled slots with no UVs or no texture → use `set_color()` / `material_*()` uniforms
- Normal-sampled slots with no texture → use ambient color only
- Modes 1-3 without `FORMAT_NORMAL` → warning, behaves like Mode 0

**Debug fallback texture:** When a required texture is missing, an 8×8 magenta/black checkerboard is used to make the error visually obvious during development.

**Matcap combination:**
- Mode 1: Matcaps in slots 1-3 multiply together (more blend modes TBD)
- Mode 3: Slot 2 (reflection) and slot 3 (matcap) multiply

**Example:**
```rust
static SPRITE_PNG: &[u8] = include_bytes!("assets/sprite.png");

fn init() {
    let (w, h, pixels) = decode_png(SPRITE_PNG);
    let tex = load_texture(w, h, pixels.as_ptr());
}
```

### Meshes (Retained Mode)

Load meshes in `init()`, draw by handle in `render()`. Specify vertex format when loading.

```rust
fn load_mesh(
    data: *const u8,
    vertex_count: u32,
    format: u32              // Vertex format flags
) -> u32

fn load_mesh_indexed(
    data: *const u8,
    vertex_count: u32,
    indices: *const u16,
    index_count: u32,
    format: u32              // Vertex format flags
) -> u32

fn draw_mesh(handle: u32)
```

**Example:**
```rust
static mut CUBE_MESH: u32 = 0;

// Cube with normals for lighting (pos + uv + normal = 8 floats per vertex)
static CUBE_VERTS: &[f32] = &[
    // pos(3), uv(2), normal(3)
    -1.0, -1.0,  1.0,  0.0, 0.0,  0.0, 0.0, 1.0,  // front face...
    // ...
];

fn init() {
    unsafe {
        CUBE_MESH = load_mesh_indexed(
            CUBE_VERTS.as_ptr() as *const u8,
            24,  // 24 vertices
            CUBE_INDICES.as_ptr(),
            36,  // 36 indices
            FORMAT_UV_NORMAL
        );
    }
}

fn render() {
    unsafe {
        texture_bind(CUBE_TEXTURE);
        set_color(0xFFFFFFFF);  // White tint
        transform_translate(0.0, 0.0, -5.0);
        draw_mesh(CUBE_MESH);
    }
}
```

### Immediate Mode 3D

For dynamic geometry, skinned meshes, or prototyping. Push vertices each frame (buffered internally, flushed once per frame).

```rust
fn draw_triangles(
    data: *const u8,
    vertex_count: u32,
    format: u32              // Vertex format flags
)

fn draw_triangles_indexed(
    data: *const u8,
    vertex_count: u32,
    indices: *const u16,
    index_count: u32,
    format: u32              // Vertex format flags
)
```

**Note:** Immediate mode is convenient but less efficient. Prefer `load_mesh` + `draw_mesh` for static geometry. Use immediate mode for:
- Skinned/animated meshes (CPU-transformed vertices)
- Procedural geometry
- Debug visualization

### Transform Stack

```rust
fn transform_identity()
fn transform_translate(x: f32, y: f32, z: f32)
fn transform_rotate(angle_deg: f32, x: f32, y: f32, z: f32)
fn transform_scale(x: f32, y: f32, z: f32)
fn transform_push()
fn transform_pop()
fn transform_set(matrix: *const f32)    // 16 floats, column-major
```

**Math conventions:**
- Matrices are **column-major** (compatible with glam, WGSL, OpenGL)
- Column vectors: `v' = M * v`
- Angles are in **degrees** (converted internally to radians)
- Y-up coordinate system, right-handed

### 2D Drawing

**Simple:**

```rust
fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32)
fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32)
fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32)
```

**With source region (for sprite sheets):**

```rust
fn draw_sprite_region(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    color: u32
)
```

**Full control (region + rotation + origin):**

```rust
fn draw_sprite_ex(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    origin_x: f32, origin_y: f32,   // Pivot point (0-1, default 0,0 = top-left)
    angle_deg: f32,                  // Rotation in degrees
    color: u32
)
```

Example with centered rotation:
```rust
// Rotate 45° around center
draw_sprite_ex(100.0, 100.0, 32.0, 32.0, 0.0, 0.0, 32.0, 32.0, 0.5, 0.5, 45.0, 0xFFFFFFFF);
```

### Render State

```rust
fn set_color(color: u32)                // Tint color (0xRRGGBBAA), multiplied with vertex color
fn depth_test(enabled: u32)             // 0 = off, 1 = on
fn cull_mode(mode: u32)                 // 0 = none, 1 = back, 2 = front
fn blend_mode(mode: u32)                // 0 = none, 1 = alpha, 2 = additive, 3 = multiply
fn texture_filter(filter: u32)          // 0 = nearest, 1 = linear
```

**Color handling:**
- `set_color()` sets a uniform tint color
- If vertex format includes COLOR → per-vertex color × uniform color
- If vertex format has no COLOR → uniform color only
- Default: white (0xFFFFFFFF)

---

## Audio FFI

> **TODO [needs clarification]:** Audio system is shelved for initial implementation.

---

## Complete Example

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { loop {} }

#[link(wasm_import_module = "emberware")]
extern "C" {
    fn set_clear_color(color: u32);
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn trigger_right(player: u32) -> f32;
    fn player_count() -> u32;
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

static mut PLAYER_X: [f32; 4] = [160.0; 4];
static mut PLAYER_Y: [f32; 4] = [120.0; 4];

#[no_mangle]
pub extern "C" fn init() {
    unsafe { set_clear_color(0x1a1a2eFF); }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        for p in 0..player_count() {
            let i = p as usize;

            // Analog stick movement
            PLAYER_X[i] += left_stick_x(p) * 5.0;
            PLAYER_Y[i] += left_stick_y(p) * 5.0;

            // Boost with right trigger
            let boost = 1.0 + trigger_right(p) * 2.0;
            PLAYER_X[i] += left_stick_x(p) * boost;

            // Clamp to screen
            PLAYER_X[i] = PLAYER_X[i].clamp(0.0, 300.0);
            PLAYER_Y[i] = PLAYER_Y[i].clamp(0.0, 220.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        let colors = [0x4a9fffFF, 0xff6b6bFF, 0x6bff6bFF, 0xffff6bFF];
        for p in 0..player_count() as usize {
            draw_rect(PLAYER_X[p], PLAYER_Y[p], 20.0, 20.0, colors[p]);
        }

        let title = b"Emberware Z Demo";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 12.0, 0xFFFFFFFF);
    }
}
```
