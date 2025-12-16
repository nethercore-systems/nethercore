# Lighting Functions

Dynamic lighting for Modes 2 and 3 (up to 4 lights).

## Directional Lights

### light_set

Sets a directional light direction.

**Signature:**
```rust
fn light_set(index: u32, x: f32, y: f32, z: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| x, y, z | `f32` | Light direction (from light, will be normalized) |

**Example:**
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

---

### light_color

Sets a light's color.

**Signature:**
```rust
fn light_color(index: u32, color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| color | `u32` | Light color as `0xRRGGBBAA` |

**Example:**
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

---

### light_intensity

Sets a light's intensity.

**Signature:**
```rust
fn light_intensity(index: u32, intensity: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| intensity | `f32` | Light intensity (0.0-8.0, default 1.0) |

**Example:**
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

---

### light_enable

Enables a light.

**Signature:**
```rust
fn light_enable(index: u32)
```

**Example:**
```rust
fn render() {
    // Enable lights 0 and 1
    light_enable(0);
    light_enable(1);
}
```

---

### light_disable

Disables a light.

**Signature:**
```rust
fn light_disable(index: u32)
```

**Example:**
```rust
fn render() {
    // Disable light 2 when entering dark area
    if in_dark_zone {
        light_disable(2);
    }
}
```

---

## Point Lights

### light_set_point

Sets a point light position.

**Signature:**
```rust
fn light_set_point(index: u32, x: f32, y: f32, z: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| x, y, z | `f32` | World position of the light |

**Example:**
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

---

### light_range

Sets a point light's falloff range.

**Signature:**
```rust
fn light_range(index: u32, range: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| index | `u32` | Light index (0-3) |
| range | `f32` | Maximum range/falloff distance |

**Example:**
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

---

## Standard Lighting Setups

### Three-Point Lighting

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

### Outdoor Sunlight

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

### Indoor Point Lights

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

### Dynamic Torch Effect

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

---

## Lighting Notes

- **Maximum 4 lights** (indices 0-3)
- **Directional lights** have no position, only direction
- **Point lights** have position and range falloff
- **Sun lighting** comes from `sky_set_sun()` in addition to explicit lights
- **Ambient** comes from the procedural sky automatically
- Works only in **Mode 2** (Metallic-Roughness) and **Mode 3** (Specular-Shininess)

**See Also:** [Sky Functions](./sky.md), [Materials](./materials.md), [Render Modes Guide](../guides/render-modes.md)
