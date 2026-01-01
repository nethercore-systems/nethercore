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

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


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
        // Set camera every frame (immediate mode)
        camera_set(0.0, 0.0, 1.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Title using built-in font
        let title = b"Font Atlas Demo";
        set_color(0xFFFF00FF);
        draw_text(title.as_ptr(), title.len() as u32, 20.0, 20.0, 32.0);

        // Instructions
        let hint = b"[Stick] Scroll the atlas";
        set_color(0xAAAAAAFF);
        draw_text(hint.as_ptr(), hint.len() as u32, 20.0, 70.0, 20.0);

        let info = b"Texture atlas loaded from ROM";
        set_color(0x88FF88FF);
        draw_text(info.as_ptr(), info.len() as u32, 20.0, 100.0, 18.0);

        // Bind the font atlas texture
        texture_bind(FONT_ATLAS);
        set_color(0xFFFFFFFF);

        // Draw the atlas as a sprite grid to demonstrate how
        // individual glyphs would be drawn with UV coordinates
        let base_y = 180.0 + SCROLL_Y;
        let anim_scale = 1.0 + sinf(TIME) * 0.1;

        // Draw full atlas preview (larger)
        draw_sprite_region(
            30.0,
            base_y,
            200.0 * anim_scale,
            200.0 * anim_scale,
            0.0, 0.0,  // UV top-left
            1.0, 1.0,  // UV bottom-right
            0xFFFFFFFF, // White color
        );

        // Label for full atlas
        let label1 = b"Full Atlas";
        set_color(0xFFFFFFFF);
        draw_text(label1.as_ptr(), label1.len() as u32, 50.0, base_y - 30.0, 18.0);

        // Draw individual "glyphs" (quarters of the texture) - larger and spaced better
        // This demonstrates how you'd render individual characters
        // from a font atlas using UV coordinates

        let glyph_x = 300.0;
        let glyph_size = 90.0;
        let glyph_gap = 100.0;

        // Label for glyph regions
        let label2 = b"UV Regions";
        set_color(0xFFFFFFFF);
        draw_text(label2.as_ptr(), label2.len() as u32, glyph_x + 40.0, base_y - 30.0, 18.0);

        // Top-left quarter
        draw_sprite_region(glyph_x, base_y, glyph_size, glyph_size, 0.0, 0.0, 0.5, 0.5, 0xFFFFFFFF);
        // Top-right quarter
        draw_sprite_region(glyph_x + glyph_gap, base_y, glyph_size, glyph_size, 0.5, 0.0, 1.0, 0.5, 0xFFFFFFFF);
        // Bottom-left quarter
        draw_sprite_region(glyph_x, base_y + glyph_gap, glyph_size, glyph_size, 0.0, 0.5, 0.5, 1.0, 0xFFFFFFFF);
        // Bottom-right quarter
        draw_sprite_region(glyph_x + glyph_gap, base_y + glyph_gap, glyph_size, glyph_size, 0.5, 0.5, 1.0, 1.0, 0xFFFFFFFF);

        // Explanation text at bottom
        let note = b"Shows how sprite regions extract glyphs";
        set_color(0x888888FF);
        draw_text(note.as_ptr(), note.len() as u32, 20.0, 480.0, 16.0);
    }
}
