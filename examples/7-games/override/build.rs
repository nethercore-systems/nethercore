//! OVERRIDE Build-Time Procedural Asset Generator
//!
//! Generates all visual assets during `cargo build` using the `image` crate.
//! Assets are written to `assets/` and embedded into the ROM.

use image::{Rgba, RgbaImage};
use std::f32::consts::PI;
use std::fs;
use std::path::Path;

// ============================================================================
// COLOR PALETTES - Dark Industrial Sci-Fi
// ============================================================================

mod palette {
    use image::Rgba;

    // Base palette - metals and shadows
    pub const BLACK: Rgba<u8> = Rgba([10, 12, 15, 255]);
    pub const DARK_BG: Rgba<u8> = Rgba([18, 22, 28, 255]);
    pub const SHADOW: Rgba<u8> = Rgba([25, 30, 38, 255]);
    pub const DARK_METAL: Rgba<u8> = Rgba([35, 42, 52, 255]);
    pub const METAL: Rgba<u8> = Rgba([52, 62, 78, 255]);
    pub const LIGHT_METAL: Rgba<u8> = Rgba([72, 85, 105, 255]);
    pub const HIGHLIGHT: Rgba<u8> = Rgba([95, 112, 135, 255]);
    pub const BRIGHT: Rgba<u8> = Rgba([125, 145, 172, 255]);

    // Accent palette - gameplay colors
    pub const CYAN_DARK: Rgba<u8> = Rgba([15, 45, 55, 255]);
    pub const CYAN: Rgba<u8> = Rgba([45, 125, 145, 255]);
    pub const CYAN_BRIGHT: Rgba<u8> = Rgba([85, 185, 215, 255]);
    pub const RED_DARK: Rgba<u8> = Rgba([55, 15, 15, 255]);
    pub const RED: Rgba<u8> = Rgba([165, 35, 35, 255]);
    pub const RED_BRIGHT: Rgba<u8> = Rgba([225, 65, 65, 255]);
    pub const YELLOW_DARK: Rgba<u8> = Rgba([55, 45, 15, 255]);
    pub const YELLOW: Rgba<u8> = Rgba([195, 165, 45, 255]);
    pub const YELLOW_BRIGHT: Rgba<u8> = Rgba([245, 215, 95, 255]);
    pub const GREEN_DARK: Rgba<u8> = Rgba([15, 45, 25, 255]);
    pub const GREEN: Rgba<u8> = Rgba([45, 135, 75, 255]);
    pub const GREEN_BRIGHT: Rgba<u8> = Rgba([85, 205, 125, 255]);

    // Material palette
    pub const RUST: Rgba<u8> = Rgba([85, 52, 38, 255]);
    pub const RUST_DARK: Rgba<u8> = Rgba([52, 32, 22, 255]);
    pub const GLOW_CORE: Rgba<u8> = Rgba([145, 235, 255, 255]);
    pub const GLASS: Rgba<u8> = Rgba([85, 105, 125, 128]);
    pub const GLASS_DARK: Rgba<u8> = Rgba([45, 55, 68, 196]);
    pub const ENERGY_BLUE: Rgba<u8> = Rgba([65, 155, 225, 255]);

    pub const TRANSPARENT: Rgba<u8> = Rgba([0, 0, 0, 0]);
}

use palette::*;

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

fn blend(c1: Rgba<u8>, c2: Rgba<u8>, t: f32) -> Rgba<u8> {
    let t = t.clamp(0.0, 1.0);
    Rgba([
        (c1.0[0] as f32 * (1.0 - t) + c2.0[0] as f32 * t) as u8,
        (c1.0[1] as f32 * (1.0 - t) + c2.0[1] as f32 * t) as u8,
        (c1.0[2] as f32 * (1.0 - t) + c2.0[2] as f32 * t) as u8,
        (c1.0[3] as f32 * (1.0 - t) + c2.0[3] as f32 * t) as u8,
    ])
}

fn add_noise(c: Rgba<u8>, amount: i32, seed: u64) -> Rgba<u8> {
    // Simple deterministic noise based on seed
    let hash = |s: u64| -> i32 {
        let x = s.wrapping_mul(0x5851f42d4c957f2d);
        ((x >> 32) as i32 % (amount * 2 + 1)) - amount
    };
    Rgba([
        (c.0[0] as i32 + hash(seed)).clamp(0, 255) as u8,
        (c.0[1] as i32 + hash(seed.wrapping_add(1))).clamp(0, 255) as u8,
        (c.0[2] as i32 + hash(seed.wrapping_add(2))).clamp(0, 255) as u8,
        c.0[3],
    ])
}

fn with_alpha(c: Rgba<u8>, alpha: u8) -> Rgba<u8> {
    Rgba([c.0[0], c.0[1], c.0[2], alpha])
}

// ============================================================================
// TILESET GENERATORS
// ============================================================================

fn generate_floor_metal() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    for y in 0..8 {
        for x in 0..8 {
            let t = (x + y) as f32 / 16.0;
            let color = blend(METAL, DARK_METAL, t * 0.3);
            let seed = (y as u64 * 8 + x as u64) * 12345;
            img.put_pixel(x, y, add_noise(color, 3, seed));
        }
    }

    // Panel lines
    for x in 0..8 {
        img.put_pixel(x, 0, SHADOW);
        img.put_pixel(x, 7, LIGHT_METAL);
    }
    for y in 0..8 {
        img.put_pixel(0, y, SHADOW);
    }

    // Rivets
    img.put_pixel(1, 1, HIGHLIGHT);
    img.put_pixel(6, 1, HIGHLIGHT);
    img.put_pixel(1, 6, HIGHLIGHT);
    img.put_pixel(6, 6, HIGHLIGHT);

    img
}

fn generate_floor_grate() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    for y in 0..8 {
        for x in 0..8 {
            let seed = (y as u64 * 8 + x as u64) * 54321;
            img.put_pixel(x, y, add_noise(METAL, 2, seed));
        }
    }

    // Grate holes
    for y in (1..7).step_by(2) {
        for x in (1..7).step_by(2) {
            img.put_pixel(x, y, DARK_BG);
            if x + 1 < 8 {
                img.put_pixel(x + 1, y, DARK_BG);
            }
        }
    }

    // Horizontal bars
    for x in 0..8 {
        img.put_pixel(x, 0, LIGHT_METAL);
        img.put_pixel(x, 3, LIGHT_METAL);
        img.put_pixel(x, 6, LIGHT_METAL);
    }

    img
}

fn generate_floor_panel() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    for y in 0..8 {
        for x in 0..8 {
            let seed = (y as u64 * 8 + x as u64) * 98765;
            img.put_pixel(x, y, add_noise(LIGHT_METAL, 2, seed));
        }
    }

    for x in 0..8 {
        img.put_pixel(x, 0, HIGHLIGHT);
    }
    for y in 0..8 {
        img.put_pixel(0, y, HIGHLIGHT);
    }

    img
}

fn generate_floor_damaged() -> RgbaImage {
    let mut img = generate_floor_metal();

    // Rust patches (deterministic positions)
    let rust_positions = [(2, 3), (5, 5), (3, 6)];
    for (rx, ry) in rust_positions {
        img.put_pixel(rx, ry, RUST_DARK);
        // Spread rust
        for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
            let nx = (rx as i32 + dx) as u32;
            let ny = (ry as i32 + dy) as u32;
            if nx < 8 && ny < 8 {
                img.put_pixel(nx, ny, RUST);
            }
        }
    }

    // Diagonal crack
    for i in 0..8 {
        img.put_pixel(i, i, BLACK);
    }

    img
}

fn generate_wall_solid() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    for y in 0..8 {
        for x in 0..8 {
            let t = y as f32 / 8.0;
            let color = blend(DARK_METAL, SHADOW, t * 0.3);
            let seed = (y as u64 * 8 + x as u64) * 11111;
            img.put_pixel(x, y, add_noise(color, 3, seed));
        }
    }

    for x in 0..8 {
        img.put_pixel(x, 0, METAL);
        img.put_pixel(x, 4, SHADOW);
    }

    img
}

fn generate_wall_window() -> RgbaImage {
    let mut img = generate_wall_solid();

    // Window glass
    for y in 2..6 {
        for x in 2..6 {
            let t = (y - 2) as f32 / 4.0;
            img.put_pixel(x, y, blend(GLASS, GLASS_DARK, t));
        }
    }

    // Frame
    for x in 1..7 {
        img.put_pixel(x, 1, LIGHT_METAL);
        img.put_pixel(x, 6, LIGHT_METAL);
    }
    for y in 1..7 {
        img.put_pixel(1, y, LIGHT_METAL);
        img.put_pixel(6, y, LIGHT_METAL);
    }

    img
}

fn generate_wall_vent() -> RgbaImage {
    let mut img = generate_wall_solid();

    for y in 2..6 {
        for x in 2..6 {
            img.put_pixel(x, y, BLACK);
        }
        img.put_pixel(1, y, LIGHT_METAL);
        img.put_pixel(6, y, SHADOW);
    }

    img
}

fn generate_wall_pipe() -> RgbaImage {
    let mut img = generate_wall_solid();

    for y in 0..8 {
        img.put_pixel(2, y, SHADOW);
        img.put_pixel(3, y, METAL);
        img.put_pixel(4, y, LIGHT_METAL);
    }

    // Pipe joint
    for x in 2..5 {
        img.put_pixel(x, 4, METAL);
    }

    img
}

fn generate_wall_screen() -> RgbaImage {
    let mut img = generate_wall_solid();

    // Screen background
    for y in 2..6 {
        for x in 2..6 {
            img.put_pixel(x, y, BLACK);
        }
    }

    // Glowing elements
    img.put_pixel(3, 3, CYAN);
    img.put_pixel(4, 3, CYAN);
    img.put_pixel(3, 4, CYAN);

    // Frame
    for x in 1..7 {
        img.put_pixel(x, 1, METAL);
        img.put_pixel(x, 6, METAL);
    }
    for y in 1..7 {
        img.put_pixel(1, y, METAL);
        img.put_pixel(6, y, METAL);
    }

    img
}

fn generate_wall_doorframe() -> RgbaImage {
    let mut img = generate_wall_solid();

    for y in 0..8 {
        img.put_pixel(0, y, LIGHT_METAL);
        img.put_pixel(7, y, LIGHT_METAL);
        for x in 2..6 {
            img.put_pixel(x, y, BLACK);
        }
    }

    img
}

// ============================================================================
// DOOR SPRITES (8x16)
// ============================================================================

fn generate_door_closed() -> RgbaImage {
    let mut img = RgbaImage::new(8, 16);

    for y in 0..16 {
        for x in 0..8 {
            let seed = (y as u64 * 8 + x as u64) * 22222;
            img.put_pixel(x, y, add_noise(DARK_METAL, 2, seed));
        }
    }

    // Panel lines
    for y in [0, 5, 10, 15] {
        for x in 0..8 {
            img.put_pixel(x, y, SHADOW);
        }
    }

    // Center line
    for y in 0..16 {
        img.put_pixel(4, y, SHADOW);
    }

    // Highlights
    for y in [1, 6, 11] {
        for x in 0..8 {
            img.put_pixel(x, y, LIGHT_METAL);
        }
    }

    // Lock indicator
    img.put_pixel(6, 8, YELLOW);
    img.put_pixel(5, 8, SHADOW);

    img
}

fn generate_door_open() -> RgbaImage {
    let mut img = RgbaImage::new(8, 16);

    // Door compressed to left side
    for y in 0..16 {
        for x in 0..3 {
            let seed = (y as u64 * 8 + x as u64) * 33333;
            img.put_pixel(x, y, add_noise(DARK_METAL, 2, seed));
        }
    }

    // Panel lines on compressed part
    for y in [0, 5, 10, 15] {
        for x in 0..3 {
            img.put_pixel(x, y, SHADOW);
        }
    }

    img
}

fn generate_door_locked() -> RgbaImage {
    let mut img = generate_door_closed();

    // Red lock indicator
    for y in 7..10 {
        for x in 5..7 {
            img.put_pixel(x, y, RED_BRIGHT);
        }
    }

    // Warning stripes
    for i in 0..16 {
        let x = (i / 2) % 8;
        if i % 4 < 2 {
            img.put_pixel(x, i, YELLOW);
        }
    }

    img
}

// ============================================================================
// TRAP SPRITES (8x8)
// ============================================================================

fn generate_trap_spike() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    // Background
    for y in 0..8 {
        for x in 0..8 {
            img.put_pixel(x, y, DARK_BG);
        }
    }

    // Spike triangle
    let center_x = 4;
    for y in 4..8 {
        let width = (8 - y) / 2;
        for x in (center_x - width as u32)..(center_x + width as u32 + 1) {
            if x < 8 {
                img.put_pixel(x, y, LIGHT_METAL);
            }
        }
    }

    // Tip
    img.put_pixel(4, 4, BRIGHT);

    // Base plate
    for x in 0..8 {
        img.put_pixel(x, 7, SHADOW);
    }

    img
}

fn generate_trap_gas() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    for y in 0..8 {
        for x in 0..8 {
            let seed = (y as u64 * 8 + x as u64) * 44444;
            img.put_pixel(x, y, add_noise(DARK_METAL, 2, seed));
        }
    }

    // Vent holes
    for y in (1..7).step_by(2) {
        for x in (1..7).step_by(2) {
            img.put_pixel(x, y, SHADOW);
        }
    }

    // Hazard stripes
    for x in 0..8 {
        img.put_pixel(x, 0, YELLOW);
        img.put_pixel(x, 7, YELLOW);
    }

    img
}

fn generate_trap_laser() -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    for y in 0..8 {
        for x in 0..8 {
            img.put_pixel(x, y, DARK_METAL);
        }
    }

    // Lens (center glow)
    let center = 4.0;
    for y in 0..8 {
        for x in 0..8 {
            let dist = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            if dist < 2.0 {
                img.put_pixel(x, y, GLASS);
            }
            if dist < 1.5 {
                img.put_pixel(x, y, RED_BRIGHT);
            }
        }
    }

    for x in 0..8 {
        img.put_pixel(x, 0, SHADOW);
        img.put_pixel(x, 7, LIGHT_METAL);
    }

    img
}

// ============================================================================
// CHARACTER SPRITES
// ============================================================================

fn generate_runner_idle(frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(8, 12);
    let cx = 4u32;

    // Head
    for y in 0..3 {
        for x in (cx - 1)..(cx + 2) {
            if x < 8 {
                img.put_pixel(x, y, CYAN_DARK);
            }
        }
    }
    img.put_pixel(cx, 1, GLOW_CORE); // Visor

    // Body
    for y in 3..8 {
        for x in (cx - 2)..(cx + 3) {
            if x < 8 {
                img.put_pixel(x, y, CYAN);
            }
        }
    }

    // Legs with subtle bob
    let bob = ((frame as f32 * PI / 2.0).sin() * 0.5) as i32;
    for y in 8..12 {
        if cx >= 1 {
            img.put_pixel(cx - 1, y, CYAN_DARK);
        }
        let y2 = (y as i32 + bob).clamp(8, 11) as u32;
        if cx + 1 < 8 {
            img.put_pixel(cx + 1, y2, CYAN_DARK);
        }
    }

    // Arms
    for y in 4..7 {
        if cx >= 3 {
            img.put_pixel(cx - 3, y, CYAN_DARK);
        }
        if cx + 3 < 8 {
            img.put_pixel(cx + 3, y, CYAN_DARK);
        }
    }

    img
}

fn generate_runner_walk(frame: u32) -> RgbaImage {
    let mut img = generate_runner_idle(frame);
    let cx = 4u32;

    // Clear legs
    for y in 8..12 {
        for x in 0..8 {
            img.put_pixel(x, y, TRANSPARENT);
        }
    }

    // Animated legs
    let offset = ((frame as f32 * PI / 2.0).sin() * 2.0) as i32;
    for y in 8..12 {
        let x1 = (cx as i32 - 1 + offset).clamp(0, 7) as u32;
        let x2 = (cx as i32 + 1 - offset).clamp(0, 7) as u32;
        img.put_pixel(x1, y, CYAN_DARK);
        img.put_pixel(x2, y, CYAN_DARK);
    }

    img
}

fn generate_runner_sprint(frame: u32) -> RgbaImage {
    let mut img = generate_runner_walk(frame);

    // Motion trail
    for y in 4..8 {
        let x = (1i32 - frame as i32).clamp(0, 7) as u32;
        img.put_pixel(x, y, with_alpha(CYAN_BRIGHT, 128));
    }

    img
}

fn generate_runner_crouch(_frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(8, 12);
    let cx = 4u32;
    let start_y = 4u32;

    // Head (lower)
    for y in start_y..(start_y + 2) {
        for x in (cx - 1)..(cx + 2) {
            if x < 8 {
                img.put_pixel(x, y, CYAN_DARK);
            }
        }
    }
    img.put_pixel(cx, start_y, GLOW_CORE);

    // Compressed body
    for y in (start_y + 2)..(start_y + 5) {
        for x in (cx - 2)..(cx + 3) {
            if x < 8 && y < 12 {
                img.put_pixel(x, y, CYAN);
            }
        }
    }

    // Bent legs
    for y in (start_y + 5)..12 {
        if cx >= 1 {
            img.put_pixel(cx - 1, y, CYAN_DARK);
        }
        if cx + 1 < 8 {
            img.put_pixel(cx + 1, y, CYAN_DARK);
        }
    }

    img
}

fn generate_runner_death(frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(8, 12);
    let fade = (255.0 * (1.0 - frame as f32 / 4.0)) as u8;
    let cx = 4;

    // Dissolving fragments
    for y in 0..12 {
        for x in (cx - 2)..(cx + 3) {
            if x < 8 {
                let seed = (y as u64 * 8 + x as u64) * 55555 + frame as u64;
                let hash = seed.wrapping_mul(0x5851f42d4c957f2d) >> 32;
                if hash % 4 > frame as u64 {
                    let offset_y = (y + frame).min(11);
                    img.put_pixel(x, offset_y, with_alpha(CYAN, fade));
                }
            }
        }
    }

    img
}

fn generate_drone_hover(frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(6, 6);

    let bob = ((frame as f32 * PI / 2.0).sin()) as i32;
    let cx = 3;
    let cy = (3 + bob).clamp(1, 4) as u32;

    // Diamond body
    for y in 0..6 {
        for x in 0..6 {
            let dist = (x as i32 - cx).abs() + (y as i32 - cy as i32).abs();
            if dist <= 2 {
                img.put_pixel(x, y, DARK_METAL);
            }
        }
    }

    // Eye
    if cy < 6 {
        img.put_pixel(cx as u32, cy, RED_BRIGHT);
    }

    // Propeller blur
    let prop_alpha = (128.0 * (1.0 - frame as f32 / 4.0)) as u8;
    img.put_pixel(1, 0, with_alpha(LIGHT_METAL, prop_alpha));
    img.put_pixel(4, 0, with_alpha(LIGHT_METAL, prop_alpha));

    img
}

// ============================================================================
// VISUAL EFFECTS
// ============================================================================

fn generate_vfx_gas_cloud(frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(16, 16);
    let center = 8.0;
    let radius = 4.0 + frame as f32;

    for y in 0..16 {
        for x in 0..16 {
            let dist = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            if dist < radius {
                let density = 1.0 - (dist / radius);
                let alpha = (180.0 * density * (1.0 - frame as f32 / 8.0)).max(0.0) as u8;
                img.put_pixel(x, y, with_alpha(YELLOW, alpha));
            }
        }
    }

    img
}

fn generate_vfx_laser_beam() -> RgbaImage {
    let mut img = RgbaImage::new(32, 2);

    for x in 0..32 {
        img.put_pixel(x, 0, with_alpha(RED, 128));
        img.put_pixel(x, 1, RED_BRIGHT);
    }

    img
}

fn generate_vfx_core_glow(frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(16, 16);
    let center = 8.0;
    let pulse = 4.0 + 2.0 * (frame as f32 * PI / 2.0).sin();

    for y in 0..16 {
        for x in 0..16 {
            let dist = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            if dist < pulse {
                let intensity = 1.0 - (dist / pulse);
                let alpha = (255.0 * intensity) as u8;
                img.put_pixel(x, y, with_alpha(GLOW_CORE, alpha));
            }
        }
    }

    img
}

fn generate_vfx_sprint_dust(frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    for i in 0..3 {
        let x = (6i32 - frame as i32 - i * 2).clamp(0, 7) as u32;
        let y = 4u32;
        if x < 8 {
            let alpha = (180.0 * (1.0 - (frame + i as u32) as f32 / 4.0)).max(0.0) as u8;
            img.put_pixel(x, y, with_alpha(LIGHT_METAL, alpha));
        }
    }

    img
}

fn generate_vfx_elimination_flash(frame: u32) -> RgbaImage {
    let mut img = RgbaImage::new(16, 16);
    let center = 8.0;
    let inner_radius = frame as f32 * 2.0;
    let outer_radius = inner_radius + 2.0;

    for y in 0..16 {
        for x in 0..16 {
            let dist = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            if inner_radius <= dist && dist <= outer_radius {
                let alpha = (255.0 * (1.0 - frame as f32 / 4.0)).max(0.0) as u8;
                img.put_pixel(x, y, with_alpha(RED_BRIGHT, alpha));
            }
        }
    }

    img
}

// ============================================================================
// UI ELEMENTS
// ============================================================================

fn generate_ui_energy_bar(fill: f32) -> RgbaImage {
    let mut img = RgbaImage::new(32, 4);

    // Background
    for y in 0..4 {
        for x in 0..32 {
            img.put_pixel(x, y, DARK_BG);
        }
    }

    // Border
    for x in 0..32 {
        img.put_pixel(x, 0, LIGHT_METAL);
        img.put_pixel(x, 3, LIGHT_METAL);
    }
    for y in 0..4 {
        img.put_pixel(0, y, LIGHT_METAL);
        img.put_pixel(31, y, LIGHT_METAL);
    }

    // Fill
    let fill_width = ((30.0 * fill) as u32).min(30);
    for y in 1..3 {
        for x in 1..(1 + fill_width) {
            img.put_pixel(x, y, ENERGY_BLUE);
        }
    }

    img
}

fn generate_ui_core_indicator(active: bool) -> RgbaImage {
    let mut img = RgbaImage::new(8, 8);

    // Border
    for x in 0..8 {
        img.put_pixel(x, 0, LIGHT_METAL);
        img.put_pixel(x, 7, LIGHT_METAL);
    }
    for y in 0..8 {
        img.put_pixel(0, y, LIGHT_METAL);
        img.put_pixel(7, y, LIGHT_METAL);
    }

    // Center
    let color = if active { CYAN_BRIGHT } else { SHADOW };
    for y in 2..6 {
        for x in 2..6 {
            img.put_pixel(x, y, color);
        }
    }

    img
}

fn generate_ui_timer_digit(digit: u32) -> RgbaImage {
    let mut img = RgbaImage::new(5, 7);

    // 7-segment patterns
    let segments: [[bool; 7]; 10] = [
        [true, true, true, true, true, true, false],    // 0
        [false, true, true, false, false, false, false], // 1
        [true, true, false, true, true, false, true],   // 2
        [true, true, true, true, false, false, true],   // 3
        [false, true, true, false, false, true, true],  // 4
        [true, false, true, true, false, true, true],   // 5
        [true, false, true, true, true, true, true],    // 6
        [true, true, true, false, false, false, false], // 7
        [true, true, true, true, true, true, true],     // 8
        [true, true, true, true, false, true, true],    // 9
    ];

    let pattern = segments[digit as usize % 10];

    // Top
    if pattern[0] {
        for x in 1..4 {
            img.put_pixel(x, 0, CYAN_BRIGHT);
        }
    }
    // Top-right
    if pattern[1] {
        for y in 1..3 {
            img.put_pixel(4, y, CYAN_BRIGHT);
        }
    }
    // Bottom-right
    if pattern[2] {
        for y in 4..6 {
            img.put_pixel(4, y, CYAN_BRIGHT);
        }
    }
    // Bottom
    if pattern[3] {
        for x in 1..4 {
            img.put_pixel(x, 6, CYAN_BRIGHT);
        }
    }
    // Bottom-left
    if pattern[4] {
        for y in 4..6 {
            img.put_pixel(0, y, CYAN_BRIGHT);
        }
    }
    // Top-left
    if pattern[5] {
        for y in 1..3 {
            img.put_pixel(0, y, CYAN_BRIGHT);
        }
    }
    // Middle
    if pattern[6] {
        for x in 1..4 {
            img.put_pixel(x, 3, CYAN_BRIGHT);
        }
    }

    img
}

fn generate_ui_power_button(active: bool) -> RgbaImage {
    let mut img = RgbaImage::new(12, 12);

    // Background
    for y in 0..12 {
        for x in 0..12 {
            img.put_pixel(x, y, DARK_BG);
        }
    }

    // Border
    for x in 0..12 {
        img.put_pixel(x, 0, LIGHT_METAL);
        img.put_pixel(x, 11, LIGHT_METAL);
    }
    for y in 0..12 {
        img.put_pixel(0, y, LIGHT_METAL);
        img.put_pixel(11, y, LIGHT_METAL);
    }

    // Power icon
    let color = if active { GREEN_BRIGHT } else { METAL };
    let center = 6.0;

    for y in 0..12 {
        for x in 0..12 {
            let dist = ((x as f32 - center).powi(2) + (y as f32 - center).powi(2)).sqrt();
            if 3.0 <= dist && dist <= 4.0 {
                img.put_pixel(x, y, color);
            }
        }
    }

    for y in 2..6 {
        img.put_pixel(6, y, color);
    }

    img
}

// ============================================================================
// AUDIO SYNTHESIS MODULE
// ============================================================================

mod audio {
    use hound::{WavSpec, WavWriter};
    use std::f32::consts::PI;
    use std::path::Path;

    const SAMPLE_RATE: u32 = 22050; // Lower for smaller files

    pub fn save_wav(samples: &[f32], path: &Path) {
        let spec = WavSpec {
            channels: 1,
            sample_rate: SAMPLE_RATE,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = WavWriter::create(path, spec).unwrap();
        for &sample in samples {
            let s = (sample.clamp(-1.0, 1.0) * 32767.0) as i16;
            writer.write_sample(s).unwrap();
        }
        writer.finalize().unwrap();
    }

    // Oscillators
    fn sine(phase: f32) -> f32 {
        (phase * 2.0 * PI).sin()
    }

    fn square(phase: f32) -> f32 {
        if phase.fract() < 0.5 { 1.0 } else { -1.0 }
    }

    fn saw(phase: f32) -> f32 {
        2.0 * phase.fract() - 1.0
    }

    fn noise(seed: u32) -> f32 {
        let x = seed.wrapping_mul(0x5851f42d).wrapping_add(0x14057b7e);
        (x as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    // ADSR Envelope
    fn adsr(t: f32, a: f32, d: f32, s: f32, r: f32, duration: f32) -> f32 {
        if t < a {
            t / a // Attack
        } else if t < a + d {
            1.0 - (1.0 - s) * ((t - a) / d) // Decay
        } else if t < duration - r {
            s // Sustain
        } else if t < duration {
            s * (1.0 - (t - (duration - r)) / r) // Release
        } else {
            0.0
        }
    }

    // Simple low-pass filter
    fn lpf(samples: &mut [f32], cutoff: f32) {
        let rc = 1.0 / (cutoff * 2.0 * PI);
        let dt = 1.0 / SAMPLE_RATE as f32;
        let alpha = dt / (rc + dt);

        let mut prev = samples[0];
        for s in samples.iter_mut() {
            *s = prev + alpha * (*s - prev);
            prev = *s;
        }
    }

    // ========== SOUND EFFECTS ==========

    pub fn sfx_footstep() -> Vec<f32> {
        let duration = 0.08;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.005, 0.03, 0.0, 0.04, duration);
            *s = noise(i as u32) * env * 0.3;
        }
        lpf(&mut buf, 2000.0);
        buf
    }

    pub fn sfx_footstep_sprint() -> Vec<f32> {
        let duration = 0.06;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.003, 0.02, 0.0, 0.03, duration);
            *s = noise(i as u32) * env * 0.4;
        }
        lpf(&mut buf, 3000.0);
        buf
    }

    pub fn sfx_door_open() -> Vec<f32> {
        let duration = 0.25;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.01, 0.1, 0.3, 0.14, duration);
            let freq = 150.0 + t * 200.0; // Rising pitch
            let phase = t * freq / SAMPLE_RATE as f32;
            *s = (sine(phase) * 0.5 + noise(i as u32) * 0.3) * env * 0.5;
        }
        lpf(&mut buf, 1500.0);
        buf
    }

    pub fn sfx_door_close() -> Vec<f32> {
        let duration = 0.2;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.01, 0.05, 0.2, 0.13, duration);
            let freq = 200.0 - t * 100.0; // Falling pitch
            let phase = t * freq / SAMPLE_RATE as f32;
            *s = (sine(phase) * 0.6 + noise(i as u32) * 0.4) * env * 0.6;
        }
        lpf(&mut buf, 1200.0);
        buf
    }

    pub fn sfx_door_locked() -> Vec<f32> {
        let duration = 0.15;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.005, 0.05, 0.0, 0.09, duration);
            // Buzzer + click
            let phase = t * 400.0;
            *s = (square(phase) * 0.3 + noise(i as u32) * 0.2) * env * 0.5;
        }
        buf
    }

    pub fn sfx_trap_spike() -> Vec<f32> {
        let duration = 0.3;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.01, 0.1, 0.1, 0.18, duration);
            // Metallic scrape + impact
            let freq = 800.0 - t * 600.0;
            let phase = t * freq;
            *s = (saw(phase) * 0.4 + noise(i as u32) * 0.3) * env * 0.6;
        }
        lpf(&mut buf, 2500.0);
        buf
    }

    pub fn sfx_trap_gas() -> Vec<f32> {
        let duration = 0.5;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.05, 0.1, 0.6, 0.25, duration);
            // Hissing noise
            *s = noise(i as u32) * env * 0.4;
        }
        lpf(&mut buf, 4000.0);
        buf
    }

    pub fn sfx_trap_laser() -> Vec<f32> {
        let duration = 0.4;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.02, 0.1, 0.5, 0.18, duration);
            // Electric hum with harmonics
            let phase = t * 200.0;
            *s = (sine(phase) * 0.5 + sine(phase * 2.0) * 0.3 + sine(phase * 3.0) * 0.2) * env * 0.5;
        }
        buf
    }

    pub fn sfx_core_pickup() -> Vec<f32> {
        let duration = 0.4;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.01, 0.1, 0.5, 0.19, duration);
            // Rising arpeggio chime
            let note = (t * 8.0) as u32 % 4;
            let freq = match note {
                0 => 523.25, // C5
                1 => 659.25, // E5
                2 => 783.99, // G5
                _ => 1046.5, // C6
            };
            let phase = t * freq;
            *s = sine(phase) * env * 0.5;
        }
        buf
    }

    pub fn sfx_drone_spawn() -> Vec<f32> {
        let duration = 0.3;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.02, 0.1, 0.4, 0.18, duration);
            // Mechanical deployment sound
            let freq = 100.0 + t * 150.0;
            let phase = t * freq;
            *s = (square(phase) * 0.3 + noise(i as u32) * 0.2) * env * 0.5;
        }
        lpf(&mut buf, 2000.0);
        buf
    }

    pub fn sfx_drone_hover() -> Vec<f32> {
        let duration = 1.0; // Loop
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            // Oscillating hum
            let freq = 150.0 + (t * 4.0 * PI).sin() * 20.0;
            let phase = t * freq;
            *s = sine(phase) * 0.3;
        }
        buf
    }

    pub fn sfx_runner_death() -> Vec<f32> {
        let duration = 0.5;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.01, 0.2, 0.0, 0.29, duration);
            // Impact + static
            let impact = if t < 0.1 { noise(i as u32) * (1.0 - t * 10.0) } else { 0.0 };
            let static_noise = noise(i as u32 + 1000) * 0.3 * (1.0 - t / duration);
            *s = (impact * 0.8 + static_noise) * env;
        }
        lpf(&mut buf, 3000.0);
        buf
    }

    pub fn sfx_alarm() -> Vec<f32> {
        let duration = 0.8;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.01, 0.1, 0.7, 0.09, duration);
            // Alternating tones
            let freq = if ((t * 8.0) as u32) % 2 == 0 { 800.0 } else { 600.0 };
            let phase = t * freq;
            *s = square(phase) * env * 0.4;
        }
        buf
    }

    pub fn sfx_lights_off() -> Vec<f32> {
        let duration = 0.2;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.01, 0.05, 0.0, 0.14, duration);
            // Power-down sweep
            let freq = 1000.0 - t * 800.0;
            let phase = t * freq;
            *s = sine(phase) * env * 0.4;
        }
        buf
    }

    pub fn sfx_timer_warning() -> Vec<f32> {
        let duration = 0.15;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.005, 0.05, 0.0, 0.095, duration);
            let phase = t * 880.0; // A5
            *s = sine(phase) * env * 0.5;
        }
        buf
    }

    pub fn sfx_extraction_open() -> Vec<f32> {
        let duration = 0.6;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.02, 0.2, 0.4, 0.28, duration);
            // Triumphant chord
            let c5 = sine(t * 523.25) * 0.3;
            let e5 = sine(t * 659.25) * 0.3;
            let g5 = sine(t * 783.99) * 0.3;
            *s = (c5 + e5 + g5) * env;
        }
        buf
    }

    // ========== MUSIC ==========

    pub fn mus_menu() -> Vec<f32> {
        let duration = 8.0; // 8 second loop
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        let notes = [130.81, 146.83, 164.81, 174.61]; // C3, D3, E3, F3

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let beat = (t * 0.5) as usize % 4;
            let freq = notes[beat];
            let phase = t * freq;

            // Ambient pad
            let pad = sine(phase) * 0.2 + sine(phase * 2.0) * 0.1;
            // Slow arpeggio
            let arp_freq = notes[(t * 2.0) as usize % 4] * 2.0;
            let arp = sine(t * arp_freq) * 0.15 * ((t * 4.0 * PI).sin() * 0.5 + 0.5);

            *s = pad + arp;
        }
        lpf(&mut buf, 2000.0);
        buf
    }

    pub fn mus_gameplay() -> Vec<f32> {
        let duration = 16.0; // 16 second loop
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;

            // Tension bass
            let bass_freq = 55.0 + (t * 0.5 * PI).sin() * 10.0;
            let bass = sine(t * bass_freq) * 0.25;

            // Pulsing pad
            let pulse = ((t * 2.0 * PI).sin() * 0.5 + 0.5);
            let pad = sine(t * 110.0) * 0.1 * pulse;

            // Subtle percussion
            let beat = (t * 2.0) as u32;
            let kick = if beat % 4 == 0 {
                let bt = t.fract() * 2.0;
                if bt < 0.1 { sine(bt * 60.0) * (1.0 - bt * 10.0) * 0.3 } else { 0.0 }
            } else { 0.0 };

            *s = bass + pad + kick;
        }
        lpf(&mut buf, 1500.0);
        buf
    }

    pub fn mus_chase() -> Vec<f32> {
        let duration = 4.0; // 4 second loop (faster)
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;

            // Fast bass
            let bass_notes = [82.41, 92.50, 82.41, 103.83]; // E2, F#2, E2, G#2
            let beat = (t * 4.0) as usize % 4;
            let bass = saw(t * bass_notes[beat]) * 0.3;

            // Urgent hi-hat
            let hat = if (t * 8.0).fract() < 0.1 {
                noise(i as u32) * 0.2
            } else { 0.0 };

            // Kick
            let kick = if (t * 4.0).fract() < 0.05 {
                sine(t.fract() * 100.0) * 0.4
            } else { 0.0 };

            *s = bass + hat + kick;
        }
        lpf(&mut buf, 3000.0);
        buf
    }

    pub fn mus_victory() -> Vec<f32> {
        let duration = 2.0;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.1, 0.3, 0.5, 1.0, duration);

            // Major chord arpeggio
            let notes = [261.63, 329.63, 392.00, 523.25]; // C4, E4, G4, C5
            let note_idx = ((t * 4.0) as usize).min(3);
            let freq = notes[note_idx];
            let phase = t * freq;

            *s = sine(phase) * env * 0.5;
        }
        buf
    }

    pub fn mus_defeat() -> Vec<f32> {
        let duration = 2.0;
        let samples = (duration * SAMPLE_RATE as f32) as usize;
        let mut buf = vec![0.0; samples];

        for (i, s) in buf.iter_mut().enumerate() {
            let t = i as f32 / SAMPLE_RATE as f32;
            let env = adsr(t, 0.1, 0.5, 0.2, 1.0, duration);

            // Minor descending
            let notes = [293.66, 261.63, 233.08, 220.00]; // D4, C4, Bb3, A3
            let note_idx = ((t * 2.0) as usize).min(3);
            let freq = notes[note_idx];
            let phase = t * freq;

            *s = sine(phase) * env * 0.4;
        }
        buf
    }
}

// ============================================================================
// MAIN BUILD SCRIPT
// ============================================================================

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let base_path = Path::new("assets");

    // Create directories
    for dir in &["tilesets", "sprites", "vfx", "ui"] {
        fs::create_dir_all(base_path.join(dir)).unwrap();
    }

    println!("cargo:warning=Generating OVERRIDE procedural assets...");

    // Floor tiles
    generate_floor_metal().save(base_path.join("tilesets/floor_metal.png")).unwrap();
    generate_floor_grate().save(base_path.join("tilesets/floor_grate.png")).unwrap();
    generate_floor_panel().save(base_path.join("tilesets/floor_panel.png")).unwrap();
    generate_floor_damaged().save(base_path.join("tilesets/floor_damaged.png")).unwrap();

    // Wall tiles
    generate_wall_solid().save(base_path.join("tilesets/wall_solid.png")).unwrap();
    generate_wall_window().save(base_path.join("tilesets/wall_window.png")).unwrap();
    generate_wall_vent().save(base_path.join("tilesets/wall_vent.png")).unwrap();
    generate_wall_pipe().save(base_path.join("tilesets/wall_pipe.png")).unwrap();
    generate_wall_screen().save(base_path.join("tilesets/wall_screen.png")).unwrap();
    generate_wall_doorframe().save(base_path.join("tilesets/wall_doorframe.png")).unwrap();

    // Doors
    generate_door_closed().save(base_path.join("sprites/door_closed.png")).unwrap();
    generate_door_open().save(base_path.join("sprites/door_open.png")).unwrap();
    generate_door_locked().save(base_path.join("sprites/door_locked.png")).unwrap();

    // Traps
    generate_trap_spike().save(base_path.join("sprites/trap_spike.png")).unwrap();
    generate_trap_gas().save(base_path.join("sprites/trap_gas.png")).unwrap();
    generate_trap_laser().save(base_path.join("sprites/trap_laser.png")).unwrap();

    // Runner character (4 frames each animation)
    for frame in 0..4 {
        generate_runner_idle(frame).save(base_path.join(format!("sprites/runner_idle_{frame}.png"))).unwrap();
        generate_runner_walk(frame).save(base_path.join(format!("sprites/runner_walk_{frame}.png"))).unwrap();
        generate_runner_sprint(frame).save(base_path.join(format!("sprites/runner_sprint_{frame}.png"))).unwrap();
        generate_runner_crouch(frame).save(base_path.join(format!("sprites/runner_crouch_{frame}.png"))).unwrap();
        generate_runner_death(frame).save(base_path.join(format!("sprites/runner_death_{frame}.png"))).unwrap();
    }

    // Drone (4 frames)
    for frame in 0..4 {
        generate_drone_hover(frame).save(base_path.join(format!("sprites/drone_hover_{frame}.png"))).unwrap();
    }

    // VFX
    for frame in 0..8 {
        generate_vfx_gas_cloud(frame).save(base_path.join(format!("vfx/gas_cloud_{frame}.png"))).unwrap();
    }
    generate_vfx_laser_beam().save(base_path.join("vfx/laser_beam.png")).unwrap();
    for frame in 0..4 {
        generate_vfx_core_glow(frame).save(base_path.join(format!("vfx/core_glow_{frame}.png"))).unwrap();
        generate_vfx_sprint_dust(frame).save(base_path.join(format!("vfx/sprint_dust_{frame}.png"))).unwrap();
        generate_vfx_elimination_flash(frame).save(base_path.join(format!("vfx/elimination_flash_{frame}.png"))).unwrap();
    }

    // UI elements
    for (fill, name) in [(0.0, "0"), (0.25, "25"), (0.5, "50"), (0.75, "75"), (1.0, "100")] {
        generate_ui_energy_bar(fill).save(base_path.join(format!("ui/energy_bar_{name}.png"))).unwrap();
    }
    generate_ui_core_indicator(true).save(base_path.join("ui/core_indicator_active.png")).unwrap();
    generate_ui_core_indicator(false).save(base_path.join("ui/core_indicator_inactive.png")).unwrap();
    for digit in 0..10 {
        generate_ui_timer_digit(digit).save(base_path.join(format!("ui/timer_digit_{digit}.png"))).unwrap();
    }
    generate_ui_power_button(true).save(base_path.join("ui/power_button_active.png")).unwrap();
    generate_ui_power_button(false).save(base_path.join("ui/power_button_inactive.png")).unwrap();

    // Audio directory
    fs::create_dir_all(base_path.join("audio/sfx")).unwrap();
    fs::create_dir_all(base_path.join("audio/music")).unwrap();

    // Sound effects
    audio::save_wav(&audio::sfx_footstep(), &base_path.join("audio/sfx/footstep.wav"));
    audio::save_wav(&audio::sfx_footstep_sprint(), &base_path.join("audio/sfx/footstep_sprint.wav"));
    audio::save_wav(&audio::sfx_door_open(), &base_path.join("audio/sfx/door_open.wav"));
    audio::save_wav(&audio::sfx_door_close(), &base_path.join("audio/sfx/door_close.wav"));
    audio::save_wav(&audio::sfx_door_locked(), &base_path.join("audio/sfx/door_locked.wav"));
    audio::save_wav(&audio::sfx_trap_spike(), &base_path.join("audio/sfx/trap_spike.wav"));
    audio::save_wav(&audio::sfx_trap_gas(), &base_path.join("audio/sfx/trap_gas.wav"));
    audio::save_wav(&audio::sfx_trap_laser(), &base_path.join("audio/sfx/trap_laser.wav"));
    audio::save_wav(&audio::sfx_core_pickup(), &base_path.join("audio/sfx/core_pickup.wav"));
    audio::save_wav(&audio::sfx_drone_spawn(), &base_path.join("audio/sfx/drone_spawn.wav"));
    audio::save_wav(&audio::sfx_drone_hover(), &base_path.join("audio/sfx/drone_hover.wav"));
    audio::save_wav(&audio::sfx_runner_death(), &base_path.join("audio/sfx/runner_death.wav"));
    audio::save_wav(&audio::sfx_alarm(), &base_path.join("audio/sfx/alarm.wav"));
    audio::save_wav(&audio::sfx_lights_off(), &base_path.join("audio/sfx/lights_off.wav"));
    audio::save_wav(&audio::sfx_timer_warning(), &base_path.join("audio/sfx/timer_warning.wav"));
    audio::save_wav(&audio::sfx_extraction_open(), &base_path.join("audio/sfx/extraction_open.wav"));

    // Music
    audio::save_wav(&audio::mus_menu(), &base_path.join("audio/music/menu.wav"));
    audio::save_wav(&audio::mus_gameplay(), &base_path.join("audio/music/gameplay.wav"));
    audio::save_wav(&audio::mus_chase(), &base_path.join("audio/music/chase.wav"));
    audio::save_wav(&audio::mus_victory(), &base_path.join("audio/music/victory.wav"));
    audio::save_wav(&audio::mus_defeat(), &base_path.join("audio/music/defeat.wav"));

    println!("cargo:warning=Generated 80 visual + 21 audio procedural assets in assets/");
}
