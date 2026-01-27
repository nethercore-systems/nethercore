//! GLB Rigid Transform Animation Example
//!
//! Demonstrates rigid transform animation using imported GLTF animation data.
//! Unlike skinned animation (bone weights + skeleton_bind), rigid animation
//! uses keyframe_read() to sample transforms and applies them manually.
//!
//! Key concepts demonstrated:
//! - Multiple .glb files for separate mesh pieces (auto-converted at pack time)
//! - Animation imported from GLB via keyframe_read() (not procedural!)
//! - Transforms applied hierarchically using push_translate/push_rotate
//!
//! The mechanical arm consists of:
//! - Base: rotating platform (Node 0)
//! - Arm: extending segment (Node 1)
//! - Claw: opening/closing gripper (Node 2)
//!
//! Key difference from skinned animation:
//! - Skinned: skeleton_bind() + keyframe_bind() + one deformable mesh
//! - Rigid: keyframe_read() + push_translate/push_rotate + multiple meshes
//!
//! Build workflow:
//!   cargo xtask build-examples   # Runs gen-glb-rigid-assets, then nether build/pack
//!   nether run                   # Launch in emulator
//!
//! Controls:
//! - Left stick: Rotate camera view
//! - A button: Toggle animation pause
//! - D-pad Up/Down: Adjust animation speed

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
#[path = "../../../../include/zx/mod.rs"]
mod ffi;
use ffi::*;


// ============================================================================
// BoneTransform Parsing (from keyframe_read)
// ============================================================================

/// BoneTransform from keyframe_read (40 bytes per bone)
/// Layout: rotation[4] (quat xyzw) + position[3] + scale[3]
#[derive(Clone, Copy)]
struct BoneTransform {
    rotation: [f32; 4], // quaternion [x, y, z, w]
    position: [f32; 3],
    scale: [f32; 3],
}

impl BoneTransform {
    fn from_bytes(bytes: &[u8]) -> Self {
        // Parse 10 f32 values from 40 bytes (little-endian)
        let mut floats = [0.0f32; 10];
        for i in 0..10 {
            let offset = i * 4;
            let arr = [bytes[offset], bytes[offset + 1], bytes[offset + 2], bytes[offset + 3]];
            floats[i] = f32::from_le_bytes(arr);
        }

        Self {
            rotation: [floats[0], floats[1], floats[2], floats[3]],
            position: [floats[4], floats[5], floats[6]],
            scale: [floats[7], floats[8], floats[9]],
        }
    }

    /// Convert quaternion to axis-angle and apply rotation
    /// For simplicity, we'll extract Euler angles (approximate)
    fn apply(&self) {
        unsafe {
            // Apply translation
            push_translate(self.position[0], self.position[1], self.position[2]);

            // Extract Euler angles from quaternion (simplified for Y-dominant rotations)
            let [qx, qy, qz, qw] = self.rotation;

            // Y rotation (yaw) - most common for our animation
            let siny_cosp = 2.0 * (qw * qy + qz * qx);
            let cosy_cosp = 1.0 - 2.0 * (qy * qy + qz * qz);
            let yaw = libm::atan2f(siny_cosp, cosy_cosp);

            // X rotation (pitch)
            let sinp = 2.0 * (qw * qx - qy * qz);
            let pitch = if libm::fabsf(sinp) >= 1.0 {
                libm::copysignf(core::f32::consts::PI / 2.0, sinp)
            } else {
                libm::asinf(sinp)
            };

            // Apply rotations (convert to degrees)
            push_rotate_y(yaw * 180.0 / core::f32::consts::PI);
            push_rotate_x(pitch * 180.0 / core::f32::consts::PI);

            // Note: Scale is ignored for simplicity (all scales are 1.0)
        }
    }
}

// ============================================================================
// Constants
// ============================================================================

const NODE_COUNT: usize = 3;
const BYTES_PER_BONE: usize = 40;

// ============================================================================
// Game State
// ============================================================================

// Mesh handles (loaded from separate GLB files)
static mut MESH_BASE: u32 = 0;
static mut MESH_ARM: u32 = 0;
static mut MESH_CLAW: u32 = 0;

// Skeleton and animation handles
static mut SKELETON: u32 = 0;
static mut ANIMATION: u32 = 0;

// Animation state
static mut FRAME_COUNT: u16 = 0;
static mut BONE_COUNT: u16 = 0;
static mut ANIM_TIME: f32 = 0.0;
static mut ANIM_SPEED: f32 = 1.0;
static mut PAUSED: bool = false;

/// Camera for orbit control
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 1.0,
    target_z: 0.0,
    distance: 5.0,
    elevation: 25.0,
    azimuth: 45.0,
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
        // Dark industrial background
        set_clear_color(0x2a2a2aFF);

        // Load mesh pieces from ROM (each from separate GLB file)
        MESH_BASE = rom_mesh(b"mesh_base".as_ptr(), 9);
        MESH_ARM = rom_mesh(b"mesh_arm".as_ptr(), 8);
        MESH_CLAW = rom_mesh(b"mesh_claw".as_ptr(), 9);

        // Load skeleton (for keyframe access, NOT skinning)
        SKELETON = rom_skeleton(b"rigid_skeleton".as_ptr(), 14);

        // Load animation
        ANIMATION = rom_keyframes(b"anim_operate".as_ptr(), 12);
        FRAME_COUNT = keyframes_frame_count(ANIMATION) as u16;
        BONE_COUNT = keyframes_bone_count(ANIMATION) as u16;
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

        // Advance animation time
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

        // Calculate current frame
        let frame = (ANIM_TIME as u32) % (FRAME_COUNT as u32);

        // ====================================================================
        // Read animation keyframe data
        // ====================================================================
        let mut keyframe_buffer = [0u8; NODE_COUNT * BYTES_PER_BONE]; // 3 nodes * 40 bytes
        keyframe_read(ANIMATION, frame, keyframe_buffer.as_mut_ptr());

        // Parse transforms for each node
        let base_xform = BoneTransform::from_bytes(&keyframe_buffer[0..40]);
        let arm_xform = BoneTransform::from_bytes(&keyframe_buffer[40..80]);
        let claw_xform = BoneTransform::from_bytes(&keyframe_buffer[80..120]);

        // ====================================================================
        // Draw meshes with sampled transforms
        // ====================================================================
        // Unlike skinned animation, we apply transforms manually per mesh.
        // Each mesh gets the accumulated transform of its parent chain.

        // ----------------------------------------------------------------
        // BASE: Uses its own transform only
        // ----------------------------------------------------------------
        push_identity();
        base_xform.apply();
        set_color(0x606060FF); // Dark gray
        draw_mesh(MESH_BASE);

        // ----------------------------------------------------------------
        // ARM: Inherits from base, then applies its own transform
        // ----------------------------------------------------------------
        push_identity();
        base_xform.apply();
        arm_xform.apply();
        set_color(0xE08040FF); // Orange
        draw_mesh(MESH_ARM);

        // ----------------------------------------------------------------
        // CLAW: Inherits from base + arm, then applies its own
        // ----------------------------------------------------------------
        push_identity();
        base_xform.apply();
        arm_xform.apply();
        claw_xform.apply();
        set_color(0x40A0E0FF); // Blue
        draw_mesh(MESH_CLAW);

        // Draw UI
        draw_ui(frame);
    }
}

fn draw_ui(current_frame: u32) {
    unsafe {
        let y = 10.0;
        let line_h = 18.0;

        // Title
        let title = b"GLB Rigid Transform Animation";
        set_color(0xFFFFFFFF,
        );
        draw_text(
            title.as_ptr(), title.len() as u32, 10.0, y, 16.0);

        // Subtitle
        let subtitle = b"Imported animation via keyframe_read()";
        set_color(0xAAAAAAFF,
        );
        draw_text(
            subtitle.as_ptr(), subtitle.len() as u32, 10.0, y + line_h, 12.0);

        // Mesh info
        let mesh_info = b"3 meshes: Base (gray), Arm (orange), Claw (blue)";
        set_color(0x88FF88FF,
        );
        draw_text(
            mesh_info.as_ptr(), mesh_info.len() as u32, 10.0, y + line_h * 2.5, 10.0);

        // Animation type
        let anim_info = b"Animation: keyframe_read() + push_translate/rotate";
        set_color(0x8888FFFF,
        );
        draw_text(
            anim_info.as_ptr(), anim_info.len() as u32, 10.0, y + line_h * 3.5, 10.0);

        // Key difference
        let diff_info = b"Uses keyframe_read() (NOT keyframe_bind!)";
        set_color(0xFFFF88FF,
        );
        draw_text(
            diff_info.as_ptr(), diff_info.len() as u32, 10.0, y + line_h * 4.5, 10.0);

        // Status
        let status = if PAUSED {
            b"Status: PAUSED" as &[u8]
        } else {
            b"Status: Playing" as &[u8]
        };
        set_color(0x888888FF,
        );
        draw_text(
            status.as_ptr(), status.len() as u32, 10.0, y + line_h * 6.0, 10.0);

        // Controls
        let ctrl1 = b"Left stick: Rotate view | A: Toggle pause";
        set_color(0x666666FF,
        );
        draw_text(
            ctrl1.as_ptr(), ctrl1.len() as u32, 10.0, y + line_h * 7.5, 10.0);

        let ctrl2 = b"D-pad Up/Down: Animation speed";
        set_color(0x666666FF,
        );
        draw_text(
            ctrl2.as_ptr(), ctrl2.len() as u32, 10.0, y + line_h * 8.5, 10.0);

        // Use case
        let usecase = b"Use case: Machines, doors, turrets, vehicles";
        set_color(0x44FF44FF,
        );
        draw_text(
            usecase.as_ptr(), usecase.len() as u32, 10.0, y + line_h * 10.0, 10.0);
    }
}
