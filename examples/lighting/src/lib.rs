//! Lighting Example
//!
//! Demonstrates Emberware Z's PBR lighting system (render mode 2).
//!
//! Features demonstrated:
//! - `render_mode()` to select PBR rendering (mode 2)
//! - `set_sky()` for procedural sky lighting
//! - `light_set()`, `light_color()`, `light_intensity()` for dynamic lights
//! - `material_metallic()`, `material_roughness()` for PBR materials
//! - Interactive light positioning via analog sticks
//! - Sphere mesh for demonstrating lighting
//!
//! Note: render_mode is init-only (cannot change at runtime).
//! To see other modes, change RENDER_MODE constant and rebuild.
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.
//!
//! Controls:
//! - Left stick: Rotate sphere
//! - Right stick: Move primary light (X/Y)
//! - Triggers: Adjust metallic (LT) and roughness (RT)
//! - D-pad Up/Down: Adjust light intensity
//! - A/B/X/Y: Toggle lights 0-3

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration (init-only)
    fn set_clear_color(color: u32);
    fn render_mode(mode: u32);
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
    fn right_stick_x(player: u32) -> f32;
    fn right_stick_y(player: u32) -> f32;
    fn trigger_left(player: u32) -> f32;
    fn trigger_right(player: u32) -> f32;
    fn button_pressed(player: u32, button: u32) -> u32;
    fn button_held(player: u32, button: u32) -> u32;

    // Lighting (Mode 2/3)
    fn light_set(index: u32, x: f32, y: f32, z: f32);
    fn light_color(index: u32, r: f32, g: f32, b: f32);
    fn light_intensity(index: u32, intensity: f32);
    fn light_disable(index: u32);

    // Materials (Mode 2/3)
    fn material_metallic(value: f32);
    fn material_roughness(value: f32);

    // Mesh
    fn load_mesh_indexed(
        data: *const f32,
        vertex_count: u32,
        indices: *const u16,
        index_count: u32,
        format: u32,
    ) -> u32;
    fn draw_mesh(handle: u32);

    // Transform
    fn transform_identity();
    fn transform_rotate(angle_deg: f32, x: f32, y: f32, z: f32);

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);

    // 2D UI
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);
}

/// Fast inverse square root (Quake III style)
/// Good enough for normalizing vectors
fn fast_inv_sqrt(x: f32) -> f32 {
    let half_x = 0.5 * x;
    let i = x.to_bits();
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    y * (1.5 - half_x * y * y) // One Newton-Raphson iteration
}

// Button constants
const BUTTON_UP: u32 = 0;
const BUTTON_DOWN: u32 = 1;
const BUTTON_A: u32 = 4;
const BUTTON_B: u32 = 5;
const BUTTON_X: u32 = 6;
const BUTTON_Y: u32 = 7;

/// Vertex format: POS_NORMAL = 4 (position + normal, 6 floats per vertex)
const FORMAT_POS_NORMAL: u32 = 4;

/// Render mode: 0=Unlit, 1=Matcap, 2=PBR, 3=Hybrid
/// Change this and rebuild to see different modes
const RENDER_MODE: u32 = 2;

/// Sphere mesh handle
static mut SPHERE_MESH: u32 = 0;

/// Current rotation angles (degrees)
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Light positions (normalized direction vectors)
static mut LIGHT_DIRS: [[f32; 3]; 4] = [
    [0.5, 0.8, 0.3],   // Light 0: upper-right-front (main)
    [-0.7, 0.3, 0.5],  // Light 1: upper-left
    [0.3, -0.5, 0.7],  // Light 2: lower-front
    [-0.3, 0.6, -0.5], // Light 3: upper-back
];

/// Light enabled states
static mut LIGHT_ENABLED: [bool; 4] = [true, true, false, false];

/// Light colors (RGB)
static LIGHT_COLORS: [[f32; 3]; 4] = [
    [1.0, 0.95, 0.9],  // Light 0: warm white
    [0.6, 0.7, 1.0],   // Light 1: cool blue
    [1.0, 0.7, 0.5],   // Light 2: orange
    [0.7, 1.0, 0.7],   // Light 3: green
];

/// Light intensity
static mut LIGHT_INTENSITY: f32 = 1.5;

/// Material properties
static mut METALLIC: f32 = 0.0;
static mut ROUGHNESS: f32 = 0.3;

// UV sphere generation (procedural at compile time isn't practical, so we generate at runtime)
// For simplicity, use a predefined low-poly sphere (icosphere-style)

/// Simple icosphere vertices (12 vertices, 20 faces)
/// Each vertex: [x, y, z, nx, ny, nz] (position = normal for unit sphere)
static ICOSPHERE_VERTS: [f32; 12 * 6] = {
    // Normalize factor for (1, PHI, 0): sqrt(1 + PHI^2) ≈ 1.902
    // Pre-normalized coordinates
    const N: f32 = 0.5257311; // 1 / sqrt(1 + PHI^2)
    const P: f32 = 0.8506508; // PHI / sqrt(1 + PHI^2)

    [
        // Vertex 0-4: top "cap"
        -N,  P,  0.0,  -N,  P,  0.0,
         N,  P,  0.0,   N,  P,  0.0,
        -N, -P,  0.0,  -N, -P,  0.0,
         N, -P,  0.0,   N, -P,  0.0,

        // Vertex 4-7: middle ring 1
         0.0, -N,  P,   0.0, -N,  P,
         0.0,  N,  P,   0.0,  N,  P,
         0.0, -N, -P,   0.0, -N, -P,
         0.0,  N, -P,   0.0,  N, -P,

        // Vertex 8-11: middle ring 2
         P,  0.0, -N,   P,  0.0, -N,
         P,  0.0,  N,   P,  0.0,  N,
        -P,  0.0, -N,  -P,  0.0, -N,
        -P,  0.0,  N,  -P,  0.0,  N,
    ]
};

/// Icosphere faces (20 triangles, 60 indices)
static ICOSPHERE_INDICES: [u16; 60] = [
    // 5 faces around vertex 0
    0, 11, 5,
    0, 5, 1,
    0, 1, 7,
    0, 7, 10,
    0, 10, 11,
    // 5 adjacent faces
    1, 5, 9,
    5, 11, 4,
    11, 10, 2,
    10, 7, 6,
    7, 1, 8,
    // 5 faces around vertex 3
    3, 9, 4,
    3, 4, 2,
    3, 2, 6,
    3, 6, 8,
    3, 8, 9,
    // 5 adjacent faces
    4, 9, 5,
    2, 4, 11,
    6, 2, 10,
    8, 6, 7,
    9, 8, 1,
];

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background (visible behind sky)
        set_clear_color(0x101020FF);

        // Set render mode (PBR)
        render_mode(RENDER_MODE);

        // Set up procedural sky
        // Midday sky with warm sun for nice PBR lighting
        set_sky(
            0.5, 0.6, 0.7,      // horizon color (light blue-gray)
            0.2, 0.4, 0.8,      // zenith color (deeper blue)
            0.5, 0.8, 0.3,      // sun direction (normalized)
            1.5, 1.4, 1.2,      // sun color (warm white, HDR)
            200.0,              // sun sharpness
        );

        // Set up camera
        camera_set(0.0, 0.0, 4.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Enable depth testing
        depth_test(1);

        // Load the sphere mesh
        SPHERE_MESH = load_mesh_indexed(
            ICOSPHERE_VERTS.as_ptr(),
            12,
            ICOSPHERE_INDICES.as_ptr(),
            60,
            FORMAT_POS_NORMAL,
        );

        // Initialize lights
        for i in 0..4u32 {
            let dir = LIGHT_DIRS[i as usize];
            light_set(i, dir[0], dir[1], dir[2]);
            let color = LIGHT_COLORS[i as usize];
            light_color(i, color[0], color[1], color[2]);
            light_intensity(i, LIGHT_INTENSITY);
            if !LIGHT_ENABLED[i as usize] {
                light_disable(i);
            }
        }

        // Set initial material
        material_metallic(METALLIC);
        material_roughness(ROUGHNESS);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Rotate sphere with left stick
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);
        ROTATION_Y += stick_x * 2.0;
        ROTATION_X += stick_y * 2.0;

        // Auto-rotate when stick is centered
        if stick_x.abs() < 0.1 && stick_y.abs() < 0.1 {
            ROTATION_Y += 0.3;
        }

        // Move primary light with right stick
        let right_x = right_stick_x(0);
        let right_y = right_stick_y(0);
        if right_x.abs() > 0.1 || right_y.abs() > 0.1 {
            LIGHT_DIRS[0][0] += right_x * 0.02;
            LIGHT_DIRS[0][1] += right_y * 0.02;
            // Normalize using fast inverse sqrt approximation
            let len_sq = LIGHT_DIRS[0][0] * LIGHT_DIRS[0][0]
                + LIGHT_DIRS[0][1] * LIGHT_DIRS[0][1]
                + LIGHT_DIRS[0][2] * LIGHT_DIRS[0][2];
            if len_sq > 0.0001 {
                let inv_len = fast_inv_sqrt(len_sq);
                LIGHT_DIRS[0][0] *= inv_len;
                LIGHT_DIRS[0][1] *= inv_len;
                LIGHT_DIRS[0][2] *= inv_len;
            }
            light_set(0, LIGHT_DIRS[0][0], LIGHT_DIRS[0][1], LIGHT_DIRS[0][2]);
        }

        // Adjust metallic with left trigger
        let lt = trigger_left(0);
        if lt > 0.1 {
            METALLIC = lt;
            material_metallic(METALLIC);
        }

        // Adjust roughness with right trigger
        let rt = trigger_right(0);
        if rt > 0.1 {
            ROUGHNESS = rt;
            material_roughness(ROUGHNESS);
        }

        // Adjust intensity with D-pad
        if button_held(0, BUTTON_UP) != 0 {
            LIGHT_INTENSITY += 0.02;
            if LIGHT_INTENSITY > 5.0 {
                LIGHT_INTENSITY = 5.0;
            }
            for i in 0..4u32 {
                if LIGHT_ENABLED[i as usize] {
                    light_intensity(i, LIGHT_INTENSITY);
                }
            }
        }
        if button_held(0, BUTTON_DOWN) != 0 {
            LIGHT_INTENSITY -= 0.02;
            if LIGHT_INTENSITY < 0.0 {
                LIGHT_INTENSITY = 0.0;
            }
            for i in 0..4u32 {
                if LIGHT_ENABLED[i as usize] {
                    light_intensity(i, LIGHT_INTENSITY);
                }
            }
        }

        // Toggle lights with face buttons
        if button_pressed(0, BUTTON_A) != 0 {
            LIGHT_ENABLED[0] = !LIGHT_ENABLED[0];
            if LIGHT_ENABLED[0] {
                light_intensity(0, LIGHT_INTENSITY);
            } else {
                light_disable(0);
            }
        }
        if button_pressed(0, BUTTON_B) != 0 {
            LIGHT_ENABLED[1] = !LIGHT_ENABLED[1];
            if LIGHT_ENABLED[1] {
                light_intensity(1, LIGHT_INTENSITY);
            } else {
                light_disable(1);
            }
        }
        if button_pressed(0, BUTTON_X) != 0 {
            LIGHT_ENABLED[2] = !LIGHT_ENABLED[2];
            if LIGHT_ENABLED[2] {
                light_intensity(2, LIGHT_INTENSITY);
            } else {
                light_disable(2);
            }
        }
        if button_pressed(0, BUTTON_Y) != 0 {
            LIGHT_ENABLED[3] = !LIGHT_ENABLED[3];
            if LIGHT_ENABLED[3] {
                light_intensity(3, LIGHT_INTENSITY);
            } else {
                light_disable(3);
            }
        }
    }
}

/// Simple integer to string conversion for displaying values
fn format_float(val: f32, buf: &mut [u8]) -> usize {
    // Format as X.XX
    let whole = val as i32;
    let frac = ((val - whole as f32).abs() * 100.0) as i32;

    let mut i = 0;
    if whole < 0 {
        buf[i] = b'-';
        i += 1;
    }
    let whole_abs = if whole < 0 { -whole } else { whole };

    // Write whole part
    if whole_abs >= 10 {
        buf[i] = b'0' + (whole_abs / 10) as u8;
        i += 1;
    }
    buf[i] = b'0' + (whole_abs % 10) as u8;
    i += 1;

    buf[i] = b'.';
    i += 1;

    // Write fractional part
    buf[i] = b'0' + (frac / 10) as u8;
    i += 1;
    buf[i] = b'0' + (frac % 10) as u8;
    i += 1;

    i
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Draw the sphere
        transform_identity();
        transform_rotate(ROTATION_X, 1.0, 0.0, 0.0);
        transform_rotate(ROTATION_Y, 0.0, 1.0, 0.0);

        set_color(0xFFFFFFFF);
        draw_mesh(SPHERE_MESH);

        // Draw UI overlay
        let y = 10.0;
        let line_h = 16.0;

        // Mode indicator
        let mode_text = match RENDER_MODE {
            0 => b"Mode 0: Unlit" as &[u8],
            1 => b"Mode 1: Matcap" as &[u8],
            2 => b"Mode 2: PBR" as &[u8],
            3 => b"Mode 3: Hybrid" as &[u8],
            _ => b"Unknown Mode" as &[u8],
        };
        draw_text(mode_text.as_ptr(), mode_text.len() as u32, 10.0, y, 12.0, 0xFFFFFFFF);

        // Material properties
        let mut buf = [0u8; 32];

        // Metallic
        let prefix = b"Metallic (LT): ";
        let len = format_float(METALLIC, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 10.0, y + line_h, 10.0, 0xCCCCCCFF);

        // Roughness
        let prefix = b"Roughness (RT): ";
        let len = format_float(ROUGHNESS, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 10.0, y + line_h * 2.0, 10.0, 0xCCCCCCFF);

        // Intensity
        let prefix = b"Intensity (D-pad): ";
        let len = format_float(LIGHT_INTENSITY, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 10.0, y + line_h * 3.0, 10.0, 0xCCCCCCFF);

        // Light status
        let lights_label = b"Lights (A/B/X/Y):";
        draw_text(lights_label.as_ptr(), lights_label.len() as u32, 10.0, y + line_h * 4.5, 10.0, 0xCCCCCCFF);

        // Draw light indicators
        for i in 0..4 {
            let x = 10.0 + (i as f32) * 25.0;
            let color = if LIGHT_ENABLED[i] {
                // Convert light color to packed format
                let r = (LIGHT_COLORS[i][0] * 255.0) as u32;
                let g = (LIGHT_COLORS[i][1] * 255.0) as u32;
                let b = (LIGHT_COLORS[i][2] * 255.0) as u32;
                (r << 24) | (g << 16) | (b << 8) | 0xFF
            } else {
                0x404040FF // Dim gray when off
            };
            draw_rect(x, y + line_h * 5.5, 20.0, 12.0, color);
        }

        // Controls hint
        let hint = b"L-Stick: Rotate  R-Stick: Move Light";
        draw_text(hint.as_ptr(), hint.len() as u32, 10.0, y + line_h * 7.0, 8.0, 0x888888FF);
    }
}
