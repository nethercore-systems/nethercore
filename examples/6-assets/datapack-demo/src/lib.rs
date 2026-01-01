//! Data Pack Demo
//!
//! Demonstrates loading assets from a ROM data pack using `rom_*` FFI functions.
//! Assets are bundled via `nether pack` and go directly to VRAM/audio memory,
//! bypassing WASM linear memory for efficient rollback.
//!
//! Build workflow:
//!   1. `nether build` — Compile Rust to WASM
//!   2. `nether pack` — Bundle assets into data pack
//!   3. `nether run` — Launch in emulator

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libm::{fabsf, sinf};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// =============================================================================
// FFI Declarations
// =============================================================================

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


// Button constants
const BUTTON_A: u32 = 4;

// =============================================================================
// Game State
// =============================================================================

// Asset handles (loaded from data pack in init)
static mut CUBE_TEXTURE: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut BEEP_SOUND: u32 = 0;

// Animation state
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;
static mut BOUNCE: f32 = 0.0;

// =============================================================================
// Game Entry Points
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background
        set_clear_color(0x1a2a3aFF);

        // Enable depth testing
        depth_test(1);

        // Load assets from ROM data pack
        // These go directly to VRAM, not WASM memory!
        CUBE_TEXTURE = rom_texture(b"cube_texture".as_ptr(), 12);
        CUBE_MESH = rom_mesh(b"cube_mesh".as_ptr(), 9);
        BEEP_SOUND = rom_sound(b"beep".as_ptr(), 4);

        // Asset ID lengths:
        // "cube_texture" = 12 chars
        // "cube_mesh" = 9 chars
        // "beep" = 4 chars
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Read analog stick for rotation control
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        // Rotate based on input, or auto-rotate when idle
        if fabsf(stick_x) > 0.1 || fabsf(stick_y) > 0.1 {
            ROTATION_Y += stick_x * 3.0;
            ROTATION_X += stick_y * 3.0;
        } else {
            ROTATION_Y += 0.5;
            ROTATION_X += 0.3;
        }

        // Bounce animation
        BOUNCE += 0.05;

        // Play sound on button press
        if button_pressed(0, BUTTON_A) != 0 {
            play_sound(BEEP_SOUND, 0.8, 0.0);
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 0.0, 4.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Calculate bounce offset
        let bounce_y = sinf(BOUNCE) * 0.2;

        // Apply transforms
        push_identity();
        push_translate(0.0, bounce_y, 0.0);
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);

        // Bind texture and draw cube
        texture_bind(CUBE_TEXTURE);
        set_color(0xFFFFFFFF);
        draw_mesh(CUBE_MESH);

        // Draw instructions
        let title = b"Data Pack Demo";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 20.0, 20.0, 32.0);

        let hint = b"[A] Play sound  [Stick] Rotate";
        set_color(0xAAAAAAFF);
        draw_text(hint.as_ptr(), hint.len() as u32, 20.0, 70.0, 20.0);

        let info = b"Assets loaded from ROM data pack";
        set_color(0x88FF88FF);
        draw_text(info.as_ptr(), info.len() as u32, 20.0, 100.0, 18.0);
    }
}
