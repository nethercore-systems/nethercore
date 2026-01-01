//! Asset Test Example
//!
//! Demonstrates loading assets from nether-export generated files.
//! Uses load_zmesh and load_ztex FFI functions which parse the binary format host-side.
//!
//! Assets were converted using nether-export from:
//! - examples/assets/cube.obj (text format) -> cube.nczxmesh
//! - examples/assets/checkerboard.png -> checkerboard.nczxtex

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// =============================================================================
// Embedded Asset Data (pre-converted with nether-export)
// Uses shared assets from examples/assets/ folder
// =============================================================================

static CUBE_MESH_DATA: &[u8] = include_bytes!("../../../assets/cube.nczxmesh");
static CHECKERBOARD_TEX_DATA: &[u8] = include_bytes!("../../../assets/checkerboard.nczxtex");

// =============================================================================
// FFI Declarations
// =============================================================================

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


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

        // Load mesh from embedded .nczxmesh data (host parses header)
        CUBE_MESH = load_zmesh(
            CUBE_MESH_DATA.as_ptr(),
            CUBE_MESH_DATA.len() as u32,
        );

        // Load texture from embedded .nczxtex data (host parses header)
        CHECKERBOARD_TEX = load_ztex(
            CHECKERBOARD_TEX_DATA.as_ptr(),
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
