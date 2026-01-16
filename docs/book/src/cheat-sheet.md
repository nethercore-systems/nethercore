# Cheat Sheet

All Nethercore ZX FFI functions on one page.

---

## System

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
delta_time() -> f32                    // Seconds since last tick
elapsed_time() -> f32                  // Total seconds since start
tick_count() -> u64                    // Current tick number
log(ptr, len)                          // Log message to console
quit()                                 // Exit to library
random() -> u32                        // Deterministic random u32
random_range(min, max) -> i32          // Random i32 in [min, max)
random_f32() -> f32                    // Random f32 in [0.0, 1.0)
random_f32_range(min, max) -> f32      // Random f32 in [min, max)
player_count() -> u32                  // Number of players (1-4)
local_player_mask() -> u32             // Bitmask of local players
```

**Screen Constants:** `screen::WIDTH`=960, `screen::HEIGHT`=540
{{#endtab}}

{{#tab name="C/C++"}}
```c
float delta_time(void);                // Seconds since last tick
float elapsed_time(void);              // Total seconds since start
uint64_t tick_count(void);             // Current tick number
void log_msg(ptr, len);                // Log message to console
void quit(void);                       // Exit to library
uint32_t random(void);                 // Deterministic random u32
int32_t random_range(int32_t min, int32_t max);    // Random i32 in [min, max)
float random_f32(void);                // Random f32 in [0.0, 1.0)
float random_f32_range(float min, float max);      // Random f32 in [min, max)
uint32_t player_count(void);           // Number of players (1-4)
uint32_t local_player_mask(void);      // Bitmask of local players
```

**Screen Constants:** `NCZX_SCREEN_WIDTH`=960, `NCZX_SCREEN_HEIGHT`=540
{{#endtab}}

{{#tab name="Zig"}}
```zig
delta_time() f32                       // Seconds since last tick
elapsed_time() f32                     // Total seconds since start
tick_count() u64                       // Current tick number
log_msg(ptr, len) void                 // Log message to console
quit() void                            // Exit to library
random() u32                           // Deterministic random u32
random_range(min: i32, max: i32) i32   // Random i32 in [min, max)
random_f32() f32                       // Random f32 in [0.0, 1.0)
random_f32_range(min: f32, max: f32) f32  // Random f32 in [min, max)
player_count() u32                     // Number of players (1-4)
local_player_mask() u32                // Bitmask of local players
```

**Screen Constants:** `Screen.width`=960, `Screen.height`=540
{{#endtab}}

{{#endtabs}}

---

## Configuration (Init-Only)

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
set_tick_rate(fps)                     // 0=24, 1=30, 2=60, 3=120
set_clear_color(0xRRGGBBAA)            // Background color
// render_mode set via nether.toml     // 0=Lambert, 1=Matcap, 2=MR, 3=SS
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void set_tick_rate(uint32_t fps);      // NCZX_TICK_RATE_24/30/60/120
void set_clear_color(uint32_t color);  // Background color
// render_mode set via nether.toml     // NCZX_RENDER_LAMBERT/MATCAP/PBR/HYBRID
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
set_tick_rate(fps: u32) void           // 0=24, 1=30, 2=60, 3=120
set_clear_color(color: u32) void       // Background color
// render_mode set via nether.toml     // 0=Lambert, 1=Matcap, 2=MR, 3=SS
```
{{#endtab}}

{{#endtabs}}

---

## Input

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Buttons (player: 0-3, button: 0-13)
button_held(player, button) -> u32     // 1 if held
button_pressed(player, button) -> u32  // 1 if just pressed
button_released(player, button) -> u32 // 1 if just released
buttons_held(player) -> u32            // Bitmask of held
buttons_pressed(player) -> u32         // Bitmask of pressed
buttons_released(player) -> u32        // Bitmask of released

// Sticks (-1.0 to 1.0)
left_stick_x(player) -> f32
left_stick_y(player) -> f32
right_stick_x(player) -> f32
right_stick_y(player) -> f32
left_stick(player, &mut x, &mut y)     // Both axes
right_stick(player, &mut x, &mut y)

// Triggers (0.0 to 1.0)
trigger_left(player) -> f32
trigger_right(player) -> f32
```

**Button Constants:** UP=0, DOWN=1, LEFT=2, RIGHT=3, A=4, B=5, X=6, Y=7, LB=8, RB=9, L3=10, R3=11, START=12, SELECT=13
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Buttons (player: 0-3, button: NCZX_BUTTON_*)
uint32_t button_held(player, button);     // 1 if held
uint32_t button_pressed(player, button);  // 1 if just pressed
uint32_t button_released(player, button); // 1 if just released
uint32_t buttons_held(player);            // Bitmask of held
uint32_t buttons_pressed(player);         // Bitmask of pressed
uint32_t buttons_released(player);        // Bitmask of released

// Sticks (-1.0 to 1.0)
float left_stick_x(player);
float left_stick_y(player);
float right_stick_x(player);
float right_stick_y(player);
void left_stick(player, float* x, float* y);   // Both axes
void right_stick(player, float* x, float* y);

// Triggers (0.0 to 1.0)
float trigger_left(player);
float trigger_right(player);
```

**Button Constants:** `NCZX_BUTTON_UP`=0, `NCZX_BUTTON_DOWN`=1, `NCZX_BUTTON_LEFT`=2, `NCZX_BUTTON_RIGHT`=3, `NCZX_BUTTON_A`=4, `NCZX_BUTTON_B`=5, `NCZX_BUTTON_X`=6, `NCZX_BUTTON_Y`=7, `NCZX_BUTTON_L1`=8, `NCZX_BUTTON_R1`=9, `NCZX_BUTTON_L3`=10, `NCZX_BUTTON_R3`=11, `NCZX_BUTTON_START`=12, `NCZX_BUTTON_SELECT`=13
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Buttons (player: 0-3, button: Button.*)
button_held(player: u32, button: u32) u32     // 1 if held
button_pressed(player: u32, button: u32) u32  // 1 if just pressed
button_released(player: u32, button: u32) u32 // 1 if just released
buttons_held(player: u32) u32                 // Bitmask of held
buttons_pressed(player: u32) u32              // Bitmask of pressed
buttons_released(player: u32) u32             // Bitmask of released

// Sticks (-1.0 to 1.0)
left_stick_x(player: u32) f32
left_stick_y(player: u32) f32
right_stick_x(player: u32) f32
right_stick_y(player: u32) f32
left_stick(player: u32, x: *f32, y: *f32) void  // Both axes
right_stick(player: u32, x: *f32, y: *f32) void

// Triggers (0.0 to 1.0)
trigger_left(player: u32) f32
trigger_right(player: u32) f32
```

**Button Constants:** `Button.up`=0, `Button.down`=1, `Button.left`=2, `Button.right`=3, `Button.a`=4, `Button.b`=5, `Button.x`=6, `Button.y`=7, `Button.l1`=8, `Button.r1`=9, `Button.l3`=10, `Button.r3`=11, `Button.start`=12, `Button.select`=13
{{#endtab}}

{{#endtabs}}

---

## Camera

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
camera_set(x, y, z, target_x, target_y, target_z)
camera_fov(degrees)                    // Default: 60
push_view_matrix(m0..m15)              // Custom 4x4 view matrix
push_projection_matrix(m0..m15)        // Custom 4x4 projection
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void camera_set(x, y, z, target_x, target_y, target_z);
void camera_fov(float degrees);        // Default: 60
void push_view_matrix(m0..m15);        // Custom 4x4 view matrix
void push_projection_matrix(m0..m15);  // Custom 4x4 projection
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32) void
camera_fov(degrees: f32) void          // Default: 60
// push_view_matrix and push_projection_matrix take 16 f32 parameters
```
{{#endtab}}

{{#endtabs}}

---

## Transforms

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
push_identity()                        // Reset to identity
transform_set(matrix_ptr)              // Set from 4x4 matrix
push_translate(x, y, z)
push_rotate_x(degrees)
push_rotate_y(degrees)
push_rotate_z(degrees)
push_rotate(degrees, axis_x, axis_y, axis_z)
push_scale(x, y, z)
push_scale_uniform(s)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void push_identity(void);              // Reset to identity
void transform_set(const float* matrix_ptr);  // Set from 4x4 matrix
void push_translate(float x, float y, float z);
void push_rotate_x(float degrees);
void push_rotate_y(float degrees);
void push_rotate_z(float degrees);
void push_rotate(float degrees, float axis_x, float axis_y, float axis_z);
void push_scale(float x, float y, float z);
void push_scale_uniform(float s);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
push_identity() void                   // Reset to identity
transform_set(matrix_ptr: [*]const f32) void  // Set from 4x4 matrix
push_translate(x: f32, y: f32, z: f32) void
push_rotate_x(degrees: f32) void
push_rotate_y(degrees: f32) void
push_rotate_z(degrees: f32) void
push_rotate(degrees: f32, axis_x: f32, axis_y: f32, axis_z: f32) void
push_scale(x: f32, y: f32, z: f32) void
push_scale_uniform(s: f32) void
```
{{#endtab}}

{{#endtabs}}

---

## Render State

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
set_color(0xRRGGBBAA)                  // Tint color
cull_mode(mode)                        // 0=none (default), 1=back, 2=front
texture_filter(filter)                 // 0=nearest, 1=linear
uniform_alpha(level)                   // 0-15 dither alpha
dither_offset(x, y)                    // 0-3 pattern offset
z_index(n)                             // 2D ordering within pass (0=back, higher=front)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void set_color(uint32_t color);        // Tint color
void cull_mode(uint32_t mode);         // NCZX_CULL_NONE (default)/BACK/FRONT
void texture_filter(uint32_t filter);  // 0=nearest, 1=linear
void uniform_alpha(uint32_t level);    // 0-15 dither alpha
void dither_offset(uint32_t x, uint32_t y);  // 0-3 pattern offset
void z_index(uint32_t n);              // 2D ordering within pass (0=back, higher=front)
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
set_color(color: u32) void             // Tint color
cull_mode(mode: u32) void              // CullMode.none (default)/back/front
texture_filter(filter: u32) void       // 0=nearest, 1=linear
uniform_alpha(level: u32) void         // 0-15 dither alpha
dither_offset(x: u32, y: u32) void     // 0-3 pattern offset
z_index(n: u32) void                   // 2D ordering within pass (0=back, higher=front)
```
{{#endtab}}

{{#endtabs}}

---

## Render Passes (Execution Barriers)

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
begin_pass(clear_depth)                // New pass with optional depth clear
begin_pass_stencil_write(ref_val, clear_depth)  // Create stencil mask
begin_pass_stencil_test(ref_val, clear_depth)   // Render inside mask
begin_pass_full(...)                   // Full control (8 params)
```

**Use Cases:**
- FPS viewmodels: `begin_pass(1)` clears depth, gun renders on top
- Portals: `begin_pass_stencil_write(1,0)` then `begin_pass_stencil_test(1,1)`
{{#endtab}}

{{#tab name="C/C++"}}
```c
void begin_pass(uint32_t clear_depth); // New pass with optional depth clear
void begin_pass_stencil_write(uint32_t ref_val, uint32_t clear_depth);
void begin_pass_stencil_test(uint32_t ref_val, uint32_t clear_depth);
void begin_pass_full(uint32_t depth_compare, uint32_t depth_write,
                     uint32_t clear_depth, uint32_t stencil_compare,
                     uint32_t stencil_ref, uint32_t stencil_pass_op,
                     uint32_t stencil_fail_op, uint32_t stencil_depth_fail_op);
```

**Constants:** `NCZX_COMPARE_*`, `NCZX_STENCIL_OP_*`
{{#endtab}}

{{#tab name="Zig"}}
```zig
begin_pass(clear_depth: u32) void
begin_pass_stencil_write(ref_val: u32, clear_depth: u32) void
begin_pass_stencil_test(ref_val: u32, clear_depth: u32) void
begin_pass_full(...) void              // Full control (8 params)
```

**Constants:** `compare.*`, `stencil_op.*`
{{#endtab}}

{{#endtabs}}

---

## Textures

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
load_texture(w, h, pixels_ptr) -> u32  // Init-only, returns handle
texture_bind(handle)                   // Bind to slot 0
texture_bind_slot(handle, slot)        // Bind to slot 0-3
matcap_blend_mode(slot, mode)          // 0=mul, 1=add, 2=hsv
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t load_texture(uint32_t w, uint32_t h, const uint8_t* pixels);  // Init-only
void texture_bind(uint32_t handle);    // Bind to slot 0
void texture_bind_slot(uint32_t handle, uint32_t slot);  // Bind to slot 0-3
void matcap_blend_mode(uint32_t slot, uint32_t mode);    // 0=mul, 1=add, 2=hsv
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
load_texture(w: u32, h: u32, pixels: [*]const u8) u32  // Init-only
texture_bind(handle: u32) void         // Bind to slot 0
texture_bind_slot(handle: u32, slot: u32) void  // Bind to slot 0-3
matcap_blend_mode(slot: u32, mode: u32) void    // 0=mul, 1=add, 2=hsv
```
{{#endtab}}

{{#endtabs}}

---

## Meshes

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Retained (init-only)
load_mesh(data_ptr, vertex_count, format) -> u32
load_mesh_indexed(data_ptr, vcount, idx_ptr, icount, fmt) -> u32
load_mesh_packed(data_ptr, vertex_count, format) -> u32
load_mesh_indexed_packed(data_ptr, vcount, idx_ptr, icount, fmt) -> u32
draw_mesh(handle)

// Immediate
draw_triangles(data_ptr, vertex_count, format)
draw_triangles_indexed(data_ptr, vcount, idx_ptr, icount, fmt)
```

**Vertex Formats:** POS=0, UV=1, COLOR=2, UV_COLOR=3, NORMAL=4, UV_NORMAL=5, COLOR_NORMAL=6, UV_COLOR_NORMAL=7, SKINNED=8, TANGENT=16 (combine with NORMAL)
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Retained (init-only)
uint32_t load_mesh(const float* data, uint32_t vcount, uint32_t fmt);
uint32_t load_mesh_indexed(const float* data, uint32_t vcount,
                           const uint16_t* idx, uint32_t icount, uint32_t fmt);
uint32_t load_mesh_packed(const uint8_t* data, uint32_t vcount, uint32_t fmt);
uint32_t load_mesh_indexed_packed(const uint8_t* data, uint32_t vcount,
                                  const uint16_t* idx, uint32_t icount, uint32_t fmt);
void draw_mesh(uint32_t handle);

// Immediate
void draw_triangles(const float* data, uint32_t vcount, uint32_t fmt);
void draw_triangles_indexed(const float* data, uint32_t vcount,
                            const uint16_t* idx, uint32_t icount, uint32_t fmt);
```

**Vertex Formats:** `NCZX_FORMAT_POS`=0, `NCZX_FORMAT_UV`=1, `NCZX_FORMAT_COLOR`=2, `NCZX_FORMAT_NORMAL`=4, `NCZX_FORMAT_SKINNED`=8, `NCZX_FORMAT_TANGENT`=16 (combinable, TANGENT requires NORMAL)
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Retained (init-only)
load_mesh(data: [*]const f32, vcount: u32, fmt: u32) u32
load_mesh_indexed(data: [*]const f32, vcount: u32, idx: [*]const u16, icount: u32, fmt: u32) u32
draw_mesh(handle: u32) void

// Immediate
draw_triangles(data: [*]const f32, vcount: u32, fmt: u32) void
draw_triangles_indexed(data: [*]const f32, vcount: u32, idx: [*]const u16, icount: u32, fmt: u32) void
```

**Vertex Formats:** `Format.pos`=0, `Format.uv`=1, `Format.color`=2, `Format.normal`=4, `Format.skinned`=8, `Format.tangent`=16 (combinable, tangent requires normal)
{{#endtab}}

{{#endtabs}}

---

## Procedural Meshes (Init-Only)

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
cube(sx, sy, sz) -> u32
sphere(radius, segments, rings) -> u32
cylinder(r_bot, r_top, height, segments) -> u32
plane(sx, sz, subdiv_x, subdiv_z) -> u32
torus(major_r, minor_r, major_seg, minor_seg) -> u32
capsule(radius, height, segments, rings) -> u32

// With explicit UV naming (same behavior)
cube_uv, sphere_uv, cylinder_uv, plane_uv, torus_uv, capsule_uv
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t cube(float sx, float sy, float sz);
uint32_t sphere(float radius, uint32_t segments, uint32_t rings);
uint32_t cylinder(float r_bot, float r_top, float height, uint32_t segments);
uint32_t plane(float sx, float sz, uint32_t subdiv_x, uint32_t subdiv_z);
uint32_t torus(float major_r, float minor_r, uint32_t major_seg, uint32_t minor_seg);
uint32_t capsule(float radius, float height, uint32_t segments, uint32_t rings);

// With explicit UV naming (same behavior)
cube_uv, sphere_uv, cylinder_uv, plane_uv, torus_uv, capsule_uv
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
cube(sx: f32, sy: f32, sz: f32) u32
sphere(radius: f32, segments: u32, rings: u32) u32
cylinder(r_bot: f32, r_top: f32, height: f32, segments: u32) u32
plane(sx: f32, sz: f32, subdiv_x: u32, subdiv_z: u32) u32
torus(major_r: f32, minor_r: f32, major_seg: u32, minor_seg: u32) u32
capsule(radius: f32, height: f32, segments: u32, rings: u32) u32

// With explicit UV naming (same behavior)
cube_uv, sphere_uv, cylinder_uv, plane_uv, torus_uv, capsule_uv
```
{{#endtab}}

{{#endtabs}}

---

## Materials

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Mode 2 (Metallic-Roughness)
material_metallic(value)               // 0.0-1.0
material_roughness(value)              // 0.0-1.0
material_emissive(value)               // Glow intensity
material_rim(intensity, power)         // Rim light
material_albedo(texture)               // Bind to slot 0
material_mre(texture)                  // Bind MRE to slot 1
material_normal(texture)               // Bind normal map to slot 3

// Mode 3 (Specular-Shininess)
material_shininess(value)              // 0.0-1.0 → 1-256
material_specular(0xRRGGBBAA)          // Specular color

// Override flags
use_uniform_color(enabled)
use_uniform_metallic(enabled)
use_uniform_roughness(enabled)
use_uniform_emissive(enabled)
skip_normal_map(skip)                  // 0=use normal map, 1=use vertex normal
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Mode 2 (Metallic-Roughness)
void material_metallic(float value);   // 0.0-1.0
void material_roughness(float value);  // 0.0-1.0
void material_emissive(float value);   // Glow intensity
void material_rim(float intensity, float power);  // Rim light
void material_albedo(uint32_t texture);    // Bind to slot 0
void material_mre(uint32_t texture);       // Bind MRE to slot 1
void material_normal(uint32_t texture);    // Bind normal map to slot 3

// Mode 3 (Specular-Shininess)
void material_shininess(float value);  // 0.0-1.0 → 1-256
void material_specular(uint32_t color);    // Specular color

// Override flags
void use_uniform_color(uint32_t enabled);
void use_uniform_metallic(uint32_t enabled);
void use_uniform_roughness(uint32_t enabled);
void use_uniform_emissive(uint32_t enabled);
void skip_normal_map(uint32_t skip);       // 0=use normal map, 1=use vertex normal
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Mode 2 (Metallic-Roughness)
material_metallic(value: f32) void     // 0.0-1.0
material_roughness(value: f32) void    // 0.0-1.0
material_emissive(value: f32) void     // Glow intensity
material_rim(intensity: f32, power: f32) void  // Rim light
material_albedo(texture: u32) void     // Bind to slot 0
material_mre(texture: u32) void        // Bind MRE to slot 1
material_normal(texture: u32) void     // Bind normal map to slot 3

// Mode 3 (Specular-Shininess)
material_shininess(value: f32) void    // 0.0-1.0 → 1-256
material_specular(color: u32) void     // Specular color

// Override flags
use_uniform_color(enabled: u32) void
use_uniform_metallic(enabled: u32) void
use_uniform_roughness(enabled: u32) void
use_uniform_emissive(enabled: u32) void
skip_normal_map(skip: u32) void        // 0=use normal map, 1=use vertex normal
```
{{#endtab}}

{{#endtabs}}

---

## Lighting

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Directional lights (index 0-3)
light_set(index, dir_x, dir_y, dir_z)
light_color(index, 0xRRGGBBAA)
light_intensity(index, intensity)      // 0.0-8.0
light_enable(index)
light_disable(index)

// Point lights
light_set_point(index, x, y, z)
light_range(index, range)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Directional lights (index 0-3)
void light_set(uint32_t index, float dir_x, float dir_y, float dir_z);
void light_color(uint32_t index, uint32_t color);
void light_intensity(uint32_t index, float intensity);  // 0.0-8.0
void light_enable(uint32_t index);
void light_disable(uint32_t index);

// Point lights
void light_set_point(uint32_t index, float x, float y, float z);
void light_range(uint32_t index, float range);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Directional lights (index 0-3)
light_set(index: u32, dir_x: f32, dir_y: f32, dir_z: f32) void
light_color(index: u32, color: u32) void
light_intensity(index: u32, intensity: f32) void  // 0.0-8.0
light_enable(index: u32) void
light_disable(index: u32) void

// Point lights
light_set_point(index: u32, x: f32, y: f32, z: f32) void
light_range(index: u32, range: f32) void
```
{{#endtab}}

{{#endtabs}}

---

## Environment (EPU)

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// 8 Environment Modes (layer: 0=base, 1=overlay)
env_gradient(layer, zenith, sky_h, ground_h, nadir, rotation, shift, sun_elev, disk, halo, intensity, haze, warmth, cloudiness, cloud_phase)  // Mode 0
env_cells(layer, family, variant, density, size_min, size_max, intensity, shape, motion, parallax, height_bias, clustering, color_a, color_b, phase, seed)  // Mode 1
env_lines(layer, variant, line_type, thickness, spacing, fade, parallax, color_primary, color_accent, accent_every, phase, profile, warp, wobble, glow, axis_x, axis_y, axis_z, seed)  // Mode 2
env_silhouette(layer, family, jaggedness, layer_count, color_near, color_far, sky_zenith, sky_horizon, parallax_rate, seed, phase, fog, wind)  // Mode 3
env_nebula(layer, family, coverage, softness, intensity, scale, detail, warp, flow, parallax, height_bias, contrast, color_a, color_b, axis_x, axis_y, axis_z, phase, seed)  // Mode 4
env_room(layer, color_ceiling, color_floor, color_walls, panel_size, panel_gap, light_x, light_y, light_z, light_intensity, light_tint, corner_darken, room_scale, viewer_x, viewer_y, viewer_z, accent, accent_mode, roughness, phase)  // Mode 5
env_veil(layer, family, density, width, taper, curvature, edge_soft, height_min, height_max, color_near, color_far, glow, parallax, axis_x, axis_y, axis_z, phase, seed)  // Mode 6
env_rings(layer, family, ring_count, thickness, color_a, color_b, center_color, center_falloff, spiral_twist, axis_x, axis_y, axis_z, phase, wobble, noise, dash, glow, seed)  // Mode 7

// Blending and rendering
env_blend(mode)               // 0=alpha, 1=add, 2=mul, 3=screen
draw_env()                    // Call first in render()
matcap_set(slot, texture)     // Slot 1-3 (Mode 1 only)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// 8 Environment Modes (layer: 0=base, 1=overlay)
void env_gradient(u32 layer, u32 zenith, u32 sky_h, u32 ground_h, u32 nadir, f32 rotation, f32 shift, f32 sun_elev, u32 disk, u32 halo, u32 intensity, u32 haze, u32 warmth, u32 cloudiness, u32 cloud_phase);  // Mode 0
void env_cells(u32 layer, u32 family, u32 variant, u32 density, u32 size_min, u32 size_max, u32 intensity, u32 shape, u32 motion, u32 parallax, u32 height_bias, u32 clustering, u32 color_a, u32 color_b, u32 phase, u32 seed);  // Mode 1
void env_lines(u32 layer, u32 variant, u32 line_type, u32 thickness, f32 spacing, f32 fade, u32 parallax, u32 color_primary, u32 color_accent, u32 accent_every, u32 phase, u32 profile, u32 warp, u32 wobble, u32 glow, f32 axis_x, f32 axis_y, f32 axis_z, u32 seed);  // Mode 2
void env_silhouette(u32 layer, u32 family, u32 jaggedness, u32 layer_count, u32 color_near, u32 color_far, u32 sky_zenith, u32 sky_horizon, u32 parallax_rate, u32 seed, u32 phase, u32 fog, u32 wind);  // Mode 3
void env_nebula(u32 layer, u32 family, u32 coverage, u32 softness, u32 intensity, u32 scale, u32 detail, u32 warp, u32 flow, u32 parallax, u32 height_bias, u32 contrast, u32 color_a, u32 color_b, f32 axis_x, f32 axis_y, f32 axis_z, u32 phase, u32 seed);  // Mode 4
void env_room(u32 layer, u32 color_ceiling, u32 color_floor, u32 color_walls, f32 panel_size, u32 panel_gap, f32 light_x, f32 light_y, f32 light_z, u32 light_intensity, u32 light_tint, u32 corner_darken, f32 room_scale, i32 viewer_x, i32 viewer_y, i32 viewer_z, u32 accent, u32 accent_mode, u32 roughness, u32 phase);  // Mode 5
void env_veil(u32 layer, u32 family, u32 density, u32 width, u32 taper, u32 curvature, u32 edge_soft, u32 height_min, u32 height_max, u32 color_near, u32 color_far, u32 glow, u32 parallax, f32 axis_x, f32 axis_y, f32 axis_z, u32 phase, u32 seed);  // Mode 6
void env_rings(u32 layer, u32 family, u32 ring_count, u32 thickness, u32 color_a, u32 color_b, u32 center_color, u32 center_falloff, f32 spiral_twist, f32 axis_x, f32 axis_y, f32 axis_z, u32 phase, u32 wobble, u32 noise, u32 dash, u32 glow, u32 seed);  // Mode 7

// Blending and rendering
void env_blend(uint32_t mode);                    // 0=alpha, 1=add, 2=mul, 3=screen
void draw_env(void);                              // Call first in render()
void matcap_set(uint32_t slot, uint32_t texture); // Slot 1-3 (Mode 1 only)
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// 8 Environment Modes (layer: 0=base, 1=overlay)
env_gradient(layer: u32, zenith: u32, sky_h: u32, ground_h: u32, nadir: u32, rotation: f32, shift: f32, sun_elev: f32, disk: u32, halo: u32, intensity: u32, haze: u32, warmth: u32, cloudiness: u32, cloud_phase: u32) void  // Mode 0
env_cells(layer: u32, family: u32, variant: u32, density: u32, size_min: u32, size_max: u32, intensity: u32, shape: u32, motion: u32, parallax: u32, height_bias: u32, clustering: u32, color_a: u32, color_b: u32, phase: u32, seed: u32) void  // Mode 1
env_lines(layer: u32, variant: u32, line_type: u32, thickness: u32, spacing: f32, fade: f32, parallax: u32, color_primary: u32, color_accent: u32, accent_every: u32, phase: u32, profile: u32, warp: u32, wobble: u32, glow: u32, axis_x: f32, axis_y: f32, axis_z: f32, seed: u32) void  // Mode 2
env_silhouette(layer: u32, family: u32, jaggedness: u32, layer_count: u32, color_near: u32, color_far: u32, sky_zenith: u32, sky_horizon: u32, parallax_rate: u32, seed: u32, phase: u32, fog: u32, wind: u32) void  // Mode 3
env_nebula(layer: u32, family: u32, coverage: u32, softness: u32, intensity: u32, scale: u32, detail: u32, warp: u32, flow: u32, parallax: u32, height_bias: u32, contrast: u32, color_a: u32, color_b: u32, axis_x: f32, axis_y: f32, axis_z: f32, phase: u32, seed: u32) void  // Mode 4
env_room(layer: u32, color_ceiling: u32, color_floor: u32, color_walls: u32, panel_size: f32, panel_gap: u32, light_x: f32, light_y: f32, light_z: f32, light_intensity: u32, light_tint: u32, corner_darken: u32, room_scale: f32, viewer_x: i32, viewer_y: i32, viewer_z: i32, accent: u32, accent_mode: u32, roughness: u32, phase: u32) void  // Mode 5
env_veil(layer: u32, family: u32, density: u32, width: u32, taper: u32, curvature: u32, edge_soft: u32, height_min: u32, height_max: u32, color_near: u32, color_far: u32, glow: u32, parallax: u32, axis_x: f32, axis_y: f32, axis_z: f32, phase: u32, seed: u32) void  // Mode 6
env_rings(layer: u32, family: u32, ring_count: u32, thickness: u32, color_a: u32, color_b: u32, center_color: u32, center_falloff: u32, spiral_twist: f32, axis_x: f32, axis_y: f32, axis_z: f32, phase: u32, wobble: u32, noise: u32, dash: u32, glow: u32, seed: u32) void  // Mode 7

// Blending and rendering
env_blend(mode: u32) void             // 0=alpha, 1=add, 2=mul, 3=screen
draw_env() void                       // Call first in render()
matcap_set(slot: u32, texture: u32) void  // Slot 1-3 (Mode 1 only)
```
{{#endtab}}

{{#endtabs}}

---

## 2D Drawing

**Note:** Use `set_color(0xRRGGBBAA)` before drawing to set the tint color. Source coordinates (`src_*`) are UV values (0.0-1.0), not pixels.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Sprites (use set_color() for tinting)
draw_sprite(x, y, w, h)
draw_sprite_region(x, y, w, h, src_x, src_y, src_w, src_h)  // UV coords (0.0-1.0)
draw_sprite_ex(x, y, w, h, src_x, src_y, src_w, src_h, ox, oy, angle)

// Primitives (use set_color() for color)
draw_rect(x, y, w, h)
draw_line(x1, y1, x2, y2, thickness)
draw_circle(x, y, radius)                      // Filled, 16 segments
draw_circle_outline(x, y, radius, thickness)

// Text (use set_color() for color)
draw_text(ptr, len, x, y, size)
text_width(ptr, len, size) -> f32              // Measure text width
load_font(tex, char_w, char_h, first_cp, count) -> u32
load_font_ex(tex, widths_ptr, char_h, first_cp, count) -> u32
font_bind(handle)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Sprites (use set_color() for tinting)
void draw_sprite(float x, float y, float w, float h);
void draw_sprite_region(float x, float y, float w, float h,
                        float src_x, float src_y, float src_w, float src_h);  // UV coords (0.0-1.0)
void draw_sprite_ex(float x, float y, float w, float h,
                    float src_x, float src_y, float src_w, float src_h,
                    float ox, float oy, float angle);

// Primitives (use set_color() for color)
void draw_rect(float x, float y, float w, float h);
void draw_line(float x1, float y1, float x2, float y2, float thickness);
void draw_circle(float x, float y, float radius);
void draw_circle_outline(float x, float y, float radius, float thickness);

// Text (use set_color() for color)
void draw_text(const uint8_t* ptr, uint32_t len, float x, float y, float size);
float text_width(const uint8_t* ptr, uint32_t len, float size);
uint32_t load_font(uint32_t tex, uint32_t char_w, uint32_t char_h, uint32_t first_cp, uint32_t count);
uint32_t load_font_ex(uint32_t tex, const uint8_t* widths, uint32_t char_h, uint32_t first_cp, uint32_t count);
void font_bind(uint32_t handle);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Sprites (use set_color() for tinting)
draw_sprite(x: f32, y: f32, w: f32, h: f32) void
draw_sprite_region(x: f32, y: f32, w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32) void  // UV coords (0.0-1.0)
draw_sprite_ex(x: f32, y: f32, w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32, ox: f32, oy: f32, angle: f32) void

// Primitives (use set_color() for color)
draw_rect(x: f32, y: f32, w: f32, h: f32) void
draw_line(x1: f32, y1: f32, x2: f32, y2: f32, thickness: f32) void
draw_circle(x: f32, y: f32, radius: f32) void
draw_circle_outline(x: f32, y: f32, radius: f32, thickness: f32) void

// Text (use set_color() for color)
draw_text(ptr: [*]const u8, len: u32, x: f32, y: f32, size: f32) void
text_width(ptr: [*]const u8, len: u32, size: f32) f32
load_font(tex: u32, char_w: u32, char_h: u32, first_cp: u32, count: u32) u32
load_font_ex(tex: u32, widths: [*]const u8, char_h: u32, first_cp: u32, count: u32) u32
font_bind(handle: u32) void
```
{{#endtab}}

{{#endtabs}}

---

## Billboards

**Note:** Use `set_color(0xRRGGBBAA)` before drawing to set the tint color. UV coordinates are 0.0-1.0.

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
draw_billboard(w, h, mode)             // mode: 1=sphere, 2=cylY, 3=cylX, 4=cylZ
draw_billboard_region(w, h, sx, sy, sw, sh, mode)  // UV coords (0.0-1.0)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
void draw_billboard(float w, float h, uint32_t mode);
void draw_billboard_region(float w, float h, float sx, float sy, float sw, float sh, uint32_t mode);
// Modes: NCZX_BILLBOARD_SPHERICAL, NCZX_BILLBOARD_CYLINDRICAL_Y/X/Z
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
draw_billboard(w: f32, h: f32, mode: u32) void
draw_billboard_region(w: f32, h: f32, sx: f32, sy: f32, sw: f32, sh: f32, mode: u32) void
// Modes: Billboard.spherical, Billboard.cylindrical_y/x/z
```
{{#endtab}}

{{#endtabs}}

---

## Skinning

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
load_skeleton(inverse_bind_ptr, bone_count) -> u32  // Init-only
skeleton_bind(skeleton)                // 0 to disable
set_bones(matrices_ptr, count)         // 12 floats per bone (3x4)
set_bones_4x4(matrices_ptr, count)     // 16 floats per bone (4x4)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t load_skeleton(const float* inverse_bind_ptr, uint32_t bone_count);  // Init-only
void skeleton_bind(uint32_t skeleton); // 0 to disable
void set_bones(const float* matrices_ptr, uint32_t count);  // 12 floats per bone (3x4)
void set_bones_4x4(const float* matrices_ptr, uint32_t count);  // 16 floats per bone (4x4)
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
load_skeleton(inverse_bind: [*]const f32, bone_count: u32) u32  // Init-only
skeleton_bind(skeleton: u32) void      // 0 to disable
set_bones(matrices: [*]const f32, count: u32) void  // 12 floats per bone (3x4)
set_bones_4x4(matrices: [*]const f32, count: u32) void  // 16 floats per bone (4x4)
```
{{#endtab}}

{{#endtabs}}

---

## Animation

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
keyframes_load(data_ptr, byte_size) -> u32  // Init-only
rom_keyframes(id_ptr, id_len) -> u32        // Init-only
keyframes_bone_count(handle) -> u32
keyframes_frame_count(handle) -> u32
keyframe_bind(handle, frame_index)          // GPU-side, no CPU decode
keyframe_read(handle, frame_index, out_ptr) // Read to WASM for blending
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t keyframes_load(const uint8_t* data, uint32_t byte_size);  // Init-only
uint32_t rom_keyframes(uint32_t id_ptr, uint32_t id_len);          // Init-only
uint32_t keyframes_bone_count(uint32_t handle);
uint32_t keyframes_frame_count(uint32_t handle);
void keyframe_bind(uint32_t handle, uint32_t frame_index);  // GPU-side
void keyframe_read(uint32_t handle, uint32_t frame_index, float* out_ptr);  // Read for blending
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
keyframes_load(data: [*]const u8, byte_size: u32) u32  // Init-only
rom_keyframes(id_ptr: u32, id_len: u32) u32            // Init-only
keyframes_bone_count(handle: u32) u32
keyframes_frame_count(handle: u32) u32
keyframe_bind(handle: u32, frame_index: u32) void      // GPU-side
keyframe_read(handle: u32, frame_index: u32, out: [*]f32) void  // Read for blending
```
{{#endtab}}

{{#endtabs}}

---

## Audio (SFX)

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
load_sound(data_ptr, byte_len) -> u32  // Init-only, 22kHz 16-bit mono
play_sound(sound, volume, pan)         // Auto-select channel
channel_play(ch, sound, vol, pan, loop)
channel_set(ch, volume, pan)
channel_stop(ch)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t load_sound(const int16_t* data, uint32_t byte_len);  // Init-only
void play_sound(uint32_t sound, float volume, float pan);  // Auto-select channel
void channel_play(uint32_t ch, uint32_t sound, float vol, float pan, uint32_t loop);
void channel_set(uint32_t ch, float volume, float pan);
void channel_stop(uint32_t ch);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
load_sound(data: [*]const i16, byte_len: u32) u32  // Init-only
play_sound(sound: u32, volume: f32, pan: f32) void  // Auto-select channel
channel_play(ch: u32, sound: u32, vol: f32, pan: f32, loop: u32) void
channel_set(ch: u32, volume: f32, pan: f32) void
channel_stop(ch: u32) void
```
{{#endtab}}

{{#endtabs}}

---

## Unified Music API (PCM + XM Tracker)

Works with both PCM sounds and XM tracker modules. Handle type detected by bit 31 (0=PCM, 1=tracker).

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Loading (init-only)
rom_tracker(id_ptr, id_len) -> u32     // Load XM from ROM (returns tracker handle)
load_tracker(data_ptr, len) -> u32     // Load XM from raw data

// Playback (works for both PCM and tracker)
music_play(handle, volume, looping)    // Start playing (auto-stops other type)
music_stop()                            // Stop all music
music_pause(paused)                     // 0=resume, 1=pause (tracker only)
music_set_volume(volume)                // 0.0-1.0
music_is_playing() -> u32               // 1 if playing
music_type() -> u32                     // 0=none, 1=PCM, 2=tracker

// Position (tracker-specific, no-op for PCM)
music_jump(order, row)                  // Jump to position
music_position() -> u32                 // Tracker: (order << 16) | row, PCM: sample pos
music_length(handle) -> u32             // Tracker: orders, PCM: samples
music_set_speed(speed)                  // Ticks per row (1-31)
music_set_tempo(bpm)                    // BPM (32-255)

// Query
music_info(handle) -> u32               // Tracker: (ch<<24)|(pat<<16)|(inst<<8)|len
music_name(handle, out_ptr, max) -> u32 // Tracker only (returns 0 for PCM)
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Loading (init-only)
uint32_t rom_tracker(uint32_t id_ptr, uint32_t id_len);  // Load XM from ROM
uint32_t load_tracker(const uint8_t* data, uint32_t len);

// Playback (works for both PCM and tracker)
void music_play(uint32_t handle, float volume, uint32_t looping);
void music_stop(void);
void music_pause(uint32_t paused);      // Tracker only
void music_set_volume(float volume);
uint32_t music_is_playing(void);
uint32_t music_type(void);              // 0=none, 1=PCM, 2=tracker

// Position (tracker-specific)
void music_jump(uint32_t order, uint32_t row);
uint32_t music_position(void);
uint32_t music_length(uint32_t handle);
void music_set_speed(uint32_t speed);
void music_set_tempo(uint32_t bpm);

// Query
uint32_t music_info(uint32_t handle);
uint32_t music_name(uint32_t handle, uint8_t* out, uint32_t max);
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Loading (init-only)
rom_tracker(id_ptr: u32, id_len: u32) u32  // Load XM from ROM
load_tracker(data: [*]const u8, len: u32) u32

// Playback (works for both PCM and tracker)
music_play(handle: u32, volume: f32, looping: u32) void
music_stop() void
music_pause(paused: u32) void           // Tracker only
music_set_volume(volume: f32) void
music_is_playing() u32
music_type() u32                        // 0=none, 1=PCM, 2=tracker

// Position (tracker-specific)
music_jump(order: u32, row: u32) void
music_position() u32
music_length(handle: u32) u32
music_set_speed(speed: u32) void
music_set_tempo(bpm: u32) void

// Query
music_info(handle: u32) u32
music_name(handle: u32, out: [*]u8, max: u32) u32
```
{{#endtab}}

{{#endtabs}}

**Note:** PCM and tracker music are mutually exclusive. Starting one stops the other. Load samples via `rom_sound()` before `rom_tracker()` to map tracker instruments.

---

## Save Data

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
save(slot, data_ptr, data_len) -> u32  // 0=ok, 1=bad slot, 2=too big
load(slot, data_ptr, max_len) -> u32   // Returns bytes read
delete(slot) -> u32                    // 0=ok, 1=bad slot
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t save(uint32_t slot, const uint8_t* data, uint32_t len);  // 0=ok, 1=bad slot, 2=too big
uint32_t load(uint32_t slot, uint8_t* data, uint32_t max_len);    // Returns bytes read
uint32_t delete_save(uint32_t slot);   // 0=ok, 1=bad slot
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
save(slot: u32, data: [*]const u8, len: u32) u32  // 0=ok, 1=bad slot, 2=too big
load(slot: u32, data: [*]u8, max_len: u32) u32    // Returns bytes read
delete_save(slot: u32) u32             // 0=ok, 1=bad slot
```
{{#endtab}}

{{#endtabs}}

---

## ROM Loading (Init-Only)

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
rom_texture(id_ptr, id_len) -> u32
rom_mesh(id_ptr, id_len) -> u32
rom_skeleton(id_ptr, id_len) -> u32
rom_font(id_ptr, id_len) -> u32
rom_sound(id_ptr, id_len) -> u32
rom_keyframes(id_ptr, id_len) -> u32
rom_tracker(id_ptr, id_len) -> u32     // Load XM tracker
rom_data_len(id_ptr, id_len) -> u32
rom_data(id_ptr, id_len, out_ptr, max_len) -> u32
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
uint32_t rom_texture(uint32_t id_ptr, uint32_t id_len);
uint32_t rom_mesh(uint32_t id_ptr, uint32_t id_len);
uint32_t rom_skeleton(uint32_t id_ptr, uint32_t id_len);
uint32_t rom_font(uint32_t id_ptr, uint32_t id_len);
uint32_t rom_sound(uint32_t id_ptr, uint32_t id_len);
uint32_t rom_keyframes(uint32_t id_ptr, uint32_t id_len);
uint32_t rom_tracker(uint32_t id_ptr, uint32_t id_len);  // Load XM tracker
uint32_t rom_data_len(uint32_t id_ptr, uint32_t id_len);
uint32_t rom_data(uint32_t id_ptr, uint32_t id_len, uint32_t out_ptr, uint32_t max_len);
// Helpers: NCZX_ROM_TEXTURE("id"), NCZX_ROM_MESH("id"), etc.
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
rom_texture(id_ptr: u32, id_len: u32) u32
rom_mesh(id_ptr: u32, id_len: u32) u32
rom_skeleton(id_ptr: u32, id_len: u32) u32
rom_font(id_ptr: u32, id_len: u32) u32
rom_sound(id_ptr: u32, id_len: u32) u32
rom_keyframes(id_ptr: u32, id_len: u32) u32
rom_tracker(id_ptr: u32, id_len: u32) u32  // Load XM tracker
rom_data_len(id_ptr: u32, id_len: u32) u32
rom_data(id_ptr: u32, id_len: u32, out_ptr: u32, max_len: u32) u32
```
{{#endtab}}

{{#endtabs}}

---

## Debug

{{#tabs global="lang"}}

{{#tab name="Rust"}}
```rust
// Registration (init-only)
debug_register_i8/i16/i32(name_ptr, name_len, ptr)
debug_register_u8/u16/u32(name_ptr, name_len, ptr)
debug_register_f32(name_ptr, name_len, ptr)
debug_register_bool(name_ptr, name_len, ptr)
debug_register_i32_range(name_ptr, name_len, ptr, min, max)
debug_register_f32_range(name_ptr, name_len, ptr, min, max)
debug_register_u8_range/u16_range/i16_range(...)
debug_register_vec2/vec3/rect/color(name_ptr, name_len, ptr)
debug_register_fixed_i16_q8/i32_q8/i32_q16/i32_q24(...)

// Watch (read-only)
debug_watch_i8/i16/i32/u8/u16/u32/f32/bool(name_ptr, name_len, ptr)
debug_watch_vec2/vec3/rect/color(name_ptr, name_len, ptr)

// Groups
debug_group_begin(name_ptr, name_len)
debug_group_end()

// Frame control
debug_is_paused() -> i32               // 1 if paused
debug_get_time_scale() -> f32          // 1.0 = normal
```
{{#endtab}}

{{#tab name="C/C++"}}
```c
// Registration (init-only)
void debug_register_i8/i16/i32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
void debug_register_u8/u16/u32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
void debug_register_f32(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
void debug_register_bool(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
void debug_register_i32_range(uint32_t name_ptr, uint32_t name_len, uint32_t ptr, int32_t min, int32_t max);
void debug_register_f32_range(uint32_t name_ptr, uint32_t name_len, uint32_t ptr, float min, float max);
void debug_register_vec2/vec3/rect/color(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);

// Watch (read-only)
void debug_watch_i8/i16/i32/u8/u16/u32/f32/bool(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);
void debug_watch_vec2/vec3/rect/color(uint32_t name_ptr, uint32_t name_len, uint32_t ptr);

// Groups
void debug_group_begin(uint32_t name_ptr, uint32_t name_len);
void debug_group_end(void);

// Frame control
int32_t debug_is_paused(void);         // 1 if paused
float debug_get_time_scale(void);      // 1.0 = normal
```
{{#endtab}}

{{#tab name="Zig"}}
```zig
// Registration (init-only) - similar pattern for all types
debug_register_f32(name_ptr: u32, name_len: u32, ptr: u32) void
debug_register_i32(name_ptr: u32, name_len: u32, ptr: u32) void
debug_register_bool(name_ptr: u32, name_len: u32, ptr: u32) void
debug_register_i32_range(name_ptr: u32, name_len: u32, ptr: u32, min: i32, max: i32) void
debug_register_f32_range(name_ptr: u32, name_len: u32, ptr: u32, min: f32, max: f32) void
debug_register_vec2/vec3/rect/color(name_ptr: u32, name_len: u32, ptr: u32) void

// Watch (read-only) - similar pattern
debug_watch_f32(name_ptr: u32, name_len: u32, ptr: u32) void
debug_watch_vec2/vec3/rect/color(name_ptr: u32, name_len: u32, ptr: u32) void

// Groups
debug_group_begin(name_ptr: u32, name_len: u32) void
debug_group_end() void

// Frame control
debug_is_paused() i32                  // 1 if paused
debug_get_time_scale() f32             // 1.0 = normal
```
{{#endtab}}

{{#endtabs}}

**Keyboard:** F3=stats, F4=inspector, F5=pause, F6=step, F7/F8=time scale
