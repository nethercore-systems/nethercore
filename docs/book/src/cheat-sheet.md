# Cheat Sheet

All Emberware ZX FFI functions on one page.

---

## System

```rust
delta_time() -> f32                    // Seconds since last tick
elapsed_time() -> f32                  // Total seconds since start
tick_count() -> u64                    // Current tick number
log(ptr, len)                          // Log message to console
quit()                                 // Exit to library
random() -> u32                        // Deterministic random
player_count() -> u32                  // Number of players (1-4)
local_player_mask() -> u32             // Bitmask of local players
```

---

## Configuration (Init-Only)

```rust
set_resolution(res)                    // 0=360p, 1=540p, 2=720p, 3=1080p
set_tick_rate(fps)                     // 0=24, 1=30, 2=60, 3=120
set_clear_color(0xRRGGBBAA)            // Background color
render_mode(mode)                      // 0=Unlit, 1=Matcap, 2=MR, 3=SS
```

---

## Input

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

---

## Camera

```rust
camera_set(x, y, z, target_x, target_y, target_z)
camera_fov(degrees)                    // Default: 60
push_view_matrix(m0..m15)              // Custom 4x4 view matrix
push_projection_matrix(m0..m15)        // Custom 4x4 projection
```

---

## Transforms

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

---

## Render State

```rust
set_color(0xRRGGBBAA)                  // Tint color
depth_test(enabled)                    // 0=off, 1=on
cull_mode(mode)                        // 0=none, 1=back, 2=front
blend_mode(mode)                       // 0=none, 1=alpha, 2=add, 3=mul
texture_filter(filter)                 // 0=nearest, 1=linear
uniform_alpha(level)                   // 0-15 dither alpha
dither_offset(x, y)                    // 0-3 pattern offset
```

---

## Textures

```rust
load_texture(w, h, pixels_ptr) -> u32  // Init-only, returns handle
texture_bind(handle)                   // Bind to slot 0
texture_bind_slot(handle, slot)        // Bind to slot 0-3
matcap_blend_mode(slot, mode)          // 0=mul, 1=add, 2=hsv
```

---

## Meshes

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

**Vertex Formats:** POS=0, UV=1, COLOR=2, UV_COLOR=3, NORMAL=4, UV_NORMAL=5, COLOR_NORMAL=6, UV_COLOR_NORMAL=7, +SKINNED=8

---

## Procedural Meshes (Init-Only)

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

---

## Materials

```rust
// Mode 2 (Metallic-Roughness)
material_metallic(value)               // 0.0-1.0
material_roughness(value)              // 0.0-1.0
material_emissive(value)               // Glow intensity
material_rim(intensity, power)         // Rim light
material_albedo(texture)               // Bind to slot 0
material_mre(texture)                  // Bind MRE to slot 1

// Mode 3 (Specular-Shininess)
material_shininess(value)              // 0.0-1.0 → 1-256
material_specular(0xRRGGBBAA)          // Specular color
material_specular_color(r, g, b)       // RGB floats
material_specular_damping(value)

// Override flags
use_uniform_color(enabled)
use_uniform_metallic(enabled)
use_uniform_roughness(enabled)
use_uniform_emissive(enabled)
use_uniform_specular(enabled)
use_matcap_reflection(enabled)
```

---

## Lighting

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

---

## Sky & Matcap

```rust
sky_set_colors(horizon, zenith)        // 0xRRGGBBAA colors
sky_set_sun(dx, dy, dz, color, sharpness)
draw_sky()                             // Call first in render()
matcap_set(slot, texture)              // Slot 1-3
```

---

## 2D Drawing

```rust
draw_sprite(x, y, w, h, color)
draw_sprite_region(x, y, w, h, src_x, src_y, src_w, src_h, color)
draw_sprite_ex(x, y, w, h, src_x, src_y, src_w, src_h, ox, oy, angle, color)
draw_rect(x, y, w, h, color)
draw_text(ptr, len, x, y, size, color)
load_font(tex, char_w, char_h, first_cp, count) -> u32
load_font_ex(tex, widths_ptr, char_h, first_cp, count) -> u32
font_bind(handle)
```

---

## Billboards

```rust
draw_billboard(w, h, mode, color)      // mode: 1=sphere, 2=cylY, 3=cylX, 4=cylZ
draw_billboard_region(w, h, sx, sy, sw, sh, mode, color)
```

---

## Skinning

```rust
load_skeleton(inverse_bind_ptr, bone_count) -> u32  // Init-only
skeleton_bind(skeleton)                // 0 to disable
set_bones(matrices_ptr, count)         // 12 floats per bone (3x4)
set_bones_4x4(matrices_ptr, count)     // 16 floats per bone (4x4)
```

---

## Animation

```rust
keyframes_load(data_ptr, byte_size) -> u32  // Init-only
rom_keyframes(id_ptr, id_len) -> u32        // Init-only
keyframes_bone_count(handle) -> u32
keyframes_frame_count(handle) -> u32
keyframe_bind(handle, frame_index)          // GPU-side, no CPU decode
keyframe_read(handle, frame_index, out_ptr) // Read to WASM for blending
```

---

## Audio

```rust
load_sound(data_ptr, byte_len) -> u32  // Init-only, 22kHz 16-bit mono
play_sound(sound, volume, pan)         // Auto-select channel
channel_play(ch, sound, vol, pan, loop)
channel_set(ch, volume, pan)
channel_stop(ch)
music_play(sound, volume)
music_stop()
music_set_volume(volume)
```

---

## Save Data

```rust
save(slot, data_ptr, data_len) -> u32  // 0=ok, 1=bad slot, 2=too big
load(slot, data_ptr, max_len) -> u32   // Returns bytes read
delete(slot) -> u32                    // 0=ok, 1=bad slot
```

---

## ROM Loading (Init-Only)

```rust
rom_texture(id_ptr, id_len) -> u32
rom_mesh(id_ptr, id_len) -> u32
rom_skeleton(id_ptr, id_len) -> u32
rom_font(id_ptr, id_len) -> u32
rom_sound(id_ptr, id_len) -> u32
rom_keyframes(id_ptr, id_len) -> u32
rom_data_len(id_ptr, id_len) -> u32
rom_data(id_ptr, id_len, out_ptr, max_len) -> u32
```

---

## Debug

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

**Keyboard:** F3=panel, F5=pause, F6=step, F7/F8=time scale
