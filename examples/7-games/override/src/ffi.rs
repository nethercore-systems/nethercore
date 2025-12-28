// Nethercore ZX FFI Bindings (minimal subset for OVERRIDE)
// Local copy without inner doc comments to avoid include! issues

#![allow(unused)]

// Button constants
pub const BTN_UP: u32 = 0;
pub const BTN_DOWN: u32 = 1;
pub const BTN_LEFT: u32 = 2;
pub const BTN_RIGHT: u32 = 3;
pub const BTN_A: u32 = 4;
pub const BTN_B: u32 = 5;
pub const BTN_X: u32 = 6;
pub const BTN_Y: u32 = 7;
pub const BTN_L1: u32 = 8;
pub const BTN_R1: u32 = 9;
pub const BTN_L3: u32 = 10;
pub const BTN_R3: u32 = 11;
pub const BTN_START: u32 = 12;
pub const BTN_SELECT: u32 = 13;

#[link(wasm_import_module = "env")]
extern "C" {
    // System
    pub fn delta_time() -> f32;
    pub fn elapsed_time() -> f32;
    pub fn tick_count() -> u64;
    pub fn log(ptr: *const u8, len: u32);
    pub fn quit();
    pub fn set_tick_rate(rate: u32);

    // Rollback-safe random
    pub fn random() -> u32;
    pub fn random_range(min: i32, max: i32) -> i32;
    pub fn random_f32() -> f32;

    // Session
    pub fn player_count() -> u32;
    pub fn local_player_mask() -> u32;

    // Input - buttons (returns 1 or 0)
    pub fn button_held(player: u32, button: u32) -> u32;
    pub fn button_pressed(player: u32, button: u32) -> u32;
    pub fn button_released(player: u32, button: u32) -> u32;
    pub fn buttons_held(player: u32) -> u32;
    pub fn buttons_pressed(player: u32) -> u32;

    // Input - analog
    pub fn left_stick_x(player: u32) -> f32;
    pub fn left_stick_y(player: u32) -> f32;
    pub fn right_stick_x(player: u32) -> f32;
    pub fn right_stick_y(player: u32) -> f32;

    // Render state
    pub fn set_color(color: u32);
    pub fn depth_test(enabled: u32);
    pub fn cull_mode(mode: u32);
    pub fn texture_filter(filter: u32);
    pub fn set_dither_level(level: u32);
    pub fn set_clear_color(color: u32);
    pub fn clear();

    // Camera and transforms
    pub fn render_mode(mode: u32);
    pub fn camera_pos(x: f32, y: f32, z: f32);
    pub fn camera_look_at(x: f32, y: f32, z: f32);
    pub fn camera_fov(degrees: f32);
    pub fn transform_identity();
    pub fn transform_translate(x: f32, y: f32, z: f32);
    pub fn transform_rotate(angle_deg: f32, x: f32, y: f32, z: f32);
    pub fn transform_scale(x: f32, y: f32, z: f32);
    pub fn transform_push();
    pub fn transform_pop();

    // 2D Drawing
    pub fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    pub fn draw_rect_outline(x: f32, y: f32, w: f32, h: f32, color: u32);
    pub fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    // Sprites and textures
    pub fn load_texture(data: *const u8, len: u32) -> u32;
    pub fn bind_texture(handle: u32);
    pub fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32);
    pub fn draw_sprite_region(
        x: f32, y: f32, w: f32, h: f32,
        src_x: f32, src_y: f32, src_w: f32, src_h: f32,
        color: u32
    );

    // Audio
    pub fn load_sound(data: *const u8, len: u32) -> u32;
    pub fn play_sound(sound: u32, volume: f32, pan: f32);
    pub fn load_music(data: *const u8, len: u32) -> u32;
    pub fn music_play(handle: u32, volume: f32, looping: u32);
    pub fn music_stop();
    pub fn set_music_volume(volume: f32);

    // ROM loading
    pub fn rom_texture(id: *const u8, len: u32) -> u32;
    pub fn rom_sound(id: *const u8, len: u32) -> u32;
    pub fn rom_music(id: *const u8, len: u32) -> u32;

    // Procedural drawing
    pub fn draw_box(x: f32, y: f32, z: f32, w: f32, h: f32, d: f32);
    pub fn draw_sphere(x: f32, y: f32, z: f32, radius: f32, segments: u32);
    pub fn draw_cylinder(x: f32, y: f32, z: f32, radius: f32, height: f32, segments: u32);
}
