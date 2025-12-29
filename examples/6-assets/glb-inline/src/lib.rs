//! GLB Inline Example
//!
//! Demonstrates referencing raw .glb files directly in nether.toml.
//! The build system auto-converts GLB to native formats at pack time.
//!
//! Key features:
//! - Direct .glb file paths (no pre-conversion needed)
//! - Multiple animations extracted from the same GLB using animation_name
//! - Runtime animation switching
//!
//! Controls:
//! - L1/R1 Bumpers: Switch between animations (Wave, Bounce, Twist)
//! - A button: Toggle pause
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

#[link(wasm_import_module = "env")]
extern "C" {
    fn set_clear_color(color: u32);
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;

    // ROM asset loading
    fn rom_texture(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_mesh(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_skeleton(id_ptr: *const u8, id_len: u32) -> u32;
    fn rom_keyframes(id_ptr: *const u8, id_len: u32) -> u32;

    // Keyframe queries
    fn keyframes_frame_count(handle: u32) -> u16;
    fn keyframes_bone_count(handle: u32) -> u8;

    // Texture binding
    fn texture_bind(handle: u32);

    // Skeleton & animation binding
    fn skeleton_bind(skeleton: u32);
    fn keyframe_bind(handle: u32, frame_index: u32);

    fn draw_mesh(handle: u32);

    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_deg: f32);

    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// ============================================================================
// Animation Names (for UI display)
// ============================================================================

const ANIMATION_NAMES: [&[u8]; 3] = [
    b"Wave",
    b"Bounce",
    b"Twist",
];

// ============================================================================
// Game State
// ============================================================================

// Asset handles
static mut TEXTURE: u32 = 0;
static mut MESH: u32 = 0;
static mut SKELETON: u32 = 0;
static mut ANIMATIONS: [u32; 3] = [0; 3];
static mut FRAME_COUNTS: [u16; 3] = [0; 3];

// Animation state
static mut CURRENT_ANIM: usize = 0;
static mut ANIM_TIME: f32 = 0.0;
static mut ANIM_SPEED: f32 = 1.0;
static mut PAUSED: bool = false;

// View rotation
static mut VIEW_ROTATION: f32 = 0.0;

/// Camera for orbit control
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 1.5,
    target_z: 0.0,
    distance: 6.0,
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
        // Dark blue background
        set_clear_color(0x1a2a3aFF);
        depth_test(1);

        // Load assets from ROM data pack
        // These were auto-converted from the raw .glb file at pack time!
        TEXTURE = rom_texture(b"checker".as_ptr(), 7);
        MESH = rom_mesh(b"character".as_ptr(), 9);
        SKELETON = rom_skeleton(b"skeleton".as_ptr(), 8);

        // Load all three animations (extracted from same GLB by animation_name)
        ANIMATIONS[0] = rom_keyframes(b"anim_wave".as_ptr(), 9);
        ANIMATIONS[1] = rom_keyframes(b"anim_bounce".as_ptr(), 11);
        ANIMATIONS[2] = rom_keyframes(b"anim_twist".as_ptr(), 10);

        // Get frame counts for each animation
        for i in 0..3 {
            FRAME_COUNTS[i] = keyframes_frame_count(ANIMATIONS[i]);
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Update camera
        CAMERA.update();

        // Switch animation with L1/R1 bumpers
        if button_pressed(0, button::L1) != 0 {
            if CURRENT_ANIM == 0 {
                CURRENT_ANIM = 2;
            } else {
                CURRENT_ANIM -= 1;
            }
            ANIM_TIME = 0.0; // Reset on switch
        }
        if button_pressed(0, button::R1) != 0 {
            CURRENT_ANIM = (CURRENT_ANIM + 1) % 3;
            ANIM_TIME = 0.0; // Reset on switch
        }

        // Toggle pause with A
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

        // Slow view rotation
        VIEW_ROTATION += 0.15;
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        // Get current animation and frame
        let anim = ANIMATIONS[CURRENT_ANIM];
        let frame_count = FRAME_COUNTS[CURRENT_ANIM] as u32;
        let frame = if frame_count > 0 {
            (ANIM_TIME as u32) % frame_count
        } else {
            0
        };

        // ====================================================================
        // LEFT: Textured animated mesh
        // ====================================================================
        {
            texture_bind(TEXTURE);
            skeleton_bind(SKELETON);
            keyframe_bind(anim, frame);

            push_identity();
            push_translate(-1.5, 0.0, 0.0);
            set_color(0xFFFFFFFF);
            draw_mesh(MESH);
        }

        // ====================================================================
        // RIGHT: Untextured (vertex colors visible)
        // ====================================================================
        {
            texture_bind(0);
            skeleton_bind(SKELETON);
            keyframe_bind(anim, frame);

            push_identity();
            push_translate(1.5, 0.0, 0.0);
            push_rotate_y(VIEW_ROTATION);
            set_color(0xFFFFFFFF);
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
        let title = b"GLB Inline Example";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0, 0xFFFFFFFF);

        // Subtitle
        let subtitle = b"Raw .glb file references - auto-converted at pack time";
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, 10.0, y + line_h, 11.0, 0xAAAAAAFF);

        // Current animation
        let anim_label = b"Animation: ";
        draw_text(anim_label.as_ptr(), anim_label.len() as u32, 10.0, y + line_h * 2.5, 12.0, 0x88FF88FF);

        let anim_name = ANIMATION_NAMES[CURRENT_ANIM];
        draw_text(anim_name.as_ptr(), anim_name.len() as u32, 100.0, y + line_h * 2.5, 12.0, 0xFFFF88FF);

        // Animation info
        let frame_count = FRAME_COUNTS[CURRENT_ANIM];
        let info = b"(L1/R1 to switch, 3 anims from 1 GLB)";
        draw_text(info.as_ptr(), info.len() as u32, 10.0, y + line_h * 3.5, 10.0, 0x8888FFFF);

        // Status
        let status = if PAUSED {
            b"Status: PAUSED" as &[u8]
        } else {
            b"Status: Playing" as &[u8]
        };
        draw_text(status.as_ptr(), status.len() as u32, 10.0, y + line_h * 5.0, 10.0, 0x888888FF);

        // Key feature explanation
        let feature1 = b"KEY: nether.toml references .glb directly";
        draw_text(feature1.as_ptr(), feature1.len() as u32, 10.0, y + line_h * 6.5, 10.0, 0x44FF44FF);

        let feature2 = b"animation_name selects which anim to extract";
        draw_text(feature2.as_ptr(), feature2.len() as u32, 10.0, y + line_h * 7.5, 10.0, 0x44FF44FF);

        // Controls
        let ctrl1 = b"L1/R1: Switch anim | A: Pause | D-pad: Speed";
        draw_text(ctrl1.as_ptr(), ctrl1.len() as u32, 10.0, y + line_h * 9.0, 10.0, 0x666666FF);

        let ctrl2 = b"Left stick: Rotate view";
        draw_text(ctrl2.as_ptr(), ctrl2.len() as u32, 10.0, y + line_h * 10.0, 10.0, 0x666666FF);
    }
}
