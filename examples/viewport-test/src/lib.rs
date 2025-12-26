//! Viewport Test Example
//!
//! Demonstrates split-screen rendering with the viewport() function.
//!
//! Features:
//! - 2-player horizontal split
//! - 4-player quad split
//! - Per-viewport camera with automatic aspect ratio
//! - Press A to toggle between 2P and 4P modes
//!
//! Each viewport shows the same scene from a different camera angle,
//! with camera aspect ratio automatically matching the viewport dimensions.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Viewport (split-screen)
    fn viewport(x: u32, y: u32, width: u32, height: u32);
    fn viewport_clear();

    // Input
    fn button_held(player: u32, button: u32) -> u32;
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;

    // Procedural mesh generation
    fn cube(size_x: f32, size_y: f32, size_z: f32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;
    fn plane(size_x: f32, size_z: f32, subdivisions_x: u32, subdivisions_z: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // 2D drawing
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}

// Input button constants
const BUTTON_A: u32 = 1;

// Screen dimensions
const SCREEN_WIDTH: u32 = 960;
const SCREEN_HEIGHT: u32 = 540;

// Mesh handles
static mut CUBE_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;
static mut FLOOR_MESH: u32 = 0;

// State
static mut IS_4_PLAYER: bool = false;
static mut PREV_A_BUTTON: u32 = 0;
static mut TIME: f32 = 0.0;

// Camera positions for each player (orbit around scene)
static mut CAMERA_ANGLES: [f32; 4] = [0.0, 90.0, 180.0, 270.0];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x2a2a3aFF);
        render_mode(0);
        depth_test(1);

        // Generate scene objects
        CUBE_MESH = cube(1.0, 1.0, 1.0);
        SPHERE_MESH = sphere(0.5, 16, 8);
        FLOOR_MESH = plane(10.0, 10.0, 4, 4);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        TIME += 1.0 / 60.0;

        // Toggle between 2P and 4P mode
        let a_button = button_held(0, BUTTON_A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            IS_4_PLAYER = !IS_4_PLAYER;
        }
        PREV_A_BUTTON = a_button;

        // Update camera angles based on player input
        for i in 0..4 {
            let stick_x = left_stick_x(i);
            if stick_x.abs() > 0.1 {
                CAMERA_ANGLES[i as usize] += stick_x * 2.0;
            }
        }
    }
}

/// Draw the scene from the current camera position
unsafe fn draw_scene() {
    // Floor
    push_identity();
    push_translate(0.0, -1.0, 0.0);
    set_color(0x404050FF);
    draw_mesh(FLOOR_MESH);

    // Central cube (rotating)
    push_identity();
    push_rotate_y(TIME * 30.0);
    set_color(0xFF6644FF);
    draw_mesh(CUBE_MESH);

    // Orbiting spheres
    for i in 0..4 {
        let angle = TIME * 60.0 + (i as f32) * 90.0;
        let rad = angle.to_radians();
        let x = libm::cosf(rad) * 3.0;
        let z = libm::sinf(rad) * 3.0;

        push_identity();
        push_translate(x, 0.0, z);

        // Different color for each sphere
        let colors = [0x44FF88FF, 0x4488FFFF, 0xFF44AAFF, 0xFFAA44FF];
        set_color(colors[i as usize]);
        draw_mesh(SPHERE_MESH);
    }
}

/// Set camera for a player viewport
unsafe fn setup_camera(player: usize) {
    let angle = CAMERA_ANGLES[player];
    let rad = angle.to_radians();
    let distance = 8.0;
    let height = 5.0;

    let cam_x = libm::cosf(rad) * distance;
    let cam_z = libm::sinf(rad) * distance;

    camera_set(cam_x, height, cam_z, 0.0, 0.0, 0.0);
    camera_fov(60.0);
}

/// Draw player label in the viewport
unsafe fn draw_player_label(player: usize) {
    let text = match player {
        0 => "P1",
        1 => "P2",
        2 => "P3",
        3 => "P4",
        _ => "??",
    };
    draw_text(text.as_ptr(), text.len() as u32, 10.0, 10.0, 24.0, 0xFFFFFFFF);
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        if IS_4_PLAYER {
            // 4-player quad split
            let half_w = SCREEN_WIDTH / 2;
            let half_h = SCREEN_HEIGHT / 2;

            // Top-left (P1)
            viewport(0, 0, half_w, half_h);
            setup_camera(0);
            draw_scene();
            draw_player_label(0);

            // Top-right (P2)
            viewport(half_w, 0, half_w, half_h);
            setup_camera(1);
            draw_scene();
            draw_player_label(1);

            // Bottom-left (P3)
            viewport(0, half_h, half_w, half_h);
            setup_camera(2);
            draw_scene();
            draw_player_label(2);

            // Bottom-right (P4)
            viewport(half_w, half_h, half_w, half_h);
            setup_camera(3);
            draw_scene();
            draw_player_label(3);

            // Draw divider lines on fullscreen viewport
            viewport_clear();
            draw_rect(half_w as f32 - 1.0, 0.0, 2.0, SCREEN_HEIGHT as f32, 0x000000FF);
            draw_rect(0.0, half_h as f32 - 1.0, SCREEN_WIDTH as f32, 2.0, 0x000000FF);
        } else {
            // 2-player horizontal split
            let half_w = SCREEN_WIDTH / 2;

            // Left (P1)
            viewport(0, 0, half_w, SCREEN_HEIGHT);
            setup_camera(0);
            draw_scene();
            draw_player_label(0);

            // Right (P2)
            viewport(half_w, 0, half_w, SCREEN_HEIGHT);
            setup_camera(1);
            draw_scene();
            draw_player_label(1);

            // Draw divider line on fullscreen viewport
            viewport_clear();
            draw_rect(half_w as f32 - 1.0, 0.0, 2.0, SCREEN_HEIGHT as f32, 0x000000FF);
        }

        // HUD overlay on fullscreen
        viewport_clear();
        let mode_text = if IS_4_PLAYER { "4-Player Mode (Press A to switch)" } else { "2-Player Mode (Press A to switch)" };
        draw_text(
            mode_text.as_ptr(),
            mode_text.len() as u32,
            SCREEN_WIDTH as f32 / 2.0 - 180.0,
            SCREEN_HEIGHT as f32 - 30.0,
            16.0,
            0xCCCCCCFF,
        );
    }
}
