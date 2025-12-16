# Texture Functions

Loading, binding, and configuring textures.

## Loading Textures

### load_texture

Loads an RGBA8 texture from WASM memory.

**Signature:**
```rust
fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| width | `u32` | Texture width in pixels |
| height | `u32` | Texture height in pixels |
| pixels | `*const u8` | Pointer to RGBA8 pixel data (4 bytes per pixel) |

**Returns:** Texture handle (non-zero on success)

**Constraints:** Init-only. Must be called in `init()`.

**Example:**
```rust
static mut PLAYER_TEX: u32 = 0;

// Embedded pixel data (8x8 checkerboard)
const CHECKER: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let mut i = 0;
    while i < 64 {
        let x = i % 8;
        let y = i / 8;
        let white = ((x + y) % 2) == 0;
        let idx = i * 4;
        pixels[idx] = if white { 255 } else { 0 };     // R
        pixels[idx + 1] = if white { 255 } else { 0 }; // G
        pixels[idx + 2] = if white { 255 } else { 0 }; // B
        pixels[idx + 3] = 255;                          // A
        i += 1;
    }
    pixels
};

fn init() {
    unsafe {
        PLAYER_TEX = load_texture(8, 8, CHECKER.as_ptr());
    }
}
```

**Note:** Prefer `rom_texture()` for assets bundled in the ROM data pack.

**See Also:** [rom_texture](./rom-loading.md#rom_texture)

---

## Binding Textures

### texture_bind

Binds a texture to slot 0 (albedo/diffuse).

**Signature:**
```rust
fn texture_bind(handle: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| handle | `u32` | Texture handle from `load_texture()` or `rom_texture()` |

**Example:**
```rust
fn render() {
    texture_bind(player_tex);
    draw_mesh(player_model);

    texture_bind(enemy_tex);
    draw_mesh(enemy_model);
}
```

---

### texture_bind_slot

Binds a texture to a specific slot.

**Signature:**
```rust
fn texture_bind_slot(handle: u32, slot: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| handle | `u32` | Texture handle |
| slot | `u32` | Texture slot (0-3) |

**Texture Slots:**

| Slot | Purpose |
|------|---------|
| 0 | Albedo/diffuse texture |
| 1 | MRE texture (Mode 2) or Specular (Mode 3) |
| 2 | Reserved |
| 3 | Reserved |

**Example:**
```rust
fn render() {
    // Bind albedo to slot 0
    texture_bind_slot(albedo_tex, 0);

    // Bind MRE (Metallic/Roughness/Emissive) to slot 1
    texture_bind_slot(mre_tex, 1);

    draw_mesh(pbr_model);
}
```

---

## Matcap Textures

### matcap_blend_mode

Sets the blend mode for a matcap texture slot.

**Signature:**
```rust
fn matcap_blend_mode(slot: u32, mode: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| slot | `u32` | Matcap slot (1-3) |
| mode | `u32` | Blend mode |

**Blend Modes:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | Multiply | Darkens (shadows, ambient occlusion) |
| 1 | Add | Brightens (highlights, rim light) |
| 2 | HSV Modulate | Hue/saturation shift |

**Example:**
```rust
fn init() {
    render_mode(1); // Matcap mode
}

fn render() {
    // Dark matcap for shadows (multiply)
    matcap_set(1, shadow_matcap);
    matcap_blend_mode(1, 0);

    // Bright matcap for highlights (add)
    matcap_set(2, highlight_matcap);
    matcap_blend_mode(2, 1);

    texture_bind(albedo_tex);
    draw_mesh(character);
}
```

**See Also:** [matcap_set](./sky.md#matcap_set), [Render Modes Guide](../guides/render-modes.md)

---

## Texture Formats

### RGBA8

Standard 8-bit RGBA format. 4 bytes per pixel.

```rust
// Pixel layout: [R, G, B, A, R, G, B, A, ...]
let pixels: [u8; 4 * 4 * 4] = [
    255, 0, 0, 255,    // Red pixel
    0, 255, 0, 255,    // Green pixel
    0, 0, 255, 255,    // Blue pixel
    255, 255, 255, 128, // Semi-transparent white
    // ... more pixels
];
```

### Texture Tips

- **Power-of-two** dimensions recommended (8, 16, 32, 64, 128, 256, 512)
- **Texture atlases** reduce bind calls and improve batching
- Use `rom_texture()` for large textures (bypasses WASM memory)
- Use `load_texture()` only for small procedural/runtime textures

---

## Complete Example

```rust
static mut CHECKER_TEX: u32 = 0;
static mut GRADIENT_TEX: u32 = 0;

// Generate checkerboard at compile time
const CHECKER_PIXELS: [u8; 16 * 16 * 4] = {
    let mut pixels = [0u8; 16 * 16 * 4];
    let mut i = 0;
    while i < 256 {
        let x = i % 16;
        let y = i / 16;
        let white = ((x / 2 + y / 2) % 2) == 0;
        let idx = i * 4;
        let c = if white { 200 } else { 50 };
        pixels[idx] = c;
        pixels[idx + 1] = c;
        pixels[idx + 2] = c;
        pixels[idx + 3] = 255;
        i += 1;
    }
    pixels
};

// Generate gradient at compile time
const GRADIENT_PIXELS: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 8 * 8 * 4];
    let mut i = 0;
    while i < 64 {
        let x = i % 8;
        let y = i / 8;
        let idx = i * 4;
        pixels[idx] = (x * 32) as u8;     // R increases right
        pixels[idx + 1] = (y * 32) as u8; // G increases down
        pixels[idx + 2] = 128;             // B constant
        pixels[idx + 3] = 255;
        i += 1;
    }
    pixels
};

fn init() {
    unsafe {
        CHECKER_TEX = load_texture(16, 16, CHECKER_PIXELS.as_ptr());
        GRADIENT_TEX = load_texture(8, 8, GRADIENT_PIXELS.as_ptr());
    }
}

fn render() {
    unsafe {
        // Draw floor with checker texture
        texture_bind(CHECKER_TEX);
        texture_filter(0); // Nearest for crisp pixels
        push_identity();
        push_scale(10.0, 1.0, 10.0);
        draw_mesh(plane);

        // Draw object with gradient
        texture_bind(GRADIENT_TEX);
        texture_filter(1); // Linear for smooth
        push_identity();
        push_translate(0.0, 1.0, 0.0);
        draw_mesh(cube);
    }
}
```
