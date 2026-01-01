//! Multi-Skinned Procedural Example - Inverse Bind Matrix Validation
//!
//! Validates Nethercore ZX's inverse bind matrix logic:
//! - `load_skeleton()` - uploading inverse bind matrices
//! - `skeleton_bind()` - switching between skeletons
//! - Multiple meshes with independent animations
//! - Proper GPU skinning with inverse bind multiplication
//!
//! This example tests the GPU path: `final_pos = bone_matrix * inverse_bind * vertex`
//! which is different from raw mode where bone matrices are applied directly.
//!
//! Controls:
//! - Left stick: Rotate view
//! - A button: Toggle animation pause
//! - D-pad Up/Down: Adjust animation speed

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
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
// Constants
// ============================================================================

/// Vertex format: POS_NORMAL_SKINNED
const FORMAT_NORMAL: u32 = 4;
const FORMAT_SKINNED: u32 = 8;
const FORMAT_POS_NORMAL_SKINNED: u32 = FORMAT_NORMAL | FORMAT_SKINNED;

/// Floats per vertex: pos(3) + normal(3) + bone_indices(1) + weights(4) = 11
const FLOATS_PER_VERTEX: usize = 11;

/// Floats per 3x4 bone matrix (column-major)
const BONE_MATRIX_FLOATS: usize = 12;

// ============================================================================
// Arm 1: 3-bone vertical arm (same as existing skinned-mesh but with inverse bind)
// ============================================================================

const ARM1_BONES: usize = 3;
const ARM1_SEGMENT_LENGTH: f32 = 1.5;

/// Inverse bind matrices for Arm 1
/// Bone positions at bind pose:
/// - Bone 0: y = 0 (origin)
/// - Bone 1: y = 1.5 (first joint)
/// - Bone 2: y = 3.0 (second joint)
///
/// Inverse bind = inverse of bone world transform at bind pose
static ARM1_INVERSE_BIND: [[f32; 12]; ARM1_BONES] = [
    // Bone 0: inverse of identity = identity
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, 0.0, 0.0],
    // Bone 1: inverse of translate(0, 1.5, 0) = translate(0, -1.5, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, -1.5, 0.0],
    // Bone 2: inverse of translate(0, 3.0, 0) = translate(0, -3.0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, -3.0, 0.0],
];

// ============================================================================
// Arm 2: 4-bone horizontal arm (different skeleton to test switching)
// ============================================================================

const ARM2_BONES: usize = 4;
const ARM2_SEGMENT_LENGTH: f32 = 1.0;

/// Inverse bind matrices for Arm 2 (horizontal, along X axis)
/// Bone positions at bind pose:
/// - Bone 0: x = 0 (origin)
/// - Bone 1: x = 1.0
/// - Bone 2: x = 2.0
/// - Bone 3: x = 3.0
static ARM2_INVERSE_BIND: [[f32; 12]; ARM2_BONES] = [
    // Bone 0: identity
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, 0.0, 0.0],
    // Bone 1: inverse of translate(1, 0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  -1.0, 0.0, 0.0],
    // Bone 2: inverse of translate(2, 0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  -2.0, 0.0, 0.0],
    // Bone 3: inverse of translate(3, 0, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  -3.0, 0.0, 0.0],
];

// ============================================================================
// Game State
// ============================================================================

static mut ARM1_MESH: u32 = 0;
static mut ARM1_SKELETON: u32 = 0;
static mut ARM1_BONES_DATA: [f32; ARM1_BONES * BONE_MATRIX_FLOATS] = [0.0; ARM1_BONES * BONE_MATRIX_FLOATS];

static mut ARM2_MESH: u32 = 0;
static mut ARM2_SKELETON: u32 = 0;
static mut ARM2_BONES_DATA: [f32; ARM2_BONES * BONE_MATRIX_FLOATS] = [0.0; ARM2_BONES * BONE_MATRIX_FLOATS];

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
// Math Utilities
// ============================================================================

#[inline]
fn sin(x: f32) -> f32 {
    libm::sinf(x)
}

#[inline]
fn cos(x: f32) -> f32 {
    libm::cosf(x)
}

/// Create 3x4 identity matrix (column-major)
fn mat3x4_identity() -> [f32; 12] {
    [
        1.0, 0.0, 0.0,  // col 0
        0.0, 1.0, 0.0,  // col 1
        0.0, 0.0, 1.0,  // col 2
        0.0, 0.0, 0.0,  // col 3 (translation)
    ]
}

/// Create 3x4 rotation matrix around Z axis (column-major)
fn mat3x4_rotation_z(angle: f32) -> [f32; 12] {
    let c = cos(angle);
    let s = sin(angle);
    [
        c,   s,   0.0,  // col 0
        -s,  c,   0.0,  // col 1
        0.0, 0.0, 1.0,  // col 2
        0.0, 0.0, 0.0,  // col 3
    ]
}

/// Create 3x4 rotation matrix around Y axis (column-major)
fn mat3x4_rotation_y(angle: f32) -> [f32; 12] {
    let c = cos(angle);
    let s = sin(angle);
    [
        c,   0.0, -s,   // col 0
        0.0, 1.0, 0.0,  // col 1
        s,   0.0, c,    // col 2
        0.0, 0.0, 0.0,  // col 3
    ]
}

/// Create 3x4 translation matrix (column-major)
fn mat3x4_translation(x: f32, y: f32, z: f32) -> [f32; 12] {
    [
        1.0, 0.0, 0.0,  // col 0
        0.0, 1.0, 0.0,  // col 1
        0.0, 0.0, 1.0,  // col 2
        x,   y,   z,    // col 3
    ]
}

/// Multiply two 3x4 matrices: out = a * b (column-major)
fn mat3x4_multiply(a: &[f32; 12], b: &[f32; 12]) -> [f32; 12] {
    // Column-major indexing: col_i starts at index i*3
    let mut out = [0.0f32; 12];

    // Result col 0 = A.rot * B.col0
    out[0] = a[0]*b[0] + a[3]*b[1] + a[6]*b[2];
    out[1] = a[1]*b[0] + a[4]*b[1] + a[7]*b[2];
    out[2] = a[2]*b[0] + a[5]*b[1] + a[8]*b[2];

    // Result col 1 = A.rot * B.col1
    out[3] = a[0]*b[3] + a[3]*b[4] + a[6]*b[5];
    out[4] = a[1]*b[3] + a[4]*b[4] + a[7]*b[5];
    out[5] = a[2]*b[3] + a[5]*b[4] + a[8]*b[5];

    // Result col 2 = A.rot * B.col2
    out[6] = a[0]*b[6] + a[3]*b[7] + a[6]*b[8];
    out[7] = a[1]*b[6] + a[4]*b[7] + a[7]*b[8];
    out[8] = a[2]*b[6] + a[5]*b[7] + a[8]*b[8];

    // Result col 3 = A.rot * B.col3 + A.col3
    out[9]  = a[0]*b[9] + a[3]*b[10] + a[6]*b[11] + a[9];
    out[10] = a[1]*b[9] + a[4]*b[10] + a[7]*b[11] + a[10];
    out[11] = a[2]*b[9] + a[5]*b[10] + a[8]*b[11] + a[11];

    out
}

// ============================================================================
// Mesh Generation
// ============================================================================

/// Generate a box segment mesh for Arm 1 (vertical, along Y)
fn generate_arm1_mesh() -> ([f32; 72 * FLOATS_PER_VERTEX], [u16; 108]) {
    let mut vertices = [0.0f32; 72 * FLOATS_PER_VERTEX];
    let mut indices = [0u16; 108];

    let half_w = 0.15;
    let seg_height = ARM1_SEGMENT_LENGTH;

    let mut v_idx = 0;
    let mut i_idx = 0;

    // Generate 3 box segments
    for seg in 0..ARM1_BONES {
        let y_base = seg as f32 * seg_height;
        let bone = seg as u32;
        let base_vert = (seg * 24) as u16;

        // Pack bone indices (same bone for all 4 indices)
        let bone_packed = f32::from_bits(bone | (bone << 8) | (bone << 16) | (bone << 24));

        // 6 faces, 4 vertices each = 24 vertices per segment
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            // Front (+Z)
            ([0.0, 0.0, 1.0], [
                [-half_w, y_base, half_w],
                [half_w, y_base, half_w],
                [half_w, y_base + seg_height, half_w],
                [-half_w, y_base + seg_height, half_w],
            ]),
            // Back (-Z)
            ([0.0, 0.0, -1.0], [
                [half_w, y_base, -half_w],
                [-half_w, y_base, -half_w],
                [-half_w, y_base + seg_height, -half_w],
                [half_w, y_base + seg_height, -half_w],
            ]),
            // Right (+X)
            ([1.0, 0.0, 0.0], [
                [half_w, y_base, half_w],
                [half_w, y_base, -half_w],
                [half_w, y_base + seg_height, -half_w],
                [half_w, y_base + seg_height, half_w],
            ]),
            // Left (-X)
            ([-1.0, 0.0, 0.0], [
                [-half_w, y_base, -half_w],
                [-half_w, y_base, half_w],
                [-half_w, y_base + seg_height, half_w],
                [-half_w, y_base + seg_height, -half_w],
            ]),
            // Top (+Y)
            ([0.0, 1.0, 0.0], [
                [-half_w, y_base + seg_height, half_w],
                [half_w, y_base + seg_height, half_w],
                [half_w, y_base + seg_height, -half_w],
                [-half_w, y_base + seg_height, -half_w],
            ]),
            // Bottom (-Y)
            ([0.0, -1.0, 0.0], [
                [-half_w, y_base, -half_w],
                [half_w, y_base, -half_w],
                [half_w, y_base, half_w],
                [-half_w, y_base, half_w],
            ]),
        ];

        for (face_idx, (normal, corners)) in faces.iter().enumerate() {
            let face_base = base_vert + (face_idx * 4) as u16;

            for corner in corners {
                // Position
                vertices[v_idx] = corner[0];
                vertices[v_idx + 1] = corner[1];
                vertices[v_idx + 2] = corner[2];
                // Normal
                vertices[v_idx + 3] = normal[0];
                vertices[v_idx + 4] = normal[1];
                vertices[v_idx + 5] = normal[2];
                // Bone indices (packed)
                vertices[v_idx + 6] = bone_packed;
                // Weights: 100% to this bone
                vertices[v_idx + 7] = 1.0;
                vertices[v_idx + 8] = 0.0;
                vertices[v_idx + 9] = 0.0;
                vertices[v_idx + 10] = 0.0;
                v_idx += FLOATS_PER_VERTEX;
            }

            // Two triangles per face
            indices[i_idx] = face_base;
            indices[i_idx + 1] = face_base + 1;
            indices[i_idx + 2] = face_base + 2;
            indices[i_idx + 3] = face_base;
            indices[i_idx + 4] = face_base + 2;
            indices[i_idx + 5] = face_base + 3;
            i_idx += 6;
        }
    }

    (vertices, indices)
}

/// Generate a box segment mesh for Arm 2 (horizontal, along X)
fn generate_arm2_mesh() -> ([f32; 96 * FLOATS_PER_VERTEX], [u16; 144]) {
    let mut vertices = [0.0f32; 96 * FLOATS_PER_VERTEX];
    let mut indices = [0u16; 144];

    let half_h = 0.12;
    let seg_len = ARM2_SEGMENT_LENGTH;

    let mut v_idx = 0;
    let mut i_idx = 0;

    // Generate 4 box segments
    for seg in 0..ARM2_BONES {
        let x_base = seg as f32 * seg_len;
        let bone = seg as u32;
        let base_vert = (seg * 24) as u16;

        let bone_packed = f32::from_bits(bone | (bone << 8) | (bone << 16) | (bone << 24));

        // 6 faces for horizontal box
        let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
            // Front (+Z)
            ([0.0, 0.0, 1.0], [
                [x_base, -half_h, half_h],
                [x_base + seg_len, -half_h, half_h],
                [x_base + seg_len, half_h, half_h],
                [x_base, half_h, half_h],
            ]),
            // Back (-Z)
            ([0.0, 0.0, -1.0], [
                [x_base + seg_len, -half_h, -half_h],
                [x_base, -half_h, -half_h],
                [x_base, half_h, -half_h],
                [x_base + seg_len, half_h, -half_h],
            ]),
            // Top (+Y)
            ([0.0, 1.0, 0.0], [
                [x_base, half_h, half_h],
                [x_base + seg_len, half_h, half_h],
                [x_base + seg_len, half_h, -half_h],
                [x_base, half_h, -half_h],
            ]),
            // Bottom (-Y)
            ([0.0, -1.0, 0.0], [
                [x_base, -half_h, -half_h],
                [x_base + seg_len, -half_h, -half_h],
                [x_base + seg_len, -half_h, half_h],
                [x_base, -half_h, half_h],
            ]),
            // Right (+X)
            ([1.0, 0.0, 0.0], [
                [x_base + seg_len, -half_h, half_h],
                [x_base + seg_len, -half_h, -half_h],
                [x_base + seg_len, half_h, -half_h],
                [x_base + seg_len, half_h, half_h],
            ]),
            // Left (-X)
            ([-1.0, 0.0, 0.0], [
                [x_base, -half_h, -half_h],
                [x_base, -half_h, half_h],
                [x_base, half_h, half_h],
                [x_base, half_h, -half_h],
            ]),
        ];

        for (face_idx, (normal, corners)) in faces.iter().enumerate() {
            let face_base = base_vert + (face_idx * 4) as u16;

            for corner in corners {
                vertices[v_idx] = corner[0];
                vertices[v_idx + 1] = corner[1];
                vertices[v_idx + 2] = corner[2];
                vertices[v_idx + 3] = normal[0];
                vertices[v_idx + 4] = normal[1];
                vertices[v_idx + 5] = normal[2];
                vertices[v_idx + 6] = bone_packed;
                vertices[v_idx + 7] = 1.0;
                vertices[v_idx + 8] = 0.0;
                vertices[v_idx + 9] = 0.0;
                vertices[v_idx + 10] = 0.0;
                v_idx += FLOATS_PER_VERTEX;
            }

            indices[i_idx] = face_base;
            indices[i_idx + 1] = face_base + 1;
            indices[i_idx + 2] = face_base + 2;
            indices[i_idx + 3] = face_base;
            indices[i_idx + 4] = face_base + 2;
            indices[i_idx + 5] = face_base + 3;
            i_idx += 6;
        }
    }

    (vertices, indices)
}

// ============================================================================
// Animation
// ============================================================================

/// Update Arm 1 bone matrices (vertical arm with Z rotations)
/// Returns world-space bone transforms
fn update_arm1_bones(time: f32) {
    let bones = unsafe { &mut *addr_of_mut!(ARM1_BONES_DATA) };

    let angle0 = sin(time) * 0.4;
    let angle1 = sin(time * 1.3 + 0.5) * 0.5;
    let angle2 = sin(time * 0.9 + 1.0) * 0.3;

    // Bone 0: rotate at origin
    let rot0 = mat3x4_rotation_z(angle0);
    bones[0..12].copy_from_slice(&rot0);

    // Bone 1: parent transform * local transform
    // Local: translate to joint position, then rotate
    let trans1 = mat3x4_translation(0.0, ARM1_SEGMENT_LENGTH, 0.0);
    let rot1 = mat3x4_rotation_z(angle1);
    let local1 = mat3x4_multiply(&trans1, &rot1);
    let world1 = mat3x4_multiply(&rot0, &local1);
    bones[12..24].copy_from_slice(&world1);

    // Bone 2: bone1_world * local transform
    let trans2 = mat3x4_translation(0.0, ARM1_SEGMENT_LENGTH, 0.0);
    let rot2 = mat3x4_rotation_z(angle2);
    let local2 = mat3x4_multiply(&trans2, &rot2);
    let world2 = mat3x4_multiply(&world1, &local2);
    bones[24..36].copy_from_slice(&world2);
}

/// Update Arm 2 bone matrices (horizontal arm with Y rotations)
fn update_arm2_bones(time: f32) {
    let bones = unsafe { &mut *addr_of_mut!(ARM2_BONES_DATA) };

    let angle0 = sin(time * 0.7) * 0.3;
    let angle1 = sin(time * 1.1 + 0.3) * 0.4;
    let angle2 = sin(time * 0.8 + 0.6) * 0.35;
    let angle3 = sin(time * 1.2 + 0.9) * 0.25;

    // Bone 0: rotate at origin
    let rot0 = mat3x4_rotation_y(angle0);
    bones[0..12].copy_from_slice(&rot0);

    // Bone 1
    let trans1 = mat3x4_translation(ARM2_SEGMENT_LENGTH, 0.0, 0.0);
    let rot1 = mat3x4_rotation_y(angle1);
    let local1 = mat3x4_multiply(&trans1, &rot1);
    let world1 = mat3x4_multiply(&rot0, &local1);
    bones[12..24].copy_from_slice(&world1);

    // Bone 2
    let trans2 = mat3x4_translation(ARM2_SEGMENT_LENGTH, 0.0, 0.0);
    let rot2 = mat3x4_rotation_y(angle2);
    let local2 = mat3x4_multiply(&trans2, &rot2);
    let world2 = mat3x4_multiply(&world1, &local2);
    bones[24..36].copy_from_slice(&world2);

    // Bone 3
    let trans3 = mat3x4_translation(ARM2_SEGMENT_LENGTH, 0.0, 0.0);
    let rot3 = mat3x4_rotation_y(angle3);
    let local3 = mat3x4_multiply(&trans3, &rot3);
    let world3 = mat3x4_multiply(&world2, &local3);
    bones[36..48].copy_from_slice(&world3);
}

// ============================================================================
// Game Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a2a1aFF);
        depth_test(1);

        // Generate and load Arm 1 mesh
        let (verts1, indices1) = generate_arm1_mesh();
        ARM1_MESH = load_mesh_indexed(
            verts1.as_ptr(),
            72,
            indices1.as_ptr(),
            108,
            FORMAT_POS_NORMAL_SKINNED,
        );

        // Load Arm 1 skeleton with inverse bind matrices
        ARM1_SKELETON = load_skeleton(
            ARM1_INVERSE_BIND.as_ptr() as *const f32,
            ARM1_BONES as u32,
        );

        // Generate and load Arm 2 mesh
        let (verts2, indices2) = generate_arm2_mesh();
        ARM2_MESH = load_mesh_indexed(
            verts2.as_ptr(),
            96,
            indices2.as_ptr(),
            144,
            FORMAT_POS_NORMAL_SKINNED,
        );

        // Load Arm 2 skeleton with inverse bind matrices
        ARM2_SKELETON = load_skeleton(
            ARM2_INVERSE_BIND.as_ptr() as *const f32,
            ARM2_BONES as u32,
        );

        // Initialize bone matrices to identity
        for i in 0..ARM1_BONES {
            let ident = mat3x4_identity();
            ARM1_BONES_DATA[i * 12..(i + 1) * 12].copy_from_slice(&ident);
        }
        for i in 0..ARM2_BONES {
            let ident = mat3x4_identity();
            ARM2_BONES_DATA[i * 12..(i + 1) * 12].copy_from_slice(&ident);
        }
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
            ANIM_TIME += 0.05 * ANIM_SPEED;
        }

        // Update bone matrices
        update_arm1_bones(ANIM_TIME);
        update_arm2_bones(ANIM_TIME * 0.8);  // Slightly different speed
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        // Draw Arm 1 (vertical, positioned left)
        skeleton_bind(ARM1_SKELETON);
        set_bones(ARM1_BONES_DATA.as_ptr(), ARM1_BONES as u32);
        push_identity();
        push_translate(-3.0, 0.0, 0.0);
        set_color(0xE09060FF);  // Orange
        draw_mesh(ARM1_MESH);

        // Draw Arm 2 (horizontal, positioned right)
        skeleton_bind(ARM2_SKELETON);
        set_bones(ARM2_BONES_DATA.as_ptr(), ARM2_BONES as u32);
        push_identity();
        push_translate(1.0, 2.0, 0.0);
        set_color(0x60A0E0FF);  // Blue
        draw_mesh(ARM2_MESH);

        // Draw UI
        draw_ui();
    }
}

fn draw_ui() {
    unsafe {
        let y = 10.0;
        let line_h = 18.0;

        let title = b"Multi-Skinned Procedural";
        set_color(0xFFFFFFFF);
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0);

        let subtitle = b"Testing inverse bind matrices";
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
    }
}
