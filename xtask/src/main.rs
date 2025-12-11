mod cart;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Emberware build tasks
#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Emberware build and development tasks")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[allow(clippy::large_enum_variant)]
enum Commands {
    /// Build and install example games
    BuildExamples,

    /// Cart management (create, inspect ROMs)
    Cart {
        #[command(subcommand)]
        command: cart::CartCommand,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::BuildExamples => build_examples(),
        Commands::Cart { command } => cart::execute(command),
    }
}

fn build_examples() -> Result<()> {
    let project_root = project_root();
    let examples_dir = project_root.join("examples");
    let games_dir = get_games_dir()?;

    // Ensure games directory exists
    fs::create_dir_all(&games_dir).context("Failed to create games directory")?;

    println!("Games directory: {}", games_dir.display());
    println!();

    // Get all example directories that have a Cargo.toml (are buildable)
    let examples: Vec<_> = fs::read_dir(&examples_dir)
        .context("Failed to read examples directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            path.is_dir() && path.join("Cargo.toml").exists()
        })
        .collect();

    println!("Building {} examples...", examples.len());
    println!();

    let mut success_count = 0;
    let mut fail_count = 0;
    let mut skipped_count = 0;

    for example in examples {
        let example_path = example.path();
        let example_name = example.file_name();
        let example_name_str = example_name.to_string_lossy();

        // Check if this is a data pack example (has ember.toml)
        let ember_toml_path = example_path.join("ember.toml");
        let has_ember_toml = ember_toml_path.exists();

        println!("Building {}...", example_name_str);

        // Build the example to WASM
        let build_status = Command::new("cargo")
            .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
            .current_dir(&example_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .status();

        match build_status {
            Ok(status) if status.success() => {
                // Find the built WASM file
                let target_dir = example_path.join("target/wasm32-unknown-unknown/release");

                let wasm_file = fs::read_dir(&target_dir).ok().and_then(|entries| {
                    entries.filter_map(|e| e.ok()).find(|e| {
                        e.path()
                            .extension()
                            .and_then(|ext| ext.to_str())
                            .map(|ext| ext == "wasm")
                            .unwrap_or(false)
                    })
                });

                if let Some(wasm_file) = wasm_file {
                    if has_ember_toml {
                        // Data pack example - use ember pack
                        match build_with_ember_pack(
                            &example_path,
                            &wasm_file.path(),
                            &games_dir,
                            &example_name_str,
                        ) {
                            Ok(_) => {
                                println!("  ✓ Installed (with data pack)");
                                success_count += 1;
                            }
                            Err(e) => {
                                // Check if it's just missing assets
                                let err_str = e.to_string();
                                if err_str.contains("Failed to load")
                                    || err_str.contains("No such file")
                                {
                                    println!(
                                        "  ⊘ Skipped (missing assets - this is a template example)"
                                    );
                                    skipped_count += 1;
                                } else {
                                    println!("  ✗ Pack failed: {}", e);
                                    fail_count += 1;
                                }
                            }
                        }
                    } else {
                        // Standard WASM-only example
                        install_wasm_example(&wasm_file.path(), &games_dir, &example_name_str)?;
                        println!("  ✓ Installed");
                        success_count += 1;
                    }
                } else {
                    println!("  ✗ No WASM file found");
                    fail_count += 1;
                }
            }
            Ok(_) => {
                println!("  ✗ Build failed");
                fail_count += 1;
            }
            Err(e) => {
                println!("  ✗ Build error: {}", e);
                fail_count += 1;
            }
        }
    }

    println!();
    println!(
        "Done! {} succeeded, {} skipped, {} failed",
        success_count, skipped_count, fail_count
    );
    println!("Examples installed to: {}", games_dir.display());
    println!("You can now run 'cargo run' to play them in Emberware Z.");

    if skipped_count > 0 {
        println!();
        println!("Note: Skipped examples are templates that demonstrate data pack usage.");
        println!("      Add assets to their assets/ folder and rebuild to use them.");
    }

    Ok(())
}

/// Install a WASM-only example (no data pack)
fn install_wasm_example(wasm_path: &Path, games_dir: &Path, example_name: &str) -> Result<()> {
    // Create game directory
    let game_dir = games_dir.join(example_name);
    fs::create_dir_all(&game_dir)?;

    // Copy WASM file to rom.wasm
    let rom_path = game_dir.join("rom.wasm");
    fs::copy(wasm_path, &rom_path).context("Failed to copy WASM file")?;

    // Create manifest.json
    let title = example_name
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let manifest = serde_json::json!({
        "id": example_name,
        "title": title,
        "author": "Emberware Examples",
        "version": "0.1.0",
        "downloaded_at": chrono::Utc::now().to_rfc3339()
    });

    let manifest_path = game_dir.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .context("Failed to write manifest")?;

    Ok(())
}

/// Build a data pack example using ember pack
fn build_with_ember_pack(
    example_path: &Path,
    wasm_path: &Path,
    games_dir: &Path,
    example_name: &str,
) -> Result<()> {
    // Try to find ember CLI (built from this workspace)
    let project_root = project_root();
    let ember_path = project_root.join("target/release/ember");
    let ember_path_debug = project_root.join("target/debug/ember");

    // Check if ember is built, if not build it
    let ember_exe = if ember_path.exists() {
        ember_path
    } else if ember_path_debug.exists() {
        ember_path_debug
    } else {
        println!("    Building ember CLI...");
        let status = Command::new("cargo")
            .args(["build", "--release", "-p", "ember-cli"])
            .current_dir(&project_root)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .context("Failed to build ember CLI")?;

        if !status.success() {
            anyhow::bail!("Failed to build ember CLI");
        }
        ember_path
    };

    // Create game directory
    let game_dir = games_dir.join(example_name);
    fs::create_dir_all(&game_dir)?;

    // Output path for the ROM
    let rom_output = game_dir.join("rom.ewz");

    // Run ember pack
    let output = Command::new(&ember_exe)
        .args([
            "pack",
            "--manifest",
            "ember.toml",
            "--wasm",
            &wasm_path.to_string_lossy(),
            "-o",
            &rom_output.to_string_lossy(),
        ])
        .current_dir(example_path)
        .output()
        .context("Failed to run ember pack")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("{}", stderr.trim());
    }

    // Create manifest.json
    let title = example_name
        .replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let manifest = serde_json::json!({
        "id": example_name,
        "title": title,
        "author": "Emberware Examples",
        "version": "0.1.0",
        "downloaded_at": chrono::Utc::now().to_rfc3339()
    });

    let manifest_path = game_dir.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .context("Failed to write manifest")?;

    Ok(())
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn get_games_dir() -> Result<PathBuf> {
    directories::ProjectDirs::from("io", "emberware", "emberware")
        .map(|dirs| dirs.data_dir().join("games"))
        .context("Failed to determine data directory")
}
