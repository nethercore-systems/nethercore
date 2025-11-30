//! Skinned Mesh Example
//!
//! Demonstrates GPU skinning with Emberware Z's skeletal animation system.
//!
//! Features demonstrated:
//! - `load_mesh_indexed()` with FORMAT_NORMAL | FORMAT_SKINNED
//! - `set_bones()` to upload bone matrices each frame
//! - CPU-side bone animation (sine wave demo)
//! - Simple bone hierarchy (3-bone arm)
//!
//! Workflow: CPU animation -> GPU skinning
//! 1. In init(): Load skinned mesh with bone indices/weights baked into vertices
//! 2. Each update(): Animate skeleton on CPU (update bone transforms)
//! 3. Each render(): Call set_bones() then draw_mesh()
//!
//! Controls:
//! - Left stick: Rotate view
//! - A button: Toggle animation pause
//! - D-pad Up/Down: Adjust animation speed

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);
    fn set_sky(
        horizon_r: f32, horizon_g: f32, horizon_b: f32,
        zenith_r: f32, zenith_g: f32, zenith_b: f32,
        sun_dir_x: f32, sun_dir_y: f32, sun_dir_z: f32,
        sun_r: f32, sun_g: f32, sun_b: f32,
        sun_sharpness: f32,
    );

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

    // GPU Skinning
    fn set_bones(matrices_ptr: *const f32, count: u32);

    // Transform
    fn transform_identity();
    fn transform_rotate(angle_deg: f32, x: f32, y: f32, z: f32);

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
/// Stride: 12 + 12 + 4 + 16 = 44 bytes = 11 floats
const FORMAT_POS_NORMAL_SKINNED: u32 = FORMAT_NORMAL | FORMAT_SKINNED;

/// Number of bones in our simple skeleton
const NUM_BONES: usize = 3;

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

/// Bone matrices (3 bones x 16 floats each = 48 floats)
/// Each 4x4 matrix is stored in column-major order
static mut BONE_MATRICES: [f32; NUM_BONES * 16] = [0.0; NUM_BONES * 16];

/// Simple sine approximation using Taylor series (good for small angles)
/// sin(x) ≈ x - x^3/6 + x^5/120 for |x| < π
fn sin_approx(mut x: f32) -> f32 {
    // Normalize to [-π, π]
    const TWO_PI: f32 = 6.283185307;
    const PI: f32 = 3.141592654;
    while x > PI { x -= TWO_PI; }
    while x < -PI { x += TWO_PI; }

    let x2 = x * x;
    let x3 = x2 * x;
    let x5 = x3 * x2;
    x - x3 / 6.0 + x5 / 120.0
}

/// cos(x) = sin(x + π/2)
fn cos_approx(x: f32) -> f32 {
    const HALF_PI: f32 = 1.570796327;
    sin_approx(x + HALF_PI)
}

/// Create a 4x4 identity matrix (column-major)
fn mat4_identity(out: &mut [f32; 16]) {
    *out = [
        1.0, 0.0, 0.0, 0.0, // column 0
        0.0, 1.0, 0.0, 0.0, // column 1
        0.0, 0.0, 1.0, 0.0, // column 2
        0.0, 0.0, 0.0, 1.0, // column 3
    ];
}

/// Create a rotation matrix around Z axis (column-major)
fn mat4_rotation_z(out: &mut [f32; 16], angle: f32) {
    let c = cos_approx(angle);
    let s = sin_approx(angle);
    *out = [
        c,   s,   0.0, 0.0, // column 0
        -s,  c,   0.0, 0.0, // column 1
        0.0, 0.0, 1.0, 0.0, // column 2
        0.0, 0.0, 0.0, 1.0, // column 3
    ];
}

/// Create a translation matrix (column-major)
fn mat4_translation(out: &mut [f32; 16], x: f32, y: f32, z: f32) {
    *out = [
        1.0, 0.0, 0.0, 0.0, // column 0
        0.0, 1.0, 0.0, 0.0, // column 1
        0.0, 0.0, 1.0, 0.0, // column 2
        x,   y,   z,   1.0, // column 3
    ];
}

/// Multiply two 4x4 matrices (column-major): out = a * b
fn mat4_multiply(out: &mut [f32; 16], a: &[f32; 16], b: &[f32; 16]) {
    let mut result = [0.0f32; 16];
    for col in 0..4 {
        for row in 0..4 {
            let mut sum = 0.0;
            for k in 0..4 {
                sum += a[k * 4 + row] * b[col * 4 + k];
            }
            result[col * 4 + row] = sum;
        }
    }
    *out = result;
}

/// Generate a cylindrical arm segment mesh with skinning data
/// Returns (vertices, indices) where vertices include bone indices and weights
///
/// The arm is made of 3 segments, each influenced by a bone:
/// - Segment 0: primarily bone 0, blended with bone 1 at the joint
/// - Segment 1: primarily bone 1, blended with bones 0 and 2 at joints
/// - Segment 2: primarily bone 2, blended with bone 1 at the joint
fn generate_arm_mesh() -> ([f32; 360 * 11], [u16; 324]) {
    // Cylinder parameters
    const SEGMENTS: usize = 6;   // Around circumference
    const RINGS: usize = 10;     // Along length (including end caps)
    const RADIUS: f32 = 0.2;
    const TOTAL_LENGTH: f32 = SEGMENT_LENGTH * 3.0;

    // Vertex layout: pos(3) + normal(3) + bone_indices(4u8 as 1f32) + bone_weights(4)
    // Total: 11 floats per vertex
    let mut vertices = [0.0f32; 360 * 11];  // 6 segments * 10 rings * 6 verts = 360 verts
    let mut indices = [0u16; 324];          // (SEGMENTS * (RINGS-1) * 2 * 3) triangles

    let mut v_idx = 0;
    let mut i_idx = 0;

    // Generate vertices for each ring
    for ring in 0..RINGS {
        let y = (ring as f32 / (RINGS - 1) as f32) * TOTAL_LENGTH - TOTAL_LENGTH * 0.5;

        // Determine bone weights based on position along arm
        // Position 0.0 = bone 0, 0.33 = bone 1, 0.66 = bone 2
        let normalized_pos = (y + TOTAL_LENGTH * 0.5) / TOTAL_LENGTH; // 0.0 to 1.0

        // Calculate bone weights with smooth blending
        let bone0_weight: f32;
        let bone1_weight: f32;
        let bone2_weight: f32;

        if normalized_pos < 0.33 {
            // First segment: bone 0 dominant, blend to bone 1
            let blend = normalized_pos / 0.33;
            bone0_weight = 1.0 - blend * 0.5;
            bone1_weight = blend * 0.5;
            bone2_weight = 0.0;
        } else if normalized_pos < 0.66 {
            // Middle segment: bone 1 dominant, blend from 0 and to 2
            let blend = (normalized_pos - 0.33) / 0.33;
            bone0_weight = (1.0 - blend) * 0.3;
            bone1_weight = 0.4 + (1.0 - (2.0 * blend - 1.0).abs()) * 0.3;
            bone2_weight = blend * 0.3;
        } else {
            // Last segment: bone 2 dominant, blend from bone 1
            let blend = (normalized_pos - 0.66) / 0.34;
            bone0_weight = 0.0;
            bone1_weight = (1.0 - blend) * 0.5;
            bone2_weight = 0.5 + blend * 0.5;
        }

        // Normalize weights (should already sum to ~1.0 but ensure it)
        let weight_sum = bone0_weight + bone1_weight + bone2_weight;
        let w0 = bone0_weight / weight_sum;
        let w1 = bone1_weight / weight_sum;
        let w2 = bone2_weight / weight_sum;

        // Pack bone indices as 4 bytes into a u32, then reinterpret as f32
        // Indices: [0, 1, 2, 0] (4th unused, set to 0)
        let bone_indices_packed: u32 = 0 | (1 << 8) | (2 << 16) | (0 << 24);
        let bone_indices_f32 = f32::from_bits(bone_indices_packed);

        for seg in 0..SEGMENTS {
            let angle = (seg as f32 / SEGMENTS as f32) * 6.283185307;
            let nx = cos_approx(angle);
            let nz = sin_approx(angle);
            let x = nx * RADIUS;
            let z = nz * RADIUS;

            // Position
            vertices[v_idx] = x;
            vertices[v_idx + 1] = y;
            vertices[v_idx + 2] = z;

            // Normal (pointing outward from cylinder axis)
            vertices[v_idx + 3] = nx;
            vertices[v_idx + 4] = 0.0;
            vertices[v_idx + 5] = nz;

            // Bone indices (packed as 4 u8 into a single float's bits)
            vertices[v_idx + 6] = bone_indices_f32;

            // Bone weights
            vertices[v_idx + 7] = w0;
            vertices[v_idx + 8] = w1;
            vertices[v_idx + 9] = w2;
            vertices[v_idx + 10] = 0.0; // 4th weight unused

            v_idx += 11;
        }

        // Generate triangles connecting this ring to the next
        if ring < RINGS - 1 {
            let ring_start = (ring * SEGMENTS) as u16;
            let next_ring_start = ((ring + 1) * SEGMENTS) as u16;

            for seg in 0..SEGMENTS {
                let curr = ring_start + seg as u16;
                let next = ring_start + ((seg + 1) % SEGMENTS) as u16;
                let curr_up = next_ring_start + seg as u16;
                let next_up = next_ring_start + ((seg + 1) % SEGMENTS) as u16;

                // Two triangles per quad
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

/// Update bone matrices based on animation time
/// Creates a wave-like bending motion through the arm
fn update_bones(time: f32) {
    unsafe {
        // Each bone applies a rotation then translation
        // Bone 0: base of arm (at origin, rotates around Z)
        // Bone 1: middle joint (offset by segment_length, rotates around Z)
        // Bone 2: end joint (offset by another segment_length, rotates around Z)

        // Calculate rotation angles for each bone (wave motion)
        let angle0 = sin_approx(time) * 0.3;
        let angle1 = sin_approx(time + 1.0) * 0.5;
        let angle2 = sin_approx(time + 2.0) * 0.4;

        // Bone 0: Just rotation at the base
        let mut rot0 = [0.0f32; 16];
        let mut trans0 = [0.0f32; 16];
        mat4_rotation_z(&mut rot0, angle0);
        mat4_translation(&mut trans0, 0.0, -SEGMENT_LENGTH * 1.5, 0.0); // Move to base
        mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr() as *mut [f32; 16]), &rot0, &trans0);

        // Bone 1: Translation + rotation, relative to bone 0
        // First apply bone 0's transform, then translate up, then rotate
        let mut rot1 = [0.0f32; 16];
        let mut trans1 = [0.0f32; 16];
        let mut bone0_final = [0.0f32; 16];
        let mut temp = [0.0f32; 16];

        // Copy bone 0's final transform
        for i in 0..16 {
            bone0_final[i] = BONE_MATRICES[i];
        }

        mat4_translation(&mut trans1, 0.0, SEGMENT_LENGTH, 0.0);
        mat4_rotation_z(&mut rot1, angle1);
        mat4_multiply(&mut temp, &rot1, &trans1);
        mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr().add(16) as *mut [f32; 16]), &bone0_final, &temp);

        // Bone 2: Translation + rotation, relative to bone 1
        let mut bone1_final = [0.0f32; 16];
        let mut rot2 = [0.0f32; 16];
        let mut trans2 = [0.0f32; 16];

        for i in 0..16 {
            bone1_final[i] = BONE_MATRICES[16 + i];
        }

        mat4_translation(&mut trans2, 0.0, SEGMENT_LENGTH, 0.0);
        mat4_rotation_z(&mut rot2, angle2);
        mat4_multiply(&mut temp, &rot2, &trans2);
        mat4_multiply(&mut *(BONE_MATRICES.as_mut_ptr().add(32) as *mut [f32; 16]), &bone1_final, &temp);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x1a1a2eFF);

        // Set up procedural sky for lighting
        set_sky(
            0.5, 0.6, 0.7,      // horizon color
            0.2, 0.4, 0.8,      // zenith color
            0.5, 0.8, 0.3,      // sun direction
            1.5, 1.4, 1.2,      // sun color (HDR)
            200.0,              // sun sharpness
        );

        // Set up camera
        camera_set(0.0, 1.0, 8.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Enable depth testing
        depth_test(1);

        // Generate and load the arm mesh
        let (vertices, indices) = generate_arm_mesh();
        ARM_MESH = load_mesh_indexed(
            vertices.as_ptr(),
            60, // 6 segments * 10 rings = 60 vertices
            indices.as_ptr(),
            324, // index count
            FORMAT_POS_NORMAL_SKINNED,
        );

        // Initialize bone matrices to identity
        for i in 0..NUM_BONES {
            let mut identity = [0.0f32; 16];
            mat4_identity(&mut identity);
            for j in 0..16 {
                BONE_MATRICES[i * 16 + j] = identity[j];
            }
        }
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // View rotation with left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        VIEW_ROTATION_Y += stick_x * 2.0;
        VIEW_ROTATION_X += stick_y * 2.0;

        // Clamp vertical rotation
        if VIEW_ROTATION_X > 89.0 { VIEW_ROTATION_X = 89.0; }
        if VIEW_ROTATION_X < -89.0 { VIEW_ROTATION_X = -89.0; }

        // Toggle pause with A button
        if button_pressed(0, BUTTON_A) != 0 {
            PAUSED = !PAUSED;
        }

        // Adjust animation speed with D-pad
        if button_held(0, BUTTON_UP) != 0 {
            ANIM_SPEED += 0.02;
            if ANIM_SPEED > 3.0 { ANIM_SPEED = 3.0; }
        }
        if button_held(0, BUTTON_DOWN) != 0 {
            ANIM_SPEED -= 0.02;
            if ANIM_SPEED < 0.1 { ANIM_SPEED = 0.1; }
        }

        // Update animation time
        if !PAUSED {
            ANIM_TIME += 0.05 * ANIM_SPEED;
            // Keep time bounded to avoid precision issues
            const TWO_PI: f32 = 6.283185307;
            if ANIM_TIME > TWO_PI * 100.0 {
                ANIM_TIME -= TWO_PI * 100.0;
            }
        }

        // Update bone matrices based on animation
        update_bones(ANIM_TIME);
    }
}

/// Format a float value to string buffer, returns length written
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
        // Upload bone matrices to GPU before drawing
        set_bones(BONE_MATRICES.as_ptr(), NUM_BONES as u32);

        // Apply view rotation
        transform_identity();
        transform_rotate(VIEW_ROTATION_X, 1.0, 0.0, 0.0);
        transform_rotate(VIEW_ROTATION_Y, 0.0, 1.0, 0.0);

        // Draw the arm mesh
        set_color(0xE0C090FF); // Warm skin-like color
        draw_mesh(ARM_MESH);

        // Draw UI
        let y = 10.0;
        let line_h = 14.0;

        let title = b"GPU Skinning Demo";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 12.0, 0xFFFFFFFF);

        let mut buf = [0u8; 32];

        // Animation speed
        let prefix = b"Speed (D-pad): ";
        let len = format_float(ANIM_SPEED, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 10.0, y + line_h, 10.0, 0xCCCCCCFF);

        // Pause status
        let status = if PAUSED { b"Status: PAUSED (A)" as &[u8] } else { b"Status: Playing (A)" as &[u8] };
        draw_text(status.as_ptr(), status.len() as u32, 10.0, y + line_h * 2.0, 10.0, 0xCCCCCCFF);

        // Bone info
        let bones_label = b"3 bones, smooth weight blending";
        draw_text(bones_label.as_ptr(), bones_label.len() as u32, 10.0, y + line_h * 3.5, 9.0, 0x888888FF);

        // Controls hint
        let hint = b"L-Stick: Rotate view";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, y + line_h * 4.5, 8.0, 0x666666FF);
    }
}
