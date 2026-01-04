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
use examples_common::checkerboard_8x8;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Declarations
// ============================================================================

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

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
static mut UNIFORM_COLOR: u32 = 0xFFB450FF; // Warm orange
static mut UNIFORM_METALLIC: u8 = 230; // ~0.9 * 255
static mut UNIFORM_ROUGHNESS: u8 = 51;  // ~0.2 * 255
static mut UNIFORM_EMISSIVE: f32 = 0.0;

// Mesh handles
static mut SPHERE_UV_MESH: u32 = 0;
static mut SPHERE_MESH: u32 = 0;

// Texture handles
static mut ALBEDO_TEXTURE: u32 = 0;
static mut MRE_TEXTURE: u32 = 0;

// Texture data (compile-time generated checkerboards, 0xRRGGBBAA format)
// Albedo: cyan/magenta checkerboard
const ALBEDO_DATA: [u8; 256] = checkerboard_8x8(0x00C8C8FF, 0xC800C8FF);
// MRE: metallic/roughness/emissive in R/G/B channels
// Pattern A: high metallic (255), low roughness (51 = 0.2), no emissive (M=1.0, R=0.2, E=0.0)
// Pattern B: low metallic (0), high roughness (204 = 0.8), no emissive (M=0.0, R=0.8, E=0.0)
const MRE_DATA: [u8; 256] = checkerboard_8x8(0xFF3300FF, 0x00CC00FF);

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

        // Load textures (data is compile-time generated)
        ALBEDO_TEXTURE = load_texture(8, 8, ALBEDO_DATA.as_ptr());
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
    debug_register_color(b"color".as_ptr(), 5, &UNIFORM_COLOR as *const u32 as *const u8);
    debug_register_u8(b"metallic".as_ptr(), 8, &UNIFORM_METALLIC);
    debug_register_u8(b"roughness".as_ptr(), 9, &UNIFORM_ROUGHNESS);
    debug_register_f32(b"emissive".as_ptr(), 8, &UNIFORM_EMISSIVE as *const f32 as *const u8);
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

        set_color(title_color,
        );
        draw_text(
            b"Material Override Demo".as_ptr(), 21, -0.25, 0.42, 0.035);
        set_color(0x888888FF,
        );
        draw_text(
            b"Press F4 for debug panel".as_ptr(), 24, -0.2, 0.35, 0.02);

        set_color(label_color,
        );
        draw_text(
            b"UV Sphere".as_ptr(), 9, -0.40, -0.28, label_size);
        set_color(0x888888FF,
        );
        draw_text(
            b"(textured)".as_ptr(), 10, -0.40, -0.33, 0.018);

        set_color(label_color,
        );
        draw_text(
            b"Non-UV Sphere".as_ptr(), 13, 0.13, -0.28, label_size);
        set_color(0x888888FF,
        );
        draw_text(
            b"(material_* funcs)".as_ptr(), 18, 0.10, -0.33, 0.018);

        // Status text
        let all_overrides = USE_UNIFORM_COLOR_FLAG != 0
            && USE_UNIFORM_METALLIC_FLAG != 0
            && USE_UNIFORM_ROUGHNESS_FLAG != 0
            && USE_UNIFORM_EMISSIVE_FLAG != 0;

        if all_overrides {
            set_color(0x66FF66FF,
            );
        draw_text(
                b"All overrides ON - spheres match!".as_ptr(), 33, -0.28, -0.42, 0.022);
        } else {
            set_color(0xFFCC66FF,
            );
        draw_text(
                b"Toggle overrides to match spheres".as_ptr(), 33, -0.28, -0.42, 0.022);
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
        set_color(UNIFORM_COLOR);
        material_metallic(UNIFORM_METALLIC as f32 / 255.0);
        material_roughness(UNIFORM_ROUGHNESS as f32 / 255.0);
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
        set_color(UNIFORM_COLOR);
        material_metallic(UNIFORM_METALLIC as f32 / 255.0);
        material_roughness(UNIFORM_ROUGHNESS as f32 / 255.0);
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