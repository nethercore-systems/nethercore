//! Procedural Shapes Example
//!
//! Demonstrates all procedural mesh generation functions with optional texture and normal mapping.
//! Uses Lambert mode (render_mode 0) which supports normal mapping:
//! - Plain: Uniform color, no textures
//! - Textured: UV debug texture showing coordinate mapping
//! - Normal Mapped: Procedural normal maps (waves, bricks, ripples)
//!
//! Shapes:
//! - cube() / cube_uv() / cube_tangent() — Box with flat normals
//! - sphere() / sphere_uv() / sphere_tangent() — UV sphere with smooth normals
//! - cylinder() / cylinder_uv() — Cylinder with caps
//! - cylinder() (cone variant) — Tapered cylinder (plain mode only)
//! - plane() / plane_uv() / plane_tangent() — Subdivided ground plane
//! - torus() / torus_uv() / torus_tangent() — Donut shape
//! - capsule() / capsule_uv() — Pill shape
//!
//! Controls:
//! - A button: Cycle through shapes
//! - B button: Cycle material mode (Plain → Textured → Normal Map)
//! - X button: Cycle normal map type (Waves → Bricks → Ripples)
//! - Left stick: Rotate shape
//! - Auto-rotates for visual inspection
//!

#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

/// Render mode: 0=Plain, 1=Textured, 2=Normal Mapped
static mut RENDER_MODE: u32 = 0;

/// Current shape index
static mut CURRENT_SHAPE: u32 = 0;

/// Current normal map type (0=Waves, 1=Bricks, 2=Ripples)
static mut NORMAL_MAP_TYPE: u32 = 0;

/// Mesh handles for plain shapes (7 shapes)
static mut MESH_HANDLES_PLAIN: [u32; 7] = [0; 7];

/// Mesh handles for UV-enabled shapes (6 shapes, no cone UV variant)
static mut MESH_HANDLES_UV: [u32; 6] = [0; 6];

/// Mesh handles for tangent-enabled shapes (4 shapes: cube, sphere, plane, torus)
static mut MESH_HANDLES_TANGENT: [u32; 4] = [0; 4];

/// Texture handle for UV debug texture
static mut TEXTURE_HANDLE: u32 = 0;

/// Texture handle for albedo texture (for normal map mode)
static mut ALBEDO_TEXTURE: u32 = 0;

/// Normal map texture handles (3 types)
static mut NORMAL_MAP_HANDLES: [u32; 3] = [0; 3];

/// Current rotation angles
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Previous button states (for edge detection)
static mut PREV_A_BUTTON: u32 = 0;
static mut PREV_B_BUTTON: u32 = 0;
static mut PREV_X_BUTTON: u32 = 0;

/// Shape names for plain mode
static SHAPE_NAMES_PLAIN: [&str; 7] = [
    "Cube (1x1x1)",
    "Sphere (r=1.5, 32x16)",
    "Cylinder (r=1, h=2, 24 segs)",
    "Cone (r=1.5->0, h=2, 24 segs)",
    "Plane (3x3, 8x8 subdivs)",
    "Torus (R=1.5, r=0.5, 32x16)",
    "Capsule (r=0.8, h=2, 24x8)",
];

/// Shape names for UV mode
static SHAPE_NAMES_UV: [&str; 6] = [
    "Cube (UV box unwrap)",
    "Sphere (UV equirectangular)",
    "Cylinder (UV cylindrical)",
    "Plane (UV grid)",
    "Torus (UV wrapped)",
    "Capsule (UV hybrid)",
];

/// Shape names for tangent/normal map mode
static SHAPE_NAMES_TANGENT: [&str; 4] = [
    "Cube (Normal Mapped)",
    "Sphere (Normal Mapped)",
    "Plane (Normal Mapped)",
    "Torus (Normal Mapped)",
];

/// Generate a colorful UV debug texture (64x64)
fn generate_uv_debug_texture() -> [u8; 64 * 64 * 4] {
    let mut pixels = [0u8; 64 * 64 * 4];
    let checker_size = 8;

    for y in 0..64 {
        for x in 0..64 {
            let idx = (y * 64 + x) * 4;
            let r = ((x * 255) / 63) as u8;
            let g = ((y * 255) / 63) as u8;
            let checker_x = x / checker_size;
            let checker_y = y / checker_size;
            let is_checker = (checker_x + checker_y) % 2 == 0;
            let b = if is_checker { 255 } else { 64 };

            pixels[idx] = r;
            pixels[idx + 1] = g;
            pixels[idx + 2] = b;
            pixels[idx + 3] = 255;
        }
    }
    pixels
}

/// Generate a simple gray albedo texture (64x64)
fn generate_albedo_texture() -> [u8; 64 * 64 * 4] {
    let mut pixels = [0u8; 64 * 64 * 4];
    for y in 0..64 {
        for x in 0..64 {
            let idx = (y * 64 + x) * 4;
            // Light gray with subtle variation
            let base = 180u8;
            let variation = ((x + y) % 8) as u8 * 2;
            pixels[idx] = base + variation;
            pixels[idx + 1] = base + variation;
            pixels[idx + 2] = base + variation;
            pixels[idx + 3] = 255;
        }
    }
    pixels
}

/// Fixed-point sine approximation (no libm dependency)
/// Input: angle in range 0-255 (representing 0-2*PI)
/// Output: value in range -127 to 127
fn sin_fixed(angle: i32) -> i32 {
    // Simple lookup table for quarter wave
    const QUARTER: [i32; 64] = [
        0, 3, 6, 9, 12, 16, 19, 22, 25, 28, 31, 34, 37, 40, 43, 46, 49, 51, 54, 57, 60, 63, 65, 68,
        71, 73, 76, 78, 81, 83, 85, 88, 90, 92, 94, 96, 98, 100, 102, 104, 106, 107, 109, 110, 112,
        113, 115, 116, 117, 118, 119, 120, 121, 122, 123, 123, 124, 125, 125, 126, 126, 126, 127,
        127,
    ];

    let a = angle & 255;
    let quadrant = a / 64;
    let idx = a % 64;

    match quadrant {
        0 => QUARTER[idx as usize],
        1 => QUARTER[63 - idx as usize],
        2 => -QUARTER[idx as usize],
        _ => -QUARTER[63 - idx as usize],
    }
}

/// Generate a wave normal map (64x64)
/// Creates a rippling wave pattern
fn generate_waves_normal_map() -> [u8; 64 * 64 * 4] {
    let mut pixels = [0u8; 64 * 64 * 4];

    for y in 0..64 {
        for x in 0..64 {
            let idx = (y * 64 + x) * 4;

            // Diagonal waves using fixed-point math (reduced frequency: 3 instead of 8)
            let wave_angle = ((x as i32 + y as i32) * 3) & 255;
            let wave = sin_fixed(wave_angle);

            // Secondary perpendicular wave (reduced frequency: 2 instead of 6)
            let wave2_angle = ((x as i32 - y as i32 + 128) * 2) & 255;
            let wave2 = sin_fixed(wave2_angle);

            // Convert wave values to normal perturbation
            // Normal = (dx, dy, z) where dx/dy are wave derivatives
            let dx = (wave / 2) as i8; // Stronger effect for visibility
            let dy = (wave2 / 2) as i8;

            // Convert to tangent-space normal (x, y, z)
            // Z should be mostly positive (pointing out of surface)
            let nx = (dx as i32 + 128) as u8;
            let ny = (dy as i32 + 128) as u8;
            let nz = 180u8; // Slightly lower Z for more pronounced effect

            pixels[idx] = nx;
            pixels[idx + 1] = ny;
            pixels[idx + 2] = nz;
            pixels[idx + 3] = 255;
        }
    }
    pixels
}

/// Generate a brick normal map (64x64)
/// Creates a tiled brick pattern with mortar grooves
fn generate_bricks_normal_map() -> [u8; 64 * 64 * 4] {
    let mut pixels = [0u8; 64 * 64 * 4];

    // Larger bricks for better visibility (2 bricks wide, 4 rows)
    let brick_width = 32;
    let brick_height = 16;
    let mortar_size = 2;

    for y in 0..64 {
        for x in 0..64 {
            let idx = (y * 64 + x) * 4;

            // Calculate brick position with stagger
            let row = y / brick_height;
            let stagger = if row % 2 == 1 { brick_width / 2 } else { 0 };
            let brick_x = (x + stagger) % brick_width;
            let brick_y = y % brick_height;

            // Check if in mortar
            let in_mortar_x = brick_x < mortar_size || brick_x >= brick_width - mortar_size;
            let in_mortar_y = brick_y < mortar_size || brick_y >= brick_height - mortar_size;
            let in_mortar = in_mortar_x || in_mortar_y;

            let (nx, ny, nz) = if in_mortar {
                // Mortar is recessed - normal points straight out but darker
                (128u8, 128u8, 180u8)
            } else {
                // Brick surface with slight random variation
                let variation = ((x * 7 + y * 13) % 20) as i32 - 10;
                let nx = (128i32 + variation) as u8;
                let ny = (128i32 - variation / 2) as u8;
                (nx, ny, 230u8)
            };

            pixels[idx] = nx;
            pixels[idx + 1] = ny;
            pixels[idx + 2] = nz;
            pixels[idx + 3] = 255;
        }
    }
    pixels
}

/// Generate a concentric ripples normal map (64x64)
/// Creates circular ripples emanating from center
fn generate_ripples_normal_map() -> [u8; 64 * 64 * 4] {
    let mut pixels = [0u8; 64 * 64 * 4];

    let center_x = 32i32;
    let center_y = 32i32;

    for y in 0..64 {
        for x in 0..64 {
            let idx = (y * 64 + x) * 4;

            // Distance from center (approximate sqrt using fixed point)
            let dx = x as i32 - center_x;
            let dy = y as i32 - center_y;

            // Simple distance approximation: max(|dx|, |dy|) + min(|dx|, |dy|)/2
            let abs_dx = if dx < 0 { -dx } else { dx };
            let abs_dy = if dy < 0 { -dy } else { dy };
            let (max_d, min_d) = if abs_dx > abs_dy {
                (abs_dx, abs_dy)
            } else {
                (abs_dy, abs_dx)
            };
            let dist = max_d + min_d / 2;

            // Ripple wave based on distance (reduced frequency: 4 instead of 12)
            let wave_angle = (dist * 4) & 255;
            let wave = sin_fixed(wave_angle);

            // Normal perturbation points radially (stronger effect for visibility)
            let len = if dist > 0 { dist } else { 1 };
            let nx_offset = (wave * dx / len / 2) as i8;
            let ny_offset = (wave * dy / len / 2) as i8;

            let nx = (128i32 + nx_offset as i32) as u8;
            let ny = (128i32 + ny_offset as i32) as u8;
            let nz = 180u8; // Lower Z for more pronounced effect

            pixels[idx] = nx;
            pixels[idx + 1] = ny;
            pixels[idx + 2] = nz;
            pixels[idx + 3] = 255;
        }
    }
    pixels
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Configure rendering
        set_clear_color(0x1a1a2eFF); // Dark blue

        // Generate all 7 plain procedural shapes
        MESH_HANDLES_PLAIN[0] = cube(1.0, 1.0, 1.0);
        MESH_HANDLES_PLAIN[1] = sphere(1.5, 32, 16);
        MESH_HANDLES_PLAIN[2] = cylinder(1.0, 1.0, 2.0, 24);
        MESH_HANDLES_PLAIN[3] = cylinder(1.5, 0.0, 2.0, 24); // Cone
        MESH_HANDLES_PLAIN[4] = plane(3.0, 3.0, 8, 8);
        MESH_HANDLES_PLAIN[5] = torus(1.5, 0.5, 32, 16);
        MESH_HANDLES_PLAIN[6] = capsule(0.8, 2.0, 24, 8);

        // Generate all 6 UV-enabled procedural shapes
        MESH_HANDLES_UV[0] = cube_uv(1.0, 1.0, 1.0);
        MESH_HANDLES_UV[1] = sphere_uv(1.5, 32, 16);
        MESH_HANDLES_UV[2] = cylinder_uv(1.0, 1.0, 2.0, 24);
        MESH_HANDLES_UV[3] = plane_uv(3.0, 3.0, 8, 8);
        MESH_HANDLES_UV[4] = torus_uv(1.5, 0.5, 32, 16);
        MESH_HANDLES_UV[5] = capsule_uv(0.8, 2.0, 24, 8);

        // Generate tangent-enabled shapes for normal mapping
        MESH_HANDLES_TANGENT[0] = cube_tangent(1.0, 1.0, 1.0);
        MESH_HANDLES_TANGENT[1] = sphere_tangent(1.5, 32, 16);
        MESH_HANDLES_TANGENT[2] = plane_tangent(3.0, 3.0, 8, 8);
        MESH_HANDLES_TANGENT[3] = torus_tangent(1.5, 0.5, 32, 16);

        // Generate and load UV debug texture
        let texture_pixels = generate_uv_debug_texture();
        TEXTURE_HANDLE = load_texture(64, 64, texture_pixels.as_ptr());

        // Generate and load albedo texture for normal map mode
        let albedo_pixels = generate_albedo_texture();
        ALBEDO_TEXTURE = load_texture(64, 64, albedo_pixels.as_ptr());

        // Generate and load normal map textures
        let waves_pixels = generate_waves_normal_map();
        NORMAL_MAP_HANDLES[0] = load_texture(64, 64, waves_pixels.as_ptr());

        let bricks_pixels = generate_bricks_normal_map();
        NORMAL_MAP_HANDLES[1] = load_texture(64, 64, bricks_pixels.as_ptr());

        let ripples_pixels = generate_ripples_normal_map();
        NORMAL_MAP_HANDLES[2] = load_texture(64, 64, ripples_pixels.as_ptr());
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // B button cycles render mode (0=Plain, 1=Textured, 2=Normal Map)
        let b_button = button_held(0, button::B);
        if b_button != 0 && PREV_B_BUTTON == 0 {
            RENDER_MODE = (RENDER_MODE + 1) % 3;

            // Reset shape index when switching modes to avoid invalid index
            let max_shapes = match RENDER_MODE {
                0 => 7, // Plain mode has cone
                1 => 6, // UV mode
                _ => 4, // Tangent mode (cube, sphere, plane, torus)
            };
            if CURRENT_SHAPE >= max_shapes {
                CURRENT_SHAPE = 0;
            }
        }
        PREV_B_BUTTON = b_button;

        // X button cycles normal map type (only in normal map mode)
        let x_button = button_held(0, button::X);
        if x_button != 0 && PREV_X_BUTTON == 0 && RENDER_MODE == 2 {
            NORMAL_MAP_TYPE = (NORMAL_MAP_TYPE + 1) % 3;
        }
        PREV_X_BUTTON = x_button;

        // A button cycles through shapes
        let a_button = button_held(0, button::A);
        if a_button != 0 && PREV_A_BUTTON == 0 {
            let max_shapes = match RENDER_MODE {
                0 => 7,
                1 => 6,
                _ => 4,
            };
            CURRENT_SHAPE = (CURRENT_SHAPE + 1) % max_shapes;
        }
        PREV_A_BUTTON = a_button;

        // Rotation control (left stick)
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        if stick_x.abs() > 0.1 || stick_y.abs() > 0.1 {
            ROTATION_Y += stick_x * 2.0;
            ROTATION_X += stick_y * 2.0;
        } else {
            // Auto-rotate when idle
            ROTATION_Y += 0.5;
            ROTATION_X += 0.3;
        }
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Set camera every frame (immediate mode)
        camera_set(0.0, 5.0, 10.0, 0.0, 0.0, 0.0);
        camera_fov(60.0);

        // Configure materials based on mode
        // Note: render_mode is set in nether.toml - we vary materials here
        match RENDER_MODE {
            0 => {
                // Plain mode - no textures, just uniform color
                use_uniform_color(1);
                skip_normal_map(1);
                material_metallic(0.0);
                material_roughness(0.8);
            }
            1 => {
                // Textured mode - UV debug texture, no normal map
                use_uniform_color(0);
                skip_normal_map(1);
                texture_bind(TEXTURE_HANDLE);
                material_metallic(0.0);
                material_roughness(0.6);
            }
            _ => {
                // Normal map mode - albedo + normal map
                use_uniform_color(0);
                skip_normal_map(0);
                material_albedo(ALBEDO_TEXTURE);
                material_normal(NORMAL_MAP_HANDLES[NORMAL_MAP_TYPE as usize]);
                material_metallic(0.0);
                material_roughness(0.5);
            }
        }

        // Draw current shape
        push_identity();
        push_rotate_y(ROTATION_Y);
        push_rotate_x(ROTATION_X);

        // Special positioning for plane (tilt to be visible from camera)
        let is_plane = match RENDER_MODE {
            0 => CURRENT_SHAPE == 4, // Plain mode: plane is index 4
            1 => CURRENT_SHAPE == 3, // UV mode: plane is index 3
            _ => CURRENT_SHAPE == 2, // Tangent mode: plane is index 2
        };

        if is_plane {
            push_rotate_x(-45.0); // Additional tilt for plane
        }

        set_color(0xFFFFFFFF); // White (no tint)

        // Draw from appropriate mesh array based on mode
        match RENDER_MODE {
            0 => draw_mesh(MESH_HANDLES_PLAIN[CURRENT_SHAPE as usize]),
            1 => draw_mesh(MESH_HANDLES_UV[CURRENT_SHAPE as usize]),
            _ => draw_mesh(MESH_HANDLES_TANGENT[CURRENT_SHAPE as usize]),
        }

        // Draw UI - mode indicator
        let mode_text = match RENDER_MODE {
            0 => "Mode: PLAIN",
            1 => "Mode: TEXTURED",
            _ => "Mode: NORMAL MAP",
        };
        let mode_color = match RENDER_MODE {
            0 => 0xFFFFFFFF,
            1 => 0x88FF88FF,
            _ => 0xFF8888FF,
        };
        set_color(mode_color);
        draw_text(mode_text.as_ptr(), mode_text.len() as u32, 10.0, 10.0, 20.0);

        // Draw shape name
        let shape_name = match RENDER_MODE {
            0 => SHAPE_NAMES_PLAIN[CURRENT_SHAPE as usize],
            1 => SHAPE_NAMES_UV[CURRENT_SHAPE as usize],
            _ => SHAPE_NAMES_TANGENT[CURRENT_SHAPE as usize],
        };
        set_color(0xFFFFFFFF);
        draw_text(
            shape_name.as_ptr(),
            shape_name.len() as u32,
            10.0,
            35.0,
            18.0,
        );

        // Draw controls
        let instruction = "A: shapes | B: mode | X: normal type | Stick: rotate";
        set_color(0xAAAAAAFF);
        draw_text(
            instruction.as_ptr(),
            instruction.len() as u32,
            10.0,
            60.0,
            14.0,
        );

        // Draw mode-specific info
        match RENDER_MODE {
            1 => {
                let uv_info = "UV Debug: Red=U, Green=V, Blue=Checker";
                set_color(0x88FF88FF);
                draw_text(uv_info.as_ptr(), uv_info.len() as u32, 10.0, 85.0, 12.0);
            }
            2 => {
                // Build combined string to avoid overlap
                let normal_info = match NORMAL_MAP_TYPE {
                    0 => "Normal texture: Waves",
                    1 => "Normal texture: Bricks",
                    _ => "Normal texture: Ripples",
                };
                set_color(0xFF8888FF);
                draw_text(normal_info.as_ptr(), normal_info.len() as u32, 10.0, 85.0, 14.0);
            }
            _ => {}
        }
    }
}
