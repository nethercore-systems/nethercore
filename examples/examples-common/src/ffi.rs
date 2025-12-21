//! FFI declarations for Nethercore ZX
//!
//! Common FFI functions used by all inspector examples.

#[link(wasm_import_module = "env")]
extern "C" {
    // ========================================================================
    // Configuration (init-only)
    // ========================================================================
    pub fn set_clear_color(color: u32);
    pub fn render_mode(mode: u32);

    // ========================================================================
    // Camera
    // ========================================================================
    pub fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    pub fn camera_fov(fov_degrees: f32);

    // ========================================================================
    // Input
    // ========================================================================
    pub fn left_stick_x(player: u32) -> f32;
    pub fn left_stick_y(player: u32) -> f32;
    pub fn right_stick_x(player: u32) -> f32;
    pub fn right_stick_y(player: u32) -> f32;
    pub fn trigger_left(player: u32) -> f32;
    pub fn trigger_right(player: u32) -> f32;
    pub fn button_pressed(player: u32, button: u32) -> u32;
    pub fn button_held(player: u32, button: u32) -> u32;

    // ========================================================================
    // Time
    // ========================================================================
    pub fn elapsed_time() -> f32;
    pub fn delta_time() -> f32;

    // ========================================================================
    // Procedural Mesh Generation
    // ========================================================================
    pub fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    pub fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    pub fn torus(major_radius: f32, minor_radius: f32, major_segments: u32, minor_segments: u32) -> u32;

    // ========================================================================
    // Textures
    // ========================================================================
    pub fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32;
    pub fn texture_bind(handle: u32);
    pub fn texture_bind_slot(handle: u32, slot: u32);
    pub fn texture_filter(filter: u32);

    // ========================================================================
    // Mesh Drawing
    // ========================================================================
    pub fn draw_mesh(handle: u32);

    // ========================================================================
    // Transform
    // ========================================================================
    pub fn push_identity();
    pub fn push_translate(x: f32, y: f32, z: f32);
    pub fn push_rotate_x(angle_deg: f32);
    pub fn push_rotate_y(angle_deg: f32);
    pub fn push_rotate_z(angle_deg: f32);
    pub fn push_scale(x: f32, y: f32, z: f32);
    pub fn push_scale_uniform(scale: f32);

    // ========================================================================
    // Render State
    // ========================================================================
    pub fn set_color(color: u32);
    pub fn depth_test(enabled: u32);
    pub fn cull_mode(mode: u32);
    pub fn blend_mode(mode: u32);

    // ========================================================================
    // Sky
    // ========================================================================
    pub fn sky_set_colors(horizon_color: u32, zenith_color: u32);
    pub fn draw_sky();

    // ========================================================================
    // Environment (Multi-Environment v3)
    // ========================================================================
    pub fn env_gradient_set(
        zenith: u32,
        sky_horizon: u32,
        ground_horizon: u32,
        nadir: u32,
        rotation: f32,
        shift: f32,
    );
    pub fn env_scatter_set(
        variant: u32,
        density: u32,
        size: u32,
        glow: u32,
        streak_length: u32,
        color_primary: u32,
        color_secondary: u32,
        parallax_rate: u32,
        parallax_size: u32,
        phase: u32,
    );
    pub fn env_lines_set(
        variant: u32,
        line_type: u32,
        thickness: u32,
        spacing: f32,
        fade_distance: f32,
        color_primary: u32,
        color_accent: u32,
        accent_every: u32,
        phase: u32,
    );
    pub fn env_silhouette_set(
        jaggedness: u32,
        layer_count: u32,
        color_near: u32,
        color_far: u32,
        sky_zenith: u32,
        sky_horizon: u32,
        parallax_rate: u32,
        seed: u32,
    );
    pub fn env_rectangles_set(
        variant: u32,
        density: u32,
        lit_ratio: u32,
        size_min: u32,
        size_max: u32,
        aspect: u32,
        color_primary: u32,
        color_variation: u32,
        parallax_rate: u32,
        phase: u32,
    );
    pub fn env_room_set(
        color_ceiling: u32,
        color_floor: u32,
        color_walls: u32,
        panel_size: f32,
        panel_gap: u32,
        light_dir_x: f32,
        light_dir_y: f32,
        light_dir_z: f32,
        light_intensity: u32,
        corner_darken: u32,
        room_scale: f32,
        viewer_x: i32,
        viewer_y: i32,
        viewer_z: i32,
    );
    pub fn env_curtains_set(
        layer_count: u32,
        density: u32,
        height_min: u32,
        height_max: u32,
        width: u32,
        spacing: u32,
        waviness: u32,
        color_near: u32,
        color_far: u32,
        glow: u32,
        parallax_rate: u32,
        phase: u32,
    );
    pub fn env_rings_set(
        ring_count: u32,
        thickness: u32,
        color_a: u32,
        color_b: u32,
        center_color: u32,
        center_falloff: u32,
        spiral_twist: f32,
        axis_x: f32,
        axis_y: f32,
        axis_z: f32,
        phase: u32,
    );
    pub fn env_select_pair(base_mode: u32, overlay_mode: u32);
    pub fn env_blend_mode(mode: u32);

    // ========================================================================
    // Lighting (Mode 2 & 3)
    // ========================================================================
    pub fn light_set(index: u32, x: f32, y: f32, z: f32);
    pub fn light_set_point(index: u32, x: f32, y: f32, z: f32);
    pub fn light_color(index: u32, color: u32);
    pub fn light_intensity(index: u32, intensity: f32);
    pub fn light_range(index: u32, range: f32);
    pub fn light_enable(index: u32);
    pub fn light_disable(index: u32);

    // ========================================================================
    // Materials (Mode 2)
    // ========================================================================
    pub fn material_metallic(value: f32);
    pub fn material_roughness(value: f32);
    pub fn material_emissive(value: f32);
    pub fn material_rim(intensity: f32, power: f32);

    // ========================================================================
    // Materials (Mode 3)
    // ========================================================================
    pub fn material_shininess(value: f32);
    pub fn material_specular(color: u32);

    // ========================================================================
    // Matcap (Mode 1)
    // ========================================================================
    pub fn matcap_set(slot: u32, texture: u32);
    pub fn matcap_blend_mode(slot: u32, mode: u32);

    // ========================================================================
    // 2D UI
    // ========================================================================
    pub fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    pub fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
    pub fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32);

    // ========================================================================
    // Debug Inspection
    // ========================================================================
    pub fn debug_group_begin(name: *const u8, name_len: u32);
    pub fn debug_group_end();
    pub fn debug_register_f32(name: *const u8, name_len: u32, ptr: *const f32);
    pub fn debug_register_i32(name: *const u8, name_len: u32, ptr: *const i32);
    pub fn debug_register_u8(name: *const u8, name_len: u32, ptr: *const u8);
    pub fn debug_register_bool(name: *const u8, name_len: u32, ptr: *const u8);
    pub fn debug_register_color(name: *const u8, name_len: u32, ptr: *const u8);
    pub fn debug_watch_f32(name: *const u8, name_len: u32, ptr: *const f32);
    pub fn debug_watch_i32(name: *const u8, name_len: u32, ptr: *const i32);
}

// Button constants
pub const BUTTON_UP: u32 = 0;
pub const BUTTON_DOWN: u32 = 1;
pub const BUTTON_LEFT: u32 = 2;
pub const BUTTON_RIGHT: u32 = 3;
pub const BUTTON_A: u32 = 4;
pub const BUTTON_B: u32 = 5;
pub const BUTTON_X: u32 = 6;
pub const BUTTON_Y: u32 = 7;
pub const BUTTON_LB: u32 = 8;
pub const BUTTON_RB: u32 = 9;
pub const BUTTON_L3: u32 = 10;
pub const BUTTON_R3: u32 = 11;
pub const BUTTON_START: u32 = 12;
pub const BUTTON_SELECT: u32 = 13;
