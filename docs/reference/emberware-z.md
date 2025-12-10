# Emberware Z — Console Specification

Emberware Z is a 5th-generation fantasy console targeting PS1/N64/Saturn aesthetics with modern conveniences.

## Console Specs

| Spec | Value |
|------|-------|
| **Aesthetic** | PS1/N64/Saturn (5th gen) |
| **Resolution** | 360p, 540p (default), 720p, 1080p |
| **Color depth** | RGBA8 |
| **Tick rate** | 24, 30, 60 (default), 120 fps |
| **Memory** | 8MB unified (code + assets + game state) |
| **VRAM** | 4MB (GPU textures and mesh buffers) |
| **CPU budget** | 4ms per tick (at 60fps) |
| **Netcode** | Deterministic rollback via GGRS |
| **Max players** | 4 (any mix of local + remote) |

### Memory Model

Emberware Z uses a **unified 8MB memory model**. Everything lives in WASM linear memory:
- Compiled game code
- Static data and embedded assets (`include_bytes!`)
- Stack (function calls, local variables)
- Heap (dynamic allocations, game state)

This entire memory is automatically snapshotted for rollback netcode using xxHash3 checksums (~0.5ms per save). Games cannot exceed the 8MB limit — the host enforces this via wasmtime's ResourceLimiter.

**Memory Budget Guidelines:**

| Component | Typical Size | Notes |
|-----------|--------------|-------|
| Code | 50-200 KB | Even complex games |
| Textures (before VRAM upload) | 1-4 MB | Uploaded to GPU in `init()` |
| Audio | 500 KB - 2 MB | Use tracker music for BGM |
| Animations | ~100 KB/character | With keyframe compression |
| Game state | 10-100 KB | Entities, physics, etc. |

**Example: Full fighting game budget (~4.4MB)**
- 8 characters with meshes, textures, animations: ~1.5MB
- 3 stages: ~1MB
- Sound effects: ~650KB
- Music (tracker): ~120KB
- Effects, UI, code: ~1.1MB

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
// Directional lights (default)
fn light_set(index: u32, x: f32, y: f32, z: f32)  // index 0-3, direction vector
fn light_color(index: u32, color: u32)             // 0xRRGGBBAA
fn light_intensity(index: u32, intensity: f32)    // 0.0-8.0 range
fn light_enable(index: u32)
fn light_disable(index: u32)

// Point lights
fn light_set_point(index: u32, x: f32, y: f32, z: f32)  // World-space position
fn light_range(index: u32, range: f32)                   // Falloff distance

// Sun comes from procedural sky (sky_set_sun)
```

**Directional Lights:** The default light type. The `x`, `y`, `z` parameters in `light_set()` specify the direction rays travel (normalized internally). For a light from above, use `(0, -1, 0)`.

**Point Lights:** Emit light from a position in world space with distance-based falloff. Call `light_set_point()` to convert a light slot to a point light at a specific position. Use `light_range()` to control the falloff distance — light intensity reaches zero at this distance.

**Attenuation:** Point lights use smooth quadratic falloff: `(1 - distance/range)²`

**Example: Point Light Setup**
```rust
// Set light 0 as a point light above the player
light_set_point(0, player_x, player_y + 2.0, player_z);
light_color(0, 0xFFAA44FF);  // Warm orange
light_intensity(0, 3.0);      // HDR intensity (0-8 range)
light_range(0, 15.0);         // Light reaches zero at 15 units
```

**Example: Orbiting Point Light**
```rust
fn update() {
    // Orbit point light around the player
    let angle = elapsed_time() * 2.0;  // 2 radians per second
    let orbit_radius = 3.0;
    let px = player_x + sin(angle) * orbit_radius;
    let pz = player_z + cos(angle) * orbit_radius;
    light_set_point(1, px, player_y + 1.5, pz);
}
```

**Intensity Range:** Intensity now uses 0.0-8.0 range for HDR support. Values above 1.0 create brighter-than-expected lights useful for point light falloff.

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
// Directional lights (index 0-3)
fn light_set(index: u32, x: f32, y: f32, z: f32)  // Direction vector
fn light_color(index: u32, color: u32)             // 0xRRGGBBAA
fn light_intensity(index: u32, intensity: f32)    // 0.0-8.0 range
fn light_enable(index: u32)
fn light_disable(index: u32)

// Point lights
fn light_set_point(index: u32, x: f32, y: f32, z: f32)  // World position
fn light_range(index: u32, range: f32)                   // Falloff distance

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

#### Sky Configuration Functions

```rust
fn sky_set_colors(
    horizon_color: u32,  // Horizon color (0xRRGGBBAA)
    zenith_color: u32    // Zenith color (0xRRGGBBAA)
)

fn sky_set_sun(
    dir_x: f32, dir_y: f32, dir_z: f32,  // Sun direction (will be normalized)
    color: u32,                            // Sun color (0xRRGGBBAA)
    sharpness: f32                         // Sun disc sharpness (0.0-1.0)
)

fn draw_sky()
```

**Default:** All zeros (black sky, no sun). Call `sky_set_colors()` and `sky_set_sun()` in `init()` to configure.

#### Sky Usage

The sky can be used for:
1. **Visible Background** — Call `draw_sky()` to render gradient behind all geometry
2. **Ambient lighting** — Sky provides diffuse ambient term (all lit modes, automatic)
3. **Sun direction/color** — Drives sun specular and rim lighting in Modes 2-3

**Algorithm:**
```
sky_gradient = lerp(horizon_color, zenith_color, direction.y * 0.5 + 0.5)
sun_amount = max(0, dot(direction, sun_direction))
sun_contribution = sun_color * pow(sun_amount, sun_sharpness)
final_color = sky_gradient + sun_contribution
```

**Recommended:** Use the same `sun_direction` for sky and lights to maintain visual consistency.

#### Rendering the Sky

**IMPORTANT:** Call `draw_sky()` **FIRST** in your `render()` function, before any 3D geometry:

```rust
fn render() {
    // Step 1: Configure sky colors
    sky_set_colors(
        0xB2D8F2FF,   // Horizon: light blue
        0x3366B2FF    // Zenith: darker blue
    );

    // Step 2: Configure sun
    sky_set_sun(
        0.5, 0.707, 0.5,   // Direction: 45° elevation, southeast
        0xFFF2E6FF,        // Color: warm white
        0.98               // Sharpness: fairly sharp disc
    );

    // Step 3: Draw sky FIRST (before any geometry)
    draw_sky();

    // Step 4: Set up camera and draw scene
    camera_set_perspective(60.0, 16.0 / 9.0, 0.1, 1000.0);
    camera_look_at(
        player_x, player_y + 5.0, player_z - 10.0,
        player_x, player_y, player_z,
        0.0, 1.0, 0.0
    );

    // Draw scene geometry (appears in front of sky)
    draw_mesh(terrain);
    draw_mesh(player);
}
```

**Notes:**
- `draw_sky()` renders a fullscreen gradient at the far plane (always behind geometry)
- Sky rendering is automatic — just configure colors/sun, then call `draw_sky()`
- Works in all render modes (0-3)
- Depth write is disabled, so sky doesn't interfere with depth testing
- Performance: <1ms GPU time at 1080p (single fullscreen triangle)

#### Example Sky Presets

```rust
// Midday
fn init() {
    sky_set_colors(0xB2CDE6FF, 0x4D80E6FF);  // Light blue → mid blue
    sky_set_sun(0.3, 0.8, 0.5, 0xFFF2E6FF, 0.98);  // Warm white sun
}

// Sunset
fn init() {
    sky_set_colors(0xFF804DFF, 0x4D1A80FF);  // Orange → purple
    sky_set_sun(0.8, 0.2, 0.0, 0xFFE673FF, 0.95);  // Golden sun
}

// Overcast (no visible sun)
fn init() {
    sky_set_colors(0x9999A6FF, 0x666673FF);  // Gray gradient
    sky_set_sun(0.0, 1.0, 0.0, 0x000000FF, 0.0);  // No sun
}

// Night Sky
fn init() {
    sky_set_colors(0x0D0D1AFF, 0x03030DFF);  // Dark blue gradient
    sky_set_sun(0.0, -1.0, 0.0, 0x1A1A26FF, 0.5);  // Moon (dim, below horizon)
}
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
fn set_bones(matrices: *const f32, count: u32)  // 12 floats per bone (3×4 matrix, row-major)
```

Call `set_bones()` before `draw_mesh()` or `draw_triangles()` to upload the current bone transforms. Maximum 256 bones per skeleton.

**Matrix layout (row-major, 12 floats per bone):**

```text
[m00, m01, m02, tx]  // row 0: X axis + translation X
[m10, m11, m12, ty]  // row 1: Y axis + translation Y
[m20, m21, m22, tz]  // row 2: Z axis + translation Z
// row 3 [0, 0, 0, 1] is implicit (affine transform)
```

This format saves 25% memory compared to 4×4 matrices (48 bytes vs 64 bytes per bone) while preserving full precision for affine transformations.

**Workflow:**

1. **In `init()`:** Load skinned mesh with bone indices/weights baked into vertex data
2. **Each `update()`:** Animate skeleton on CPU (update bone transforms from keyframes/blend trees)
3. **Each `render()`:** Call `set_bones()` with current transforms, then `draw_mesh()`

**Example:**

```rust
static mut CHARACTER_MESH: u32 = 0;
static mut BONE_MATRICES: [f32; 256 * 12] = [0.0; 256 * 12];  // Up to 256 bones (3x4 format)
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

        // Initialize bone matrices to identity (3x4 row-major)
        for i in 0..BONE_COUNT as usize {
            BONE_MATRICES[i * 12 + 0] = 1.0;  // row0.x (X axis)
            BONE_MATRICES[i * 12 + 5] = 1.0;  // row1.y (Y axis)
            BONE_MATRICES[i * 12 + 10] = 1.0; // row2.z (Z axis)
            // Translation (row0.w, row1.w, row2.w) default to 0.0
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
        set_bones(BONE_MATRICES.as_ptr(), BONE_COUNT);  // Upload 3x4 bone transforms
        transform_translate(0.0, 0.0, -5.0);
        draw_mesh(CHARACTER_MESH);
    }
}
```

**Notes:**
- Bone matrices are world-space (or object-space if you prefer — just be consistent)
- The vertex shader computes: `skinned_pos = Σ(bone_weight[i] * bone_matrix[bone_index[i]] * vertex_pos)`
- Normals are also transformed using the bone rotation (upper 3×3 submatrix)
- For best performance, limit to 4 bones per vertex with normalized weights
- CPU-side animation (keyframes, blend trees, IK) is left to the developer
- Industry standard: Unity, Unreal, and most engines use 3×4 matrices for skeletal animation

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

### Procedural Mesh Generation

Emberware Z provides helper functions to generate common 3D primitives procedurally. These functions generate meshes with proper normals and UV mapping, returning mesh handles that can be drawn with `draw_mesh()`.

All procedural meshes use **vertex format 5** (POS_UV_NORMAL): `[x, y, z, u, v, nx, ny, nz]` — 8 floats per vertex. This format works with all render modes (0-3).

```rust
fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32
fn sphere(radius: f32, segments: u32, rings: u32) -> u32
fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) -> u32
fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32
fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32
fn capsule(radius: f32, height: f32, segments: u32, rings: u32) -> u32
```

All functions must be called in `init()` (meshes are immutable after initialization). Returns mesh handle (>0) on success, 0 on error.

**Examples:**

**Cube:**
```rust
let cube_mesh = cube(1.0, 1.5, 0.5);  // Width=2, Height=3, Depth=1
```
Creates a box with 24 vertices (4 per face) and flat normals. UV coordinates map 0-1 on each face for texture tiling. Parameters are half-extents (not full sizes).

**Sphere:**
```rust
let sphere_mesh = sphere(2.0, 32, 16);  // Radius=2, 32 segments, 16 rings
```
Creates a UV sphere with smooth normals using latitude/longitude grid. Higher segment/ring counts create smoother spheres. Minimum: 3 segments, 2 rings. Maximum: 256 segments/rings.

**Cylinder:**
```rust
let cylinder_mesh = cylinder(1.0, 1.0, 3.0, 24);  // Uniform cylinder
let cone_mesh = cylinder(2.0, 0.0, 3.0, 24);     // Cone (top radius = 0)
```
Creates a cylinder or cone with top/bottom caps. If `radius_top != radius_bottom`, creates a tapered cylinder. Caps are omitted if radius is 0. Minimum: 3 segments. Maximum: 256 segments.

**Plane:**
```rust
let ground = plane(10.0, 10.0, 4, 4);  // 10×10 plane with 4×4 subdivisions
```
Creates a subdivided plane on the XZ plane (Y=0), facing up. Useful for terrain, floors, or quad-based effects. Minimum: 1 subdivision. Maximum: 256 subdivisions per axis.

**Torus:**
```rust
let donut = torus(2.0, 0.5, 48, 24);  // Major radius=2, minor=0.5
```
Creates a torus (donut shape). `major_radius` is the distance from center to tube center, `minor_radius` is the tube thickness. Minimum: 3 segments. Maximum: 256 segments.

**Capsule:**
```rust
let pill = capsule(0.5, 2.0, 16, 8);  // Radius=0.5, cylinder height=2
```
Creates a capsule (cylinder with hemispherical ends). Total height = `height + 2 * radius`. Useful for character colliders or pill-shaped objects. If `height` is 0, generates a sphere. Minimum: 3 segments, 1 ring. Maximum: 256 segments, 128 rings.

**UV Mapping:**
- **Cube:** Box unwrap (0-1 per face)
- **Sphere:** Equirectangular (latitude/longitude)
- **Cylinder:** Radial unwrap for body, polar projection for caps
- **Plane:** Simple 0-1 grid
- **Torus:** Wrapped both axes
- **Capsule:** Radial for body, polar for hemispheres

**Normal Generation:**
- **Cube:** Flat normals (one per face)
- **Sphere, Torus, Capsule:** Smooth normals
- **Cylinder:** Smooth for body, flat for caps
- **Plane:** Flat (pointing up)

**Performance Note:** All geometry generation happens on the CPU during `init()`. The generated vertices are uploaded to GPU buffers and stored as retained meshes. Generation is deterministic and safe for rollback netcode.

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

## Stereo Panning

Emberware Z supports stereo panning for all sound playback. Sounds are loaded as mono (single channel) but can be positioned in the stereo field during playback.

### How Panning Works

**Pan Range:** -1.0 (full left) to 1.0 (full right), 0.0 = center

**Equal-Power Panning:** Uses constant-power law to maintain perceived loudness across the stereo field:
- `pan = -1.0`: 100% left speaker, 0% right speaker
- `pan = 0.0`: 70.7% left speaker, 70.7% right speaker (center, -3dB each)
- `pan = +1.0`: 0% left speaker, 100% right speaker

This ensures sounds don't become quieter when centered, providing smooth and natural stereo positioning.

### Positional Audio Example

```rust
fn update() {
    // Calculate 2D positional audio for enemy footsteps
    let listener_x = player.position.x;
    let enemy_x = enemy.position.x;
    let offset = enemy_x - listener_x;

    // Pan: -1.0 (left) when enemy is 50+ units left, +1.0 (right) when 50+ units right
    let pan = (offset / 50.0).clamp(-1.0, 1.0);

    // Volume falloff: silent at 100+ units distance
    let distance = offset.abs();
    let volume = (1.0 - (distance / 100.0).min(1.0)).max(0.0);

    channel_set(0, volume, pan);  // Update channel 0 every frame
}
```

### Limitations

- **Pan changes on already-playing sounds:** When using `channel_set()` to update pan on a currently-playing sound, the new pan value is stored but won't take effect until the sound finishes or is restarted. This is a limitation of the underlying audio system.
- **Workaround:** For real-time pan changes, use short looping sounds or restart the sound with `channel_play()` when pan changes significantly.

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
