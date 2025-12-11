//! Data Pack Demo
//!
//! Demonstrates loading assets from a ROM data pack using `rom_*` FFI functions.
//! Assets are bundled via `ember pack` and go directly to VRAM/audio memory,
//! bypassing WASM linear memory for efficient rollback.
//!
//! Build workflow:
//!   1. `ember build` — Compile Rust to WASM
//!   2. `ember pack` — Bundle assets into data pack
//!   3. `ember run` — Launch in emulator

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

#[link(wasm_import_module = "env")]
extern "C" {
    // ROM Data Pack Loading (init-only)
    // These load assets directly to VRAM/audio memory, bypassing WASM memory
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_sound(id_ptr: *const u8, id_len: u32) -> u32;

    // Configuration (init-only)
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_x(angle_deg: f32);
    fn push_rotate_y(angle_deg: f32);

    // Texture
    fn texture_bind(handle: u32);

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // Audio
    fn play_sound(sound: u32, volume: f32, pan: f32);

    // Text
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, scale: f32, color: u32);
}

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

        // Set up camera
        camera_set(0.0, 0.0, 4.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

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
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 2.0, 0xFFFFFFFF);

        let hint = b"[A] Play sound  [Stick] Rotate";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 40.0, 1.0, 0xAAAAAAFF);

        let info = b"Assets loaded from ROM data pack";
        draw_text(info.as_ptr(), info.len() as u32, 10.0, 60.0, 1.0, 0x88FF88FF);
    }
}
