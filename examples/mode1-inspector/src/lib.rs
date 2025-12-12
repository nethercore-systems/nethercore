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

use inspector_common::*;
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

// Matcap texture handles (10 types)
static mut MATCAP_TEXTURES: [u32; 10] = [0; 10];

// Camera
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 0.0,
    target_z: 0.0,
    distance: 5.0,
    elevation: 20.0,
    azimuth: 0.0,
    auto_orbit_speed: 0.0, // No auto-orbit for matcap - user controls
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
/// Types: 0=Basic, 1=Rim, 2=Metal, 3=Toon, 4=Iridescent, 5=Glow
fn generate_matcap(kind: i32) -> [u8; MATCAP_SIZE * MATCAP_SIZE * 4] {
    let mut pixels = [0u8; MATCAP_SIZE * MATCAP_SIZE * 4];

    for y in 0..MATCAP_SIZE {
        for x in 0..MATCAP_SIZE {
            // Normalize to -1..1
            // X: left (-1) to right (+1)
            // Y: top (+1) to bottom (-1) - standard matcap convention (highlights at top)
            let nx = (x as f32 / (MATCAP_SIZE as f32 / 2.0)) - 1.0;
            let ny = 1.0 - (y as f32 / (MATCAP_SIZE as f32 / 2.0));
            let dist_sq = nx * nx + ny * ny;
            let dist = sqrtf(dist_sq);

            // Compute normal Z (assuming hemisphere)
            let nz = if dist <= 1.0 {
                sqrtf(1.0 - dist_sq)
            } else {
                0.0
            };

            let (r, g, b) = match kind {
                0 => matcap_studio_warm(nx, ny, nz, dist),
                1 => matcap_cyan_rim(dist),
                2 => matcap_gold(nx, ny, nz, dist),
                3 => matcap_jade(dist, nz),
                4 => matcap_iridescent(nx, ny),
                5 => matcap_red_wax(dist),
                6 => matcap_chrome(nx, ny, nz, dist),
                7 => matcap_copper(nx, ny, nz, dist),
                8 => matcap_pearl(nx, ny, nz, dist),
                9 => matcap_gray_basic(nx, ny, nz, dist),
                _ => (255, 255, 255),
            };

            let i = (y * MATCAP_SIZE + x) * 4;
            pixels[i] = r;
            pixels[i + 1] = g;
            pixels[i + 2] = b;
            pixels[i + 3] = if dist <= 1.0 { 255 } else { 0 };
        }
    }

    pixels
}

/// Studio warm matcap - orange highlights, blue shadows (classic studio lighting)
fn matcap_studio_warm(nx: f32, ny: f32, nz: f32, dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (60, 50, 80);
    }

    // Light coming from top-right
    let light_x = 0.5;
    let light_y = 0.7;
    let light_z = 0.5;
    let light_len = sqrtf(light_x * light_x + light_y * light_y + light_z * light_z);
    let lx = light_x / light_len;
    let ly = light_y / light_len;
    let lz = light_z / light_len;

    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);

    // Warm (orange) for lit areas, cool (blue) for shadows
    let warm = (255.0, 200.0, 150.0);  // Orange-ish highlight
    let cool = (60.0, 70.0, 120.0);    // Blue-ish shadow

    let r = (cool.0 + (warm.0 - cool.0) * ndotl) as u8;
    let g = (cool.1 + (warm.1 - cool.1) * ndotl) as u8;
    let b = (cool.2 + (warm.2 - cool.2) * ndotl) as u8;

    (r, g, b)
}

/// Cyan rim matcap - dark purple center, cyan edges (sci-fi look)
fn matcap_cyan_rim(dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (0, 0, 0);
    }

    // Fresnel-like effect: edges are bright cyan
    let rim = powf(dist, 2.0);
    let center = (30.0, 20.0, 60.0);   // Dark purple center
    let edge = (100.0, 255.0, 255.0);  // Cyan edge

    let r = (center.0 + (edge.0 - center.0) * rim) as u8;
    let g = (center.1 + (edge.1 - center.1) * rim) as u8;
    let b = (center.2 + (edge.2 - center.2) * rim) as u8;

    (r, g, b)
}

/// Gold metal matcap - warm golden reflections
fn matcap_gold(nx: f32, ny: f32, nz: f32, dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (40, 30, 10);
    }

    // Specular highlight
    let light_x = 0.3;
    let light_y = 0.8;
    let light_z = 0.5;
    let light_len = sqrtf(light_x * light_x + light_y * light_y + light_z * light_z);
    let lx = light_x / light_len;
    let ly = light_y / light_len;
    let lz = light_z / light_len;

    // View direction (0, 0, 1)
    let vz = 1.0;

    // Half vector
    let hx = lx;
    let hy = ly;
    let hz = lz + vz;
    let h_len = sqrtf(hx * hx + hy * hy + hz * hz);
    let hx = hx / h_len;
    let hy = hy / h_len;
    let hz = hz / h_len;

    let ndoth = (nx * hx + ny * hy + nz * hz).max(0.0);
    let spec = powf(ndoth, 64.0);

    // Base metallic gradient
    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);
    let base = 0.15 + 0.5 * ndotl;

    let v = base + spec * 0.8;
    let v = v.min(1.0);

    // Gold tint: high red, medium green, low blue
    let r = (v * 255.0) as u8;
    let g = (v * 200.0) as u8;
    let b = (v * 80.0) as u8;

    (r, g, b)
}

/// Jade matcap - teal/green with stepped bands (stylized jade stone)
fn matcap_jade(dist: f32, nz: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (20, 50, 40);
    }

    // Use normal Z to create bands
    let bands = 4.0;
    let v = floorf(nz * bands) / bands;

    // Jade green tones
    let dark = (30.0, 80.0, 70.0);
    let light = (150.0, 230.0, 200.0);

    let r = (dark.0 + (light.0 - dark.0) * v) as u8;
    let g = (dark.1 + (light.1 - dark.1) * v) as u8;
    let b = (dark.2 + (light.2 - dark.2) * v) as u8;

    (r, g, b)
}

/// Iridescent matcap - rainbow gradient
fn matcap_iridescent(nx: f32, ny: f32) -> (u8, u8, u8) {
    // Map position to hue
    let angle = libm::atan2f(ny, nx);
    let hue = (angle + PI) / (2.0 * PI);

    hsv_to_rgb(hue, 0.7, 0.9)
}

/// Red wax matcap - warm sculpting look (like ZBrush default)
fn matcap_red_wax(dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (40, 10, 10);
    }

    // Inverse distance - bright in center with subsurface-like falloff
    let v = 1.0 - dist;
    let v = powf(v, 1.2);  // Soften the falloff

    // Red wax tones - warm red/orange
    let dark = (80.0, 30.0, 25.0);
    let light = (255.0, 180.0, 150.0);

    let r = (dark.0 + (light.0 - dark.0) * v) as u8;
    let g = (dark.1 + (light.1 - dark.1) * v) as u8;
    let b = (dark.2 + (light.2 - dark.2) * v) as u8;

    (r, g, b)
}

/// Chrome matcap - bright silver/white metal with sharp highlights
fn matcap_chrome(nx: f32, ny: f32, nz: f32, dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (30, 30, 35);
    }

    // Light from top-right
    let light_x = 0.4;
    let light_y = 0.8;
    let light_z = 0.4;
    let light_len = sqrtf(light_x * light_x + light_y * light_y + light_z * light_z);
    let lx = light_x / light_len;
    let ly = light_y / light_len;
    let lz = light_z / light_len;

    // Half vector for specular
    let hz = 1.0 + lz;
    let h_len = sqrtf(lx * lx + ly * ly + hz * hz);
    let hx = lx / h_len;
    let hy = ly / h_len;
    let hz = hz / h_len;

    let ndoth = (nx * hx + ny * hy + nz * hz).max(0.0);
    let spec = powf(ndoth, 128.0);  // Very sharp highlight

    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);
    let base = 0.15 + 0.35 * ndotl;

    let v = base + spec * 0.9;
    let v = (v.min(1.0) * 255.0) as u8;

    // Slight cool tint
    let r = v;
    let g = v;
    let b = ((v as f32) * 1.05).min(255.0) as u8;

    (r, g, b)
}

/// Copper matcap - warm reddish-orange metal
fn matcap_copper(nx: f32, ny: f32, nz: f32, dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (30, 15, 10);
    }

    let light_x = 0.3;
    let light_y = 0.8;
    let light_z = 0.5;
    let light_len = sqrtf(light_x * light_x + light_y * light_y + light_z * light_z);
    let lx = light_x / light_len;
    let ly = light_y / light_len;
    let lz = light_z / light_len;

    let hz = 1.0 + lz;
    let h_len = sqrtf(lx * lx + ly * ly + hz * hz);
    let hx = lx / h_len;
    let hy = ly / h_len;
    let hz = hz / h_len;

    let ndoth = (nx * hx + ny * hy + nz * hz).max(0.0);
    let spec = powf(ndoth, 48.0);

    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);
    let base = 0.2 + 0.5 * ndotl;

    let v = base + spec * 0.7;
    let v = v.min(1.0);

    // Copper tint: high red, medium green, low blue
    let r = (v * 255.0) as u8;
    let g = (v * 160.0) as u8;
    let b = (v * 100.0) as u8;

    (r, g, b)
}

/// Pearl matcap - soft iridescent with pastel tones
fn matcap_pearl(nx: f32, ny: f32, nz: f32, dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (200, 200, 210);
    }

    // Base brightness from normal
    let base = 0.7 + 0.3 * nz;

    // Subtle hue shift based on viewing angle
    let angle = libm::atan2f(ny, nx);
    let hue = (angle + PI) / (2.0 * PI);

    // Very desaturated pastel colors
    let (hr, hg, hb) = hsv_to_rgb(hue, 0.15, 1.0);

    let r = ((hr as f32) * base) as u8;
    let g = ((hg as f32) * base) as u8;
    let b = ((hb as f32) * base) as u8;

    (r, g, b)
}

/// Gray basic matcap - classic grayscale for blending
fn matcap_gray_basic(nx: f32, ny: f32, nz: f32, dist: f32) -> (u8, u8, u8) {
    if dist > 1.0 {
        return (128, 128, 128);
    }

    // Light coming from top-right
    let light_x = 0.5;
    let light_y = 0.7;
    let light_z = 0.5;
    let light_len = sqrtf(light_x * light_x + light_y * light_y + light_z * light_z);
    let lx = light_x / light_len;
    let ly = light_y / light_len;
    let lz = light_z / light_len;

    let ndotl = (nx * lx + ny * ly + nz * lz).max(0.0);

    let ambient = 0.2;
    let diffuse = 0.8;
    let v = ambient + diffuse * ndotl;
    let v = (v * 255.0) as u8;

    (v, v, v)
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

        // Generate all 10 matcap textures
        for i in 0..10 {
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
        MATCAP1_INDEX = MATCAP1_INDEX.clamp(0, 9);
        MATCAP2_INDEX = MATCAP2_INDEX.clamp(0, 9);
        MATCAP3_INDEX = MATCAP3_INDEX.clamp(0, 9);
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
            matcap_set(1, 0);  // Disable slot
        }

        // Slot 2
        if MATCAP2_ENABLED != 0 {
            matcap_set(2, MATCAP_TEXTURES[MATCAP2_INDEX as usize]);
            matcap_blend_mode(2, MATCAP2_BLEND as u32);
        } else {
            matcap_set(2, 0);
        }

        // Slot 3
        if MATCAP3_ENABLED != 0 {
            matcap_set(3, MATCAP_TEXTURES[MATCAP3_INDEX as usize]);
            matcap_blend_mode(3, MATCAP3_BLEND as u32);
        } else {
            matcap_set(3, 0);
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

unsafe fn draw_ui() {
    // Draw header
    draw_text(
        b"MODE 1: MATCAP".as_ptr(),
        14,
        10.0, 10.0, 16.0,
        0xFFFFFFFF,
    );

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
    draw_text(shape_name, shape_len, 10.0, 30.0, 12.0, 0xFFCCCCCC);

    // Matcap type names for reference (matches generate_matcap indices)
    // 0=Studio Warm, 1=Cyan Rim, 2=Gold, 3=Jade, 4=Iridescent, 5=Red Wax
    let _type_names = [
        "Studio", "CyanRim", "Gold", "Jade", "Rainbow", "RedWax"
    ];
    let _blend_names = ["Multiply", "Add", "HSV"];

    // Slot info
    let slot1_info = if MATCAP1_ENABLED != 0 { "Slot1: ON" } else { "Slot1: OFF" };
    draw_text(slot1_info.as_ptr(), slot1_info.len() as u32, 10.0, 50.0, 10.0, 0xFFAAAAFF);

    let slot2_info = if MATCAP2_ENABLED != 0 { "Slot2: ON" } else { "Slot2: OFF" };
    draw_text(slot2_info.as_ptr(), slot2_info.len() as u32, 10.0, 62.0, 10.0, 0xFFAAFFAA);

    let slot3_info = if MATCAP3_ENABLED != 0 { "Slot3: ON" } else { "Slot3: OFF" };
    draw_text(slot3_info.as_ptr(), slot3_info.len() as u32, 10.0, 74.0, 10.0, 0xFFFFAAAA);

    // Controls hint
    draw_text(
        b"A/B: Cycle shape".as_ptr(),
        16,
        10.0, 450.0, 10.0,
        0xFF888888,
    );
    draw_text(
        b"Left stick: Rotate".as_ptr(),
        18,
        10.0, 462.0, 10.0,
        0xFF888888,
    );
    draw_text(
        b"Right stick: Camera".as_ptr(),
        19,
        10.0, 474.0, 10.0,
        0xFF888888,
    );
}
