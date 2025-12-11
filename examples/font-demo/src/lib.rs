//! Font Demo
//!
//! Demonstrates loading texture atlases from a ROM data pack.
//! In a full implementation, bitmap font atlases would be loaded this way,
//! with glyph metrics used to render individual characters.
//!
//! This example shows the texture loading mechanism that would be used
//! for custom font rendering.

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libm::sinf;

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
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;

    // Configuration (init-only)
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn left_stick_y(player: u32) -> f32;

    // Built-in text drawing (uses default font)
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, scale: f32, color: u32);

    // Sprite drawing (for custom font rendering)
    fn texture_bind(handle: u32);
    fn draw_sprite_region(x: f32, y: f32, w: f32, h: f32, u0: f32, v0: f32, u1: f32, v1: f32);

    // Render state
    fn set_color(color: u32);
}

// =============================================================================
// Game State
// =============================================================================

// Font atlas texture handle (loaded from data pack)
static mut FONT_ATLAS: u32 = 0;

// Animation state
static mut TIME: f32 = 0.0;
static mut SCROLL_Y: f32 = 0.0;

// =============================================================================
// Game Entry Points
// =============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x0a0a1aFF);

        // Set up 2D camera
        camera_set(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Load font atlas texture from ROM data pack
        FONT_ATLAS = rom_texture(b"font_atlas".as_ptr(), 10);

        // "font_atlas" = 10 chars
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Scroll with stick
        let stick_y = left_stick_y(0);
        SCROLL_Y += stick_y * 2.0;
        if SCROLL_Y < -100.0 {
            SCROLL_Y = -100.0;
        }
        if SCROLL_Y > 100.0 {
            SCROLL_Y = 100.0;
        }

        // Update time for animation
        TIME += 0.02;
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Title using built-in font
        let title = b"Font Atlas Demo";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, 10.0, 2.0, 0xFFFF00FF);

        // Instructions
        let hint = b"[Stick] Scroll";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, 40.0, 1.0, 0xAAAAAAFF);

        let info = b"Texture atlas loaded from ROM data pack";
        draw_text(info.as_ptr(), info.len() as u32, 10.0, 60.0, 1.0, 0x88FF88FF);

        // Bind the font atlas texture
        texture_bind(FONT_ATLAS);
        set_color(0xFFFFFFFF);

        // Draw the atlas as a sprite grid to demonstrate how
        // individual glyphs would be drawn with UV coordinates
        let base_y = 100.0 + SCROLL_Y;
        let scale = 1.0 + sinf(TIME) * 0.1;

        // Draw full atlas preview
        draw_sprite_region(
            50.0,
            base_y,
            128.0 * scale,
            128.0 * scale,
            0.0, 0.0,  // UV top-left
            1.0, 1.0,  // UV bottom-right
        );

        // Draw individual "glyphs" (quarters of the texture)
        // This demonstrates how you'd render individual characters
        // from a font atlas using UV coordinates

        // Top-left quarter
        draw_sprite_region(220.0, base_y, 48.0, 48.0, 0.0, 0.0, 0.5, 0.5);
        // Top-right quarter
        draw_sprite_region(280.0, base_y, 48.0, 48.0, 0.5, 0.0, 1.0, 0.5);
        // Bottom-left quarter
        draw_sprite_region(220.0, base_y + 60.0, 48.0, 48.0, 0.0, 0.5, 0.5, 1.0);
        // Bottom-right quarter
        draw_sprite_region(280.0, base_y + 60.0, 48.0, 48.0, 0.5, 0.5, 1.0, 1.0);

        // Explanation text
        let note1 = b"Left: Full atlas";
        draw_text(note1.as_ptr(), note1.len() as u32, 10.0, 300.0, 1.0, 0xCCCCCCFF);

        let note2 = b"Right: Individual glyph UV regions";
        draw_text(note2.as_ptr(), note2.len() as u32, 10.0, 320.0, 1.0, 0xCCCCCCFF);

        let note3 = b"In practice, use glyph metrics for proper text layout";
        draw_text(note3.as_ptr(), note3.len() as u32, 10.0, 360.0, 1.0, 0x888888FF);
    }
}
