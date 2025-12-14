//! Skinned Mesh Example - GPU Skinning Validator (Animation System v2)
//!
//! Validates Emberware Z's skeletal animation system:
//! - 3x4 bone matrix format (12 floats per bone, column-major)
//! - GPU skinning with smooth weight blending
//! - Multiple bone hierarchy animation
//! - Procedural/dynamic bone animation using immediate bones buffer
//!
//! Matrix Format (column-major, consistent with transform_set):
//! Each bone matrix is stored as 4 columns × 3 elements:
//! ```text
//! [col0.x, col0.y, col0.z]  // X axis
//! [col1.x, col1.y, col1.z]  // Y axis
//! [col2.x, col2.y, col2.z]  // Z axis
//! [tx,     ty,     tz    ]  // translation
//! // implicit 4th row [0, 0, 0, 1] (affine transform)
//! ```
//!
//! Animation System v2 Features:
//! - set_bones() appends to per-frame immediate_bones buffer (indexed access)
//! - Multiple bone states can coexist in a single frame
//! - GPU reads bones via indexed offset (no re-uploads between draws)
//!
//! Workflow: CPU animation -> GPU skinning
//! 1. In init(): Load skinned mesh with bone indices/weights baked into vertices
//! 2. Each update(): Animate skeleton on CPU (update bone transforms)
//! 3. Each render(): Call set_bones() then draw_mesh()
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted).
//!
//! Controls:
//! - Left stick: Rotate view
//! - A button: Toggle animation pause
//! - D-pad Up/Down: Adjust animation speed

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
    fn left_stick_x(player: u32) -> f32;
    fn left_stick_y(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;

    // Mesh
    fn load_mesh_indexed(
        data: *const f32,
        vertex_count: u32,
        indices: *const u16,
        index_count: u32,
        format: u32,
    ) -> u32;
    fn draw_mesh(handle: u32);

    // GPU Skinning - 3x4 matrices (12 floats per bone, row-major)
    fn set_bones(matrices_ptr: *const f32, count: u32);

    // Transform
    fn push_identity();

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // 2D UI
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

/// Vertex format flags
const FORMAT_NORMAL: u32 = 4;
const FORMAT_SKINNED: u32 = 8;

/// POS_NORMAL_SKINNED format (12)
/// Layout: pos(3f) + normal(3f) + bone_indices(4u8) + bone_weights(4f)
const FORMAT_POS_NORMAL_SKINNED: u32 = FORMAT_NORMAL | FORMAT_SKINNED;

/// Number of bones in our simple skeleton
const NUM_BONES: usize = 3;

/// Floats per 3x4 bone matrix (row-major)
const BONE_MATRIX_FLOATS: usize = 12;

/// Arm segment length
const SEGMENT_LENGTH: f32 = 1.5;

/// Mesh handle
static mut ARM_MESH: u32 = 0;

/// View rotation
static mut VIEW_ROTATION_X: f32 = 15.0;
static mut VIEW_ROTATION_Y: f32 = 0.0;

/// Animation time (in radians, wraps around)
static mut ANIM_TIME: f32 = 0.0;

/// Animation speed multiplier
static mut ANIM_SPEED: f32 = 1.0;

/// Animation paused
static mut PAUSED: bool = false;

/// Bone matrices (3 bones × 12 floats each = 36 floats)
/// Each 3x4 matrix is stored in row-major order
static mut BONE_MATRICES: [f32; NUM_BONES * BONE_MATRIX_FLOATS] = [0.0; NUM_BONES * BONE_MATRIX_FLOATS];

// ============================================================================
// Math Utilities (using libm for accurate no_std math)
// ============================================================================

#[inline]
fn sin_approx(x: f32) -> f32 {
    libm::sinf(x)
}

#[inline]
fn cos_approx(x: f32) -> f32 {
    libm::cosf(x)
}

// ============================================================================
// 3x4 Matrix Operations (Column-Major - consistent with transform_set)
// ============================================================================
//
// Memory layout (12 floats):
//   [col0.x, col0.y, col0.z,  // X axis
//    col1.x, col1.y, col1.z,  // Y axis
//    col2.x, col2.y, col2.z,  // Z axis
//    tx,     ty,     tz    ]  // translation
//
// This is the same convention as transform_set, view_matrix_set, etc.

/// Create a 3x4 identity matrix (column-major)
fn mat3x4_identity(out: &mut [f32; 12]) {
    *out = [
        1.0, 0.0, 0.0, // col 0: X axis
        0.0, 1.0, 0.0, // col 1: Y axis
        0.0, 0.0, 1.0, // col 2: Z axis
        0.0, 0.0, 0.0, // col 3: translation
    ];
}

/// Create a 3x4 rotation matrix around Z axis (column-major)
fn mat3x4_rotation_z(out: &mut [f32; 12], angle: f32) {
    let c = cos_approx(angle);
    let s = sin_approx(angle);
    *out = [
        c,   s,   0.0, // col 0: X axis rotated
        -s,  c,   0.0, // col 1: Y axis rotated
        0.0, 0.0, 1.0, // col 2: Z axis unchanged
        0.0, 0.0, 0.0, // col 3: no translation
    ];
}

/// Create a 3x4 translation matrix (column-major)
fn mat3x4_translation(out: &mut [f32; 12], x: f32, y: f32, z: f32) {
    *out = [
        1.0, 0.0, 0.0, // col 0: X axis
        0.0, 1.0, 0.0, // col 1: Y axis
        0.0, 0.0, 1.0, // col 2: Z axis
        x,   y,   z,   // col 3: translation
    ];
}

/// Multiply two 3x4 matrices: out = a * b (both column-major)
/// For column vectors: (A * B) * v = A * (B * v), so B is applied first.
/// Treats implicit 4th row as [0, 0, 0, 1]
fn mat3x4_multiply(out: &mut [f32; 12], a: &[f32; 12], b: &[f32; 12]) {
    // Column-major indexing: col_i starts at index i*3
    // a's 3x3 rotation is cols 0-2, translation is col 3
    // Same for b

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
}

// ============================================================================
// Mesh Generation
// ============================================================================

/// Generate a cylindrical arm segment mesh with skinning data
fn generate_arm_mesh() -> ([f32; 60 * 11], [u16; 324]) {
    const SEGMENTS: usize = 6;
    const RINGS: usize = 10;
    const RADIUS: f32 = 0.2;
    const TOTAL_LENGTH: f32 = SEGMENT_LENGTH * 3.0;

    let mut vertices = [0.0f32; 60 * 11];
    let mut indices = [0u16; 324];

    let mut v_idx = 0;
    let mut i_idx = 0;

    for ring in 0..RINGS {
        let y = (ring as f32 / (RINGS - 1) as f32) * TOTAL_LENGTH - TOTAL_LENGTH * 0.5;
        let normalized_pos = (y + TOTAL_LENGTH * 0.5) / TOTAL_LENGTH;

        // Calculate bone weights with smooth blending
        let (bone0_weight, bone1_weight, bone2_weight) = if normalized_pos < 0.33 {
            let blend = normalized_pos / 0.33;
            (1.0 - blend * 0.5, blend * 0.5, 0.0)
        } else if normalized_pos < 0.66 {
            let blend = (normalized_pos - 0.33) / 0.33;
            ((1.0 - blend) * 0.3, 0.4 + (1.0 - (2.0 * blend - 1.0).abs()) * 0.3, blend * 0.3)
        } else {
            let blend = (normalized_pos - 0.66) / 0.34;
            (0.0, (1.0 - blend) * 0.5, 0.5 + blend * 0.5)
        };

        let weight_sum = bone0_weight + bone1_weight + bone2_weight;
        let (w0, w1, w2) = (bone0_weight / weight_sum, bone1_weight / weight_sum, bone2_weight / weight_sum);

        // Pack bone indices as 4 bytes into a u32
        let bone_indices_packed: u32 = 0 | (1 << 8) | (2 << 16) | (0 << 24);
        let bone_indices_f32 = f32::from_bits(bone_indices_packed);

        for seg in 0..SEGMENTS {
            let angle = (seg as f32 / SEGMENTS as f32) * 6.283185307;
            let (nx, nz) = (cos_approx(angle), sin_approx(angle));
            let (x, z) = (nx * RADIUS, nz * RADIUS);

            vertices[v_idx] = x;
            vertices[v_idx + 1] = y;
            vertices[v_idx + 2] = z;
            vertices[v_idx + 3] = nx;
            vertices[v_idx + 4] = 0.0;
            vertices[v_idx + 5] = nz;
            vertices[v_idx + 6] = bone_indices_f32;
            vertices[v_idx + 7] = w0;
            vertices[v_idx + 8] = w1;
            vertices[v_idx + 9] = w2;
            vertices[v_idx + 10] = 0.0;
            v_idx += 11;
        }

        if ring < RINGS - 1 {
            let ring_start = (ring * SEGMENTS) as u16;
            let next_ring_start = ((ring + 1) * SEGMENTS) as u16;

            for seg in 0..SEGMENTS {
                let curr = ring_start + seg as u16;
                let next = ring_start + ((seg + 1) % SEGMENTS) as u16;
                let curr_up = next_ring_start + seg as u16;
                let next_up = next_ring_start + ((seg + 1) % SEGMENTS) as u16;

                indices[i_idx] = curr;
                indices[i_idx + 1] = next;
                indices[i_idx + 2] = curr_up;
                indices[i_idx + 3] = next;
                indices[i_idx + 4] = next_up;
                indices[i_idx + 5] = curr_up;
                i_idx += 6;
            }
        }
    }

    (vertices, indices)
}

// ============================================================================
// Animation
// ============================================================================

/// Update bone matrices based on animation time
/// Creates a wave-like bending motion through the arm
///
/// For proper skeletal animation with column vectors (M * v), rotation around
/// a pivot point P requires: T(P) * R * T(-P)
/// - T(-P): move pivot to origin
/// - R: rotate around origin
/// - T(P): move back to pivot position
fn update_bones(time: f32) {
    // Use addr_of_mut! to avoid Rust 2024 static mut reference warnings
    let bones = unsafe { &mut *core::ptr::addr_of_mut!(BONE_MATRICES) };

    let angle0 = sin_approx(time) * 0.3;
    let angle1 = sin_approx(time + 1.0) * 0.5;
    let angle2 = sin_approx(time + 2.0) * 0.4;

    let mut rot = [0.0f32; 12];
    let mut trans_to = [0.0f32; 12];
    let mut trans_from = [0.0f32; 12];
    let mut temp1 = [0.0f32; 12];
    let mut temp2 = [0.0f32; 12];

    // Joint positions in bind pose (mesh spans y = -2.25 to +2.25)
    // Joint 0 at base: y = -SEGMENT_LENGTH * 1.5 = -2.25
    // Joint 1 in middle: y = -SEGMENT_LENGTH * 0.5 = -0.75
    // Joint 2 near top: y = SEGMENT_LENGTH * 0.5 = 0.75
    let joint0_y = -SEGMENT_LENGTH * 1.5;
    let joint1_y = -SEGMENT_LENGTH * 0.5;
    let joint2_y = SEGMENT_LENGTH * 0.5;

    // Bone 0: rotate around base joint (y = -2.25)
    // bone0 = T(joint0) * R * T(-joint0)
    mat3x4_translation(&mut trans_from, 0.0, -joint0_y, 0.0); // T(-P): move joint to origin
    mat3x4_rotation_z(&mut rot, angle0);
    mat3x4_translation(&mut trans_to, 0.0, joint0_y, 0.0); // T(P): move back
    mat3x4_multiply(&mut temp1, &rot, &trans_from); // R * T(-P)
    {
        let bone0: &mut [f32; 12] = (&mut bones[0..12]).try_into().unwrap();
        mat3x4_multiply(bone0, &trans_to, &temp1); // T(P) * R * T(-P)
    }

    // Bone 1: inherit bone 0's transform, then rotate around joint 1
    // bone1 = bone0 * T(joint1) * R * T(-joint1)
    let bone0_copy: [f32; 12] = bones[0..12].try_into().unwrap();
    mat3x4_translation(&mut trans_from, 0.0, -joint1_y, 0.0); // T(-P)
    mat3x4_rotation_z(&mut rot, angle1);
    mat3x4_translation(&mut trans_to, 0.0, joint1_y, 0.0); // T(P)
    mat3x4_multiply(&mut temp1, &rot, &trans_from); // R * T(-P)
    mat3x4_multiply(&mut temp2, &trans_to, &temp1); // T(P) * R * T(-P)
    {
        let bone1: &mut [f32; 12] = (&mut bones[12..24]).try_into().unwrap();
        mat3x4_multiply(bone1, &bone0_copy, &temp2); // bone0 * local_transform
    }

    // Bone 2: inherit bone 1's transform, then rotate around joint 2
    // bone2 = bone1 * T(joint2) * R * T(-joint2)
    let bone1_copy: [f32; 12] = bones[12..24].try_into().unwrap();
    mat3x4_translation(&mut trans_from, 0.0, -joint2_y, 0.0); // T(-P)
    mat3x4_rotation_z(&mut rot, angle2);
    mat3x4_translation(&mut trans_to, 0.0, joint2_y, 0.0); // T(P)
    mat3x4_multiply(&mut temp1, &rot, &trans_from); // R * T(-P)
    mat3x4_multiply(&mut temp2, &trans_to, &temp1); // T(P) * R * T(-P)
    {
        let bone2: &mut [f32; 12] = (&mut bones[24..36]).try_into().unwrap();
        mat3x4_multiply(bone2, &bone1_copy, &temp2); // bone1 * local_transform
    }
}

// ============================================================================
// Game Lifecycle
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        set_clear_color(0x1a1a2eFF);
        depth_test(1);

        let (vertices, indices) = generate_arm_mesh();
        ARM_MESH = load_mesh_indexed(
            vertices.as_ptr(),
            60,
            indices.as_ptr(),
            324,
            FORMAT_POS_NORMAL_SKINNED,
        );

        // Initialize bone matrices to identity (3x4 format)
        let bones = &mut *core::ptr::addr_of_mut!(BONE_MATRICES);
        for i in 0..NUM_BONES {
            let mut identity = [0.0f32; 12];
            mat3x4_identity(&mut identity);
            bones[i * 12..(i + 1) * 12].copy_from_slice(&identity);
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        VIEW_ROTATION_Y += stick_x * 2.0;
        VIEW_ROTATION_X += stick_y * 2.0;

        if VIEW_ROTATION_X > 89.0 { VIEW_ROTATION_X = 89.0; }
        if VIEW_ROTATION_X < -89.0 { VIEW_ROTATION_X = -89.0; }

        if button_pressed(0, BUTTON_A) != 0 {
            PAUSED = !PAUSED;
        }

        if button_held(0, BUTTON_UP) != 0 {
            ANIM_SPEED += 0.02;
            if ANIM_SPEED > 3.0 { ANIM_SPEED = 3.0; }
        }
        if button_held(0, BUTTON_DOWN) != 0 {
            ANIM_SPEED -= 0.02;
            if ANIM_SPEED < 0.1 { ANIM_SPEED = 0.1; }
        }

        if !PAUSED {
            ANIM_TIME += 0.05 * ANIM_SPEED;
            const TWO_PI: f32 = 6.283185307;
            if ANIM_TIME > TWO_PI * 100.0 {
                ANIM_TIME -= TWO_PI * 100.0;
            }
        }

        update_bones(ANIM_TIME);
    }
}

fn format_float(val: f32, buf: &mut [u8]) -> usize {
    let whole = val as i32;
    let frac = ((val - whole as f32).abs() * 100.0) as i32;

    let mut i = 0;
    if whole < 0 {
        buf[i] = b'-';
        i += 1;
    }
    let whole_abs = if whole < 0 { -whole } else { whole };

    if whole_abs >= 10 {
        buf[i] = b'0' + (whole_abs / 10) as u8;
        i += 1;
    }
    buf[i] = b'0' + (whole_abs % 10) as u8;
    i += 1;
    buf[i] = b'.';
    i += 1;
    buf[i] = b'0' + (frac / 10) as u8;
    i += 1;
    buf[i] = b'0' + (frac % 10) as u8;
    i += 1;
    i
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 1.0, 8.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Upload 3x4 bone matrices to GPU (12 floats × 3 bones)
        let bones = &*core::ptr::addr_of!(BONE_MATRICES);
        set_bones(bones.as_ptr(), NUM_BONES as u32);

        push_identity();
        set_color(0xE0C090FF);
        draw_mesh(ARM_MESH);

        // Draw UI (scaled for 960x540 resolution)
        let y = 10.0;
        let line_h = 18.0;

        let title = b"GPU Skinning (3x4 Matrices)";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0, 0xFFFFFFFF);

        let mut buf = [0u8; 32];

        let prefix = b"Speed (D-pad): ";
        let len = format_float(ANIM_SPEED, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 10.0, y + line_h, 12.0, 0xCCCCCCFF);

        let status = if PAUSED { b"Status: PAUSED (A)" as &[u8] } else { b"Status: Playing (A)" as &[u8] };
        draw_text(status.as_ptr(), status.len() as u32, 10.0, y + line_h * 2.0, 12.0, 0xCCCCCCFF);

        let bones_label = b"3 bones, 12 floats/bone (25% savings)";
        draw_text(bones_label.as_ptr(), bones_label.len() as u32, 10.0, y + line_h * 3.5, 10.0, 0x888888FF);

        let hint = b"L-Stick: Rotate view";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, y + line_h * 4.5, 10.0, 0x666666FF);
    }
}
