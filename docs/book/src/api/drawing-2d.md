# 2D Drawing Functions

Screen-space sprites, rectangles, and text rendering.

## Sprites

### draw_sprite

Draws a textured quad at screen coordinates.

**Signature:**
```rust
fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position (top-left corner) |
| w, h | `f32` | Size in pixels |
| color | `u32` | Tint color as `0xRRGGBBAA` |

**Example:**
```rust
fn render() {
    // Draw full texture
    texture_bind(player_sprite);
    draw_sprite(100.0, 100.0, 64.0, 64.0, 0xFFFFFFFF);

    // Tinted sprite
    draw_sprite(200.0, 100.0, 64.0, 64.0, 0xFF8080FF);
}
```

---

### draw_sprite_region

Draws a region of a texture (sprite sheet).

**Signature:**
```rust
fn draw_sprite_region(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    color: u32
)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position |
| w, h | `f32` | Destination size in pixels |
| src_x, src_y | `f32` | Source position in texture (pixels) |
| src_w, src_h | `f32` | Source size in texture (pixels) |
| color | `u32` | Tint color |

**Example:**
```rust
// Sprite sheet: 4x4 grid of 32x32 sprites
fn draw_frame(frame: u32) {
    let col = frame % 4;
    let row = frame / 4;
    draw_sprite_region(
        100.0, 100.0, 64.0, 64.0,           // Destination (scaled 2x)
        (col * 32) as f32, (row * 32) as f32, 32.0, 32.0, // Source
        0xFFFFFFFF
    );
}
```

---

### draw_sprite_ex

Draws a sprite with rotation and custom origin.

**Signature:**
```rust
fn draw_sprite_ex(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    origin_x: f32, origin_y: f32,
    angle_deg: f32,
    color: u32
)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position |
| w, h | `f32` | Destination size |
| src_x, src_y, src_w, src_h | `f32` | Source region |
| origin_x, origin_y | `f32` | Rotation origin (0-1 normalized) |
| angle_deg | `f32` | Rotation angle in degrees |
| color | `u32` | Tint color |

**Example:**
```rust
fn render() {
    // Rotating sprite around center
    draw_sprite_ex(
        200.0, 200.0, 64.0, 64.0,    // Position and size
        0.0, 0.0, 32.0, 32.0,        // Full texture
        0.5, 0.5,                     // Center origin
        elapsed_time() * 90.0,        // Rotation (90 deg/sec)
        0xFFFFFFFF
    );

    // Rotating around bottom-center (like a pendulum)
    draw_sprite_ex(
        300.0, 200.0, 64.0, 64.0,
        0.0, 0.0, 32.0, 32.0,
        0.5, 1.0,                     // Bottom-center origin
        (elapsed_time() * 2.0).sin() * 30.0,
        0xFFFFFFFF
    );
}
```

---

## Rectangles

### draw_rect

Draws a solid color rectangle.

**Signature:**
```rust
fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position (top-left) |
| w, h | `f32` | Size in pixels |
| color | `u32` | Fill color as `0xRRGGBBAA` |

**Example:**
```rust
fn render() {
    // Health bar background
    draw_rect(10.0, 10.0, 100.0, 20.0, 0x333333FF);

    // Health bar fill
    let health_width = (health / max_health) * 96.0;
    draw_rect(12.0, 12.0, health_width, 16.0, 0x00FF00FF);

    // Semi-transparent overlay
    draw_rect(0.0, 0.0, 960.0, 540.0, 0x00000080);
}
```

---

## Text

### draw_text

Draws text using the bound font.

**Signature:**
```rust
fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32)
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| ptr | `*const u8` | Pointer to UTF-8 string |
| len | `u32` | String length in bytes |
| x, y | `f32` | Screen position |
| size | `f32` | Font size in pixels |
| color | `u32` | Text color |

**Example:**
```rust
fn render() {
    let text = b"SCORE: 12345";
    draw_text(text.as_ptr(), text.len() as u32, 10.0, 10.0, 16.0, 0xFFFFFFFF);

    let title = b"GAME OVER";
    draw_text(title.as_ptr(), title.len() as u32, 400.0, 270.0, 48.0, 0xFF0000FF);
}
```

---

## Custom Fonts

### load_font

Loads a fixed-width bitmap font.

**Signature:**
```rust
fn load_font(
    texture: u32,
    char_width: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| texture | `u32` | Font texture atlas handle |
| char_width | `u32` | Width of each character in pixels |
| char_height | `u32` | Height of each character in pixels |
| first_codepoint | `u32` | First character code (usually 32 for space) |
| char_count | `u32` | Number of characters in atlas |

**Returns:** Font handle

**Constraints:** Init-only.

**Example:**
```rust
fn init() {
    unsafe {
        FONT_TEXTURE = load_texture(128, 64, FONT_PIXELS.as_ptr());
        // 8x8 font starting at space (32), 96 characters
        MY_FONT = load_font(FONT_TEXTURE, 8, 8, 32, 96);
    }
}
```

---

### load_font_ex

Loads a variable-width bitmap font.

**Signature:**
```rust
fn load_font_ex(
    texture: u32,
    widths_ptr: *const u8,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) -> u32
```

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| texture | `u32` | Font texture atlas handle |
| widths_ptr | `*const u8` | Pointer to array of character widths |
| char_height | `u32` | Height of each character |
| first_codepoint | `u32` | First character code |
| char_count | `u32` | Number of characters |

**Returns:** Font handle

**Constraints:** Init-only.

**Example:**
```rust
// Width table for characters ' ' through '~'
static CHAR_WIDTHS: [u8; 96] = [
    4, 2, 4, 6, 6, 6, 6, 2, 3, 3, 4, 6, 2, 4, 2, 4, // space to /
    6, 4, 6, 6, 6, 6, 6, 6, 6, 6, 2, 2, 4, 6, 4, 6, // 0 to ?
    // ... etc
];

fn init() {
    unsafe {
        PROP_FONT = load_font_ex(FONT_TEX, CHAR_WIDTHS.as_ptr(), 12, 32, 96);
    }
}
```

---

### font_bind

Binds a font for subsequent `draw_text()` calls.

**Signature:**
```rust
fn font_bind(font_handle: u32)
```

**Example:**
```rust
fn render() {
    // Use custom font
    font_bind(MY_FONT);
    draw_text(b"Custom Text".as_ptr(), 11, 10.0, 10.0, 16.0, 0xFFFFFFFF);

    // Switch to different font
    font_bind(TITLE_FONT);
    draw_text(b"Title".as_ptr(), 5, 100.0, 50.0, 32.0, 0xFFD700FF);
}
```

---

## Complete Example

```rust
static mut UI_FONT: u32 = 0;
static mut ICON_SHEET: u32 = 0;

fn init() {
    unsafe {
        UI_FONT = rom_font(b"ui_font".as_ptr(), 7);
        ICON_SHEET = rom_texture(b"icons".as_ptr(), 5);
    }
}

fn render() {
    unsafe {
        // Disable depth for 2D overlay
        depth_test(0);
        blend_mode(1);

        // Background panel
        draw_rect(5.0, 5.0, 200.0, 80.0, 0x00000099);

        // Health bar
        draw_rect(10.0, 10.0, 102.0, 12.0, 0x333333FF);
        draw_rect(11.0, 11.0, health as f32, 10.0, 0x00FF00FF);

        // Health icon
        texture_bind(ICON_SHEET);
        draw_sprite_region(
            10.0, 25.0, 16.0, 16.0,   // Position
            0.0, 0.0, 16.0, 16.0,     // Heart icon
            0xFFFFFFFF
        );

        // Score text
        font_bind(UI_FONT);
        let score_text = b"SCORE: 12345";
        draw_text(score_text.as_ptr(), score_text.len() as u32,
                  30.0, 25.0, 12.0, 0xFFFFFFFF);

        // Animated coin icon
        let frame = ((elapsed_time() * 8.0) as u32) % 4;
        draw_sprite_region(
            10.0, 45.0, 16.0, 16.0,
            (frame * 16) as f32, 16.0, 16.0, 16.0,
            0xFFD700FF
        );

        // Re-enable depth for 3D
        depth_test(1);
        blend_mode(0);
    }
}
```

**See Also:** [rom_font](./rom-loading.md#rom_font), [Textures](./textures.md)
