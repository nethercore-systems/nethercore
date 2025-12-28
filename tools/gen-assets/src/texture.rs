//! Convention-based texture generation system
//!
//! Texture Naming Convention:
//! - `{id}.png` - Base/albedo texture (always generated)
//! - `{id}_emissive.png` - Emissive texture (Mode 2 PBR, optional)
//!
//! This module provides a unified way to define asset textures with their
//! visual properties, and generates all required textures automatically.

use proc_gen::texture::*;
use noise::{NoiseFn, Perlin, Worley};
use std::path::Path;

/// Asset category for filtering and organization
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetCategory {
    Hero,
    Enemy,
    Elite,
    Boss,
    Pickup,
    Arena,
    Projectile,
    Effect,
    UI,
    Test,
}

/// Texture generation style
#[derive(Clone, Copy)]
pub enum TextureStyle {
    /// Solid color fill
    Solid,
    /// Brushed metal effect with seed for variation
    Metal { seed: u32 },
    /// Rocky/stone surface with seed for variation
    Stone { seed: u32 },
    /// Vertical gradient (top to bottom darkening)
    GradientV,
    /// Horizontal gradient
    GradientH,
    /// Radial gradient (center outward)
    GradientRadial,
    /// Checkerboard pattern
    Checker { cell_size: u32 },
    /// Crystal/voronoi pattern (uses secondary_color as edge color)
    Crystal { seed: u32 },
    /// Fish scale pattern - overlapping circular scales
    Scales { seed: u32 },
    /// Organic membrane - translucent with veins
    Organic { seed: u32 },
    /// Barnacle/encrusted surface - bumpy with deposits
    Barnacles { seed: u32 },
    /// Bioluminescent spots - dark base with glowing dots
    Bioluminescent { seed: u32 },
    /// Coral texture - ridged organic surface
    Coral { seed: u32 },
    /// Shell texture - nacreous/pearlescent layers
    Shell { seed: u32 },
    /// Kelp/seaweed - vertical organic stripes
    Seaweed { seed: u32 },
    /// Deep sea silt/sediment - fine layered particles
    Sediment { seed: u32 },
}

/// Asset texture definition - everything needed to generate textures for one asset
#[derive(Clone)]
pub struct AssetTexture {
    /// Asset ID (matches mesh ID)
    pub id: &'static str,
    /// Asset category for filtering
    pub category: AssetCategory,
    /// Base color RGBA
    pub base_color: [u8; 4],
    /// Texture generation style
    pub style: TextureStyle,
    /// Texture size (width, height)
    pub size: (u32, u32),
    /// Secondary color for gradients/patterns (optional)
    pub secondary_color: Option<[u8; 4]>,
    /// Emissive color (if Some, generates {id}_emissive.png)
    pub emissive: Option<[u8; 4]>,
}

impl AssetTexture {
    /// Create a simple solid color texture
    pub const fn solid(id: &'static str, category: AssetCategory, color: [u8; 4]) -> Self {
        Self {
            id,
            category,
            base_color: color,
            style: TextureStyle::Solid,
            size: (64, 64),
            secondary_color: None,
            emissive: None,
        }
    }

    /// Create a metal texture
    pub const fn metal(id: &'static str, category: AssetCategory, color: [u8; 4], seed: u32) -> Self {
        Self {
            id,
            category,
            base_color: color,
            style: TextureStyle::Metal { seed },
            size: (128, 128),
            secondary_color: None,
            emissive: None,
        }
    }

    /// Create a stone texture
    pub const fn stone(id: &'static str, category: AssetCategory, color: [u8; 4], seed: u32) -> Self {
        Self {
            id,
            category,
            base_color: color,
            style: TextureStyle::Stone { seed },
            size: (128, 128),
            secondary_color: None,
            emissive: None,
        }
    }

    /// Create a vertical gradient texture
    pub const fn gradient_v(id: &'static str, category: AssetCategory, top: [u8; 4], bottom: [u8; 4]) -> Self {
        Self {
            id,
            category,
            base_color: top,
            style: TextureStyle::GradientV,
            size: (64, 64),
            secondary_color: Some(bottom),
            emissive: None,
        }
    }

    /// Create a radial gradient texture
    pub const fn gradient_radial(id: &'static str, category: AssetCategory, center: [u8; 4], edge: [u8; 4]) -> Self {
        Self {
            id,
            category,
            base_color: center,
            style: TextureStyle::GradientRadial,
            size: (64, 64),
            secondary_color: Some(edge),
            emissive: None,
        }
    }

    /// Create a checker pattern texture
    pub const fn checker(id: &'static str, category: AssetCategory, color1: [u8; 4], color2: [u8; 4], cell_size: u32) -> Self {
        Self {
            id,
            category,
            base_color: color1,
            style: TextureStyle::Checker { cell_size },
            size: (64, 64),
            secondary_color: Some(color2),
            emissive: None,
        }
    }

    /// Builder: set texture size
    pub const fn with_size(mut self, width: u32, height: u32) -> Self {
        self.size = (width, height);
        self
    }

    /// Builder: add emissive texture
    pub const fn with_emissive(mut self, color: [u8; 4]) -> Self {
        self.emissive = Some(color);
        self
    }

    /// Builder: set category
    pub const fn with_category(mut self, category: AssetCategory) -> Self {
        self.category = category;
        self
    }

    /// Generate all textures for this asset
    pub fn generate(&self, output_dir: &Path) {
        let (width, height) = self.size;
        let secondary = self.secondary_color.unwrap_or(self.base_color);

        // Generate base texture
        let base_tex = match self.style {
            TextureStyle::Solid => solid(width, height, self.base_color),
            TextureStyle::Metal { seed } => metal(width, height, self.base_color, seed),
            TextureStyle::Stone { seed } => stone(width, height, self.base_color, seed),
            TextureStyle::GradientV => gradient_v(width, height, self.base_color, secondary),
            TextureStyle::GradientH => gradient_h(width, height, self.base_color, secondary),
            TextureStyle::GradientRadial => gradient_radial(width, height, self.base_color, secondary),
            TextureStyle::Checker { cell_size } => checker(width, height, cell_size, self.base_color, secondary),
            TextureStyle::Crystal { seed } => crystal(width, height, self.base_color, secondary, seed),
            TextureStyle::Scales { seed } => generate_scales(width, height, self.base_color, secondary, seed),
            TextureStyle::Organic { seed } => generate_organic(width, height, self.base_color, secondary, seed),
            TextureStyle::Barnacles { seed } => generate_barnacles(width, height, self.base_color, secondary, seed),
            TextureStyle::Bioluminescent { seed } => generate_bioluminescent(width, height, self.base_color, secondary, seed),
            TextureStyle::Coral { seed } => generate_coral(width, height, self.base_color, secondary, seed),
            TextureStyle::Shell { seed } => generate_shell(width, height, self.base_color, secondary, seed),
            TextureStyle::Seaweed { seed } => generate_seaweed(width, height, self.base_color, secondary, seed),
            TextureStyle::Sediment { seed } => generate_sediment(width, height, self.base_color, secondary, seed),
        };

        let base_path = output_dir.join(format!("{}.png", self.id));
        write_png(&base_tex, &base_path).expect("Failed to write base texture");
        println!("    -> {}", base_path.display());

        // Generate emissive texture if specified
        if let Some(emissive_color) = self.emissive {
            let emissive_tex = solid(width / 2, height / 2, emissive_color);
            let emissive_path = output_dir.join(format!("{}_emissive.png", self.id));
            write_png(&emissive_tex, &emissive_path).expect("Failed to write emissive texture");
            println!("    -> {}", emissive_path.display());
        }
    }

    /// Check if this asset has an emissive texture
    pub fn has_emissive(&self) -> bool {
        self.emissive.is_some()
    }
}

/// Generate all textures for a list of asset definitions
pub fn generate_all_textures(assets: &[AssetTexture], output_dir: &Path) {
    std::fs::create_dir_all(output_dir).expect("Failed to create texture directory");

    for asset in assets {
        asset.generate(output_dir);
    }
}

/// Generate textures filtered by category
pub fn generate_textures_by_category(assets: &[AssetTexture], category: AssetCategory, output_dir: &Path) {
    let filtered: Vec<_> = assets.iter().filter(|t| t.category == category).collect();
    println!("\n  Generating {} {} textures...", filtered.len(), category_name(category));
    std::fs::create_dir_all(output_dir).expect("Failed to create texture directory");
    for asset in filtered {
        asset.generate(output_dir);
    }
}

/// Generate textures filtered by multiple categories
pub fn generate_textures_by_categories(assets: &[AssetTexture], categories: &[AssetCategory], output_dir: &Path) {
    let filtered: Vec<_> = assets.iter().filter(|t| categories.contains(&t.category)).collect();
    println!("\n  Generating {} textures...", filtered.len());
    std::fs::create_dir_all(output_dir).expect("Failed to create texture directory");
    for asset in filtered {
        asset.generate(output_dir);
    }
}

/// Get human-readable category name
fn category_name(category: AssetCategory) -> &'static str {
    match category {
        AssetCategory::Hero => "hero",
        AssetCategory::Enemy => "enemy",
        AssetCategory::Elite => "elite",
        AssetCategory::Boss => "boss",
        AssetCategory::Pickup => "pickup",
        AssetCategory::Arena => "arena",
        AssetCategory::Projectile => "projectile",
        AssetCategory::Effect => "effect",
        AssetCategory::UI => "UI",
        AssetCategory::Test => "test",
    }
}

// =============================================================================
// UNDERWATER TEXTURE GENERATORS
// =============================================================================

/// Helper: Lerp between two colors
fn lerp_color(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    [
        (a[0] as f32 + (b[0] as f32 - a[0] as f32) * t) as u8,
        (a[1] as f32 + (b[1] as f32 - a[1] as f32) * t) as u8,
        (a[2] as f32 + (b[2] as f32 - a[2] as f32) * t) as u8,
        (a[3] as f32 + (b[3] as f32 - a[3] as f32) * t) as u8,
    ]
}

/// Generate fish scale pattern - overlapping circular scales
fn generate_scales(width: u32, height: u32, base: [u8; 4], highlight: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base);
    let perlin = Perlin::new(seed);

    // Scale parameters
    let scale_w = 8.0;
    let scale_h = 6.0;

    for y in 0..height {
        for x in 0..width {
            let fx = x as f32;
            let fy = y as f32;

            // Offset every other row for overlapping scales
            let row = (fy / scale_h) as i32;
            let offset = if row % 2 == 0 { 0.0 } else { scale_w * 0.5 };

            // Find position within scale cell
            let cx = ((fx + offset) % scale_w) - scale_w * 0.5;
            let cy = (fy % scale_h) - scale_h * 0.3; // Offset center up for overlap

            // Distance from scale center (elliptical)
            let dist = ((cx / scale_w).powi(2) + (cy / scale_h).powi(2)).sqrt();

            // Add some noise variation
            let noise = perlin.get([x as f64 * 0.1, y as f64 * 0.1]) as f32 * 0.1;

            // Create scale edge highlight
            let edge = (1.0 - dist * 2.0).max(0.0).min(1.0);
            let rim = ((dist - 0.3).abs() * 10.0).max(0.0).min(1.0);

            let t = (edge * 0.7 + rim * 0.3 + noise).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(base, highlight, t));
        }
    }

    buffer
}

/// Generate organic membrane texture - translucent with veins
fn generate_organic(width: u32, height: u32, base: [u8; 4], vein_color: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base);
    let perlin = Perlin::new(seed);
    let worley = Worley::new(seed.wrapping_add(1));

    for y in 0..height {
        for x in 0..width {
            let fx = x as f64;
            let fy = y as f64;

            // Voronoi for cellular structure
            let cell = worley.get([fx * 0.05, fy * 0.05]);

            // Perlin for organic variation
            let organic = perlin.get([fx * 0.03, fy * 0.03]);

            // Vein pattern using distance to cell edges
            let vein_strength = (1.0 - cell.abs()) * 2.0;
            let vein = (vein_strength - 0.3).max(0.0).min(1.0);

            // Combine with organic noise
            let t = ((vein * 0.6) as f32 + organic as f32 * 0.2 + 0.2).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(base, vein_color, t));
        }
    }

    buffer
}

/// Generate barnacle/encrusted surface - bumpy with deposits
fn generate_barnacles(width: u32, height: u32, base: [u8; 4], deposit: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base);
    let worley = Worley::new(seed);
    let perlin = Perlin::new(seed.wrapping_add(1));

    for y in 0..height {
        for x in 0..width {
            let fx = x as f64;
            let fy = y as f64;

            // Voronoi creates barnacle centers
            let cell_dist = worley.get([fx * 0.08, fy * 0.08]);

            // Create circular barnacle shapes
            let barnacle = (0.3 - cell_dist.abs()).max(0.0) * 3.0;

            // Add rim highlight
            let rim = ((cell_dist.abs() - 0.15).abs() * 10.0).max(0.0).min(1.0);

            // Add surface noise
            let noise = perlin.get([fx * 0.1, fy * 0.1]) * 0.2;

            let t = (barnacle as f32 * 0.6 + rim as f32 * 0.3 + noise as f32).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(base, deposit, t));
        }
    }

    buffer
}

/// Generate bioluminescent texture - dark base with glowing spots
fn generate_bioluminescent(width: u32, height: u32, dark: [u8; 4], glow: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, dark);
    let worley = Worley::new(seed);
    let perlin = Perlin::new(seed.wrapping_add(1));

    for y in 0..height {
        for x in 0..width {
            let fx = x as f64;
            let fy = y as f64;

            // Voronoi for spot centers (sparse)
            let cell_dist = worley.get([fx * 0.04, fy * 0.04]);

            // Create glowing spot falloff
            let spot = (0.2 - cell_dist.abs()).max(0.0) * 5.0;
            let spot_glow = spot.powi(2); // Sharper falloff

            // Add pulsing variation
            let pulse = (perlin.get([fx * 0.02, fy * 0.02]) * 0.5 + 0.5) as f32;

            let t = (spot_glow as f32 * pulse).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(dark, glow, t));
        }
    }

    buffer
}

/// Generate coral texture - ridged organic surface
fn generate_coral(width: u32, height: u32, base: [u8; 4], ridge: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base);
    let perlin = Perlin::new(seed);

    for y in 0..height {
        for x in 0..width {
            let fx = x as f64;
            let fy = y as f64;

            // Multi-frequency noise for organic ridges
            let n1 = perlin.get([fx * 0.05, fy * 0.05]);
            let n2 = perlin.get([fx * 0.1, fy * 0.1]) * 0.5;
            let n3 = perlin.get([fx * 0.2, fy * 0.2]) * 0.25;

            let combined = n1 + n2 + n3;

            // Create ridge pattern by taking absolute value
            let ridge_pattern = combined.abs();

            // Add brain coral-like grooves
            let groove = ((combined * 10.0).sin() * 0.5 + 0.5) as f32;

            let t = (ridge_pattern as f32 * 0.6 + groove * 0.4).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(base, ridge, t));
        }
    }

    buffer
}

/// Generate shell texture - nacreous/pearlescent layers
fn generate_shell(width: u32, height: u32, base: [u8; 4], nacre: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base);
    let perlin = Perlin::new(seed);

    for y in 0..height {
        for x in 0..width {
            let fx = x as f64;
            let fy = y as f64;

            // Concentric growth lines
            let center_x = width as f64 / 2.0;
            let center_y = height as f64 / 2.0;
            let dist = ((fx - center_x).powi(2) + (fy - center_y).powi(2)).sqrt();

            // Wavy growth lines with noise
            let wave = perlin.get([fx * 0.02, fy * 0.02]) * 5.0;
            let growth_line = ((dist * 0.3 + wave).sin() * 0.5 + 0.5) as f32;

            // Iridescent shimmer
            let shimmer = perlin.get([fx * 0.1, fy * 0.1]) as f32 * 0.3;

            let t = (growth_line * 0.7 + shimmer + 0.15).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(base, nacre, t));
        }
    }

    buffer
}

/// Generate seaweed/kelp texture - vertical organic stripes
fn generate_seaweed(width: u32, height: u32, base: [u8; 4], dark: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base);
    let perlin = Perlin::new(seed);

    for y in 0..height {
        for x in 0..width {
            let fx = x as f64;
            let fy = y as f64;

            // Vertical stripe with waviness
            let wave = perlin.get([fy * 0.03, 0.0]) * 8.0;
            let stripe = ((fx + wave) * 0.15).sin();

            // Add vertical variation (blade texture)
            let blade = perlin.get([fx * 0.05, fy * 0.02]) * 0.4;

            // Darker edges
            let edge = (stripe.abs() - 0.5).max(0.0) * 2.0;

            let t = (edge as f32 + blade as f32 + 0.3).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(base, dark, t));
        }
    }

    buffer
}

/// Generate sediment texture - fine layered particles
fn generate_sediment(width: u32, height: u32, base: [u8; 4], layer: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base);
    let perlin = Perlin::new(seed);

    for y in 0..height {
        for x in 0..width {
            let fx = x as f64;
            let fy = y as f64;

            // Horizontal layering
            let layer_noise = perlin.get([fx * 0.02, 0.0]) * 3.0;
            let layers = ((fy * 0.1 + layer_noise).sin() * 0.5 + 0.5) as f32;

            // Fine grain noise
            let grain = perlin.get([fx * 0.2, fy * 0.2]) as f32 * 0.3;

            // Larger deposits
            let deposits = perlin.get([fx * 0.05, fy * 0.05]) as f32 * 0.2;

            let t = (layers * 0.5 + grain + deposits + 0.2).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(base, layer, t));
        }
    }

    buffer
}
