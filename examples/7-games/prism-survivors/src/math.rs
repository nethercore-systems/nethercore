//! Math helper functions for Prism Survivors

use crate::state::XP_PER_LEVEL;

#[link(wasm_import_module = "env")]
extern "C" {
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// =============================================================================
// Math Helpers
// =============================================================================

pub fn sqrt(x: f32) -> f32 { libm::sqrtf(x) }
pub fn sin(x: f32) -> f32 { libm::sinf(x) }
pub fn cos(x: f32) -> f32 { libm::cosf(x) }
pub fn atan2(y: f32, x: f32) -> f32 { libm::atan2f(y, x) }
pub fn abs(x: f32) -> f32 { if x < 0.0 { -x } else { x } }
pub fn clamp(v: f32, min: f32, max: f32) -> f32 { if v < min { min } else if v > max { max } else { v } }
pub fn dist(x1: f32, y1: f32, x2: f32, y2: f32) -> f32 { let dx = x2-x1; let dy = y2-y1; sqrt(dx*dx+dy*dy) }
pub fn norm(x: f32, y: f32) -> (f32, f32) { let l = sqrt(x*x+y*y); if l > 0.0001 { (x/l, y/l) } else { (0.0, 0.0) } }

pub fn draw_str(s: &[u8], x: f32, y: f32, size: f32, color: u32) {
    unsafe { draw_text(s.as_ptr(), s.len() as u32, x, y, size, color); }
}

pub fn fmt_num(n: u32, buf: &mut [u8]) -> usize {
    if n == 0 { buf[0] = b'0'; return 1; }
    let mut v = n; let mut i = 0;
    while v > 0 && i < buf.len() { buf[i] = b'0' + (v % 10) as u8; v /= 10; i += 1; }
    for j in 0..i/2 { let t = buf[j]; buf[j] = buf[i-1-j]; buf[i-1-j] = t; }
    i
}

// Calculate XP required for a given level (1-indexed)
// Uses table for levels 1-15, exponential scaling after
pub fn xp_for_level(level: u32) -> u32 {
    match level {
        0 => 0, // Invalid level
        1..=15 => XP_PER_LEVEL[(level - 1) as usize],
        _ => {
            // Exponential scaling: base = 850, growth = 1.15x per level
            // Cap iterations to prevent overflow
            let base = 850.0_f32;
            let extra_levels = (level - 15).min(85); // Cap to prevent overflow
            let mult = 1.15_f32;
            // Manual power loop (no powi in no_std)
            let mut result = base;
            for _ in 0..extra_levels {
                result *= mult;
            }
            // Clamp to reasonable max to prevent overflow
            (result as u32).min(1_000_000)
        }
    }
}

// HSV to RGB for rainbow effect (H in degrees, returns RGBA u32)
pub fn hsv_to_rgb(h: f32) -> u32 {
    let h = h % 360.0;
    let s = 1.0f32;
    let v = 1.0f32;
    let c = v * s;
    let x = c * (1.0 - abs((h / 60.0) % 2.0 - 1.0));
    let m = v - c;
    let (r, g, b) = if h < 60.0 { (c, x, 0.0) }
        else if h < 120.0 { (x, c, 0.0) }
        else if h < 180.0 { (0.0, c, x) }
        else if h < 240.0 { (0.0, x, c) }
        else if h < 300.0 { (x, 0.0, c) }
        else { (c, 0.0, x) };
    let r = ((r + m) * 255.0) as u32;
    let g = ((g + m) * 255.0) as u32;
    let b = ((b + m) * 255.0) as u32;
    (r << 24) | (g << 16) | (b << 8) | 0xFF
}
