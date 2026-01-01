//! Mode 1 Inspector - Matcap Rendering
//!
//! Demonstrates the matcap rendering mode with:
//! - 3 matcap texture slots with blend modes
//! - 6 procedural matcap types
//! - Preview planes for each slot
//! - Shape cycling and color control

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

use examples_common::*;
use libm::{sqrtf, powf, fabsf, floorf};

// ============================================================================
// State
// ============================================================================

// Shape
static mut SHAPE_INDEX: i32 = 0;
static mut ROTATION_SPEED: f32 = 0.5;
static mut OBJECT_COLOR: u32 = 0xFFFFFFFF;

// Matcap slot 1
static mut MATCAP1_INDEX: i32 = 0;
static mut MATCAP1_BLEND: i32 = 0;  // 0=Multiply, 1=Add, 2=HSV
static mut MATCAP1_ENABLED: u8 = 1;

// Matcap slot 2
static mut MATCAP2_INDEX: i32 = 1;
static mut MATCAP2_BLEND: i32 = 0;
static mut MATCAP2_ENABLED: u8 = 0;

// Matcap slot 3
static mut MATCAP3_INDEX: i32 = 2;
static mut MATCAP3_BLEND: i32 = 1;
static mut MATCAP3_ENABLED: u8 = 0;

// Texture filtering
static mut FILTER_MODE: i32 = 1;  // 0=nearest, 1=linear

// Mesh handles
static mut SPHERE_MESH: u32 = 0;
static mut CUBE_MESH: u32 = 0;
static mut TORUS_MESH: u32 = 0;

// Matcap texture handles (20 types: 10 base + 10 blending)
static mut MATCAP_TEXTURES: [u32; 20] = [0; 20];

// Camera
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 0.0,
    target_z: 0.0,
    distance: 5.0,
    elevation: 20.0,
    azimuth: 0.0,
    auto_orbit_speed: 0.0, // No auto-orbit for matcap - user controls
    stick_control: StickControl::RightStick,
    fov: 60.0,
};

// Rotation
static mut ROTATION: ShapeRotation = ShapeRotation {
    x: 0.0,
    y: 0.0,
    speed: 30.0,
};

const PI: f32 = 3.14159265;
const MATCAP_SIZE: usize = 64;

// ============================================================================
// Procedural Matcap Generation
// ============================================================================

/// Generate a matcap texture of the given type
/// Types 0-9: Base color matcaps (standalone materials)
/// Types 10-19: Blending matcaps (designed for layering)
fn generate_matcap(kind: i32) -> [u8; MATCAP_SIZE * MATCAP_SIZE * 4] {
    let mut pixels = [0u8; MATCAP_SIZE * MATCAP_SIZE * 4];

    for y in 0..MATCAP_SIZE {
        for x in 0..MATCAP_SIZE {
            // Normalize to -1..1 with proper edge handling
            let nx = (x as f32 / (MATCAP_SIZE - 1) as f32) * 2.0 - 1.0;
            let ny = 1.0 - (y as f32 / (MATCAP_SIZE - 1) as f32) * 2.0;
            let dist_sq = nx * nx + ny * ny;
            let raw_dist = sqrtf(dist_sq);
            let dist = raw_dist.min(1.0);
            let nz = sqrtf(1.0 - dist * dist);

            let (r, g, b) = match kind {
                // === BASE COLOR MATCAPS (0-9) ===
                0 => matcap_studio_warm(nx, ny, nz),
                1 => matcap_studio_cool(nx, ny, nz),
                2 => matcap_clay_gray(nx, ny, nz),
                3 => matcap_red_wax(nz, dist),
                4 => matcap_jade(nz),
                5 => matcap_gold(nx, ny, nz),
                6 => matcap_chrome(nx, ny, nz),
                7 => matcap_copper(nx, ny, nz),
                8 => matcap_pearl(nx, ny, nz),
                9 => matcap_skin(nx, ny, nz, dist),
                // === BLENDING MATCAPS (10-19) ===
                10 => matcap_rim_light(dist),
                11 => matcap_cyan_glow(dist),
                12 => matcap_top_light(ny, nz),
                13 => matcap_specular_dot(nx, ny, nz),
                14 => matcap_rainbow(nx, ny),
                15 => matcap_hue_shift_warm(nx, ny),
                16 => matcap_hue_shift_cool(nx, ny),
                17 => matcap_ambient_occlusion(dist),
                18 => matcap_inner_glow(dist),
                19 => matcap_gradient_bands(nz),
                _ => (128, 128, 128),
            };

            let i = (y * MATCAP_SIZE + x) * 4;
            pixels[i] = r;
            pixels[i + 1] = g;
            pixels[i + 2] = b;
            pixels[i + 3] = if raw_dist <= 1.0 { 255 } else { 0 };
        }
    }

    pixels
}

// ============================================================================
// BASE COLOR MATCAPS (0-9) - Standalone materials
// ============================================================================

/// 0: Studio Warm - orange highlights, blue shadows (classic 3-point lighting)
fn matcap_studio_warm(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.5, 0.7, 0.5);
    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);

    let warm = (255.0, 200.0, 150.0);
    let cool = (60.0, 70.0, 120.0);

    lerp_color(cool, warm, ndotl)
}

/// 1: Studio Cool - blue highlights, warm shadows (inverse of warm)
fn matcap_studio_cool(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.5, 0.7, 0.5);
    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);

    let cool = (150.0, 200.0, 255.0);
    let warm = (120.0, 80.0, 60.0);

    lerp_color(warm, cool, ndotl)
}

/// 2: Clay Gray - neutral sculpting preview (ZBrush-style)
fn matcap_clay_gray(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.4, 0.8, 0.4);
    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);

    // Add subtle specular
    let (hx, hy, hz) = half_vector(lx, ly, lz);
    let spec = powf((nx * hx + ny * hy + nz * hz).max(0.0), 32.0);

    let v = 0.25 + 0.5 * ndotl + 0.25 * spec;
    let v = (v.min(1.0) * 255.0) as u8;
    (v, v, v)
}

/// 3: Red Wax - warm sculpting material with subsurface feel
fn matcap_red_wax(nz: f32, dist: f32) -> (u8, u8, u8) {
    let front = powf(nz, 0.8);
    let rim = powf(dist, 1.5) * 0.3;
    let v = (front + rim).min(1.0);

    let dark = (80.0, 30.0, 25.0);
    let light = (255.0, 160.0, 130.0);

    lerp_color(dark, light, v)
}

/// 4: Jade - stylized green stone with banded look
fn matcap_jade(nz: f32) -> (u8, u8, u8) {
    let bands = floorf(nz * 5.0) / 5.0;
    let smooth = nz * 0.3 + bands * 0.7;

    let dark = (40.0, 90.0, 80.0);
    let light = (140.0, 220.0, 190.0);

    lerp_color(dark, light, smooth)
}

/// 5: Gold - metallic gold with specular highlight
fn matcap_gold(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.3, 0.8, 0.5);
    let (hx, hy, hz) = half_vector(lx, ly, lz);

    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);
    let spec = powf((nx * hx + ny * hy + nz * hz).max(0.0), 64.0);

    let v = (0.15 + 0.5 * ndotl + 0.8 * spec).min(1.0);
    ((v * 255.0) as u8, (v * 200.0) as u8, (v * 80.0) as u8)
}

/// 6: Chrome - bright silver metal with sharp highlight
fn matcap_chrome(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.4, 0.8, 0.4);
    let (hx, hy, hz) = half_vector(lx, ly, lz);

    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);
    let spec = powf((nx * hx + ny * hy + nz * hz).max(0.0), 128.0);

    let v = (0.15 + 0.35 * ndotl + 0.9 * spec).min(1.0);
    let v = (v * 255.0) as u8;
    (v, v, ((v as f32) * 1.02).min(255.0) as u8)
}

/// 7: Copper - warm reddish-orange metal
fn matcap_copper(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.3, 0.8, 0.5);
    let (hx, hy, hz) = half_vector(lx, ly, lz);

    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);
    let spec = powf((nx * hx + ny * hy + nz * hz).max(0.0), 48.0);

    let v = (0.2 + 0.5 * ndotl + 0.7 * spec).min(1.0);
    ((v * 255.0) as u8, (v * 160.0) as u8, (v * 100.0) as u8)
}

/// 8: Pearl - soft iridescent white with pastel tints
fn matcap_pearl(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let base = 0.7 + 0.3 * nz;
    let hue = (libm::atan2f(ny, nx) + PI) / (2.0 * PI);
    let (hr, hg, hb) = hsv_to_rgb(hue, 0.12, 1.0);

    (((hr as f32) * base) as u8, ((hg as f32) * base) as u8, ((hb as f32) * base) as u8)
}

/// 9: Skin - warm skin tone with subsurface scattering feel
fn matcap_skin(nx: f32, ny: f32, nz: f32, dist: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.4, 0.7, 0.6);
    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);

    // Subsurface approximation - red shows through at edges
    let sss = powf(dist, 2.0) * 0.3;

    let shadow = (180.0, 120.0, 100.0);
    let lit = (255.0, 220.0, 200.0);
    let (r, g, b) = lerp_color(shadow, lit, ndotl);

    // Add red at edges for SSS
    ((r as f32 + sss * 40.0).min(255.0) as u8, g, b)
}

// ============================================================================
// BLENDING MATCAPS (10-19) - Designed for layering
// ============================================================================

/// 10: Rim Light - bright white edges, dark center (use with Add)
fn matcap_rim_light(dist: f32) -> (u8, u8, u8) {
    let rim = powf(dist, 2.5);
    let v = (rim * 255.0) as u8;
    (v, v, v)
}

/// 11: Cyan Glow - sci-fi edge glow (use with Add)
fn matcap_cyan_glow(dist: f32) -> (u8, u8, u8) {
    let rim = powf(dist, 2.0);
    ((rim * 50.0) as u8, (rim * 200.0) as u8, (rim * 255.0) as u8)
}

/// 12: Top Light - directional gradient from above (use with Multiply)
fn matcap_top_light(ny: f32, nz: f32) -> (u8, u8, u8) {
    // Brighter where normal points up
    let v = (ny * 0.3 + nz * 0.3 + 0.5).clamp(0.0, 1.0);
    let v = (v * 255.0) as u8;
    (v, v, v)
}

/// 13: Specular Dot - sharp highlight point (use with Add)
fn matcap_specular_dot(nx: f32, ny: f32, nz: f32) -> (u8, u8, u8) {
    let (lx, ly, lz) = normalize_light(0.3, 0.7, 0.6);
    let (hx, hy, hz) = half_vector(lx, ly, lz);
    let spec = powf((nx * hx + ny * hy + nz * hz).max(0.0), 256.0);
    let v = (spec * 255.0) as u8;
    (v, v, v)
}

/// 14: Rainbow - full hue rotation (use with HSV for iridescence)
fn matcap_rainbow(nx: f32, ny: f32) -> (u8, u8, u8) {
    let hue = (libm::atan2f(ny, nx) + PI) / (2.0 * PI);
    hsv_to_rgb(hue, 0.8, 0.9)
}

/// 15: Hue Shift Warm - shifts colors toward orange/red (use with HSV)
fn matcap_hue_shift_warm(nx: f32, ny: f32) -> (u8, u8, u8) {
    let angle = libm::atan2f(ny, nx);
    let hue = 0.08 + (angle + PI) / (2.0 * PI) * 0.15; // Orange range
    hsv_to_rgb(hue, 0.6, 0.9)
}

/// 16: Hue Shift Cool - shifts colors toward blue/cyan (use with HSV)
fn matcap_hue_shift_cool(nx: f32, ny: f32) -> (u8, u8, u8) {
    let angle = libm::atan2f(ny, nx);
    let hue = 0.5 + (angle + PI) / (2.0 * PI) * 0.15; // Blue range
    hsv_to_rgb(hue, 0.6, 0.9)
}

/// 17: Ambient Occlusion - darkens edges (use with Multiply)
fn matcap_ambient_occlusion(dist: f32) -> (u8, u8, u8) {
    let ao = 1.0 - powf(dist, 1.5) * 0.5;
    let v = (ao * 255.0) as u8;
    (v, v, v)
}

/// 18: Inner Glow - bright center, fades to edges (use with Add)
fn matcap_inner_glow(dist: f32) -> (u8, u8, u8) {
    let glow = 1.0 - powf(dist, 1.2);
    let v = (glow * 200.0) as u8;
    (v, v, v)
}

/// 19: Gradient Bands - toon-style stepped shading (use with Multiply)
fn matcap_gradient_bands(nz: f32) -> (u8, u8, u8) {
    let bands = floorf(nz * 4.0) / 4.0;
    let v = (0.4 + bands * 0.6).min(1.0);
    let v = (v * 255.0) as u8;
    (v, v, v)
}

// ============================================================================
// Helper Functions
// ============================================================================

/// Normalize a light direction vector
fn normalize_light(x: f32, y: f32, z: f32) -> (f32, f32, f32) {
    let len = sqrtf(x * x + y * y + z * z);
    (x / len, y / len, z / len)
}

/// Compute half vector for Blinn-Phong specular (view is always 0,0,1)
fn half_vector(lx: f32, ly: f32, lz: f32) -> (f32, f32, f32) {
    let hz = lz + 1.0;
    let len = sqrtf(lx * lx + ly * ly + hz * hz);
    (lx / len, ly / len, hz / len)
}

/// Linear interpolate between two RGB colors
fn lerp_color(a: (f32, f32, f32), b: (f32, f32, f32), t: f32) -> (u8, u8, u8) {
    let r = (a.0 + (b.0 - a.0) * t) as u8;
    let g = (a.1 + (b.1 - a.1) * t) as u8;
    let blu = (a.2 + (b.2 - a.2) * t) as u8;
    (r, g, blu)
}

/// Convert HSV to RGB (h, s, v all in 0..1)
fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let c = v * s;
    let h_prime = h * 6.0;
    let x = c * (1.0 - fabsf(h_prime % 2.0 - 1.0));

    let (r1, g1, b1) = if h_prime < 1.0 {
        (c, x, 0.0)
    } else if h_prime < 2.0 {
        (x, c, 0.0)
    } else if h_prime < 3.0 {
        (0.0, c, x)
    } else if h_prime < 4.0 {
        (0.0, x, c)
    } else if h_prime < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let m = v - c;
    let r = ((r1 + m) * 255.0) as u8;
    let g = ((g1 + m) * 255.0) as u8;
    let b = ((b1 + m) * 255.0) as u8;

    (r, g, b)
}

// ============================================================================
// Debug Registration
// ============================================================================

unsafe fn register_debug_values() {
    // Shape group
    debug_group_begin(b"shape".as_ptr(), 5);
    debug_register_i32(b"shape".as_ptr(), 5, &SHAPE_INDEX);
    debug_register_f32(b"rotation".as_ptr(), 8, &ROTATION_SPEED);
    debug_register_color(b"color".as_ptr(), 5, &OBJECT_COLOR as *const u32 as *const u8);
    debug_group_end();

    // Rendering group
    debug_group_begin(b"rendering".as_ptr(), 9);
    debug_register_i32(b"filter".as_ptr(), 6, &FILTER_MODE);
    debug_group_end();

    // Matcap slot 1
    debug_group_begin(b"slot_1".as_ptr(), 6);
    debug_register_bool(b"enabled".as_ptr(), 7, &MATCAP1_ENABLED);
    debug_register_i32(b"matcap".as_ptr(), 6, &MATCAP1_INDEX);
    debug_register_i32(b"blend".as_ptr(), 5, &MATCAP1_BLEND);
    debug_group_end();

    // Matcap slot 2
    debug_group_begin(b"slot_2".as_ptr(), 6);
    debug_register_bool(b"enabled".as_ptr(), 7, &MATCAP2_ENABLED);
    debug_register_i32(b"matcap".as_ptr(), 6, &MATCAP2_INDEX);
    debug_register_i32(b"blend".as_ptr(), 5, &MATCAP2_BLEND);
    debug_group_end();

    // Matcap slot 3
    debug_group_begin(b"slot_3".as_ptr(), 6);
    debug_register_bool(b"enabled".as_ptr(), 7, &MATCAP3_ENABLED);
    debug_register_i32(b"matcap".as_ptr(), 6, &MATCAP3_INDEX);
    debug_register_i32(b"blend".as_ptr(), 5, &MATCAP3_BLEND);
    debug_group_end();
}

// ============================================================================
// Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Set matcap render mode
        render_mode(1);

        // Enable depth testing
        depth_test(1);

        // Setup camera
        CAMERA.apply();
        camera_fov(60.0);

        // Clear color - dark gray (0xRRGGBBAA format)
        set_clear_color(0x2A2A2AFF);

        // Generate meshes
        SPHERE_MESH = sphere(1.0, 32, 24);
        CUBE_MESH = cube(1.6, 1.6, 1.6);
        TORUS_MESH = torus(0.8, 0.35, 32, 24);

        // Generate all 20 matcap textures (0-9 base, 10-19 blending)
        for i in 0..20 {
            let pixels = generate_matcap(i as i32);
            MATCAP_TEXTURES[i] = load_texture(MATCAP_SIZE as u32, MATCAP_SIZE as u32, pixels.as_ptr());
        }

        // Register debug values
        register_debug_values();
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Handle shape cycling with A/B buttons
        if button_pressed(0, BUTTON_A) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 1) % 3;
        }
        if button_pressed(0, BUTTON_B) != 0 {
            SHAPE_INDEX = (SHAPE_INDEX + 2) % 3;  // -1 mod 3
        }

        // Clamp values
        SHAPE_INDEX = SHAPE_INDEX.clamp(0, 2);
        MATCAP1_INDEX = MATCAP1_INDEX.clamp(0, 19);
        MATCAP2_INDEX = MATCAP2_INDEX.clamp(0, 19);
        MATCAP3_INDEX = MATCAP3_INDEX.clamp(0, 19);
        MATCAP1_BLEND = MATCAP1_BLEND.clamp(0, 2);
        MATCAP2_BLEND = MATCAP2_BLEND.clamp(0, 2);
        MATCAP3_BLEND = MATCAP3_BLEND.clamp(0, 2);
        FILTER_MODE = FILTER_MODE.clamp(0, 1);

        // Update rotation (uses left stick internally)
        ROTATION.speed = ROTATION_SPEED * 60.0;  // Convert to degrees/sec
        ROTATION.update();

        // Update camera (uses right stick internally)
        CAMERA.update();
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        // Apply texture filter mode
        texture_filter(FILTER_MODE as u32);

        // Configure matcap slots
        // Slot 1
        if MATCAP1_ENABLED != 0 {
            matcap_set(1, MATCAP_TEXTURES[MATCAP1_INDEX as usize]);
            matcap_blend_mode(1, MATCAP1_BLEND as u32);
        } else {
            matcap_set(1, 0);
            matcap_blend_mode(1, 0);  // Reset to Multiply so white has no effect
        }

        // Slot 2
        if MATCAP2_ENABLED != 0 {
            matcap_set(2, MATCAP_TEXTURES[MATCAP2_INDEX as usize]);
            matcap_blend_mode(2, MATCAP2_BLEND as u32);
        } else {
            matcap_set(2, 0);
            matcap_blend_mode(2, 0);  // Reset to Multiply so white has no effect
        }

        // Slot 3
        if MATCAP3_ENABLED != 0 {
            matcap_set(3, MATCAP_TEXTURES[MATCAP3_INDEX as usize]);
            matcap_blend_mode(3, MATCAP3_BLEND as u32);
        } else {
            matcap_set(3, 0);
            matcap_blend_mode(3, 0);  // Reset to Multiply so white has no effect
        }

        // Set object color
        set_color(OBJECT_COLOR);

        // Draw main shape
        push_identity();
        push_rotate_y(ROTATION.y);
        push_rotate_x(ROTATION.x);

        let mesh = match SHAPE_INDEX {
            0 => SPHERE_MESH,
            1 => CUBE_MESH,
            _ => TORUS_MESH,
        };
        draw_mesh(mesh);

        // Draw 2D texture previews (shows raw matcap texture)
        draw_preview_quads();

        // Draw UI hints
        draw_ui();
    }
}

unsafe fn draw_preview_quads() {
    // Draw matcap texture previews as screen-space quads on right side
    // This shows the actual texture content, not affected by 3D camera
    // Screen resolution is 960x540

    let preview_size = 48.0;
    let padding = 8.0;
    let base_x = 960.0 - padding - preview_size;  // Right-aligned
    let base_y = 10.0;  // Top of screen

    // Preview for slot 1
    if MATCAP1_ENABLED != 0 {
        texture_bind(MATCAP_TEXTURES[MATCAP1_INDEX as usize]);
        draw_sprite(base_x, base_y, preview_size, preview_size, 0xFFFFFFFF);
    }

    // Preview for slot 2
    if MATCAP2_ENABLED != 0 {
        texture_bind(MATCAP_TEXTURES[MATCAP2_INDEX as usize]);
        draw_sprite(base_x, base_y + preview_size + padding, preview_size, preview_size, 0xFFFFFFFF);
    }

    // Preview for slot 3
    if MATCAP3_ENABLED != 0 {
        texture_bind(MATCAP_TEXTURES[MATCAP3_INDEX as usize]);
        draw_sprite(base_x, base_y + (preview_size + padding) * 2.0, preview_size, preview_size, 0xFFFFFFFF);
    }

    // Unbind texture
    texture_bind(0);
}

/// Get matcap info: (name, role, best_blend)
/// role: "Base" or "Blend"
/// best_blend: recommended blend mode for blending matcaps, empty for base
fn matcap_info(index: i32) -> (&'static str, &'static str, &'static str) {
    match index {
        // Base matcaps (0-9)
        0 => ("StudioWarm", "Base", ""),
        1 => ("StudioCool", "Base", ""),
        2 => ("ClayGray", "Base", ""),
        3 => ("RedWax", "Base", ""),
        4 => ("Jade", "Base", ""),
        5 => ("Gold", "Base", ""),
        6 => ("Chrome", "Base", ""),
        7 => ("Copper", "Base", ""),
        8 => ("Pearl", "Base", ""),
        9 => ("Skin", "Base", ""),
        // Blending matcaps (10-19)
        10 => ("RimLight", "Blend", "Add"),
        11 => ("CyanGlow", "Blend", "Add"),
        12 => ("TopLight", "Blend", "Mul"),
        13 => ("SpecDot", "Blend", "Add"),
        14 => ("Rainbow", "Blend", "HSV"),
        15 => ("HueWarm", "Blend", "HSV"),
        16 => ("HueCool", "Blend", "HSV"),
        17 => ("AO", "Blend", "Mul"),
        18 => ("InnerGlow", "Blend", "Add"),
        19 => ("Bands", "Blend", "Mul"),
        _ => ("???", "???", ""),
    }
}

/// Get blend mode name
fn blend_name(mode: i32) -> &'static str {
    match mode {
        0 => "Mul",
        1 => "Add",
        2 => "HSV",
        _ => "???",
    }
}

unsafe fn draw_ui() {
    // Right side positioning (to the left of preview quads)
    // Preview quads are at x=904, need room for ~420px of text
    let text_x = 470.0;
    let preview_size = 48.0;
    let padding = 8.0;
    let slot_spacing = preview_size + padding;  // Match preview quad spacing

    // Draw header on right side
    set_color(0xFFFFFFFF,
    );
        draw_text(
        b"MODE 1: MATCAP".as_ptr(), 14, text_x, 10.0, 14.0);

    // Shape info
    let shape_name = match SHAPE_INDEX {
        0 => b"Shape: Sphere".as_ptr(),
        1 => b"Shape: Cube".as_ptr(),
        _ => b"Shape: Torus".as_ptr(),
    };
    let shape_len = match SHAPE_INDEX {
        0 => 13,
        1 => 11,
        _ => 12,
    };
    set_color(0xFFCCCCCC);
        draw_text(shape_name, shape_len, text_x, 28.0, 11.0);

    // Slot info - positioned to align with preview quads
    // Slot 1 info (aligned with first preview at y=10)
    let slot1_y = 48.0;
    draw_slot_info(1, MATCAP1_ENABLED != 0, MATCAP1_INDEX, MATCAP1_BLEND, text_x, slot1_y, 0xFFAAAAFF);

    // Slot 2 info (aligned with second preview)
    let slot2_y = slot1_y + slot_spacing;
    draw_slot_info(2, MATCAP2_ENABLED != 0, MATCAP2_INDEX, MATCAP2_BLEND, text_x, slot2_y, 0xFFAAFFAA);

    // Slot 3 info (aligned with third preview)
    let slot3_y = slot2_y + slot_spacing;
    draw_slot_info(3, MATCAP3_ENABLED != 0, MATCAP3_INDEX, MATCAP3_BLEND, text_x, slot3_y, 0xFFFFAAAA);

    // Controls hint at bottom right - comprehensive
    set_color(0xFF888888,
    );
        draw_text(
        b"A/B: Cycle shape | X/Y: Select slot".as_ptr(), 37, text_x, 486.0, 10.0);
    set_color(0xFF888888,
    );
        draw_text(
        b"L-Stick: Rotate | R-Stick: Camera".as_ptr(), 34, text_x, 500.0, 10.0);
    set_color(0xFF888888,
    );
        draw_text(
        b"F4: Debug Inspector (edit blend modes)".as_ptr(), 38, text_x, 514.0, 10.0);
}

unsafe fn draw_slot_info(slot: i32, enabled: bool, matcap_idx: i32, blend_mode: i32, x: f32, y: f32, color: u32) {
    // Slot label with colon (6 chars * ~15px = ~90px)
    let slot_label = match slot {
        1 => b"Slot1:".as_ptr(),
        2 => b"Slot2:".as_ptr(),
        _ => b"Slot3:".as_ptr(),
    };
    set_color(color);
        draw_text(slot_label, 6, x, y, 11.0);

    if !enabled {
        set_color(0xFF666666);
        draw_text(b"OFF".as_ptr(), 3, x + 100.0, y, 11.0);
        return;
    }

    let (name, role, best) = matcap_info(matcap_idx);
    let blend = blend_name(blend_mode);

    // Line 1: Name [Role]
    // Name starts after "Slot1:" (~100px), longest name is "StudioWarm" (10 chars * 15 = 150px)
    set_color(0xFFFFFFFF);
        draw_text(name.as_ptr(), name.len() as u32, x + 100.0, y, 11.0);
    set_color(0xFF666666);
        draw_text(b"[".as_ptr(), 1, x + 260.0, y, 11.0);
    set_color(0xFF888888);
        draw_text(role.as_ptr(), role.len() as u32, x + 275.0, y, 11.0);
    set_color(0xFF666666);
        draw_text(b"]".as_ptr(), 1, x + 355.0, y, 11.0);

    // Line 2: Blend mode and best recommendation
    set_color(0xFF888888);
        draw_text(b"Blend:".as_ptr(), 6, x + 100.0, y + 14.0, 10.0);
    set_color(color);
        draw_text(blend.as_ptr(), blend.len() as u32, x + 195.0, y + 14.0, 10.0);

    if !best.is_empty() {
        set_color(0xFF555555);
        draw_text(b"(best:".as_ptr(), 6, x + 250.0, y + 14.0, 10.0);
        set_color(0xFF777777);
        draw_text(best.as_ptr(), best.len() as u32, x + 345.0, y + 14.0, 10.0);
        set_color(0xFF555555);
        draw_text(b")".as_ptr(), 1, x + 400.0, y + 14.0, 10.0);
    }
}
