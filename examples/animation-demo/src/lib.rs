//! Animation Demo - ROM-Backed Keyframe Animation System (Animation System v2)
//!
//! Demonstrates Emberware ZX's keyframe animation system:
//! - Loading keyframes from ROM data pack (no WASM memory used)
//! - Step-based "stamp" animation with keyframe_bind() - uses static GPU buffers
//! - Blended animation with keyframe_read() + set_bones_4x4() - uses immediate bones buffer
//!
//! Animation System v2 Features:
//! - All keyframe data uploaded to GPU once during init (no per-frame decode/upload)
//! - keyframe_bind() sets an offset into static all_keyframes buffer
//! - set_bones_4x4() appends to per-frame immediate_bones buffer
//! - Both paths are GPU-optimized with minimal CPU overhead
//!
//! Controls:
//! - A button: Toggle between stamp and blended mode
//! - D-pad Up/Down: Adjust playback speed
//! - Left stick: Rotate view

#![no_std]
#![no_main]

use core::panic::PanicInfo;
use core::ptr::addr_of_mut;
use examples_common::{DebugCamera, StickControl};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Imports
// ============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(x: f32, y: f32, z: f32, target_x: f32, target_y: f32, target_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Input
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

    // Keyframe Animation (NEW!)
    fn rom_keyframes(id_ptr: *const u8, id_len: u32) -> u32;
    fn keyframes_bone_count(handle: u32) -> u8;
    fn keyframes_frame_count(handle: u32) -> u16;
    fn keyframe_read(handle: u32, index: u32, out_ptr: *mut u8);
    fn keyframe_bind(handle: u32, index: u32);

    // Skinning
    fn set_bones(matrices_ptr: *const f32, count: u32);
    fn set_bones_4x4(matrices_ptr: *const f32, count: u32);

    // Transform
    fn push_identity();

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // 2D UI
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// ============================================================================
// Constants
// ============================================================================

const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;

/// Vertex format: POS_NORMAL_SKINNED
const FORMAT_NORMAL: u32 = 4;
const FORMAT_SKINNED: u32 = 8;
const FORMAT_POS_NORMAL_SKINNED: u32 = FORMAT_NORMAL | FORMAT_SKINNED;

/// Number of bones in our test animation
const NUM_BONES: usize = 3;

/// Floats per 3x4 bone matrix
const BONE_MATRIX_3X4_FLOATS: usize = 12;

/// Floats per 4x4 bone matrix
const BONE_MATRIX_4X4_FLOATS: usize = 16;

/// Size of BoneTransform struct (40 bytes)
/// Layout: rotation[4] + position[3] + scale[3] as f32
const BONE_TRANSFORM_SIZE: usize = 40;

/// Arm segment length
const SEGMENT_LENGTH: f32 = 1.5;

// ============================================================================
// Game State
// ============================================================================

/// Mesh handle for the arm
static mut ARM_MESH: u32 = 0;

/// Keyframe animation handle
static mut WAVE_ANIM: u32 = 0;

/// Frame count from animation
static mut FRAME_COUNT: u16 = 0;

/// Camera for orbit control
static mut CAMERA: DebugCamera = DebugCamera {
    target_x: 0.0,
    target_y: 0.0,
    target_z: 0.0,
    distance: 8.0,
    elevation: 20.0,
    azimuth: 0.0,
    auto_orbit_speed: 0.0,
    stick_control: StickControl::LeftStick,
    fov: 60.0,
};

/// Animation time (fractional frame)
static mut ANIM_TIME: f32 = 0.0;

/// Playback speed multiplier
static mut ANIM_SPEED: f32 = 1.0;

/// Animation mode: false = stamp (keyframe_bind), true = blended (keyframe_read)
static mut BLENDED_MODE: bool = false;

/// Keyframe buffers for blending (2 frames × 3 bones × 40 bytes = 240 bytes)
static mut KEYFRAME_BUF_A: [u8; NUM_BONES * BONE_TRANSFORM_SIZE] = [0; NUM_BONES * BONE_TRANSFORM_SIZE];
static mut KEYFRAME_BUF_B: [u8; NUM_BONES * BONE_TRANSFORM_SIZE] = [0; NUM_BONES * BONE_TRANSFORM_SIZE];

/// Output bone matrices for blended mode (3 bones × 16 floats = 48 floats)
static mut BONE_MATRICES_4X4: [f32; NUM_BONES * BONE_MATRIX_4X4_FLOATS] = [0.0; NUM_BONES * BONE_MATRIX_4X4_FLOATS];

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

#[inline]
fn sqrt(x: f32) -> f32 {
    libm::sqrtf(x)
}

fn fract(x: f32) -> f32 {
    x - libm::floorf(x)
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Normalized linear interpolation for quaternions
fn nlerp(a: &[f32; 4], b: &[f32; 4], t: f32) -> [f32; 4] {
    // Check if quaternions are on opposite hemispheres
    let dot = a[0] * b[0] + a[1] * b[1] + a[2] * b[2] + a[3] * b[3];
    let sign = if dot < 0.0 { -1.0 } else { 1.0 };

    let result = [
        lerp(a[0], b[0] * sign, t),
        lerp(a[1], b[1] * sign, t),
        lerp(a[2], b[2] * sign, t),
        lerp(a[3], b[3] * sign, t),
    ];

    // Normalize
    let len = sqrt(
        result[0] * result[0]
            + result[1] * result[1]
            + result[2] * result[2]
            + result[3] * result[3],
    );
    [
        result[0] / len,
        result[1] / len,
        result[2] / len,
        result[3] / len,
    ]
}

// ============================================================================
// BoneTransform Parsing
// ============================================================================

/// BoneTransform from keyframe_read (40 bytes)
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

    fn lerp(&self, other: &Self, t: f32) -> Self {
        Self {
            rotation: nlerp(&self.rotation, &other.rotation, t),
            position: [
                lerp(self.position[0], other.position[0], t),
                lerp(self.position[1], other.position[1], t),
                lerp(self.position[2], other.position[2], t),
            ],
            scale: [
                lerp(self.scale[0], other.scale[0], t),
                lerp(self.scale[1], other.scale[1], t),
                lerp(self.scale[2], other.scale[2], t),
            ],
        }
    }

    /// Build 4x4 column-major matrix from TRS
    fn to_matrix_4x4(&self) -> [f32; 16] {
        let [qx, qy, qz, qw] = self.rotation;
        let [px, py, pz] = self.position;
        let [sx, sy, sz] = self.scale;

        let xx = qx * qx;
        let yy = qy * qy;
        let zz = qz * qz;
        let xy = qx * qy;
        let xz = qx * qz;
        let yz = qy * qz;
        let wx = qw * qx;
        let wy = qw * qy;
        let wz = qw * qz;

        [
            // Column 0 (scaled by sx)
            sx * (1.0 - 2.0 * (yy + zz)),
            sx * (2.0 * (xy + wz)),
            sx * (2.0 * (xz - wy)),
            0.0,
            // Column 1 (scaled by sy)
            sy * (2.0 * (xy - wz)),
            sy * (1.0 - 2.0 * (xx + zz)),
            sy * (2.0 * (yz + wx)),
            0.0,
            // Column 2 (scaled by sz)
            sz * (2.0 * (xz + wy)),
            sz * (2.0 * (yz - wx)),
            sz * (1.0 - 2.0 * (xx + yy)),
            0.0,
            // Column 3 (translation)
            px,
            py,
            pz,
            1.0,
        ]
    }
}

// ============================================================================
// Mesh Generation
// ============================================================================

/// Generate a cylindrical arm mesh with skinning data
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
            (
                (1.0 - blend) * 0.3,
                0.4 + (1.0 - (2.0 * blend - 1.0).abs()) * 0.3,
                blend * 0.3,
            )
        } else {
            let blend = (normalized_pos - 0.66) / 0.34;
            (0.0, (1.0 - blend) * 0.5, 0.5 + blend * 0.5)
        };

        let weight_sum = bone0_weight + bone1_weight + bone2_weight;
        let (w0, w1, w2) = (
            bone0_weight / weight_sum,
            bone1_weight / weight_sum,
            bone2_weight / weight_sum,
        );

        // Pack bone indices as 4 bytes into u32
        let bone_indices_packed: u32 = 0 | (1 << 8) | (2 << 16) | (0 << 24);
        let bone_indices_f32 = f32::from_bits(bone_indices_packed);

        for seg in 0..SEGMENTS {
            let angle = (seg as f32 / SEGMENTS as f32) * 6.283185307;
            let (nx, nz) = (cos(angle), sin(angle));
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

                // Counter-clockwise winding when viewed from outside
                indices[i_idx] = curr;
                indices[i_idx + 1] = curr_up;
                indices[i_idx + 2] = next;
                indices[i_idx + 3] = next;
                indices[i_idx + 4] = curr_up;
                indices[i_idx + 5] = next_up;
                i_idx += 6;
            }
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
        depth_test(1);

        // Load the arm mesh
        let (vertices, indices) = generate_arm_mesh();
        ARM_MESH = load_mesh_indexed(
            vertices.as_ptr(),
            60,
            indices.as_ptr(),
            324,
            FORMAT_POS_NORMAL_SKINNED,
        );

        // Load keyframes from ROM data pack
        let wave_id = b"wave";
        WAVE_ANIM = rom_keyframes(wave_id.as_ptr(), wave_id.len() as u32);

        // Query animation info
        FRAME_COUNT = keyframes_frame_count(WAVE_ANIM);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Update camera
        CAMERA.update();

        // Toggle animation mode
        if button_pressed(0, BUTTON_A) != 0 {
            BLENDED_MODE = !BLENDED_MODE;
        }

        // Adjust playback speed
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

        // Advance animation time
        ANIM_TIME += 0.5 * ANIM_SPEED;
        if ANIM_TIME >= FRAME_COUNT as f32 {
            ANIM_TIME -= FRAME_COUNT as f32;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Apply camera
        CAMERA.apply();

        let frame_count = FRAME_COUNT as u32;

        if BLENDED_MODE {
            // Blended mode: interpolate between frames
            let frame_a = ANIM_TIME as u32 % frame_count;
            let frame_b = (frame_a + 1) % frame_count;
            let blend = fract(ANIM_TIME);

            // Read keyframes into WASM memory
            let buf_a = &mut *addr_of_mut!(KEYFRAME_BUF_A);
            let buf_b = &mut *addr_of_mut!(KEYFRAME_BUF_B);
            keyframe_read(WAVE_ANIM, frame_a, buf_a.as_mut_ptr());
            keyframe_read(WAVE_ANIM, frame_b, buf_b.as_mut_ptr());

            // Blend and build matrices
            let matrices = &mut *addr_of_mut!(BONE_MATRICES_4X4);
            for i in 0..NUM_BONES {
                let offset = i * BONE_TRANSFORM_SIZE;
                let transform_a = BoneTransform::from_bytes(&buf_a[offset..offset + BONE_TRANSFORM_SIZE]);
                let transform_b = BoneTransform::from_bytes(&buf_b[offset..offset + BONE_TRANSFORM_SIZE]);
                let blended = transform_a.lerp(&transform_b, blend);
                let mat = blended.to_matrix_4x4();
                matrices[i * 16..(i + 1) * 16].copy_from_slice(&mat);
            }

            // Upload 4x4 matrices to GPU
            set_bones_4x4(matrices.as_ptr(), NUM_BONES as u32);
        } else {
            // Stamp mode: direct keyframe binding (no WASM memory used!)
            let frame = ANIM_TIME as u32 % frame_count;
            keyframe_bind(WAVE_ANIM, frame);
        }

        // Draw the mesh
        push_identity();
        set_color(0xE0C090FF);
        draw_mesh(ARM_MESH);

        // Draw UI
        draw_ui();
    }
}

fn draw_ui() {
    unsafe {
        let y = 10.0;
        let line_h = 18.0;

        let title = b"Animation Demo";
        draw_text(title.as_ptr(), title.len() as u32, 10.0, y, 16.0, 0xFFFFFFFF);

        // Mode indicator
        let mode_text = if BLENDED_MODE {
            b"Mode: BLENDED (keyframe_read)" as &[u8]
        } else {
            b"Mode: STAMP (keyframe_bind)" as &[u8]
        };
        draw_text(
            mode_text.as_ptr(),
            mode_text.len() as u32,
            10.0,
            y + line_h,
            12.0,
            if BLENDED_MODE { 0x90EE90FF } else { 0xFFB6C1FF },
        );

        // Frame info
        let frame = ANIM_TIME as u32 % FRAME_COUNT as u32;
        let mut buf = [0u8; 32];
        let prefix = b"Frame: ";
        buf[..prefix.len()].copy_from_slice(prefix);
        let len = format_int(frame as i32, &mut buf[prefix.len()..]);
        draw_text(
            buf.as_ptr(),
            (prefix.len() + len) as u32,
            10.0,
            y + line_h * 2.0,
            12.0,
            0xCCCCCCFF,
        );

        // Speed
        let prefix = b"Speed: ";
        buf[..prefix.len()].copy_from_slice(prefix);
        let len = format_float(ANIM_SPEED, &mut buf[prefix.len()..]);
        draw_text(
            buf.as_ptr(),
            (prefix.len() + len) as u32,
            10.0,
            y + line_h * 3.0,
            12.0,
            0xCCCCCCFF,
        );

        // Controls
        let ctrl1 = b"A: Toggle mode";
        draw_text(
            ctrl1.as_ptr(),
            ctrl1.len() as u32,
            10.0,
            y + line_h * 4.5,
            10.0,
            0x888888FF,
        );
        let ctrl2 = b"D-pad: Speed";
        draw_text(
            ctrl2.as_ptr(),
            ctrl2.len() as u32,
            10.0,
            y + line_h * 5.5,
            10.0,
            0x888888FF,
        );
    }
}

fn format_int(val: i32, buf: &mut [u8]) -> usize {
    let mut i = 0;
    let mut n = val;
    if n < 0 {
        buf[i] = b'-';
        i += 1;
        n = -n;
    }
    if n >= 10 {
        buf[i] = b'0' + (n / 10) as u8;
        i += 1;
    }
    buf[i] = b'0' + (n % 10) as u8;
    i + 1
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
