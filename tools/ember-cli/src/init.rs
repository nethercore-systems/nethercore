//! Init command - create a new ember.toml manifest
//!
//! Creates a default manifest file with helpful comments explaining each field.

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

/// Arguments for the init command
#[derive(Args)]
pub struct InitArgs {
    /// Path to project directory (defaults to current directory)
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Game ID (defaults to directory name)
    #[arg(long)]
    pub id: Option<String>,

    /// Game title
    #[arg(long)]
    pub title: Option<String>,

    /// Overwrite existing ember.toml
    #[arg(long)]
    pub force: bool,
}

/// Execute the init command
pub fn execute(args: InitArgs) -> Result<()> {
    let project_dir = args
        .path
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let manifest_path = project_dir.join("ember.toml");

    // Check if manifest already exists
    if manifest_path.exists() && !args.force {
        anyhow::bail!(
            "ember.toml already exists at {}\nUse --force to overwrite",
            manifest_path.display()
        );
    }

    // Derive defaults from project directory
    let dir_name = project_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-game")
        .to_string();

    let game_id = args.id.unwrap_or_else(|| {
        // Convert to kebab-case
        dir_name
            .chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .trim_matches('-')
            .to_string()
    });

    let game_title = args.title.unwrap_or_else(|| {
        // Convert to Title Case
        dir_name
            .split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    });

    // Detect if this is a Rust project
    let is_rust_project = project_dir.join("Cargo.toml").exists();

    // Generate manifest content
    let manifest_content = generate_manifest(&game_id, &game_title, is_rust_project);

    // Write manifest
    std::fs::write(&manifest_path, &manifest_content)
        .with_context(|| format!("Failed to write manifest: {}", manifest_path.display()))?;

    println!("Created ember.toml");
    println!("  Game ID: {}", game_id);
    println!("  Title: {}", game_title);

    if is_rust_project {
        println!("  Detected Rust project - using cargo build defaults");
    }

    println!();
    println!("Next steps:");
    println!("  1. Edit ember.toml to customize your game metadata");
    println!("  2. Add render_mode() call in your game's init() function");
    println!("  3. Run 'ember build' to compile and pack your game");

    Ok(())
}

/// Generate manifest content with helpful comments
fn generate_manifest(game_id: &str, game_title: &str, is_rust_project: bool) -> String {
    let mut content = String::new();

    // Header
    content.push_str("# Ember Game Manifest\n");
    content
        .push_str("# See: https://github.com/emberware-io/emberware/docs/book/\n");
    content.push_str("\n");

    // Game section
    content.push_str("[game]\n");
    content.push_str(&format!("id = \"{}\"\n", game_id));
    content.push_str(&format!("title = \"{}\"\n", game_title));
    content.push_str("author = \"Your Name\"\n");
    content.push_str("version = \"0.1.0\"\n");
    content.push_str("\n");
    content.push_str("# Optional metadata\n");
    content.push_str("# description = \"A short description of your game\"\n");
    content.push_str("# tags = [\"action\", \"puzzle\"]\n");
    content.push_str("\n");

    // Build section
    content.push_str("# Build Configuration\n");
    content.push_str("# Defaults work for standard Rust projects. Customize for Zig, C, etc.\n");
    content.push_str("#\n");

    if is_rust_project {
        content.push_str("# [build]\n");
        content.push_str("# script = \"cargo build --target wasm32-unknown-unknown --release\"\n");
        content.push_str("# wasm = \"target/wasm32-unknown-unknown/release/your_game.wasm\"\n");
    } else {
        content.push_str("[build]\n");
        content.push_str("# Uncomment and modify for your build system:\n");
        content.push_str("#\n");
        content.push_str("# Rust (default if omitted):\n");
        content.push_str("# script = \"cargo build --target wasm32-unknown-unknown --release\"\n");
        content.push_str("#\n");
        content.push_str("# Zig:\n");
        content.push_str("# script = \"zig build -Doptimize=ReleaseFast\"\n");
        content.push_str("# wasm = \"zig-out/bin/game.wasm\"\n");
        content.push_str("#\n");
        content.push_str("# C/C++ (with wasi-sdk):\n");
        content.push_str("# script = \"make release\"\n");
        content.push_str("# wasm = \"build/game.wasm\"\n");
    }
    content.push_str("\n");

    // Render mode explanation
    content.push_str("# Render Mode\n");
    content.push_str("# Set in your game code by calling render_mode(N) in init():\n");
    content.push_str("#   0 = Unlit (RGBA8 textures, no lighting)\n");
    content.push_str("#   1 = Matcap (stylized lighting via matcap texture)\n");
    content.push_str("#   2 = PBR Metallic-Roughness (physically based rendering)\n");
    content.push_str("#   3 = Blinn-Phong Specular-Shininess (classic lighting)\n");
    content.push_str("#\n");
    content.push_str("# Textures are automatically compressed based on render mode:\n");
    content.push_str("#   Mode 0: RGBA8 (uncompressed)\n");
    content.push_str("#   Mode 1-3: BC7 (4x compression)\n");
    content.push_str("\n");

    // Assets section
    content.push_str("# Assets\n");
    content.push_str("# Declare assets to bundle into the .ewz cartridge.\n");
    content.push_str("# Load in game code with rom_texture(), rom_mesh(), etc.\n");
    content.push_str("#\n");
    content.push_str("# [[assets.textures]]\n");
    content.push_str("# id = \"player\"        # ID used in rom_texture(\"player\")\n");
    content.push_str("# path = \"assets/player.png\"\n");
    content.push_str("#\n");
    content.push_str("# [[assets.meshes]]\n");
    content.push_str("# id = \"level\"\n");
    content.push_str("# path = \"assets/level.ewzmesh\"\n");
    content.push_str("#\n");
    content.push_str("# [[assets.sounds]]\n");
    content.push_str("# id = \"jump\"\n");
    content.push_str("# path = \"assets/jump.wav\"\n");
    content.push_str("#\n");
    content.push_str("# [[assets.keyframes]]\n");
    content.push_str("# id = \"walk\"\n");
    content.push_str("# path = \"assets/walk.ewzanim\"\n");
    content.push_str("#\n");
    content.push_str("# [[assets.data]]\n");
    content.push_str("# id = \"levels\"\n");
    content.push_str("# path = \"assets/levels.bin\"\n");

    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_generate_manifest_rust() {
        let content = generate_manifest("my-game", "My Game", true);
        assert!(content.contains("id = \"my-game\""));
        assert!(content.contains("title = \"My Game\""));
        assert!(content.contains("# [build]")); // Commented out for Rust
    }

    #[test]
    fn test_generate_manifest_non_rust() {
        let content = generate_manifest("zig-game", "Zig Game", false);
        assert!(content.contains("id = \"zig-game\""));
        assert!(content.contains("[build]")); // Uncommented for non-Rust
        assert!(content.contains("# Zig:"));
    }

    #[test]
    fn test_init_creates_file() {
        let dir = tempdir().unwrap();
        let args = InitArgs {
            path: Some(dir.path().to_path_buf()),
            id: Some("test-game".to_string()),
            title: Some("Test Game".to_string()),
            force: false,
        };

        execute(args).unwrap();

        let manifest_path = dir.path().join("ember.toml");
        assert!(manifest_path.exists());

        let content = std::fs::read_to_string(&manifest_path).unwrap();
        assert!(content.contains("id = \"test-game\""));
        assert!(content.contains("title = \"Test Game\""));
    }

    #[test]
    fn test_init_fails_if_exists() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("ember.toml");
        std::fs::write(&manifest_path, "existing").unwrap();

        let args = InitArgs {
            path: Some(dir.path().to_path_buf()),
            id: None,
            title: None,
            force: false,
        };

        let result = execute(args);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("already exists"));
    }

    #[test]
    fn test_init_force_overwrites() {
        let dir = tempdir().unwrap();
        let manifest_path = dir.path().join("ember.toml");
        std::fs::write(&manifest_path, "existing").unwrap();

        let args = InitArgs {
            path: Some(dir.path().to_path_buf()),
            id: Some("new-game".to_string()),
            title: None,
            force: true,
        };

        execute(args).unwrap();

        let content = std::fs::read_to_string(&manifest_path).unwrap();
        assert!(content.contains("id = \"new-game\""));
    }
}
