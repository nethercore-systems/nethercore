//! Material Override Demo (Mode 2 PBR)
//!
//! Demonstrates the material override flags feature for switching between
//! texture sampling and uniform values at runtime when using UV-mapped meshes.
//!
//! Features:
//! - Side-by-side comparison of textured vs uniform materials
//! - Debug inspection panel to toggle override flags
//! - Visual feedback showing effect of each override
//!
//! Usage:
//! 1. Run the game
//! 2. Press F4 to open the debug panel
//! 3. Toggle the override flags to see the difference
//! 4. The left sphere uses textures, right sphere uses uniform overrides

#![no_std]
#![no_main]

use core::f32::consts::PI;
use core::panic::PanicInfo;
use libm::{cosf, sinf};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Declarations
// ============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn render_mode(mode: u32);
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(eye_x: f32, eye_y: f32, eye_z: f32, center_x: f32, center_y: f32, center_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_degrees: f32);
    fn push_scale_uniform(scale: f32);

    // Material functions - existing
    fn set_color(rgba: u32);
    fn material_metallic(value: f32);
    fn material_roughness(value: f32);
    fn material_emissive(value: f32);

    // Material override flags - NEW
    fn use_uniform_color(enabled: u32);
    fn use_uniform_metallic(enabled: u32);
    fn use_uniform_roughness(enabled: u32);
    fn use_uniform_emissive(enabled: u32);

    // Sky and lighting
    fn light_set(index: u32, x: f32, y: f32, z: f32);
    fn light_color(index: u32, color: u32);
    fn light_intensity(index: u32, intensity: f32);

    // Procedural mesh generation (UV version for texture support)
    fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32;

    // Texture loading and binding
    fn load_texture(width: u32, height: u32, data_ptr: *const u8) -> u32;
    fn texture_bind(handle: u32);

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Text rendering
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

    // Time
    fn elapsed_time() -> f32;

    // Debug inspection
    fn debug_group_begin(name: *const u8, name_len: u32);
    fn debug_group_end();
    fn debug_register_bool(name: *const u8, name_len: u32, ptr: *const u8);
    fn debug_register_f32(name: *const u8, name_len: u32, ptr: *const f32);
    fn debug_register_color(name: *const u8, name_len: u32, ptr: *const u8);
}

// ============================================================================
// State
// ============================================================================

// Override flags (exposed in debug panel)
static mut USE_UNIFORM_COLOR: u8 = 0;
static mut USE_UNIFORM_METALLIC: u8 = 0;
static mut USE_UNIFORM_ROUGHNESS: u8 = 0;
static mut USE_UNIFORM_EMISSIVE: u8 = 0;

// Uniform material values (exposed in debug panel)
static mut UNIFORM_COLOR: [u8; 4] = [255, 128, 64, 255]; // Orange
static mut UNIFORM_METALLIC: f32 = 1.0;
static mut UNIFORM_ROUGHNESS: f32 = 0.3;
static mut UNIFORM_EMISSIVE: f32 = 0.0;

// Mesh handles
static mut SPHERE_MESH: u32 = 0;

// Texture handles
static mut CHECKER_TEXTURE: u32 = 0;

// Checkerboard texture buffer (8x8 RGBA = 256 bytes)
static mut CHECKER_DATA: [u8; 256] = [0; 256];

// Animation
static mut ROTATION: f32 = 0.0;

// ============================================================================
// Public API
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x1A1A2EFF);

        // Set Mode 2 (PBR Metallic-Roughness)
        render_mode(2);

        // Setup lighting
        light_set(0, -0.5, -0.7, -0.5);
        light_color(0, 0xFFFFFFFF);
        light_intensity(0, 1.0);

        light_set(1, 0.5, -0.3, 0.5);
        light_color(1, 0x8080FFFF);
        light_intensity(1, 0.3);

        // Generate sphere mesh with UVs (required for texture override demo)
        SPHERE_MESH = sphere_uv(1.0, 24, 12);

        // Create checkerboard texture (8x8, cyan/magenta pattern)
        create_checkerboard_texture();
        CHECKER_TEXTURE = load_texture(8, 8, CHECKER_DATA.as_ptr());

        // Register debug values
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    // Override flags group
    debug_group_begin(b"override_flags".as_ptr(), 14);
    debug_register_bool(b"use_uniform_color".as_ptr(), 17, &USE_UNIFORM_COLOR);
    debug_register_bool(b"use_uniform_metallic".as_ptr(), 20, &USE_UNIFORM_METALLIC);
    debug_register_bool(
        b"use_uniform_roughness".as_ptr(),
        21,
        &USE_UNIFORM_ROUGHNESS,
    );
    debug_register_bool(b"use_uniform_emissive".as_ptr(), 20, &USE_UNIFORM_EMISSIVE);
    debug_group_end();

    // Uniform values group
    debug_group_begin(b"uniform_values".as_ptr(), 14);
    debug_register_color(b"color".as_ptr(), 5, UNIFORM_COLOR.as_ptr());
    debug_register_f32(b"metallic".as_ptr(), 8, &UNIFORM_METALLIC);
    debug_register_f32(b"roughness".as_ptr(), 9, &UNIFORM_ROUGHNESS);
    debug_register_f32(b"emissive".as_ptr(), 8, &UNIFORM_EMISSIVE);
    debug_group_end();
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Slow rotation
        ROTATION += 0.3;
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Camera setup
        camera_set(0.0, 1.0, 6.0, 0.0, 0.0, 0.0);
        camera_fov(45.0);

        // Draw left sphere - Default (texture) workflow
        // Even without textures bound, this shows the "default" uniform values
        draw_sphere_default(-1.8, 0.0, 0.0, 1.0);

        // Draw right sphere - Using override flags
        draw_sphere_with_overrides(1.8, 0.0, 0.0, 1.0);

        // Draw labels
        let label_size = 0.025;
        let label_color = 0xCCCCCCFF;
        let title_color = 0xFFCC66FF;

        draw_text(
            b"Material Override Demo".as_ptr(),
            21,
            -0.25,
            0.42,
            0.035,
            title_color,
        );
        draw_text(
            b"Press F4 for debug panel".as_ptr(),
            24,
            -0.2,
            0.35,
            0.02,
            0x888888FF,
        );

        draw_text(
            b"Default".as_ptr(),
            7,
            -0.38,
            -0.28,
            label_size,
            label_color,
        );
        draw_text(
            b"(no overrides)".as_ptr(),
            14,
            -0.43,
            -0.33,
            0.018,
            0x888888FF,
        );

        draw_text(
            b"Uniform Overrides".as_ptr(),
            17,
            0.15,
            -0.28,
            label_size,
            label_color,
        );
        draw_text(
            b"(toggle in F4)".as_ptr(),
            14,
            0.16,
            -0.33,
            0.018,
            0x888888FF,
        );
    }
}

// ============================================================================
// Rendering Helpers
// ============================================================================

/// Draw a sphere with default material (no overrides) - shows texture
fn draw_sphere_default(x: f32, y: f32, z: f32, radius: f32) {
    unsafe {
        // Bind checkerboard texture to slot 0 (albedo)
        texture_bind(CHECKER_TEXTURE);

        // Disable all overrides - use texture values
        use_uniform_color(0);
        use_uniform_metallic(0);
        use_uniform_roughness(0);
        use_uniform_emissive(0);

        // Set default material values (multiplied with texture)
        set_color(0xFFFFFFFF); // White (don't tint texture)
        material_metallic(0.0);
        material_roughness(0.5);
        material_emissive(0.0);

        // Transform and draw
        push_identity();
        push_translate(x, y, z);
        push_rotate_y(ROTATION);
        push_scale_uniform(radius);
        draw_mesh(SPHERE_MESH);
    }
}

/// Draw a sphere with uniform overrides based on debug panel settings
fn draw_sphere_with_overrides(x: f32, y: f32, z: f32, radius: f32) {
    unsafe {
        // Bind same checkerboard texture (will be overridden when flag is set)
        texture_bind(CHECKER_TEXTURE);

        // Apply override flags from debug panel
        use_uniform_color(USE_UNIFORM_COLOR as u32);
        use_uniform_metallic(USE_UNIFORM_METALLIC as u32);
        use_uniform_roughness(USE_UNIFORM_ROUGHNESS as u32);
        use_uniform_emissive(USE_UNIFORM_EMISSIVE as u32);

        // Set uniform values (used when override flag is enabled)
        set_color(color_to_u32(&UNIFORM_COLOR));
        material_metallic(UNIFORM_METALLIC);
        material_roughness(UNIFORM_ROUGHNESS);
        material_emissive(UNIFORM_EMISSIVE);

        // Transform and draw
        push_identity();
        push_translate(x, y, z);
        push_rotate_y(ROTATION);
        push_scale_uniform(radius);
        draw_mesh(SPHERE_MESH);

        // Reset overrides for next frame
        use_uniform_color(0);
        use_uniform_metallic(0);
        use_uniform_roughness(0);
        use_uniform_emissive(0);
    }
}

/// Convert RGBA bytes to u32 color (0xRRGGBBAA format)
fn color_to_u32(rgba: &[u8; 4]) -> u32 {
    ((rgba[0] as u32) << 24) | ((rgba[1] as u32) << 16) | ((rgba[2] as u32) << 8) | (rgba[3] as u32)
}

/// Create a checkerboard texture pattern in CHECKER_DATA buffer
unsafe fn create_checkerboard_texture() {
    // 8x8 checkerboard with cyan and magenta squares
    let cyan: [u8; 4] = [0, 200, 200, 255];
    let magenta: [u8; 4] = [200, 0, 200, 255];

    for y in 0..8 {
        for x in 0..8 {
            let idx = (y * 8 + x) * 4;
            let is_light = ((x + y) % 2) == 0;
            let color = if is_light { &cyan } else { &magenta };
            CHECKER_DATA[idx] = color[0];
            CHECKER_DATA[idx + 1] = color[1];
            CHECKER_DATA[idx + 2] = color[2];
            CHECKER_DATA[idx + 3] = color[3];
        }
    }
}
