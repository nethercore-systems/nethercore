//! Debug utilities
//!
//! Helper functions for debug registration and UI drawing.

use crate::ffi::*;
use crate::color;

/// Draw a simple UI header
pub fn draw_header(title: &[u8], y: f32) {
    unsafe {
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 20.0, color::TEXT_WHITE);
    }
}

/// Draw a UI label
pub fn draw_label(text: &[u8], y: f32) {
    unsafe {
        draw_text(text.as_ptr(), text.len() as u32, 10.0, y, 16.0, color::TEXT_GRAY);
    }
}

/// Draw a hint/instruction
pub fn draw_hint(text: &[u8], y: f32) {
    unsafe {
        draw_text(text.as_ptr(), text.len() as u32, 10.0, y, 14.0, color::TEXT_DIM);
    }
}

/// Draw common inspector UI hints
pub fn draw_common_hints(y_start: f32) {
    draw_hint(b"Press A to cycle shapes", y_start);
    draw_hint(b"Left stick to rotate object", y_start + 20.0);
    draw_hint(b"Right stick to orbit camera", y_start + 40.0);
    draw_hint(b"F4 to open Debug Inspector", y_start + 60.0);
}

/// Register a directional light's debug values
pub unsafe fn register_directional_light_debug(
    name: &[u8],
    enabled: *const u8,
    dir_x: *const f32,
    dir_y: *const f32,
    dir_z: *const f32,
    color: *const u8,
    intensity: *const f32,
) {
    debug_group_begin(name.as_ptr(), name.len() as u32);
    debug_register_bool(b"enabled".as_ptr(), 7, enabled);
    debug_register_f32(b"dir_x".as_ptr(), 5, dir_x);
    debug_register_f32(b"dir_y".as_ptr(), 5, dir_y);
    debug_register_f32(b"dir_z".as_ptr(), 5, dir_z);
    debug_register_color(b"color".as_ptr(), 5, color);
    debug_register_f32(b"intensity".as_ptr(), 9, intensity);
    debug_group_end();
}

/// Register a point light's debug values
pub unsafe fn register_point_light_debug(
    name: &[u8],
    enabled: *const u8,
    pos_x: *const f32,
    pos_y: *const f32,
    pos_z: *const f32,
    color: *const u8,
    intensity: *const f32,
    range: *const f32,
) {
    debug_group_begin(name.as_ptr(), name.len() as u32);
    debug_register_bool(b"enabled".as_ptr(), 7, enabled);
    debug_register_f32(b"pos_x".as_ptr(), 5, pos_x);
    debug_register_f32(b"pos_y".as_ptr(), 5, pos_y);
    debug_register_f32(b"pos_z".as_ptr(), 5, pos_z);
    debug_register_color(b"color".as_ptr(), 5, color);
    debug_register_f32(b"intensity".as_ptr(), 9, intensity);
    debug_register_f32(b"range".as_ptr(), 5, range);
    debug_group_end();
}

/// Register Mode 2 (MR) material debug values
pub unsafe fn register_mr_material_debug(
    metallic: *const u8,
    roughness: *const u8,
    emissive: *const f32,
    rim_intensity: *const f32,
    rim_power: *const f32,
) {
    debug_group_begin(b"material".as_ptr(), 8);
    debug_register_u8(b"metallic".as_ptr(), 8, metallic);
    debug_register_u8(b"roughness".as_ptr(), 9, roughness);
    debug_register_f32(b"emissive".as_ptr(), 8, emissive);
    debug_register_f32(b"rim_intensity".as_ptr(), 13, rim_intensity);
    debug_register_f32(b"rim_power".as_ptr(), 9, rim_power);
    debug_group_end();
}

/// Register Mode 3 (SS) material debug values
pub unsafe fn register_ss_material_debug(
    shininess: *const f32,
    specular_color: *const u8,
    emissive: *const f32,
    rim_intensity: *const f32,
    rim_power: *const f32,
) {
    debug_group_begin(b"material".as_ptr(), 8);
    debug_register_f32(b"shininess".as_ptr(), 9, shininess);
    debug_register_color(b"specular_color".as_ptr(), 14, specular_color);
    debug_register_f32(b"emissive".as_ptr(), 8, emissive);
    debug_register_f32(b"rim_intensity".as_ptr(), 13, rim_intensity);
    debug_register_f32(b"rim_power".as_ptr(), 9, rim_power);
    debug_group_end();
}
