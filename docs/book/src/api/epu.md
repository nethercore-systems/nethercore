# Environment Processing Unit (EPU) v2

The Environment Processing Unit is ZX's GPU-driven procedural environment system. It renders backgrounds when you call `epu_draw()` and provides ambient lighting data for lit shaders.

## Overview

The EPU uses a **128-byte** instruction-based configuration (8 x 128-bit instructions) evaluated by GPU compute shaders into octahedral environment maps. This provides:

- Procedural backgrounds (sky/walls/void)
- Ambient lighting data for objects (diffuse ambient + reflection color)
- Multi-environment support via `env_id` indexing
- Animation via instruction parameters
- Direct RGB24 colors (no palette indirection)
- Explicit emissive control for lighting contribution

## v2 Changes Summary

| Aspect | v1 | v2 |
|--------|----|----|
| Instruction size | 64-bit | 128-bit |
| Environment size | 64 bytes | 128 bytes |
| Opcode bits | 4-bit (16 opcodes) | 5-bit (32 opcodes) |
| Region | 2-bit enum | 3-bit combinable mask |
| Blend modes | 4 modes | 8 modes (+SCREEN, HSV_MOD, MIN, OVERLAY) |
| Color | 8-bit palette index | RGB24 x 2 per layer |
| Emissive | Implicit (ADD=emissive) | Explicit 4-bit (0-15) |
| Alpha | None | 4-bit x 2 (Bayer-friendly) |
| Parameters | 3 (param_a/b/c) | 4 (+param_d) |
| Palette buffer | Required | **REMOVED** |

---

## FFI Functions

### epu_set

Upload an environment configuration to a slot.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
/// Set an EPU environment configuration.
///
/// # Arguments
/// * `env_id` - Environment slot ID (0-255)
/// * `config_ptr` - Pointer to 16 u64 values (128 bytes total)
///                  First 8 values are high words, next 8 are low words
fn epu_set(env_id: u32, config_ptr: *const u64)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Set an EPU environment configuration.
///
/// @param env_id Environment slot ID (0-255)
/// @param config_ptr Pointer to 16 u64 values (128 bytes total)
void epu_set(uint32_t env_id, const uint64_t* config_ptr);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Set an EPU environment configuration.
/// env_id: Environment slot ID (0-255)
/// config_ptr: Pointer to 16 u64 values (128 bytes total)
pub extern fn epu_set(env_id: u32, config_ptr: [*]const u64) void;
```
{{#endtab}}

{{#endtabs}}

### epu_draw

Draw the background using the specified EPU environment.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
/// Draw the background using the specified EPU environment.
///
/// # Arguments
/// * `env_id` - Environment slot ID (0-255)
fn epu_draw(env_id: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
/// Draw the background using the specified EPU environment.
///
/// @param env_id Environment slot ID (0-255)
void epu_draw(uint32_t env_id);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
/// Draw the background using the specified EPU environment.
/// env_id: Environment slot ID (0-255)
pub extern fn epu_draw(env_id: u32) void;
```
{{#endtab}}

{{#endtabs}}

Call this **first** in your `render()` function, before any 3D geometry.

> **Note:** Ambient lighting is computed entirely on the GPU and applied automatically to 3D geometry.
> There is no CPU-accessible ambient query function because GPU readback would break rollback determinism.

---

## Builder API

The builder API provides a safer, more ergonomic way to construct EPU layers without manual bit-packing.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
epu_begin()                                      // Start building a new layer
epu_layer_opcode(opcode: u8)                     // Set opcode (0-8)
epu_layer_region(region: u8)                     // Set region mask (bitfield)
epu_layer_blend(blend: u8)                       // Set blend mode (0-7)
epu_layer_emissive(emissive: u8)                 // Set emissive level (0-15)
epu_layer_color_a(r: u8, g: u8, b: u8)           // Primary RGB24 color
epu_layer_color_b(r: u8, g: u8, b: u8)           // Secondary RGB24 color
epu_layer_alpha_a(alpha: u8)                     // Primary alpha (0-15)
epu_layer_alpha_b(alpha: u8)                     // Secondary alpha (0-15)
epu_layer_intensity(intensity: u8)               // Layer brightness (0-255)
epu_layer_params(a: u8, b: u8, c: u8, d: u8)     // Opcode-specific params
epu_layer_direction(x: i16, y: i16, z: i16)      // Direction vector
epu_finish(env_id: u8, layer_index: u8)          // Commit layer to env slot
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void epu_begin(void);                            // Start building a new layer
void epu_layer_opcode(uint8_t opcode);           // RAMP=1, LOBE=2, BAND=3, FOG=4, DECAL=5, GRID=6, SCATTER=7, FLOW=8
void epu_layer_region(uint8_t region);           // SKY=4, WALLS=2, FLOOR=1 (bitfield, ALL=7)
void epu_layer_blend(uint8_t blend);             // ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7
void epu_layer_emissive(uint8_t emissive);       // 0=decorative, 15=full lighting contribution
void epu_layer_color_a(uint8_t r, uint8_t g, uint8_t b);   // Primary RGB24
void epu_layer_color_b(uint8_t r, uint8_t g, uint8_t b);   // Secondary RGB24
void epu_layer_alpha_a(uint8_t alpha);           // 0-15 (Bayer dither compatible)
void epu_layer_alpha_b(uint8_t alpha);           // 0-15 (Bayer dither compatible)
void epu_layer_intensity(uint8_t intensity);     // 0-255
void epu_layer_params(uint8_t a, uint8_t b, uint8_t c, uint8_t d);
void epu_layer_direction(int16_t x, int16_t y, int16_t z);
void epu_finish(uint8_t env_id, uint8_t layer_index);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn epu_begin() void;
pub extern fn epu_layer_opcode(opcode: u8) void;
pub extern fn epu_layer_region(region: u8) void;
pub extern fn epu_layer_blend(blend: u8) void;
pub extern fn epu_layer_emissive(emissive: u8) void;
pub extern fn epu_layer_color_a(r: u8, g: u8, b: u8) void;
pub extern fn epu_layer_color_b(r: u8, g: u8, b: u8) void;
pub extern fn epu_layer_alpha_a(alpha: u8) void;
pub extern fn epu_layer_alpha_b(alpha: u8) void;
pub extern fn epu_layer_intensity(intensity: u8) void;
pub extern fn epu_layer_params(a: u8, b: u8, c: u8, d: u8) void;
pub extern fn epu_layer_direction(x: i16, y: i16, z: i16) void;
pub extern fn epu_finish(env_id: u8, layer_index: u8) void;
```
{{#endtab}}

{{#endtabs}}

### Builder API Example

{{#tabs global="lang"}}

{{#tab name="C/C++"}}
```c
void setup_sunset_environment(void) {
    // Layer 0: RAMP base gradient
    epu_begin();
    epu_layer_opcode(1);                    // RAMP
    epu_layer_region(7);                    // ALL regions
    epu_layer_blend(0);                     // ADD
    epu_layer_emissive(10);                 // Moderate lighting
    epu_layer_color_a(255, 140, 80);        // Sky: sunset orange
    epu_layer_color_b(30, 20, 40);          // Floor: dark purple
    epu_layer_alpha_a(15);
    epu_layer_alpha_b(15);
    epu_layer_intensity(200);
    epu_layer_params(180, 100, 0x84, 120);  // Wall: pink-ish
    epu_layer_direction(0, 32767, 0);       // Up = +Y
    epu_finish(0, 0);

    // Layer 1: Sun LOBE
    epu_begin();
    epu_layer_opcode(2);                    // LOBE
    epu_layer_region(4);                    // SKY only
    epu_layer_blend(0);                     // ADD
    epu_layer_emissive(15);                 // Full emissive
    epu_layer_color_a(255, 200, 100);       // Core: warm yellow
    epu_layer_color_b(255, 100, 50);        // Edge: deep orange
    epu_layer_alpha_a(15);
    epu_layer_alpha_b(10);
    epu_layer_intensity(255);
    epu_layer_params(200, 0, 0, 80);        // Sharp falloff
    epu_layer_direction(16384, 8192, 0);    // Sun position
    epu_finish(0, 1);
}
```
{{#endtab}}

{{#tab name="Rust"}}
```rust
fn setup_sunset_environment() {
    // Layer 0: RAMP base gradient
    epu_begin();
    epu_layer_opcode(1);                    // RAMP
    epu_layer_region(7);                    // ALL regions
    epu_layer_blend(0);                     // ADD
    epu_layer_emissive(10);
    epu_layer_color_a(255, 140, 80);        // Sky: sunset orange
    epu_layer_color_b(30, 20, 40);          // Floor: dark purple
    epu_layer_alpha_a(15);
    epu_layer_alpha_b(15);
    epu_layer_intensity(200);
    epu_layer_params(180, 100, 0x84, 120);
    epu_layer_direction(0, 32767, 0);
    epu_finish(0, 0);

    // Layer 1: Sun LOBE
    epu_begin();
    epu_layer_opcode(2);                    // LOBE
    epu_layer_region(4);                    // SKY only
    epu_layer_blend(0);                     // ADD
    epu_layer_emissive(15);
    epu_layer_color_a(255, 200, 100);
    epu_layer_color_b(255, 100, 50);
    epu_layer_alpha_a(15);
    epu_layer_alpha_b(10);
    epu_layer_intensity(255);
    epu_layer_params(200, 0, 0, 80);
    epu_layer_direction(16384, 8192, 0);
    epu_finish(0, 1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn setup_sunset_environment() void {
    // Layer 0: RAMP base gradient
    epu_begin();
    epu_layer_opcode(1);
    epu_layer_region(7);
    epu_layer_blend(0);
    epu_layer_emissive(10);
    epu_layer_color_a(255, 140, 80);
    epu_layer_color_b(30, 20, 40);
    epu_layer_alpha_a(15);
    epu_layer_alpha_b(15);
    epu_layer_intensity(200);
    epu_layer_params(180, 100, 0x84, 120);
    epu_layer_direction(0, 32767, 0);
    epu_finish(0, 0);

    // Layer 1: Sun LOBE
    epu_begin();
    epu_layer_opcode(2);
    epu_layer_region(4);
    epu_layer_blend(0);
    epu_layer_emissive(15);
    epu_layer_color_a(255, 200, 100);
    epu_layer_color_b(255, 100, 50);
    epu_layer_alpha_a(15);
    epu_layer_alpha_b(10);
    epu_layer_intensity(255);
    epu_layer_params(200, 0, 0, 80);
    epu_layer_direction(16384, 8192, 0);
    epu_finish(0, 1);
}
```
{{#endtab}}

{{#endtabs}}

---

## Configuration Layout

Each environment is exactly **8 x 128-bit instructions** (128 bytes total):

| Slot | Type | Recommended Use |
|------|------|------------------|
| 0 | Bounds | `RAMP` enclosure + base colors |
| 1 | Bounds | `LOBE` (sun/neon spill) |
| 2 | Bounds | `BAND` (horizon ring) |
| 3 | Bounds | `FOG` (absorption/haze) |
| 4 | Feature | `DECAL` (sun disk, signage, portals) |
| 5 | Feature | `GRID` (panels, architectural lines) |
| 6 | Feature | `SCATTER` (stars, dust, windows) |
| 7 | Feature | `FLOW` (clouds, rain, caustics) |

---

## Instruction Bit Layout (128-bit)

Each instruction is packed as two `u64` values:

### High Word (bits 127..64)

```
bits 127..123: opcode     (5)  - Which algorithm to run
bits 122..120: region     (3)  - Bitfield: SKY=0b100, WALLS=0b010, FLOOR=0b001
bits 119..117: blend      (3)  - How to combine layer output (8 modes)
bits 116..113: emissive   (4)  - L_light0 contribution (0=none, 15=full)
bit  112:      reserved   (1)  - Future use
bits 111..88:  color_a    (24) - RGB24 primary color
bits 87..64:   color_b    (24) - RGB24 secondary color
```

### Low Word (bits 63..0)

```
bits 63..56:   intensity  (8)  - Layer brightness
bits 55..48:   param_a    (8)  - Opcode-specific
bits 47..40:   param_b    (8)  - Opcode-specific
bits 39..32:   param_c    (8)  - Opcode-specific
bits 31..24:   param_d    (8)  - Opcode-specific
bits 23..8:    direction  (16) - Octahedral-encoded direction (u8,u8)
bits 7..4:     alpha_a    (4)  - color_a alpha (0=transparent, 15=opaque)
bits 3..0:     alpha_b    (4)  - color_b alpha (0=transparent, 15=opaque)
```

---

## Opcodes

| Opcode | Name | Kind | Purpose |
|--------|------|------|---------|
| `0x00` | `NOP` | Any | Disable layer |
| `0x01` | `RAMP` | Bounds | Enclosure gradient (sky/walls/floor) |
| `0x02` | `LOBE` | Bounds | Directional glow (sun, lamp, neon spill) |
| `0x03` | `BAND` | Bounds | Horizon band / ring |
| `0x04` | `FOG` | Bounds | Atmospheric absorption |
| `0x05` | `DECAL` | Feature | Sharp SDF shape (disk/ring/rect/line) |
| `0x06` | `GRID` | Feature | Repeating lines/panels |
| `0x07` | `SCATTER` | Feature | Point field (stars/dust/bubbles) |
| `0x08` | `FLOW` | Feature | Animated noise/streaks/caustics |
| `0x09..0x1F` | Reserved | - | Future expansion |

---

## Region Mask (3-bit bitfield)

Regions are combinable using bitwise OR:

| Value | Binary | Name | Meaning |
|-------|--------|------|---------|
| 7 | `0b111` | `ALL` | Apply to sky + walls + floor |
| 4 | `0b100` | `SKY` | Sky/ceiling only |
| 2 | `0b010` | `WALLS` | Wall/horizon belt only |
| 1 | `0b001` | `FLOOR` | Floor/ground only |
| 6 | `0b110` | `SKY_WALLS` | Sky + walls |
| 5 | `0b101` | `SKY_FLOOR` | Sky + floor |
| 3 | `0b011` | `WALLS_FLOOR` | Walls + floor |
| 0 | `0b000` | `NONE` | Layer disabled |

---

## Blend Modes (3-bit, 8 modes)

| Value | Name | Formula |
|-------|------|---------|
| 0 | `ADD` | `dst + src * a` |
| 1 | `MULTIPLY` | `dst * mix(1, src, a)` |
| 2 | `MAX` | `max(dst, src * a)` |
| 3 | `LERP` | `mix(dst, src, a)` |
| 4 | `SCREEN` | `1 - (1-dst)*(1-src*a)` |
| 5 | `HSV_MOD` | HSV shift dst by src |
| 6 | `MIN` | `min(dst, src * a)` |
| 7 | `OVERLAY` | Photoshop-style overlay |

---

## Emissive Field (4-bit)

The emissive field controls how much a layer contributes to lighting:

| Value | Contribution |
|-------|--------------|
| 0 | Decorative only (no lighting contribution) |
| 1-14 | Scaled contribution (value/15 * layer output) |
| 15 | Full emissive (100% lighting contribution) |

This allows explicit control over whether a visually bright element lights the scene.

---

## Dual-Color System

Each layer has two RGB24 colors (`color_a` and `color_b`) with independent 4-bit alpha:

| Opcode | color_a | color_b |
|--------|---------|---------|
| `RAMP` | Sky color | Floor color (wall via params) |
| `LOBE` | Core glow | Edge tint |
| `BAND` | Center color | Edge gradient |
| `FOG` | Fog tint | Horizon tint |
| `DECAL` | Fill color | Outline color |
| `GRID` | Line color | Cell background |
| `SCATTER` | Base color | Color variation |
| `FLOW` | Primary color | Secondary color |

---

## Per-Opcode Parameter Reference

### RAMP (Enclosure Gradient)

| Field | Purpose |
|-------|---------|
| `color_a` | Sky/ceiling color |
| `color_b` | Floor/ground color |
| `param_a` | Wall color R |
| `param_b` | Wall color G |
| `param_c[7:4]` | Ceiling Y threshold (0-15) |
| `param_c[3:0]` | Floor Y threshold (0-15) |
| `param_d` | Wall color B |
| `intensity` | Softness (gradient smoothness) |
| `direction` | Up vector |

### LOBE (Directional Glow)

| Field | Purpose |
|-------|---------|
| `color_a` | Core glow color |
| `color_b` | Edge tint color |
| `intensity` | Brightness |
| `param_a` | Exponent (sharpness, 0-255 maps to 1-64) |
| `param_b` | Animation speed |
| `param_c` | Animation mode (0=none, 1=pulse, 2=flicker) |
| `param_d` | Edge blend amount |
| `direction` | Lobe center direction |

### BAND (Horizon Ring)

| Field | Purpose |
|-------|---------|
| `color_a` | Center color |
| `color_b` | Edge gradient color |
| `intensity` | Brightness |
| `param_a` | Width |
| `param_b` | Vertical offset |
| `param_c` | Scroll speed |
| `param_d` | Gradient sharpness |
| `direction` | Band normal axis |

### FOG (Atmospheric Absorption)

| Field | Purpose |
|-------|---------|
| `color_a` | Fog tint color |
| `color_b` | Horizon tint color |
| `intensity` | Density |
| `param_a` | Vertical bias |
| `param_b` | Falloff curve |
| `param_c` | Horizon blend amount |
| `direction` | Up vector |

Use `blend = MULTIPLY` for fog.

### DECAL (Sharp SDF Shape)

| Field | Purpose |
|-------|---------|
| `color_a` | Fill color |
| `color_b` | Outline color |
| `intensity` | Brightness |
| `param_a[7:4]` | Shape (0=disk, 1=ring, 2=rect, 3=line) |
| `param_a[3:0]` | Edge softness |
| `param_b` | Size |
| `param_c` | Pulse animation speed |
| `param_d` | Outline width |
| `direction` | Shape center |
| `alpha_a` | Fill alpha |
| `alpha_b` | Outline alpha |

### GRID (Repeating Lines)

| Field | Purpose |
|-------|---------|
| `color_a` | Line color |
| `color_b` | Cell background color |
| `intensity` | Brightness |
| `param_a` | Scale (repetition count) |
| `param_b` | Line thickness |
| `param_c[7:4]` | Pattern (0=stripes, 1=grid, 2=checker) |
| `param_c[3:0]` | Scroll speed |
| `param_d` | Cell fill amount |
| `alpha_a` | Line alpha |
| `alpha_b` | Background alpha |

### SCATTER (Point Field)

| Field | Purpose |
|-------|---------|
| `color_a` | Base point color |
| `color_b` | Color variation |
| `intensity` | Brightness |
| `param_a` | Density |
| `param_b` | Point size |
| `param_c[7:4]` | Twinkle amount |
| `param_c[3:0]` | Random seed |
| `param_d` | Color variation amount |
| `alpha_a` | Point alpha |

### FLOW (Animated Noise)

| Field | Purpose |
|-------|---------|
| `color_a` | Primary color |
| `color_b` | Secondary color |
| `intensity` | Brightness |
| `param_a` | Scale |
| `param_b` | Animation speed |
| `param_c[7:4]` | Noise octaves (0-4) |
| `param_c[3:0]` | Pattern (0=noise, 1=streaks, 2=caustic) |
| `param_d` | Color blend amount |
| `direction` | Flow direction |
| `alpha_a` | Flow alpha |

---

## Quick Start

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
use glam::Vec3;

fn init() {
    let mut builder = epu_begin();

    // Sky gradient with direct RGB colors
    builder.ramp_enclosure(
        Vec3::Y,                        // up vector
        Rgb24::new(135, 206, 235),      // sky: light blue
        Rgb24::new(255, 200, 150),      // wall: warm horizon
        Rgb24::new(34, 139, 34),        // floor: forest green
        10,                             // ceil_y threshold
        5,                              // floor_y threshold
        180,                            // softness
        15,                             // emissive (full lighting)
    );

    // Sun glow
    let sun_dir = Vec3::new(0.5, 0.7, 0.3).normalize();
    builder.lobe(
        sun_dir,
        Rgb24::new(255, 255, 200),      // core: warm white
        Rgb24::new(255, 180, 100),      // edge: orange
        180, 32, 0, 0, 128, 15,
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
    // ...
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
var env_config: [16]u64 = undefined;  // 8 hi + 8 lo words

export fn init() void {
    // Build environment config (see EPU RFC for encoding)
    // ...
    epu_set(0, &env_config);
}

export fn render() void {
    epu_draw(0);  // Draw environment background
    // ... draw scene geometry
}
```
{{#endtab}}

{{#endtabs}}

---

## Legacy Compatibility

The `draw_env()` function is retained for backwards compatibility and draws `env_id = 0`.

---

## See Also

- [EPU Environments Guide](../guides/epu-environments.md) - Recipes and examples
- [EPU Architecture Overview](../architecture/epu-overview.md) - Compute pipeline details
- [EPU RFC](../../../../EPU%20RFC.md) - Full specification with WGSL code
