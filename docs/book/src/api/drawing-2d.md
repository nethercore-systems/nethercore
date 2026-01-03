# 2D Drawing Functions

Screen-space sprites, rectangles, and text rendering.

> **Color:** Use `set_color(0xRRGGBBAA)` before drawing to set the tint color. Default is white (`0xFFFFFFFF`).

## Screen Coordinate System

All 2D drawing uses screen-space coordinates with the following conventions:

| Property | Value |
|----------|-------|
| **Resolution** | 960×540 pixels (fixed) |
| **Origin** | Top-left corner (0, 0) |
| **X-axis** | Increases rightward (0 to 960) |
| **Y-axis** | Increases downward (0 to 540) |
| **Anchor point** | Top-left corner of sprite/rect |

```
(0,0) ───────────────────────────► X (960)
  │
  │   ┌────────┐
  │   │ sprite │  x,y is top-left corner
  │   │  w×h   │
  │   └────────┘
  ▼
  Y (540)
```

The screen is always rendered at 960×540 internally and scaled to fit the window. This ensures pixel-perfect positioning regardless of window size.

## Sprites

### draw_sprite

Draws a textured quad at screen coordinates.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_sprite(x: f32, y: f32, w: f32, h: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_sprite(float x, float y, float w, float h);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_sprite(x: f32, y: f32, w: f32, h: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position (top-left corner) |
| w, h | `f32` | Size in pixels |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Draw full texture
    texture_bind(player_sprite);
    draw_sprite(100.0, 100.0, 64.0, 64.0);

    // Tinted sprite
    set_color(0xFF8080FF);
    draw_sprite(200.0, 100.0, 64.0, 64.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Draw full texture
    texture_bind(player_sprite);
    draw_sprite(100.0f, 100.0f, 64.0f, 64.0f);

    // Tinted sprite
    set_color(0xFF8080FF);
    draw_sprite(200.0f, 100.0f, 64.0f, 64.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Draw full texture
    texture_bind(player_sprite);
    draw_sprite(100.0, 100.0, 64.0, 64.0);

    // Tinted sprite
    set_color(0xFF8080FF);
    draw_sprite(200.0, 100.0, 64.0, 64.0);
}
```
{{#endtab}}

{{#endtabs}}

---

### draw_sprite_region

Draws a region of a texture (sprite sheet).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_sprite_region(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_sprite_region(
    float x, float y, float w, float h,
    float src_x, float src_y, float src_w, float src_h
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_sprite_region(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position |
| w, h | `f32` | Destination size in pixels |
| src_x, src_y | `f32` | Source UV position (0.0-1.0) |
| src_w, src_h | `f32` | Source UV size (0.0-1.0) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Sprite sheet: 4x4 grid of 32x32 sprites (128x128 texture)
fn draw_frame(frame: u32) {
    let col = frame % 4;
    let row = frame / 4;
    // UV coordinates: frame size / texture size
    let uv_size = 32.0 / 128.0;  // = 0.25
    draw_sprite_region(
        100.0, 100.0, 64.0, 64.0,           // Destination (scaled 2x)
        col as f32 * uv_size, row as f32 * uv_size, uv_size, uv_size  // Source UV
    );
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Sprite sheet: 4x4 grid of 32x32 sprites (128x128 texture)
NCZX_EXPORT void draw_frame(uint32_t frame) {
    uint32_t col = frame % 4;
    uint32_t row = frame / 4;
    // UV coordinates: frame size / texture size
    float uv_size = 32.0f / 128.0f;  // = 0.25
    draw_sprite_region(
        100.0f, 100.0f, 64.0f, 64.0f,       // Destination (scaled 2x)
        (float)col * uv_size, (float)row * uv_size, uv_size, uv_size  // Source UV
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Sprite sheet: 4x4 grid of 32x32 sprites (128x128 texture)
export fn draw_frame(frame: u32) void {
    const col = frame % 4;
    const row = frame / 4;
    // UV coordinates: frame size / texture size
    const uv_size = 32.0 / 128.0;  // = 0.25
    draw_sprite_region(
        100.0, 100.0, 64.0, 64.0,           // Destination (scaled 2x)
        @as(f32, @floatFromInt(col)) * uv_size, @as(f32, @floatFromInt(row)) * uv_size, uv_size, uv_size  // Source UV
    );
}
```
{{#endtab}}

{{#endtabs}}

---

### draw_sprite_ex

Draws a sprite with rotation and custom origin.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_sprite_ex(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    origin_x: f32, origin_y: f32,
    angle_deg: f32
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_sprite_ex(
    float x, float y, float w, float h,
    float src_x, float src_y, float src_w, float src_h,
    float origin_x, float origin_y,
    float angle_deg
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_sprite_ex(
    x: f32, y: f32, w: f32, h: f32,
    src_x: f32, src_y: f32, src_w: f32, src_h: f32,
    origin_x: f32, origin_y: f32,
    angle_deg: f32
) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position |
| w, h | `f32` | Destination size |
| src_x, src_y, src_w, src_h | `f32` | Source UV region (0.0-1.0) |
| origin_x, origin_y | `f32` | Rotation origin (0-1 normalized) |
| angle_deg | `f32` | Rotation angle in degrees |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Rotating sprite around center
    draw_sprite_ex(
        200.0, 200.0, 64.0, 64.0,    // Position and size
        0.0, 0.0, 1.0, 1.0,          // Full texture UV
        0.5, 0.5,                     // Center origin
        elapsed_time() * 90.0         // Rotation (90 deg/sec)
    );

    // Rotating around bottom-center (like a pendulum)
    draw_sprite_ex(
        300.0, 200.0, 64.0, 64.0,
        0.0, 0.0, 1.0, 1.0,
        0.5, 1.0,                     // Bottom-center origin
        (elapsed_time() * 2.0).sin() * 30.0
    );
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Rotating sprite around center
    draw_sprite_ex(
        200.0f, 200.0f, 64.0f, 64.0f,    // Position and size
        0.0f, 0.0f, 1.0f, 1.0f,          // Full texture UV
        0.5f, 0.5f,                      // Center origin
        elapsed_time() * 90.0f           // Rotation (90 deg/sec)
    );

    // Rotating around bottom-center (like a pendulum)
    draw_sprite_ex(
        300.0f, 200.0f, 64.0f, 64.0f,
        0.0f, 0.0f, 1.0f, 1.0f,
        0.5f, 1.0f,                      // Bottom-center origin
        sinf(elapsed_time() * 2.0f) * 30.0f
    );
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Rotating sprite around center
    draw_sprite_ex(
        200.0, 200.0, 64.0, 64.0,    // Position and size
        0.0, 0.0, 1.0, 1.0,          // Full texture UV
        0.5, 0.5,                     // Center origin
        elapsed_time() * 90.0         // Rotation (90 deg/sec)
    );

    // Rotating around bottom-center (like a pendulum)
    draw_sprite_ex(
        300.0, 200.0, 64.0, 64.0,
        0.0, 0.0, 1.0, 1.0,
        0.5, 1.0,                     // Bottom-center origin
        @sin(elapsed_time() * 2.0) * 30.0
    );
}
```
{{#endtab}}

{{#endtabs}}

---

## Rectangles

### draw_rect

Draws a solid color rectangle.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_rect(x: f32, y: f32, w: f32, h: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_rect(float x, float y, float w, float h);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_rect(x: f32, y: f32, w: f32, h: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Screen position (top-left) |
| w, h | `f32` | Size in pixels |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Health bar background
    set_color(0x333333FF);
    draw_rect(10.0, 10.0, 100.0, 20.0);

    // Health bar fill
    set_color(0x00FF00FF);
    let health_width = (health / max_health) * 96.0;
    draw_rect(12.0, 12.0, health_width, 16.0);

    // Semi-transparent overlay
    set_color(0x00000080);
    draw_rect(0.0, 0.0, 960.0, 540.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Health bar background
    set_color(0x333333FF);
    draw_rect(10.0f, 10.0f, 100.0f, 20.0f);

    // Health bar fill
    set_color(0x00FF00FF);
    float health_width = (health / max_health) * 96.0f;
    draw_rect(12.0f, 12.0f, health_width, 16.0f);

    // Semi-transparent overlay
    set_color(0x00000080);
    draw_rect(0.0f, 0.0f, 960.0f, 540.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Health bar background
    set_color(0x333333FF);
    draw_rect(10.0, 10.0, 100.0, 20.0);

    // Health bar fill
    set_color(0x00FF00FF);
    const health_width = (health / max_health) * 96.0;
    draw_rect(12.0, 12.0, health_width, 16.0);

    // Semi-transparent overlay
    set_color(0x00000080);
    draw_rect(0.0, 0.0, 960.0, 540.0);
}
```
{{#endtab}}

{{#endtabs}}

---

## Lines & Circles

### draw_line

Draws a line between two points.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_line(float x1, float y1, float x2, float y2, float thickness);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x1, y1 | `f32` | Start point in screen pixels |
| x2, y2 | `f32` | End point in screen pixels |
| thickness | `f32` | Line thickness in pixels |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Diagonal line
    set_color(0xFFFFFFFF);
    draw_line(100.0, 100.0, 300.0, 200.0, 2.0);

    // Box outline
    set_color(0x00FF00FF);
    draw_line(50.0, 50.0, 150.0, 50.0, 1.0);
    draw_line(150.0, 50.0, 150.0, 150.0, 1.0);
    draw_line(150.0, 150.0, 50.0, 150.0, 1.0);
    draw_line(50.0, 150.0, 50.0, 50.0, 1.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Diagonal line
    set_color(0xFFFFFFFF);
    draw_line(100.0f, 100.0f, 300.0f, 200.0f, 2.0f);

    // Box outline
    set_color(0x00FF00FF);
    draw_line(50.0f, 50.0f, 150.0f, 50.0f, 1.0f);
    draw_line(150.0f, 50.0f, 150.0f, 150.0f, 1.0f);
    draw_line(150.0f, 150.0f, 50.0f, 150.0f, 1.0f);
    draw_line(50.0f, 150.0f, 50.0f, 50.0f, 1.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Diagonal line
    set_color(0xFFFFFFFF);
    draw_line(100.0, 100.0, 300.0, 200.0, 2.0);

    // Box outline
    set_color(0x00FF00FF);
    draw_line(50.0, 50.0, 150.0, 50.0, 1.0);
    draw_line(150.0, 50.0, 150.0, 150.0, 1.0);
    draw_line(150.0, 150.0, 50.0, 150.0, 1.0);
    draw_line(50.0, 150.0, 50.0, 50.0, 1.0);
}
```
{{#endtab}}

{{#endtabs}}

---

### draw_circle

Draws a filled circle.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_circle(x: f32, y: f32, radius: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_circle(float x, float y, float radius);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_circle(x: f32, y: f32, radius: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Center position in screen pixels |
| radius | `f32` | Circle radius in pixels |

**Notes:** Rendered as a 16-segment approximation.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Player indicator
    set_color(0x00FF00FF);
    draw_circle(player_x, player_y, 10.0);

    // Semi-transparent area effect
    set_color(0xFF000040);
    draw_circle(500.0, 300.0, 50.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Player indicator
    set_color(0x00FF00FF);
    draw_circle(player_x, player_y, 10.0f);

    // Semi-transparent area effect
    set_color(0xFF000040);
    draw_circle(500.0f, 300.0f, 50.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Player indicator
    set_color(0x00FF00FF);
    draw_circle(player_x, player_y, 10.0);

    // Semi-transparent area effect
    set_color(0xFF000040);
    draw_circle(500.0, 300.0, 50.0);
}
```
{{#endtab}}

{{#endtabs}}

---

### draw_circle_outline

Draws a circle outline.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_circle_outline(x: f32, y: f32, radius: f32, thickness: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_circle_outline(float x, float y, float radius, float thickness);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_circle_outline(x: f32, y: f32, radius: f32, thickness: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x, y | `f32` | Center position in screen pixels |
| radius | `f32` | Circle radius in pixels |
| thickness | `f32` | Line thickness in pixels |

**Notes:** Rendered as 16 line segments.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Selection ring
    set_color(0xFFFF00FF);
    draw_circle_outline(selected_x, selected_y, 20.0, 2.0);

    // Range indicator
    set_color(0x00FF0080);
    draw_circle_outline(tower_x, tower_y, attack_range, 1.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Selection ring
    set_color(0xFFFF00FF);
    draw_circle_outline(selected_x, selected_y, 20.0f, 2.0f);

    // Range indicator
    set_color(0x00FF0080);
    draw_circle_outline(tower_x, tower_y, attack_range, 1.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Selection ring
    set_color(0xFFFF00FF);
    draw_circle_outline(selected_x, selected_y, 20.0, 2.0);

    // Range indicator
    set_color(0x00FF0080);
    draw_circle_outline(tower_x, tower_y, attack_range, 1.0);
}
```
{{#endtab}}

{{#endtabs}}

---

## Text

### draw_text

Draws text using the bound font.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void draw_text(const uint8_t* ptr, uint32_t len, float x, float y, float size);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn draw_text(ptr: [*]const u8, len: u32, x: f32, y: f32, size: f32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| ptr | `*const u8` | Pointer to UTF-8 string |
| len | `u32` | String length in bytes |
| x, y | `f32` | Screen position |
| size | `f32` | Font size in pixels |

> **Note:** Use `set_color(0xRRGGBBAA)` before calling `draw_text()` to set the text color.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    let text = b"SCORE: 12345";
    set_color(0xFFFFFFFF);
    draw_text(text.as_ptr(), text.len() as u32, 10.0, 10.0, 16.0);

    let title = b"GAME OVER";
    set_color(0xFF0000FF);
    draw_text(title.as_ptr(), title.len() as u32, 400.0, 270.0, 48.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    const char* text = "SCORE: 12345";
    set_color(0xFFFFFFFF);
    draw_text((const uint8_t*)text, strlen(text), 10.0f, 10.0f, 16.0f);

    const char* title = "GAME OVER";
    set_color(0xFF0000FF);
    draw_text((const uint8_t*)title, strlen(title), 400.0f, 270.0f, 48.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    const text = "SCORE: 12345";
    set_color(0xFFFFFFFF);
    draw_text(text, text.len, 10.0, 10.0, 16.0);

    const title = "GAME OVER";
    set_color(0xFF0000FF);
    draw_text(title, title.len, 400.0, 270.0, 48.0);
}
```
{{#endtab}}

{{#endtabs}}

---

### text_width

Measures the width of text when rendered.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn text_width(ptr: *const u8, len: u32, size: f32) -> f32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT float text_width(const uint8_t* ptr, uint32_t len, float size);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn text_width(ptr: [*]const u8, len: u32, size: f32) f32;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| ptr | `*const u8` | Pointer to UTF-8 string |
| len | `u32` | String length in bytes |
| size | `f32` | Font size in pixels |

**Returns:** Width in pixels that the text would occupy when rendered.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    let text = b"CENTERED";
    let width = text_width(text.as_ptr(), text.len() as u32, 32.0);

    // Center text on screen
    let x = (screen::WIDTH as f32 - width) / 2.0;
    set_color(0xFFFFFFFF);
    draw_text(text.as_ptr(), text.len() as u32, x, 270.0, 32.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    const char* text = "CENTERED";
    float width = text_width((const uint8_t*)text, 8, 32.0f);

    /* Center text on screen */
    float x = (NCZX_SCREEN_WIDTH - width) / 2.0f;
    set_color(0xFFFFFFFF);
    draw_text((const uint8_t*)text, 8, x, 270.0f, 32.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    const text = "CENTERED";
    const width = text_width(text, text.len, 32.0);

    // Center text on screen
    const x = (@as(f32, @floatFromInt(Screen.width)) - width) / 2.0;
    set_color(0xFFFFFFFF);
    draw_text(text, text.len, x, 270.0, 32.0);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [draw_text](#draw_text)

---

## Custom Fonts

### load_font

Loads a fixed-width bitmap font.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_font(
    texture: u32,
    char_width: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t load_font(
    uint32_t texture,
    uint32_t char_width,
    uint32_t char_height,
    uint32_t first_codepoint,
    uint32_t char_count
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_font(
    texture: u32,
    char_width: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) u32;
```
{{#endtab}}

{{#endtabs}}

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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    unsafe {
        FONT_TEXTURE = load_texture(128, 64, FONT_PIXELS.as_ptr());
        // 8x8 font starting at space (32), 96 characters
        MY_FONT = load_font(FONT_TEXTURE, 8, 8, 32, 96);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init() {
    FONT_TEXTURE = load_texture(128, 64, FONT_PIXELS);
    // 8x8 font starting at space (32), 96 characters
    MY_FONT = load_font(FONT_TEXTURE, 8, 8, 32, 96);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    FONT_TEXTURE = load_texture(128, 64, FONT_PIXELS);
    // 8x8 font starting at space (32), 96 characters
    MY_FONT = load_font(FONT_TEXTURE, 8, 8, 32, 96);
}
```
{{#endtab}}

{{#endtabs}}

---

### load_font_ex

Loads a variable-width bitmap font.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn load_font_ex(
    texture: u32,
    widths_ptr: *const u8,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT uint32_t load_font_ex(
    uint32_t texture,
    const uint8_t* widths_ptr,
    uint32_t char_height,
    uint32_t first_codepoint,
    uint32_t char_count
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn load_font_ex(
    texture: u32,
    widths_ptr: [*]const u8,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32
) u32;
```
{{#endtab}}

{{#endtabs}}

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

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Width table for characters ' ' through '~'
static uint8_t CHAR_WIDTHS[96] = {
    4, 2, 4, 6, 6, 6, 6, 2, 3, 3, 4, 6, 2, 4, 2, 4, // space to /
    6, 4, 6, 6, 6, 6, 6, 6, 6, 6, 2, 2, 4, 6, 4, 6, // 0 to ?
    // ... etc
};

NCZX_EXPORT void init() {
    PROP_FONT = load_font_ex(FONT_TEX, CHAR_WIDTHS, 12, 32, 96);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Width table for characters ' ' through '~'
const CHAR_WIDTHS = [96]u8{
    4, 2, 4, 6, 6, 6, 6, 2, 3, 3, 4, 6, 2, 4, 2, 4, // space to /
    6, 4, 6, 6, 6, 6, 6, 6, 6, 6, 2, 2, 4, 6, 4, 6, // 0 to ?
    // ... etc
};

export fn init() void {
    PROP_FONT = load_font_ex(FONT_TEX, &CHAR_WIDTHS, 12, 32, 96);
}
```
{{#endtab}}

{{#endtabs}}

---

### font_bind

Binds a font for subsequent `draw_text()` calls.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn font_bind(font_handle: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void font_bind(uint32_t font_handle);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn font_bind(font_handle: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Use custom font
    font_bind(MY_FONT);
    set_color(0xFFFFFFFF);
    draw_text(b"Custom Text".as_ptr(), 11, 10.0, 10.0, 16.0);

    // Switch to different font
    font_bind(TITLE_FONT);
    set_color(0xFFD700FF);
    draw_text(b"Title".as_ptr(), 5, 100.0, 50.0, 32.0);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render() {
    // Use custom font
    font_bind(MY_FONT);
    set_color(0xFFFFFFFF);
    draw_text((const uint8_t*)"Custom Text", strlen("Custom Text"), 10.0f, 10.0f, 16.0f);

    // Switch to different font
    font_bind(TITLE_FONT);
    set_color(0xFFD700FF);
    draw_text((const uint8_t*)"Title", strlen("Title"), 100.0f, 50.0f, 32.0f);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Use custom font
    font_bind(MY_FONT);
    set_color(0xFFFFFFFF);
    draw_text("Custom Text", "Custom Text".len, 10.0, 10.0, 16.0);

    // Switch to different font
    font_bind(TITLE_FONT);
    set_color(0xFFD700FF);
    draw_text("Title", "Title".len, 100.0, 50.0, 32.0);
}
```
{{#endtab}}

{{#endtabs}}

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
static mut UI_FONT: u32 = 0;
static mut ICON_SHEET: u32 = 0;

// Icon sheet is 64x32 pixels with 16x16 icons
const ICON_SIZE: f32 = 16.0;
const SHEET_W: f32 = 64.0;
const SHEET_H: f32 = 32.0;
const ICON_UV_W: f32 = ICON_SIZE / SHEET_W;  // 0.25
const ICON_UV_H: f32 = ICON_SIZE / SHEET_H;  // 0.5

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
        set_color(0x00000099);
        draw_rect(5.0, 5.0, 200.0, 80.0);

        // Health bar background
        set_color(0x333333FF);
        draw_rect(10.0, 10.0, 102.0, 12.0);

        // Health bar fill
        set_color(0x00FF00FF);
        draw_rect(11.0, 11.0, health as f32, 10.0);

        // Health icon (first icon in sheet)
        texture_bind(ICON_SHEET);
        set_color(0xFFFFFFFF);
        draw_sprite_region(
            10.0, 25.0, 16.0, 16.0,      // Position and size
            0.0, 0.0, ICON_UV_W, ICON_UV_H  // UV region (0.0-1.0)
        );

        // Score text
        font_bind(UI_FONT);
        set_color(0xFFFFFFFF);
        let score_text = b"SCORE: 12345";
        draw_text(score_text.as_ptr(), score_text.len() as u32,
                  30.0, 25.0, 12.0);

        // Animated coin icon (4 frames in bottom row)
        let frame = ((elapsed_time() * 8.0) as u32) % 4;
        set_color(0xFFD700FF);
        draw_sprite_region(
            10.0, 45.0, 16.0, 16.0,
            frame as f32 * ICON_UV_W, 0.5, ICON_UV_W, ICON_UV_H  // UV coords
        );

        // Re-enable depth for 3D
        depth_test(1);
        blend_mode(0);
    }
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
static uint32_t UI_FONT = 0;
static uint32_t ICON_SHEET = 0;

/* Icon sheet is 64x32 pixels with 16x16 icons */
#define ICON_SIZE 16.0f
#define SHEET_W 64.0f
#define SHEET_H 32.0f
#define ICON_UV_W (ICON_SIZE / SHEET_W)  /* 0.25 */
#define ICON_UV_H (ICON_SIZE / SHEET_H)  /* 0.5 */

NCZX_EXPORT void init() {
    UI_FONT = rom_font("ui_font", strlen("ui_font"));
    ICON_SHEET = rom_texture("icons", strlen("icons"));
}

NCZX_EXPORT void render() {
    /* Disable depth for 2D overlay */
    depth_test(0);
    blend_mode(1);

    /* Background panel */
    set_color(0x00000099);
    draw_rect(5.0f, 5.0f, 200.0f, 80.0f);

    /* Health bar background */
    set_color(0x333333FF);
    draw_rect(10.0f, 10.0f, 102.0f, 12.0f);

    /* Health bar fill */
    set_color(0x00FF00FF);
    draw_rect(11.0f, 11.0f, (float)health, 10.0f);

    /* Health icon (first icon in sheet) */
    texture_bind(ICON_SHEET);
    set_color(0xFFFFFFFF);
    draw_sprite_region(
        10.0f, 25.0f, 16.0f, 16.0f,      /* Position and size */
        0.0f, 0.0f, ICON_UV_W, ICON_UV_H  /* UV region (0.0-1.0) */
    );

    /* Score text */
    font_bind(UI_FONT);
    set_color(0xFFFFFFFF);
    const char* score_text = "SCORE: 12345";
    draw_text((const uint8_t*)score_text, strlen(score_text),
              30.0f, 25.0f, 12.0f);

    /* Animated coin icon (4 frames in bottom row) */
    uint32_t frame = ((uint32_t)(elapsed_time() * 8.0f)) % 4;
    set_color(0xFFD700FF);
    draw_sprite_region(
        10.0f, 45.0f, 16.0f, 16.0f,
        (float)frame * ICON_UV_W, 0.5f, ICON_UV_W, ICON_UV_H  /* UV coords */
    );

    /* Re-enable depth for 3D */
    depth_test(1);
    blend_mode(0);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
var UI_FONT: u32 = 0;
var ICON_SHEET: u32 = 0;

// Icon sheet is 64x32 pixels with 16x16 icons
const ICON_SIZE: f32 = 16.0;
const SHEET_W: f32 = 64.0;
const SHEET_H: f32 = 32.0;
const ICON_UV_W: f32 = ICON_SIZE / SHEET_W;  // 0.25
const ICON_UV_H: f32 = ICON_SIZE / SHEET_H;  // 0.5

export fn init() void {
    UI_FONT = rom_font("ui_font", "ui_font".len);
    ICON_SHEET = rom_texture("icons", "icons".len);
}

export fn render() void {
    // Disable depth for 2D overlay
    depth_test(0);
    blend_mode(1);

    // Background panel
    set_color(0x00000099);
    draw_rect(5.0, 5.0, 200.0, 80.0);

    // Health bar background
    set_color(0x333333FF);
    draw_rect(10.0, 10.0, 102.0, 12.0);

    // Health bar fill
    set_color(0x00FF00FF);
    draw_rect(11.0, 11.0, @floatFromInt(health), 10.0);

    // Health icon (first icon in sheet)
    texture_bind(ICON_SHEET);
    set_color(0xFFFFFFFF);
    draw_sprite_region(
        10.0, 25.0, 16.0, 16.0,      // Position and size
        0.0, 0.0, ICON_UV_W, ICON_UV_H  // UV region (0.0-1.0)
    );

    // Score text
    font_bind(UI_FONT);
    set_color(0xFFFFFFFF);
    const score_text = "SCORE: 12345";
    draw_text(score_text, score_text.len,
              30.0, 25.0, 12.0);

    // Animated coin icon (4 frames in bottom row)
    const frame = @as(u32, @intFromFloat(elapsed_time() * 8.0)) % 4;
    set_color(0xFFD700FF);
    draw_sprite_region(
        10.0, 45.0, 16.0, 16.0,
        @as(f32, @floatFromInt(frame)) * ICON_UV_W, 0.5, ICON_UV_W, ICON_UV_H  // UV coords
    );

    // Re-enable depth for 3D
    depth_test(1);
    blend_mode(0);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [rom_font](./rom-loading.md#rom_font), [Textures](./textures.md)
