//! Blinn-Phong Material Gallery (Mode 3)
//!
//! Demonstrates:
//! - Normalized Blinn-Phong lighting (Gotanda 2010)
//! - 66 material presets in 11×6 grid:
//!   - Row 1: Natural Materials (organic surfaces)
//!   - Row 2: Realistic Metals (conductors)
//!   - Row 3: Synthetic & Industrial (man-made)
//!   - Row 4: Geological (rocks, gems, minerals)
//!   - Row 5: Fantasy & Magical (enchanted materials)
//!   - Row 6: Alien & Impossible (sci-fi/surreal)
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

const MATERIALS: [Material; 66] = [
    // =========================================================================
    // ROW 1: NATURAL MATERIALS (indices 0-10)
    // Organic surfaces from living things
    // =========================================================================

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
    // Wet Skin - moist flesh, subsurface scattering look
    Material {
        name: "Wet Skin",
        albedo: 0xD9B3A6FF,
        specular: 0xE6CCBFFF,
        shininess: 0.7,
        rim_intensity: 0.3,
        rim_power: 0.25,
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
    // Moss - soft green, extremely matte
    Material {
        name: "Moss",
        albedo: 0x4D6633FF,
        specular: 0x334422FF,
        shininess: 0.08,
        rim_intensity: 0.1,
        rim_power: 0.4,
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
    // Feather - iridescent shimmer, dark base
    Material {
        name: "Feather",
        albedo: 0x334455FF,
        specular: 0x6688AAFF,
        shininess: 0.5,
        rim_intensity: 0.2,
        rim_power: 0.2,
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
    // Coral - pink organic calcium
    Material {
        name: "Coral",
        albedo: 0xE67373FF,
        specular: 0xFF9999FF,
        shininess: 0.35,
        rim_intensity: 0.15,
        rim_power: 0.25,
        emissive: 0.0,
    },

    // =========================================================================
    // ROW 2: REALISTIC METALS (indices 11-21)
    // Conductors with colored reflections
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
        albedo: 0xE6E6E6FF,
        specular: 0xF2F2F2FF,
        shininess: 0.85,
        rim_intensity: 0.15,
        rim_power: 0.12,
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
    // Brass - warm yellow alloy
    Material {
        name: "Brass",
        albedo: 0xB5A642FF,
        specular: 0xDDCC66FF,
        shininess: 0.6,
        rim_intensity: 0.2,
        rim_power: 0.15,
        emissive: 0.0,
    },
    // Aluminum - matte silvery, distinct from chrome
    Material {
        name: "Aluminum",
        albedo: 0xA0A0A8FF,
        specular: 0xC0C0C8FF,
        shininess: 0.5,
        rim_intensity: 0.1,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Titanium - dark gray with blue tint
    Material {
        name: "Titanium",
        albedo: 0x5A6068FF,
        specular: 0x9099A8FF,
        shininess: 0.7,
        rim_intensity: 0.15,
        rim_power: 0.15,
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
    // ROW 3: SYNTHETIC & INDUSTRIAL (indices 22-32)
    // Man-made materials
    // =========================================================================

    // Matte Plastic - neutral diffuse
    Material {
        name: "Matte Plastic",
        albedo: 0x80808CFF,
        specular: 0x60606AFF,
        shininess: 0.3,
        rim_intensity: 0.0,
        rim_power: 0.0,
        emissive: 0.0,
    },
    // Glossy Plastic - shiny red toy
    Material {
        name: "Glossy Plastic",
        albedo: 0xCC3333FF,
        specular: 0xFFAAAAFF,
        shininess: 0.75,
        rim_intensity: 0.15,
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
    // Glass - transparent feel, strong rim
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
    // Concrete - rough matte gray
    Material {
        name: "Concrete",
        albedo: 0x808080FF,
        specular: 0x505050FF,
        shininess: 0.1,
        rim_intensity: 0.0,
        rim_power: 0.0,
        emissive: 0.0,
    },
    // Carbon Fiber - dark woven pattern look
    Material {
        name: "Carbon Fiber",
        albedo: 0x202020FF,
        specular: 0x606060FF,
        shininess: 0.6,
        rim_intensity: 0.15,
        rim_power: 0.2,
        emissive: 0.0,
    },
    // Vinyl - purple record material
    Material {
        name: "Vinyl",
        albedo: 0x6633AAFF,
        specular: 0xAA88EEFF,
        shininess: 0.7,
        rim_intensity: 0.2,
        rim_power: 0.15,
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

    // =========================================================================
    // ROW 4: GEOLOGICAL (indices 33-43)
    // Rocks, minerals, gems
    // =========================================================================

    // Stone - generic gray rock
    Material {
        name: "Stone",
        albedo: 0x6B6B6BFF,
        specular: 0x4A4A4AFF,
        shininess: 0.15,
        rim_intensity: 0.05,
        rim_power: 0.3,
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
    // Crystal - clear quartz, prismatic
    Material {
        name: "Crystal",
        albedo: 0xE8F0F8FF,
        specular: 0xFFFFFFFF,
        shininess: 0.9,
        rim_intensity: 0.4,
        rim_power: 0.1,
        emissive: 0.05,
    },
    // Amethyst - purple crystal gem
    Material {
        name: "Amethyst",
        albedo: 0x6B3FA0FF,
        specular: 0xAA77EEFF,
        shininess: 0.8,
        rim_intensity: 0.35,
        rim_power: 0.15,
        emissive: 0.08,
    },
    // Ice - frozen water, blue-white
    Material {
        name: "Ice",
        albedo: 0xC8E8FFFF,
        specular: 0xE8FFFFFF,
        shininess: 0.85,
        rim_intensity: 0.45,
        rim_power: 0.12,
        emissive: 0.0,
    },
    // Ruby - deep red precious gem
    Material {
        name: "Ruby",
        albedo: 0x991133FF,
        specular: 0xFF4466FF,
        shininess: 0.88,
        rim_intensity: 0.4,
        rim_power: 0.12,
        emissive: 0.1,
    },

    // =========================================================================
    // ROW 5: FANTASY & MAGICAL (indices 44-54)
    // Enchanted and supernatural materials
    // =========================================================================

    // Enchanted - golden magic glow
    Material {
        name: "Enchanted",
        albedo: 0xB8962EFF,
        specular: 0xFFDD88FF,
        shininess: 0.75,
        rim_intensity: 0.5,
        rim_power: 0.15,
        emissive: 0.25,
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
        name: "Dragon Scale",
        albedo: 0x2D5A2DFF,
        specular: 0x66FF66FF,
        shininess: 0.8,
        rim_intensity: 0.35,
        rim_power: 0.15,
        emissive: 0.05,
    },
    // Fairy Dust - sparkling pink magic
    Material {
        name: "Fairy Dust",
        albedo: 0xFFB6C1FF,
        specular: 0xFFDDEEFF,
        shininess: 0.7,
        rim_intensity: 0.5,
        rim_power: 0.12,
        emissive: 0.35,
    },
    // Blood Moon - deep crimson lunar
    Material {
        name: "Blood Moon",
        albedo: 0x660022FF,
        specular: 0xFF3344FF,
        shininess: 0.65,
        rim_intensity: 0.55,
        rim_power: 0.12,
        emissive: 0.2,
    },
    // Starlight - celestial white radiance
    Material {
        name: "Starlight",
        albedo: 0xE8E0F0FF,
        specular: 0xFFFFFFFF,
        shininess: 0.85,
        rim_intensity: 0.6,
        rim_power: 0.1,
        emissive: 0.4,
    },
    // Arcane - deep magical purple energy
    Material {
        name: "Arcane",
        albedo: 0x2A1A4AFF,
        specular: 0x7744CCFF,
        shininess: 0.7,
        rim_intensity: 0.55,
        rim_power: 0.1,
        emissive: 0.25,
    },
    // Ice Magic - frozen spell effect
    Material {
        name: "Ice Magic",
        albedo: 0x88CCFFFF,
        specular: 0xCCFFFFFF,
        shininess: 0.85,
        rim_intensity: 0.6,
        rim_power: 0.08,
        emissive: 0.3,
    },
    // Phoenix - blazing rebirth fire
    Material {
        name: "Phoenix",
        albedo: 0xFF4400FF,
        specular: 0xFFCC00FF,
        shininess: 0.65,
        rim_intensity: 0.85,
        rim_power: 0.08,
        emissive: 0.65,
    },

    // =========================================================================
    // ROW 6: ALIEN & IMPOSSIBLE (indices 55-65)
    // Sci-fi and surreal materials
    // =========================================================================

    // Holographic - vibrant iridescent display
    Material {
        name: "Holographic",
        albedo: 0x99DDFFFF,
        specular: 0xFFAAFFFF,
        shininess: 0.85,
        rim_intensity: 0.6,
        rim_power: 0.08,
        emissive: 0.35,
    },
    // Plasma - electric blue energy
    Material {
        name: "Plasma",
        albedo: 0x2244AAFF,
        specular: 0x88DDFFFF,
        shininess: 0.8,
        rim_intensity: 0.7,
        rim_power: 0.1,
        emissive: 0.5,
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
    // Alien Flesh - organic pink-purple
    Material {
        name: "Alien Flesh",
        albedo: 0x8B4466FF,
        specular: 0xCC88AAFF,
        shininess: 0.55,
        rim_intensity: 0.4,
        rim_power: 0.2,
        emissive: 0.1,
    },
    // Bioluminescent - deep sea creature glow
    Material {
        name: "Bioluminescent",
        albedo: 0x116655FF,
        specular: 0x44FFCCFF,
        shininess: 0.7,
        rim_intensity: 0.75,
        rim_power: 0.08,
        emissive: 0.55,
    },
    // Nebula - cosmic purple gas cloud
    Material {
        name: "Nebula",
        albedo: 0x332255FF,
        specular: 0xAA66FFFF,
        shininess: 0.5,
        rim_intensity: 0.55,
        rim_power: 0.1,
        emissive: 0.3,
    },
    // Black Hole - event horizon darkness
    Material {
        name: "Black Hole",
        albedo: 0x000000FF,
        specular: 0x110011FF,
        shininess: 0.98,
        rim_intensity: 0.8,
        rim_power: 0.05,
        emissive: 0.0,
    },
    // Antimatter - inverse reality
    Material {
        name: "Antimatter",
        albedo: 0x1A1A2EFF,
        specular: 0xFF44FFFF,
        shininess: 0.85,
        rim_intensity: 0.7,
        rim_power: 0.08,
        emissive: 0.25,
    },
    // Living Metal - organic chrome T-1000
    Material {
        name: "Living Metal",
        albedo: 0x607080FF,
        specular: 0xAABBCCFF,
        shininess: 0.88,
        rim_intensity: 0.4,
        rim_power: 0.12,
        emissive: 0.1,
    },
    // Neon - cyberpunk glowing sign
    Material {
        name: "Neon",
        albedo: 0xFF0066FF,
        specular: 0xFF88CCFF,
        shininess: 0.7,
        rim_intensity: 0.8,
        rim_power: 0.06,
        emissive: 0.7,
    },
];

// Row category names
const ROW_NAMES: [&str; 6] = [
    "NATURAL",
    "METALS",
    "SYNTHETIC",
    "GEOLOGICAL",
    "FANTASY",
    "ALIEN",
];

// ============================================================================
// Grid Layout Constants
// ============================================================================

const COLS: usize = 11;
const ROWS: usize = 6;
const SPACING_X: f32 = 2.0;
const SPACING_Y: f32 = 2.2;
const SPHERE_RADIUS: f32 = 0.8;

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

        // Set Mode 3 (Blinn-Phong)
        render_mode(3);

        // Setup rotating lights
        light_set(0, -0.7, -0.2, -0.7);
        light_color(0, 0xFF4D4DFF);
        light_intensity(0, 0.6);

        light_set(1, 0.7, -0.2, -0.7);
        light_color(1, 0x4DFF4DFF);
        light_intensity(1, 0.6);

        light_set(2, -0.7, -0.2, 0.7);
        light_color(2, 0x4D4DFFFF);
        light_intensity(2, 0.6);

        light_set(3, 0.7, -0.2, 0.7);
        light_color(3, 0xFFFF4DFF);
        light_intensity(3, 0.6);

        // Generate lower-poly sphere (16x8 segments)
        SPHERE_MESH = sphere(1.0, 16, 8);
    }
}

#[no_mangle]
pub extern "C" fn update() {
    unsafe {
        // Rotate lights slowly
        LIGHT_ANGLE += 0.5;

        // Rotate spheres
        ROTATION += 0.5;

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
        // Camera to fit 11x6 grid - fill the screen
        camera_set(
            0.0, 0.0, 10.0,  // eye
            0.0, 0.0, 0.0,   // center
        );
        camera_fov(60.0);

        // Calculate grid offsets (center the grid)
        let grid_width = (COLS - 1) as f32 * SPACING_X;
        let grid_height = (ROWS - 1) as f32 * SPACING_Y;
        let start_x = -grid_width / 2.0;
        let start_y = grid_height / 2.0;

        // Draw all 66 materials
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
        let text_size = 0.012;
        let label_color = 0xCCCCCCFF;
        let header_color = 0xFFCC66FF;
        let header_size = 0.018;

        // Row headers (left side)
        for row in 0..ROWS {
            let y = start_y - row as f32 * SPACING_Y;
            let screen_x = -0.98;
            let screen_y = y * 0.052;

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

                let screen_x = x * 0.052 - 0.04;
                let screen_y = y * 0.052 - 0.05;

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
