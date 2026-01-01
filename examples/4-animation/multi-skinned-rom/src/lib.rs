//! Multi-Skinned ROM Example - ROM-Backed Skeletal Animation
//!
//! Demonstrates Nethercore ZX's ROM asset loading with skeletal animation:
//! - `rom_skeleton()` - loading inverse bind matrices from data pack
//! - `rom_mesh()` - loading skinned mesh from data pack
//! - `rom_keyframes()` - loading animations from data pack
//! - `skeleton_bind()` - enabling inverse bind mode
//! - `keyframe_bind()` - binding pre-decoded GPU keyframes
//!
//! This validates the complete ROM → GPU → render pipeline for skeletal animation.
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

// Character 1: 3-bone vertical arm
static mut ARM1_MESH: u32 = 0;
static mut ARM1_SKELETON: u32 = 0;
static mut ARM1_ANIM: u32 = 0;
static mut ARM1_FRAME_COUNT: u16 = 0;

// Character 2: 4-bone horizontal arm
static mut ARM2_MESH: u32 = 0;
static mut ARM2_SKELETON: u32 = 0;
static mut ARM2_ANIM: u32 = 0;
static mut ARM2_FRAME_COUNT: u16 = 0;

/// Camera for orbit control
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 3.0,
    target_z: 0.0,
    distance: 15.0,
    elevation: 20.0,
    azimuth: 0.0,
    auto_orbit_speed: 0.0,
    stick_control: StickControl::LeftStick,
    fov: 60.0,
};

static mut ANIM_TIME: f32 = 0.0;
static mut ANIM_SPEED: f32 = 1.0;
static mut PAUSED: bool = false;

// ============================================================================
// Game Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1e2a1eFF);
        depth_test(1);

        // Load character 1 assets from ROM
        ARM1_MESH = rom_mesh(b"arm1_mesh".as_ptr(), 9);
        ARM1_SKELETON = rom_skeleton(b"arm1_skel".as_ptr(), 9);
        ARM1_ANIM = rom_keyframes(b"wave1".as_ptr(), 5);
        ARM1_FRAME_COUNT = keyframes_frame_count(ARM1_ANIM) as u16;

        // Load character 2 assets from ROM
        ARM2_MESH = rom_mesh(b"arm2_mesh".as_ptr(), 9);
        ARM2_SKELETON = rom_skeleton(b"arm2_skel".as_ptr(), 9);
        ARM2_ANIM = rom_keyframes(b"wave2".as_ptr(), 5);
        ARM2_FRAME_COUNT = keyframes_frame_count(ARM2_ANIM) as u16;
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

        // Adjust speed
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
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        // Character 1: vertical arm (left side)
        {
            let frame = (ANIM_TIME as u32) % (ARM1_FRAME_COUNT as u32);

            // IMPORTANT: Bind skeleton BEFORE keyframe_bind
            // This enables inverse bind mode (FLAG_SKINNING_MODE)
            skeleton_bind(ARM1_SKELETON);
            keyframe_bind(ARM1_ANIM, frame);

            push_identity();
            push_translate(-3.0, 0.0, 0.0);
            set_color(0xE09060FF);  // Orange
            draw_mesh(ARM1_MESH);
        }

        // Character 2: horizontal arm (right side)
        {
            // Different animation phase for visual variety
            let offset_time = ANIM_TIME + 15.0;
            let frame = (offset_time as u32) % (ARM2_FRAME_COUNT as u32);

            skeleton_bind(ARM2_SKELETON);
            keyframe_bind(ARM2_ANIM, frame);

            push_identity();
            push_translate(1.0, 2.0, 0.0);
            set_color(0x60A0E0FF);  // Blue
            draw_mesh(ARM2_MESH);
        }

        // Draw UI
        draw_ui();
    }
}

fn draw_ui() {
    unsafe {
        let y = 10.0;
        let line_h = 18.0;

        let title = b"Multi-Skinned ROM";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0);

        let subtitle = b"ROM-loaded skeletons with keyframe_bind";
        set_color(0xAAAAAAFF);
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, 10.0, y + line_h, 12.0);

        let arm1_info = b"Arm 1: 3 bones, vertical (orange)";
        set_color(0xE09060FF);
        draw_text(arm1_info.as_ptr(), arm1_info.len() as u32, 10.0, y + line_h * 2.5, 10.0);

        let arm2_info = b"Arm 2: 4 bones, horizontal (blue)";
        set_color(0x60A0E0FF);
        draw_text(arm2_info.as_ptr(), arm2_info.len() as u32, 10.0, y + line_h * 3.5, 10.0);

        let status = if PAUSED {
            b"Status: PAUSED (A)" as &[u8]
        } else {
            b"Status: Playing (A)" as &[u8]
        };
        set_color(0x888888FF);
        draw_text(status.as_ptr(), status.len() as u32, 10.0, y + line_h * 5.0, 10.0);

        // Controls
        let ctrl1 = b"Left stick: Rotate view | A: Toggle pause";
        set_color(0x666666FF);
        draw_text(ctrl1.as_ptr(), ctrl1.len() as u32, 10.0, y + line_h * 6.0, 10.0);

        let ctrl2 = b"D-pad Up/Down: Animation speed";
        set_color(0x666666FF);
        draw_text(ctrl2.as_ptr(), ctrl2.len() as u32, 10.0, y + line_h * 7.0, 10.0);

        let info = b"Uses skeleton_bind + keyframe_bind";
        set_color(0x44FF44FF);
        draw_text(info.as_ptr(), info.len() as u32, 10.0, y + line_h * 8.5, 10.0);
    }
}
