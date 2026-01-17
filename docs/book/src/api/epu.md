# Environment Processing Unit (EPU)

The Environment Processing Unit is ZX’s procedural background + ambient environment system. It renders an infinite “environment” when you call `draw_env()` and is also sampled by lit shaders for sky/ambient.

## Overview

Multi-Environment v4 provides:
- **8 environment modes** — Gradient, Cells, Lines, Silhouette, Nebula, Room, Veil, Rings
- **Dual-layer system** — Configure base (`layer=0`) and overlay (`layer=1`) independently
- **Same-mode layering** — Base and overlay may use the same mode with different parameters
- **Blend modes** — `env_blend(0..3)` controls how overlay composites onto base
- **Loopable animation** — Most modes take a `phase` that wraps cleanly (0–65535)

For mode selection and example recipes, see [EPU Environments](../guides/epu-environments.md).

All environments are rendered by calling `draw_env()` in your `render()` function.

## Quick Use

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    unsafe {
        // Base layer
        env_gradient(
            0,
            0x2E65FFFF, 0xA9D8FFFF, 0x4D8B4DFF, 0x102010FF,
            0.35, 0.00, 0.95,
            10, 72, 230, 32, 24, 40,
            0,
        );

        // Overlay layer (example)
        env_rings(
            1,
            0, // Portal
            48, 28,
            0x2EE7FFFF, 0x0B2B4CFF,
            0xE8FFFFFF, 190,
            25.0,
            0.0, 0.0, 1.0,
            0,
            9000, 32, 24, 160, 41,
        );
        env_blend(3); // Screen

        draw_env();
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    env_gradient(
        0,
        0x2E65FFFF, 0xA9D8FFFF, 0x4D8B4DFF, 0x102010FF,
        0.35f, 0.00f, 0.95f,
        10u, 72u, 230u, 32u, 24u, 40u,
        0u
    );

    env_rings(
        1,
        0u, // Portal
        48u, 28u,
        0x2EE7FFFFu, 0x0B2B4CFFu,
        0xE8FFFFFFu, 190u,
        25.0f,
        0.0f, 0.0f, 1.0f,
        0u,
        9000u, 32u, 24u, 160u, 41u
    );
    env_blend(3u); // Screen

    draw_env();
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    env_gradient(
        0,
        0x2E65FFFF, 0xA9D8FFFF, 0x4D8B4DFF, 0x102010FF,
        0.35, 0.00, 0.95,
        10, 72, 230, 32, 24, 40,
        0,
    );

    env_rings(
        1,
        0,
        48, 28,
        0x2EE7FFFF, 0x0B2B4CFF,
        0xE8FFFFFF, 190,
        25.0,
        0.0, 0.0, 1.0,
        0,
        9000, 32, 24, 160, 41,
    );
    env_blend(3);

    draw_env();
}
```
{{#endtab}}

{{#endtabs}}

## Conventions

- `layer` is always `0` (base) or `1` (overlay). Each `env_*()` call sets the mode for that layer.
- Colors are usually `0xRRGGBBAA`. For parameters documented as `0xRRGGBB00`, the low byte is reserved/overwritten; use `00` for clarity.
- Many integer parameters are packed to 8 bits (or fewer). Values outside the documented range may be truncated/clamped.
- `phase` is treated as a wrapping 16-bit value (0–65535). Advance it with `wrapping_add()` for seamless looping.
- For `axis_x/y/z`, pass a normalized direction. If near-zero, ZX falls back to a sensible default per mode.

## GPU Snapshot (v4 / 64 bytes)

The packed environment state sent to the GPU is **exactly 64 bytes** (16-byte aligned):

- `header: u32` — base mode (bits 0–2), overlay mode (bits 3–5), blend mode (bits 6–7)
- `data: [u32; 14]` — 7 words per layer:
  - base layer: `data[0..7)` → `w0..w6`
  - overlay layer: `data[7..14)` → `w0..w6`

Per-mode word layouts are documented below; packers must **zero unused/reserved bytes** to avoid stale-byte leaks across mode switches.

---

## env_blend

Sets the blend mode used to composite overlay onto base.

Blend modes:
- `0` — Alpha (`lerp(base, overlay, overlay.a)`)
- `1` — Add (`base + overlay`)
- `2` — Multiply (`base * overlay`)
- `3` — Screen (`1 - (1-base) * (1-overlay)`)

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

---

## Mode 0: Gradient (Featured Sky)

4-color sky/ground gradient plus featured sky controls (sun disc + halo, horizon haze, stylized cloud bands). Shader is trig-free; CPU packs sun direction.

### env_gradient

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_gradient(
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
    cloudiness: u32,
    cloud_phase: u32,
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
    uint32_t cloudiness,
    uint32_t cloud_phase
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
    cloudiness: u32,
    cloud_phase: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Packed layout (per layer):
- `w0..w3`: `zenith`, `sky_horizon`, `ground_horizon`, `nadir` (RGBA8)
- `w4`: `cloud_phase:u16 (low16) | shift:f16 (high16)`
- `w5`: `sun_dir_oct16 (low16) | sun_disk:u8 | sun_halo:u8`
- `w6`: `sun_intensity:u8 | horizon_haze:u8 | sun_warmth:u8 | cloudiness:u8`

---

## Mode 1: Cells (Particles / Tiles / Lights)

Unified cell generator with two families:
- Family `0`: particles (stars/snow/rain/embers/bubbles/warp)
- Family `1`: tiles/lights (Mondrian/Truchet, buildings, bands, panels)

### env_cells

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_cells(
    layer: u32,
    family: u32,
    variant: u32,
    density: u32,
    size_min: u32,
    size_max: u32,
    intensity: u32,
    shape: u32,
    motion: u32,
    parallax: u32,
    height_bias: u32,
    clustering: u32,
    color_a: u32,
    color_b: u32,
    phase: u32,
    seed: u32,
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_cells(
    uint32_t layer,
    uint32_t family,
    uint32_t variant,
    uint32_t density,
    uint32_t size_min,
    uint32_t size_max,
    uint32_t intensity,
    uint32_t shape,
    uint32_t motion,
    uint32_t parallax,
    uint32_t height_bias,
    uint32_t clustering,
    uint32_t color_a,
    uint32_t color_b,
    uint32_t phase,
    uint32_t seed
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_cells(
    layer: u32,
    family: u32,
    variant: u32,
    density: u32,
    size_min: u32,
    size_max: u32,
    intensity: u32,
    shape: u32,
    motion: u32,
    parallax: u32,
    height_bias: u32,
    clustering: u32,
    color_a: u32,
    color_b: u32,
    phase: u32,
    seed: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Notes:
- `phase` is treated as `u16` (wraps). Avoid using `phase` directly as a hash input; animation is designed to be loopable and shimmer-free.
- `parallax` also selects bounded internal depth slices for **Family 0: Particles**: `0–95` → 1 slice, `96–191` → 2 slices, `192–255` → 3 slices (farthest slices are smaller + less parallax-biased).
- `seed=0` means “auto”: derive a deterministic seed from the packed payload.

Packed layout (per layer):
- `w0`: `family:u8 | variant:u8 | density:u8 | intensity:u8`
- `w1`: `size_min:u8 | size_max:u8 | shape:u8 | motion:u8`
- `w2..w3`: `color_a`, `color_b` (RGBA8)
- `w4`: `parallax:u8 | reserved:u24` (**reserved must be zero**)
- `w5`: `phase:u16 (low16) | height_bias:u8 | clustering:u8`
- `w6`: `seed:u32` (`0` = auto)

---

## Mode 2: Lines (Grid / Lanes / Scanlines / Bands)

Anti-aliased line patterns for floors, ceilings, or a spherical wrap.

Enums:
- `variant`: `0`=Floor, `1`=Ceiling, `2`=Sphere
- `line_type`: `0`=Horizontal, `1`=Vertical, `2`=Grid
- `profile`: `0`=Grid, `1`=Lanes, `2`=Scanlines, `3`=Caustic Bands

### env_lines

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_lines(
    layer: u32,
    variant: u32,
    line_type: u32,
    thickness: u32,
    spacing: f32,
    fade_distance: f32,
    parallax: u32,
    color_primary: u32,
    color_accent: u32,
    accent_every: u32,
    phase: u32,
    profile: u32,
    warp: u32,
    wobble: u32,
    glow: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    seed: u32,
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
    uint32_t parallax,
    uint32_t color_primary,
    uint32_t color_accent,
    uint32_t accent_every,
    uint32_t phase,
    uint32_t profile,
    uint32_t warp,
    uint32_t wobble,
    uint32_t glow,
    float axis_x,
    float axis_y,
    float axis_z,
    uint32_t seed
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
    parallax: u32,
    color_primary: u32,
    color_accent: u32,
    accent_every: u32,
    phase: u32,
    profile: u32,
    warp: u32,
    wobble: u32,
    glow: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    seed: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Notes:
- `phase` is treated as `u16` (wraps). Avoid using `phase` directly as a hash input; scrolling and wobble are designed to be loopable and shimmer-free.
- `parallax` also selects bounded internal depth slices: `0–95` → 1 slice, `96–191` → 2 slices, `192–255` → 3 slices (extra slices are offset and less dominant).
- `seed=0` means “auto”: derive a deterministic seed from the packed payload.

Packed layout (per layer):
- `w0`: `variant:u2 | line_type:u2 | thickness:u8 | accent_every:u8 | parallax:u8 | reserved:u4`
- `w1`: `spacing:f16 | fade_distance:f16`
- `w2..w3`: `color_primary`, `color_accent` (RGBA8)
- `w4`: `phase:u16 (low16) | axis_oct16:u16 (high16)`
- `w5`: `warp:u8 | glow:u8 | wobble:u8 | profile:u8`
- `w6`: `seed:u32` (`0` = auto)

---

## Mode 3: Silhouette (Horizon Shapes)

Layered horizon silhouettes (mountains/city/forest/waves) with bounded depth layers (≤3). Works as an anchor (includes sky) or as an overlay (set sky alpha to 0 and blend with Alpha).

Enums:
- `family`: `0`=Mountains, `1`=City skyline, `2`=Forest canopy, `3`=Waves/Coral

### env_silhouette

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_silhouette(
    layer: u32,
    family: u32,
    jaggedness: u32,
    layer_count: u32,
    color_near: u32,
    color_far: u32,
    sky_zenith: u32,
    sky_horizon: u32,
    parallax_rate: u32,
    seed: u32,
    phase: u32,
    fog: u32,
    wind: u32,
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_silhouette(
    uint32_t layer,
    uint32_t family,
    uint32_t jaggedness,
    uint32_t layer_count,
    uint32_t color_near,
    uint32_t color_far,
    uint32_t sky_zenith,
    uint32_t sky_horizon,
    uint32_t parallax_rate,
    uint32_t seed,
    uint32_t phase,
    uint32_t fog,
    uint32_t wind
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_silhouette(
    layer: u32,
    family: u32,
    jaggedness: u32,
    layer_count: u32,
    color_near: u32,
    color_far: u32,
    sky_zenith: u32,
    sky_horizon: u32,
    parallax_rate: u32,
    seed: u32,
    phase: u32,
    fog: u32,
    wind: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Notes:
- `layer_count` is clamped to `1..=3`.
- `phase` is treated as `u16` (wraps). Use `phase_rate` in your game to control motion speed.
- `seed=0` means “auto”: derive from the packed payload.

Packed layout (per layer):
- `w0`: `family:u8 | jaggedness:u8 | layer_count:u8 | parallax_rate:u8`
- `w1..w4`: `color_near`, `color_far`, `sky_zenith`, `sky_horizon` (RGBA8)
- `w5`: `seed:u32` (`0` = auto)
- `w6`: `phase:u16 (low16) | fog:u8 | wind:u8`

---

## Mode 4: Nebula (Fog / Clouds / Aurora / Ink / Plasma / Kaleido)

Continuous soft fields with bounded noise cost (≤2 octaves). Designed for haze/fog layers and “gallery” abstract looks.

Enums:
- `family`: `0`=Fog, `1`=Clouds, `2`=Aurora, `3`=Ink, `4`=Plasma/Blobs, `5`=Kaleido

### env_nebula

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_nebula(
    layer: u32,
    family: u32,
    coverage: u32,
    softness: u32,
    intensity: u32,
    scale: u32,
    detail: u32,
    warp: u32,
    flow: u32,
    parallax: u32,
    height_bias: u32,
    contrast: u32,
    color_a: u32,
    color_b: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    seed: u32,
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_nebula(
    uint32_t layer,
    uint32_t family,
    uint32_t coverage,
    uint32_t softness,
    uint32_t intensity,
    uint32_t scale,
    uint32_t detail,
    uint32_t warp,
    uint32_t flow,
    uint32_t parallax,
    uint32_t height_bias,
    uint32_t contrast,
    uint32_t color_a,
    uint32_t color_b,
    float axis_x,
    float axis_y,
    float axis_z,
    uint32_t phase,
    uint32_t seed
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_nebula(
    layer: u32,
    family: u32,
    coverage: u32,
    softness: u32,
    intensity: u32,
    scale: u32,
    detail: u32,
    warp: u32,
    flow: u32,
    parallax: u32,
    height_bias: u32,
    contrast: u32,
    color_a: u32,
    color_b: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    seed: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Notes:
- `phase` is treated as `u16` (wraps); motion is designed to be loopable (closed path) rather than “scroll forever”.
- `parallax` also selects bounded internal depth slices: `0–95` → 1 slice, `96–191` → 2 slices, `192–255` → 3 slices (far slices are calmer + less emissive).
- `seed=0` means “auto”: derive from the packed payload.

Packed layout (per layer):
- `w0`: `family:u8 | coverage:u8 | softness:u8 | intensity:u8`
- `w1`: `scale:u8 | detail:u8 | warp:u8 | flow:u8`
- `w2..w3`: `color_a`, `color_b` (RGBA8)
- `w4`: `height_bias:u8 | contrast:u8 | parallax:u8 | reserved:u8` (**reserved must be zero**)
- `w5`: `axis_oct16 (low16) | phase:u16 (high16)`
- `w6`: `seed:u32` (`0` = auto)

---

## Mode 5: Room (Interior Box)

Interior volume mode: ray-box mapping from a packed viewer position, with directional lighting, seams/panels, and loopable animated accents.

Enums:
- `accent_mode`: `0`=Seams, `1`=Sweep, `2`=Seams+Sweep, `3`=Pulse

### env_room

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_room(
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
    light_tint: u32,
    corner_darken: u32,
    room_scale: f32,
    viewer_x: i32,
    viewer_y: i32,
    viewer_z: i32,
    accent: u32,
    accent_mode: u32,
    roughness: u32,
    phase: u32,
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
    uint32_t light_tint,
    uint32_t corner_darken,
    float room_scale,
    int32_t viewer_x,
    int32_t viewer_y,
    int32_t viewer_z,
    uint32_t accent,
    uint32_t accent_mode,
    uint32_t roughness,
    uint32_t phase
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
    light_tint: u32,
    corner_darken: u32,
    room_scale: f32,
    viewer_x: i32,
    viewer_y: i32,
    viewer_z: i32,
    accent: u32,
    accent_mode: u32,
    roughness: u32,
    phase: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Notes:
- `color_*` and `light_tint` are `0xRRGGBB00` (alpha byte is not used as color alpha).
- `viewer_x/y/z` are snorm8-ish offsets: `-128..127` maps to roughly `-1..1` inside the room.
- `phase` is treated as `u16` (wraps).

Packed layout (per layer):
- `w0`: `color_ceiling_RGB(24) | viewer_x_snorm8(8)`
- `w1`: `color_floor_RGB(24) | viewer_y_snorm8(8)`
- `w2`: `color_walls_RGB(24) | viewer_z_snorm8(8)`
- `w3`: `panel_size:f16 (low16) | panel_gap:u8 | corner_darken:u8`
- `w4`: `light_dir_oct16 (low16) | light_intensity:u8 | room_scale_u8`
- `w5`: `accent_mode:u8 | accent:u8 | phase:u16 (high16)`
- `w6`: `light_tint_RGB(24) | roughness:u8`

---

## Mode 6: Veil (Bands / Pillars / Drapes / Shards)

Axis-aligned SDF bands with bounded depth slices (1–3) and fwidth-based AA.

Enums:
- `family`: `0`=Pillars/Trunks, `1`=Drapes/Ribbons, `2`=Shards/Crystals, `3`=Soft Veils

### env_veil

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_veil(
    layer: u32,
    family: u32,
    density: u32,
    width: u32,
    taper: u32,
    curvature: u32,
    edge_soft: u32,
    height_min: u32,
    height_max: u32,
    color_near: u32,
    color_far: u32,
    glow: u32,
    parallax: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    seed: u32,
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_veil(
    uint32_t layer,
    uint32_t family,
    uint32_t density,
    uint32_t width,
    uint32_t taper,
    uint32_t curvature,
    uint32_t edge_soft,
    uint32_t height_min,
    uint32_t height_max,
    uint32_t color_near,
    uint32_t color_far,
    uint32_t glow,
    uint32_t parallax,
    float axis_x,
    float axis_y,
    float axis_z,
    uint32_t phase,
    uint32_t seed
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_veil(
    layer: u32,
    family: u32,
    density: u32,
    width: u32,
    taper: u32,
    curvature: u32,
    edge_soft: u32,
    height_min: u32,
    height_max: u32,
    color_near: u32,
    color_far: u32,
    glow: u32,
    parallax: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    seed: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Notes:
- `density=0` disables the layer (alpha=0).
- `height_min`/`height_max` gate by dot-height along `axis`; if min > max, they are swapped.
- `parallax` selects 1–3 bounded depth slices (see inspector presets).
- `seed=0` means “auto”: derive from the packed payload.

Packed layout (per layer):
- `w0`: `family:u8 | density:u8 | width:u8 | taper:u8`
- `w1`: `curvature:u8 | edge_soft:u8 | height_min:u8 | height_max:u8`
- `w2..w3`: `color_near`, `color_far` (RGBA8)
- `w4`: `glow:u8 | parallax:u8 | reserved:u16` (**reserved must be zero**)
- `w5`: `axis_oct16 (low16) | phase:u16 (high16)`
- `w6`: `seed:u32` (`0` = auto)

---

## Mode 7: Rings (Portals / Tunnels / Vortex / Radar)

Crisp focal rings with trig-free radial distance + pseudo-azimuth mapping, plus secondary motion knobs.

Enums:
- `family`: `0`=Portal, `1`=Tunnel, `2`=Hypnotic, `3`=Radar

### env_rings

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn env_rings(
    layer: u32,
    family: u32,
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
    phase: u32,
    wobble: u32,
    noise: u32,
    dash: u32,
    glow: u32,
    seed: u32,
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void env_rings(
    uint32_t layer,
    uint32_t family,
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
    uint32_t phase,
    uint32_t wobble,
    uint32_t noise,
    uint32_t dash,
    uint32_t glow,
    uint32_t seed
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn env_rings(
    layer: u32,
    family: u32,
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
    phase: u32,
    wobble: u32,
    noise: u32,
    dash: u32,
    glow: u32,
    seed: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

Notes:
- `phase` is treated as `u16` (wraps).
- `wobble` is treated as `u16` (wraps).
- `seed` is stored as `u8`; `seed=0` means “auto”.

Packed layout (per layer):
- `w0`: `ring_count:u8 | thickness:u8 | center_falloff:u8 | family:u8`
- `w1..w3`: `color_a`, `color_b`, `center_color` (RGBA8)
- `w4`: `spiral_twist:f16 (low16) | axis_oct16 (high16)`
- `w5`: `phase:u16 (low16) | wobble:u16 (high16)`
- `w6`: `noise:u8 | dash:u8 | glow:u8 | seed:u8`

---

## Curated Presets (Small Starter Set)

These are directly from the mode sheets (see the inspector examples for full preset sets and UI controls).

```rust
// Mode 0: Gradient — Clear Day
env_gradient(0, 0x2E65FFFF, 0xA9D8FFFF, 0x4D8B4DFF, 0x102010FF, 0.35, 0.00, 0.95, 10, 72, 230, 32, 24, 40, 0);

// Mode 1: Cells — Starfield Calm (Particles/Stars)
env_cells(1, 0, 0, 120, 2, 10, 200, 220, 64, 140, 100, 40, 0xDDE6FFFF, 0xFFF2C0FF, 0, 0);

// Mode 2: Lines — Synth Grid
env_lines(1, 0, 2, 18, 2.25, 80.0, 0, 0x00FFB0C0, 0xFF3AF0FF, 8, 0, 0, 24, 0, 96, 0.0, 0.0, 1.0, 0x4D2F5A10);

// Mode 3: Silhouette — Mountain Range (Dusk Layers)
env_silhouette(0, 0, 170, 3, 0x141422FF, 0x2B2E45FF, 0x0B1538FF, 0xD9774FFF, 170, 0, 0, 160, 0);

// Mode 4: Nebula — Foggy Dawn
env_nebula(1, 0, 170, 220, 10, 190, 40, 30, 70, 0, 128, 35, 0xA9B9C7FF, 0xF2B59CFF, 0.0, 1.0, 0.0, 0, 0);

// Mode 5: Room — Sterile Lab
env_room(0, 0xEAF4FF00, 0xC7CCD200, 0xE2E6EA00, 0.65, 42, 0.0, -1.0, 0.0, 210, 0xD8F4FF00, 35, 2.6, 0, 0, 0, 80, 0, 90, 0);

// Mode 6: Veil — Neon Drapes
env_veil(1, 1, 140, 28, 190, 170, 80, 88, 248, 0xFF2BD6FF, 0x00E5FFFF, 220, 200, 0.15, 0.97, 0.18, 0, 0);

// Mode 7: Rings — Stargate Portal
env_rings(1, 0, 48, 28, 0x2EE7FFFF, 0x0B2B4CFF, 0xE8FFFFFF, 190, 25.0, 0.0, 0.0, 1.0, 0, 9000, 32, 24, 160, 41);
```
