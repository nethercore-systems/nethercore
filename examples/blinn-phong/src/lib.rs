//! Blinn-Phong Material Gallery (Mode 3)
//!
//! Demonstrates:
//! - Normalized Blinn-Phong lighting (Gotanda 2010)
//! - 30 material presets in 6×5 grid:
//!   - Row 1: Metals (conductors with colored specular)
//!   - Row 2: Minerals (dielectric stones and gems)
//!   - Row 3: Organic (living materials, subsurface rim)
//!   - Row 4: Synthetic (man-made materials)
//!   - Row 5: Impossible (non-physical fantasy materials)
//! - Screen-space text labels for each material
//! - Row category headers

#![no_std]
#![no_main]

use core::f32::consts::PI;
use core::panic::PanicInfo;
use libm::{cosf, sinf};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    core::arch::wasm32::unreachable()
}

// ============================================================================
// FFI Declarations
// ============================================================================

#[link(wasm_import_module = "env")]
extern "C" {
    // Configuration
    fn render_mode(mode: u32);
    fn set_clear_color(color: u32);

    // Camera
    fn camera_set(eye_x: f32, eye_y: f32, eye_z: f32, center_x: f32, center_y: f32, center_z: f32);
    fn camera_fov(fov_degrees: f32);

    // Transform
    fn push_identity();
    fn push_translate(x: f32, y: f32, z: f32);
    fn push_rotate_y(angle_degrees: f32);
    fn push_scale_uniform(scale: f32);

    // Material functions
    fn set_color(rgba: u32);
    fn material_rim(intensity: f32, power: f32);
    fn material_shininess(value: f32);
    fn material_emissive(value: f32);
    fn material_specular(value: u32);

    // Sky and lighting
    fn light_set(index: u32, x: f32, y: f32, z: f32);
    fn light_color(index: u32, color: u32);
    fn light_intensity(index: u32, intensity: f32);

    // Procedural mesh generation
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Text rendering
    fn draw_text(ptr: *const u8, len: u32, x: f32, y: f32, size: f32, color: u32);
}

// ============================================================================
// Material Presets
// ============================================================================

/// Material preset with pre-computed hex colors
/// Color format: 0xRRGGBBAA (R in highest byte, A in lowest)
#[derive(Clone, Copy)]
struct Material {
    name: &'static str,
    albedo: u32,
    specular: u32,
    shininess: f32,
    rim_intensity: f32,
    rim_power: f32,
    emissive: f32,
}

const MATERIALS: [Material; 40] = [
    // =========================================================================
    // ROW 1: METALS (indices 0-7)
    // Conductors with colored specular reflections
    // =========================================================================

    // Gold - warm yellow-orange reflections
    Material {
        name: "Gold",
        albedo: 0xE69933FF,
        specular: 0xFFCC66FF,
        shininess: 0.8,
        rim_intensity: 0.2,
        rim_power: 0.15,
        emissive: 0.0,
    },
    // Silver - neutral bright reflections
    Material {
        name: "Silver",
        albedo: 0xC0C0C0FF,
        specular: 0xE8E8E8FF,
        shininess: 0.85,
        rim_intensity: 0.2,
        rim_power: 0.12,
        emissive: 0.0,
    },
    // Chrome - mirror-like perfection
    Material {
        name: "Chrome",
        albedo: 0xCCCCCCFF,
        specular: 0xFFFFFFFF,
        shininess: 0.95,
        rim_intensity: 0.25,
        rim_power: 0.08,
        emissive: 0.0,
    },
    // Copper - warm reddish-orange
    Material {
        name: "Copper",
        albedo: 0x995933FF,
        specular: 0xCC804DFF,
        shininess: 0.65,
        rim_intensity: 0.25,
        rim_power: 0.16,
        emissive: 0.0,
    },
    // Bronze - aged yellowish alloy
    Material {
        name: "Bronze",
        albedo: 0x8C6239FF,
        specular: 0xCCA366FF,
        shininess: 0.55,
        rim_intensity: 0.15,
        rim_power: 0.18,
        emissive: 0.0,
    },
    // Iron - cool dark gray
    Material {
        name: "Iron",
        albedo: 0x4D5566FF,
        specular: 0x8899AAFF,
        shininess: 0.5,
        rim_intensity: 0.1,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Rust - corroded oxidized iron
    Material {
        name: "Rust",
        albedo: 0x8B4513FF,
        specular: 0x553311FF,
        shininess: 0.15,
        rim_intensity: 0.05,
        rim_power: 0.3,
        emissive: 0.0,
    },
    // Patina - green oxidized copper (verdigris)
    Material {
        name: "Patina",
        albedo: 0x4A7C6FFF,
        specular: 0x6B9E8AFF,
        shininess: 0.35,
        rim_intensity: 0.1,
        rim_power: 0.25,
        emissive: 0.0,
    },

    // =========================================================================
    // ROW 2: MINERALS (indices 8-15)
    // Dielectric stones and gems
    // =========================================================================

    // Obsidian - volcanic black glass
    Material {
        name: "Obsidian",
        albedo: 0x1A1A1AFF,
        specular: 0x606068FF,
        shininess: 0.85,
        rim_intensity: 0.2,
        rim_power: 0.12,
        emissive: 0.0,
    },
    // Marble - white with slight sheen
    Material {
        name: "Marble",
        albedo: 0xF0F0F0FF,
        specular: 0xFFFFFFFF,
        shininess: 0.6,
        rim_intensity: 0.1,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Jade - green translucent gem
    Material {
        name: "Jade",
        albedo: 0x4D9966FF,
        specular: 0x99FFBBFF,
        shininess: 0.72,
        rim_intensity: 0.35,
        rim_power: 0.22,
        emissive: 0.05,
    },
    // Lapis - deep royal blue
    Material {
        name: "Lapis",
        albedo: 0x1E3A5FFF,
        specular: 0x4477AAFF,
        shininess: 0.65,
        rim_intensity: 0.15,
        rim_power: 0.18,
        emissive: 0.0,
    },
    // Sandstone - tan rough desert rock
    Material {
        name: "Sandstone",
        albedo: 0xC9A86CFF,
        specular: 0x8B7355FF,
        shininess: 0.12,
        rim_intensity: 0.05,
        rim_power: 0.35,
        emissive: 0.0,
    },
    // Granite - speckled gray, polished
    Material {
        name: "Granite",
        albedo: 0x5A5A5AFF,
        specular: 0x808080FF,
        shininess: 0.5,
        rim_intensity: 0.08,
        rim_power: 0.22,
        emissive: 0.0,
    },
    // Slate - dark gray layered
    Material {
        name: "Slate",
        albedo: 0x404850FF,
        specular: 0x606870FF,
        shininess: 0.35,
        rim_intensity: 0.08,
        rim_power: 0.25,
        emissive: 0.0,
    },
    // Amber - warm orange, fossilized resin
    Material {
        name: "Amber",
        albedo: 0xCC7722FF,
        specular: 0xFFAA44FF,
        shininess: 0.7,
        rim_intensity: 0.4,
        rim_power: 0.2,
        emissive: 0.08,
    },

    // =========================================================================
    // ROW 3: ORGANIC (indices 16-23)
    // Living materials with subsurface rim effects
    // =========================================================================

    // Skin - moist flesh, subsurface scattering look
    Material {
        name: "Skin",
        albedo: 0xD9B3A6FF,
        specular: 0xE6CCBFFF,
        shininess: 0.7,
        rim_intensity: 0.3,
        rim_power: 0.25,
        emissive: 0.0,
    },
    // Leather - tanned hide, warm brown, low sheen
    Material {
        name: "Leather",
        albedo: 0x664026FF,
        specular: 0x4D4033FF,
        shininess: 0.3,
        rim_intensity: 0.1,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Fur - soft fibers, subsurface rim glow
    Material {
        name: "Fur",
        albedo: 0x8B7355FF,
        specular: 0x554433FF,
        shininess: 0.15,
        rim_intensity: 0.25,
        rim_power: 0.3,
        emissive: 0.0,
    },
    // Scales - reptilian, iridescent green-blue
    Material {
        name: "Scales",
        albedo: 0x2D4D3DFF,
        specular: 0x66AA88FF,
        shininess: 0.6,
        rim_intensity: 0.2,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Wood - polished grain, warm brown
    Material {
        name: "Wood",
        albedo: 0x8B6914FF,
        specular: 0x554422FF,
        shininess: 0.25,
        rim_intensity: 0.05,
        rim_power: 0.25,
        emissive: 0.0,
    },
    // Bone - ivory white, subtle waxy sheen
    Material {
        name: "Bone",
        albedo: 0xE8DCC8FF,
        specular: 0xCCBBA0FF,
        shininess: 0.4,
        rim_intensity: 0.1,
        rim_power: 0.25,
        emissive: 0.0,
    },
    // Chitin - insect exoskeleton, dark glossy
    Material {
        name: "Chitin",
        albedo: 0x1A1A0DFF,
        specular: 0x444433FF,
        shininess: 0.7,
        rim_intensity: 0.15,
        rim_power: 0.18,
        emissive: 0.0,
    },
    // Shell - pearlescent nacre
    Material {
        name: "Shell",
        albedo: 0xF0E8E0FF,
        specular: 0xFFEEDDFF,
        shininess: 0.65,
        rim_intensity: 0.15,
        rim_power: 0.2,
        emissive: 0.0,
    },

    // =========================================================================
    // ROW 4: SYNTHETIC (indices 24-31)
    // Man-made materials
    // =========================================================================

    // Glossy Plastic - shiny red toy
    Material {
        name: "Plastic",
        albedo: 0xCC3333FF,
        specular: 0xFFAAAAFF,
        shininess: 0.75,
        rim_intensity: 0.15,
        rim_power: 0.15,
        emissive: 0.0,
    },
    // Glass - clear, strong rim (fakes transparency)
    Material {
        name: "Glass",
        albedo: 0xE0F0F8FF,
        specular: 0xFFFFFFFF,
        shininess: 0.95,
        rim_intensity: 0.4,
        rim_power: 0.1,
        emissive: 0.0,
    },
    // Ceramic - white porcelain
    Material {
        name: "Ceramic",
        albedo: 0xF5F5F0FF,
        specular: 0xFFFFFFFF,
        shininess: 0.8,
        rim_intensity: 0.1,
        rim_power: 0.15,
        emissive: 0.0,
    },
    // Rubber - very matte black, slight sheen
    Material {
        name: "Rubber",
        albedo: 0x262626FF,
        specular: 0x404040FF,
        shininess: 0.15,
        rim_intensity: 0.05,
        rim_power: 0.3,
        emissive: 0.0,
    },
    // Carbon Fiber - dark woven pattern look
    Material {
        name: "Carbon",
        albedo: 0x202020FF,
        specular: 0x606060FF,
        shininess: 0.6,
        rim_intensity: 0.15,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Cloth - woven fabric, very matte
    Material {
        name: "Cloth",
        albedo: 0x6B5B4BFF,
        specular: 0x4A4A4AFF,
        shininess: 0.1,
        rim_intensity: 0.15,
        rim_power: 0.35,
        emissive: 0.0,
    },
    // Silk - shiny smooth fabric
    Material {
        name: "Silk",
        albedo: 0xE8D0D0FF,
        specular: 0xFFE8E8FF,
        shininess: 0.6,
        rim_intensity: 0.25,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Wax - translucent cream, subsurface
    Material {
        name: "Wax",
        albedo: 0xF5E6C8FF,
        specular: 0xFFEED0FF,
        shininess: 0.45,
        rim_intensity: 0.35,
        rim_power: 0.25,
        emissive: 0.05,
    },

    // =========================================================================
    // ROW 5: IMPOSSIBLE (indices 32-39)
    // Non-physical fantasy materials
    // =========================================================================

    // Mithril - legendary elvish silver
    Material {
        name: "Mithril",
        albedo: 0xC8D0D8FF,
        specular: 0xE8F0FFFF,
        shininess: 0.92,
        rim_intensity: 0.3,
        rim_power: 0.1,
        emissive: 0.1,
    },
    // Dragon Scale - iridescent green armor
    Material {
        name: "Dragon",
        albedo: 0x2D5A2DFF,
        specular: 0x66FF66FF,
        shininess: 0.8,
        rim_intensity: 0.35,
        rim_power: 0.15,
        emissive: 0.05,
    },
    // Void - event horizon darkness, light bends around it
    Material {
        name: "Void",
        albedo: 0x000000FF,
        specular: 0x110011FF,
        shininess: 0.98,
        rim_intensity: 0.8,
        rim_power: 0.05,
        emissive: 0.0,
    },
    // Lava - molten rock, intense orange glow
    Material {
        name: "Lava",
        albedo: 0x441100FF,
        specular: 0xFF4400FF,
        shininess: 0.5,
        rim_intensity: 0.9,
        rim_power: 0.08,
        emissive: 0.75,
    },
    // Cursed - sickly dark purple corruption
    Material {
        name: "Cursed",
        albedo: 0x4D2255FF,
        specular: 0x8833AAFF,
        shininess: 0.6,
        rim_intensity: 0.6,
        rim_power: 0.12,
        emissive: 0.15,
    },
    // Ethereal - ghostly translucent blue
    Material {
        name: "Ethereal",
        albedo: 0x88AACCFF,
        specular: 0xCCEEFFFF,
        shininess: 0.5,
        rim_intensity: 0.7,
        rim_power: 0.1,
        emissive: 0.3,
    },
    // Slime - bright green goo
    Material {
        name: "Slime",
        albedo: 0x33CC4DFF,
        specular: 0x99FF99FF,
        shininess: 0.9,
        rim_intensity: 0.45,
        rim_power: 0.12,
        emissive: 0.15,
    },
    // Holographic - vibrant iridescent, specular doesn't match albedo
    Material {
        name: "Holo",
        albedo: 0x99DDFFFF,
        specular: 0xFFAAFFFF,
        shininess: 0.85,
        rim_intensity: 0.6,
        rim_power: 0.08,
        emissive: 0.35,
    },
];

// Row category names
const ROW_NAMES: [&str; 5] = [
    "METALS",
    "MINERALS",
    "ORGANIC",
    "SYNTHETIC",
    "IMPOSSIBLE",
];

// ============================================================================
// Grid Layout Constants
// ============================================================================

const COLS: usize = 8;
const ROWS: usize = 5;
const SPACING_X: f32 = 2.2;
const SPACING_Y: f32 = 2.2;
const SPHERE_RADIUS: f32 = 1.1;

// ============================================================================
// Global State
// ============================================================================

static mut ROTATION: f32 = 0.0;
static mut LIGHT_ANGLE: f32 = 0.0;

/// Sphere mesh handle
static mut SPHERE_MESH: u32 = 0;

// ============================================================================
// Public API
// ============================================================================

#[no_mangle]
pub extern "C" fn init() {
    unsafe {
        // Dark background
        set_clear_color(0x101020FF);

        // Set Mode 3 (Blinn-Phong SS)
        render_mode(3);

        // Setup rotating lights
        light_set(0, -0.7, -0.2, -0.7);
        light_color(0, 0xFF4D4DFF);
        light_intensity(0, 0.25);

        light_set(1, 0.7, -0.2, -0.7);
        light_color(1, 0x4DFF4DFF);
        light_intensity(1, 0.25);

        light_set(2, -0.7, -0.2, 0.7);
        light_color(2, 0x4D4DFFFF);
        light_intensity(2, 0.25);

        light_set(3, 0.7, -0.2, 0.7);
        light_color(3, 0xFFFF4DFF);
        light_intensity(3, 0.25);

        // Generate sphere with enough detail for larger display
        SPHERE_MESH = sphere(1.0, 24, 12);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Rotate lights slowly
        LIGHT_ANGLE += 0.5;

        // Rotate spheres
        ROTATION += 0.25;

        // Update light positions based on rotation
        let angle_rad = LIGHT_ANGLE * PI / 180.0;
        let cos = cosf(angle_rad);
        let sin = sinf(angle_rad);

        light_set(0, -cos * 0.7, -0.2, -sin * 0.7);
        light_set(1, cos * 0.7, -0.2, -sin * 0.7);
        light_set(2, -cos * 0.7, -0.2, sin * 0.7);
        light_set(3, cos * 0.7, -0.2, sin * 0.7);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {
        // Camera to fit 6x5 grid - fill the screen
        camera_set(
            0.0, 0.0, 22.0,  // eye (moved back for larger spheres)
            0.0, 0.0, 0.0,   // center
        );
        camera_fov(30.0);

        // Calculate grid offsets (center the grid, shift right for headers)
        let grid_width = (COLS - 1) as f32 * SPACING_X;
        let grid_height = (ROWS - 1) as f32 * SPACING_Y;
        let start_x = -grid_width / 2.0 + 1.5;  // Shift right for category headers
        let start_y = grid_height / 2.0;

        // Draw all 30 materials
        for row in 0..ROWS {
            for col in 0..COLS {
                let idx = row * COLS + col;
                let material = &MATERIALS[idx];

                let x = start_x + col as f32 * SPACING_X;
                let y = start_y - row as f32 * SPACING_Y;

                draw_sphere_with_material([x, y, 0.0], SPHERE_RADIUS, material);
            }
        }

        // Draw text labels
        let text_size = 0.015;
        let label_color = 0xCCCCCCFF;
        let header_color = 0xFFCC66FF;
        let header_size = 0.022;

        // Row headers (left side)
        for row in 0..ROWS {
            let y = start_y - row as f32 * SPACING_Y;
            let screen_x = -0.98;
            let screen_y = y * 0.042;

            let name = ROW_NAMES[row];
            draw_text(name.as_ptr(), name.len() as u32, screen_x, screen_y, header_size, header_color);
        }

        // Material labels (below each sphere)
        for row in 0..ROWS {
            for col in 0..COLS {
                let idx = row * COLS + col;
                let material = &MATERIALS[idx];

                let x = start_x + col as f32 * SPACING_X;
                let y = start_y - row as f32 * SPACING_Y;

                let screen_x = x * 0.042 - 0.03;
                let screen_y = y * 0.042 - 0.06;

                draw_text(material.name.as_ptr(), material.name.len() as u32, screen_x, screen_y, text_size, label_color);
            }
        }
    }
}

// ============================================================================
// Rendering Helpers
// ============================================================================

fn draw_sphere_with_material(position: [f32; 3], radius: f32, material: &Material) {
    unsafe {
        // Set material properties
        set_color(material.albedo);
        material_shininess(material.shininess);
        material_rim(material.rim_intensity, material.rim_power);
        material_emissive(material.emissive);
        material_specular(material.specular);

        // Set transform and draw mesh
        push_identity();
        push_translate(position[0], position[1], position[2]);
        push_rotate_y(ROTATION);
        push_scale_uniform(radius);

        draw_mesh(SPHERE_MESH);
    }
}
