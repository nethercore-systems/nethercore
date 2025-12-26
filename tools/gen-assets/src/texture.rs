//! Convention-based texture generation system
//!
//! Texture Naming Convention:
//! - `{id}.png` - Base/albedo texture (always generated)
//! - `{id}_emissive.png` - Emissive texture (Mode 2 PBR, optional)
//!
//! This module provides a unified way to define asset textures with their
//! visual properties, and generates all required textures automatically.

use proc_gen::texture::*;
use std::path::Path;

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
}

/// Asset texture definition - everything needed to generate textures for one asset
#[derive(Clone)]
pub struct AssetTexture {
    /// Asset ID (matches mesh ID)
    pub id: &'static str,
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
    pub const fn solid(id: &'static str, color: [u8; 4]) -> Self {
        Self {
            id,
            base_color: color,
            style: TextureStyle::Solid,
            size: (64, 64),
            secondary_color: None,
            emissive: None,
        }
    }

    /// Create a metal texture
    pub const fn metal(id: &'static str, color: [u8; 4], seed: u32) -> Self {
        Self {
            id,
            base_color: color,
            style: TextureStyle::Metal { seed },
            size: (128, 128),
            secondary_color: None,
            emissive: None,
        }
    }

    /// Create a stone texture
    pub const fn stone(id: &'static str, color: [u8; 4], seed: u32) -> Self {
        Self {
            id,
            base_color: color,
            style: TextureStyle::Stone { seed },
            size: (128, 128),
            secondary_color: None,
            emissive: None,
        }
    }

    /// Create a vertical gradient texture
    pub const fn gradient_v(id: &'static str, top: [u8; 4], bottom: [u8; 4]) -> Self {
        Self {
            id,
            base_color: top,
            style: TextureStyle::GradientV,
            size: (64, 64),
            secondary_color: Some(bottom),
            emissive: None,
        }
    }

    /// Create a radial gradient texture
    pub const fn gradient_radial(id: &'static str, center: [u8; 4], edge: [u8; 4]) -> Self {
        Self {
            id,
            base_color: center,
            style: TextureStyle::GradientRadial,
            size: (64, 64),
            secondary_color: Some(edge),
            emissive: None,
        }
    }

    /// Create a checker pattern texture
    pub const fn checker(id: &'static str, color1: [u8; 4], color2: [u8; 4], cell_size: u32) -> Self {
        Self {
            id,
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
