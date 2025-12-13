//! Multi-Skinned ROM Example - ROM-Backed Skeletal Animation
//!
//! Demonstrates Emberware Z's ROM asset loading with skeletal animation:
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

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Imports
// ============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    fn left_stick_x(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;

    // ROM asset loading
    fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_skeleton(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_keyframes(id_ptr: *const u8, id_len: u32) -> u32;

    // Keyframe queries
    fn keyframes_frame_count(handle: u32) -> u16;

    // Skeleton & animation binding
    fn skeleton_bind(skeleton: u32);
    fn keyframe_bind(handle: u32, frame_index: u32);

    fn draw_mesh(handle: u32);

    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);

    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// ============================================================================
// Constants
// ============================================================================

const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

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

static mut VIEW_ROTATION_Y: f32 = 0.0;
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
        // Camera to see both ROM-loaded animated arms
        camera_set(0.0, 4.0, 15.0, 0.0, 3.0, 0.0);
        camera_fov(50.0);
        depth_test(1);

        // Load character 1 assets from ROM
        ARM1_MESH = rom_mesh(b"arm1_mesh".as_ptr(), 9);
        ARM1_SKELETON = rom_skeleton(b"arm1_skel".as_ptr(), 9);
        ARM1_ANIM = rom_keyframes(b"wave1".as_ptr(), 5);
        ARM1_FRAME_COUNT = keyframes_frame_count(ARM1_ANIM);

        // Load character 2 assets from ROM
        ARM2_MESH = rom_mesh(b"arm2_mesh".as_ptr(), 9);
        ARM2_SKELETON = rom_skeleton(b"arm2_skel".as_ptr(), 9);
        ARM2_ANIM = rom_keyframes(b"wave2".as_ptr(), 5);
        ARM2_FRAME_COUNT = keyframes_frame_count(ARM2_ANIM);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // View rotation
        let stick_x = left_stick_x(0);
        VIEW_ROTATION_Y += stick_x * 2.0;

        // Toggle pause
        if button_pressed(0, BUTTON_A) != 0 {
            PAUSED = !PAUSED;
        }

        // Adjust speed
        if button_held(0, BUTTON_UP) != 0 {
            ANIM_SPEED += 0.02;
            if ANIM_SPEED > 3.0 {
                ANIM_SPEED = 3.0;
            }
        }
        if button_held(0, BUTTON_DOWN) != 0 {
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
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0, 0xFFFFFFFF);

        let subtitle = b"ROM-loaded skeletons with keyframe_bind";
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, 10.0, y + line_h, 12.0, 0xAAAAAAFF);

        let arm1_info = b"Arm 1: 3 bones, vertical (orange)";
        draw_text(arm1_info.as_ptr(), arm1_info.len() as u32, 10.0, y + line_h * 2.5, 10.0, 0xE09060FF);

        let arm2_info = b"Arm 2: 4 bones, horizontal (blue)";
        draw_text(arm2_info.as_ptr(), arm2_info.len() as u32, 10.0, y + line_h * 3.5, 10.0, 0x60A0E0FF);

        let status = if PAUSED {
            b"Status: PAUSED (A)" as &[u8]
        } else {
            b"Status: Playing (A)" as &[u8]
        };
        draw_text(status.as_ptr(), status.len() as u32, 10.0, y + line_h * 5.0, 10.0, 0x888888FF);

        let controls = b"D-pad: Speed, L-Stick: View";
        draw_text(controls.as_ptr(), controls.len() as u32, 10.0, y + line_h * 6.0, 10.0, 0x666666FF);

        let info = b"Uses skeleton_bind + keyframe_bind";
        draw_text(info.as_ptr(), info.len() as u32, 10.0, y + line_h * 7.5, 10.0, 0x44FF44FF);
    }
}
