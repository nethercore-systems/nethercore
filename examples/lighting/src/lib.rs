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

/// Subdivision level for icosphere (0 = 12 verts, 1 = 42 verts, 2 = 162 verts, 3 = 642 verts)
const SUBDIVISION_LEVEL: usize = 2;

// Maximum vertices and indices for level 3 subdivision
const MAX_VERTS: usize = 642 * 6; // 642 vertices * 6 floats (pos + normal)
const MAX_INDICES: usize = 1280 * 3; // 1280 triangles * 3 indices

/// Subdivided icosphere vertex buffer (populated at init)
static mut SUBDIVIDED_VERTS: [f32; MAX_VERTS] = [0.0; MAX_VERTS];
static mut SUBDIVIDED_INDICES: [u16; MAX_INDICES] = [0; MAX_INDICES];
static mut SUBDIVIDED_VERT_COUNT: usize = 0;
static mut SUBDIVIDED_INDEX_COUNT: usize = 0;

/// Base icosphere vertices (12 vertices, 20 faces)
/// Each vertex: [x, y, z, nx, ny, nz] (position = normal for unit sphere)
static BASE_ICOSPHERE_VERTS: [f32; 12 * 6] = {
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

/// Base icosphere faces (20 triangles, 60 indices)
static BASE_ICOSPHERE_INDICES: [u16; 60] = [
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

/// Subdivide icosphere by splitting each triangle into 4 smaller triangles
/// and projecting new vertices onto the unit sphere
fn subdivide_icosphere(level: usize) {
    unsafe {
        // Start with base icosphere
        let mut verts = [0.0f32; MAX_VERTS];
        let mut indices = [0u16; MAX_INDICES];

        // Copy base vertices (position + normal)
        for i in 0..12 {
            for j in 0..6 {
                verts[i * 6 + j] = BASE_ICOSPHERE_VERTS[i * 6 + j];
            }
        }
        let mut vert_count = 12;

        // Copy base indices
        for i in 0..60 {
            indices[i] = BASE_ICOSPHERE_INDICES[i];
        }
        let mut index_count = 60;

        // Perform subdivision
        for _ in 0..level {
            let old_index_count = index_count;
            let old_vert_count = vert_count;

            // Create temporary storage for new triangles
            let mut new_indices = [0u16; MAX_INDICES];
            let mut new_index_count = 0;

            // Process each triangle
            let mut tri_idx = 0;
            while tri_idx < old_index_count {
                let i0 = indices[tri_idx] as usize;
                let i1 = indices[tri_idx + 1] as usize;
                let i2 = indices[tri_idx + 2] as usize;

                // Get vertex positions
                let v0 = [verts[i0 * 6], verts[i0 * 6 + 1], verts[i0 * 6 + 2]];
                let v1 = [verts[i1 * 6], verts[i1 * 6 + 1], verts[i1 * 6 + 2]];
                let v2 = [verts[i2 * 6], verts[i2 * 6 + 1], verts[i2 * 6 + 2]];

                // Calculate midpoints
                let m01 = normalize([
                    (v0[0] + v1[0]) * 0.5,
                    (v0[1] + v1[1]) * 0.5,
                    (v0[2] + v1[2]) * 0.5,
                ]);
                let m12 = normalize([
                    (v1[0] + v2[0]) * 0.5,
                    (v1[1] + v2[1]) * 0.5,
                    (v1[2] + v2[2]) * 0.5,
                ]);
                let m20 = normalize([
                    (v2[0] + v0[0]) * 0.5,
                    (v2[1] + v0[1]) * 0.5,
                    (v2[2] + v0[2]) * 0.5,
                ]);

                // Find or create vertex indices for midpoints
                let i01 = find_or_add_vertex(&mut verts, &mut vert_count, m01);
                let i12 = find_or_add_vertex(&mut verts, &mut vert_count, m12);
                let i20 = find_or_add_vertex(&mut verts, &mut vert_count, m20);

                // Create 4 new triangles
                // Center triangle
                new_indices[new_index_count] = i01;
                new_indices[new_index_count + 1] = i12;
                new_indices[new_index_count + 2] = i20;
                new_index_count += 3;

                // Corner triangle 0
                new_indices[new_index_count] = i0 as u16;
                new_indices[new_index_count + 1] = i01;
                new_indices[new_index_count + 2] = i20;
                new_index_count += 3;

                // Corner triangle 1
                new_indices[new_index_count] = i1 as u16;
                new_indices[new_index_count + 1] = i12;
                new_indices[new_index_count + 2] = i01;
                new_index_count += 3;

                // Corner triangle 2
                new_indices[new_index_count] = i2 as u16;
                new_indices[new_index_count + 1] = i20;
                new_indices[new_index_count + 2] = i12;
                new_index_count += 3;

                tri_idx += 3;
            }

            // Copy new indices back
            for i in 0..new_index_count {
                indices[i] = new_indices[i];
            }
            index_count = new_index_count;
            vert_count = old_vert_count + (new_index_count - old_index_count) / 2; // Rough estimate
        }

        // Copy results to global storage
        for i in 0..vert_count * 6 {
            SUBDIVIDED_VERTS[i] = verts[i];
        }
        for i in 0..index_count {
            SUBDIVIDED_INDICES[i] = indices[i];
        }
        SUBDIVIDED_VERT_COUNT = vert_count;
        SUBDIVIDED_INDEX_COUNT = index_count;
    }
}

/// Normalize a vector to unit length
fn normalize(v: [f32; 3]) -> [f32; 3] {
    let len_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len_sq > 0.0001 {
        let inv_len = fast_inv_sqrt(len_sq);
        [v[0] * inv_len, v[1] * inv_len, v[2] * inv_len]
    } else {
        v
    }
}

/// Find existing vertex or add a new one (position = normal for unit sphere)
fn find_or_add_vertex(verts: &mut [f32], vert_count: &mut usize, pos: [f32; 3]) -> u16 {
    // For simplicity, always add new vertex (edge sharing handled by proximity check)
    // In production, use a hashmap for exact deduplication
    const EPSILON: f32 = 0.0001;

    // Check if vertex already exists (simple linear search)
    for i in 0..*vert_count {
        let vx = verts[i * 6];
        let vy = verts[i * 6 + 1];
        let vz = verts[i * 6 + 2];
        let dx = vx - pos[0];
        let dy = vy - pos[1];
        let dz = vz - pos[2];
        if dx * dx + dy * dy + dz * dz < EPSILON {
            return i as u16;
        }
    }

    // Add new vertex
    let idx = *vert_count;
    verts[idx * 6] = pos[0];
    verts[idx * 6 + 1] = pos[1];
    verts[idx * 6 + 2] = pos[2];
    verts[idx * 6 + 3] = pos[0]; // Normal = position for unit sphere
    verts[idx * 6 + 4] = pos[1];
    verts[idx * 6 + 5] = pos[2];
    *vert_count += 1;
    idx as u16
}

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

        // Generate subdivided icosphere
        subdivide_icosphere(SUBDIVISION_LEVEL);

        // Load the sphere mesh
        SPHERE_MESH = load_mesh_indexed(
            SUBDIVIDED_VERTS.as_ptr(),
            SUBDIVIDED_VERT_COUNT as u32,
            SUBDIVIDED_INDICES.as_ptr(),
            SUBDIVIDED_INDEX_COUNT as u32,
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
                light_enable(0);
            } else {
                light_disable(0);
            }
        }
        if button_pressed(0, BUTTON_B) != 0 {
            LIGHT_ENABLED[1] = !LIGHT_ENABLED[1];
            if LIGHT_ENABLED[1] {
                light_enable(1);
            } else {
                light_disable(1);
            }
        }
        if button_pressed(0, BUTTON_X) != 0 {
            LIGHT_ENABLED[2] = !LIGHT_ENABLED[2];
            if LIGHT_ENABLED[2] {
                light_enable(2);
            } else {
                light_disable(2);
            }
        }
        if button_pressed(0, BUTTON_Y) != 0 {
            LIGHT_ENABLED[3] = !LIGHT_ENABLED[3];
            if LIGHT_ENABLED[3] {
                light_enable(3);
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
        let y = 20.0;
        let line_h = 50.0;

        // Mode indicator
        let mode_text = match RENDER_MODE {
            0 => b"Mode 0: Unlit" as &[u8],
            1 => b"Mode 1: Matcap" as &[u8],
            2 => b"Mode 2: PBR" as &[u8],
            3 => b"Mode 3: Hybrid" as &[u8],
            _ => b"Unknown Mode" as &[u8],
        };
        draw_text(mode_text.as_ptr(), mode_text.len() as u32, 20.0, y, 48.0, 0xFFFFFFFF);

        // Material properties
        let mut buf = [0u8; 32];

        // Metallic
        let prefix = b"Metallic (LT): ";
        let len = format_float(METALLIC, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 20.0, y + line_h, 40.0, 0xCCCCCCFF);

        // Roughness
        let prefix = b"Roughness (RT): ";
        let len = format_float(ROUGHNESS, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 20.0, y + line_h * 2.0, 40.0, 0xCCCCCCFF);

        // Intensity
        let prefix = b"Intensity (D-pad): ";
        let len = format_float(LIGHT_INTENSITY, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 20.0, y + line_h * 3.0, 40.0, 0xCCCCCCFF);

        // Light status
        let lights_label = b"Lights (A/B/X/Y):";
        draw_text(lights_label.as_ptr(), lights_label.len() as u32, 20.0, y + line_h * 4.5, 40.0, 0xCCCCCCFF);

        // Draw light indicators
        for i in 0..4 {
            let x = 20.0 + (i as f32) * 50.0;
            let color = if LIGHT_ENABLED[i] {
                // Convert light color to packed format
                let r = (LIGHT_COLORS[i][0] * 255.0) as u32;
                let g = (LIGHT_COLORS[i][1] * 255.0) as u32;
                let b = (LIGHT_COLORS[i][2] * 255.0) as u32;
                (r << 24) | (g << 16) | (b << 8) | 0xFF
            } else {
                0x404040FF // Dim gray when off
            };
            draw_rect(x, y + line_h * 5.5, 40.0, 30.0, color);
        }

        // Controls hint
        let hint = b"L-Stick: Rotate  R-Stick: Move Light";
        draw_text(hint.as_ptr(), hint.len() as u32, 20.0, y + line_h * 7.0, 32.0, 0x888888FF);
    }
}
