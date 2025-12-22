# Graphics Configuration

Console configuration and render state functions.

## Configuration (Init-Only)

These functions **must be called in `init()`** and cannot be changed at runtime.

### set_tick_rate

Sets the game's tick rate (updates per second).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn set_tick_rate(fps: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void set_tick_rate(uint32_t fps);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn set_tick_rate(fps: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Value | Tick Rate |
|-------|-----------|
| 0 | 24 fps |
| 1 | 30 fps |
| 2 | 60 fps - **default** |
| 3 | 120 fps |

**Constraints:** Init-only. Affects GGRS synchronization.

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    set_tick_rate(2); // 60 fps
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    set_tick_rate(2); // 60 fps
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    set_tick_rate(2); // 60 fps
}
```
{{#endtab}}

{{#endtabs}}

---

### set_clear_color

Sets the background clear color.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn set_clear_color(color: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void set_clear_color(uint32_t color);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn set_clear_color(color: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| color | `u32` | RGBA color as `0xRRGGBBAA` |

**Constraints:** Init-only. Default is `0x000000FF` (black).

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    set_clear_color(0x1a1a2eFF); // Dark blue
    set_clear_color(0x87CEEBFF); // Sky blue
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    set_clear_color(0x1a1a2eFF); // Dark blue
    set_clear_color(0x87CEEBFF); // Sky blue
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    set_clear_color(0x1a1a2eFF); // Dark blue
    set_clear_color(0x87CEEBFF); // Sky blue
}
```
{{#endtab}}

{{#endtabs}}

---

### render_mode

Sets the rendering mode (shader pipeline).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render_mode(mode: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void render_mode(uint32_t mode);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn render_mode(mode: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | Unlit | Flat colors, no lighting |
| 1 | Matcap | Pre-baked lighting via matcap textures |
| 2 | Metallic-Roughness | PBR-style Blinn-Phong with MRE textures |
| 3 | Specular-Shininess | Traditional Blinn-Phong |

**Constraints:** Init-only. Default is mode 0 (Unlit).

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    render_mode(2); // PBR-style lighting
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    render_mode(2); // PBR-style lighting
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    render_mode(2); // PBR-style lighting
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [Render Modes Guide](../guides/render-modes.md)

---

## Render State

These functions can be called anytime during `render()` to change draw state.

### set_color

Sets the uniform tint color for subsequent draws.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn set_color(color: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void set_color(uint32_t color);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn set_color(color: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| color | `u32` | RGBA color as `0xRRGGBBAA` |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
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
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
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
{{#endtab}}

{{#endtabs}}

---

### depth_test

Enables or disables depth testing.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn depth_test(enabled: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void depth_test(uint32_t enabled);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn depth_test(enabled: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| enabled | `u32` | `1` to enable, `0` to disable |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // 3D scene with depth
    depth_test(1);
    draw_mesh(level);
    draw_mesh(player);

    // UI overlay without depth
    depth_test(0);
    draw_sprite(0.0f, 0.0f, 100.0f, 50.0f, 0xFFFFFFFF);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // 3D scene with depth
    depth_test(1);
    draw_mesh(level);
    draw_mesh(player);

    // UI overlay without depth
    depth_test(0);
    draw_sprite(0.0, 0.0, 100.0, 50.0, 0xFFFFFFFF);
}
```
{{#endtab}}

{{#endtabs}}

---

### cull_mode

Sets face culling mode.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn cull_mode(mode: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void cull_mode(uint32_t mode);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn cull_mode(mode: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | None | Draw both sides |
| 1 | Back | Cull back faces (default) |
| 2 | Front | Cull front faces |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
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
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
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
{{#endtab}}

{{#endtabs}}

---

### blend_mode

Sets the alpha blending mode.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn blend_mode(mode: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void blend_mode(uint32_t mode);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn blend_mode(mode: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | None | No blending (opaque) |
| 1 | Alpha | Standard transparency |
| 2 | Additive | Add colors (glow effects) |
| 3 | Multiply | Multiply colors (shadows) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
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
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
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
{{#endtab}}

{{#endtabs}}

---

### texture_filter

Sets texture filtering mode.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn texture_filter(filter: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void texture_filter(uint32_t filter);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn texture_filter(filter: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Value | Mode | Description |
|-------|------|-------------|
| 0 | Nearest | Pixelated (retro look) |
| 1 | Linear | Smooth (modern look) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Pixel art sprites
    texture_filter(0);
    draw_sprite(0.0f, 0.0f, 64.0f, 64.0f, 0xFFFFFFFF);

    // Photo textures
    texture_filter(1);
    draw_mesh(realistic_model);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Pixel art sprites
    texture_filter(0);
    draw_sprite(0.0, 0.0, 64.0, 64.0, 0xFFFFFFFF);

    // Photo textures
    texture_filter(1);
    draw_mesh(realistic_model);
}
```
{{#endtab}}

{{#endtabs}}

---

### uniform_alpha

Sets the dither alpha level for PS1-style transparency.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn uniform_alpha(level: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void uniform_alpha(uint32_t level);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn uniform_alpha(level: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| level | `u32` | Alpha level 0-15 (0 = invisible, 15 = opaque) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Fade in effect
    uint32_t alpha = (uint32_t)(fade_progress * 15.0f);
    uniform_alpha(alpha);
    draw_mesh(fading_object);

    // Reset to fully opaque
    uniform_alpha(15);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Fade in effect
    const alpha: u32 = @intFromFloat(fade_progress * 15.0);
    uniform_alpha(alpha);
    draw_mesh(fading_object);

    // Reset to fully opaque
    uniform_alpha(15);
}
```
{{#endtab}}

{{#endtabs}}

**See Also:** [dither_offset](#dither_offset)

---

### dither_offset

Sets the dither pattern offset for animated dithering.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn dither_offset(x: u32, y: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void dither_offset(uint32_t x, uint32_t y);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn dither_offset(x: u32, y: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| x | `u32` | X offset 0-3 |
| y | `u32` | Y offset 0-3 |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Animate dither pattern for shimmer effect
    let frame = tick_count() as u32;
    dither_offset(frame % 4, (frame / 4) % 4);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Animate dither pattern for shimmer effect
    uint32_t frame = (uint32_t)tick_count();
    dither_offset(frame % 4, (frame / 4) % 4);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Animate dither pattern for shimmer effect
    const frame: u32 = @intCast(tick_count());
    dither_offset(frame % 4, (frame / 4) % 4);
}
```
{{#endtab}}

{{#endtabs}}

---

## Complete Example

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn init() {
    // Configure console
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
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void init(void) {
    // Configure console
    set_tick_rate(2);         // 60 fps
    set_clear_color(0x1a1a2eFF);
    render_mode(2);           // PBR lighting
}

NCZX_EXPORT void render(void) {
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
    draw_sprite(10.0f, 10.0f, 200.0f, 50.0f, 0xFFFFFFFF);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn init() void {
    // Configure console
    set_tick_rate(2);         // 60 fps
    set_clear_color(0x1a1a2eFF);
    render_mode(2);           // PBR lighting
}

export fn render() void {
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
{{#endtab}}

{{#endtabs}}
