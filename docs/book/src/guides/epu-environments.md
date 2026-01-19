# EPU Environments

The Environment Processing Unit (EPU) is ZX's GPU-driven procedural background and ambient environment system.

- It renders an infinite environment when you call `epu_draw(env_id)`.
- The same environment is sampled by lit shaders for ambient/reflection lighting.

For exact FFI signatures and instruction encoding, see the [Environment (EPU) API](../api/epu.md).

For the full specification (Rust builder API, WGSL code, data model), see the [EPU RFC](../../../../EPU%20RFC.md).

---

## Quick Start

1. Build an `EpuConfig` using the builder API (or pack instructions manually).
2. Upload the config with `epu_set(env_id, config_ptr)`.
3. Call `epu_draw(env_id)` in your `render()` function.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
use glam::Vec3;

fn init() {
    let mut builder = epu_begin();

    // Open sky with direct RGB colors
    builder.ramp_enclosure(
        Vec3::Y,                        // up vector
        Rgb24::new(135, 206, 235),      // sky: light blue
        Rgb24::new(255, 200, 150),      // wall: warm horizon
        Rgb24::new(34, 139, 34),        // floor: forest green
        10, 5,                          // ceil_y, floor_y thresholds
        180,                            // softness
        15,                             // emissive (full lighting)
    );

    // Sun glow with two-color lobe
    let sun_dir = Vec3::new(0.5, 0.7, 0.3).normalize();
    builder.lobe(
        sun_dir,
        Rgb24::new(255, 255, 200),      // core: warm white
        Rgb24::new(255, 180, 100),      // edge: orange tint
        180,                            // intensity
        32,                             // exponent
        0, 0,                           // no animation
        128,                            // edge blend
        15,                             // emissive
    );

    let config = builder.finish();
    unsafe { epu_set(0, config.layers_hi.as_ptr()); }
}

fn render() {
    unsafe {
        epu_draw(0);  // Draw environment background
        // ... draw scene geometry
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint64_t env_config[16];  // 8 hi words + 8 lo words

void init(void) {
    // Build environment config (see EPU RFC for encoding)
    epu_set(0, env_config);
}

void render(void) {
    epu_draw(0);  // Draw environment background
    // ... draw scene geometry
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var env_config: [16]u64 = undefined;

export fn init() void {
    // Build environment config (see EPU RFC for encoding)
    epu_set(0, &env_config);
}

export fn render() void {
    epu_draw(0);  // Draw environment background
    // ... draw scene geometry
}
```
{{#endtab}}

{{#endtabs}}

Tips:
- Use the Rust builder API from the EPU RFC for ergonomic config building.
- For simple cases, `draw_env()` still works (draws env_id 0).

---

## Architecture Overview

The EPU uses a 128-byte instruction-based configuration:

| Slot | Type | Recommended Use |
|------|------|------------------|
| 0-3 | Bounds | `RAMP`, `LOBE`, `BAND`, `FOG` |
| 4-7 | Features | `DECAL`, `GRID`, `SCATTER`, `FLOW` |

**Bounds** define the low-frequency envelope (enclosure gradient, directional glow, horizon bands, fog).

**Features** add high-frequency motifs (sun disk, grids, stars, clouds).

---

## Opcode Overview

| Opcode | Name | Best For | Common Role |
|--------|------|----------|-------------|
| 0x01 | RAMP | Sky/ground anchor, enclosure | Bounds slot 0 |
| 0x02 | LOBE | Sun glow, neon spill | Bounds |
| 0x03 | BAND | Horizon ring | Bounds |
| 0x04 | FOG | Atmospheric absorption | Bounds |
| 0x05 | DECAL | Sun disk, portals, signage | Feature |
| 0x06 | GRID | Panels, architectural lines | Feature |
| 0x07 | SCATTER | Stars, dust, windows | Feature |
| 0x08 | FLOW | Clouds, rain, caustics | Feature |

---

## Dual-Color System

Each layer has two RGB24 colors with independent alpha. This enables:

- **Gradients**: RAMP uses color_a for sky, color_b for floor
- **Two-tone effects**: LOBE uses color_a for core, color_b for edge tint
- **Fill + outline**: DECAL uses color_a for fill, color_b for outline
- **Color variation**: SCATTER varies between color_a and color_b per point

Example: creating a sunset sky with warm horizon

```rust
builder.ramp_enclosure(
    Vec3::Y,
    Rgb24::new(50, 80, 140),        // sky: deep blue
    Rgb24::new(255, 150, 80),       // wall: orange horizon
    Rgb24::new(30, 50, 30),         // floor: dark green
    12, 4, 200, 15,
);
```

---

## Emissive Control

The 4-bit emissive field (0-15) controls how much a layer contributes to scene lighting:

| Emissive | Effect |
|----------|--------|
| 0 | Decorative only - no lighting contribution |
| 1-7 | Subtle ambient contribution |
| 8-14 | Strong light source |
| 15 | Full emissive - lights the scene at full intensity |

This separates visual appearance from lighting behavior:

```rust
// Decorative grid that does NOT light the scene
builder.grid(GridParams {
    region: EpuRegion::Walls,
    blend: EpuBlend::Add,
    emissive: 0,                     // decorative only!
    line_color: Rgb24::new(50, 200, 255),
    bg_color: Rgb24::new(0, 0, 0),
    intensity: 60,
    // ...
});

// Neon sign that DOES light the scene
builder.decal(DecalParams {
    region: EpuRegion::Walls,
    blend: EpuBlend::Add,
    emissive: 15,                    // full lighting contribution
    fill_color: Rgb24::new(255, 0, 100),
    outline_color: Rgb24::new(255, 100, 150),
    intensity: 255,
    // ...
});
```

---

## Common Recipes

| Environment | Bounds | Features |
|-------------|--------|----------|
| Sunny meadow | RAMP + LOBE (sun) | DECAL (sun disk) + FLOW (clouds) |
| Cyberpunk alley | RAMP + LOBE x2 + FOG | GRID (panels) + DECAL (sign) + FLOW (rain) + SCATTER (windows) |
| Underwater cave | RAMP + LOBE + FOG | FLOW (caustics) + SCATTER (bubbles) |
| Space station | RAMP + LOBE + BAND | GRID (panels) + DECAL (warning) + SCATTER (indicators) |
| Void + stars | RAMP (black) | SCATTER (stars, emissive=15) |

---

## Example: Sunny Meadow

```rust
fn sunny_meadow() -> EpuConfig {
    let mut e = epu_begin();

    // Open sky enclosure
    e.ramp_enclosure(
        Vec3::Y,
        Rgb24::new(135, 206, 235),      // sky: light blue
        Rgb24::new(255, 200, 150),      // wall: warm horizon
        Rgb24::new(34, 139, 34),        // floor: forest green
        10, 5, 180, 15,
    );

    // Sun glow
    let sun_dir = Vec3::new(0.5, 0.7, 0.3).normalize();
    e.lobe(
        sun_dir,
        Rgb24::new(255, 255, 200),      // warm white
        Rgb24::new(255, 180, 100),      // orange edge
        180, 32, 0, 0, 128, 15,
    );

    // Sun disk
    e.decal(DecalParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Add,
        emissive: 15,
        shape: DecalShape::Disk,
        dir: sun_dir,
        fill_color: Rgb24::new(255, 255, 255),
        outline_color: Rgb24::new(255, 200, 100),
        intensity: 255,
        softness_q: 2,
        size: 12,
        pulse_speed: 0,
        outline_width: 20,
        fill_alpha: 15,
        outline_alpha: 8,
    });

    // Gentle clouds
    e.flow(FlowParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Screen,
        emissive: 0,                    // clouds don't emit light
        dir: Vec3::X,
        primary_color: Rgb24::new(255, 255, 255),
        secondary_color: Rgb24::new(200, 200, 220),
        intensity: 60,
        scale: 64,
        speed: 20,
        octaves: 2,
        pattern: FlowPattern::Noise,
        color_blend: 128,
        alpha: 10,
    });

    epu_finish(e)
}
```

---

## Example: Void with Stars

```rust
fn void_with_stars() -> EpuConfig {
    let mut e = epu_begin();

    // Black void
    e.ramp_enclosure(
        Vec3::Y,
        Rgb24::new(0, 0, 0),            // sky: black
        Rgb24::new(0, 0, 0),            // wall: black
        Rgb24::new(0, 0, 0),            // floor: black
        15, 0, 10, 15,
    );

    // Stars are the only light source
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        emissive: 15,                   // stars light the scene
        base_color: Rgb24::new(255, 255, 255),
        var_color: Rgb24::new(200, 220, 255),
        intensity: 255,
        density: 200,
        size: 20,
        twinkle_q: 8,
        seed: 3,
        color_variation: 80,
        alpha: 15,
    });

    epu_finish(e)
}
```

---

## Example: Cyberpunk Alley

```rust
fn cyberpunk_alley() -> EpuConfig {
    let mut e = epu_begin();

    // Dark enclosed alley
    e.ramp_enclosure(
        Vec3::Y,
        Rgb24::new(10, 10, 20),         // sky: dark blue
        Rgb24::new(30, 25, 35),         // wall: dark purple
        Rgb24::new(15, 15, 20),         // floor: dark gray
        12, 3, 80, 8,
    );

    // Left neon (pink)
    e.lobe(
        Vec3::new(-1.0, 0.3, 0.0).normalize(),
        Rgb24::new(255, 50, 150),       // hot pink
        Rgb24::new(255, 100, 200),      // soft pink edge
        200, 16, 0, 0, 100, 15,
    );

    // Right neon (cyan)
    e.lobe(
        Vec3::new(1.0, 0.3, 0.0).normalize(),
        Rgb24::new(0, 255, 255),        // cyan
        Rgb24::new(100, 255, 255),      // soft cyan edge
        180, 20, 0, 0, 80, 15,
    );

    // Atmospheric fog (use MULTIPLY blend)
    e.fog(
        Vec3::Y,
        Rgb24::new(100, 80, 120),       // purple fog
        Rgb24::new(80, 60, 100),        // darker horizon
        40, 128, 100, 60, 0,            // fog doesn't emit
    );

    // Neon grid on walls (decorative, no lighting)
    e.grid(GridParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add,
        emissive: 0,                    // decorative only
        line_color: Rgb24::new(50, 200, 255),
        bg_color: Rgb24::new(0, 0, 0),
        intensity: 40,
        scale: 32,
        thickness: 20,
        pattern: GridPattern::Grid,
        scroll_q: 0,
        cell_fill: 0,
        line_alpha: 15,
        bg_alpha: 0,
    });

    epu_finish(e)
}
```

---

## Example: Underwater Cave

```rust
fn underwater_cave() -> EpuConfig {
    let mut e = epu_begin();

    // Blue-green enclosed space
    e.ramp_enclosure(
        Vec3::Y,
        Rgb24::new(20, 60, 80),         // sky: deep blue
        Rgb24::new(30, 50, 60),         // wall: murky blue-green
        Rgb24::new(20, 40, 50),         // floor: dark blue
        14, 2, 120, 10,
    );

    // Light from above
    e.lobe(
        Vec3::Y,
        Rgb24::new(100, 200, 255),      // blue-white
        Rgb24::new(50, 150, 200),       // deeper blue edge
        120, 24, 0, 0, 100, 12,
    );

    // Heavy fog/murk
    e.fog(
        Vec3::Y,
        Rgb24::new(30, 80, 100),        // blue-green murk
        Rgb24::new(20, 60, 80),         // darker at horizon
        80, 100, 120, 80, 0,
    );

    // Caustics
    e.flow(FlowParams {
        region: EpuRegion::SkyWalls,
        blend: EpuBlend::Add,
        emissive: 8,                    // subtle lighting contribution
        dir: Vec3::new(0.0, -1.0, 0.5).normalize(),
        primary_color: Rgb24::new(150, 220, 255),
        secondary_color: Rgb24::new(100, 180, 220),
        intensity: 80,
        scale: 80,
        speed: 40,
        octaves: 2,
        pattern: FlowPattern::Caustic,
        color_blend: 100,
        alpha: 12,
    });

    // Bubbles
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add,
        emissive: 0,                    // bubbles don't emit
        base_color: Rgb24::new(200, 230, 255),
        var_color: Rgb24::new(150, 200, 230),
        intensity: 100,
        density: 60,
        size: 30,
        twinkle_q: 4,
        seed: 7,
        color_variation: 40,
        alpha: 8,
    });

    epu_finish(e)
}
```

---

## Region Masks

Features can target specific regions using the 3-bit region mask:

```rust
// Grid only on walls
builder.grid(GridParams {
    region: EpuRegion::Walls,
    // ...
});

// Stars everywhere
builder.scatter(ScatterParams {
    region: EpuRegion::All,
    // ...
});

// Caustics on sky and walls (not floor)
builder.flow(FlowParams {
    region: EpuRegion::SkyWalls,
    // ...
});
```

Available regions:
- `All` (0b111) - everywhere
- `Sky` (0b100) - ceiling/sky only
- `Walls` (0b010) - horizon belt only
- `Floor` (0b001) - ground only
- `SkyWalls` (0b110), `SkyFloor` (0b101), `WallsFloor` (0b011) - combinations

---

## Multi-Environment Support

The EPU supports up to 256 environment slots. Use different `env_id` values for:

- Split-screen viewports (each player sees different environment)
- Indoor/outdoor transitions
- Portal effects

```rust
// Viewport 1: outdoor
epu_draw(0);
draw_scene();

// Viewport 2: indoor
epu_draw(1);
draw_scene();
```

---

## Ambient Lighting

Use `epu_get_ambient()` for custom lighting calculations:

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
let ambient = unsafe { epu_get_ambient(0, normal.x, normal.y, normal.z) };
let r = ((ambient >> 24) & 0xFF) as f32 / 255.0;
let g = ((ambient >> 16) & 0xFF) as f32 / 255.0;
let b = ((ambient >> 8) & 0xFF) as f32 / 255.0;
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t ambient = epu_get_ambient(0, normal.x, normal.y, normal.z);
float r = (float)((ambient >> 24) & 0xFF) / 255.0f;
float g = (float)((ambient >> 16) & 0xFF) / 255.0f;
float b = (float)((ambient >> 8) & 0xFF) / 255.0f;
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const ambient = epu_get_ambient(0, normal.x, normal.y, normal.z);
const r = @intToFloat(f32, (ambient >> 24) & 0xFF) / 255.0;
const g = @intToFloat(f32, (ambient >> 16) & 0xFF) / 255.0;
const b = @intToFloat(f32, (ambient >> 8) & 0xFF) / 255.0;
```
{{#endtab}}

{{#endtabs}}

For most use cases, the automatic EPU lighting is sufficient.

---

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Shimmer in reflections | Reduce high-frequency detail in features |
| Too noisy | Reduce `intensity` or `density` parameters |
| Features not visible | Check `region` mask matches viewing direction |
| Features not lighting objects | Set `emissive` > 0 for layers that should contribute to lighting |
| Fog not absorbing | Use `blend = MULTIPLY` for fog layers |
| Colors look washed out | Check alpha values (0-15, where 15 = opaque) |

---

## See Also

- [EPU API Reference](../api/epu.md) - FFI signatures and instruction encoding
- [EPU Architecture Overview](../architecture/epu-overview.md) - Compute pipeline details
- [EPU RFC](../../../../EPU%20RFC.md) - Full specification with Rust builder API
