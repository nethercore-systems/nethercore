# Sky Functions

Procedural sky rendering and environment lighting.

## Sky Configuration

### sky_set_colors

Sets the sky gradient colors.

**Signature:**
```rust
fn sky_set_colors(horizon_color: u32, zenith_color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| horizon_color | `u32` | Color at horizon as `0xRRGGBBAA` |
| zenith_color | `u32` | Color at top of sky as `0xRRGGBBAA` |

**Example:**
```rust
fn init() {
    // Bright day sky
    sky_set_colors(0xB2D8F2FF, 0x3366B2FF);
}

fn render() {
    // Dynamic time of day
    let t = (elapsed_time() * 0.1) % 1.0;
    if t < 0.5 {
        // Day
        sky_set_colors(0xB2D8F2FF, 0x3366B2FF);
    } else {
        // Sunset
        sky_set_colors(0xFF804DFF, 0x4D1A80FF);
    }
}
```

---

### sky_set_sun

Configures the sun for sky rendering and lighting.

**Signature:**
```rust
fn sky_set_sun(dir_x: f32, dir_y: f32, dir_z: f32, color: u32, sharpness: f32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| dir_x, dir_y, dir_z | `f32` | Sun direction (will be normalized) |
| color | `u32` | Sun color as `0xRRGGBBAA` |
| sharpness | `f32` | Sun disc sharpness (0.0-1.0, higher = smaller sun) |

**Example:**
```rust
fn init() {
    // Morning sun from the east
    sky_set_sun(0.8, 0.3, 0.0, 0xFFE6B3FF, 0.95);

    // Midday sun from above
    sky_set_sun(0.0, 1.0, 0.0, 0xFFF2E6FF, 0.98);

    // Evening sun from the west
    sky_set_sun(-0.8, 0.2, 0.0, 0xFF9933FF, 0.90);
}
```

---

### draw_sky

Renders the procedural sky as a background.

**Signature:**
```rust
fn draw_sky()
```

**Important:** Call **first** in your `render()` function, before any geometry.

**Example:**
```rust
fn render() {
    // 1. Draw sky first (renders at far plane)
    draw_sky();

    // 2. Set up camera
    camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);

    // 3. Draw scene (appears in front of sky)
    draw_mesh(terrain);
    draw_mesh(player);
}
```

---

## Matcap Textures

### matcap_set

Binds a matcap texture to a slot (Mode 1 only).

**Signature:**
```rust
fn matcap_set(slot: u32, texture: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Matcap slot (1-3) |
| texture | `u32` | Texture handle |

**Example:**
```rust
fn init() {
    render_mode(1); // Matcap mode

    // Load matcap textures
    SHADOW_MATCAP = rom_texture(b"matcap_shadow".as_ptr(), 13);
    HIGHLIGHT_MATCAP = rom_texture(b"matcap_highlight".as_ptr(), 16);
}

fn render() {
    // Bind matcaps
    matcap_set(1, SHADOW_MATCAP);
    matcap_set(2, HIGHLIGHT_MATCAP);

    // Configure blend modes
    matcap_blend_mode(1, 0); // Multiply for shadows
    matcap_blend_mode(2, 1); // Add for highlights

    // Draw
    texture_bind(character_albedo);
    draw_mesh(character);
}
```

---

## Sky Presets

### Midday

```rust
fn setup_midday() {
    sky_set_colors(0xB2CDE6FF, 0x4D80E6FF);  // Light blue → mid blue
    sky_set_sun(0.3, 0.8, 0.5, 0xFFF2E6FF, 0.98);
}
```

### Sunset

```rust
fn setup_sunset() {
    sky_set_colors(0xFF804DFF, 0x4D1A80FF);  // Orange → purple
    sky_set_sun(0.8, 0.2, 0.0, 0xFFE673FF, 0.95);
}
```

### Overcast

```rust
fn setup_overcast() {
    sky_set_colors(0x9999A6FF, 0x666673FF);  // Gray gradient
    sky_set_sun(0.0, 1.0, 0.0, 0x404040FF, 0.5);  // Dim, diffuse
}
```

### Night

```rust
fn setup_night() {
    sky_set_colors(0x0D0D1AFF, 0x03030DFF);  // Dark blue
    sky_set_sun(0.5, 0.3, 0.0, 0x8888AAFF, 0.85);  // Moon
}
```

### Dawn

```rust
fn setup_dawn() {
    sky_set_colors(0xFFB380FF, 0x4D6680FF);  // Warm orange → cool blue
    sky_set_sun(0.9, 0.1, 0.3, 0xFFCC99FF, 0.92);
}
```

---

## Complete Example

```rust
static mut TIME_OF_DAY: f32 = 0.5; // 0.0 = midnight, 0.5 = noon, 1.0 = midnight

fn update() {
    unsafe {
        // Advance time
        TIME_OF_DAY += delta_time() * 0.01; // 100 seconds per day
        if TIME_OF_DAY >= 1.0 {
            TIME_OF_DAY -= 1.0;
        }
    }
}

fn render() {
    unsafe {
        // Calculate sun position based on time
        let sun_angle = TIME_OF_DAY * 6.28318; // Full rotation
        let sun_y = sun_angle.sin();
        let sun_x = sun_angle.cos();

        // Interpolate sky colors based on time
        let (horizon, zenith, sun_color) = if TIME_OF_DAY < 0.25 {
            // Night to dawn
            let t = TIME_OF_DAY / 0.25;
            (
                lerp_color(0x0D0D1AFF, 0xFFB380FF, t),
                lerp_color(0x03030DFF, 0x4D6680FF, t),
                lerp_color(0x333355FF, 0xFFCC99FF, t),
            )
        } else if TIME_OF_DAY < 0.5 {
            // Dawn to noon
            let t = (TIME_OF_DAY - 0.25) / 0.25;
            (
                lerp_color(0xFFB380FF, 0xB2D8F2FF, t),
                lerp_color(0x4D6680FF, 0x3366B2FF, t),
                lerp_color(0xFFCC99FF, 0xFFF2E6FF, t),
            )
        } else if TIME_OF_DAY < 0.75 {
            // Noon to dusk
            let t = (TIME_OF_DAY - 0.5) / 0.25;
            (
                lerp_color(0xB2D8F2FF, 0xFF804DFF, t),
                lerp_color(0x3366B2FF, 0x4D1A80FF, t),
                lerp_color(0xFFF2E6FF, 0xFFE673FF, t),
            )
        } else {
            // Dusk to night
            let t = (TIME_OF_DAY - 0.75) / 0.25;
            (
                lerp_color(0xFF804DFF, 0x0D0D1AFF, t),
                lerp_color(0x4D1A80FF, 0x03030DFF, t),
                lerp_color(0xFFE673FF, 0x333355FF, t),
            )
        };

        sky_set_colors(horizon, zenith);
        sky_set_sun(sun_x, sun_y.max(0.1), 0.3, sun_color, 0.95);

        // Draw sky first
        draw_sky();

        // Set up camera and draw scene
        camera_set(0.0, 5.0, 15.0, 0.0, 0.0, 0.0);
        draw_mesh(terrain);
        draw_mesh(buildings);
    }
}
```

**See Also:** [Lighting](./lighting.md), [Materials](./materials.md), [Render Modes Guide](../guides/render-modes.md)
