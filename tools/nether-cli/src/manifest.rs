//! Nether.toml manifest parsing
//!
//! Shared manifest structures used by compile, pack, and build commands.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Nether.toml manifest structure
#[derive(Debug, Deserialize)]
pub struct NetherManifest {
    pub game: GameSection,
    #[serde(default)]
    pub build: BuildSection,
    #[serde(default)]
    pub assets: AssetsSection,
}

/// Game metadata section
#[derive(Debug, Deserialize)]
pub struct GameSection {
    pub id: String,
    pub title: String,
    pub author: String,
    pub version: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Build configuration section
#[derive(Debug, Default, Deserialize)]
pub struct BuildSection {
    /// Build script to execute (e.g., "cargo build --target wasm32-unknown-unknown --release")
    pub script: Option<String>,
    /// Path to WASM output file
    pub wasm: Option<String>,
}

/// Assets section containing all asset declarations
#[derive(Debug, Default, Deserialize)]
pub struct AssetsSection {
    #[serde(default)]
    pub textures: Vec<AssetEntry>,
    #[serde(default)]
    pub meshes: Vec<AssetEntry>,
    #[serde(default)]
    pub skeletons: Vec<AssetEntry>,
    #[serde(default)]
    pub keyframes: Vec<AssetEntry>,
    #[serde(default)]
    pub animations: Vec<AssetEntry>, // Alias for keyframes
    #[serde(default)]
    pub sounds: Vec<AssetEntry>,
    #[serde(default)]
    pub trackers: Vec<AssetEntry>,
    #[serde(default)]
    pub data: Vec<AssetEntry>,
}

/// Single asset entry
#[derive(Debug, Deserialize)]
pub struct AssetEntry {
    pub id: String,
    pub path: String,
}

impl NetherManifest {
    /// Load manifest from file
    pub fn load(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read manifest: {}", path.display()))?;
        Self::parse(&content)
    }

    /// Parse manifest from string
    pub fn parse(content: &str) -> Result<Self> {
        toml::from_str(content).context("Failed to parse nether.toml")
    }

    /// Get the build script, with default for Rust projects
    pub fn build_script(&self, debug: bool) -> String {
        if let Some(script) = &self.build.script {
            // If user specified a script, handle debug mode substitution
            if debug && script.contains("--release") {
                script.replace("--release", "")
            } else {
                script.clone()
            }
        } else {
            // Default: cargo build for wasm32
            if debug {
                "cargo build --target wasm32-unknown-unknown".to_string()
            } else {
                "cargo build --target wasm32-unknown-unknown --release".to_string()
            }
        }
    }

    /// Find the WASM file path
    ///
    /// Priority:
    /// 1. Explicit path from manifest build.wasm
    /// 2. Auto-detect from target directory
    pub fn find_wasm(&self, project_dir: &Path, debug: bool) -> Result<PathBuf> {
        // 1. Check explicit path
        if let Some(wasm_path) = &self.build.wasm {
            let path = project_dir.join(wasm_path);
            if path.exists() {
                return Ok(path);
            }
            anyhow::bail!("WASM file not found at specified path: {}", path.display());
        }

        // 2. Auto-detect from target directory
        let profile = if debug { "debug" } else { "release" };
        let target_dir = project_dir.join(format!("target/wasm32-unknown-unknown/{}/", profile));

        if !target_dir.exists() {
            anyhow::bail!(
                "Target directory not found: {}\nRun 'nether compile' first.",
                target_dir.display()
            );
        }

        // Find .wasm file
        let wasm_file = std::fs::read_dir(&target_dir)
            .with_context(|| format!("Failed to read target directory: {}", target_dir.display()))?
            .filter_map(|e| e.ok())
            .find(|e| {
                e.path()
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "wasm")
                    .unwrap_or(false)
            })
            .map(|e| e.path());

        wasm_file.ok_or_else(|| {
            anyhow::anyhow!(
                "No WASM file found in {}\nRun 'nether compile' first.",
                target_dir.display()
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_minimal() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test-game"
title = "Test Game"
author = "Test Author"
version = "1.0.0"
"#,
        )
        .unwrap();

        assert_eq!(manifest.game.id, "test-game");
        assert_eq!(manifest.game.title, "Test Game");
        assert!(manifest.build.script.is_none());
        assert!(manifest.assets.textures.is_empty());
    }

    #[test]
    fn test_manifest_with_build_section() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "zig-game"
title = "Zig Game"
author = "Author"
version = "1.0.0"

[build]
script = "zig build -Drelease"
wasm = "zig-out/bin/game.wasm"
"#,
        )
        .unwrap();

        assert_eq!(
            manifest.build.script,
            Some("zig build -Drelease".to_string())
        );
        assert_eq!(
            manifest.build.wasm,
            Some("zig-out/bin/game.wasm".to_string())
        );
    }

    #[test]
    fn test_manifest_with_assets() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "asset-game"
title = "Asset Game"
author = "Author"
version = "0.1.0"

[[assets.textures]]
id = "player"
path = "assets/player.png"

[[assets.meshes]]
id = "level"
path = "assets/level.nczxmesh"
"#,
        )
        .unwrap();

        assert_eq!(manifest.assets.textures.len(), 1);
        assert_eq!(manifest.assets.textures[0].id, "player");
        assert_eq!(manifest.assets.meshes.len(), 1);
        assert_eq!(manifest.assets.meshes[0].id, "level");
    }

    #[test]
    fn test_build_script_default() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test"
title = "Test"
author = "Author"
version = "1.0.0"
"#,
        )
        .unwrap();

        assert_eq!(
            manifest.build_script(false),
            "cargo build --target wasm32-unknown-unknown --release"
        );
        assert_eq!(
            manifest.build_script(true),
            "cargo build --target wasm32-unknown-unknown"
        );
    }

    #[test]
    fn test_build_script_custom() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test"
title = "Test"
author = "Author"
version = "1.0.0"

[build]
script = "zig build -Drelease"
"#,
        )
        .unwrap();

        assert_eq!(manifest.build_script(false), "zig build -Drelease");
    }
}
