//! Nether.toml manifest parsing
//!
//! Shared manifest structures used by compile, pack, and build commands.

use anyhow::{Context, Result};
use nethercore_shared::console::TickRate;
use nethercore_shared::is_safe_game_id;
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
    #[serde(default)]
    pub netplay: NetplaySection,
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

    /// Enable BC7 texture compression (4:1 ratio).
    /// Recommended for Matcap/BP render modes.
    /// Default: false (uncompressed RGBA8)
    #[serde(default)]
    pub compress_textures: bool,

    /// Render mode: 0=Lambert, 1=Matcap, 2=MRBP, 3=SSBP
    /// Default: 0 (Lambert)
    #[serde(default)]
    pub render_mode: u8,

    /// Tick rate in Hz for netplay (30, 60, or 120).
    /// Must be consistent for rollback netcode.
    /// Default: 60
    #[serde(default = "default_tick_rate")]
    pub tick_rate: u32,

    /// Maximum players supported (1-4).
    /// Default: 4 (multiplayer is Nethercore's core feature)
    #[serde(default = "default_max_players")]
    pub max_players: u8,
}

fn default_tick_rate() -> u32 {
    60
}

fn default_max_players() -> u8 {
    4
}

/// Netplay configuration section
#[derive(Debug, Deserialize)]
pub struct NetplaySection {
    /// Whether this game supports online netplay.
    /// Default: true (multiplayer is Nethercore's core feature)
    #[serde(default = "default_netplay_enabled")]
    pub enabled: bool,
}

impl Default for NetplaySection {
    fn default() -> Self {
        Self { enabled: true }
    }
}

fn default_netplay_enabled() -> bool {
    true
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
    /// Asset ID. Required for most assets, but optional for wildcard animation imports.
    ///
    /// For animations: if both `id` and `animation_name` are omitted, all animations
    /// from the file are imported using their original names (with optional `id_prefix`).
    #[serde(default)]
    pub id: Option<String>,

    /// Path to the asset file (relative to nether.toml)
    pub path: String,

    /// Animation name to extract from GLB file (for animations/keyframes).
    /// If not specified with an explicit `id`, uses the first animation.
    /// If both `id` and `animation_name` are omitted, imports ALL animations.
    #[serde(default)]
    pub animation_name: Option<String>,

    /// Skin name to extract from GLB file (for skeletons and skinned meshes).
    /// If not specified, uses the first skin in the file.
    #[serde(default)]
    pub skin_name: Option<String>,

    /// ID prefix for wildcard animation imports.
    /// Applied to each extracted animation: `{prefix}{animation_name}`
    /// Use this to prevent collisions when importing from multiple GLB files.
    #[serde(default)]
    pub id_prefix: Option<String>,

    /// For tracker assets: whether to include patterns (default: true).
    /// Set to false to use XM file only as a sample library without
    /// registering it as a playable tracker.
    #[serde(default)]
    pub patterns: Option<bool>,
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

        // Find candidate .wasm files (note: cargo won't clean up old outputs when crates are renamed).
        let mut wasm_files: Vec<PathBuf> = std::fs::read_dir(&target_dir)
            .with_context(|| format!("Failed to read target directory: {}", target_dir.display()))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "wasm")
                    .unwrap_or(false)
            })
            .collect();

        if wasm_files.is_empty() {
            anyhow::bail!(
                "No WASM file found in {}\nRun 'nether compile' first.",
                target_dir.display()
            );
        }

        if wasm_files.len() == 1 {
            return Ok(wasm_files.remove(0));
        }

        // Prefer a file that matches the game id (cargo outputs underscores for '-' in crate names).
        let expected_stem = self.game.id.replace('-', "_");
        let expected_name = format!("{}.wasm", expected_stem);
        if let Some(path) = wasm_files
            .iter()
            .find(|p| p.file_name().and_then(|n| n.to_str()) == Some(expected_name.as_str()))
        {
            return Ok(path.clone());
        }

        // Otherwise, prefer the newest file (best-effort heuristic).
        wasm_files.sort_by(|a, b| {
            let a_time = std::fs::metadata(a).and_then(|m| m.modified()).ok();
            let b_time = std::fs::metadata(b).and_then(|m| m.modified()).ok();
            b_time.cmp(&a_time)
        });

        eprintln!(
            "Warning: multiple .wasm outputs found in {} but none match game.id='{}'. \
Consider setting build.wasm in nether.toml. Using newest: {}",
            target_dir.display(),
            self.game.id,
            wasm_files[0].display()
        );

        Ok(wasm_files.remove(0))
    }

    /// Validate manifest fields
    pub fn validate(&self) -> Result<()> {
        if !is_safe_game_id(&self.game.id) {
            anyhow::bail!(
                "Invalid game.id '{}' in nether.toml (must be a safe single path component)",
                self.game.id
            );
        }

        if self.game.render_mode > 3 {
            anyhow::bail!(
                "Invalid render_mode {} in nether.toml (must be 0-3: 0=Lambert, 1=Matcap, 2=PBR, 3=Hybrid)",
                self.game.render_mode
            );
        }

        // Validate tick_rate
        if TickRate::from_hz(self.game.tick_rate).is_none() {
            anyhow::bail!(
                "Invalid tick_rate {} in nether.toml (must be 30, 60, or 120)",
                self.game.tick_rate
            );
        }

        // Validate max_players
        if self.game.max_players < 1 || self.game.max_players > 4 {
            anyhow::bail!(
                "Invalid max_players {} in nether.toml (must be 1-4)",
                self.game.max_players
            );
        }

        // Warn if netplay enabled but max_players is 1
        if self.netplay.enabled && self.game.max_players == 1 {
            eprintln!(
                "Warning: netplay.enabled=true but max_players=1. Consider setting max_players >= 2 for multiplayer."
            );
        }

        Ok(())
    }

    /// Get the validated TickRate enum
    pub fn tick_rate(&self) -> TickRate {
        TickRate::from_hz(self.game.tick_rate).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

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
    fn test_manifest_invalid_game_id() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "../evil"
title = "Test Game"
author = "Test Author"
version = "1.0.0"
"#,
        )
        .unwrap();

        assert!(manifest.validate().is_err());
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
        assert_eq!(manifest.assets.textures[0].id, Some("player".to_string()));
        assert_eq!(manifest.assets.meshes.len(), 1);
        assert_eq!(manifest.assets.meshes[0].id, Some("level".to_string()));
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

    #[test]
    fn test_render_mode_explicit() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test"
title = "Test"
author = "Author"
version = "1.0.0"
render_mode = 2
"#,
        )
        .unwrap();

        assert_eq!(manifest.game.render_mode, 2);
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_render_mode_default() {
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

        assert_eq!(manifest.game.render_mode, 0); // Default to Lambert
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_render_mode_invalid() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test"
title = "Test"
author = "Author"
version = "1.0.0"
render_mode = 5
"#,
        )
        .unwrap();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_netplay_fields_default() {
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

        assert_eq!(manifest.game.tick_rate, 60);
        assert_eq!(manifest.game.max_players, 4); // Multiplayer is Nethercore's core feature
        assert!(manifest.netplay.enabled); // Netplay enabled by default
        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_netplay_fields_explicit() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "fighter"
title = "Fighter"
author = "Author"
version = "1.0.0"
tick_rate = 120
max_players = 4

[netplay]
enabled = true
"#,
        )
        .unwrap();

        assert_eq!(manifest.game.tick_rate, 120);
        assert_eq!(manifest.game.max_players, 4);
        assert!(manifest.netplay.enabled);
        assert!(manifest.validate().is_ok());
        assert_eq!(manifest.tick_rate(), TickRate::Fixed120);
    }

    #[test]
    fn test_tick_rate_invalid() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test"
title = "Test"
author = "Author"
version = "1.0.0"
tick_rate = 45
"#,
        )
        .unwrap();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_max_players_invalid_zero() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test"
title = "Test"
author = "Author"
version = "1.0.0"
max_players = 0
"#,
        )
        .unwrap();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_max_players_invalid_five() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "test"
title = "Test"
author = "Author"
version = "1.0.0"
max_players = 5
"#,
        )
        .unwrap();

        assert!(manifest.validate().is_err());
    }

    #[test]
    fn test_find_wasm_prefers_game_id_match_when_multiple() {
        let manifest = NetherManifest::parse(
            r#"
[game]
id = "epu-showcase"
title = "Test"
author = "Author"
version = "0.1.0"
"#,
        )
        .unwrap();

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("nether_find_wasm_test_{}", unique));
        let target_dir = temp_dir.join("target/wasm32-unknown-unknown/release");
        std::fs::create_dir_all(&target_dir).unwrap();

        // Write a "wrong" wasm after the correct one to make it newer.
        let correct = target_dir.join("epu_showcase.wasm");
        std::fs::write(&correct, b"").unwrap();
        let wrong = target_dir.join("epu_inspector.wasm");
        std::fs::write(&wrong, b"").unwrap();

        let found = manifest.find_wasm(&temp_dir, false).unwrap();
        assert_eq!(found, correct);

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
