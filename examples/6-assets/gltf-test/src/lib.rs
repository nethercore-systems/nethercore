//! GLTF Pipeline Test
//!
//! Demonstrates the complete GLTF/GLB asset pipeline:
//! - Textured mesh with UV coordinates (checkerboard pattern)
//! - Vertex colors (red/green/blue per bone segment)
//! - Skinned mesh with bone weights
//! - Skeleton with inverse bind matrices
//! - Keyframe animation playback
//!
//! This validates:
//! - GLTF mesh import (positions, normals, UVs, vertex colors, joints, weights)
//! - GLTF skeleton import (inverse bind matrices)
//! - GLTF animation import (translation, rotation, scale per bone)
//! - Texture binding with UV mapping
//! - Vertex color rendering
//!
//! Build workflow:
//!   1. cargo run -p gen-gltf-test-assets   # Generate assets from GLB
//!   2. nether build                         # Compile Rust to WASM
//!   3. nether pack                          # Bundle assets into data pack
//!   4. nether run                           # Launch in emulator
//!
//! Controls:
//! - A button: Toggle animation pause
//! - D-pad Up/Down: Adjust animation speed
//! - Left stick: Rotate view

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use examples_common::{button, DebugCamera, StickControl};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Imports
// ============================================================================

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;


// ============================================================================
// Game State
// ============================================================================

// Asset handles
static mut TEXTURE: u32 = 0;
static mut MESH: u32 = 0;
static mut SKELETON: u32 = 0;
static mut ANIMATION: u32 = 0;

// Animation state
static mut FRAME_COUNT: u16 = 0;
static mut BONE_COUNT: u8 = 0;
static mut ANIM_TIME: f32 = 0.0;
static mut ANIM_SPEED: f32 = 1.0;
static mut PAUSED: bool = false;

// View rotation (for untextured comparison)
static mut VIEW_ROTATION: f32 = 0.0;

/// Camera for orbit control
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 1.5,
    target_z: 0.0,
    distance: 8.0,
    elevation: 15.0,
    azimuth: 30.0,
    auto_orbit_speed: 0.0,
    stick_control: StickControl::LeftStick,
    fov: 60.0,
};

// ============================================================================
// Game Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue-green background
        set_clear_color(0x1a2a3aFF);
        depth_test(1);

        // Load all assets from ROM data pack
        TEXTURE = rom_texture(b"checker_texture".as_ptr(), 15);
        MESH = rom_mesh(b"test_mesh".as_ptr(), 9);
        SKELETON = rom_skeleton(b"test_skeleton".as_ptr(), 13);
        ANIMATION = rom_keyframes(b"test_anim".as_ptr(), 9);

        // Query animation properties
        FRAME_COUNT = keyframes_frame_count(ANIMATION);
        BONE_COUNT = keyframes_bone_count(ANIMATION);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Update camera
        CAMERA.update();

        // Toggle pause
        if button_pressed(0, button::A) != 0 {
            PAUSED = !PAUSED;
        }

        // Adjust speed with D-pad
        if button_held(0, button::UP) != 0 {
            ANIM_SPEED += 0.02;
            if ANIM_SPEED > 3.0 {
                ANIM_SPEED = 3.0;
            }
        }
        if button_held(0, button::DOWN) != 0 {
            ANIM_SPEED -= 0.02;
            if ANIM_SPEED < 0.1 {
                ANIM_SPEED = 0.1;
            }
        }

        // Advance animation
        if !PAUSED {
            ANIM_TIME += 0.5 * ANIM_SPEED;
        }

        // Slow view rotation for untextured comparison
        VIEW_ROTATION += 0.2;
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        // Calculate current frame
        let frame = (ANIM_TIME as u32) % (FRAME_COUNT as u32);

        // ====================================================================
        // LEFT: Textured animated mesh (demonstrates UV mapping)
        // ====================================================================
        {
            // Bind texture BEFORE drawing
            texture_bind(TEXTURE);

            // Bind skeleton (enables inverse bind mode)
            skeleton_bind(SKELETON);

            // Bind animation keyframe
            keyframe_bind(ANIMATION, frame);

            // Position and draw
            push_identity();
            push_translate(-2.0, 0.0, 0.0);
            set_color(0xFFFFFFFF); // White (texture color passthrough)
            draw_mesh(MESH);
        }

        // ====================================================================
        // RIGHT: Untextured animated mesh (demonstrates skinning)
        // ====================================================================
        {
            // No texture binding (uses vertex colors or solid color)
            texture_bind(0); // Unbind texture

            // Same skeleton and animation
            skeleton_bind(SKELETON);
            keyframe_bind(ANIMATION, frame);

            // Position and draw with a tint color
            push_identity();
            push_translate(2.0, 0.0, 0.0);
            push_rotate_y(VIEW_ROTATION);
            set_color(0xE09060FF); // Orange tint
            draw_mesh(MESH);
        }

        // Draw UI
        draw_ui(frame);
    }
}

fn draw_ui(current_frame: u32) {
    unsafe {
        let y = 10.0;
        let line_h = 18.0;

        // Title
        let title = b"GLTF Pipeline Test";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0);

        // Subtitle
        let subtitle = b"Textured, UV-mapped, animated skinned mesh";
        set_color(0xAAAAAAFF);
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, 10.0, y + line_h, 12.0);

        // Info about left mesh
        let left_info = b"LEFT: Textured (UV mapping)";
        set_color(0x88FF88FF);
        draw_text(left_info.as_ptr(), left_info.len() as u32, 10.0, y + line_h * 2.5, 10.0);

        // Info about right mesh
        let right_info = b"RIGHT: Vertex colors (R/G/B per bone)";
        set_color(0xE09060FF);
        draw_text(right_info.as_ptr(), right_info.len() as u32, 10.0, y + line_h * 3.5, 10.0);

        // Animation stats
        // We can't easily format numbers in no_std, so show static info
        let anim_info = b"Animation: 3 bones, 30 frames";
        set_color(0x8888FFFF);
        draw_text(anim_info.as_ptr(), anim_info.len() as u32, 10.0, y + line_h * 5.0, 10.0);

        // Status
        let status = if PAUSED {
            b"Status: PAUSED" as &[u8]
        } else {
            b"Status: Playing" as &[u8]
        };
        set_color(0x888888FF);
        draw_text(status.as_ptr(), status.len() as u32, 10.0, y + line_h * 6.0, 10.0);

        // Controls
        let ctrl1 = b"Left stick: Rotate view | A: Toggle pause";
        set_color(0x666666FF);
        draw_text(ctrl1.as_ptr(), ctrl1.len() as u32, 10.0, y + line_h * 7.5, 10.0);

        let ctrl2 = b"D-pad Up/Down: Animation speed";
        set_color(0x666666FF);
        draw_text(ctrl2.as_ptr(), ctrl2.len() as u32, 10.0, y + line_h * 8.5, 10.0);

        // Pipeline validation
        let validate = b"Validates: mesh, normals, UVs, colors, skeleton, anim";
        set_color(0x44FF44FF);
        draw_text(validate.as_ptr(), validate.len() as u32, 10.0, y + line_h * 10.0, 10.0);
    }
}
