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
fn material_metallic(value: f32)            // 0.0 = dielectric, 1.0 = metal
fn material_roughness(value: f32)           // 0.0 = mirror, 1.0 = rough
fn material_emissive(value: f32)            // Glow intensity
```

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

### Textures

Games embed assets via `include_bytes!()` and pass raw pixels — no file-based loading. All resources are created in `init()` and automatically cleaned up on game shutdown.

```rust
fn texture_create(width: u32, height: u32, pixels: *const u8) -> u32
fn texture_bind(handle: u32)
```

**Example:**
```rust
static SPRITE_PNG: &[u8] = include_bytes!("assets/sprite.png");

fn init() {
    let (w, h, pixels) = decode_png(SPRITE_PNG);
    let tex = texture_create(w, h, pixels.as_ptr());
}
```

### 3D Drawing

```rust
fn draw_triangle(
    x0: f32, y0: f32, z0: f32, u0: f32, v0: f32,
    x1: f32, y1: f32, z1: f32, u1: f32, v1: f32,
    x2: f32, y2: f32, z2: f32, u2: f32, v2: f32,
    color: u32
)

fn draw_mesh(
    vertices: *const f32,    // x, y, z, u, v per vertex
    vertex_count: u32,
    indices: *const u16,
    index_count: u32,
    color: u32
)
```

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
fn depth_test(enabled: u32)             // 0 = off, 1 = on
fn cull_mode(mode: u32)                 // 0 = none, 1 = back, 2 = front
fn blend_mode(mode: u32)                // 0 = none, 1 = alpha, 2 = additive, 3 = multiply
fn texture_filter(filter: u32)          // 0 = nearest, 1 = linear
```

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
