//! Asset Test Example
//!
//! Demonstrates loading assets from nether-export generated files.
//! Uses load_zmesh and load_ztex FFI functions which parse the binary format host-side.
//!
//! Assets were converted using nether-export from:
//! - assets/cube.obj (text format) -> cube.ewzmesh
//! - assets/checkerboard.png -> checkerboard.ewztex

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// =============================================================================
// Embedded Asset Data (pre-converted with nether-export)
// =============================================================================

static CUBE_MESH_DATA: &[u8] = include_bytes!("../assets/cube.ewzmesh");
static CHECKERBOARD_TEX_DATA: &[u8] = include_bytes!("../assets/checkerboard.ewztex");

// =============================================================================
// FFI Declarations
// =============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // NetherZ format loading (parses header host-side)
    fn load_zmesh(data_ptr: u32, data_len: u32) -> u32;
    fn load_ztex(data_ptr: u32, data_len: u32) -> u32;

    // Configuration
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;

    // Texture
    fn texture_bind(handle: u32);

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Transform
    fn push_identity();
    fn push_rotate_x(angle_deg: f32);
    fn push_rotate_y(angle_deg: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);
}

// =============================================================================
// Game State
// =============================================================================

/// Loaded asset handles
static mut CUBE_MESH: u32 = 0;
static mut CHECKERBOARD_TEX: u32 = 0;

/// Current rotation angles (degrees)
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

// =============================================================================
// Game Entry Points
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background
        set_clear_color(0x1a1a2eFF);

        // Enable depth testing for proper 3D rendering
        depth_test(1);

        // Load mesh from embedded .ewzmesh data (host parses header)
        CUBE_MESH = load_zmesh(
            CUBE_MESH_DATA.as_ptr() as u32,
            CUBE_MESH_DATA.len() as u32,
        );

        // Load texture from embedded .ewztex data (host parses header)
        CHECKERBOARD_TEX = load_ztex(
            CHECKERBOARD_TEX_DATA.as_ptr() as u32,
            CHECKERBOARD_TEX_DATA.len() as u32,
        );
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Read analog stick input for rotation
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        // Rotate based on stick input
        ROTATION_Y += stick_x * 2.0;
        ROTATION_X += stick_y * 2.0;

        // Auto-rotate when stick is centered
        if stick_x.abs() < 0.1 && stick_y.abs() < 0.1 {
            ROTATION_Y += 0.5;
            ROTATION_X += 0.3;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 0.0, 4.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Apply rotation
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);

        // Bind the checkerboard texture
        texture_bind(CHECKERBOARD_TEX);

        // White color (no tint)
        set_color(0xFFFFFFFF);

        // Draw the cube mesh
        draw_mesh(CUBE_MESH);
    }
}
