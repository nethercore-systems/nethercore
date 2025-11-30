//! Cube Example
//!
//! Demonstrates retained mode 3D drawing with `load_mesh_indexed` and `draw_mesh`.
//! A textured cube rotates based on analog stick input.
//!
//! Features:
//! - `load_mesh_indexed()` to create a mesh in init()
//! - `draw_mesh()` to render the retained mesh
//! - Vertex format: POS_UV_NORMAL (format 5)
//! - Camera setup with `camera_set()` and `camera_fov()`
//! - Interactive rotation via analog stick
//! - Mode 0 with normals for simple Lambert shading
//! - Procedural sky via `set_sky()`
//!
//! Note: Rollback state is automatic (entire WASM memory is snapshotted). No save_state/load_state needed.

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

    // Textures
    fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32;
    fn texture_bind(handle: u32);
    fn texture_filter(filter: u32);

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
}

/// Vertex format: POS_UV_NORMAL = 5
/// Each vertex: position (3) + uv (2) + normal (3) = 8 floats
const FORMAT_POS_UV_NORMAL: u32 = 5;

/// Cube mesh handle
static mut CUBE_MESH: u32 = 0;

/// Texture handle
static mut TEXTURE: u32 = 0;

/// Current rotation angles (degrees)
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Cube vertices: 24 vertices (4 per face, for proper normals)
/// Format: [x, y, z, u, v, nx, ny, nz] per vertex
static CUBE_VERTICES: [f32; 24 * 8] = [
    // Front face (z = 1)
    -1.0, -1.0,  1.0,  0.0, 1.0,  0.0, 0.0, 1.0,
     1.0, -1.0,  1.0,  1.0, 1.0,  0.0, 0.0, 1.0,
     1.0,  1.0,  1.0,  1.0, 0.0,  0.0, 0.0, 1.0,
    -1.0,  1.0,  1.0,  0.0, 0.0,  0.0, 0.0, 1.0,

    // Back face (z = -1)
     1.0, -1.0, -1.0,  0.0, 1.0,  0.0, 0.0, -1.0,
    -1.0, -1.0, -1.0,  1.0, 1.0,  0.0, 0.0, -1.0,
    -1.0,  1.0, -1.0,  1.0, 0.0,  0.0, 0.0, -1.0,
     1.0,  1.0, -1.0,  0.0, 0.0,  0.0, 0.0, -1.0,

    // Top face (y = 1)
    -1.0,  1.0,  1.0,  0.0, 1.0,  0.0, 1.0, 0.0,
     1.0,  1.0,  1.0,  1.0, 1.0,  0.0, 1.0, 0.0,
     1.0,  1.0, -1.0,  1.0, 0.0,  0.0, 1.0, 0.0,
    -1.0,  1.0, -1.0,  0.0, 0.0,  0.0, 1.0, 0.0,

    // Bottom face (y = -1)
    -1.0, -1.0, -1.0,  0.0, 1.0,  0.0, -1.0, 0.0,
     1.0, -1.0, -1.0,  1.0, 1.0,  0.0, -1.0, 0.0,
     1.0, -1.0,  1.0,  1.0, 0.0,  0.0, -1.0, 0.0,
    -1.0, -1.0,  1.0,  0.0, 0.0,  0.0, -1.0, 0.0,

    // Right face (x = 1)
     1.0, -1.0,  1.0,  0.0, 1.0,  1.0, 0.0, 0.0,
     1.0, -1.0, -1.0,  1.0, 1.0,  1.0, 0.0, 0.0,
     1.0,  1.0, -1.0,  1.0, 0.0,  1.0, 0.0, 0.0,
     1.0,  1.0,  1.0,  0.0, 0.0,  1.0, 0.0, 0.0,

    // Left face (x = -1)
    -1.0, -1.0, -1.0,  0.0, 1.0,  -1.0, 0.0, 0.0,
    -1.0, -1.0,  1.0,  1.0, 1.0,  -1.0, 0.0, 0.0,
    -1.0,  1.0,  1.0,  1.0, 0.0,  -1.0, 0.0, 0.0,
    -1.0,  1.0, -1.0,  0.0, 0.0,  -1.0, 0.0, 0.0,
];

/// Cube indices: 6 faces * 2 triangles * 3 vertices = 36 indices
static CUBE_INDICES: [u16; 36] = [
    // Front face
    0, 1, 2, 2, 3, 0,
    // Back face
    4, 5, 6, 6, 7, 4,
    // Top face
    8, 9, 10, 10, 11, 8,
    // Bottom face
    12, 13, 14, 14, 15, 12,
    // Right face
    16, 17, 18, 18, 19, 16,
    // Left face
    20, 21, 22, 22, 23, 20,
];

/// 8x8 checkerboard texture (RGBA8)
const CHECKERBOARD: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    let gray = [0x80, 0x80, 0x80, 0xFF];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = if (x + y) % 2 == 0 { white } else { gray };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
            x += 1;
        }
        y += 1;
    }
    pixels
};

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue background (visible if sky isn't rendered)
        set_clear_color(0x1a1a2eFF);

        // Set up procedural sky for Lambert lighting
        // Midday sky: blue gradient with bright sun
        set_sky(
            0.6, 0.7, 0.8,      // horizon color (light blue-gray)
            0.3, 0.5, 0.9,      // zenith color (deeper blue)
            0.5, 0.8, 0.3,      // sun direction (normalized: upper-right-front)
            1.5, 1.4, 1.3,      // sun color (warm white, slightly HDR)
            150.0,              // sun sharpness
        );

        // Set up camera
        camera_set(0.0, 0.0, 5.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Enable depth testing for proper 3D rendering
        depth_test(1);

        // Load checkerboard texture
        TEXTURE = load_texture(8, 8, CHECKERBOARD.as_ptr());
        texture_filter(0); // Nearest neighbor for crisp pixels

        // Load the cube mesh
        CUBE_MESH = load_mesh_indexed(
            CUBE_VERTICES.as_ptr(),
            24, // vertex count
            CUBE_INDICES.as_ptr(),
            36, // index count
            FORMAT_POS_UV_NORMAL,
        );
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Read analog stick input for rotation
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        // Rotate based on stick input (90 degrees per second at full deflection)
        ROTATION_Y += stick_x * 2.0;
        ROTATION_X += stick_y * 2.0;

        // Also slowly auto-rotate when stick is centered
        if stick_x.abs() < 0.1 && stick_y.abs() < 0.1 {
            ROTATION_Y += 0.5;
            ROTATION_X += 0.3;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Reset transform
        transform_identity();

        // Apply rotations
        transform_rotate(ROTATION_X, 1.0, 0.0, 0.0);
        transform_rotate(ROTATION_Y, 0.0, 1.0, 0.0);

        // Bind texture and set color
        texture_bind(TEXTURE);
        set_color(0xFFFFFFFF); // White (no tint)

        // Draw the cube
        draw_mesh(CUBE_MESH);
    }
}
