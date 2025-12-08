//! Blinn-Phong Example (Mode 3)
//!
//! Demonstrates:
//! - Normalized Blinn-Phong lighting (Gotanda 2010)
//! - Procedural sphere() for smooth geometry
//! - Multiple material presets (gold, silver, leather, wet skin, plastic)
//! - Shininess variation (1-256 range)
//! - Rim lighting controls
//! - 4 directional lights + sun
//! - Interactive material switching

#![no_std]
#![no_main]

use core::f32::consts::PI;
use core::panic::PanicInfo;
use libm::{cosf, sinf};

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    // Trigger a WASM trap so runtime can catch the error
    // instead of infinite loop which freezes the game
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
    fn light_enable(index: u32);
    fn light_disable(index: u32);

    // Procedural mesh generation
    fn sphere(radius: f32, segments: u32, rings: u32) -> u32;

    // Mesh drawing
    fn draw_mesh(handle: u32);

    // Input
    fn button_pressed(player: u32, button: u32) -> u32;
}

// Button constants (from Z input)
const BTN_FACE_DOWN: u32 = 1 << 0;   // A/Cross
const BTN_FACE_RIGHT: u32 = 1 << 1;  // B/Circle
const BTN_FACE_LEFT: u32 = 1 << 2;   // X/Square
const BTN_FACE_UP: u32 = 1 << 3;     // Y/Triangle

// ============================================================================
// Material Presets
// ============================================================================

/// Material preset with pre-computed hex colors
/// Color format: 0xRRGGBBAA (R in highest byte, A in lowest)
#[derive(Clone, Copy)]
struct Material {
    name: &'static str,
    albedo: u32,      // Base color (0xRRGGBBAA)
    specular: u32,    // Specular highlight color (0xRRGGBBAA)
    shininess: f32,
    rim_intensity: f32,
    rim_power: f32,
    emissive: f32,
}

const MATERIALS: [Material; 9] = [
    // Gold armor - warm orange specular, high shininess, subtle rim
    Material {
        name: "Gold Armor",
        albedo: 0xE69933FF,    // [0.9, 0.6, 0.2]
        specular: 0xFFCC66FF,  // [1.0, 0.8, 0.4]
        shininess: 0.8,        // Maps to ~205 (tight highlights)
        rim_intensity: 0.2,
        rim_power: 0.15,       // Maps to ~4.8 (broad rim)
        emissive: 0.0,
    },
    // Silver metal - neutral white specular, very high shininess, minimal rim
    Material {
        name: "Silver Metal",
        albedo: 0xE6E6E6FF,    // [0.9, 0.9, 0.9]
        specular: 0xF2F2F2FF,  // [0.95, 0.95, 0.95]
        shininess: 0.85,       // Maps to ~217 (very tight highlights)
        rim_intensity: 0.15,
        rim_power: 0.12,       // Maps to ~3.8
        emissive: 0.0,
    },
    // Leather - dark brown, low shininess, subtle rim
    Material {
        name: "Leather",
        albedo: 0x664026FF,    // [0.4, 0.25, 0.15]
        specular: 0x4D4033FF,  // [0.3, 0.25, 0.2]
        shininess: 0.3,        // Maps to ~77 (broad highlights)
        rim_intensity: 0.1,
        rim_power: 0.2,        // Maps to ~6.4
        emissive: 0.0,
    },
    // Wet skin - bright specular, medium-high shininess, strong rim
    Material {
        name: "Wet Skin",
        albedo: 0xD9B3A6FF,    // [0.85, 0.7, 0.65]
        specular: 0xE6CCBFFF,  // [0.9, 0.8, 0.75]
        shininess: 0.7,        // Maps to ~179 (medium-tight highlights)
        rim_intensity: 0.3,
        rim_power: 0.25,       // Maps to ~8.0
        emissive: 0.0,
    },
    // Matte plastic - gray, medium shininess, no rim
    Material {
        name: "Matte Plastic",
        albedo: 0x80808CFF,    // [0.5, 0.5, 0.55]
        specular: 0x80808CFF,  // [0.5, 0.5, 0.55]
        shininess: 0.5,        // Maps to ~128 (medium highlights)
        rim_intensity: 0.0,
        rim_power: 0.0,
        emissive: 0.0,
    },
    // Emissive crystal - bright cyan, high shininess, strong rim, glowing
    Material {
        name: "Glowing Crystal",
        albedo: 0x4DB3E6FF,    // [0.3, 0.7, 0.9]
        specular: 0xCCFFFFFF,  // [0.8, 1.0, 1.0]
        shininess: 0.75,       // Maps to ~192
        rim_intensity: 0.4,
        rim_power: 0.18,       // Maps to ~5.7
        emissive: 0.3,         // Self-illumination
    },
    // Brushed copper - warm metallic with directional grain
    Material {
        name: "Brushed Copper",
        albedo: 0x995933FF,    // [0.6, 0.35, 0.2]
        specular: 0xCC804DFF,  // [0.8, 0.5, 0.3] warm copper-tinted highlights
        shininess: 0.65,       // Maps to ~166 (medium highlights, shows brushing)
        rim_intensity: 0.25,
        rim_power: 0.16,       // Maps to ~5.1
        emissive: 0.0,
    },
    // Polished steel - cool metallic, very reflective
    Material {
        name: "Polished Steel",
        albedo: 0x4D5966FF,    // [0.3, 0.35, 0.4]
        specular: 0xF2F2FFFF,  // [0.95, 0.95, 1.0] bright blue-white highlights
        shininess: 0.88,       // Maps to ~225 (very tight, mirror-like)
        rim_intensity: 0.2,
        rim_power: 0.1,        // Maps to ~3.2
        emissive: 0.0,
    },
    // Neon pink - cyberpunk glow
    Material {
        name: "Neon Pink",
        albedo: 0x4D1A33FF,    // [0.3, 0.1, 0.2]
        specular: 0xFF4DB3FF,  // [1.0, 0.3, 0.7] hot pink specular
        shininess: 0.6,        // Maps to ~154
        rim_intensity: 0.5,    // Strong rim for that neon effect
        rim_power: 0.2,        // Maps to ~6.4
        emissive: 0.4,         // Glowing like the crystal
    },
];

// ============================================================================
// Global State
// ============================================================================

static mut CURRENT_MATERIAL: usize = 0;
static mut ROTATION: f32 = 0.0;
static mut LIGHT_ANGLE: f32 = 0.0;
static mut SHOW_ALL_MATERIALS: bool = true;

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
        // Set Mode 3 (Blinn-Phong)

        render_mode(3);


        // Setup 4 dynamic lights in a rotating pattern
        // Light 0: Red from front-left
        light_set(0, -0.7, -0.2, -0.7);
        light_color(0, 0xFF4D4DFF);  // Red
        light_intensity(0, 0.6);
        // light_enable(0);

        // Light 1: Green from front-right
        light_set(1, 0.7, -0.2, -0.7);
        light_color(1, 0x4DFF4DFF);  // Green
        light_intensity(1, 0.6);
        // light_enable(1);

        // Light 2: Blue from back-left
        light_set(2, -0.7, -0.2, 0.7);
        light_color(2, 0x4D4DFFFF);  // Blue
        light_intensity(2, 0.6);
        // light_enable(2);

        // Light 3: Yellow from back-right
        light_set(3, 0.7, -0.2, 0.7);
        light_color(3, 0xFFFF4DFF);  // Yellow
        light_intensity(3, 0.6);
        // light_enable(3);

        // Generate smooth sphere procedurally (64x32 segments for high quality)
        SPHERE_MESH = sphere(1.0, 64, 32);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Rotate lights slowly
        LIGHT_ANGLE += 0.5;

        // Rotate spheres
        ROTATION += 0.5;

        // Handle input - cycle materials
        if button_pressed(0, BTN_FACE_DOWN) != 0 {
            CURRENT_MATERIAL = (CURRENT_MATERIAL + 1) % MATERIALS.len();
        }

        if button_pressed(0, BTN_FACE_RIGHT) != 0 {
            CURRENT_MATERIAL = if CURRENT_MATERIAL == 0 {
                MATERIALS.len() - 1
            } else {
                CURRENT_MATERIAL - 1
            };
        }

        // Toggle between single material and all materials display
        if button_pressed(0, BTN_FACE_UP) != 0 {
            SHOW_ALL_MATERIALS = !SHOW_ALL_MATERIALS;
        }

        // Update light positions based on rotation
        let angle_rad = LIGHT_ANGLE * PI / 180.0;
        let cos = cosf(angle_rad);
        let sin = sinf(angle_rad);

        // Rotate lights in horizontal plane
        light_set(0, -cos * 0.7, -0.2, -sin * 0.7);
        light_set(1, cos * 0.7, -0.2, -sin * 0.7);
        light_set(2, -cos * 0.7, -0.2, sin * 0.7);
        light_set(3, cos * 0.7, -0.2, sin * 0.7);
    }
}

#[no_mangle]
pub extern "C" fn render() {
    unsafe {

        camera_set(
            0.0, 0.0, 8.0,  // eye - reasonable distance to see all 6 spheres
            0.0, 0.0, 0.0,   // center
        );
        camera_fov(60.0);

        if SHOW_ALL_MATERIALS {
            // Display all materials in a grid
            let spacing = 3.0;
            let positions = [
                [-spacing, spacing, 0.0],    // Top-left
                [0.0, spacing, 0.0],          // Top-center
                [spacing, spacing, 0.0],      // Top-right
                [-spacing, 0.0, 0.0], // Center-left
                [0.0, 0.0, 0.0], // Center
                [spacing, 0.0, 0.0], // Center-right
                [-spacing, -spacing, 0.0],    // Bottom-left
                [0.0, -spacing, 0.0],         // Bottom-center
                [spacing, -spacing, 0.0],     // Bottom-right
            ];

            for (i, material) in MATERIALS.iter().enumerate() {
                draw_sphere_with_material(positions[i], 1.2, material);
            }
        } else {
            // Display single material with rotation
            let material = &MATERIALS[CURRENT_MATERIAL];
            draw_sphere_with_material([0.0, 0.0, 0.0], 2.0, material);
        }
    }
}

// ============================================================================
// Rendering Helpers
// ============================================================================

fn draw_sphere_with_material(position: [f32; 3], radius: f32, material: &Material) {
    unsafe {
        // Set material properties using pre-computed hex colors
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
