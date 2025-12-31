//! ZX-specific asset preview implementation
//!
//! This module provides the preview mode for Nethercore ZX ROMs, allowing
//! developers to inspect and visualize bundled assets without running the game.
//!
//! # Usage
//!
//! ```bash
//! nethercore-zx game.nczx --preview
//! nethercore-zx game.nczx --preview --asset textures/player
//! ```

mod loader;
pub mod viewers;

pub use loader::ZXPreviewLoader;

use anyhow::Result;

/// Configuration for the preview application
#[derive(Debug, Clone)]
pub struct PreviewConfig {
    /// Path to the ROM file to preview
    pub rom_path: std::path::PathBuf,

    /// Optional specific asset to focus on (e.g., "textures/player")
    pub asset_path: Option<String>,

    /// Window scale factor
    pub scale: u32,
}

impl Default for PreviewConfig {
    fn default() -> Self {
        Self {
            rom_path: std::path::PathBuf::new(),
            scale: 2,
            asset_path: None,
        }
    }
}

/// Metadata about a ROM for preview display
#[derive(Debug, Clone)]
pub struct PreviewMetadata {
    /// Game ID/slug
    pub id: String,

    /// Display title
    pub title: String,

    /// Author name
    pub author: String,

    /// Version string
    pub version: String,
}

/// Container for loaded preview data
#[derive(Debug)]
pub struct PreviewData<D> {
    /// The loaded data pack
    pub data_pack: D,

    /// ROM metadata
    pub metadata: PreviewMetadata,
}

/// Asset categories for navigation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AssetCategory {
    Textures,
    Meshes,
    Sounds,
    Trackers,
    Animations,
    Skeletons,
    Fonts,
    Data,
}

impl AssetCategory {
    /// Get all categories in display order
    pub fn all() -> &'static [AssetCategory] {
        &[
            AssetCategory::Textures,
            AssetCategory::Meshes,
            AssetCategory::Sounds,
            AssetCategory::Trackers,
            AssetCategory::Animations,
            AssetCategory::Skeletons,
            AssetCategory::Fonts,
            AssetCategory::Data,
        ]
    }

    /// Get display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            AssetCategory::Textures => "Textures",
            AssetCategory::Meshes => "Meshes",
            AssetCategory::Sounds => "Sounds",
            AssetCategory::Trackers => "Trackers",
            AssetCategory::Animations => "Animations",
            AssetCategory::Skeletons => "Skeletons",
            AssetCategory::Fonts => "Fonts",
            AssetCategory::Data => "Data",
        }
    }

    /// Get icon for the category (using ASCII for now)
    pub fn icon(&self) -> &'static str {
        match self {
            AssetCategory::Textures => "[T]",
            AssetCategory::Meshes => "[M]",
            AssetCategory::Sounds => "[S]",
            AssetCategory::Trackers => "[X]",
            AssetCategory::Animations => "[A]",
            AssetCategory::Skeletons => "[K]",
            AssetCategory::Fonts => "[F]",
            AssetCategory::Data => "[D]",
        }
    }
}

/// Run the ZX preview application
///
/// This is the main entry point for preview mode. It loads the ROM,
/// creates the preview window, and runs the asset viewer UI.
pub fn run(config: PreviewConfig) -> Result<()> {
    use nethercore_core::app::event_loop;
    use nethercore_core::app::preview::{PreviewApp, PreviewConfig as CorePreviewConfig};

    // Convert config to core format
    let core_config = CorePreviewConfig {
        rom_path: config.rom_path,
        initial_asset: config.asset_path,
    };

    // Create and run the preview app
    let app = PreviewApp::<crate::console::NethercoreZX, ZXPreviewLoader>::new(core_config);
    event_loop::run(app)?;

    Ok(())
}

/// Parse an asset path like "textures/player" into (category, id)
fn parse_asset_path(path: &str) -> Option<(AssetCategory, String)> {
    let parts: Vec<&str> = path.splitn(2, '/').collect();
    if parts.len() != 2 {
        return None;
    }

    let category = match parts[0].to_lowercase().as_str() {
        "textures" | "texture" | "tex" => AssetCategory::Textures,
        "meshes" | "mesh" => AssetCategory::Meshes,
        "sounds" | "sound" | "sfx" => AssetCategory::Sounds,
        "trackers" | "tracker" | "music" | "xm" => AssetCategory::Trackers,
        "animations" | "animation" | "anim" => AssetCategory::Animations,
        "skeletons" | "skeleton" | "skel" => AssetCategory::Skeletons,
        "fonts" | "font" => AssetCategory::Fonts,
        "data" | "raw" => AssetCategory::Data,
        _ => return None,
    };

    Some((category, parts[1].to_string()))
}
