//! Manifest parsing and build orchestration
//!
//! Parses assets.toml and coordinates asset conversion.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::{EWZ_MESH_EXT, EWZ_SOUND_EXT, EWZ_TEXTURE_EXT};

/// Root manifest structure
#[derive(Debug, Deserialize)]
pub struct Manifest {
    #[serde(default)]
    pub output: OutputConfig,
    #[serde(default)]
    pub codegen: Option<CodegenConfig>,
    #[serde(default)]
    pub meshes: HashMap<String, MeshEntry>,
    #[serde(default)]
    pub textures: HashMap<String, TextureEntry>,
    #[serde(default)]
    pub sounds: HashMap<String, SoundEntry>,
    #[serde(default)]
    pub fonts: HashMap<String, FontEntry>,
}

#[derive(Debug, Deserialize, Default)]
pub struct OutputConfig {
    #[serde(default = "default_output_dir")]
    pub dir: PathBuf,
}

fn default_output_dir() -> PathBuf {
    PathBuf::from("assets/")
}

#[derive(Debug, Deserialize)]
pub struct CodegenConfig {
    pub rust: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum MeshEntry {
    Simple(PathBuf),
    Detailed {
        path: PathBuf,
        #[serde(default)]
        format: Option<String>,
    },
}

impl MeshEntry {
    pub fn path(&self) -> &Path {
        match self {
            MeshEntry::Simple(p) => p,
            MeshEntry::Detailed { path, .. } => path,
        }
    }

    pub fn format(&self) -> Option<&str> {
        match self {
            MeshEntry::Simple(_) => None,
            MeshEntry::Detailed { format, .. } => format.as_deref(),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum TextureEntry {
    Simple(PathBuf),
    Detailed { path: PathBuf },
}

impl TextureEntry {
    pub fn path(&self) -> &Path {
        match self {
            TextureEntry::Simple(p) => p,
            TextureEntry::Detailed { path } => path,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum SoundEntry {
    Simple(PathBuf),
    Detailed { path: PathBuf },
}

impl SoundEntry {
    pub fn path(&self) -> &Path {
        match self {
            SoundEntry::Simple(p) => p,
            SoundEntry::Detailed { path } => path,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum FontEntry {
    Simple(PathBuf),
    Detailed {
        path: PathBuf,
        #[serde(default = "default_font_size")]
        #[allow(dead_code)]
        size: u32,
    },
}

fn default_font_size() -> u32 {
    16
}

impl FontEntry {
    pub fn path(&self) -> &Path {
        match self {
            FontEntry::Simple(p) => p,
            FontEntry::Detailed { path, .. } => path,
        }
    }

    #[allow(dead_code)]
    pub fn size(&self) -> u32 {
        match self {
            FontEntry::Simple(_) => default_font_size(),
            FontEntry::Detailed { size, .. } => *size,
        }
    }
}

/// Load and parse a manifest file
pub fn load_manifest(path: &Path) -> Result<Manifest> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read manifest: {:?}", path))?;
    let manifest: Manifest = toml::from_str(&content)
        .with_context(|| format!("Failed to parse manifest: {:?}", path))?;
    Ok(manifest)
}

/// Validate a manifest without building
pub fn validate(manifest: &Manifest) -> Result<()> {
    // Check that all source files exist
    for (name, entry) in &manifest.meshes {
        if !entry.path().exists() {
            anyhow::bail!("Mesh '{}' source not found: {:?}", name, entry.path());
        }
    }
    for (name, entry) in &manifest.textures {
        if !entry.path().exists() {
            anyhow::bail!("Texture '{}' source not found: {:?}", name, entry.path());
        }
    }
    for (name, entry) in &manifest.sounds {
        if !entry.path().exists() {
            anyhow::bail!("Sound '{}' source not found: {:?}", name, entry.path());
        }
    }
    for (name, entry) in &manifest.fonts {
        if !entry.path().exists() {
            anyhow::bail!("Font '{}' source not found: {:?}", name, entry.path());
        }
    }
    Ok(())
}

/// Build all assets from a manifest
pub fn build_all(manifest: &Manifest, output_override: Option<&Path>) -> Result<()> {
    let output_dir = output_override.unwrap_or(&manifest.output.dir);
    std::fs::create_dir_all(output_dir)?;

    // Convert meshes
    for (name, entry) in &manifest.meshes {
        let output = output_dir.join(format!("{}.{}", name, EWZ_MESH_EXT));
        tracing::info!("Converting mesh: {} -> {:?}", name, output);

        // Detect format by extension
        let ext = entry
            .path()
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_lowercase())
            .unwrap_or_default();

        match ext.as_str() {
            "obj" => crate::mesh::convert_obj(entry.path(), &output, entry.format())?,
            "gltf" | "glb" => crate::mesh::convert_gltf(entry.path(), &output, entry.format())?,
            _ => anyhow::bail!("Unsupported mesh format for '{}': {:?}", name, entry.path()),
        }
    }

    // Convert textures
    for (name, entry) in &manifest.textures {
        let output = output_dir.join(format!("{}.{}", name, EWZ_TEXTURE_EXT));
        tracing::info!("Converting texture: {} -> {:?}", name, output);
        crate::texture::convert_image(entry.path(), &output)?;
    }

    // Convert sounds
    for (name, entry) in &manifest.sounds {
        let output = output_dir.join(format!("{}.{}", name, EWZ_SOUND_EXT));
        tracing::info!("Converting sound: {} -> {:?}", name, output);
        crate::audio::convert_wav(entry.path(), &output)?;
    }

    // Fonts are deferred
    if !manifest.fonts.is_empty() {
        tracing::warn!(
            "Font conversion not yet implemented, skipping {} fonts",
            manifest.fonts.len()
        );
    }

    // Generate Rust code if configured
    if let Some(codegen) = &manifest.codegen {
        if let Some(rust_path) = &codegen.rust {
            tracing::info!("Generating Rust module: {:?}", rust_path);
            crate::codegen::generate_rust_module(manifest, rust_path)?;
        }
    }

    Ok(())
}
