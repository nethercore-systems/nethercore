# Graphics Configuration

Console configuration and render state functions.

## Configuration (Init-Only)

These functions **must be called in `init()`** and cannot be changed at runtime.

### set_resolution

Sets the render resolution.

**Signature:**
```rust
fn set_resolution(res: u32)
```

**Parameters:**

| Value | Resolution |
|-------|------------|
| 0 | 360p (640x360) |
| 1 | 540p (960x540) - **default** |
| 2 | 720p (1280x720) |
| 3 | 1080p (1920x1080) |

**Constraints:** Init-only. Cannot be changed after `init()` returns.

**Example:**
```rust
fn init() {
    set_resolution(2); // 720p
}
```

---

### set_tick_rate

Sets the game's tick rate (updates per second).

**Signature:**
```rust
fn set_tick_rate(fps: u32)
```

**Parameters:**

| Value | Tick Rate |
|-------|-----------|
| 0 | 24 fps |
| 1 | 30 fps |
| 2 | 60 fps - **default** |
| 3 | 120 fps |

**Constraints:** Init-only. Affects GGRS synchronization.

**Example:**
```rust
fn init() {
    set_tick_rate(2); // 60 fps
}
```

---

### set_clear_color

Sets the background clear color.

**Signature:**
```rust
fn set_clear_color(color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| color | `u32` | RGBA color as `0xRRGGBBAA` |

**Constraints:** Init-only. Default is `0x000000FF` (black).

**Example:**
```rust
fn init() {
    set_clear_color(0x1a1a2eFF); // Dark blue
    set_clear_color(0x87CEEBFF); // Sky blue
}
```

---

### render_mode

Sets the rendering mode (shader pipeline).

**Signature:**
```rust
fn render_mode(mode: u32)
```

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | Unlit | Flat colors, no lighting |
| 1 | Matcap | Pre-baked lighting via matcap textures |
| 2 | Metallic-Roughness | PBR-style Blinn-Phong with MRE textures |
| 3 | Specular-Shininess | Traditional Blinn-Phong |

**Constraints:** Init-only. Default is mode 0 (Unlit).

**Example:**
```rust
fn init() {
    render_mode(2); // PBR-style lighting
}
```

**See Also:** [Render Modes Guide](../guides/render-modes.md)

---

## Render State

These functions can be called anytime during `render()` to change draw state.

### set_color

Sets the uniform tint color for subsequent draws.

**Signature:**
```rust
fn set_color(color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| color | `u32` | RGBA color as `0xRRGGBBAA` |

**Example:**
```rust
fn render() {
    // White (no tint)
    set_color(0xFFFFFFFF);
    draw_mesh(model);

    // Red tint
    set_color(0xFF0000FF);
    draw_mesh(enemy);

    // 50% transparent
    set_color(0xFFFFFF80);
    draw_mesh(ghost);
}
```

---

### depth_test

Enables or disables depth testing.

**Signature:**
```rust
fn depth_test(enabled: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| enabled | `u32` | `1` to enable, `0` to disable |

**Example:**
```rust
fn render() {
    // 3D scene with depth
    depth_test(1);
    draw_mesh(level);
    draw_mesh(player);

    // UI overlay without depth
    depth_test(0);
    draw_sprite(0.0, 0.0, 100.0, 50.0, 0xFFFFFFFF);
}
```

---

### cull_mode

Sets face culling mode.

**Signature:**
```rust
fn cull_mode(mode: u32)
```

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | None | Draw both sides |
| 1 | Back | Cull back faces (default) |
| 2 | Front | Cull front faces |

**Example:**
```rust
fn render() {
    // Normal geometry
    cull_mode(1); // Back-face culling
    draw_mesh(solid_object);

    // Skybox (inside-out)
    cull_mode(2); // Front-face culling
    draw_mesh(skybox);

    // Double-sided foliage
    cull_mode(0); // No culling
    draw_mesh(leaves);
}
```

---

### blend_mode

Sets the alpha blending mode.

**Signature:**
```rust
fn blend_mode(mode: u32)
```

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | None | No blending (opaque) |
| 1 | Alpha | Standard transparency |
| 2 | Additive | Add colors (glow effects) |
| 3 | Multiply | Multiply colors (shadows) |

**Example:**
```rust
fn render() {
    // Opaque geometry first
    blend_mode(0);
    draw_mesh(level);
    draw_mesh(player);

    // Transparent objects (sorted back-to-front)
    blend_mode(1);
    draw_mesh(window);

    // Additive glow effects
    blend_mode(2);
    draw_mesh(fire_particles);
    draw_mesh(laser_beam);
}
```

---

### texture_filter

Sets texture filtering mode.

**Signature:**
```rust
fn texture_filter(filter: u32)
```

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | Nearest | Pixelated (retro look) |
| 1 | Linear | Smooth (modern look) |

**Example:**
```rust
fn render() {
    // Pixel art sprites
    texture_filter(0);
    draw_sprite(0.0, 0.0, 64.0, 64.0, 0xFFFFFFFF);

    // Photo textures
    texture_filter(1);
    draw_mesh(realistic_model);
}
```

---

### uniform_alpha

Sets the dither alpha level for PS1-style transparency.

**Signature:**
```rust
fn uniform_alpha(level: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| level | `u32` | Alpha level 0-15 (0 = invisible, 15 = opaque) |

**Example:**
```rust
fn render() {
    // Fade in effect
    let alpha = (fade_progress * 15.0) as u32;
    uniform_alpha(alpha);
    draw_mesh(fading_object);

    // Reset to fully opaque
    uniform_alpha(15);
}
```

**See Also:** [dither_offset](#dither_offset)

---

### dither_offset

Sets the dither pattern offset for animated dithering.

**Signature:**
```rust
fn dither_offset(x: u32, y: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x | `u32` | X offset 0-3 |
| y | `u32` | Y offset 0-3 |

**Example:**
```rust
fn render() {
    // Animate dither pattern for shimmer effect
    let frame = tick_count() as u32;
    dither_offset(frame % 4, (frame / 4) % 4);
}
```

---

## Complete Example

```rust
fn init() {
    // Configure console
    set_resolution(1);        // 540p
    set_tick_rate(2);         // 60 fps
    set_clear_color(0x1a1a2eFF);
    render_mode(2);           // PBR lighting
}

fn render() {
    // Draw 3D scene
    depth_test(1);
    cull_mode(1);
    blend_mode(0);
    texture_filter(1);

    set_color(0xFFFFFFFF);
    draw_mesh(level);
    draw_mesh(player);

    // Draw transparent water
    blend_mode(1);
    set_color(0x4080FF80);
    draw_mesh(water);

    // Draw UI (no depth, alpha blending)
    depth_test(0);
    texture_filter(0);
    draw_sprite(10.0, 10.0, 200.0, 50.0, 0xFFFFFFFF);
}
```
