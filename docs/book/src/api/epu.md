# Environment Processing Unit (EPU)

The Environment Processing Unit is Nethercore ZX's procedural background rendering system featuring 8 distinct modes with multi-layer compositing support.

## Overview

Multi-Environment v4 provides:
- **8 Environment Modes** — Gradient, Scatter, Lines, Silhouette, Rectangles, Room, Curtains, Rings
- **Dual-Layer System** — Configure base (layer 0) and overlay (layer 1) independently
- **Same-Mode Layering** — Use the same mode with different parameters on both layers
- **Blend Modes** — Alpha, Add, Multiply, Screen blending for creative effects
- **Animated Parameters** — Phase parameters for seamless looping animations (Scatter, Lines, Rectangles, Curtains, Rings)
- **Parallax Depth** — Multiple depth layers for pseudo-3D effects (Silhouette, Curtains)

All environments are rendered by calling `draw_env()` first in your `render()` function.

---

## Conventions

- `layer` is always `0` (base) or `1` (overlay). Each `env_*()` call sets the mode for that layer.
- Colors are usually `0xRRGGBBAA`. For parameters documented as `0xRRGGBB00`, the low byte is reserved (ignored/overwritten); use `00` for clarity.
- Many integer parameters are packed to 8 bits (or fewer). Values outside the documented range may be truncated/clamped.
- `phase` is a 16-bit wrapping value (0–65535). Advance it with `wrapping_add()` for seamless looping.

---

## Mode 0: Gradient (Featured Sky)

Creates a 4-color sky/ground gradient with **featured sky** controls (sun disc + halo, horizon haze, stylized cloud bands).

### env_gradient

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_gradient(
    layer: u32,          // 0 = base layer, 1 = overlay layer
    zenith: u32,         // Color directly overhead (0xRRGGBBAA)
    sky_horizon: u32,    // Sky color at horizon level (0xRRGGBBAA)
    ground_horizon: u32, // Ground color at horizon level (0xRRGGBBAA)
    nadir: u32,          // Color directly below (0xRRGGBBAA)
    rotation: f32,       // Sun azimuth around Y axis in radians (0 = +Z, π/2 = +X)
    shift: f32,          // Horizon vertical shift (-1.0 to 1.0)
    sun_elevation: f32,  // Sun elevation in radians (0 = horizon, π/2 = zenith)
    sun_disk: u32,       // 0-255
    sun_halo: u32,       // 0-255
    sun_intensity: u32,  // 0-255 (0 disables sun)
    horizon_haze: u32,   // 0-255
    sun_warmth: u32,     // 0-255
    cloudiness: u32      // 0-255
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_gradient(
    uint32_t layer,
    uint32_t zenith,
    uint32_t sky_horizon,
    uint32_t ground_horizon,
    uint32_t nadir,
    float rotation,
    float shift,
    float sun_elevation,
    uint32_t sun_disk,
    uint32_t sun_halo,
    uint32_t sun_intensity,
    uint32_t horizon_haze,
    uint32_t sun_warmth,
    uint32_t cloudiness
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_gradient(
    layer: u32,
    zenith: u32,
    sky_horizon: u32,
    ground_horizon: u32,
    nadir: u32,
    rotation: f32,
    shift: f32,
    sun_elevation: f32,
    sun_disk: u32,
    sun_halo: u32,
    sun_intensity: u32,
    horizon_haze: u32,
    sun_warmth: u32,
    cloudiness: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| zenith | `u32` | Color directly overhead |
| sky_horizon | `u32` | Sky color at horizon level |
| ground_horizon | `u32` | Ground color at horizon level |
| nadir | `u32` | Color directly below |
| rotation | `f32` | Sun azimuth in radians (0 = +Z, π/2 = +X) |
| shift | `f32` | Horizon vertical shift (-1.0 to 1.0, 0.0 = equator) |
| sun_elevation | `f32` | Sun elevation in radians (0 = horizon, π/2 = zenith) |
| sun_disk | `u32` | Sun disc size (0–255) |
| sun_halo | `u32` | Sun halo size (0–255) |
| sun_intensity | `u32` | Sun intensity (0 disables sun) |
| horizon_haze | `u32` | Haze near the horizon (0–255) |
| sun_warmth | `u32` | Sun color warmth (0 = neutral/white, 255 = warm/orange) |
| cloudiness | `u32` | Stylized cloud bands (0 disables, 255 = strongest) |

**Notes:**
- For a pure gradient (no featured sky), set `sun_intensity = 0`, `horizon_haze = 0`, and `cloudiness = 0`.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        const DEG2RAD: f32 = 0.0174532925;

        // Blue day sky on base layer
        env_gradient(
            0,          // Base layer
            0x191970FF, // Midnight blue zenith
            0x87CEEBFF, // Sky blue horizon
            0x228B22FF, // Forest green ground horizon
            0x2F4F4FFF, // Dark slate nadir
            35.0 * DEG2RAD, // Sun azimuth
            0.0,            // Horizon at equator
            35.0 * DEG2RAD, // Sun elevation
            24,             // Sun disk
            120,            // Sun halo
            90,             // Sun intensity
            60,             // Horizon haze
            40,             // Sun warmth
            40              // Cloudiness
        );

        draw_env();
    }

    // Draw your scene...
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    const float DEG2RAD = 0.0174532925f;

    // Blue day sky on base layer
    env_gradient(
        0,          // Layer 0 (base)
        0x191970FF, // Midnight blue zenith
        0x87CEEBFF, // Sky blue horizon
        0x228B22FF, // Forest green ground horizon
        0x2F4F4FFF, // Dark slate nadir
        35.0f * DEG2RAD, // Sun azimuth
        0.0f,            // Horizon at equator
        35.0f * DEG2RAD, // Sun elevation
        24u,             // Sun disk
        120u,            // Sun halo
        90u,             // Sun intensity
        60u,             // Horizon haze
        40u,             // Sun warmth
        40u              // Cloudiness
    );

    draw_env();

    // Draw your scene...
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    const DEG2RAD: f32 = 0.0174532925;

    // Blue day sky
    env_gradient(
        0,          // Layer 0 (base)
        0x191970FF, // Midnight blue zenith
        0x87CEEBFF, // Sky blue horizon
        0x228B22FF, // Forest green ground horizon
        0x2F4F4FFF, // Dark slate nadir
        35.0 * DEG2RAD, // Sun azimuth
        0.0,            // Horizon at equator
        35.0 * DEG2RAD, // Sun elevation
        24,             // Sun disk
        120,            // Sun halo
        90,             // Sun intensity
        60,             // Horizon haze
        40,             // Sun warmth
        40              // Cloudiness
    );

    draw_env();

    // Draw your scene...
}
```
{{#endtab}}

{{#endtabs}}

### Presets

```rust
const DEG2RAD: f32 = 0.0174532925;

// Day (featured sky)
env_gradient(0, 0x2a4aa8ff, 0x8ec9ffff, 0x3b2a20ff, 0x120b08ff, 35.0 * DEG2RAD, 0.0, 35.0 * DEG2RAD, 24, 120, 90, 60, 40, 40);

// Sunset (warm, hazy)
env_gradient(0, 0x1c0a3fff, 0xff7a5cff, 0x3b2a20ff, 0x0b0610ff, 95.0 * DEG2RAD, 0.08, 15.0 * DEG2RAD, 32, 160, 120, 140, 220, 120);

// Stormy (heavy bands)
env_gradient(0, 0x0e1a2fff, 0x3a5874ff, 0x1f2326ff, 0x050608ff, 10.0 * DEG2RAD, -0.02, 20.0 * DEG2RAD, 12, 220, 80, 200, 16, 220);

// Alien (stylized, saturated)
env_gradient(0, 0x2a004cff, 0x00f0ffff, 0x003820ff, 0x12001aff, 210.0 * DEG2RAD, 0.05, 30.0 * DEG2RAD, 28, 200, 110, 64, 180, 160);
```

---

## Mode 1: Scatter

Creates procedural particle fields (stars, rain, speed lines, warp effects).

### env_scatter

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_scatter(
    layer: u32,             // 0 = base layer, 1 = overlay layer
    variant: u32,           // 0=Stars, 1=Vertical, 2=Horizontal, 3=Warp
    density: u32,           // 0-255
    size: u32,              // 0-255
    glow: u32,              // 0-255
    streak_length: u32,     // 0-63
    color_primary: u32,     // 0xRRGGBB00
    color_secondary: u32,   // 0xRRGGBB00
    parallax_rate: u32,     // 0-255
    parallax_size: u32,     // 0-255
    phase: u32              // 0-65535 (animation)
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_scatter(
    uint32_t layer,
    uint32_t variant,
    uint32_t density,
    uint32_t size,
    uint32_t glow,
    uint32_t streak_length,
    uint32_t color_primary,
    uint32_t color_secondary,
    uint32_t parallax_rate,
    uint32_t parallax_size,
    uint32_t phase
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_scatter(
    layer: u32,
    variant: u32,
    density: u32,
    size: u32,
    glow: u32,
    streak_length: u32,
    color_primary: u32,
    color_secondary: u32,
    parallax_rate: u32,
    parallax_size: u32,
    phase: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Variants:**
- **0: Stars** — Static twinkling points
- **1: Vertical** — Rain/snow falling downward
- **2: Horizontal** — Speed lines for motion blur
- **3: Warp** — Radial expansion from center (hyperspace)

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| variant | `u32` | Scatter type (0–3), see variants above |
| density | `u32` | Particle density (0–255) |
| size | `u32` | Particle size (0–255) |
| glow | `u32` | Glow/bloom intensity (0–255) |
| streak_length | `u32` | Streak elongation (0–63); only used for Vertical/Horizontal variants |
| color_primary | `u32` | Primary particle color (`0xRRGGBB00`) |
| color_secondary | `u32` | Secondary particle color (`0xRRGGBB00`) |
| parallax_rate | `u32` | Reserved for parallax depth (currently no visible effect) |
| parallax_size | `u32` | Reserved for depth-based size variation (currently no visible effect) |
| phase | `u32` | Animation phase (0–65535, wraps) |

**Animation Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut STAR_PHASE: u32 = 0;

fn update() {
    unsafe {
        // Animate twinkle
        STAR_PHASE = STAR_PHASE.wrapping_add((delta_time() * 0.1 * 65535.0) as u32);
    }
}

fn render() {
    unsafe {
        // Starfield
        env_scatter(
            0,              // Layer 0 (base)
            0,              // Stars variant
            200,            // High density
            2,              // Small size
            1,              // Subtle glow
            0,              // No streaks
            0xFFFFFF00,     // White primary
            0xAAAAFF00,     // Blue-white secondary (twinkle)
            0,              // Reserved parallax_rate (currently no effect)
            0,              // Reserved parallax_size (currently no effect)
            STAR_PHASE      // Animated twinkle
        );

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t star_phase = 0;

NCZX_EXPORT void update(void) {
    // Animate twinkle
    star_phase += (uint32_t)(delta_time() * 0.1f * 65535.0f);
}

NCZX_EXPORT void render(void) {
    // Starfield
    env_scatter(
        0,              // Layer 0 (base)
        0,              // Stars variant
        200,            // High density
        2,              // Small size
        1,              // Subtle glow
        0,              // No streaks
        0xFFFFFF00,     // White primary
        0xAAAAFF00,     // Blue-white secondary (twinkle)
        0,              // Reserved parallax_rate (currently no effect)
        0,              // Reserved parallax_size (currently no effect)
        star_phase      // Animated twinkle
    );

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var star_phase: u32 = 0;

export fn update() void {
    // Animate twinkle
    star_phase +%= @intFromFloat(delta_time() * 0.1 * 65535.0);
}

export fn render() void {
    // Starfield
    env_scatter(
        0,              // Layer 0 (base)
        0,              // Stars variant
        200,            // High density
        2,              // Small size
        1,              // Subtle glow
        0,              // No streaks
        0xFFFFFF00,     // White primary
        0xAAAAFF00,     // Blue-white secondary (twinkle)
        0,              // Reserved parallax_rate (currently no effect)
        0,              // Reserved parallax_size (currently no effect)
        star_phase      // Animated twinkle
    );

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

---

## Mode 2: Lines

Creates infinite procedural grids (synthwave floors, racing tracks, holographic overlays).

### env_lines

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_lines(
    layer: u32,         // 0 = base layer, 1 = overlay layer
    variant: u32,       // 0=Floor, 1=Ceiling, 2=Sphere
    line_type: u32,     // 0=Horizontal, 1=Vertical, 2=Grid
    thickness: u32,     // 0-255
    spacing: f32,       // World units
    fade_distance: f32, // World units
    color_primary: u32, // 0xRRGGBBAA
    color_accent: u32,  // 0xRRGGBBAA
    accent_every: u32,  // Make every Nth line accent
    phase: u32          // 0-65535 (scroll)
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_lines(
    uint32_t layer,
    uint32_t variant,
    uint32_t line_type,
    uint32_t thickness,
    float spacing,
    float fade_distance,
    uint32_t color_primary,
    uint32_t color_accent,
    uint32_t accent_every,
    uint32_t phase
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_lines(
    layer: u32,
    variant: u32,
    line_type: u32,
    thickness: u32,
    spacing: f32,
    fade_distance: f32,
    color_primary: u32,
    color_accent: u32,
    accent_every: u32,
    phase: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Variants:**
- **0: Floor** — Infinite grid “below” the camera
- **1: Ceiling** — Infinite grid “above” the camera
- **2: Sphere** — Spherical grid around the camera

**Line Types:**
- **0: Horizontal** — Only horizontal lines
- **1: Vertical** — Only vertical lines
- **2: Grid** — Horizontal + vertical

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| variant | `u32` | Surface type (0–2), see variants above |
| line_type | `u32` | Line pattern type (0–2), see line types above |
| thickness | `u32` | Line thickness (0–255) |
| spacing | `f32` | Distance between lines (world units) |
| fade_distance | `f32` | Distance where lines start fading (world units) |
| color_primary | `u32` | Primary line color (`0xRRGGBBAA`) |
| color_accent | `u32` | Accent line color (`0xRRGGBBAA`) |
| accent_every | `u32` | Every Nth line uses `color_accent` |
| phase | `u32` | Scroll phase (0–65535, wraps) |

**Example: Synthwave Grid**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut GRID_PHASE: u32 = 0;

fn update() {
    unsafe {
        // Scroll grid
        GRID_PHASE = GRID_PHASE.wrapping_add((delta_time() * 2.0 * 65535.0) as u32);
    }
}

fn render() {
    unsafe {
        // Floor grid
        env_lines(
            0,              // Layer 0 (base)
            0,              // Floor variant
            2,              // Grid pattern
            2,              // Medium thickness
            2.0,            // 2-unit spacing
            50.0,           // Fade at 50 units
            0xFF00FFFF,     // Magenta primary
            0x00FFFFFF,     // Cyan accent
            4,              // Every 4th line is cyan
            GRID_PHASE      // Animated scroll
        );

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t grid_phase = 0;

NCZX_EXPORT void update(void) {
    grid_phase += (uint32_t)(delta_time() * 2.0f * 65535.0f);
}

NCZX_EXPORT void render(void) {
    env_lines(
        0,              // Layer 0 (base)
        0,              // Floor variant
        2,              // Grid pattern
        2,              // Medium thickness
        2.0f,           // 2-unit spacing
        50.0f,          // Fade at 50 units
        0xFF00FFFF,     // Magenta primary
        0x00FFFFFF,     // Cyan accent
        4,              // Every 4th line is cyan
        grid_phase      // Animated scroll
    );

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var grid_phase: u32 = 0;

export fn update() void {
    grid_phase +%= @intFromFloat(delta_time() * 2.0 * 65535.0);
}

export fn render() void {
    env_lines(
        0,              // Layer 0 (base)
        0,              // Floor variant
        2,              // Grid pattern
        2,              // Medium thickness
        2.0,            // 2-unit spacing
        50.0,           // Fade at 50 units
        0xFF00FFFF,     // Magenta primary
        0x00FFFFFF,     // Cyan accent
        4,              // Every 4th line is cyan
        grid_phase      // Animated scroll
    );

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

---

## Mode 3: Silhouette

Creates layered terrain silhouettes with procedural noise (mountains, cityscapes).

### env_silhouette

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_silhouette(
    layer: u32,         // 0 = base layer, 1 = overlay layer
    jaggedness: u32,    // 0-255 terrain roughness
    layer_count: u32,   // 1-3 depth layers
    color_near: u32,    // 0xRRGGBBAA
    color_far: u32,     // 0xRRGGBBAA
    sky_zenith: u32,    // 0xRRGGBBAA
    sky_horizon: u32,   // 0xRRGGBBAA
    parallax_rate: u32, // 0-255
    seed: u32           // Noise seed
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_silhouette(
    uint32_t layer,
    uint32_t jaggedness,
    uint32_t layer_count,
    uint32_t color_near,
    uint32_t color_far,
    uint32_t sky_zenith,
    uint32_t sky_horizon,
    uint32_t parallax_rate,
    uint32_t seed
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_silhouette(
    layer: u32,
    jaggedness: u32,
    layer_count: u32,
    color_near: u32,
    color_far: u32,
    sky_zenith: u32,
    sky_horizon: u32,
    parallax_rate: u32,
    seed: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| jaggedness | `u32` | Terrain roughness (0–255) |
| layer_count | `u32` | Depth layers (1–3) |
| color_near | `u32` | Nearest silhouette color (`0xRRGGBBAA`) |
| color_far | `u32` | Farthest silhouette color (`0xRRGGBBAA`) |
| sky_zenith | `u32` | Sky zenith color behind silhouettes (`0xRRGGBBAA`) |
| sky_horizon | `u32` | Sky horizon color behind silhouettes (`0xRRGGBBAA`) |
| parallax_rate | `u32` | Layer separation amount (0–255) |
| seed | `u32` | Deterministic seed for terrain shape |

**Example: Mountain Range**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        env_silhouette(
            0,              // Layer 0 (base)
            200,            // Jagged peaks
            3,              // 3 depth layers
            0x1a1a2eFF,     // Dark blue near
            0x4d4d66FF,     // Gray-blue far
            0xFF9966FF,     // Orange zenith
            0xFFCC99FF,     // Light orange horizon
            128,            // Moderate parallax
            42              // Seed
        );

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    env_silhouette(
        0,              // Layer 0 (base)
        200,            // Jagged peaks
        3,              // 3 depth layers
        0x1a1a2eFF,     // Dark blue near
        0x4d4d66FF,     // Gray-blue far
        0xFF9966FF,     // Orange zenith
        0xFFCC99FF,     // Light orange horizon
        128,            // Moderate parallax
        42              // Seed
    );

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    env_silhouette(
        0,              // Layer 0 (base)
        200,            // Jagged peaks
        3,              // 3 depth layers
        0x1a1a2eFF,     // Dark blue near
        0x4d4d66FF,     // Gray-blue far
        0xFF9966FF,     // Orange zenith
        0xFFCC99FF,     // Light orange horizon
        128,            // Moderate parallax
        42              // Seed
    );

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

---

## Mode 4: Rectangles

Creates rectangular light sources (city windows, control panels, screens).

### env_rectangles

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_rectangles(
    layer: u32,            // 0 = base layer, 1 = overlay layer
    variant: u32,          // 0=Scatter, 1=Buildings, 2=Bands, 3=Panels
    density: u32,          // 0-255
    lit_ratio: u32,        // 0-255 percentage lit
    size_min: u32,         // 0-63
    size_max: u32,         // 0-63
    aspect: u32,           // 0-3 aspect ratio
    color_primary: u32,    // 0xRRGGBBAA
    color_variation: u32,  // 0xRRGGBBAA
    parallax_rate: u32,    // 0-255
    phase: u32             // 0-65535 (flicker)
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_rectangles(
    uint32_t layer,
    uint32_t variant,
    uint32_t density,
    uint32_t lit_ratio,
    uint32_t size_min,
    uint32_t size_max,
    uint32_t aspect,
    uint32_t color_primary,
    uint32_t color_variation,
    uint32_t parallax_rate,
    uint32_t phase
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_rectangles(
    layer: u32,
    variant: u32,
    density: u32,
    lit_ratio: u32,
    size_min: u32,
    size_max: u32,
    aspect: u32,
    color_primary: u32,
    color_variation: u32,
    parallax_rate: u32,
    phase: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| variant | `u32` | Pattern type (0–3): Scatter, Buildings, Bands, Panels |
| density | `u32` | Rectangle density (0–255) |
| lit_ratio | `u32` | Percentage of rectangles lit (0–255, ~128 ≈ 50%) |
| size_min | `u32` | Minimum rectangle size (0–63) |
| size_max | `u32` | Maximum rectangle size (0–63) |
| aspect | `u32` | Aspect bias (0–3) |
| color_primary | `u32` | Primary rectangle color (`0xRRGGBBAA`) |
| color_variation | `u32` | Variation color (`0xRRGGBBAA`) |
| parallax_rate | `u32` | Reserved for depth/parallax (currently no visible effect) |
| phase | `u32` | Flicker phase (0–65535, wraps) |

**Example: Cyberpunk City**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        env_rectangles(
            0,              // Layer 0 (base)
            1,              // Buildings variant
            180,            // High density
            160,            // ~63% lit
            8,              // Min size
            24,             // Max size
            2,              // Tall aspect ratio
            0xFF00FFAA,     // Magenta primary
            0x00FFFF80,     // Cyan variation
            100,            // Reserved parallax (currently no effect)
            0               // Static (no flicker)
        );

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    env_rectangles(
        0,              // Layer 0 (base)
        1,              // Buildings variant
        180,            // High density
        160,            // ~63% lit
        8,              // Min size
        24,             // Max size
        2,              // Tall aspect ratio
        0xFF00FFAA,     // Magenta primary
        0x00FFFF80,     // Cyan variation
        100,            // Reserved parallax (currently no effect)
        0               // Static (no flicker)
    );

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    env_rectangles(
        0,              // Layer 0 (base)
        1,              // Buildings variant
        180,            // High density
        160,            // ~63% lit
        8,              // Min size
        24,             // Max size
        2,              // Tall aspect ratio
        0xFF00FFAA,     // Magenta primary
        0x00FFFF80,     // Cyan variation
        100,            // Reserved parallax (currently no effect)
        0               // Static (no flicker)
    );

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

---

## Mode 5: Room

Creates interior 3D box environments with directional lighting.

### env_room

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_room(
    layer: u32,          // 0 = base layer, 1 = overlay layer
    color_ceiling: u32,  // 0xRRGGBB00
    color_floor: u32,    // 0xRRGGBB00
    color_walls: u32,    // 0xRRGGBB00
    panel_size: f32,     // World units
    panel_gap: u32,      // 0-255
    light_dir_x: f32,
    light_dir_y: f32,
    light_dir_z: f32,
    light_intensity: u32,  // 0-255
    corner_darken: u32,    // 0-255
    room_scale: f32,
    viewer_x: i32,         // -128 to 127
    viewer_y: i32,
    viewer_z: i32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_room(
    uint32_t layer,
    uint32_t color_ceiling,
    uint32_t color_floor,
    uint32_t color_walls,
    float panel_size,
    uint32_t panel_gap,
    float light_dir_x,
    float light_dir_y,
    float light_dir_z,
    uint32_t light_intensity,
    uint32_t corner_darken,
    float room_scale,
    int32_t viewer_x,
    int32_t viewer_y,
    int32_t viewer_z
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_room(
    layer: u32,
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
    viewer_x: i32,
    viewer_y: i32,
    viewer_z: i32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| color_ceiling | `u32` | Ceiling color (`0xRRGGBB00`; alpha byte is overwritten internally) |
| color_floor | `u32` | Floor color (`0xRRGGBB00`; alpha byte is overwritten internally) |
| color_walls | `u32` | Wall color (`0xRRGGBB00`; alpha byte is overwritten internally) |
| panel_size | `f32` | Panel grid size (world units) |
| panel_gap | `u32` | Panel gap thickness (0–255) |
| light_dir_x, light_dir_y, light_dir_z | `f32` | Reserved for room light direction (currently no visible effect) |
| light_intensity | `u32` | Directional light intensity (0–255) |
| corner_darken | `u32` | Corner darkening amount (0–255) |
| room_scale | `f32` | Room half-extent scale |
| viewer_x, viewer_y, viewer_z | `i32` | Viewer position packed as snorm8 (-128..127 ≈ -1..1) |

**Example: Hangar**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        env_room(
            0,              // Layer 0 (base)
            0x66666600,     // Gray ceiling
            0x33333300,     // Dark gray floor
            0x4d4d4d00,     // Medium gray walls
            4.0,            // 4-unit panels
            8,              // Panel gaps
            0.3,            // Reserved light_dir (currently no effect)
            -0.7,
            0.5,
            180,            // Bright lighting
            100,            // Moderate corner darkening
            20.0,           // Large room
            0,              // Centered viewer
            0,
            0
        );

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    env_room(
        0,              // Layer 0 (base)
        0x66666600,     // Gray ceiling
        0x33333300,     // Dark gray floor
        0x4d4d4d00,     // Medium gray walls
        4.0f,           // 4-unit panels
        8,              // Panel gaps
        0.3f,           // Reserved light_dir (currently no effect)
        -0.7f,
        0.5f,
        180,            // Bright lighting
        100,            // Moderate corner darkening
        20.0f,          // Large room
        0,              // Centered viewer
        0,
        0
    );

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    env_room(
        0,              // Layer 0 (base)
        0x66666600,     // Gray ceiling
        0x33333300,     // Dark gray floor
        0x4d4d4d00,     // Medium gray walls
        4.0,            // 4-unit panels
        8,              // Panel gaps
        0.3,            // Reserved light_dir (currently no effect)
        -0.7,
        0.5,
        180,            // Bright lighting
        100,            // Moderate corner darkening
        20.0,           // Large room
        0,              // Centered viewer
        0,
        0
    );

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

---

## Mode 6: Curtains

Creates vertical structures (pillars, trees, neon strips).

### env_curtains

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_curtains(
    layer: u32,        // 0 = base layer, 1 = overlay layer
    layer_count: u32,   // 1-3
    density: u32,       // 0-255
    height_min: u32,    // 0-63
    height_max: u32,    // 0-63
    width: u32,         // 0-31
    spacing: u32,       // 0-31
    waviness: u32,      // 0-255
    color_near: u32,    // 0xRRGGBBAA
    color_far: u32,     // 0xRRGGBBAA
    glow: u32,          // 0-255
    parallax_rate: u32, // 0-255
    phase: u32          // 0-65535 (scroll)
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_curtains(
    uint32_t layer,
    uint32_t layer_count,
    uint32_t density,
    uint32_t height_min,
    uint32_t height_max,
    uint32_t width,
    uint32_t spacing,
    uint32_t waviness,
    uint32_t color_near,
    uint32_t color_far,
    uint32_t glow,
    uint32_t parallax_rate,
    uint32_t phase
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_curtains(
    layer: u32,
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
    phase: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| layer_count | `u32` | Depth layers (1–3) |
| density | `u32` | Structure density (0–255) |
| height_min | `u32` | Minimum height (0–63) |
| height_max | `u32` | Maximum height (0–63) |
| width | `u32` | Structure width (0–31) |
| spacing | `u32` | Gap between structures (0–31) |
| waviness | `u32` | Wobble/organic motion (0–255) |
| color_near | `u32` | Near color (`0xRRGGBBAA`) |
| color_far | `u32` | Far color (`0xRRGGBBAA`) |
| glow | `u32` | Glow intensity (0–255) |
| parallax_rate | `u32` | Depth separation amount (0–255) |
| phase | `u32` | Horizontal scroll phase (0–65535, wraps) |

**Example: Neon Pillars**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        env_curtains(
            0,              // Layer 0 (base)
            2,              // 2 depth layers
            30,             // Sparse density
            40,             // Min height
            60,             // Max height
            4,              // Narrow width
            12,             // Wide spacing
            30,             // Some waviness
            0xFF00FFFF,     // Magenta near
            0x8800AAFF,     // Dark magenta far
            200,            // High glow
            120,            // Strong parallax
            0               // Static
        );

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    env_curtains(
        0,              // Layer 0 (base)
        2,              // 2 depth layers
        30,             // Sparse density
        40,             // Min height
        60,             // Max height
        4,              // Narrow width
        12,             // Wide spacing
        30,             // Some waviness
        0xFF00FFFF,     // Magenta near
        0x8800AAFF,     // Dark magenta far
        200,            // High glow
        120,            // Strong parallax
        0               // Static
    );

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    env_curtains(
        0,              // Layer 0 (base)
        2,              // 2 depth layers
        30,             // Sparse density
        40,             // Min height
        60,             // Max height
        4,              // Narrow width
        12,             // Wide spacing
        30,             // Some waviness
        0xFF00FFFF,     // Magenta near
        0x8800AAFF,     // Dark magenta far
        200,            // High glow
        120,            // Strong parallax
        0               // Static
    );

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

---

## Mode 7: Rings

Creates concentric rings (portals, tunnels, vortex effects).

### env_rings

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_rings(
    layer: u32,          // 0 = base layer, 1 = overlay layer
    ring_count: u32,     // 1-255
    thickness: u32,      // 0-255
    color_a: u32,        // 0xRRGGBBAA
    color_b: u32,        // 0xRRGGBBAA
    center_color: u32,   // 0xRRGGBBAA
    center_falloff: u32, // 0-255
    spiral_twist: f32,   // Degrees
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32           // 0-65535 (rotation)
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_rings(
    uint32_t layer,
    uint32_t ring_count,
    uint32_t thickness,
    uint32_t color_a,
    uint32_t color_b,
    uint32_t center_color,
    uint32_t center_falloff,
    float spiral_twist,
    float axis_x,
    float axis_y,
    float axis_z,
    uint32_t phase
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_rings(
    layer: u32,
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
    phase: u32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| layer | `u32` | Target layer: 0 = base, 1 = overlay |
| ring_count | `u32` | Number of rings (1–255) |
| thickness | `u32` | Ring thickness (0–255) |
| color_a | `u32` | Alternating ring color A (`0xRRGGBBAA`) |
| color_b | `u32` | Alternating ring color B (`0xRRGGBBAA`) |
| center_color | `u32` | Center glow color (`0xRRGGBBAA`) |
| center_falloff | `u32` | Center falloff amount (0–255) |
| spiral_twist | `f32` | Spiral twist in degrees (0 = concentric) |
| axis_x, axis_y, axis_z | `f32` | Ring axis direction (normalized) |
| phase | `u32` | Rotation phase (0–65535, wraps) |

**Example: Portal**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut PORTAL_PHASE: u32 = 0;

fn update() {
    unsafe {
        // Spin portal
        PORTAL_PHASE = PORTAL_PHASE.wrapping_add((delta_time() * 2.0 * 65535.0) as u32);
    }
}

fn render() {
    unsafe {
        env_rings(
            0,              // Layer 0 (base)
            32,             // Many rings
            3,              // Thin rings
            0xFF00FFFF,     // Magenta
            0x00FFFFFF,     // Cyan
            0xFFFFFFFF,     // Bright white center
            200,            // Bright center falloff
            15.0,           // Spiral twist
            0.0,            // Facing camera (Z axis)
            0.0,
            1.0,
            PORTAL_PHASE    // Animated spin
        );

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t portal_phase = 0;

NCZX_EXPORT void update(void) {
    portal_phase += (uint32_t)(delta_time() * 2.0f * 65535.0f);
}

NCZX_EXPORT void render(void) {
    env_rings(
        0,              // Layer 0 (base)
        32,             // Many rings
        3,              // Thin rings
        0xFF00FFFF,     // Magenta
        0x00FFFFFF,     // Cyan
        0xFFFFFFFF,     // Bright white center
        200,            // Bright center falloff
        15.0f,          // Spiral twist
        0.0f,           // Facing camera (Z axis)
        0.0f,
        1.0f,
        portal_phase    // Animated spin
    );

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var portal_phase: u32 = 0;

export fn update() void {
    portal_phase +%= @intFromFloat(delta_time() * 2.0 * 65535.0);
}

export fn render() void {
    env_rings(
        0,              // Layer 0 (base)
        32,             // Many rings
        3,              // Thin rings
        0xFF00FFFF,     // Magenta
        0x00FFFFFF,     // Cyan
        0xFFFFFFFF,     // Bright white center
        200,            // Bright center falloff
        15.0,           // Spiral twist
        0.0,            // Facing camera (Z axis)
        0.0,
        1.0,
        portal_phase    // Animated spin
    );

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

---

## Multi-Layer Compositing

The EPU supports dual-layer rendering where you can configure a base layer (0) and overlay layer (1) independently. Each layer can use any of the 8 modes, allowing for creative combinations including using the same mode with different parameters.

### Layer System

Configure environments by specifying the layer parameter (0 or 1) in each `env_*()` function:
- **Layer 0** — Base layer rendered first
- **Layer 1** — Overlay layer composited on top using blend mode

You can use **the same mode on both layers** with different parameters. For example: stars on layer 0 and rain on layer 1 (both using scatter mode).

### env_blend

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_blend(mode: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_blend(uint32_t mode);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_blend(mode: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Blend Modes:**
- 0 — Alpha (standard alpha blending)
- 1 — Add (additive blending)
- 2 — Multiply (multiplicative)
- 3 — Screen (light blending)

**Example: Starfield Over Gradient**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        // Layer 0: dark gradient base
        env_gradient(
            0,
            0x000000FF,
            0x0a0a1aFF,
            0x0a0a1aFF,
            0x000000FF,
            0.0, // sun azimuth
            0.0, // horizon shift
            0.0, // sun elevation
            0,   // sun disk
            0,   // sun halo
            0,   // sun intensity (disabled)
            0,   // horizon haze
            0,   // sun warmth
            0,   // cloudiness
        );

        // Layer 1: twinkling stars overlay
        env_scatter(1, 0, 200, 2, 1, 0, 0xFFFFFF00, 0xAAAAFF00, 0, 0, 0);

        // Use additive blending for glowing stars
        env_blend(1); // Additive

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Layer 0: dark gradient base
    env_gradient(0, 0x000000FF, 0x0a0a1aFF, 0x0a0a1aFF, 0x000000FF, 0.0f, 0.0f, 0.0f, 0u, 0u, 0u, 0u, 0u, 0u);

    // Layer 1: twinkling stars overlay
    env_scatter(1, 0, 200, 2, 1, 0, 0xFFFFFF00, 0xAAAAFF00, 0, 0, 0);

    // Use additive blending for glowing stars
    env_blend(1);  // Additive

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Layer 0: dark gradient base
    env_gradient(0, 0x000000FF, 0x0a0a1aFF, 0x0a0a1aFF, 0x000000FF, 0.0, 0.0, 0.0, 0, 0, 0, 0, 0, 0);

    // Layer 1: twinkling stars overlay
    env_scatter(1, 0, 200, 2, 1, 0, 0xFFFFFF00, 0xAAAAFF00, 0, 0, 0);

    // Use additive blending for glowing stars
    env_blend(1);  // Additive

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

## Rendering the Environment

### draw_env

Renders the configured environment. Always call **first** in your `render()` function, before any 3D geometry.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_env()
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_env(void);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_env() void;
```
{{#endtab}}

{{#endtabs}}

---

## Matcap Textures (Mode 1 Only)

### matcap_set

Binds a matcap texture to a slot for Mode 1 (Matcap rendering).

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn matcap_set(slot: u32, texture: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void matcap_set(uint32_t slot, uint32_t texture);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn matcap_set(slot: u32, texture: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**
- `slot` — Matcap slot (1-3)
- `texture` — Texture handle from `load_texture()` or `rom_texture()`

Using this function in modes other than Matcap is allowed but has no effect.

---

**See Also:** [Lighting](./lighting.md), [Materials](./materials.md), [Render Modes Guide](../guides/render-modes.md)
