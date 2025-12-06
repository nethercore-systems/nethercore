# Emberware Z — Console Specification

Emberware Z is a 5th-generation fantasy console targeting PS1/N64/Saturn aesthetics with modern conveniences.

## Console Specs

| Spec | Value |
|------|-------|
| **Aesthetic** | PS1/N64/Saturn (5th gen) |
| **Resolution** | 360p, 540p (default), 720p, 1080p |
| **Color depth** | RGBA8 |
| **Tick rate** | 24, 30, 60 (default), 120 fps |
| **RAM** | 4MB |
| **VRAM** | 4MB |
| **CPU budget** | 4ms per tick (at 60fps) |
| **ROM size** | 12MB max (uncompressed) |
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
| 0 | **Unlit** | Sky (if normals) | Texture × vertex color. Simple Lambert if normals present. |
| 1 | **Matcap** | None (baked) | Adds view-space normal matcap sampling. Stylized, cheap. |
| 2 | **MR-Blinn-Phong** | 4 lights + sun | Metallic-roughness Blinn-Phong. Energy-conserving lighting, physical materials. |
| 3 | **Blinn-Phong** | 4 lights + sun | Classic lighting with Gotanda normalization. Explicit specular + rim lighting. Era-authentic PS1/N64 aesthetic. |

Each mode builds on the previous — textures and vertex colors always work.

#### Mode 0: Unlit (or Simple Lit)

Without normals: flat shading, no lighting calculations.
```
final_color = texture_sample * vertex_color
```

**With normals:** Simple Lambert shading using the procedural sky's sun direction. This gives you directional lighting "for free" just by including normals in your vertex format.

```
n_dot_l = max(0, dot(normal, sky.sun_direction))
direct = albedo * sky.sun_color * n_dot_l
ambient = albedo * sample_sky(normal) * 0.3
final_color = direct + ambient
```

This is much cheaper than PBR but still provides meaningful shading. To disable, set `sun_color` to black in `set_sky()`.

#### Mode 1: Matcap

Adds view-space normal sampling from up to 3 matcap textures in slots 1-3. Lighting is "baked" into the matcap — cheap stylized look.

```rust
fn matcap_set(slot: u32, texture: u32)      // slot 1-3 (texture binding slots)
```

```
view_normal = transform_normal_to_view_space(surface_normal)
matcap_uv = view_normal.xy * 0.5 + 0.5
final_color = albedo * vertex_color * matcap1 * matcap2 * matcap3
```

Matcaps in slots 1-3 multiply together by default. Unused slots default to white (no effect).

**Blend Modes:**

By default, matcaps multiply together. You can change how each matcap slot blends using:

```rust
fn matcap_blend_mode(slot: u32, mode: u32)  // slot 1-3, mode 0-2
```

**Available modes:**
- `0` = **Multiply** (default) — Standard matcap behavior, darkens
- `1` = **Add** — Additive blending, creates glow/emission effects
- `2` = **HSV Modulate** — Hue shift/iridescence, preserves luminance

**Example:**
```rust
// Slot 1: Base material (multiply)
matcap_set(1, base_matcap);
matcap_blend_mode(1, 0);  // Multiply (default)

// Slot 2: Rim light effect (add)
matcap_set(2, rim_matcap);
matcap_blend_mode(2, 1);  // Add for glow

// Slot 3: Iridescent overlay (HSV modulate)
matcap_set(3, rainbow_matcap);
matcap_blend_mode(3, 2);  // HSV modulate for color shift
```

**Use cases:**
- **Multiply**: Standard lighting, shadows, metallic reflections
- **Add**: Rim lights, glows, magical effects, highlights
- **HSV Modulate**: Iridescence (beetle shells, soap bubbles), hue tinting, color variation

**Performance:** All blend modes have identical cost. Choose based on visual needs.

Good for:
- Stylized/toon rendering
- Metallic/shiny materials without environment maps
- Consistent look regardless of scene setup
- Fast performance

#### Mode 2: Metallic-Roughness Blinn-Phong (4 Lights + Sun)

Normalized Blinn-Phong with metallic-roughness workflow. Energy-conserving lighting for physically-motivated materials:
- **Normalized Blinn-Phong** specular (Gotanda 2010 linear approximation)
- **Lambert** diffuse lighting
- **Metallic-roughness workflow** with derived specular color
- **Emissive** self-illumination
- **4 dynamic lights + sun** from procedural sky

**Reference:** Gotanda 2010 - "Practical Implementation at tri-Ace"

**Lighting Functions:**

```rust
fn light_set(index: u32, x: f32, y: f32, z: f32)  // index 0-3, direction vector
fn light_color(index: u32, r: f32, g: f32, b: f32)
fn light_intensity(index: u32, intensity: f32)
fn light_enable(index: u32)
fn light_disable(index: u32)

// Sun comes from procedural sky (set_sky_sun_direction, set_sky_sun_color)
```

All lights are directional. The `x`, `y`, `z` parameters specify the light direction (normalized internally).

**Material Properties:**

Material properties via MRE texture (R=Metallic, G=Roughness, B=Emissive):

```rust
fn material_mre(texture: u32)               // Metallic/Roughness/Emissive packed texture
fn material_albedo(texture: u32)            // Base color (linear RGB)
```

Or set directly:

```rust
fn material_metallic(value: f32)            // 0.0 = dielectric (default), 1.0 = metal
fn material_roughness(value: f32)           // 0.0 = smooth (default), 1.0 = rough
fn material_emissive(value: f32)            // Glow intensity (default: 0.0)
```

**Material defaults:** All material properties default to 0.0 (dielectric, smooth, no emissive).

**Texture Slots:**

| Slot | Purpose | Channels | Fallback |
|------|---------|----------|----------|
| 0 | Albedo | RGB: Diffuse color<br>A: Unused | White (uses material color) |
| 1 | MRE | R: Metallic<br>G: Roughness<br>B: Emissive | White (uses uniforms) |
| 2 | Unused | | (Freed up for future use) |

**Lighting Algorithm:**

```
// Roughness → Shininess mapping (power curve)
shininess = pow(256.0, 1.0 - roughness)  // Range: 256 (smooth) → 1 (rough)

// Specular color derivation (F0 calculation)
specular_color = mix(vec3(0.04), albedo, metallic)
                 // F0=0.04 for dielectrics, albedo for metals

// Gotanda normalization for energy conservation
normalization = shininess × 0.0397436 + 0.0856832

// Blinn-Phong specular (per light)
H = normalize(L + V)
NdotH = max(0, dot(N, H))
spec = normalization × pow(NdotH, shininess)
specular = specular_color × spec × light_color × NdotL

// Lambert diffuse
diffuse = albedo × light_color × NdotL

// Ambient with energy conservation
ambient_factor = 1.0 / sqrt(1.0 + normalization)
ambient = sample_sky(normal) × albedo × ambient_factor

final_color = sum(diffuse + specular) + ambient + emissive
```

**Material Examples:**

- **Polished gold** (metallic=1.0, roughness=0.2): Mirror-like with golden highlights
- **Rough plastic** (metallic=0.0, roughness=0.6): Matte with soft specular
- **Glowing crystal** (metallic=0.0, roughness=0.1, emissive=0.5): Bright with sharp highlights
- **Rusty metal** (metallic=0.8, roughness=0.8): Dull metal surface

**Roughness → Shininess Mapping:**

| Roughness | Shininess | Quality |
|-----------|-----------|---------|
| 0.0 | 256 | Mirror/Glass |
| 0.25 | 128 | Polished metal |
| 0.5 | 64 | Smooth plastic |
| 0.75 | 16 | Leather/Matte |
| 1.0 | 1 | Clay/Powder |

**Compared to Mode 3 (Blinn-Phong):**

| Feature | Mode 2 | Mode 3 |
|---------|--------|--------|
| **Specular Model** | Normalized Blinn-Phong | Normalized Blinn-Phong |
| **Metallic/Roughness** | ✅ MR workflow | ❌ Not available |
| **Specular Color** | Derived from metallic (F0=0.04) | Explicit (tex2.RGB or uniform) |
| **Rim Lighting** | ❌ No | ✅ Yes (controllable) |
| **Environment** | Procedural sky only | Procedural sky only |
| **Slot 2** | Unused/freed | Specular RGB texture |
| **Use Case** | Physical materials | Artistic control |

#### Mode 3: Normalized Blinn-Phong (Classic Lit)

Classic Blinn-Phong lighting with energy-conserving Gotanda normalization. Era-authentic for PS1/N64 aesthetic:
- **Normalized Blinn-Phong** specular (Gotanda 2010 linear approximation)
- **Lambert** diffuse lighting
- **Rim lighting** for edge definition (using sun color)
- **Emissive** self-illumination
- **4 dynamic lights + sun** (same count as Mode 2)

**Reference:** Gotanda 2010 - "Practical Implementation at tri-Ace"

**Lighting Functions (same as Mode 2):**

```rust
// 4 dynamic lights (index 0-3)
fn light_set(index: u32, x: f32, y: f32, z: f32)
fn light_color(index: u32, r: f32, g: f32, b: f32)
fn light_intensity(index: u32, intensity: f32)
fn light_enable(index: u32)
fn light_disable(index: u32)

// Sun lighting comes from procedural sky (sky_set_sun)
```

**Material Properties:**

Mode 3 reinterprets the same struct fields as Mode 2 (no API changes):

```rust
// Rim lighting (Mode 3 specific)
fn material_rim(intensity: f32, power: f32)  // intensity: 0-1, power: 0-1 → 0-32 range

// Shininess (Mode 3 alias for material_roughness)
fn material_shininess(value: f32)            // 0-1 → maps to shininess 1-256

// Emissive (same meaning as Mode 2)
fn material_emissive(value: f32)             // Glow intensity (default: 0.0)
```

**Texture Slots:**

| Slot | Purpose | Channels | Fallback |
|------|---------|----------|----------|
| 0 | Albedo | RGB: Diffuse color<br>A: Unused | White (uses material color) |
| 1 | SSE | R: Specular intensity<br>G: Shininess<br>B: Emissive | White (uses uniforms) |
| 2 | Specular | RGB: Specular highlight color<br>A: Unused | White (light-colored specular) |

**Lighting Algorithm:**

```
// Gotanda normalization for energy conservation
normalization = shininess × 0.0397436 + 0.0856832

// Specular color with intensity modulation
specular_color = texture_rgb × specular_intensity

// Blinn-Phong specular (per light)
H = normalize(L + V)
NdotH = max(0, dot(N, H))
NdotL = max(0, dot(N, L))
specular = specular_color × normalization × pow(NdotH, shininess) × light_color × NdotL

// Lambert diffuse (per light)
diffuse = albedo × light_color × NdotL

// Rim lighting (modulated by specular intensity, uses sun color)
rim_factor = pow(1 - NdotV, rim_power)
rim = sun_color × rim_factor × specular_intensity

// Ambient from sky (Gotanda-based energy conservation)
spec_norm = gotanda_normalization(shininess)
ambient_factor = 1.0 / sqrt(1.0 + spec_norm)
ambient = sample_sky(N) × albedo × ambient_factor

// Emissive
emissive_glow = albedo × emissive

final_color = sum(diffuse + specular) + ambient + rim + emissive_glow
```

**Key Features:**
- **Energy conservation:** Gotanda normalization ensures consistent brightness across shininess 1-256
- **No geometry term:** Era-authentic, classical Blinn-Phong didn't have it
- **Rim from sun:** Uses sun color for coherent scene lighting
- **Artist-friendly:** Direct control over specular color and shininess

**Shininess Mapping:**

| Value | Shininess | Visual | Use For |
|-------|-----------|--------|---------|
| 0.0-0.2 | 1-52 | Very broad, soft | Cloth, skin, rough stone |
| 0.2-0.4 | 52-103 | Broad | Leather, wood, rubber |
| 0.4-0.6 | 103-154 | Medium | Plastic, painted metal |
| 0.6-0.8 | 154-205 | Tight | Polished metal, wet surfaces |
| 0.8-1.0 | 205-256 | Very tight | Chrome, mirrors, glass |

**Example Materials:**

```rust
// Gold armor
set_color(0.9, 0.6, 0.2, 1.0);
material_shininess(0.8);       // Tight highlights
material_rim(0.2, 0.15);       // Subtle rim
material_emissive(0.0);

// Wet skin
set_color(0.85, 0.7, 0.65, 1.0);
material_shininess(0.7);       // Medium-tight highlights
material_rim(0.3, 0.25);       // Strong rim
material_emissive(0.0);

// Glowing crystal
set_color(0.3, 0.7, 0.9, 1.0);
material_shininess(0.75);      // Gem-like
material_rim(0.4, 0.18);       // Magical edge glow
material_emissive(0.3);        // Self-illumination
```

**Performance:** Both Mode 2 and Mode 3 use normalized Blinn-Phong with no geometry term — same performance, different features.

### Procedural Sky

Emberware Z includes a procedural sky system for backgrounds and environment lighting. The sky uses a hemispherical gradient with an analytical sun — no texture lookups, minimal GPU cost.

```rust
fn set_sky(
    horizon_r: f32, horizon_g: f32, horizon_b: f32,  // Horizon color (linear RGB)
    zenith_r: f32, zenith_g: f32, zenith_b: f32,     // Zenith color (linear RGB)
    sun_dir_x: f32, sun_dir_y: f32, sun_dir_z: f32,  // Normalized direction TO sun
    sun_r: f32, sun_g: f32, sun_b: f32,              // Sun color (linear RGB)
    sun_sharpness: f32                                // Sun disc sharpness (10-1000)
)
```

**Default:** All zeros (black sky, no sun). Call `set_sky()` in `init()` to enable lighting.

The sky is used for:
1. **Background** — Rendered behind all geometry (replaces clear color)
2. **Ambient lighting** — Provides diffuse ambient term via sky sampling (all lit modes)
3. **Sun direction/color** — Drives sun specular and rim lighting in Modes 2-3

**Algorithm:**
```
sky_gradient = lerp(horizon_color, zenith_color, direction.y * 0.5 + 0.5)
sun_amount = max(0, dot(direction, sun_direction))
sun_contribution = sun_color * pow(sun_amount, sun_sharpness)
final_color = sky_gradient + sun_contribution
```

**Recommended:** Use the same `sun_direction` for both `set_sky()` and `light_direction()` to maintain visual consistency.

**Example presets:**
```rust
// Midday
set_sky(0.7, 0.8, 0.9,  // horizon
        0.3, 0.5, 0.9,  // zenith
        0.3, 0.8, 0.5,  // sun direction (normalized)
        2.0, 1.9, 1.8,  // sun color (HDR)
        200.0);         // sharpness

// Sunset
set_sky(1.0, 0.5, 0.3,  // horizon (warm)
        0.3, 0.1, 0.5,  // zenith (purple)
        0.8, 0.2, 0.0,  // sun direction (low)
        3.0, 1.8, 0.9,  // sun color (orange HDR)
        100.0);         // sharpness (softer)

// Overcast (no sun disc)
set_sky(0.6, 0.6, 0.65, // horizon
        0.4, 0.4, 0.45, // zenith
        0.0, 1.0, 0.0,  // sun direction (doesn't matter)
        0.0, 0.0, 0.0,  // sun color = black (disabled)
        1.0);           // sharpness (irrelevant)
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
| **2 (MR-Blinn-Phong)** | Albedo (UV) | MRE (UV) | Unused | — |
| **3 (Blinn-Phong)** | Albedo (UV) | RSE (UV) | Specular (UV) | — |

**(N) = Normal-sampled (requires `FORMAT_NORMAL`), (UV) = UV-sampled (requires `FORMAT_UV`)**

**Slot Usage Details:**

- **Mode 2, Slot 1 (MRE):** R=Metallic, G=Roughness, B=Emissive. Defaults to uniforms if texture not bound.
- **Mode 2, Slot 2:** Unused (freed up for future features). No texture binding needed.
- **Mode 3, Slot 1 (RSE):** R=Rim intensity, G=Shininess, B=Emissive. Defaults to uniforms if texture not bound.
- **Mode 3, Slot 2 (Specular):** RGB=Specular highlight color. Defaults to uniform if texture not bound.

**Fallback rules:**
- UV-sampled slots with no UVs or no texture → use `set_color()` / `material_*()` uniforms
- Modes 1-3 without `FORMAT_NORMAL` → warning, behaves like Mode 0

**Debug fallback texture:** When a required texture is missing, an 8×8 magenta/black checkerboard is used to make the error visually obvious during development.

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

**Index Format:** Indices are **u16** (16-bit), supporting up to 65,536 vertices per mesh. This is appropriate for fantasy console aesthetics (PS1/N64 era) and provides 50% memory savings compared to u32 indices.

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

**Index Format:** Like retained meshes, indices are **u16** (16-bit) for memory efficiency.

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

### Billboarding (3D Sprites)

Draw camera-facing quads in 3D world space. Useful for particles, foliage, and classic sprite-based characters.

```rust
fn draw_billboard(w: f32, h: f32, mode: u32, color: u32)
fn draw_billboard_region(
    w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    mode: u32, color: u32
)
```

| Mode | Name | Behavior |
|------|------|----------|
| 1 | **Spherical** | Fully faces camera (all axes) |
| 2 | **Cylindrical Y** | Rotates around world Y axis only |
| 3 | **Cylindrical X** | Rotates around world X axis only |
| 4 | **Cylindrical Z** | Rotates around world Z axis only |

**When to use each mode:**
- **Spherical**: Particles, floating UI, explosions — things that should always face you
- **Cylindrical Y**: Trees, characters, signposts — upright objects that rotate to face you but stay vertical
- **Cylindrical X/Z**: Specialized effects (tire tracks, wall decals viewed from angles)

```rust
// Example: Sprite-based character that stays upright
fn render() {
    unsafe {
        texture_bind(CHARACTER_TEXTURE);
        transform_translate(player_x, player_y, player_z);
        draw_billboard(1.0, 2.0, 2, 0xFFFFFFFF);  // 1x2 unit, cylindrical Y
    }
}

// Example: Particle system
fn render_particles() {
    unsafe {
        texture_bind(PARTICLE_TEXTURE);
        for p in &PARTICLES {
            transform_push();
            transform_translate(p.x, p.y, p.z);
            transform_scale(p.size, p.size, 1.0);
            draw_billboard(1.0, 1.0, 1, p.color);  // Spherical
            transform_pop();
        }
    }
}
```

**Notes:**
- Billboards are centered at the current transform origin
- The billboard rotation is applied after the current transform stack
- For cylindrical Y mode, the billboard pivot is at the bottom center (good for characters/trees)

### 2D Drawing (Screen Space)

For UI, HUD, and overlay graphics. Coordinates are in screen pixels (0,0 = top-left). Not affected by camera or 3D transforms.

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

**Texture filtering:**

Controls how textures are sampled when scaled or viewed at an angle:

- `0` = **Nearest** (point sampling) — Sharp, pixelated, retro look
- `1` = **Linear** (bilinear filtering) — Smooth, blurred when scaled

**When to use:**
- **Nearest**: Pixel art, retro aesthetics, UI elements, text (keeps crisp edges)
- **Linear**: 3D models, photographs, natural textures (reduces aliasing)

**Visual differences:**
- Nearest: Each texel maps to a square block of pixels (no interpolation)
- Linear: Texels blend smoothly with neighbors (2×2 interpolation)

**Performance:** Nearest is slightly faster but difference is negligible on modern GPUs. Choose based on visual needs.

**PS1/N64 authenticity:** Use nearest filtering for true 5th-gen look. Linear was available but rarely used.

**Default:** Nearest (0)

**Example:**
```rust
// Pixel art textures
texture_bind(pixel_art_texture);
texture_filter(0);  // Nearest — keep sharp pixels

// Smooth 3D textures
texture_bind(rock_texture);
texture_filter(1);  // Linear — reduce aliasing
```

**Note:** Filter mode applies to the currently bound texture and persists until changed. You can mix filter modes within a single frame by calling `texture_filter()` before each draw.

---

## Custom Fonts

Emberware Z supports custom bitmap fonts for text rendering. By default, `draw_text()` uses a built-in 8×8 monospace font. You can load custom fonts from texture atlases for unique visual styles.

### load_font

```rust
fn load_font(
    texture: u32,
    char_width: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) -> u32
```

Loads a fixed-width bitmap font from a texture atlas. **Must be called in `init()` only.**

**Parameters:**
- `texture`: Texture handle containing the font atlas
- `char_width`: Width of each glyph in pixels (1-255)
- `char_height`: Height of each glyph in pixels (1-255)
- `first_codepoint`: Unicode codepoint of the first glyph (e.g., 32 for space)
- `char_count`: Number of consecutive glyphs in the font

**Returns:** Font handle (use with `font_bind()`)

**Texture atlas layout:**
- Glyphs arranged left-to-right, top-to-bottom in a grid
- Each glyph occupies a `char_width × char_height` cell
- Grid cells are calculated automatically based on texture width
- Atlas width must be divisible by `char_width`

**Example:**
```rust
// Texture: 128×64, glyphs 8×8, ASCII 32-127 (96 chars)
// Layout: 16 glyphs per row (128 / 8), 6 rows needed (96 / 16)
static FONT_PNG: &[u8] = include_bytes!("assets/font.png");

fn init() {
    let font_tex = load_texture(FONT_PNG.as_ptr(), FONT_PNG.len() as u32);
    let font = load_font(
        font_tex,
        8,   // char_width
        8,   // char_height
        32,  // first_codepoint (space)
        96   // char_count (space through tilde)
    );
    font_bind(font);  // Use for all subsequent draw_text()
}
```

---

### load_font_ex

```rust
fn load_font_ex(
    texture: u32,
    widths_ptr: *const u8,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) -> u32
```

Loads a variable-width bitmap font from a texture atlas. Like `load_font()`, but each glyph can have a different width. **Must be called in `init()` only.**

**Parameters:**
- `texture`: Texture handle containing the font atlas
- `widths_ptr`: Pointer to array of `char_count` u8 values (one width per glyph)
- `char_height`: Height of each glyph in pixels (1-255)
- `first_codepoint`: Unicode codepoint of the first glyph
- `char_count`: Number of glyphs in the font

**Returns:** Font handle (use with `font_bind()`)

**Texture atlas layout:**
- Glyphs still arranged in a grid based on the **maximum width** in the widths array
- Each glyph's actual width is read from the widths array
- Unused pixels on the right of narrow glyphs are ignored

**Example:**
```rust
static FONT_PNG: &[u8] = include_bytes!("assets/vfont.png");
static WIDTHS: &[u8] = &[
    4, 2, 5, 6, 6, 8, 7, 2,  // Widths for chars 32-39 (space, !, ", #, ...)
    // ... 88 more widths
];

fn init() {
    let font_tex = load_texture(FONT_PNG.as_ptr(), FONT_PNG.len() as u32);
    let font = load_font_ex(
        font_tex,
        WIDTHS.as_ptr(),
        12,  // char_height
        32,  // first_codepoint (space)
        96   // char_count
    );
    font_bind(font);
}
```

**Use case:** Proportional fonts for better readability and visual polish. "i" can be narrower than "w".

---

### font_bind

```rust
fn font_bind(font_handle: u32)
```

Sets the active font for all subsequent `draw_text()` calls.

**Parameters:**
- `font_handle`: Font handle from `load_font()` or `load_font_ex()`, or `0` for built-in font

**Font 0:** The built-in 8×8 monospace font (default). Supports ASCII printable characters (32-126).

**Example:**
```rust
fn render() {
    font_bind(custom_font);  // Switch to custom font
    draw_text(b"Score: 1000", 10.0, 10.0, 1.0, 0xFFFFFFFF);

    font_bind(0);  // Switch back to built-in font
    draw_text(b"Debug: FPS 60", 10.0, 30.0, 1.0, 0xFF00FFFF);
}
```

**Note:** Font binding persists across frames. Call `font_bind(0)` to reset to the built-in font.

---

### Custom Font Best Practices

**Atlas preparation:**
1. Create your font bitmap in an image editor (each glyph in a grid)
2. Export as PNG or JPEG
3. Embed in WASM with `include_bytes!("font.png")`
4. Load texture and font in `init()`

**Character coverage:**
- Minimum: ASCII printable (32-126, 95 characters)
- Extended: Add accented characters, symbols as needed
- Unicode support: Use Unicode codepoints in `first_codepoint`

**Performance:**
- Font textures are loaded into VRAM once during `init()`
- `draw_text()` generates quads dynamically each frame (immediate mode)
- Keep font atlases reasonably sized (< 512×512 for most cases)
- Variable-width fonts have ~same performance as fixed-width

**Styling:**
- Color: Pass color to `draw_text()`
- Outline/shadow: Pre-bake into font atlas texture
- Size scaling: Use `size` parameter in `draw_text()` (e.g., 2.0 = 2× larger)

---

## Audio FFI

Emberware Z includes a fully functional audio system with support for sound effects and background music.

**Audio Specs:**
- Sample rate: 22,050 Hz (22.05 kHz)
- Format: 16-bit signed PCM, mono
- Channels: 16 simultaneous sound channels (0-15) + dedicated music channel
- All sounds must be embedded as raw PCM data at compile time

### load_sound

```rust
fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32
```

Loads raw PCM sound data and returns a sound handle. **Must be called in `init()` only.**

**Parameters:**
- `data_ptr`: Pointer to raw i16 PCM data in WASM memory
- `byte_len`: Length in **bytes** (must be even, since each sample is 2 bytes)

**Returns:** Sound handle (use with play_sound, channel_play, music_play)

**Example:**
```rust
static JUMP_SFX: &[u8] = include_bytes!("assets/jump.pcm");  // Raw 16-bit PCM mono @ 22,050 Hz

fn init() {
    let jump_sound = load_sound(
        JUMP_SFX.as_ptr() as *const i16,
        JUMP_SFX.len() as u32
    );
}
```

**Note:** Sounds are embedded at compile time. Use tools like Audacity (export as "RAW (header-less)", signed 16-bit PCM, 22050 Hz, mono) or ffmpeg to convert audio files:

```bash
ffmpeg -i input.wav -ar 22050 -ac 1 -f s16le output.pcm
```

---

### play_sound

```rust
fn play_sound(sound: u32, volume: f32, pan: f32)
```

Fire-and-forget sound playback. Plays on the next available channel. Best for one-shot sounds: gunshots, jumps, coins, UI clicks.

**Parameters:**
- `sound`: Sound handle from `load_sound()`
- `volume`: 0.0 to 1.0
- `pan`: -1.0 (left) to 1.0 (right), 0.0 = center

**Example:**
```rust
fn update() {
    if button_pressed(0, BUTTON_A) {
        play_sound(jump_sound, 0.8, 0.0);  // Play at 80% volume, centered
    }
}
```

---

### channel_play

```rust
fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32)
```

Plays sound on a specific channel (0-15). Use for managed channels: positional audio, looping ambient sounds, engine sounds, footsteps.

**Parameters:**
- `channel`: 0-15
- `sound`: Sound handle from `load_sound()`
- `volume`: 0.0 to 1.0
- `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
- `looping`: 1 = loop continuously, 0 = play once

**Example:**
```rust
// Start looping engine sound on channel 0
channel_play(0, engine_sound, 0.5, 0.0, 1);

// Play footstep on channel 1 (one-shot)
channel_play(1, footstep_sound, 0.6, 0.0, 0);
```

---

### channel_set

```rust
fn channel_set(channel: u32, volume: f32, pan: f32)
```

Updates channel volume/pan in real-time. Call every frame for positional audio (update pan based on object position relative to listener).

**Parameters:**
- `channel`: 0-15
- `volume`: 0.0 to 1.0
- `pan`: -1.0 (left) to 1.0 (right), 0.0 = center

**Example:**
```rust
fn render() {
    // Update positional audio for enemy on channel 2
    let listener_x = camera.position.x;
    let enemy_x = enemy.position.x;
    let distance = (enemy_x - listener_x).abs();

    // Simple distance attenuation
    let volume = (1.0 - (distance / 100.0).min(1.0)).max(0.0);

    // Pan based on horizontal offset
    let pan = ((enemy_x - listener_x) / 50.0).clamp(-1.0, 1.0);

    channel_set(2, volume, pan);
}
```

---

### channel_stop

```rust
fn channel_stop(channel: u32)
```

Stops playback on a specific channel.

**Parameters:**
- `channel`: 0-15

**Example:**
```rust
// Stop engine sound when landing
channel_stop(0);
```

---

### music_play

```rust
fn music_play(sound: u32, volume: f32)
```

Plays looping background music on the dedicated music channel. Automatically loops. Only one music track can play at a time.

**Parameters:**
- `sound`: Sound handle from `load_sound()`
- `volume`: 0.0 to 1.0

**Example:**
```rust
fn init() {
    let bgm = load_sound(MUSIC_DATA.as_ptr() as *const i16, MUSIC_DATA.len() as u32);
    music_play(bgm, 0.7);  // Play at 70% volume, looping
}
```

---

### music_stop

```rust
fn music_stop()
```

Stops background music playback.

**Example:**
```rust
// Stop music when entering pause menu
music_stop();
```

---

### music_set_volume

```rust
fn music_set_volume(volume: f32)
```

Adjusts music volume in real-time without restarting playback.

**Parameters:**
- `volume`: 0.0 to 1.0

**Example:**
```rust
// Fade out music
fn update() {
    if fading_out {
        music_volume -= 0.01;
        music_set_volume(music_volume.max(0.0));
    }
}
```

---

### Audio Best Practices

**Channel allocation strategy:**
- Channels 0-7: Looping sounds (engines, ambient, persistent effects)
- Channels 8-15: One-shot sounds (footsteps, impacts, short effects)
- Use `play_sound()` for truly one-shot sounds (gunshots, jumps, coins)
- Use `channel_play()` when you need control (positional audio, looping, manual stop)

**Memory considerations:**
- Sounds are loaded into RAM at startup (16-bit × sample count × 2 bytes)
- A 1-second sound at 22,050 Hz = ~44 KB
- Keep individual sounds short (< 5 seconds for SFX, up to 2 minutes for music)
- Use looping for longer ambient sounds instead of embedding long files

**Positional audio:**
- Emberware Z provides stereo panning only (no HRTF or 3D audio)
- Calculate pan manually: `pan = (object_x - listener_x) / max_distance`
- Clamp pan to -1.0 to 1.0
- Update with `channel_set()` every frame for moving objects

---

## Complete Example

```rust
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
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

---

## Troubleshooting

### Common Issues

#### FFI functions not found

**Symptom:** WASM module fails to instantiate with "unknown import" errors.

**Cause:** Using the wrong import module name.

**Fix:** Use `#[link(wasm_import_module = "env")]` for all FFI imports:

```rust
#[link(wasm_import_module = "env")]
extern "C" {
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}
```

#### Configuration functions ignored

**Symptom:** `set_resolution()`, `set_tick_rate()`, `render_mode()`, or `set_clear_color()` have no effect.

**Cause:** Calling these functions outside of `init()`.

**Fix:** These are init-only functions. Call them only in your `init()` function:

```rust
#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_resolution(2);        // 720p
        set_tick_rate(2);         // 60fps
        render_mode(2);           // MR-Blinn-Phong
        set_clear_color(0x1a1a2eFF);
    }
}
```

#### Textures appear as magenta/black checkerboard

**Symptom:** Geometry renders with a magenta/black checkerboard pattern instead of your texture.

**Cause:** Texture handle is invalid or texture wasn't loaded properly.

**Fix:**
1. Verify `load_texture()` returns a non-zero handle
2. Ensure you call `texture_bind()` before drawing
3. Check that pixel data pointer and dimensions are correct

```rust
let handle = load_texture(width, height, pixels.as_ptr());
if handle == 0 {
    // Handle error - texture failed to load
}
texture_bind(handle);
draw_mesh(mesh);
```

#### Lighting not working in Mode 0

**Symptom:** Objects appear flat/unlit even though you called `set_sky()`.

**Cause:** Vertex format doesn't include normals.

**Fix:** Use a vertex format with `FORMAT_NORMAL` flag:

```rust
// Use format 5 (POS_UV_NORMAL) instead of format 1 (POS_UV)
load_mesh(data.as_ptr(), count, 5);
```

#### Transform stack overflow

**Symptom:** `transform_push()` returns 0.

**Cause:** Stack depth exceeded (max 16 entries).

**Fix:** Ensure every `transform_push()` has a matching `transform_pop()`:

```rust
for obj in objects {
    if transform_push() != 0 {
        transform_translate(obj.x, obj.y, obj.z);
        draw_mesh(obj.mesh);
        transform_pop();
    }
}
```

#### Rollback desync in multiplayer

**Symptom:** Players see different game states during netplay.

**Cause:** Non-deterministic code in `update()`.

**Fix:**
1. Always use `random()` instead of external RNG
2. Avoid floating-point operations that differ across platforms
3. Don't use system time or external state in `update()`
4. Ensure `save_state`/`load_state` capture all game state

#### GPU skinning not deforming mesh

**Symptom:** Skinned mesh renders but bones have no effect.

**Cause:** Bone matrices not uploaded or vertex format incorrect.

**Fix:**
1. Include `FORMAT_SKINNED` (8) in your vertex format
2. Call `set_bones()` before `draw_mesh()` each frame
3. Ensure bone indices in vertices are valid (0-255)
4. Verify bone weights sum to approximately 1.0

---

## Performance Tips

### Vertex Formats

- Use the **smallest vertex format** that meets your needs
- Format 0 (POS only) is 12 bytes, format 7 (POS_UV_COLOR_NORMAL) is 44 bytes
- Adding FORMAT_SKINNED increases stride by 20 bytes per vertex

### Retained vs Immediate Mode

- **Prefer `load_mesh()` + `draw_mesh()`** for static geometry
- Use `draw_triangles()` only for dynamic/procedural geometry
- Retained meshes are stored on GPU; immediate mode uploads each frame

### Batching

- Minimize texture binding changes between draw calls
- Group objects by material/texture when possible
- Use texture atlases to reduce bind calls

### Transform Stack

- Avoid deep nesting of `transform_push()`/`transform_pop()`
- Compute final transforms in `update()` when possible
- Use `transform_set()` for pre-computed matrices

### Render Modes

| Mode | Cost | Use Case |
|------|------|----------|
| 0 (Unlit) | Lowest | UI, particles, stylized games |
| 1 (Matcap) | Low | Stylized 3D, no dynamic lighting needed |
| 2 (MR-Blinn-Phong) | Medium | Physical materials, metallic-roughness workflow |
| 3 (Blinn-Phong) | Medium | Artistic control, rim lighting, explicit specular |

### Texture Guidelines

- Use power-of-two dimensions when possible
- Keep textures small (256×256 or less for retro aesthetic)
- Use nearest-neighbor filtering (`texture_filter(0)`) for sharp pixels
- VRAM budget is 4MB — monitor usage during development

### CPU Budget

- Target 4ms per tick at 60fps (update + save_state overhead)
- Profile your `update()` function if rollback causes stuttering
- Keep game state small for faster save/load during rollback
- Use fixed-point math where precision allows for determinism

### Billboards

- Use billboards for large numbers of similar objects (particles, foliage)
- Cylindrical Y mode is cheaper than spherical for upright sprites
- Batch multiple billboards in a single draw when possible
