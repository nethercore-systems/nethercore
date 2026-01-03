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
| 0 | Lambert | Simple diffuse shading |
| 1 | Matcap | Pre-baked lighting via matcap textures |
| 2 | Metallic-Roughness | PBR-style Blinn-Phong with MRE textures |
| 3 | Specular-Shininess | Traditional Blinn-Phong |

**Constraints:** Init-only. Default is mode 0 (Lambert).

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
| 0 | None | Draw both sides (default) |
| 1 | Back | Cull back faces |
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

## Render Passes (Execution Barriers)

Render passes provide execution barriers with configurable depth and stencil state. Commands in pass N are guaranteed to complete before commands in pass N+1 begin.

### begin_pass

Starts a new render pass with standard depth testing (depth enabled, compare LESS, write ON).

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn begin_pass(clear_depth: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void begin_pass(uint32_t clear_depth);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn begin_pass(clear_depth: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| clear_depth | `u32` | `1` to clear depth buffer, `0` to preserve |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Draw world normally
    draw_mesh(world);

    // Start new pass, clear depth to draw gun on top
    begin_pass(1);
    draw_mesh(fps_gun);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Draw world normally
    draw_mesh(world);

    // Start new pass, clear depth to draw gun on top
    begin_pass(1);
    draw_mesh(fps_gun);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Draw world normally
    draw_mesh(world);

    // Start new pass, clear depth to draw gun on top
    begin_pass(1);
    draw_mesh(fps_gun);
}
```
{{#endtab}}

{{#endtabs}}

---

### begin_pass_stencil_write

Starts a stencil write pass for mask creation. Depth is disabled, stencil writes reference value on pass.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn begin_pass_stencil_write(ref_value: u32, clear_depth: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void begin_pass_stencil_write(uint32_t ref_value, uint32_t clear_depth);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn begin_pass_stencil_write(ref_value: u32, clear_depth: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| ref_value | `u32` | Stencil reference value to write (0-255) |
| clear_depth | `u32` | `1` to clear depth buffer, `0` to preserve |

**Example:** See [Portal Effect](#stencil-portal-example) below.

---

### begin_pass_stencil_test

Starts a stencil test pass to render only where stencil equals reference. Depth testing is enabled.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn begin_pass_stencil_test(ref_value: u32, clear_depth: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void begin_pass_stencil_test(uint32_t ref_value, uint32_t clear_depth);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn begin_pass_stencil_test(ref_value: u32, clear_depth: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| ref_value | `u32` | Stencil reference value to test against (0-255) |
| clear_depth | `u32` | `1` to clear depth buffer (for portal interiors), `0` to preserve |

**Example:** See [Portal Effect](#stencil-portal-example) below.

---

### begin_pass_full

Starts a pass with full control over depth and stencil state.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn begin_pass_full(
    depth_compare: u32,
    depth_write: u32,
    clear_depth: u32,
    stencil_compare: u32,
    stencil_ref: u32,
    stencil_pass_op: u32,
    stencil_fail_op: u32,
    stencil_depth_fail_op: u32,
)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void begin_pass_full(
    uint32_t depth_compare,
    uint32_t depth_write,
    uint32_t clear_depth,
    uint32_t stencil_compare,
    uint32_t stencil_ref,
    uint32_t stencil_pass_op,
    uint32_t stencil_fail_op,
    uint32_t stencil_depth_fail_op
);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn begin_pass_full(
    depth_compare: u32,
    depth_write: u32,
    clear_depth: u32,
    stencil_compare: u32,
    stencil_ref: u32,
    stencil_pass_op: u32,
    stencil_fail_op: u32,
    stencil_depth_fail_op: u32,
) void;
```
{{#endtab}}

{{#endtabs}}

**Compare Function Constants:**

| Constant | Value | Description |
|----------|-------|-------------|
| `compare::NEVER` | 1 | Never pass |
| `compare::LESS` | 2 | Pass if src < dst |
| `compare::EQUAL` | 3 | Pass if src == dst |
| `compare::LESS_EQUAL` | 4 | Pass if src <= dst |
| `compare::GREATER` | 5 | Pass if src > dst |
| `compare::NOT_EQUAL` | 6 | Pass if src != dst |
| `compare::GREATER_EQUAL` | 7 | Pass if src >= dst |
| `compare::ALWAYS` | 8 | Always pass |

**Stencil Operation Constants:**

| Constant | Value | Description |
|----------|-------|-------------|
| `stencil_op::KEEP` | 0 | Keep current value |
| `stencil_op::ZERO` | 1 | Set to zero |
| `stencil_op::REPLACE` | 2 | Replace with ref value |
| `stencil_op::INCREMENT_CLAMP` | 3 | Increment, clamp to max |
| `stencil_op::DECREMENT_CLAMP` | 4 | Decrement, clamp to 0 |
| `stencil_op::INVERT` | 5 | Bitwise invert |
| `stencil_op::INCREMENT_WRAP` | 6 | Increment, wrap to 0 |
| `stencil_op::DECREMENT_WRAP` | 7 | Decrement, wrap to max |

---

### z_index

Sets the Z-order index for 2D draw ordering within a pass. Higher values draw on top.

**Signature:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn z_index(n: u32)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_IMPORT void z_index(uint32_t n);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
pub extern fn z_index(n: u32) void;
```
{{#endtab}}

{{#endtabs}}

**Parameters:**

| Name | Type | Description |
|------|------|-------------|
| n | `u32` | Z-order index (0-255) |

**Example:**

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // Background (lowest)
    z_index(0);
    draw_sprite(bg_x, bg_y, bg_w, bg_h, bg_color);

    // Game objects
    z_index(1);
    draw_sprite(obj_x, obj_y, obj_w, obj_h, obj_color);

    // UI on top
    z_index(2);
    draw_sprite(ui_x, ui_y, ui_w, ui_h, ui_color);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // Background (lowest)
    z_index(0);
    draw_sprite(bg_x, bg_y, bg_w, bg_h, bg_color);

    // Game objects
    z_index(1);
    draw_sprite(obj_x, obj_y, obj_w, obj_h, obj_color);

    // UI on top
    z_index(2);
    draw_sprite(ui_x, ui_y, ui_w, ui_h, ui_color);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // Background (lowest)
    z_index(0);
    draw_sprite(bg_x, bg_y, bg_w, bg_h, bg_color);

    // Game objects
    z_index(1);
    draw_sprite(obj_x, obj_y, obj_w, obj_h, obj_color);

    // UI on top
    z_index(2);
    draw_sprite(ui_x, ui_y, ui_w, ui_h, ui_color);
}
```
{{#endtab}}

{{#endtabs}}

---

### Stencil Portal Example {#stencil-portal-example}

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
fn render() {
    // 1. Draw main world
    draw_mesh(main_world);

    // 2. Write portal shape to stencil buffer (invisible)
    begin_pass_stencil_write(1, 0);
    draw_mesh(portal_quad);

    // 3. Draw portal interior (only where stencil == 1, clear depth)
    begin_pass_stencil_test(1, 1);
    draw_mesh(other_world);

    // 4. Return to normal rendering
    begin_pass(0);
    draw_mesh(portal_frame);
}
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
NCZX_EXPORT void render(void) {
    // 1. Draw main world
    draw_mesh(main_world);

    // 2. Write portal shape to stencil buffer (invisible)
    begin_pass_stencil_write(1, 0);
    draw_mesh(portal_quad);

    // 3. Draw portal interior (only where stencil == 1, clear depth)
    begin_pass_stencil_test(1, 1);
    draw_mesh(other_world);

    // 4. Return to normal rendering
    begin_pass(0);
    draw_mesh(portal_frame);
}
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
export fn render() void {
    // 1. Draw main world
    draw_mesh(main_world);

    // 2. Write portal shape to stencil buffer (invisible)
    begin_pass_stencil_write(1, 0);
    draw_mesh(portal_quad);

    // 3. Draw portal interior (only where stencil == 1, clear depth)
    begin_pass_stencil_test(1, 1);
    draw_mesh(other_world);

    // 4. Return to normal rendering
    begin_pass(0);
    draw_mesh(portal_frame);
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
    // Draw 3D scene (depth testing is enabled by default)
    cull_mode(1);  // Enable back-face culling for performance
    texture_filter(1);

    set_color(0xFFFFFFFF);
    draw_mesh(level);
    draw_mesh(player);

    // Draw semi-transparent water using dithering
    uniform_alpha(8);  // 50% alpha via ordered dithering
    set_color(0x4080FFFF);
    draw_mesh(water);
    uniform_alpha(15);  // Reset to fully opaque

    // Draw UI (2D draws are always on top via z_index)
    texture_filter(0);
    z_index(1);
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
    // Draw 3D scene (depth testing is enabled by default)
    cull_mode(1);  // Enable back-face culling for performance
    texture_filter(1);

    set_color(0xFFFFFFFF);
    draw_mesh(level);
    draw_mesh(player);

    // Draw semi-transparent water using dithering
    uniform_alpha(8);  // 50% alpha via ordered dithering
    set_color(0x4080FFFF);
    draw_mesh(water);
    uniform_alpha(15);  // Reset to fully opaque

    // Draw UI (2D draws are always on top via z_index)
    texture_filter(0);
    z_index(1);
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
    // Draw 3D scene (depth testing is enabled by default)
    cull_mode(1);  // Enable back-face culling for performance
    texture_filter(1);

    set_color(0xFFFFFFFF);
    draw_mesh(level);
    draw_mesh(player);

    // Draw semi-transparent water using dithering
    uniform_alpha(8);  // 50% alpha via ordered dithering
    set_color(0x4080FFFF);
    draw_mesh(water);
    uniform_alpha(15);  // Reset to fully opaque

    // Draw UI (2D draws are always on top via z_index)
    texture_filter(0);
    z_index(1);
    draw_sprite(10.0, 10.0, 200.0, 50.0, 0xFFFFFFFF);
}
```
{{#endtab}}

{{#endtabs}}
