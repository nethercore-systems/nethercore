//! Helper Functions

use super::{
    draw_text, log, rom_data_len, rom_font, rom_keyframes, rom_mesh, rom_skeleton, rom_sound,
    rom_texture, rom_tracker,
};

/// Helper to log a string slice.
///
/// # Example
/// ```rust,ignore
/// log_str("Player spawned");
/// ```
#[inline]
pub fn log_str(s: &str) {
    unsafe {
        log(s.as_ptr(), s.len() as u32);
    }
}

/// Helper to draw a text string slice.
#[inline]
pub fn draw_text_str(s: &str, x: f32, y: f32, size: f32) {
    unsafe {
        draw_text(s.as_ptr(), s.len() as u32, x, y, size);
    }
}

/// Pack RGBA color components into a u32.
///
/// # Example
/// ```rust,ignore
/// let red = rgba(255, 0, 0, 255); // 0xFF0000FF
/// ```
#[inline]
pub const fn rgba(r: u8, g: u8, b: u8, a: u8) -> u32 {
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

/// Pack RGB color components into a u32 (alpha = 255).
#[inline]
pub const fn rgb(r: u8, g: u8, b: u8) -> u32 {
    rgba(r, g, b, 255)
}

/// Helper to load a ROM texture by string literal.
///
/// # Example
/// ```rust,ignore
/// let tex = rom_texture_str("player");
/// ```
#[inline]
pub fn rom_texture_str(id: &str) -> u32 {
    unsafe { rom_texture(id.as_ptr(), id.len() as u32) }
}

/// Helper to load a ROM mesh by string literal.
#[inline]
pub fn rom_mesh_str(id: &str) -> u32 {
    unsafe { rom_mesh(id.as_ptr(), id.len() as u32) }
}

/// Helper to load a ROM sound by string literal.
#[inline]
pub fn rom_sound_str(id: &str) -> u32 {
    unsafe { rom_sound(id.as_ptr(), id.len() as u32) }
}

/// Helper to load a ROM font by string literal.
#[inline]
pub fn rom_font_str(id: &str) -> u32 {
    unsafe { rom_font(id.as_ptr(), id.len() as u32) }
}

/// Helper to load a ROM skeleton by string literal.
#[inline]
pub fn rom_skeleton_str(id: &str) -> u32 {
    unsafe { rom_skeleton(id.as_ptr(), id.len() as u32) }
}

/// Helper to load a ROM tracker by string literal.
#[inline]
pub fn rom_tracker_str(id: &str) -> u32 {
    unsafe { rom_tracker(id.as_ptr(), id.len() as u32) }
}

/// Helper to load ROM keyframes by string literal.
#[inline]
pub fn rom_keyframes_str(id: &str) -> u32 {
    unsafe { rom_keyframes(id.as_ptr(), id.len() as u32) }
}

/// Helper to get ROM data length by string literal.
#[inline]
pub fn rom_data_len_str(id: &str) -> u32 {
    unsafe { rom_data_len(id.as_ptr(), id.len() as u32) }
}

/// Helper to register an f32 debug value by string literal.
///
/// # Example
/// ```rust,ignore
/// static mut SPEED: f32 = 5.0;
/// debug_f32("speed", &SPEED);
/// ```
#[inline]
pub unsafe fn debug_f32(name: &str, ptr: &f32) {
    super::debug_register_f32(
        name.as_ptr(),
        name.len() as u32,
        ptr as *const f32 as *const u8,
    );
}

/// Helper to register an i32 debug value by string literal.
#[inline]
pub unsafe fn debug_i32(name: &str, ptr: &i32) {
    super::debug_register_i32(
        name.as_ptr(),
        name.len() as u32,
        ptr as *const i32 as *const u8,
    );
}

/// Helper to register a bool debug value by string literal.
#[inline]
pub unsafe fn debug_bool(name: &str, ptr: &bool) {
    super::debug_register_bool(
        name.as_ptr(),
        name.len() as u32,
        ptr as *const bool as *const u8,
    );
}

/// Helper to begin a debug group by string literal.
#[inline]
pub fn debug_group(name: &str) {
    unsafe {
        super::debug_group_begin(name.as_ptr(), name.len() as u32);
    }
}

/// Helper to end the current debug group.
#[inline]
pub fn debug_group_close() {
    unsafe {
        super::debug_group_end();
    }
}
