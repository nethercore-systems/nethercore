# Lighting Functions

Dynamic lighting for Modes 2 and 3 (up to 4 lights).

## Directional Lights

### light_set

Sets a directional light direction.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn light_set(index: u32, x: f32, y: f32, z: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void light_set(uint32_t index, float x, float y, float z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn light_set(index: u32, x: f32, y: f32, z: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| x, y, z | `f32` | Light direction (from light, will be normalized) |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Sun from upper right
    light_set(0, 0.5, -0.7, 0.5);
    light_enable(0);

    // Fill light from left
    light_set(1, -0.8, -0.2, 0.0);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Sun from upper right
    light_set(0, 0.5f, -0.7f, 0.5f);
    light_enable(0);

    // Fill light from left
    light_set(1, -0.8f, -0.2f, 0.0f);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Sun from upper right
    light_set(0, 0.5, -0.7, 0.5);
    light_enable(0);

    // Fill light from left
    light_set(1, -0.8, -0.2, 0.0);
    light_enable(1);
}
```
{{#endtab}}

{{#endtabs}}

---

### light_color

Sets a light's color.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn light_color(index: u32, color: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void light_color(uint32_t index, uint32_t color);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn light_color(index: u32, color: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| color | `u32` | Light color as `0xRRGGBBAA` |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Warm sunlight
    light_color(0, 0xFFF2E6FF);

    // Cool fill light
    light_color(1, 0xB3D9FFFF);

    // Red emergency light
    light_color(2, 0xFF3333FF);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Warm sunlight
    light_color(0, 0xFFF2E6FF);

    // Cool fill light
    light_color(1, 0xB3D9FFFF);

    // Red emergency light
    light_color(2, 0xFF3333FF);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Warm sunlight
    light_color(0, 0xFFF2E6FF);

    // Cool fill light
    light_color(1, 0xB3D9FFFF);

    // Red emergency light
    light_color(2, 0xFF3333FF);
}
```
{{#endtab}}

{{#endtabs}}

---

### light_intensity

Sets a light's intensity.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn light_intensity(index: u32, intensity: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void light_intensity(uint32_t index, float intensity);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn light_intensity(index: u32, intensity: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| intensity | `f32` | Light intensity (0.0-8.0, default 1.0) |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Bright main light
    light_intensity(0, 1.2);

    // Dim fill light
    light_intensity(1, 0.3);

    // Flickering torch
    let flicker = 0.8 + (elapsed_time() * 10.0).sin() * 0.2;
    light_intensity(2, flicker);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
#include <math.h>

EWZX_EXPORT void render(void) {
    // Bright main light
    light_intensity(0, 1.2f);

    // Dim fill light
    light_intensity(1, 0.3f);

    // Flickering torch
    float flicker = 0.8f + sinf(elapsed_time() * 10.0f) * 0.2f;
    light_intensity(2, flicker);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
const std = @import("std");

export fn render() void {
    // Bright main light
    light_intensity(0, 1.2);

    // Dim fill light
    light_intensity(1, 0.3);

    // Flickering torch
    const flicker = 0.8 + @sin(elapsed_time() * 10.0) * 0.2;
    light_intensity(2, flicker);
}
```
{{#endtab}}

{{#endtabs}}

---

### light_enable

Enables a light.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn light_enable(index: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void light_enable(uint32_t index);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn light_enable(index: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Enable lights 0 and 1
    light_enable(0);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Enable lights 0 and 1
    light_enable(0);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Enable lights 0 and 1
    light_enable(0);
    light_enable(1);
}
```
{{#endtab}}

{{#endtabs}}

---

### light_disable

Disables a light.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn light_disable(index: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void light_disable(uint32_t index);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn light_disable(index: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Disable light 2 when entering dark area
    if in_dark_zone {
        light_disable(2);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Disable light 2 when entering dark area
    if (in_dark_zone) {
        light_disable(2);
    }
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Disable light 2 when entering dark area
    if (in_dark_zone) {
        light_disable(2);
    }
}
```
{{#endtab}}

{{#endtabs}}

---

## Point Lights

### light_set_point

Sets a point light position.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn light_set_point(index: u32, x: f32, y: f32, z: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void light_set_point(uint32_t index, float x, float y, float z);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn light_set_point(index: u32, x: f32, y: f32, z: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| x, y, z | `f32` | World position of the light |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Torch at fixed position
    light_set_point(0, 5.0, 2.0, 3.0);
    light_color(0, 0xFFAA66FF);
    light_range(0, 10.0);
    light_enable(0);

    // Light following player
    light_set_point(1, player.x, player.y + 1.0, player.z);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Torch at fixed position
    light_set_point(0, 5.0f, 2.0f, 3.0f);
    light_color(0, 0xFFAA66FF);
    light_range(0, 10.0f);
    light_enable(0);

    // Light following player
    light_set_point(1, player.x, player.y + 1.0f, player.z);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Torch at fixed position
    light_set_point(0, 5.0, 2.0, 3.0);
    light_color(0, 0xFFAA66FF);
    light_range(0, 10.0);
    light_enable(0);

    // Light following player
    light_set_point(1, player.x, player.y + 1.0, player.z);
    light_enable(1);
}
```
{{#endtab}}

{{#endtabs}}

---

### light_range

Sets a point light's falloff range.

**Signature:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn light_range(index: u32, range: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_IMPORT void light_range(uint32_t index, float range);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn light_range(index: u32, range: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| range | `f32` | Maximum range/falloff distance |

**Example:**
{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Small candle
    light_set_point(0, candle_x, candle_y, candle_z);
    light_range(0, 3.0);
    light_intensity(0, 0.5);

    // Large bonfire
    light_set_point(1, fire_x, fire_y, fire_z);
    light_range(1, 15.0);
    light_intensity(1, 2.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Small candle
    light_set_point(0, candle_x, candle_y, candle_z);
    light_range(0, 3.0f);
    light_intensity(0, 0.5f);

    // Large bonfire
    light_set_point(1, fire_x, fire_y, fire_z);
    light_range(1, 15.0f);
    light_intensity(1, 2.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Small candle
    light_set_point(0, candle_x, candle_y, candle_z);
    light_range(0, 3.0);
    light_intensity(0, 0.5);

    // Large bonfire
    light_set_point(1, fire_x, fire_y, fire_z);
    light_range(1, 15.0);
    light_intensity(1, 2.0);
}
```
{{#endtab}}

{{#endtabs}}

---

## Standard Lighting Setups

### Three-Point Lighting

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn setup_lighting() {
    // Key light (main light source)
    light_set(0, 0.5, -0.7, 0.5);
    light_color(0, 0xFFF2E6FF);  // Warm white
    light_intensity(0, 1.0);
    light_enable(0);

    // Fill light (soften shadows)
    light_set(1, -0.8, -0.3, 0.2);
    light_color(1, 0xB3D9FFFF);  // Cool blue
    light_intensity(1, 0.3);
    light_enable(1);

    // Rim/back light (separation from background)
    light_set(2, 0.0, -0.2, -1.0);
    light_color(2, 0xFFFFFFFF);
    light_intensity(2, 0.5);
    light_enable(2);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void setup_lighting(void) {
    // Key light (main light source)
    light_set(0, 0.5f, -0.7f, 0.5f);
    light_color(0, 0xFFF2E6FF);  // Warm white
    light_intensity(0, 1.0f);
    light_enable(0);

    // Fill light (soften shadows)
    light_set(1, -0.8f, -0.3f, 0.2f);
    light_color(1, 0xB3D9FFFF);  // Cool blue
    light_intensity(1, 0.3f);
    light_enable(1);

    // Rim/back light (separation from background)
    light_set(2, 0.0f, -0.2f, -1.0f);
    light_color(2, 0xFFFFFFFF);
    light_intensity(2, 0.5f);
    light_enable(2);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
fn setup_lighting() void {
    // Key light (main light source)
    light_set(0, 0.5, -0.7, 0.5);
    light_color(0, 0xFFF2E6FF); // Warm white
    light_intensity(0, 1.0);
    light_enable(0);

    // Fill light (soften shadows)
    light_set(1, -0.8, -0.3, 0.2);
    light_color(1, 0xB3D9FFFF); // Cool blue
    light_intensity(1, 0.3);
    light_enable(1);

    // Rim/back light (separation from background)
    light_set(2, 0.0, -0.2, -1.0);
    light_color(2, 0xFFFFFFFF);
    light_intensity(2, 0.5);
    light_enable(2);
}
```
{{#endtab}}

{{#endtabs}}

### Outdoor Sunlight

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Configure sun (matches sky_set_sun direction)
    light_set(0, 0.3, -0.8, 0.5);
    light_color(0, 0xFFF8E6FF);  // Warm sunlight
    light_intensity(0, 1.2);
    light_enable(0);

    // Ambient comes from sky automatically
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Configure sun (matches sky_set_sun direction)
    light_set(0, 0.3f, -0.8f, 0.5f);
    light_color(0, 0xFFF8E6FF);  // Warm sunlight
    light_intensity(0, 1.2f);
    light_enable(0);

    // Ambient comes from sky automatically
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Configure sun (matches sky_set_sun direction)
    light_set(0, 0.3, -0.8, 0.5);
    light_color(0, 0xFFF8E6FF); // Warm sunlight
    light_intensity(0, 1.2);
    light_enable(0);

    // Ambient comes from sky automatically
}
```
{{#endtab}}

{{#endtabs}}

### Indoor Point Lights

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Overhead lamp
    light_set_point(0, room_center_x, ceiling_y - 0.5, room_center_z);
    light_color(0, 0xFFE6B3FF);
    light_range(0, 8.0);
    light_intensity(0, 1.0);
    light_enable(0);

    // Desk lamp
    light_set_point(1, desk_x, desk_y + 0.5, desk_z);
    light_color(1, 0xFFFFE6FF);
    light_range(1, 3.0);
    light_intensity(1, 0.8);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
EWZX_EXPORT void render(void) {
    // Overhead lamp
    light_set_point(0, room_center_x, ceiling_y - 0.5f, room_center_z);
    light_color(0, 0xFFE6B3FF);
    light_range(0, 8.0f);
    light_intensity(0, 1.0f);
    light_enable(0);

    // Desk lamp
    light_set_point(1, desk_x, desk_y + 0.5f, desk_z);
    light_color(1, 0xFFFFE6FF);
    light_range(1, 3.0f);
    light_intensity(1, 0.8f);
    light_enable(1);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Overhead lamp
    light_set_point(0, room_center_x, ceiling_y - 0.5, room_center_z);
    light_color(0, 0xFFE6B3FF);
    light_range(0, 8.0);
    light_intensity(0, 1.0);
    light_enable(0);

    // Desk lamp
    light_set_point(1, desk_x, desk_y + 0.5, desk_z);
    light_color(1, 0xFFFFE6FF);
    light_range(1, 3.0);
    light_intensity(1, 0.8);
    light_enable(1);
}
```
{{#endtab}}

{{#endtabs}}

### Dynamic Torch Effect

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut TORCH_FLICKER: f32 = 0.0;

fn update() {
    unsafe {
        // Randomized flicker
        let r = (random() % 1000) as f32 / 1000.0;
        TORCH_FLICKER = 0.7 + r * 0.3;
    }
}

fn render() {
    unsafe {
        light_set_point(0, torch_x, torch_y, torch_z);
        light_color(0, 0xFF8833FF);
        light_range(0, 6.0 + TORCH_FLICKER);
        light_intensity(0, TORCH_FLICKER);
        light_enable(0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static float torch_flicker = 0.0f;

EWZX_EXPORT void update(void) {
    // Randomized flicker
    float r = (float)(random_u32() % 1000) / 1000.0f;
    torch_flicker = 0.7f + r * 0.3f;
}

EWZX_EXPORT void render(void) {
    light_set_point(0, torch_x, torch_y, torch_z);
    light_color(0, 0xFF8833FF);
    light_range(0, 6.0f + torch_flicker);
    light_intensity(0, torch_flicker);
    light_enable(0);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var torch_flicker: f32 = 0.0;

export fn update() void {
    // Randomized flicker
    const r = @as(f32, @floatFromInt(random_u32() % 1000)) / 1000.0;
    torch_flicker = 0.7 + r * 0.3;
}

export fn render() void {
    light_set_point(0, torch_x, torch_y, torch_z);
    light_color(0, 0xFF8833FF);
    light_range(0, 6.0 + torch_flicker);
    light_intensity(0, torch_flicker);
    light_enable(0);
}
```
{{#endtab}}

{{#endtabs}}

---

## Lighting Notes

- **Maximum 4 lights** (indices 0-3)
- **Directional lights** have no position, only direction
- **Point lights** have position and range falloff
- **Sun lighting** comes from `sky_set_sun()` in addition to explicit lights
- **Ambient** comes from the procedural sky automatically
- Works only in **Mode 2** (Metallic-Roughness) and **Mode 3** (Specular-Shininess)

**See Also:** [Sky Functions](./sky.md), [Materials](./materials.md), [Render Modes Guide](../guides/render-modes.md)
