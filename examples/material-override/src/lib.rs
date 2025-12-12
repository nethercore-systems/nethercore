//! Material Override Demo (Mode 2 PBR)
//!
//! Demonstrates the material override flags feature for switching between
//! texture sampling and uniform values at runtime.
//!
//! Features:
//! - Two textures: albedo (color) and MRE (metallic/roughness/emissive)
//! - UV sphere: samples from textures (shows checkerboard pattern)
//! - Non-UV sphere: uses material_* functions (solid uniform values)
//! - Toggle override flags to make UV sphere look identical to non-UV sphere
//!
//! Usage:
//! 1. Run the game
//! 2. Press F4 to open the debug panel
//! 3. Toggle the override flags to progressively match the non-UV sphere
//! 4. With all overrides enabled, both spheres look identical

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::generate_checkerboard_8x8;

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
    fn camera_set(
        eye_x: f32,
        eye_y: f32,
        eye_z: f32,
        center_x: f32,
        center_y: f32,
        center_z: f32,
    );
    fn camera_fov(fov_degrees: f32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_degrees: f32);
    fn push_scale_uniform(scale: f32);

    // Material uniform values
    fn set_color(rgba: u32);
    fn material_metallic(value: f32);
    fn material_roughness(value: f32);
    fn material_emissive(value: f32);

    // Material override flags
    fn use_uniform_color(enabled: u32);
    fn use_uniform_metallic(enabled: u32);
    fn use_uniform_roughness(enabled: u32);
    fn use_uniform_emissive(enabled: u32);

    // Sky and lighting
    fn light_set(index: u32, x: f32, y: f32, z: f32);
    fn light_color(index: u32, color: u32);
    fn light_intensity(index: u32, intensity: f32);

    // Procedural mesh generation
    fn sphere_uv(radius: f32, segments: u32, rings: u32) -> u32;
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    // Texture loading and binding
    fn load_texture(width: u32, height: u32, data_ptr: *const u8) -> u32;
    fn texture_bind(handle: u32);
    fn texture_bind_slot(handle: u32, slot: u32);

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Text rendering
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);

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

// Override flags for the UV sphere (exposed in debug panel)
static mut USE_UNIFORM_COLOR_FLAG: u8 = 0;
static mut USE_UNIFORM_METALLIC_FLAG: u8 = 0;
static mut USE_UNIFORM_ROUGHNESS_FLAG: u8 = 0;
static mut USE_UNIFORM_EMISSIVE_FLAG: u8 = 0;

// Uniform material values used when overrides are enabled
// These match what the non-UV sphere uses
static mut UNIFORM_COLOR: [u8; 4] = [255, 180, 80, 255]; // Warm orange
static mut UNIFORM_METALLIC: f32 = 0.9;
static mut UNIFORM_ROUGHNESS: f32 = 0.2;
static mut UNIFORM_EMISSIVE: f32 = 0.0;

// Mesh handles
static mut SPHERE_UV_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;

// Texture handles
static mut ALBEDO_TEXTURE: u32 = 0;
static mut MRE_TEXTURE: u32 = 0;

// Texture data buffers (8x8 RGBA = 256 bytes each)
static mut ALBEDO_DATA: [u8; 256] = [0; 256];
static mut MRE_DATA: [u8; 256] = [0; 256];

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

        // Generate sphere meshes
        // UV sphere: has UV coordinates for texture sampling
        SPHERE_UV_MESH = sphere_uv(1.0, 24, 12);
        // Non-UV sphere: uses vertex colors, material values come from uniforms
        SPHERE_MESH = sphere(1.0, 24, 12);

        // Create albedo texture (checkerboard: cyan/magenta)
        let cyan = [0u8, 200, 200, 255];
        let magenta = [200u8, 0, 200, 255];
        generate_checkerboard_8x8(cyan, magenta, &mut ALBEDO_DATA);
        ALBEDO_TEXTURE = load_texture(8, 8, ALBEDO_DATA.as_ptr());

        // Create MRE texture (checkerboard for metallic/roughness/emissive)
        // R channel = metallic, G channel = roughness, B channel = emissive
        // Pattern A: high metallic (255), low roughness (51 = 0.2), no emissive
        // Pattern B: low metallic (0), high roughness (204 = 0.8), no emissive
        let metal_shiny = [255u8, 51, 0, 255]; // M=1.0, R=0.2, E=0.0
        let plastic_matte = [0u8, 204, 0, 255]; // M=0.0, R=0.8, E=0.0
        generate_checkerboard_8x8(metal_shiny, plastic_matte, &mut MRE_DATA);
        MRE_TEXTURE = load_texture(8, 8, MRE_DATA.as_ptr());

        // Register debug values
        register_debug_values();
    }
}

unsafe fn register_debug_values() {
    // Override flags group - toggle these to make UV sphere match non-UV sphere
    debug_group_begin(b"Override Flags".as_ptr(), 14);
    debug_register_bool(b"use_uniform_color".as_ptr(), 17, &USE_UNIFORM_COLOR_FLAG);
    debug_register_bool(
        b"use_uniform_metallic".as_ptr(),
        20,
        &USE_UNIFORM_METALLIC_FLAG,
    );
    debug_register_bool(
        b"use_uniform_roughness".as_ptr(),
        21,
        &USE_UNIFORM_ROUGHNESS_FLAG,
    );
    debug_register_bool(
        b"use_uniform_emissive".as_ptr(),
        20,
        &USE_UNIFORM_EMISSIVE_FLAG,
    );
    debug_group_end();

    // Uniform values group - these are used when overrides are enabled
    debug_group_begin(b"Uniform Values".as_ptr(), 14);
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

        // Left sphere: UV sphere with textures
        // When overrides are disabled: shows checkerboard pattern from textures
        // When overrides are enabled: uses uniform values (looks like right sphere)
        draw_uv_sphere(-1.8, 0.0, 0.0, 1.0);

        // Right sphere: Non-UV sphere with material_* functions
        // Always uses uniform values (solid color, uniform material properties)
        draw_non_uv_sphere(1.8, 0.0, 0.0, 1.0);

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
            b"UV Sphere".as_ptr(),
            9,
            -0.40,
            -0.28,
            label_size,
            label_color,
        );
        draw_text(
            b"(textured)".as_ptr(),
            10,
            -0.40,
            -0.33,
            0.018,
            0x888888FF,
        );

        draw_text(
            b"Non-UV Sphere".as_ptr(),
            13,
            0.13,
            -0.28,
            label_size,
            label_color,
        );
        draw_text(
            b"(material_* funcs)".as_ptr(),
            18,
            0.10,
            -0.33,
            0.018,
            0x888888FF,
        );

        // Status text
        let all_overrides = USE_UNIFORM_COLOR_FLAG != 0
            && USE_UNIFORM_METALLIC_FLAG != 0
            && USE_UNIFORM_ROUGHNESS_FLAG != 0
            && USE_UNIFORM_EMISSIVE_FLAG != 0;

        if all_overrides {
            draw_text(
                b"All overrides ON - spheres match!".as_ptr(),
                33,
                -0.28,
                -0.42,
                0.022,
                0x66FF66FF,
            );
        } else {
            draw_text(
                b"Toggle overrides to match spheres".as_ptr(),
                33,
                -0.28,
                -0.42,
                0.022,
                0xFFCC66FF,
            );
        }
    }
}

// ============================================================================
// Rendering Helpers
// ============================================================================

/// Draw UV sphere with textures (left sphere)
/// Uses texture sampling unless override flags are set
fn draw_uv_sphere(x: f32, y: f32, z: f32, radius: f32) {
    unsafe {
        // Bind textures to slots
        texture_bind(ALBEDO_TEXTURE); // Slot 0: albedo
        texture_bind_slot(MRE_TEXTURE, 1); // Slot 1: MRE (metallic/roughness/emissive)

        // Apply override flags from debug panel
        // When disabled (0): shader samples from textures
        // When enabled (1): shader uses uniform values instead
        use_uniform_color(USE_UNIFORM_COLOR_FLAG as u32);
        use_uniform_metallic(USE_UNIFORM_METALLIC_FLAG as u32);
        use_uniform_roughness(USE_UNIFORM_ROUGHNESS_FLAG as u32);
        use_uniform_emissive(USE_UNIFORM_EMISSIVE_FLAG as u32);

        // Set uniform values (used when corresponding override flag is enabled)
        set_color(color_to_u32(&UNIFORM_COLOR));
        material_metallic(UNIFORM_METALLIC);
        material_roughness(UNIFORM_ROUGHNESS);
        material_emissive(UNIFORM_EMISSIVE);

        // Transform and draw
        push_identity();
        push_translate(x, y, z);
        push_rotate_y(ROTATION);
        push_scale_uniform(radius);
        draw_mesh(SPHERE_UV_MESH);

        // Reset overrides for subsequent draws
        use_uniform_color(0);
        use_uniform_metallic(0);
        use_uniform_roughness(0);
        use_uniform_emissive(0);
    }
}

/// Draw non-UV sphere with material_* functions (right sphere)
/// Always uses uniform values - no textures
fn draw_non_uv_sphere(x: f32, y: f32, z: f32, radius: f32) {
    unsafe {
        // Non-UV sphere doesn't sample textures, so we enable all overrides
        // to force uniform values (this is the "correct" way for non-textured meshes)
        use_uniform_color(1);
        use_uniform_metallic(1);
        use_uniform_roughness(1);
        use_uniform_emissive(1);

        // Set material values - these match the uniform values in the debug panel
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

        // Reset overrides for subsequent draws
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
