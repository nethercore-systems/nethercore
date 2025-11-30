//! Billboard Example
//!
//! Demonstrates billboard drawing with all 4 modes:
//! - Mode 1 (Spherical): Fully camera-facing (particles)
//! - Mode 2 (Cylindrical Y): Rotates around Y axis only (trees, characters)
//! - Mode 3 (Cylindrical X): Rotates around X axis only
//! - Mode 4 (Cylindrical Z): Rotates around Z axis only
//!
//! Features:
//! - `draw_billboard()` for simple billboards
//! - `draw_billboard_region()` for sprite sheet billboards
//! - Side-by-side comparison of all 4 modes
//! - Particle system with spherical billboards
//! - Tree/foliage with cylindrical Y billboards
//! - Interactive camera via analog stick

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

    // Textures
    fn load_texture(width: u32, height: u32, pixels: *const u8) -> u32;
    fn texture_bind(handle: u32);
    fn texture_filter(filter: u32);

    // Transform
    fn transform_identity();
    fn transform_translate(x: f32, y: f32, z: f32);
    fn transform_rotate(angle_deg: f32, x: f32, y: f32, z: f32);

    // Billboard drawing
    fn draw_billboard(w: f32, h: f32, mode: u32, color: u32);
    fn draw_billboard_region(
        w: f32, h: f32,
        src_x: f32, src_y: f32, src_w: f32, src_h: f32,
        mode: u32, color: u32,
    );

    // Render state
    fn set_color(color: u32);
    fn depth_test(enabled: u32);
    fn blend_mode(mode: u32);

    // 2D drawing
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
    fn draw_rect(x: f32, y: f32, w: f32, h: f32, color: u32);

    // Timing
    fn elapsed_time() -> f32;

    // Random (deterministic)
    fn random() -> u32;
}

// Billboard modes
const MODE_SPHERICAL: u32 = 1;
const MODE_CYLINDRICAL_Y: u32 = 2;
const MODE_CYLINDRICAL_X: u32 = 3;
const MODE_CYLINDRICAL_Z: u32 = 4;

// Blend modes
const BLEND_ALPHA: u32 = 1;

// Button indices
const BUTTON_A: u32 = 0;
const BUTTON_B: u32 = 1;

// Texture handles
static mut SPRITE_TEXTURE: u32 = 0;
static mut TREE_TEXTURE: u32 = 0;
static mut PARTICLE_TEXTURE: u32 = 0;

// Camera state
static mut CAMERA_ANGLE: f32 = 0.0;
static mut CAMERA_HEIGHT: f32 = 3.0;
static mut CAMERA_DISTANCE: f32 = 12.0;

// Animation state
static mut FRAME_TIME: f32 = 0.0;
static mut PAUSED: bool = false;

// Particle system (simple fixed array)
const MAX_PARTICLES: usize = 32;
static mut PARTICLES: [Particle; MAX_PARTICLES] = [Particle::new(); MAX_PARTICLES];

#[derive(Clone, Copy)]
struct Particle {
    x: f32,
    y: f32,
    z: f32,
    vx: f32,
    vy: f32,
    vz: f32,
    life: f32,
    max_life: f32,
    size: f32,
    color: u32,
}

impl Particle {
    const fn new() -> Self {
        Self {
            x: 0.0, y: 0.0, z: 0.0,
            vx: 0.0, vy: 0.0, vz: 0.0,
            life: 0.0, max_life: 0.0,
            size: 0.0,
            color: 0xFFFFFFFF,
        }
    }

    fn spawn(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;

        // Random velocity (using deterministic random)
        let r1 = unsafe { random() };
        let r2 = unsafe { random() };
        let r3 = unsafe { random() };

        // Convert to -1..1 range
        let rx = (r1 % 1000) as f32 / 500.0 - 1.0;
        let rz = (r2 % 1000) as f32 / 500.0 - 1.0;
        let ry = (r3 % 1000) as f32 / 1000.0; // 0..1 for upward bias

        self.vx = rx * 0.5;
        self.vy = 1.0 + ry * 1.5; // Mostly upward
        self.vz = rz * 0.5;

        self.max_life = 1.5 + (r1 % 100) as f32 / 100.0;
        self.life = self.max_life;
        self.size = 0.15 + (r2 % 100) as f32 / 500.0;

        // Random warm colors (yellows, oranges, reds)
        let r = 255;
        let g = 128 + (r3 % 128) as u8;
        let b = (r1 % 64) as u8;
        self.color = ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | 0xFF;
    }

    fn update(&mut self, dt: f32) {
        if self.life <= 0.0 {
            return;
        }

        self.x += self.vx * dt;
        self.y += self.vy * dt;
        self.z += self.vz * dt;

        // Slow down and apply gravity
        self.vy -= 2.0 * dt;
        self.vx *= 0.98;
        self.vz *= 0.98;

        self.life -= dt;
    }

    fn is_alive(&self) -> bool {
        self.life > 0.0
    }

    fn alpha(&self) -> u8 {
        if self.life <= 0.0 || self.max_life <= 0.0 {
            return 0;
        }
        let t = self.life / self.max_life;
        (t * 255.0) as u8
    }
}

// 8x8 sprite texture (simple face)
const SPRITE_PIXELS: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let white = [0xFF, 0xFF, 0xFF, 0xFF];
    let black = [0x00, 0x00, 0x00, 0xFF];
    let yellow = [0xFF, 0xDD, 0x55, 0xFF];
    let trans = [0x00, 0x00, 0x00, 0x00];

    // Simple smiley face pattern
    // Row by row (8x8)
    let pattern: [[u8; 8]; 8] = [
        [0, 0, 1, 1, 1, 1, 0, 0], // Row 0: top
        [0, 1, 1, 1, 1, 1, 1, 0], // Row 1
        [1, 1, 2, 1, 1, 2, 1, 1], // Row 2: eyes
        [1, 1, 1, 1, 1, 1, 1, 1], // Row 3
        [1, 2, 1, 1, 1, 1, 2, 1], // Row 4: mouth corners
        [1, 1, 2, 2, 2, 2, 1, 1], // Row 5: mouth
        [0, 1, 1, 1, 1, 1, 1, 0], // Row 6
        [0, 0, 1, 1, 1, 1, 0, 0], // Row 7: bottom
    ];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = match pattern[y][x] {
                0 => trans,
                1 => yellow,
                2 => black,
                _ => white,
            };
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

// 8x8 tree texture (simple tree silhouette)
const TREE_PIXELS: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];
    let green = [0x22, 0x88, 0x22, 0xFF];
    let brown = [0x66, 0x44, 0x22, 0xFF];
    let trans = [0x00, 0x00, 0x00, 0x00];

    let pattern: [[u8; 8]; 8] = [
        [0, 0, 0, 1, 1, 0, 0, 0], // Row 0: top
        [0, 0, 1, 1, 1, 1, 0, 0], // Row 1
        [0, 1, 1, 1, 1, 1, 1, 0], // Row 2
        [1, 1, 1, 1, 1, 1, 1, 1], // Row 3
        [0, 1, 1, 1, 1, 1, 1, 0], // Row 4
        [0, 0, 1, 1, 1, 1, 0, 0], // Row 5
        [0, 0, 0, 2, 2, 0, 0, 0], // Row 6: trunk
        [0, 0, 0, 2, 2, 0, 0, 0], // Row 7: trunk
    ];

    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = match pattern[y][x] {
                0 => trans,
                1 => green,
                2 => brown,
                _ => trans,
            };
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

// 8x8 particle texture (soft glow)
const PARTICLE_PIXELS: [u8; 8 * 8 * 4] = {
    let mut pixels = [0u8; 256];

    // Radial falloff pattern (brighter in center)
    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let dx = (x as i32) - 3;
            let dy = (y as i32) - 3;
            let dist_sq = dx * dx + dy * dy;

            // Max distance squared is ~18 (corners), center is 0
            // Create smooth falloff
            let alpha = if dist_sq <= 1 {
                255
            } else if dist_sq <= 4 {
                200
            } else if dist_sq <= 9 {
                128
            } else if dist_sq <= 16 {
                64
            } else {
                0
            };

            let idx = (y * 8 + x) * 4;
            pixels[idx] = 255;     // R
            pixels[idx + 1] = 255; // G
            pixels[idx + 2] = 255; // B
            pixels[idx + 3] = alpha as u8; // A
            x += 1;
        }
        y += 1;
    }
    pixels
};

fn draw_text_str(s: &str, x: f32, y: f32, size: f32, color: u32) {
    unsafe {
        draw_text(s.as_ptr(), s.len() as u32, x, y, size, color);
    }
}

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark blue-gray background
        set_clear_color(0x2a2a3aFF);

        // Set up procedural sky
        set_sky(
            0.5, 0.6, 0.7,      // horizon (blue-gray)
            0.2, 0.3, 0.5,      // zenith (deeper blue)
            0.5, 0.8, 0.3,      // sun direction
            1.2, 1.1, 1.0,      // sun color
            100.0,              // sun sharpness
        );

        // Set up camera
        update_camera();
        camera_fov(60.0);

        // Enable depth testing and alpha blending
        depth_test(1);
        blend_mode(BLEND_ALPHA);

        // Load textures
        SPRITE_TEXTURE = load_texture(8, 8, SPRITE_PIXELS.as_ptr());
        TREE_TEXTURE = load_texture(8, 8, TREE_PIXELS.as_ptr());
        PARTICLE_TEXTURE = load_texture(8, 8, PARTICLE_PIXELS.as_ptr());

        // Use nearest-neighbor for crisp pixel art
        texture_filter(0);

        // Initialize particles (spawn a burst at the center)
        for i in 0..MAX_PARTICLES {
            PARTICLES[i].spawn(0.0, 0.0, 0.0);
            // Stagger life so they don't all die at once
            PARTICLES[i].life = (i as f32 / MAX_PARTICLES as f32) * 2.0;
        }
    }
}

fn update_camera() {
    unsafe {
        let rad = CAMERA_ANGLE * 0.0174533; // degrees to radians
        let cam_x = sin_approx(rad) * CAMERA_DISTANCE;
        let cam_z = cos_approx(rad) * CAMERA_DISTANCE;
        camera_set(cam_x, CAMERA_HEIGHT, cam_z, 0.0, 0.0, 0.0);
    }
}

// Simple sine approximation (no libm needed)
fn sin_approx(x: f32) -> f32 {
    // Normalize to -PI..PI
    let mut t = x;
    while t > 3.14159 {
        t -= 6.28318;
    }
    while t < -3.14159 {
        t += 6.28318;
    }
    // Bhaskara I's approximation
    let t2 = t * t;
    t * (1.0 - t2 / 6.0 * (1.0 - t2 / 20.0))
}

fn cos_approx(x: f32) -> f32 {
    sin_approx(x + 1.5708) // PI/2
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Read input for camera rotation
        let stick_x = left_stick_x(0);
        let stick_y = left_stick_y(0);

        // Rotate camera with left stick
        CAMERA_ANGLE += stick_x * 2.0;
        CAMERA_HEIGHT += stick_y * 0.1;
        CAMERA_HEIGHT = clamp(CAMERA_HEIGHT, 1.0, 10.0);

        update_camera();

        // Toggle pause with A button
        if button_pressed(0, BUTTON_A) != 0 {
            PAUSED = !PAUSED;
        }

        if !PAUSED {
            FRAME_TIME = elapsed_time();

            // Update particles
            let dt = 1.0 / 60.0; // Fixed timestep
            for i in 0..MAX_PARTICLES {
                PARTICLES[i].update(dt);

                // Respawn dead particles at the center
                if !PARTICLES[i].is_alive() {
                    PARTICLES[i].spawn(0.0, 0.0, 0.0);
                }
            }
        }
    }
}

fn clamp(v: f32, min: f32, max: f32) -> f32 {
    if v < min { min } else if v > max { max } else { v }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        set_color(0xFFFFFFFF);

        // === Draw mode comparison (4 columns) ===
        // Each column shows a sprite with a different billboard mode

        let spacing = 4.0;
        let positions = [
            (-spacing * 1.5, 0.0, 0.0, MODE_SPHERICAL, "Spherical"),
            (-spacing * 0.5, 0.0, 0.0, MODE_CYLINDRICAL_Y, "Cylindrical Y"),
            (spacing * 0.5, 0.0, 0.0, MODE_CYLINDRICAL_X, "Cylindrical X"),
            (spacing * 1.5, 0.0, 0.0, MODE_CYLINDRICAL_Z, "Cylindrical Z"),
        ];

        // Draw mode comparison sprites
        texture_bind(SPRITE_TEXTURE);
        for &(x, y, z, mode, _) in &positions {
            transform_identity();
            transform_translate(x, y + 3.0, z);
            draw_billboard(1.5, 1.5, mode, 0xFFFFFFFF);
        }

        // === Draw trees (Cylindrical Y - typical use case) ===
        texture_bind(TREE_TEXTURE);
        let tree_positions = [
            (-6.0, 0.0, -4.0),
            (-4.0, 0.0, -6.0),
            (4.0, 0.0, -5.0),
            (6.0, 0.0, -3.0),
            (-5.0, 0.0, 4.0),
            (5.0, 0.0, 5.0),
        ];

        for &(x, _y, z) in &tree_positions {
            transform_identity();
            transform_translate(x, 1.0, z); // Trees are 2 units tall, centered at 1.0
            draw_billboard(2.0, 2.0, MODE_CYLINDRICAL_Y, 0xFFFFFFFF);
        }

        // === Draw particle system (Spherical - particles always face camera) ===
        texture_bind(PARTICLE_TEXTURE);
        for i in 0..MAX_PARTICLES {
            let p = &PARTICLES[i];
            if p.is_alive() {
                transform_identity();
                transform_translate(p.x, p.y, p.z);

                // Apply alpha to color
                let alpha = p.alpha();
                let color = (p.color & 0xFFFFFF00) | (alpha as u32);

                draw_billboard(p.size, p.size, MODE_SPHERICAL, color);
            }
        }

        // === Draw ground plane indicator ===
        // Four corner markers to show ground level
        texture_bind(SPRITE_TEXTURE);
        let ground_markers = [
            (-8.0, 0.0, -8.0),
            (8.0, 0.0, -8.0),
            (-8.0, 0.0, 8.0),
            (8.0, 0.0, 8.0),
        ];

        for &(x, y, z) in &ground_markers {
            transform_identity();
            transform_translate(x, y, z);
            transform_rotate(90.0, 1.0, 0.0, 0.0); // Lay flat on ground
            draw_billboard(0.5, 0.5, MODE_SPHERICAL, 0x88888888);
        }

        // === Draw UI overlay ===
        draw_rect(10.0, 10.0, 280.0, 140.0, 0x000000AA);

        draw_text_str("Billboard Demo", 20.0, 25.0, 16.0, 0xFFFFFFFF);
        draw_text_str("L-Stick: Rotate camera", 20.0, 50.0, 12.0, 0xCCCCCCFF);
        draw_text_str("A: Pause/Resume", 20.0, 68.0, 12.0, 0xCCCCCCFF);

        // Mode labels at top
        draw_text_str("Compare billboard modes (top row):", 20.0, 95.0, 12.0, 0xFFDD88FF);
        draw_text_str("1=Spherical 2=CylY 3=CylX 4=CylZ", 20.0, 113.0, 12.0, 0xAAAAAAFF);

        if PAUSED {
            draw_rect(400.0, 250.0, 160.0, 40.0, 0x000000CC);
            draw_text_str("PAUSED", 440.0, 278.0, 24.0, 0xFFFF00FF);
        }
    }
}
