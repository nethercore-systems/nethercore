//! Textured Quad Example
//!
//! Demonstrates texture loading and 2D drawing:
//! - `load_texture()` to create a texture from RGBA pixel data
//! - `texture_bind()` to bind the texture for drawing
//! - `draw_sprite()` for basic 2D sprite rendering with color tinting
//!
//! A simple 8x8 checkerboard texture is rendered at screen center,
//! with a color tint that cycles over time.
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn elapsed_time() -> f32;
    fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32;
    fn texture_bind(handle: u32);
    fn draw_sprite(x: f32, y: f32, w: f32, h: f32, color: u32);
    fn texture_filter(filter: u32);
}

/// Texture handle for our checkerboard
static mut TEXTURE: u32 = 0;

/// 8x8 checkerboard pattern (RGBA8)
/// Each pixel is 4 bytes: R, G, B, A
/// Creates a cyan/magenta checkerboard
const CHECKERBOARD: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let cyan = [0x00, 0xFF, 0xFF, 0xFF];
    let magenta = [0xFF, 0x00, 0xFF, 0xFF];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = if (x + y) % 2 == 0 { cyan } else { magenta };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background
        set_clear_color(0x1a1a2eFF);

        // Load the checkerboard texture
        TEXTURE = load_texture(8, 8, CHECKERBOARD.as_ptr());

        // Use nearest-neighbor filtering for crisp pixel art look
        texture_filter(0);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    // No game state to update in this demo
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Bind our texture
        texture_bind(TEXTURE);

        // Create a cycling color tint based on elapsed time
        // Uses simple triangle wave (no trig needed in no_std)
        let t = elapsed_time();

        // Triangle wave: goes 0 -> 1 -> 0 over period
        fn triangle_wave(t: f32, period: f32) -> f32 {
            let phase = (t / period) % 1.0;
            if phase < 0.5 {
                phase * 2.0
            } else {
                2.0 - phase * 2.0
            }
        }

        // Offset each color channel for rainbow effect
        let r = (triangle_wave(t, 3.0) * 255.0) as u8;
        let g = (triangle_wave(t + 1.0, 3.0) * 255.0) as u8;
        let b = (triangle_wave(t + 2.0, 3.0) * 255.0) as u8;

        // Pack into 0xRRGGBBAA format
        let tint = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xFF;

        // Draw the textured quad centered on screen (540p = 960x540)
        // Sprite is 128x128 pixels at screen center
        let sprite_size = 128.0;
        let x = (960.0 - sprite_size) / 2.0;
        let y = (540.0 - sprite_size) / 2.0;

        // Draw with color tint (pass tint directly to draw_sprite)
        draw_sprite(x, y, sprite_size, sprite_size, tint);

        // Draw a few more at corners to show multiple sprites (white, no tint)
        draw_sprite(16.0, 16.0, 64.0, 64.0, 0xFFFFFFFF);
        draw_sprite(960.0 - 80.0, 16.0, 64.0, 64.0, 0xFFFFFFFF);
        draw_sprite(16.0, 540.0 - 80.0, 64.0, 64.0, 0xFFFFFFFF);
        draw_sprite(960.0 - 80.0, 540.0 - 80.0, 64.0, 64.0, 0xFFFFFFFF);
    }
}
