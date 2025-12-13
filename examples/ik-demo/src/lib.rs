//! IK Demo - Two-Bone Inverse Kinematics Example
//!
//! Demonstrates procedural IK animation using the skeleton system:
//! - Two-bone analytical IK solver using law of cosines
//! - Inverse bind matrices with procedural bone computation
//! - Real-time target tracking
//! - Reach limit constraints
//!
//! This validates the inverse bind matrix path with dynamically computed bone transforms.
//!
//! Controls:
//! - Left stick: Move IK target
//! - A button: Toggle between stick control and auto-circle
//! - B button: Reset target to center

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::addr_of_mut;

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
    fn left_stick_y(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;

    fn load_mesh_indexed(
        data: *const f32,
        vertex_count: u32,
        indices: *const u16,
        index_count: u32,
        format: u32,
    ) -> u32;
    fn draw_mesh(handle: u32);

    fn load_skeleton(inverse_bind_ptr: *const f32, bone_count: u32) -> u32;
    fn skeleton_bind(skeleton: u32);
    fn set_bones(matrices_ptr: *const f32, count: u32);

    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);

    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// ============================================================================
// Constants
// ============================================================================

const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;

const FORMAT_NORMAL: u32 = 4;
const FORMAT_SKINNED: u32 = 8;
const FORMAT_POS_NORMAL_SKINNED: u32 = FORMAT_NORMAL | FORMAT_SKINNED;

const FLOATS_PER_VERTEX: usize = 11;
const BONE_MATRIX_FLOATS: usize = 12;

const PI: f32 = 3.14159265;
const TWO_PI: f32 = 6.28318530;

// IK arm parameters
const NUM_BONES: usize = 2;
const UPPER_ARM_LENGTH: f32 = 2.0;
const LOWER_ARM_LENGTH: f32 = 2.0;
const MAX_REACH: f32 = UPPER_ARM_LENGTH + LOWER_ARM_LENGTH - 0.01;
const MIN_REACH: f32 = 0.1;

// Shoulder position (fixed)
const SHOULDER: [f32; 3] = [0.0, 2.0, 0.0];

/// Inverse bind matrices for the IK arm
/// At bind pose: arm is straight along +Y axis
/// - Bone 0 (upper arm): at shoulder position
/// - Bone 1 (lower arm): at elbow position (shoulder + UPPER_ARM_LENGTH along Y)
static INVERSE_BIND: [[f32; 12]; NUM_BONES] = [
    // Bone 0: inverse of translate(0, 2, 0) = translate(0, -2, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, -2.0, 0.0],
    // Bone 1: inverse of translate(0, 4, 0) = translate(0, -4, 0)
    [1.0, 0.0, 0.0,  0.0, 1.0, 0.0,  0.0, 0.0, 1.0,  0.0, -4.0, 0.0],
];

// ============================================================================
// Game State
// ============================================================================

static mut ARM_MESH: u32 = 0;
static mut ARM_SKELETON: u32 = 0;
static mut BONE_MATRICES: [f32; NUM_BONES * BONE_MATRIX_FLOATS] = [0.0; NUM_BONES * BONE_MATRIX_FLOATS];

static mut TARGET: [f32; 3] = [2.0, 3.0, 0.0];
static mut AUTO_MODE: bool = true;
static mut TIME: f32 = 0.0;

// ============================================================================
// Math Utilities
// ============================================================================

#[inline]
fn sin(x: f32) -> f32 { libm::sinf(x) }

#[inline]
fn cos(x: f32) -> f32 { libm::cosf(x) }

#[inline]
fn sqrt(x: f32) -> f32 { libm::sqrtf(x) }

#[inline]
fn acos(x: f32) -> f32 { libm::acosf(x) }

#[inline]
fn atan2(y: f32, x: f32) -> f32 { libm::atan2f(y, x) }

#[inline]
fn abs(x: f32) -> f32 { libm::fabsf(x) }

fn clamp(x: f32, min: f32, max: f32) -> f32 {
    if x < min { min } else if x > max { max } else { x }
}

/// Create 3x4 identity matrix (column-major)
fn mat3x4_identity() -> [f32; 12] {
    [
        1.0, 0.0, 0.0,
        0.0, 1.0, 0.0,
        0.0, 0.0, 1.0,
        0.0, 0.0, 0.0,
    ]
}

/// Create 3x4 rotation matrix around Z axis at a given position (column-major)
/// Equivalent to: T(pos) * Rz(angle)
fn mat3x4_rotation_z_at(pos: [f32; 3], angle: f32) -> [f32; 12] {
    let c = cos(angle);
    let s = sin(angle);
    [
        c,   s,   0.0,      // col 0
        -s,  c,   0.0,      // col 1
        0.0, 0.0, 1.0,      // col 2
        pos[0], pos[1], pos[2],  // col 3 (translation)
    ]
}

/// Multiply two 3x4 matrices: out = a * b (column-major)
fn mat3x4_multiply(a: &[f32; 12], b: &[f32; 12]) -> [f32; 12] {
    let mut out = [0.0f32; 12];

    out[0] = a[0]*b[0] + a[3]*b[1] + a[6]*b[2];
    out[1] = a[1]*b[0] + a[4]*b[1] + a[7]*b[2];
    out[2] = a[2]*b[0] + a[5]*b[1] + a[8]*b[2];

    out[3] = a[0]*b[3] + a[3]*b[4] + a[6]*b[5];
    out[4] = a[1]*b[3] + a[4]*b[4] + a[7]*b[5];
    out[5] = a[2]*b[3] + a[5]*b[4] + a[8]*b[5];

    out[6] = a[0]*b[6] + a[3]*b[7] + a[6]*b[8];
    out[7] = a[1]*b[6] + a[4]*b[7] + a[7]*b[8];
    out[8] = a[2]*b[6] + a[5]*b[7] + a[8]*b[8];

    out[9]  = a[0]*b[9] + a[3]*b[10] + a[6]*b[11] + a[9];
    out[10] = a[1]*b[9] + a[4]*b[10] + a[7]*b[11] + a[10];
    out[11] = a[2]*b[9] + a[5]*b[10] + a[8]*b[11] + a[11];

    out
}

// ============================================================================
// Two-Bone IK Solver
// ============================================================================

/// Solve two-bone IK in the XY plane (2D IK, Z ignored for simplicity)
/// Returns world-space bone matrices for upper arm and lower arm
fn solve_two_bone_ik(shoulder: [f32; 3], target: [f32; 3]) -> [[f32; 12]; 2] {
    // Vector from shoulder to target (in XY plane)
    let dx = target[0] - shoulder[0];
    let dy = target[1] - shoulder[1];
    let dist = sqrt(dx * dx + dy * dy);

    // Clamp distance to reachable range
    let clamped_dist = clamp(dist, MIN_REACH, MAX_REACH);

    // Law of cosines to find elbow angle
    // For triangle with sides a=upper, b=lower, c=dist:
    // c² = a² + b² - 2ab*cos(C)
    // cos(elbow_interior) = (a² + b² - c²) / (2ab)
    let a2 = UPPER_ARM_LENGTH * UPPER_ARM_LENGTH;
    let b2 = LOWER_ARM_LENGTH * LOWER_ARM_LENGTH;
    let c2 = clamped_dist * clamped_dist;

    let cos_elbow_interior = (a2 + b2 - c2) / (2.0 * UPPER_ARM_LENGTH * LOWER_ARM_LENGTH);
    let cos_elbow_clamped = clamp(cos_elbow_interior, -1.0, 1.0);
    let elbow_interior_angle = acos(cos_elbow_clamped);

    // The elbow bends "outward" - we want the angle relative to the upper arm
    // In our convention, 0 = straight arm, PI = fully bent
    let elbow_bend = PI - elbow_interior_angle;

    // Angle from shoulder to target
    let target_angle = atan2(dy, dx);

    // Angle offset due to elbow bend (using law of sines)
    // sin(offset) / lower = sin(elbow_interior) / dist
    let sin_elbow_interior = sin(elbow_interior_angle);
    let sin_offset = (LOWER_ARM_LENGTH * sin_elbow_interior) / clamped_dist;
    let offset_angle = libm::asinf(clamp(sin_offset, -1.0, 1.0));

    // Shoulder angle (from +Y axis in our coordinate system)
    // Our arm points up (+Y) at rest, so we rotate from there
    // target_angle is from +X, so shoulder = target_angle - PI/2 + offset
    let shoulder_angle = target_angle - PI / 2.0 + offset_angle;

    // Build bone 0 (upper arm) matrix
    // Rotates at shoulder position, angle is shoulder_angle
    let bone0 = mat3x4_rotation_z_at(shoulder, shoulder_angle);

    // Compute elbow position
    let elbow_pos = [
        shoulder[0] + UPPER_ARM_LENGTH * cos(target_angle - PI / 2.0 + offset_angle + PI / 2.0),
        shoulder[1] + UPPER_ARM_LENGTH * sin(target_angle - PI / 2.0 + offset_angle + PI / 2.0),
        shoulder[2],
    ];

    // Actually, simpler approach: elbow is at shoulder + rotated upper arm
    let upper_end_local = [0.0, UPPER_ARM_LENGTH, 0.0];  // local space
    let elbow_x = shoulder[0] + upper_end_local[0] * cos(shoulder_angle) - upper_end_local[1] * sin(shoulder_angle);
    let elbow_y = shoulder[1] + upper_end_local[0] * sin(shoulder_angle) + upper_end_local[1] * cos(shoulder_angle);
    let elbow_actual = [elbow_x, elbow_y, shoulder[2]];

    // Build bone 1 (lower arm) matrix
    // The lower arm rotates relative to upper arm, so we need world rotation
    // Lower arm angle = shoulder_angle - elbow_bend (bend is negative/inward)
    let lower_angle = shoulder_angle - elbow_bend;
    let bone1 = mat3x4_rotation_z_at(elbow_actual, lower_angle);

    [bone0, bone1]
}

// ============================================================================
// Mesh Generation
// ============================================================================

/// Generate a two-segment arm mesh (upper + lower arm)
fn generate_arm_mesh() -> ([f32; 48 * FLOATS_PER_VERTEX], [u16; 72]) {
    let mut vertices = [0.0f32; 48 * FLOATS_PER_VERTEX];
    let mut indices = [0u16; 72];

    let half_w = 0.15;

    let mut v_idx = 0;
    let mut i_idx = 0;

    // Generate 2 segments
    for seg in 0..NUM_BONES {
        let y_base = if seg == 0 { 0.0 } else { UPPER_ARM_LENGTH };
        let seg_height = if seg == 0 { UPPER_ARM_LENGTH } else { LOWER_ARM_LENGTH };
        let bone = seg as u32;
        let base_vert = (seg * 24) as u16;

        let bone_packed = f32::from_bits(bone | (bone << 8) | (bone << 16) | (bone << 24));

        // 6 faces
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
                // Position (offset by shoulder for bind pose)
                vertices[v_idx] = corner[0];
                vertices[v_idx + 1] = corner[1] + SHOULDER[1];  // Add shoulder Y offset
                vertices[v_idx + 2] = corner[2];
                // Normal
                vertices[v_idx + 3] = normal[0];
                vertices[v_idx + 4] = normal[1];
                vertices[v_idx + 5] = normal[2];
                // Bone indices
                vertices[v_idx + 6] = bone_packed;
                // Weights
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
// Game Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        // Camera to see 2-bone IK arm (total length 4 units, extending up from origin)
        camera_set(0.0, 2.5, 8.0, 0.0, 2.0, 0.0);
        camera_fov(55.0);
        depth_test(1);

        // Generate and load arm mesh
        let (verts, indices) = generate_arm_mesh();
        ARM_MESH = load_mesh_indexed(
            verts.as_ptr(),
            48,
            indices.as_ptr(),
            72,
            FORMAT_POS_NORMAL_SKINNED,
        );

        // Load skeleton with inverse bind matrices
        ARM_SKELETON = load_skeleton(
            INVERSE_BIND.as_ptr() as *const f32,
            NUM_BONES as u32,
        );

        // Initialize bone matrices
        for i in 0..NUM_BONES {
            let ident = mat3x4_identity();
            BONE_MATRICES[i * 12..(i + 1) * 12].copy_from_slice(&ident);
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        TIME += 0.02;

        // Toggle mode
        if button_pressed(0, BUTTON_A) != 0 {
            AUTO_MODE = !AUTO_MODE;
        }

        // Reset target
        if button_pressed(0, BUTTON_B) != 0 {
            TARGET = [2.0, 3.0, 0.0];
        }

        if AUTO_MODE {
            // Circular motion
            let radius = 2.5;
            let center_x = 0.0;
            let center_y = 3.0;
            TARGET[0] = center_x + cos(TIME) * radius;
            TARGET[1] = center_y + sin(TIME * 0.7) * radius * 0.8;
        } else {
            // Stick control
            let dx = left_stick_x(0) * 0.1;
            let dy = left_stick_y(0) * 0.1;
            TARGET[0] += dx;
            TARGET[1] += dy;

            // Clamp to reasonable range
            TARGET[0] = clamp(TARGET[0], -5.0, 5.0);
            TARGET[1] = clamp(TARGET[1], -1.0, 7.0);
        }

        // Solve IK
        let bones = solve_two_bone_ik(SHOULDER, TARGET);

        // Copy to bone matrices array
        let bone_data = &mut *addr_of_mut!(BONE_MATRICES);
        bone_data[0..12].copy_from_slice(&bones[0]);
        bone_data[12..24].copy_from_slice(&bones[1]);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Draw arm with IK-computed bone transforms
        skeleton_bind(ARM_SKELETON);
        set_bones(BONE_MATRICES.as_ptr(), NUM_BONES as u32);
        push_identity();
        set_color(0x80C080FF);  // Light green
        draw_mesh(ARM_MESH);

        // Note: Target and shoulder markers would use draw_sphere if available
        // The IK arm reaching toward the target demonstrates the solver

        // Draw UI
        draw_ui();
    }
}

fn draw_ui() {
    unsafe {
        let y = 10.0;
        let line_h = 18.0;

        let title = b"IK Demo - Two-Bone Solver";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0, 0xFFFFFFFF);

        let subtitle = b"Tests inverse bind with procedural IK";
        draw_text(subtitle.as_ptr(), subtitle.len() as u32, 10.0, y + line_h, 12.0, 0xAAAAAAFF);

        let mode = if AUTO_MODE {
            b"Mode: Auto circle (A to toggle)" as &[u8]
        } else {
            b"Mode: Stick control (A to toggle)" as &[u8]
        };
        draw_text(mode.as_ptr(), mode.len() as u32, 10.0, y + line_h * 2.5, 10.0, 0x88FF88FF);

        let info1 = b"Upper arm: 2.0 units (green)";
        draw_text(info1.as_ptr(), info1.len() as u32, 10.0, y + line_h * 4.0, 10.0, 0x888888FF);

        let info2 = b"Lower arm: 2.0 units";
        draw_text(info2.as_ptr(), info2.len() as u32, 10.0, y + line_h * 5.0, 10.0, 0x888888FF);

        let info3 = b"Target: red sphere";
        draw_text(info3.as_ptr(), info3.len() as u32, 10.0, y + line_h * 6.0, 10.0, 0xFF6060FF);

        let controls = b"B: Reset target";
        draw_text(controls.as_ptr(), controls.len() as u32, 10.0, y + line_h * 7.5, 10.0, 0x666666FF);
    }
}
