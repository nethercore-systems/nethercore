# Multi-Environment System Specification (v3 - Complete)

**Status:** Ready for Implementation
**Author:** Zerve
**Version:** 3.0
**Last Updated:** December 2024

---

## Summary

Add a procedural environment rendering system with **8 distinct algorithms** (modes) that can be **layered** (base + overlay) with configurable **blend modes**. Each mode generates environment colors from a direction vector, enabling dynamic skies, weather effects, architectural backgrounds, and more.

---

## Design Overview

```
┌─────────────────────────────────────────────────────────┐
│                    Environment System                    │
├─────────────────────────────────────────────────────────┤
│  8 Procedural Algorithms (modes 0-7)                    │
│  ┌─────────┬─────────┬─────────┬─────────┐             │
│  │Gradient │ Scatter │  Lines  │Silhouette│             │
│  │  (0)    │   (1)   │   (2)   │   (3)    │             │
│  ├─────────┼─────────┼─────────┼─────────┤             │
│  │Rectangles│  Room  │Curtains │  Rings  │             │
│  │   (4)   │   (5)   │   (6)   │   (7)   │             │
│  └─────────┴─────────┴─────────┴─────────┘             │
├─────────────────────────────────────────────────────────┤
│  Layering: base_mode + overlay_mode + blend_mode        │
│  Blend: Alpha | Add | Multiply | Screen                 │
├─────────────────────────────────────────────────────────┤
│  Storage: Immediate-mode (48 bytes per unique config)   │
│  Animation: u16 phase (0-65535), seamless looping       │
└─────────────────────────────────────────────────────────┘
```

---

## Current Architecture (Before This Change)

**PackedSky (16 bytes):** Embedded in each `PackedUnifiedShadingState`
```rust
pub struct PackedSky {
    pub horizon_color: u32,           // RGBA8
    pub zenith_color: u32,            // RGBA8
    pub sun_direction_oct: u32,       // Octahedral (snorm16x2)
    pub sun_color_and_sharpness: u32, // RGB8 + sharpness u8
}
```

**PackedUnifiedShadingState:** 96 bytes per shading state
- Header: 16 bytes (color, uniforms, flags)
- `sky: PackedSky`: 16 bytes (duplicated per state!)
- `lights: [PackedLight; 4]`: 48 bytes
- Animation fields: 16 bytes

**Current FFI ([sky.rs](../../emberware-z/src/ffi/sky.rs)):**
- `sky_set_colors(horizon: u32, zenith: u32)` — 2-color gradient only
- `sky_set_sun(dir_x, dir_y, dir_z, color, sharpness)` — Sun disc + provides light direction
- `draw_sky()` — Renders fullscreen sky quad

**Current @group(0) bindings:**
| Binding | Buffer | Location |
|---------|--------|----------|
| 0 | unified_transforms | common.wgsl |
| 1 | mvp_shading_indices | common.wgsl |
| 2 | shading_states | common.wgsl |
| 3 | unified_animation | common.wgsl |
| 4 | quad_instances | quad_template.wgsl only |

**Key Problem:** Sky is duplicated in every shading state (wastes memory, inflexible).

---

## Important: Environment vs Sun/Lighting

The **environment system** (this spec) handles **visual backgrounds only**:
- Sky gradients, stars, weather effects, architectural backdrops
- What you see when looking at the "sky sphere"
- Purely visual — no direct lighting contribution

The **sun/lighting system** (existing) handles **scene illumination**:
- `sky_set_sun()` defines the primary light direction for PBR/Hybrid modes
- `light_set()` / `light_set_point()` for additional lights
- Affects diffuse/specular on 3D geometry

These systems are **separate concerns**. Games will typically:
1. Call `env_gradient_set()` for visual sky appearance
2. Call `sky_set_sun()` for lighting direction (kept separate)

---

## Mode Reference

### Mode 0: Gradient
Four-color sky/ground gradient with independent transitions.

| Parameter | Type | Description |
|-----------|------|-------------|
| zenith | RGBA8 | Top of sky (straight up) |
| sky_horizon | RGBA8 | Where sky meets horizon |
| ground_horizon | RGBA8 | Where ground meets horizon |
| nadir | RGBA8 | Bottom of ground (straight down) |
| rotation | f16 | Spins gradient around vertical axis (degrees) |
| shift | f16 | Moves horizon line up/down (-1 to +1) |

**Use cases:** Outdoor skies, sunsets, underwater, alien atmospheres.

---

### Mode 1: Scatter
Cellular noise particle field with parallax layers.

| Parameter | Type | Description |
|-----------|------|-------------|
| variant | u8 | 0=Stars, 1=Vertical, 2=Horizontal, 3=Warp |
| density | u8 | Particle count (0-255) |
| size | u8 | Particle size |
| glow | u8 | Glow/bloom intensity |
| streak_length | u8 | Elongation (0=points) |
| color_primary | RGB8 | Main particle color |
| color_secondary | RGB8 | Variation color |
| parallax_rate | u8 | Layer separation (0=flat, 255=extreme) |
| parallax_size | u8 | Size variation with depth |
| phase | u16 | Animation phase (0-65535, wraps naturally) |

**Animation:** Game code increments `phase` each frame (wraps at 65535 → 0):
- Rain falling: `phase = phase.wrapping_add((delta_time * speed * 65535.0) as u16)`
- Stars twinkling: `phase = phase.wrapping_add((delta_time * 0.1 * 65535.0) as u16)`

**Perfect looping:** Shader converts phase to 0.0-1.0 range. Algorithms designed so phase=0 and phase=65535 produce identical output.

**Variants:**
- **Stars (0):** Static twinkling points
- **Vertical (1):** Rain/snow streaks
- **Horizontal (2):** Speed lines
- **Warp (3):** Radial expansion from center

**Use cases:** Space, weather, motion blur, hyperspace.

---

### Mode 2: Lines
Infinite grid lines projected onto a plane.

| Parameter | Type | Description |
|-----------|------|-------------|
| variant | u8 | 0=Floor, 1=Ceiling, 2=Sphere |
| line_type | u8 | 0=Horizontal, 1=Vertical, 2=Grid |
| thickness | u8 | Line thickness |
| spacing | f16 | Distance between lines |
| fade_distance | f16 | Distance fade start |
| color_primary | RGBA8 | Main line color |
| color_accent | RGBA8 | Accent line color |
| accent_every | u8 | Accent every Nth line |
| phase | u16 | Scroll phase (0-65535, wraps naturally) |

**Animation:** Game code increments `phase` for scrolling grid (wraps at 65535 → 0):
- Racing game: `phase = phase.wrapping_add((velocity.z * delta_time * 65535.0) as u16)`
- Synthwave: `phase = phase.wrapping_add((delta_time * 2.0 * 65535.0) as u16)`

**Perfect looping:** Grid scrolls one full "spacing" distance per phase cycle.

**Use cases:** Synthwave aesthetic, racing games, holographic/digital.

---

### Mode 3: Silhouette
Layered terrain silhouettes with parallax.

| Parameter | Type | Description |
|-----------|------|-------------|
| jaggedness | u8 | 0=smooth hills, 255=sharp peaks |
| layer_count | u8 | Number of depth layers (1-3) |
| color_near | RGBA8 | Nearest silhouette color |
| color_far | RGBA8 | Farthest silhouette color |
| sky_zenith | RGBA8 | Sky behind silhouettes |
| sky_horizon | RGBA8 | Horizon behind silhouettes |
| parallax_rate | u8 | Layer separation amount |
| seed | u32 | Noise seed for terrain shape |

**Use cases:** Mountain ranges, city horizons, forests.

---

### Mode 4: Rectangles
Rectangular light sources - windows, screens, panels.

| Parameter | Type | Description |
|-----------|------|-------------|
| variant | u8 | 0=Scatter, 1=Buildings, 2=Bands, 3=Panels |
| density | u8 | How many rectangles |
| lit_ratio | u8 | Percentage of rectangles lit (0-255) |
| size_min | u8 | Minimum rectangle size |
| size_max | u8 | Maximum rectangle size |
| aspect | u8 | Aspect ratio bias |
| color_primary | RGBA8 | Main window color |
| color_variation | RGBA8 | Color variation |
| phase | u16 | Flicker phase (0-65535, wraps naturally) |
| parallax_rate | u8 | Layer separation (scatter mode) |

**Animation:** Game code increments `phase` for window flicker (wraps at 65535 → 0):
- Slow flicker: `phase = phase.wrapping_add((delta_time * 0.5 * 65535.0) as u16)`
- Fast strobe: `phase = phase.wrapping_add((delta_time * 10.0 * 65535.0) as u16)`

**Perfect looping:** Flicker pattern repeats seamlessly at phase wrap.

**Use cases:** City at night, spaceship interior, control rooms.

---

### Mode 5: Room
Interior of a 3D box with directional lighting.

| Parameter | Type | Description |
|-----------|------|-------------|
| color_ceiling | RGBA8 | Ceiling color |
| color_floor | RGBA8 | Floor color |
| color_walls | RGBA8 | Wall color |
| panel_size | f16 | Tile/panel pattern size |
| panel_gap | u8 | Gap between panels |
| light_direction | u16 | Octahedral-encoded light dir |
| light_intensity | u8 | Directional light strength |
| corner_darken | u8 | Corner/edge darkening |
| room_scale | f16 | Room repeat distance |
| viewer_x | i16 | Viewer X (-32768 to +32767 = -1.0 to +1.0 in room) |
| viewer_y | i16 | Viewer Y (-32768 to +32767 = -1.0 to +1.0 in room) |
| viewer_z | i16 | Viewer Z (-32768 to +32767 = -1.0 to +1.0 in room) |

**Position:** Viewer position is snorm16x3, where `(0, 0, 0)` = center of room:
```rust
// Normalize player position to room bounds (-1 to +1)
let norm_x = (player.x / room_half_size).clamp(-1.0, 1.0);
let norm_y = (player.y / room_half_size).clamp(-1.0, 1.0);
let norm_z = (player.z / room_half_size).clamp(-1.0, 1.0);
env_room_set(...,
    (norm_x * 32767.0) as i32,
    (norm_y * 32767.0) as i32,
    (norm_z * 32767.0) as i32,
);
```

**Use cases:** Indoor environments, corridors, hangars, dungeons.

---

### Mode 6: Curtains
Vertical structures (pillars, trees) arranged around viewer.

| Parameter | Type | Description |
|-----------|------|-------------|
| layer_count | u8 | Depth layers (1-3) |
| density | u8 | Structures per cell |
| height_min | u8 | Minimum height |
| height_max | u8 | Maximum height |
| width | u8 | Structure width |
| spacing | u8 | Gap between structures |
| waviness | u8 | Organic wobble (0=straight) |
| color_near | RGBA8 | Nearest structure color |
| color_far | RGBA8 | Farthest structure color |
| glow | u8 | Neon/magical glow |
| parallax_rate | u8 | Layer separation |
| phase | u16 | Horizontal scroll phase (0-65535, wraps naturally) |

**Animation:** Game code increments `phase` for side-scrolling parallax (wraps at 65535 → 0):
- Running through forest: `phase = phase.wrapping_add((velocity.x * delta_time * 65535.0) as u16)`

**Perfect looping:** Structures repeat seamlessly when phase wraps.

**Use cases:** Forests, colonnades, bamboo, prison bars, neon tubes.

---

### Mode 7: Rings
Concentric rings around focal direction (tunnel/portal/vortex).

| Parameter | Type | Description |
|-----------|------|-------------|
| ring_count | u8 | Number of rings |
| thickness | u8 | Ring thickness |
| color_a | RGBA8 | First alternating color |
| color_b | RGBA8 | Second alternating color |
| center_color | RGBA8 | Bright center color |
| center_falloff | u8 | Center glow falloff |
| spiral_twist | f16 | Spiral rotation (0=concentric) |
| axis_direction | u16 | Octahedral-encoded ring axis |
| phase | u16 | Rotation phase (0-65535 = 0°-360°, wraps naturally) |

**Animation:** Game code increments `phase` for spinning rings (wraps at 65535 → 0):
- Portal spin: `phase = phase.wrapping_add((delta_time * 2.0 * 65535.0) as u16)`
- Hypnotic spiral: `phase = phase.wrapping_add((delta_time * 5.0 * 65535.0) as u16)`

**Perfect looping:** Rings complete exactly one full rotation when phase wraps (0°→360°→0°).

**Use cases:** Tunnels, portals, targets, hypnotic spirals.

---

## Layering System

Two environments render simultaneously:
- **Base layer:** Fills entire sphere
- **Overlay layer:** Composited on top

**Blend Modes:**
| Mode | Value | Formula |
|------|-------|---------|
| Alpha | 0 | `lerp(base, overlay, overlay.a)` |
| Add | 1 | `base + overlay` |
| Multiply | 2 | `base * overlay` |
| Screen | 3 | `1 - (1-base) * (1-overlay)` |

**Examples:**
- Silhouette + Scatter = Rainy mountains
- Room + Rectangles = Control room with screens
- Gradient + Curtains = Forest with sky

---

## Storage Design

Follows the same immediate-mode pattern as transforms and shading states:
- Growable `Vec` pool with HashMap deduplication
- Index stored in `PackedUnifiedShadingState`
- Reset each frame in `clear_frame()`

### PackedEnvironmentState (GPU-uploadable, hashable)

```rust
/// Complete environment configuration for one draw's environment
/// Size: 48 bytes (must be POD, hashable for deduplication)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedEnvironmentState {
    /// Header (4 bytes)
    /// bits 0-2:   base_mode (0-7)
    /// bits 3-5:   overlay_mode (0-7)
    /// bits 6-7:   blend_mode (0-3)
    /// bits 8-31:  reserved (flags, future use)
    pub header: u32,

    /// Mode parameters (44 bytes = 11 u32s)
    /// Base mode uses data[0..5] (20 bytes)
    /// Overlay mode uses data[5..10] (20 bytes)
    /// data[10] = shared/overflow
    pub data: [u32; 11],
}
// Total: 4 + 44 = 48 bytes
```

### Bit-Packed Parameter Layouts (20 bytes per mode)

Each mode gets 5 u32s (20 bytes). Parameters are bit-packed for efficiency.

**Mode 0: Gradient**
```
data[0]: zenith (RGBA8)
data[1]: sky_horizon (RGBA8)
data[2]: ground_horizon (RGBA8)
data[3]: nadir (RGBA8)
data[4]: rotation(f16, bits 0-15) + shift(f16, bits 16-31)
```

**Mode 1: Scatter**
```
data[0]: variant(2) + density(8) + size(8) + glow(8) + streak_len(6) = 32 bits
data[1]: color_primary(RGB8, 24) + parallax_rate(8) = 32 bits
data[2]: color_secondary(RGB8, 24) + parallax_size(8) = 32 bits
data[3]: phase(u16, 16) + layer_count(2) + reserved(14) = 32 bits
         ^ Game increments, wraps at 65535→0 for seamless looping
data[4]: reserved
```

**Mode 2: Lines**
```
data[0]: variant(2) + line_type(2) + thickness(8) + accent_every(8) + reserved(12) = 32 bits
data[1]: spacing(f16) + fade_distance(f16) = 32 bits
data[2]: color_primary (RGBA8)
data[3]: color_accent (RGBA8)
data[4]: phase(u16) + reserved(16)
         ^ Game increments, wraps at 65535→0 for seamless grid scrolling
```

**Mode 3: Silhouette**
```
data[0]: jaggedness(8) + layer_count(2) + parallax_rate(8) + reserved(14) = 32 bits
data[1]: color_near (RGBA8)
data[2]: color_far (RGBA8)
data[3]: sky_zenith (RGBA8)
data[4]: sky_horizon(RGB8, 24) + seed_low(8), seed continues in shared
```

**Mode 4: Rectangles**
```
data[0]: variant(2) + density(8) + lit_ratio(8) + size_min(6) + size_max(6) + aspect(2) = 32 bits
data[1]: color_primary (RGBA8)
data[2]: color_variation (RGBA8)
data[3]: parallax_rate(8) + reserved(8) + phase(u16) = 32 bits
         ^ Game increments, wraps at 65535→0 for seamless flicker
data[4]: reserved
```

**Mode 5: Room**
```
data[0]: color_ceiling (RGBA8)
data[1]: color_floor (RGBA8)
data[2]: color_walls (RGBA8)
data[3]: panel_size(f16) + panel_gap(8) + corner_darken(8) = 32 bits
data[4]: light_dir_oct(16) + light_intensity(8) + room_scale(8) = 32 bits
         Uses shared data[10] for viewer_x(snorm16) + viewer_y(snorm16)
         viewer_z(snorm16) stored in lower 16 bits of another shared slot
         snorm16: -32768 to +32767 maps to -1.0 to +1.0, center = 0
```

**Mode 6: Curtains**
```
data[0]: layer_count(2) + density(8) + height_min(6) + height_max(6) + width(5) + spacing(5) = 32 bits
data[1]: waviness(8) + glow(8) + parallax_rate(8) + reserved(8) = 32 bits
data[2]: color_near (RGBA8)
data[3]: color_far (RGBA8)
data[4]: phase(u16) + reserved(16)
         ^ Game increments, wraps at 65535→0 for seamless parallax
```

**Mode 7: Rings**
```
data[0]: ring_count(8) + thickness(8) + center_falloff(8) + reserved(8) = 32 bits
data[1]: color_a (RGBA8)
data[2]: color_b (RGBA8)
data[3]: center_color (RGBA8)
data[4]: spiral_twist(f16) + axis_oct(16) = 32 bits
         Uses shared data[10] upper 16 bits for phase(u16)
         ^ Game increments, wraps at 65535→0 for seamless rotation (0°→360°)
```

### CPU State (ffi_state.rs)

```rust
// Pool of unique environment states (deduplicated)
pub environment_states: Vec<PackedEnvironmentState>,
pub environment_state_map: HashMap<PackedEnvironmentState, EnvironmentIndex>,

// Current staging state (modified by env_* FFI calls)
pub current_environment_state: PackedEnvironmentState,
pub environment_dirty: bool,
```

### Index in Shading State

```rust
/// Newtype for type safety
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EnvironmentIndex(pub u32);

// In PackedUnifiedShadingState (replaces _animation_reserved)
pub struct PackedUnifiedShadingState {
    // ... existing fields ...
    pub environment_index: u32,  // Index into environment_states buffer
}
```

### GPU Buffer

```rust
// @group(0) @binding(4) - growable, uploaded each frame
var<storage, read> environment_states: array<PackedEnvironmentState>;
```

### Deduplication Pattern (same as shading states)

```rust
pub fn add_environment_state(&mut self) -> EnvironmentIndex {
    // Fast path: not dirty, reuse last index
    if !self.environment_dirty && !self.environment_states.is_empty() {
        return EnvironmentIndex(self.environment_states.len() as u32 - 1);
    }

    // Dedup path: check if state already exists
    if let Some(&existing_idx) = self.environment_state_map.get(&self.current_environment_state) {
        self.environment_dirty = false;
        return existing_idx;
    }

    // New path: add to pool
    let idx = self.environment_states.len() as u32;
    let env_idx = EnvironmentIndex(idx);
    self.environment_states.push(self.current_environment_state);
    self.environment_state_map.insert(self.current_environment_state, env_idx);
    self.environment_dirty = false;

    env_idx
}
```

### Frame Reset

```rust
// In clear_frame()
self.environment_states.clear();
self.environment_state_map.clear();
self.current_environment_state = PackedEnvironmentState::default();
self.environment_dirty = true;
```

---

## FFI Functions

### Mode Configuration
```rust
// Mode 0: Gradient
fn env_gradient_set(
    zenith: u32,         // RGBA8
    sky_horizon: u32,    // RGBA8
    ground_horizon: u32, // RGBA8
    nadir: u32,          // RGBA8
    rotation: f32,       // degrees
    shift: f32,          // -1 to +1
);

// Mode 1: Scatter
fn env_scatter_set(
    variant: u32,        // 0-3
    density: u32,        // 0-255
    size: u32,
    glow: u32,
    streak_length: u32,
    color_primary: u32,  // RGB8
    color_secondary: u32,
    parallax_rate: u32,
    parallax_size: u32,
    phase: u32,          // 0-65535, wraps naturally for seamless looping
);

// Mode 2: Lines
fn env_lines_set(
    variant: u32,        // 0=floor, 1=ceiling, 2=sphere
    line_type: u32,      // 0=h, 1=v, 2=grid
    thickness: u32,
    spacing: f32,
    fade_distance: f32,
    color_primary: u32,
    color_accent: u32,
    accent_every: u32,
    phase: u32,          // 0-65535, wraps naturally for seamless scrolling
);

// Mode 3: Silhouette
fn env_silhouette_set(
    jaggedness: u32,
    layer_count: u32,
    color_near: u32,
    color_far: u32,
    sky_zenith: u32,
    sky_horizon: u32,
    parallax_rate: u32,
    seed: u32,
);

// Mode 4: Rectangles
fn env_rectangles_set(
    variant: u32,        // 0-3
    density: u32,
    lit_ratio: u32,
    size_min: u32,
    size_max: u32,
    aspect: u32,
    color_primary: u32,
    color_variation: u32,
    parallax_rate: u32,
    phase: u32,          // 0-65535, wraps naturally for seamless flicker
);

// Mode 5: Room
fn env_room_set(
    color_ceiling: u32,
    color_floor: u32,
    color_walls: u32,
    panel_size: f32,
    panel_gap: u32,
    light_dir_x: f32,
    light_dir_y: f32,
    light_dir_z: f32,
    light_intensity: u32,
    corner_darken: u32,
    room_scale: f32,
    viewer_x: i32,       // snorm16: -32768 to +32767 = -1.0 to +1.0, 0 = center
    viewer_y: i32,       // snorm16: -32768 to +32767 = -1.0 to +1.0, 0 = center
    viewer_z: i32,       // snorm16: -32768 to +32767 = -1.0 to +1.0, 0 = center
);

// Mode 6: Curtains
fn env_curtains_set(
    layer_count: u32,
    density: u32,
    height_min: u32,
    height_max: u32,
    width: u32,
    spacing: u32,
    waviness: u32,
    color_near: u32,
    color_far: u32,
    glow: u32,
    parallax_rate: u32,
    phase: u32,          // 0-65535, wraps naturally for seamless parallax
);

// Mode 7: Rings
fn env_rings_set(
    ring_count: u32,
    thickness: u32,
    color_a: u32,
    color_b: u32,
    center_color: u32,
    center_falloff: u32,
    spiral_twist: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,          // 0-65535, wraps naturally for seamless rotation (0°→360°)
);
```

### Selection & Blending
```rust
/// Select base and overlay modes (same = no overlay)
fn env_select_pair(base_mode: u32, overlay_mode: u32);

/// Set blend mode: 0=alpha, 1=add, 2=multiply, 3=screen
fn env_blend_mode(mode: u32);
```

### Backwards Compatibility
```rust
// Legacy sky_set_colors() maps to 2-color gradient (horizon→sky_horizon, zenith→zenith)
// Ground colors default to darker versions of sky colors
fn sky_set_colors(horizon: u32, zenith: u32);
// → Internally: env_gradient_set(zenith, horizon, darken(horizon), darken(zenith), 0.0, 0.0)
// → Also: env_select_pair(0, 0) to ensure Gradient mode is active

// sky_set_sun() is UNCHANGED - it controls lighting, not environment visuals
fn sky_set_sun(dir_x, dir_y, dir_z, color, sharpness);
// → Still sets sun direction for PBR/Hybrid lighting
// → Sun disc in sky can be rendered by Gradient mode using sun_direction from lighting

// draw_sky() continues to work - renders current environment
fn draw_sky();
```

---

## Shader Architecture

### WGSL Structures

```wgsl
struct PackedEnvironmentState {
    // Header (first u32)
    // bits 0-2:  base_mode
    // bits 3-5:  overlay_mode
    // bits 6-7:  blend_mode
    // bits 8-31: reserved
    header: u32,

    // Mode parameters (11 u32s = 44 bytes)
    // Base mode: data[0..5], Overlay mode: data[5..10], Shared: data[10]
    data: array<u32, 11>,
}
// Total: 48 bytes

// Binding for environment states pool
@group(0) @binding(4) var<storage, read> environment_states: array<PackedEnvironmentState>;
```

### Environment Sampling Entry Point
```wgsl
// Note: No time parameter - animation is controlled via explicit offsets in data
fn sample_environment(env_index: u32, direction: vec3<f32>) -> vec4<f32> {
    let env = environment_states[env_index];

    // Unpack header (bit-packed)
    let base_mode = env.header & 0x7u;           // bits 0-2
    let overlay_mode = (env.header >> 3u) & 0x7u; // bits 3-5
    let blend_mode = (env.header >> 6u) & 0x3u;   // bits 6-7

    let base_color = sample_mode(base_mode, env.data, 0u, direction);

    if (overlay_mode == base_mode) {
        return base_color;
    }

    let overlay_color = sample_mode(overlay_mode, env.data, 5u, direction);
    return blend_layers(base_color, overlay_color, blend_mode);
}

// Each mode function takes the data array and offset into it
// Animation uses phase (u16, 0-65535) stored IN the data array
// Shader converts: let t = f32(phase) / 65535.0;  // 0.0 to 1.0
// Algorithms designed so t=0.0 and t=1.0 produce identical output (seamless loop)
fn sample_mode(mode: u32, data: array<u32, 11>, offset: u32, dir: vec3<f32>) -> vec4<f32> {
    switch (mode) {
        case 0u: { return sample_gradient(data, offset, dir); }
        case 1u: { return sample_scatter(data, offset, dir); }     // phase in data[3]
        case 2u: { return sample_lines(data, offset, dir); }       // phase in data[4]
        case 3u: { return sample_silhouette(data, offset, dir); }
        case 4u: { return sample_rectangles(data, offset, dir); }  // phase in data[3]
        case 5u: { return sample_room(data, offset, dir); }        // viewer_pos from shared data
        case 6u: { return sample_curtains(data, offset, dir); }    // phase in data[4]
        case 7u: { return sample_rings(data, offset, dir); }       // phase in shared data[10]
        default: { return vec4<f32>(1.0, 0.0, 1.0, 1.0); }  // Magenta = unimplemented
    }
}

fn blend_layers(base: vec4<f32>, overlay: vec4<f32>, mode: u32) -> vec4<f32> {
    switch (mode) {
        case 0u: { return mix(base, overlay, overlay.a); }  // Alpha
        case 1u: { return base + overlay; }                  // Add
        case 2u: { return base * overlay; }                  // Multiply
        case 3u: { return vec4(1.0) - (vec4(1.0) - base) * (vec4(1.0) - overlay); } // Screen
        default: { return base; }
    }
}
```

### Per-Mode Functions (example: Gradient)
```wgsl
// Gradient uses data[offset+0..offset+5] (5 u32s = 20 bytes)
fn sample_gradient(data: array<u32, 11>, offset: u32, dir: vec3<f32>) -> vec4<f32> {
    let zenith = unpack_rgba8(data[offset + 0u]);
    let sky_horizon = unpack_rgba8(data[offset + 1u]);
    let ground_horizon = unpack_rgba8(data[offset + 2u]);
    let nadir = unpack_rgba8(data[offset + 3u]);
    let rotation_shift = unpack2x16float(data[offset + 4u]);

    // Apply rotation around Y axis
    let rotated = rotate_y(dir, rotation_shift.x);

    // Calculate blend factor with shift
    let y = rotated.y + rotation_shift.y;

    if (y >= 0.0) {
        return mix(sky_horizon, zenith, y);
    } else {
        return mix(ground_horizon, nadir, -y);
    }
}
```

---

## Debug Inspector Integration

### Example Debug Panel Setup (in game init)
```rust
// Environment Debug Group
debug_group_begin(b"Environment\0".as_ptr());

debug_register_u32(b"Base Mode\0".as_ptr(), addr_of!(base_mode), 0, 7);
debug_register_u32(b"Overlay Mode\0".as_ptr(), addr_of!(overlay_mode), 0, 7);
debug_register_u32(b"Blend Mode\0".as_ptr(), addr_of!(blend_mode), 0, 3);

debug_group_begin(b"Gradient (0)\0".as_ptr());
debug_register_color(b"Zenith\0".as_ptr(), addr_of!(gradient_zenith));
debug_register_color(b"Sky Horizon\0".as_ptr(), addr_of!(gradient_sky_horizon));
debug_register_color(b"Ground Horizon\0".as_ptr(), addr_of!(gradient_ground_horizon));
debug_register_color(b"Nadir\0".as_ptr(), addr_of!(gradient_nadir));
debug_register_f32(b"Rotation\0".as_ptr(), addr_of!(gradient_rotation), 0.0, 360.0);
debug_register_f32(b"Shift\0".as_ptr(), addr_of!(gradient_shift), -1.0, 1.0);
debug_group_end();

debug_group_begin(b"Scatter (1)\0".as_ptr());
debug_register_u32(b"Variant\0".as_ptr(), addr_of!(scatter_variant), 0, 3);
debug_register_u32(b"Density\0".as_ptr(), addr_of!(scatter_density), 0, 255);
// ... etc
debug_group_end();

debug_group_end(); // Environment
```

### Test Harness Pattern
```rust
// In render():
if debug_changed() {
    // Re-apply all environment settings from debug values
    env_gradient_set(gradient_zenith, gradient_sky_horizon, ...);
    env_scatter_set(scatter_variant, scatter_density, ...);
    env_select_pair(base_mode, overlay_mode);
    env_blend_mode(blend_mode);
}
```

---

## Implementation Plan

### Phase 1: Infrastructure + Gradient Mode
**Goal:** Working immediate-mode environment system with one mode to validate design.

1. **Rust Structs** (`unified_shading_state.rs`)
   - Add `PackedEnvironmentState` (48 bytes, POD, Hash, Eq)
   - Add `EnvironmentIndex` newtype
   - Add bit-pack/unpack helpers for each mode's parameters
   - Remove `sky: PackedSky` from `PackedUnifiedShadingState`
   - Replace `_animation_reserved` with `environment_index: u32`
   - Update size assertions (96 → 84 bytes? verify alignment)

2. **CPU State** (`ffi_state.rs`)
   - Add `environment_states: Vec<PackedEnvironmentState>`
   - Add `environment_state_map: HashMap<PackedEnvironmentState, EnvironmentIndex>`
   - Add `current_environment_state: PackedEnvironmentState`
   - Add `environment_dirty: bool`
   - Add `add_environment_state()` with 3-tier dedup pattern
   - Update `clear_frame()` to reset environment pool

3. **GPU Buffer** (`graphics/`)
   - Add `environment_states_buffer: wgpu::Buffer` (growable)
   - Add @binding(4) to bind group layout
   - Renumber quad_instances to @binding(5)
   - Upload `environment_states` Vec each frame

4. **Shaders**
   - Add `PackedEnvironmentState` struct to `common.wgsl`
   - Add @binding(4) environment_states buffer
   - Add `sample_environment(env_index, dir, time)` entry point
   - Add `blend_layers()` function
   - Implement `sample_gradient()` only
   - Other modes return magenta (unimplemented marker)

5. **FFI Functions**
   - `env_gradient_set(...)` → writes to `current_environment_state.data[0..5]`
   - `env_select_pair(base, overlay)` → sets base_mode, overlay_mode
   - `env_blend_mode(mode)` → sets blend_mode
   - All set `environment_dirty = true`
   - Update `add_shading_state()` to call `add_environment_state()` and store index
   - Update legacy `sky_set_colors()` for backwards compat

6. **Integration**
   - Update draw calls to use `shading.environment_index`
   - Update sky shader to call `sample_environment()`

7. **Test:** Debug inspector panel for Gradient mode.

### Phase 2-8: Additional Modes (One Per Phase)
Each phase adds one mode's:
- WGSL `sample_X(data, offset, dir, time)` function
- FFI `env_X_set(...)` function (writes to `current_environment_state.data`)
- Debug inspector group
- Test in example game

**Suggested order:** Scatter → Lines → Rings → Silhouette → Room → Rectangles → Curtains

---

## Files to Modify

| File | Changes |
|------|---------|
| [unified_shading_state.rs](../../emberware-z/src/graphics/unified_shading_state.rs) | Remove `sky: PackedSky` from struct, add `PackedEnvironmentMode` type |
| [graphics/mod.rs](../../emberware-z/src/graphics/mod.rs) | Add `environments_buffer: wgpu::Buffer` field |
| [graphics/init.rs](../../emberware-z/src/graphics/init.rs) | Create 512-byte environment storage buffer |
| [graphics/pipeline.rs](../../emberware-z/src/graphics/pipeline.rs) | Add @binding(4) for environments, update bind group layout |
| [graphics/frame.rs](../../emberware-z/src/graphics/frame.rs) | Upload environments buffer when dirty |
| [ffi_state.rs](../../emberware-z/src/state/ffi_state.rs) | Add `environments: [PackedEnvironmentMode; 8]`, selection state, dirty flag |
| [sky.rs](../../emberware-z/src/ffi/sky.rs) | Update `sky_set_colors()` for backwards compat |
| [ffi/environment.rs](../../emberware-z/src/ffi/environment.rs) | **NEW** - All `env_*` FFI functions |
| [ffi/mod.rs](../../emberware-z/src/ffi/mod.rs) | Register new environment module |
| [common.wgsl](../../emberware-z/shaders/common.wgsl) | Add `PackedEnvironmentMode` struct, @binding(4) |
| [environment.wgsl](../../emberware-z/shaders/environment.wgsl) | **NEW** - All mode sampling functions |
| [quad_template.wgsl](../../emberware-z/shaders/quad_template.wgsl) | Renumber @binding(4) → @binding(5) |
| [sky_template.wgsl](../../emberware-z/shaders/sky_template.wgsl) | Call `sample_environment()` instead of reading `shading.sky` |

---

## Memory Impact

| Item | Size |
|------|------|
| PackedEnvironmentState | **48 bytes** per unique environment config |
| Environment buffer (GPU) | 48 × N bytes (N = unique configs per frame) |
| HashMap overhead (CPU) | ~48 bytes per unique entry |
| Removed from shading state | -16 bytes (PackedSky) |
| Added to shading state | +4 bytes (environment_index, replaces _animation_reserved) |

**Typical frame:** 1-5 unique environment configs = **48-240 bytes**
**Per shading state:** -12 bytes net (removed 16-byte PackedSky, added 4-byte index)

**Comparison:**
- Old design: 136 bytes × 5 configs = 680 bytes
- New design: 48 bytes × 5 configs = 240 bytes (**65% smaller**)

**Benefits:**
- Bit-packed parameters minimize wasted space
- Deduplication: 100 draws with same environment = 1 buffer entry
- No more sky duplication across shading states
- Flexible: can have multiple gradient configs, mix-and-match modes

---

## Design Decisions (Resolved)

| Question | Resolution |
|----------|------------|
| Storage pattern | Immediate-mode with Vec pool + HashMap dedup (same as transforms/shading) |
| Environment count | 8 mode types (0-7), unlimited instances via dedup |
| Invalid index handling | Debug warn + clamp to 0-7 |
| Reset behavior | Pool cleared each frame in `clear_frame()` |
| Rollback support | Not needed (visual-only, set in render) |
| Blend factor | Always 1.0, color intensity controls effect |
| Variants | Bit-packed in first u32 of mode params |
| Struct size | **48 bytes** (4 header + 44 data) |
| Data layout | Base mode uses data[0..5], overlay uses data[5..10], shared data[10] |
| Bit-packing | Header: 3+3+2 bits for modes/blend, params: per-mode layouts |
| Animation | u16 phase (0-65535), wraps naturally, algorithms designed for seamless loops |
| No time param | All animation via explicit phase values, no implicit time dependency |
| Position values | snorm16 (-32768 to +32767 = -1.0 to +1.0), 0 = center (used by Room) |
