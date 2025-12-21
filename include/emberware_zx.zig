///! Emberware ZX FFI Bindings for Zig
///!
///! This module provides all FFI function declarations for Emberware ZX games.
///! Import this module and implement init(), update(), and render().
///!
///! Usage:
///!   const zx = @import("emberware_zx.zig");
///!
///!   export fn init() void {
///!       zx.set_clear_color(zx.color.DARK_BLUE);
///!       zx.render_mode(zx.RenderMode.pbr);
///!   }
///!
///!   export fn update() void {
///!       if (zx.button_pressed(0, zx.Button.a) != 0) {
///!           // Handle input
///!       }
///!   }
///!
///!   export fn render() void {
///!       zx.draw_sky();
///!   }

// =============================================================================
// System Functions
// =============================================================================

/// Returns the fixed timestep duration in seconds.
/// This is a CONSTANT based on tick rate, NOT wall-clock time.
pub extern fn delta_time() f32;

/// Returns total elapsed game time since start in seconds.
pub extern fn elapsed_time() f32;

/// Returns the current tick number (starts at 0).
pub extern fn tick_count() u64;

/// Logs a message to the console output.
pub extern fn log_msg(ptr: [*]const u8, len: u32) void;

/// Exits the game and returns to the library.
pub extern fn quit() void;

/// Returns a deterministic random u32 from the host's seeded RNG.
pub extern fn random_u32() u32;

// =============================================================================
// Session Functions
// =============================================================================

/// Returns the number of players in the session (1-4).
pub extern fn player_count() u32;

/// Returns a bitmask of which players are local to this client.
pub extern fn local_player_mask() u32;

// =============================================================================
// Save Data Functions
// =============================================================================

pub extern fn save(slot: u32, data_ptr: [*]const u8, data_len: u32) u32;
pub extern fn load(slot: u32, data_ptr: [*]u8, max_len: u32) u32;
pub extern fn delete_save(slot: u32) u32;

// =============================================================================
// Configuration Functions (init-only)
// =============================================================================

pub extern fn set_tick_rate(rate: u32) void;
pub extern fn set_clear_color(color: u32) void;
pub extern fn render_mode(mode: u32) void;

// =============================================================================
// Camera Functions
// =============================================================================

pub extern fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32) void;
pub extern fn camera_fov(fov_degrees: f32) void;

// =============================================================================
// Transform Functions
// =============================================================================

pub extern fn push_identity() void;
pub extern fn transform_set(matrix_ptr: [*]const f32) void;
pub extern fn push_translate(x: f32, y: f32, z: f32) void;
pub extern fn push_rotate_x(angle_deg: f32) void;
pub extern fn push_rotate_y(angle_deg: f32) void;
pub extern fn push_rotate_z(angle_deg: f32) void;
pub extern fn push_rotate(angle_deg: f32, axis_x: f32, axis_y: f32, axis_z: f32) void;
pub extern fn push_scale(x: f32, y: f32, z: f32) void;
pub extern fn push_scale_uniform(s: f32) void;

// =============================================================================
// Input Functions
// =============================================================================

pub extern fn button_held(player: u32, button: u32) u32;
pub extern fn button_pressed(player: u32, button: u32) u32;
pub extern fn button_released(player: u32, button: u32) u32;
pub extern fn buttons_held(player: u32) u32;
pub extern fn buttons_pressed(player: u32) u32;
pub extern fn buttons_released(player: u32) u32;

pub extern fn left_stick_x(player: u32) f32;
pub extern fn left_stick_y(player: u32) f32;
pub extern fn right_stick_x(player: u32) f32;
pub extern fn right_stick_y(player: u32) f32;
pub extern fn left_stick(player: u32, out_x: *f32, out_y: *f32) void;
pub extern fn right_stick(player: u32, out_x: *f32, out_y: *f32) void;

pub extern fn trigger_left(player: u32) f32;
pub extern fn trigger_right(player: u32) f32;

// =============================================================================
// Render State Functions
// =============================================================================

pub extern fn set_color(color: u32) void;
pub extern fn depth_test(enabled: u32) void;
pub extern fn cull_mode(mode: u32) void;
pub extern fn blend_mode(mode: u32) void;
pub extern fn texture_filter(filter: u32) void;

// =============================================================================
// Texture Functions
// =============================================================================

pub extern fn load_texture(width: u32, height: u32, pixels_ptr: [*]const u8) u32;
pub extern fn texture_bind(handle: u32) void;
pub extern fn texture_bind_slot(handle: u32, slot: u32) void;

// =============================================================================
// Mesh Functions
// =============================================================================

pub extern fn load_mesh(data_ptr: [*]const f32, vertex_count: u32, format: u32) u32;
pub extern fn load_mesh_indexed(data_ptr: [*]const f32, vertex_count: u32, index_ptr: [*]const u16, index_count: u32, format: u32) u32;
pub extern fn draw_mesh(handle: u32) void;

// Procedural mesh generation
pub extern fn cube(size_x: f32, size_y: f32, size_z: f32) u32;
pub extern fn sphere(radius: f32, segments: u32, rings: u32) u32;
pub extern fn cylinder(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) u32;
pub extern fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) u32;
pub extern fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) u32;
pub extern fn capsule(radius: f32, height: f32, segments: u32, rings: u32) u32;

// UV variants
pub extern fn sphere_uv(radius: f32, segments: u32, rings: u32) u32;
pub extern fn plane_uv(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) u32;
pub extern fn cube_uv(size_x: f32, size_y: f32, size_z: f32) u32;
pub extern fn cylinder_uv(radius_bottom: f32, radius_top: f32, height: f32, segments: u32) u32;
pub extern fn torus_uv(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) u32;
pub extern fn capsule_uv(radius: f32, height: f32, segments: u32, rings: u32) u32;

// Immediate mode
pub extern fn draw_triangles(data_ptr: [*]const f32, vertex_count: u32, format: u32) void;
pub extern fn draw_triangles_indexed(data_ptr: [*]const f32, vertex_count: u32, index_ptr: [*]const u16, index_count: u32, format: u32) void;

// =============================================================================
// 2D Drawing
// =============================================================================

pub extern fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32) void;
pub extern fn draw_sprite_region(x: f32, y: f32, w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32, color: u32) void;
pub extern fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32) void;
pub extern fn draw_text(ptr: [*]const u8, len: u32, x: f32, y: f32, size: f32, color: u32) void;

pub extern fn load_font(texture: u32, char_width: u32, char_height: u32, first_codepoint: u32, char_count: u32) u32;
pub extern fn font_bind(font_handle: u32) void;

// =============================================================================
// Billboard Drawing
// =============================================================================

pub extern fn draw_billboard(w: f32, h: f32, mode: u32, color: u32) void;
pub extern fn draw_billboard_region(w: f32, h: f32, src_x: f32, src_y: f32, src_w: f32, src_h: f32, mode: u32, color: u32) void;

// =============================================================================
// Sky System
// =============================================================================

pub extern fn sky_set_colors(horizon_color: u32, zenith_color: u32) void;
pub extern fn sky_set_sun(dir_x: f32, dir_y: f32, dir_z: f32, color: u32, sharpness: f32) void;
pub extern fn draw_sky() void;

// =============================================================================
// Material Functions
// =============================================================================

pub extern fn material_mre(texture: u32) void;
pub extern fn material_albedo(texture: u32) void;
pub extern fn material_metallic(value: f32) void;
pub extern fn material_roughness(value: f32) void;
pub extern fn material_emissive(value: f32) void;
pub extern fn material_rim(intensity: f32, power: f32) void;
pub extern fn material_shininess(value: f32) void;
pub extern fn material_specular(color: u32) void;

// =============================================================================
// Lighting Functions
// =============================================================================

pub extern fn light_set(index: u32, x: f32, y: f32, z: f32) void;
pub extern fn light_color(index: u32, color: u32) void;
pub extern fn light_intensity(index: u32, intensity: f32) void;
pub extern fn light_enable(index: u32) void;
pub extern fn light_disable(index: u32) void;
pub extern fn light_set_point(index: u32, x: f32, y: f32, z: f32) void;
pub extern fn light_range(index: u32, range: f32) void;

// =============================================================================
// GPU Skinning
// =============================================================================

pub extern fn load_skeleton(inverse_bind_ptr: [*]const f32, bone_count: u32) u32;
pub extern fn skeleton_bind(skeleton: u32) void;
pub extern fn set_bones(matrices_ptr: [*]const f32, count: u32) void;

// =============================================================================
// Audio Functions
// =============================================================================

pub extern fn load_sound(data_ptr: [*]const i16, byte_len: u32) u32;
pub extern fn play_sound(sound: u32, volume: f32, pan: f32) void;
pub extern fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32) void;
pub extern fn channel_set(channel: u32, volume: f32, pan: f32) void;
pub extern fn channel_stop(channel: u32) void;
pub extern fn music_play(sound: u32, volume: f32) void;
pub extern fn music_stop() void;
pub extern fn music_set_volume(volume: f32) void;

// =============================================================================
// ROM Data Pack API
// =============================================================================

pub extern fn rom_texture(id_ptr: u32, id_len: u32) u32;
pub extern fn rom_mesh(id_ptr: u32, id_len: u32) u32;
pub extern fn rom_skeleton(id_ptr: u32, id_len: u32) u32;
pub extern fn rom_font(id_ptr: u32, id_len: u32) u32;
pub extern fn rom_sound(id_ptr: u32, id_len: u32) u32;
pub extern fn rom_data_len(id_ptr: u32, id_len: u32) u32;
pub extern fn rom_data(id_ptr: u32, id_len: u32, dst_ptr: u32, max_len: u32) u32;

// =============================================================================
// Constants
// =============================================================================

pub const Button = struct {
    pub const up: u32 = 0;
    pub const down: u32 = 1;
    pub const left: u32 = 2;
    pub const right: u32 = 3;
    pub const a: u32 = 4;
    pub const b: u32 = 5;
    pub const x: u32 = 6;
    pub const y: u32 = 7;
    pub const l1: u32 = 8;
    pub const r1: u32 = 9;
    pub const l3: u32 = 10;
    pub const r3: u32 = 11;
    pub const start: u32 = 12;
    pub const select: u32 = 13;
};

pub const RenderMode = struct {
    pub const lambert: u32 = 0;
    pub const matcap: u32 = 1;
    pub const pbr: u32 = 2;
    pub const hybrid: u32 = 3;
};

pub const BlendMode = struct {
    pub const none: u32 = 0;
    pub const alpha: u32 = 1;
    pub const additive: u32 = 2;
    pub const multiply: u32 = 3;
};

pub const CullMode = struct {
    pub const none: u32 = 0;
    pub const back: u32 = 1;
    pub const front: u32 = 2;
};

pub const Format = struct {
    pub const pos: u32 = 0;
    pub const uv: u32 = 1;
    pub const color: u32 = 2;
    pub const normal: u32 = 4;
    pub const skinned: u32 = 8;

    pub const pos_uv: u32 = uv;
    pub const pos_color: u32 = color;
    pub const pos_normal: u32 = normal;
    pub const pos_uv_normal: u32 = uv | normal;
    pub const pos_uv_color: u32 = uv | color;
    pub const pos_uv_color_normal: u32 = uv | color | normal;
};

pub const TickRate = struct {
    pub const fps_24: u32 = 0;
    pub const fps_30: u32 = 1;
    pub const fps_60: u32 = 2;
    pub const fps_120: u32 = 3;
};

pub const Billboard = struct {
    pub const spherical: u32 = 1;
    pub const cylindrical_y: u32 = 2;
    pub const cylindrical_x: u32 = 3;
    pub const cylindrical_z: u32 = 4;
};

pub const color = struct {
    pub const WHITE: u32 = 0xFFFFFFFF;
    pub const BLACK: u32 = 0x000000FF;
    pub const RED: u32 = 0xFF0000FF;
    pub const GREEN: u32 = 0x00FF00FF;
    pub const BLUE: u32 = 0x0000FFFF;
    pub const YELLOW: u32 = 0xFFFF00FF;
    pub const CYAN: u32 = 0x00FFFFFF;
    pub const MAGENTA: u32 = 0xFF00FFFF;
    pub const ORANGE: u32 = 0xFF8000FF;
    pub const TRANSPARENT: u32 = 0x00000000;
    pub const DARK_BLUE: u32 = 0x1a1a2eFF;
};

// =============================================================================
// Helper Functions
// =============================================================================

/// Pack RGBA color components into a u32.
pub fn rgba(r: u8, g: u8, b: u8, a: u8) u32 {
    return (@as(u32, r) << 24) | (@as(u32, g) << 16) | (@as(u32, b) << 8) | @as(u32, a);
}

/// Pack RGB color components into a u32 (alpha = 255).
pub fn rgb(r: u8, g: u8, b: u8) u32 {
    return rgba(r, g, b, 255);
}

/// Helper to log a string.
pub fn log(msg: []const u8) void {
    log_msg(msg.ptr, @intCast(msg.len));
}

/// Helper to draw text from a slice.
pub fn text(msg: []const u8, x: f32, y: f32, size: f32, col: u32) void {
    draw_text(msg.ptr, @intCast(msg.len), x, y, size, col);
}

/// Clamp a float value between min and max.
pub fn clampf(val: f32, min: f32, max: f32) f32 {
    return @max(min, @min(val, max));
}

/// Linear interpolation.
pub fn lerpf(a: f32, b: f32, t: f32) f32 {
    return a + (b - a) * t;
}
