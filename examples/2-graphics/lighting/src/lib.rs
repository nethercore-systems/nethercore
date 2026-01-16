//! Lighting Example
//!
//! Demonstrates Nethercore ZX's PBR lighting system (render mode 2).
//!
//! Features demonstrated:
//! - PBR rendering (mode 2) via `render_mode = 2` in nether.toml
//! - `sphere()` to generate a smooth sphere procedurally
//! - `set_sky()` for procedural sky lighting
//! - `light_set()`, `light_color()`, `light_intensity()` for dynamic lights
//! - `material_metallic()`, `material_roughness()` for PBR materials
//! - Interactive light positioning via analog sticks
//!
//! Note: render_mode is set in nether.toml (cannot change at runtime).
//! To see other modes, change render_mode in nether.toml and rebuild.
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
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
    core::arch::wasm32::unreachable()
}

// Import the canonical FFI bindings
#[path = "../../../../include/zx.rs"]
mod ffi;
use ffi::*;

/// Fast inverse square root (Quake III style)
/// Good enough for normalizing vectors
fn fast_inv_sqrt(x: f32) -> f32 {
    let half_x = 0.5 * x;
    let i = x.to_bits();
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    y * (1.5 - half_x * y * y) // One Newton-Raphson iteration
}

/// Fast sine approximation using Bhaskara I's formula
/// Input: angle in radians (works best for -PI to PI)
/// Accurate to about 0.0016 max error
fn fast_sin(x: f32) -> f32 {
    const PI: f32 = 3.14159265359;

    // Wrap to -PI..PI range
    let mut x = x;
    while x > PI {
        x -= 2.0 * PI;
    }
    while x < -PI {
        x += 2.0 * PI;
    }

    // Bhaskara approximation: sin(x) ≈ 16x(π - |x|) / (5π² - 4|x|(π - |x|))
    // Valid for -π ≤ x ≤ π
    let abs_x = if x < 0.0 { -x } else { x };
    let num = 16.0 * x * (PI - abs_x);
    let den = 5.0 * PI * PI - 4.0 * abs_x * (PI - abs_x);
    num / den
}

/// Fast cosine approximation (using sin identity: cos(x) = sin(x + π/2))
fn fast_cos(x: f32) -> f32 {
    const HALF_PI: f32 = 1.57079632679;
    fast_sin(x + HALF_PI)
}


/// Render mode for display text (should match nether.toml render_mode)
/// 0=Lambert, 1=Matcap, 2=PBR, 3=Hybrid
const RENDER_MODE: u32 = 2;

/// Sphere mesh handle
static mut SPHERE_MESH: u32 = 0;

/// Current rotation angles (degrees)
static mut ROTATION_X: f32 = 0.0;
static mut ROTATION_Y: f32 = 0.0;

/// Camera orbit angle (degrees) - auto-rotates around the sphere
static mut CAMERA_ORBIT_ANGLE: f32 = 0.0;

/// Light directions (rays travel convention: direction FROM light source TOWARD surface)
/// For a light "above", use (0, -1, 0) - rays going down.
static mut LIGHT_DIRS: [[f32; 3]; 4] = [
    [-0.5, -0.8, -0.3],  // Light 0: from upper-right-front (rays going down-left-back)
    [0.7, -0.3, -0.5],   // Light 1: from upper-left (rays going right-down-back)
    [-0.3, 0.5, -0.7],   // Light 2: from lower-front (rays going up-back)
    [0.3, -0.6, 0.5],    // Light 3: from upper-back (rays going down-forward)
];

/// Light enabled states
static mut LIGHT_ENABLED: [bool; 4] = [true, true, false, false];

/// Light colors (0xRRGGBBAA)
static LIGHT_COLORS: [u32; 4] = [
    0xFFF2E6FF,  // Light 0: warm white
    0x99B3FFFF,  // Light 1: cool blue
    0xFFB380FF,  // Light 2: orange
    0xB3FFB3FF,  // Light 3: green
];

/// Light intensity
static mut LIGHT_INTENSITY: f32 = 1.5;

/// Material properties
static mut METALLIC: f32 = 1.0;
static mut ROUGHNESS: f32 = 0.4;  // Lower = shinier surface with visible specular highlights

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x101020FF);

        // Note: Sky uses reasonable defaults (blue gradient with sun) from the renderer
        // No need to set sky explicitly unless you want custom sky settings

        // Generate smooth sphere procedurally
        // Using 64x32 segments for a high-quality sphere (similar to subdivision level 3)
        SPHERE_MESH = sphere(1.0, 64, 32);

        // Initialize lights
        for i in 0..4u32 {
            let dir = LIGHT_DIRS[i as usize];
            light_set(i, dir[0], dir[1], dir[2]);
            let color = LIGHT_COLORS[i as usize];
            light_color(i, color);
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

        // Orbit camera around the sphere for better highlight visibility
        CAMERA_ORBIT_ANGLE += 0.5;
        if CAMERA_ORBIT_ANGLE >= 360.0 {
            CAMERA_ORBIT_ANGLE -= 360.0;
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
        if button_held(0, button::UP) != 0 {
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
        if button_held(0, button::DOWN) != 0 {
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
        if button_pressed(0, button::A) != 0 {
            LIGHT_ENABLED[0] = !LIGHT_ENABLED[0];
            if LIGHT_ENABLED[0] {
                light_enable(0);
            } else {
                light_disable(0);
            }
        }
        if button_pressed(0, button::B) != 0 {
            LIGHT_ENABLED[1] = !LIGHT_ENABLED[1];
            if LIGHT_ENABLED[1] {
                light_enable(1);
            } else {
                light_disable(1);
            }
        }
        if button_pressed(0, button::X) != 0 {
            LIGHT_ENABLED[2] = !LIGHT_ENABLED[2];
            if LIGHT_ENABLED[2] {
                light_enable(2);
            } else {
                light_disable(2);
            }
        }
        if button_pressed(0, button::Y) != 0 {
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
        // Configure and draw environment (always draw first, before any geometry)
        env_gradient(
            0,            // base layer
            0x264D99FF,   // Zenith: darker blue
            0x99BFD9FF,   // Sky horizon: light blue
            0x99BFD9FF,   // Ground horizon: light blue
            0x2A2A2AFF,   // Nadir: dark
            0.0,          // sun azimuth
            0.0,          // horizon shift
            0.0,          // sun elevation
            0,            // sun disk
            0,            // sun halo
            0,            // sun intensity (disabled)
            0,            // horizon haze
            0,            // sun warmth
            0,            // cloudiness
            0             // cloud_phase
        );
        light_set(0, -0.7, -0.2, -0.7);  // Direction: rays from sun near horizon
        light_color(0, 0xFFFAF0FF);      // Color: warm white
        light_intensity(0, 1.0);
        draw_env();

        // Update camera position to orbit around the sphere
        let orbit_radius = 4.0;
        let angle_rad = CAMERA_ORBIT_ANGLE * 0.0174533; // Convert degrees to radians (PI / 180)

        // Calculate camera position in a circle (Y stays constant for horizontal orbit)
        let cam_x = fast_sin(angle_rad) * orbit_radius;
        let cam_y = 1.0; // Slightly elevated view
        let cam_z = fast_cos(angle_rad) * orbit_radius;

        // Camera looks at the sphere at origin
        camera_set(cam_x, cam_y, cam_z, 0.0, 0.0, 0.0);

        // Draw the sphere (no rotation for now - add your own matrix math!)
        push_identity();
        // TODO: Build rotation matrices for ROTATION_X and ROTATION_Y and call transform_set()

        set_color(0xFFFFFFFF);
        draw_mesh(SPHERE_MESH);

        // Draw UI overlay
        let y = 20.0;
        let line_h = 50.0;

        // Mode indicator
        let mode_text = match RENDER_MODE {
            0 => b"Mode 0: Lambert" as &[u8],
            1 => b"Mode 1: Matcap" as &[u8],
            2 => b"Mode 2: PBR" as &[u8],
            3 => b"Mode 3: Hybrid" as &[u8],
            _ => b"Unknown Mode" as &[u8],
        };
        set_color(0xFFFFFFFF);
        draw_text(mode_text.as_ptr(), mode_text.len() as u32, 20.0, y, 20.0);

        // Material properties
        let mut buf = [0u8; 32];

        // Metallic
        let prefix = b"Metallic (LT): ";
        let len = format_float(METALLIC, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        set_color(0xCCCCCCFF);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 20.0, y + line_h, 16.0);

        // Roughness
        let prefix = b"Roughness (RT): ";
        let len = format_float(ROUGHNESS, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        set_color(0xCCCCCCFF);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 20.0, y + line_h * 2.0, 16.0);

        // Intensity
        let prefix = b"Intensity (D-pad): ";
        let len = format_float(LIGHT_INTENSITY, &mut buf[prefix.len()..]);
        buf[..prefix.len()].copy_from_slice(prefix);
        set_color(0xCCCCCCFF);
        draw_text(buf.as_ptr(), (prefix.len() + len) as u32, 20.0, y + line_h * 3.0, 16.0);

        // Light status
        let lights_label = b"Lights (A/B/X/Y):";
        set_color(0xCCCCCCFF);
        draw_text(lights_label.as_ptr(), lights_label.len() as u32, 20.0, y + line_h * 4.5, 16.0);

        // Draw light indicators
        for i in 0..4 {
            let x = 20.0 + (i as f32) * 50.0;
            let color = if LIGHT_ENABLED[i] {
                LIGHT_COLORS[i]  // Already in packed u32 format
            } else {
                0x404040FF // Dim gray when off
            };
            set_color(color);
        draw_rect(x, y + line_h * 5.5, 40.0, 30.0);
        }

        // Controls hint
        let hint = b"L-Stick: Rotate  R-Stick: Move Light";
        set_color(0x888888FF);
        draw_text(hint.as_ptr(), hint.len() as u32, 20.0, y + line_h * 7.0, 14.0);
    }
}
