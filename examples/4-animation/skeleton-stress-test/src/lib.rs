//! Skeleton Stress Test - Many Animated Robots
//!
//! Stress tests the animation system with many independently animated skinned meshes.
//!
//! What it validates:
//! - Performance with many skeleton bindings per frame
//! - GPU buffer management under load
//! - Correct rendering with frequent skeleton switches
//! - Shared skeleton/mesh with different animation phases
//!
//! Controls:
//! - D-pad Up/Down: Adjust animation speed
//! - A button: Toggle animation pause
//! - Left stick: Rotate view

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use libm::{cosf, sinf};
use examples_common::{DebugCamera, StickControl};

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
// Constants
// ============================================================================

const PI: f32 = 3.14159265;
const TWO_PI: f32 = 6.28318530;

const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

// Vertex format flags
const FORMAT_NORMAL: u32 = 0x02;
const FORMAT_SKINNED: u32 = 0x08;
const FORMAT_POS_NORMAL_SKINNED: u32 = FORMAT_NORMAL | FORMAT_SKINNED;

// Floats per vertex: pos(3) + normal(3) + bone_indices(1) + weights(4) = 11
const FLOATS_PER_VERTEX: usize = 11;

// Grid configuration
const GRID_SIZE: usize = 6;
const NUM_ROBOTS: usize = GRID_SIZE * GRID_SIZE;
const SPACING: f32 = 2.5;

// Robot skeleton: 7 bones
// [0] torso (root)
// [1] L_hip  [2] L_knee  [3] L_foot
// [4] R_hip  [5] R_knee  [6] R_foot
const BONE_COUNT: usize = 7;

// Bone positions at bind pose
const TORSO_Y: f32 = 1.4;
const HIP_Y: f32 = 1.0;
const KNEE_Y: f32 = 0.5;
const FOOT_Y: f32 = 0.0;
const LEG_OFFSET_X: f32 = 0.2;

// ============================================================================
// Inverse Bind Matrices
// ============================================================================

// Identity rotation, translation to bring vertices to bone-local space
static INVERSE_BIND: [[f32; 12]; BONE_COUNT] = [
    // Bone 0: torso at (0, 1.4, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, -TORSO_Y, 0.0],
    // Bone 1: L_hip at (-0.2, 1.0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  LEG_OFFSET_X, -HIP_Y, 0.0],
    // Bone 2: L_knee at (-0.2, 0.5, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  LEG_OFFSET_X, -KNEE_Y, 0.0],
    // Bone 3: L_foot at (-0.2, 0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  LEG_OFFSET_X, -FOOT_Y, 0.0],
    // Bone 4: R_hip at (0.2, 1.0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  -LEG_OFFSET_X, -HIP_Y, 0.0],
    // Bone 5: R_knee at (0.2, 0.5, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  -LEG_OFFSET_X, -KNEE_Y, 0.0],
    // Bone 6: R_foot at (0.2, 0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  -LEG_OFFSET_X, -FOOT_Y, 0.0],
];

// ============================================================================
// Mesh Data
// ============================================================================

// Robot mesh: 7 boxes (torso + 6 leg segments)
// Each box = 6 faces × 4 vertices = 24 vertices, 6 faces × 2 triangles × 3 = 36 indices
const VERTICES_PER_BOX: usize = 24;
const INDICES_PER_BOX: usize = 36;
const NUM_BOXES: usize = 7;
const TOTAL_VERTICES: usize = NUM_BOXES * VERTICES_PER_BOX;  // 168
const TOTAL_INDICES: usize = NUM_BOXES * INDICES_PER_BOX;    // 252

// ============================================================================
// Game State
// ============================================================================

static mut ROBOT_MESH: u32 = 0;
static mut ROBOT_SKELETON: u32 = 0;

/// Camera for orbit control
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 1.0,
    target_z: 0.0,
    distance: 18.0,
    elevation: 20.0,
    azimuth: 0.0,
    auto_orbit_speed: 0.0,
    stick_control: StickControl::LeftStick,
    fov: 60.0,
};

static mut ANIM_TIME: f32 = 0.0;
static mut ANIM_SPEED: f32 = 1.0;
static mut PAUSED: bool = false;

// Pre-computed phase offsets for each robot
static mut PHASE_OFFSETS: [f32; NUM_ROBOTS] = [0.0; NUM_ROBOTS];

// Bone matrices buffer (reused each frame)
static mut BONE_MATRICES: [[f32; 12]; BONE_COUNT] = [[0.0; 12]; BONE_COUNT];

// ============================================================================
// Robot Mesh Generation
// ============================================================================

fn generate_robot_mesh() -> ([f32; TOTAL_VERTICES * FLOATS_PER_VERTEX], [u16; TOTAL_INDICES]) {
    let mut vertices = [0.0f32; TOTAL_VERTICES * FLOATS_PER_VERTEX];
    let mut indices = [0u16; TOTAL_INDICES];
    let mut v_offset = 0;
    let mut i_offset = 0;
    let mut base_vertex = 0u16;

    // Torso: box at center, bone 0
    add_box(
        &mut vertices, &mut indices,
        &mut v_offset, &mut i_offset, &mut base_vertex,
        [0.0, TORSO_Y, 0.0], [0.25, 0.35, 0.15], 0
    );

    // Left leg
    add_box(&mut vertices, &mut indices, &mut v_offset, &mut i_offset, &mut base_vertex,
        [-LEG_OFFSET_X, 0.75, 0.0], [0.08, 0.25, 0.08], 1);  // L_hip (thigh)
    add_box(&mut vertices, &mut indices, &mut v_offset, &mut i_offset, &mut base_vertex,
        [-LEG_OFFSET_X, 0.25, 0.0], [0.06, 0.25, 0.06], 2);  // L_knee (shin)
    add_box(&mut vertices, &mut indices, &mut v_offset, &mut i_offset, &mut base_vertex,
        [-LEG_OFFSET_X, 0.04, 0.0], [0.08, 0.04, 0.12], 3);  // L_foot

    // Right leg
    add_box(&mut vertices, &mut indices, &mut v_offset, &mut i_offset, &mut base_vertex,
        [LEG_OFFSET_X, 0.75, 0.0], [0.08, 0.25, 0.08], 4);   // R_hip (thigh)
    add_box(&mut vertices, &mut indices, &mut v_offset, &mut i_offset, &mut base_vertex,
        [LEG_OFFSET_X, 0.25, 0.0], [0.06, 0.25, 0.06], 5);   // R_knee (shin)
    add_box(&mut vertices, &mut indices, &mut v_offset, &mut i_offset, &mut base_vertex,
        [LEG_OFFSET_X, 0.04, 0.0], [0.08, 0.04, 0.12], 6);   // R_foot

    (vertices, indices)
}

fn add_box(
    vertices: &mut [f32], indices: &mut [u16],
    v_offset: &mut usize, i_offset: &mut usize, base_vertex: &mut u16,
    center: [f32; 3], half_size: [f32; 3], bone: u32
) {
    let bone_packed = f32::from_bits(bone | (bone << 8) | (bone << 16) | (bone << 24));

    // 6 faces with normals
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        // Front (+Z)
        ([0.0, 0.0, 1.0], [
            [center[0] - half_size[0], center[1] - half_size[1], center[2] + half_size[2]],
            [center[0] + half_size[0], center[1] - half_size[1], center[2] + half_size[2]],
            [center[0] + half_size[0], center[1] + half_size[1], center[2] + half_size[2]],
            [center[0] - half_size[0], center[1] + half_size[1], center[2] + half_size[2]],
        ]),
        // Back (-Z)
        ([0.0, 0.0, -1.0], [
            [center[0] + half_size[0], center[1] - half_size[1], center[2] - half_size[2]],
            [center[0] - half_size[0], center[1] - half_size[1], center[2] - half_size[2]],
            [center[0] - half_size[0], center[1] + half_size[1], center[2] - half_size[2]],
            [center[0] + half_size[0], center[1] + half_size[1], center[2] - half_size[2]],
        ]),
        // Right (+X)
        ([1.0, 0.0, 0.0], [
            [center[0] + half_size[0], center[1] - half_size[1], center[2] + half_size[2]],
            [center[0] + half_size[0], center[1] - half_size[1], center[2] - half_size[2]],
            [center[0] + half_size[0], center[1] + half_size[1], center[2] - half_size[2]],
            [center[0] + half_size[0], center[1] + half_size[1], center[2] + half_size[2]],
        ]),
        // Left (-X)
        ([-1.0, 0.0, 0.0], [
            [center[0] - half_size[0], center[1] - half_size[1], center[2] - half_size[2]],
            [center[0] - half_size[0], center[1] - half_size[1], center[2] + half_size[2]],
            [center[0] - half_size[0], center[1] + half_size[1], center[2] + half_size[2]],
            [center[0] - half_size[0], center[1] + half_size[1], center[2] - half_size[2]],
        ]),
        // Top (+Y)
        ([0.0, 1.0, 0.0], [
            [center[0] - half_size[0], center[1] + half_size[1], center[2] + half_size[2]],
            [center[0] + half_size[0], center[1] + half_size[1], center[2] + half_size[2]],
            [center[0] + half_size[0], center[1] + half_size[1], center[2] - half_size[2]],
            [center[0] - half_size[0], center[1] + half_size[1], center[2] - half_size[2]],
        ]),
        // Bottom (-Y)
        ([0.0, -1.0, 0.0], [
            [center[0] - half_size[0], center[1] - half_size[1], center[2] - half_size[2]],
            [center[0] + half_size[0], center[1] - half_size[1], center[2] - half_size[2]],
            [center[0] + half_size[0], center[1] - half_size[1], center[2] + half_size[2]],
            [center[0] - half_size[0], center[1] - half_size[1], center[2] + half_size[2]],
        ]),
    ];

    for (normal, corners) in faces.iter() {
        let face_base = *base_vertex;

        for corner in corners {
            // Position (3 floats)
            vertices[*v_offset] = corner[0];
            vertices[*v_offset + 1] = corner[1];
            vertices[*v_offset + 2] = corner[2];
            // Normal (3 floats)
            vertices[*v_offset + 3] = normal[0];
            vertices[*v_offset + 4] = normal[1];
            vertices[*v_offset + 5] = normal[2];
            // Bone indices (1 float, packed u32)
            vertices[*v_offset + 6] = bone_packed;
            // Weights (4 floats)
            vertices[*v_offset + 7] = 1.0;
            vertices[*v_offset + 8] = 0.0;
            vertices[*v_offset + 9] = 0.0;
            vertices[*v_offset + 10] = 0.0;
            *v_offset += FLOATS_PER_VERTEX;
        }

        // Two triangles per face
        indices[*i_offset] = face_base;
        indices[*i_offset + 1] = face_base + 1;
        indices[*i_offset + 2] = face_base + 2;
        indices[*i_offset + 3] = face_base;
        indices[*i_offset + 4] = face_base + 2;
        indices[*i_offset + 5] = face_base + 3;
        *i_offset += 6;
        *base_vertex += 4;
    }
}

// ============================================================================
// Walk Cycle Animation
// ============================================================================

/// Compute walk cycle bone transforms for a given phase (0.0 to 1.0)
fn compute_walk_cycle(phase: f32, bones: &mut [[f32; 12]; BONE_COUNT]) {
    let t = phase * TWO_PI;

    // Torso: slight bob up/down (2x frequency) and sway
    let torso_bob = sinf(t * 2.0) * 0.02;
    let torso_sway = sinf(t) * 0.015;

    // Left leg (phase 0 = left foot forward)
    let l_hip_angle = sinf(t) * 0.35;
    let l_knee_bend = (1.0 - cosf(t)) * 0.25;

    // Right leg (180° out of phase)
    let r_hip_angle = sinf(t + PI) * 0.35;
    let r_knee_bend = (1.0 - cosf(t + PI)) * 0.25;

    // Bone 0: Torso (just translation for bob/sway)
    bones[0] = mat3x4_translate(torso_sway, TORSO_Y + torso_bob, 0.0);

    // Left leg chain
    let l_hip_pos = [-LEG_OFFSET_X, HIP_Y, 0.0];
    bones[1] = mat3x4_rotate_x_world(l_hip_angle, l_hip_pos);

    let l_knee_local_y = KNEE_Y - HIP_Y;
    let l_knee_world = rotate_point_x(
        [l_hip_pos[0], l_hip_pos[1] + l_knee_local_y, l_hip_pos[2]],
        l_hip_pos, l_hip_angle
    );
    bones[2] = mat3x4_rotate_x_world(l_hip_angle + l_knee_bend, l_knee_world);

    let l_foot_local_y = FOOT_Y - KNEE_Y;
    let l_foot_world = rotate_point_x(
        [l_knee_world[0], l_knee_world[1] + l_foot_local_y, l_knee_world[2]],
        l_knee_world, l_hip_angle + l_knee_bend
    );
    // Foot world transform: rotation (inherited from leg chain) + position
    bones[3] = mat3x4_rotate_x_world(l_hip_angle + l_knee_bend, l_foot_world);

    // Right leg chain (mirrored)
    let r_hip_pos = [LEG_OFFSET_X, HIP_Y, 0.0];
    bones[4] = mat3x4_rotate_x_world(r_hip_angle, r_hip_pos);

    let r_knee_local_y = KNEE_Y - HIP_Y;
    let r_knee_world = rotate_point_x(
        [r_hip_pos[0], r_hip_pos[1] + r_knee_local_y, r_hip_pos[2]],
        r_hip_pos, r_hip_angle
    );
    bones[5] = mat3x4_rotate_x_world(r_hip_angle + r_knee_bend, r_knee_world);

    let r_foot_local_y = FOOT_Y - KNEE_Y;
    let r_foot_world = rotate_point_x(
        [r_knee_world[0], r_knee_world[1] + r_foot_local_y, r_knee_world[2]],
        r_knee_world, r_hip_angle + r_knee_bend
    );
    // Foot world transform: rotation (inherited from leg chain) + position
    bones[6] = mat3x4_rotate_x_world(r_hip_angle + r_knee_bend, r_foot_world);
}

// ============================================================================
// Matrix Utilities
// ============================================================================

fn mat3x4_translate(x: f32, y: f32, z: f32) -> [f32; 12] {
    [
        1.0, 0.0, 0.0,  // col 0
        0.0, 1.0, 0.0,  // col 1
        0.0, 0.0, 1.0,  // col 2
        x, y, z,        // col 3
    ]
}

/// Create a bone world transform: rotation around X + translation to world position
/// This is what child bones need to "follow" their parent's rotation
fn mat3x4_rotate_x_world(angle: f32, pos: [f32; 3]) -> [f32; 12] {
    let c = cosf(angle);
    let s = sinf(angle);
    [
        1.0, 0.0, 0.0,  // col 0
        0.0, c, s,      // col 1
        0.0, -s, c,     // col 2
        pos[0], pos[1], pos[2],  // translation to world pos
    ]
}

/// Rotate a point around the X axis at a pivot
fn rotate_point_x(point: [f32; 3], pivot: [f32; 3], angle: f32) -> [f32; 3] {
    let c = cosf(angle);
    let s = sinf(angle);

    let dy = point[1] - pivot[1];
    let dz = point[2] - pivot[2];

    [
        point[0],
        pivot[1] + c * dy - s * dz,
        pivot[2] + s * dy + c * dz,
    ]
}

// ============================================================================
// Game Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x202830FF);
        depth_test(1);

        // Generate robot mesh
        let (vertices, indices) = generate_robot_mesh();

        // Load mesh using unpacked vertex data
        ROBOT_MESH = load_mesh_indexed(
            vertices.as_ptr(),
            TOTAL_VERTICES as u32,
            indices.as_ptr(),
            TOTAL_INDICES as u32,
            FORMAT_POS_NORMAL_SKINNED,
        );

        // Load skeleton with inverse bind matrices
        ROBOT_SKELETON = load_skeleton(
            INVERSE_BIND.as_ptr() as *const f32,
            BONE_COUNT as u32
        );

        // Initialize phase offsets for staggered animation
        for i in 0..NUM_ROBOTS {
            PHASE_OFFSETS[i] = (i as f32 * 0.13) % 1.0;
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Update camera
        CAMERA.update();

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
            ANIM_TIME += 0.008 * ANIM_SPEED;
            if ANIM_TIME > 1000.0 {
                ANIM_TIME -= 1000.0;
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        let grid_offset = (GRID_SIZE as f32 - 1.0) * SPACING * 0.5;

        for row in 0..GRID_SIZE {
            for col in 0..GRID_SIZE {
                let i = row * GRID_SIZE + col;
                let x = col as f32 * SPACING - grid_offset;
                let z = row as f32 * SPACING - grid_offset;

                // Staggered phase for natural look
                let phase = (ANIM_TIME + PHASE_OFFSETS[i]) % 1.0;

                // Compute walk cycle for this robot's phase
                compute_walk_cycle(phase, &mut BONE_MATRICES);

                // Bind skeleton and upload bones
                skeleton_bind(ROBOT_SKELETON);
                set_bones(BONE_MATRICES.as_ptr() as *const f32, BONE_COUNT as u32);

                // Position and draw
                push_identity();
                push_translate(x, 0.0, z);
                set_color(robot_color(i));
                draw_mesh(ROBOT_MESH);
            }
        }

        // Draw UI
        draw_ui();
    }
}

fn robot_color(index: usize) -> u32 {
    let hue = (index as f32 / NUM_ROBOTS as f32) * 360.0;
    hsv_to_rgb(hue, 0.6, 0.9)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> u32 {
    let c = v * s;
    let hp = h / 60.0;
    let x = c * (1.0 - libm::fabsf(hp % 2.0 - 1.0));
    let m = v - c;

    let (r, g, b) = if hp < 1.0 {
        (c, x, 0.0)
    } else if hp < 2.0 {
        (x, c, 0.0)
    } else if hp < 3.0 {
        (0.0, c, x)
    } else if hp < 4.0 {
        (0.0, x, c)
    } else if hp < 5.0 {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    let r = ((r + m) * 255.0) as u32;
    let g = ((g + m) * 255.0) as u32;
    let b = ((b + m) * 255.0) as u32;
    (r << 24) | (g << 16) | (b << 8) | 0xFF
}

fn draw_ui() {
    unsafe {
        let y = 10.0;
        let line_h = 18.0;

        let title = b"Skeleton Stress Test";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0);

        let subtitle = b"36 robots with walk cycle";
        set_color(0xAAAAAAFF);
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, 10.0, y + line_h, 12.0);

        let info = b"Tests skeleton_bind per-draw";
        set_color(0x44FF44FF);
        draw_text(info.as_ptr(), info.len() as u32, 10.0, y + line_h * 2.5, 10.0);

        let status = if PAUSED {
            b"Status: PAUSED (A)" as &[u8]
        } else {
            b"Status: Playing (A)" as &[u8]
        };
        set_color(0x888888FF);
        draw_text(status.as_ptr(), status.len() as u32, 10.0, y + line_h * 4.0, 10.0);

        let controls = b"D-pad: Speed";
        set_color(0x666666FF);
        draw_text(controls.as_ptr(), controls.len() as u32, 10.0, y + line_h * 5.0, 10.0);
    }
}
