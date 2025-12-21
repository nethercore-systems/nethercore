mod cart;
mod ffi;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Nethercore build tasks
#[derive(Parser)]
#[command(name = "xtask")]
#[command(about = "Nethercore build and development tasks")]
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

    /// FFI binding generation (C/Zig from Rust)
    Ffi {
        #[command(subcommand)]
        command: ffi::FfiCommand,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::BuildExamples => build_examples(),
        Commands::Cart { command } => cart::execute(command),
        Commands::Ffi { command } => ffi::execute(command),
    }
}

fn build_examples() -> Result<()> {
    let project_root = project_root();
    let examples_dir = project_root.join("examples");
    let games_dir = get_games_dir()?;

    // Ensure games directory exists
    fs::create_dir_all(&games_dir).context("Failed to create games directory")?;

    println!("Games directory: {}", games_dir.display());

    // Run asset generators first
    run_asset_generators(&project_root)?;

    // Build nether CLI first if needed
    let nether_exe = ensure_nether_cli(&project_root)?;

    // Library crates that shouldn't be built as standalone examples
    let skip_dirs = ["inspector-common", "examples-common"];

    // Get all example directories that have a Cargo.toml (are buildable)
    let examples: Vec<_> = fs::read_dir(&examples_dir)
        .context("Failed to read examples directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let path = entry.path();
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            path.is_dir()
                && path.join("Cargo.toml").exists()
                && !skip_dirs.contains(&name_str.as_ref())
        })
        .collect();

    println!("Building {} examples...", examples.len());

    let success_count = AtomicUsize::new(0);
    let fail_count = AtomicUsize::new(0);
    let skipped_count = AtomicUsize::new(0);

    examples.par_iter().for_each(|example| {
        let example_path = example.path();
        let example_name = example.file_name();
        let example_name_str = example_name.to_string_lossy();

        // Check if this example has nether.toml
        let nether_toml_path = example_path.join("nether.toml");
        let has_nether_toml = nether_toml_path.exists();

        println!("Building {}...", example_name_str);

        if has_nether_toml {
            // Use nether build (compile + pack)
            match build_with_nether(&nether_exe, &example_path, &games_dir, &example_name_str) {
                Ok(_) => {
                    println!("  ✓ {} installed", example_name_str);
                    success_count.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    // Check if it's just missing assets (template example)
                    let err_str = e.to_string();
                    if err_str.contains("Failed to load") || err_str.contains("No such file") {
                        println!(
                            "  ⊘ {} skipped (missing assets - template example)",
                            example_name_str
                        );
                        skipped_count.fetch_add(1, Ordering::Relaxed);
                    } else {
                        println!("  ✗ {} failed: {}", example_name_str, e);
                        fail_count.fetch_add(1, Ordering::Relaxed);
                    }
                }
            }
        } else {
            // No nether.toml - use legacy WASM-only installation
            match build_wasm_only(&example_path, &games_dir, &example_name_str) {
                Ok(_) => {
                    println!(
                        "  ✓ {} installed (WASM-only, no nether.toml)",
                        example_name_str
                    );
                    success_count.fetch_add(1, Ordering::Relaxed);
                }
                Err(e) => {
                    println!("  ✗ {} failed: {}", example_name_str, e);
                    fail_count.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    });

    println!();
    println!(
        "Done! {} succeeded, {} skipped, {} failed",
        success_count.load(Ordering::Relaxed),
        skipped_count.load(Ordering::Relaxed),
        fail_count.load(Ordering::Relaxed)
    );
    println!("Examples installed to: {}", games_dir.display());
    println!("You can now run 'cargo run' to play them in Nethercore ZX.");

    if skipped_count.load(Ordering::Relaxed) > 0 {
        println!("Note: Skipped examples are templates that demonstrate data pack usage.");
        println!("      Add assets to their assets/ folder and rebuild to use them.");
    }

    Ok(())
}

/// Run all asset generator tools (tools/gen-*)
fn run_asset_generators(project_root: &Path) -> Result<()> {
    let tools_dir = project_root.join("tools");

    if !tools_dir.exists() {
        return Ok(());
    }

    // Find all gen-* directories in tools/
    let generators: Vec<_> = fs::read_dir(&tools_dir)
        .context("Failed to read tools directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            entry.path().is_dir()
                && name_str.starts_with("gen-")
                && entry.path().join("Cargo.toml").exists()
        })
        .collect();

    if generators.is_empty() {
        return Ok(());
    }

    println!(
        "Running {} asset generator(s) in parallel...",
        generators.len()
    );

    // Run generators in parallel and collect results
    let results: Vec<_> = generators
        .par_iter()
        .map(|generator| {
            let name = generator.file_name();
            let name_str = name.to_string_lossy().to_string();

            let output = Command::new("cargo")
                .args(["run", "-p", &name_str, "--release"])
                .current_dir(project_root)
                .output();

            match output {
                Ok(output) if output.status.success() => (name_str, Ok(())),
                Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    let error = stderr.lines().last().unwrap_or("unknown error").to_string();
                    (name_str, Err(error))
                }
                Err(e) => (name_str, Err(format!("Failed to run: {}", e))),
            }
        })
        .collect();

    // Print results after all generators complete
    for (name, result) in results {
        match result {
            Ok(()) => println!("  ✓ {} completed", name),
            Err(e) => println!("  ✗ {} failed: {}", name, e),
        }
    }

    println!();
    Ok(())
}

/// Ensure nether CLI is built and return path to executable
fn ensure_nether_cli(project_root: &Path) -> Result<PathBuf> {
    let nether_path = project_root.join("target/release/nether");
    let nether_path_debug = project_root.join("target/debug/nether");

    // Add .exe extension on Windows
    #[cfg(windows)]
    let nether_path = nether_path.with_extension("exe");
    #[cfg(windows)]
    let nether_path_debug = nether_path_debug.with_extension("exe");

    if nether_path.exists() {
        return Ok(nether_path);
    }
    if nether_path_debug.exists() {
        return Ok(nether_path_debug);
    }

    println!("Building nether CLI...");
    let status = Command::new("cargo")
        .args(["build", "--release", "-p", "nether-cli"])
        .current_dir(project_root)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .context("Failed to build nether CLI")?;

    if !status.success() {
        anyhow::bail!("Failed to build nether CLI");
    }

    Ok(nether_path)
}

/// Build an example using nether build (compile + pack)
fn build_with_nether(
    nether_exe: &Path,
    example_path: &Path,
    games_dir: &Path,
    example_name: &str,
) -> Result<()> {
    // Create game directory
    let game_dir = games_dir.join(example_name);
    fs::create_dir_all(&game_dir)?;

    // Output path for the ROM
    let rom_output = game_dir.join("rom.nczx");

    // Run nether build (compile + pack)
    let output = Command::new(nether_exe)
        .args([
            "build",
            "--manifest",
            "nether.toml",
            "-o",
            &rom_output.to_string_lossy(),
        ])
        .current_dir(example_path)
        .output()
        .context("Failed to run nether build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let error_msg = if !stderr.is_empty() {
            stderr.trim().to_string()
        } else if !stdout.is_empty() {
            stdout.trim().to_string()
        } else {
            "Unknown error".to_string()
        };
        anyhow::bail!("{}", error_msg);
    }

    // Create manifest.json for game library
    let title = title_case(example_name);

    let manifest = serde_json::json!({
        "id": example_name,
        "title": title,
        "author": "Nethercore Examples",
        "version": "0.1.0",
        "downloaded_at": chrono::Utc::now().to_rfc3339()
    });

    let manifest_path = game_dir.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .context("Failed to write manifest")?;

    Ok(())
}

/// Build a WASM-only example (no nether.toml)
fn build_wasm_only(example_path: &Path, games_dir: &Path, example_name: &str) -> Result<()> {
    // Build the example to WASM
    let status = Command::new("cargo")
        .args(["build", "--target", "wasm32-unknown-unknown", "--release"])
        .current_dir(example_path)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .status()
        .context("Failed to run cargo build")?;

    if !status.success() {
        anyhow::bail!("Cargo build failed");
    }

    // Find the built WASM file
    let target_dir = example_path.join("target/wasm32-unknown-unknown/release");
    let wasm_file = fs::read_dir(&target_dir)
        .context("Failed to read target directory")?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "wasm")
                .unwrap_or(false)
        })
        .map(|e| e.path())
        .context("No WASM file found in target directory")?;

    // Create game directory
    let game_dir = games_dir.join(example_name);
    fs::create_dir_all(&game_dir)?;

    // Copy WASM file to rom.wasm
    let rom_path = game_dir.join("rom.wasm");
    fs::copy(&wasm_file, &rom_path).context("Failed to copy WASM file")?;

    // Create manifest.json
    let title = title_case(example_name);

    let manifest = serde_json::json!({
        "id": example_name,
        "title": title,
        "author": "Nethercore Examples",
        "version": "0.1.0",
        "downloaded_at": chrono::Utc::now().to_rfc3339()
    });

    let manifest_path = game_dir.join("manifest.json");
    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
        .context("Failed to write manifest")?;

    Ok(())
}

/// Convert example name to title case
fn title_case(name: &str) -> String {
    name.replace('-', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn project_root() -> PathBuf {
    Path::new(&env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn get_games_dir() -> Result<PathBuf> {
    directories::ProjectDirs::from("io.nethercore", "", "Nethercore")
        .map(|dirs| dirs.data_dir().join("games"))
        .context("Failed to determine data directory")
}
