use anyhow::{Context, Result};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: cargo xtask <command>");
        eprintln!("Commands:");
        eprintln!("  build-examples    Build and install example games");
        std::process::exit(1);
    }

    match args[1].as_str() {
        "build-examples" => build_examples(),
        cmd => {
            eprintln!("Unknown command: {}", cmd);
            std::process::exit(1);
        }
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

    // Get all example directories
    let examples = fs::read_dir(&examples_dir)
        .context("Failed to read examples directory")?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_dir())
        .collect::<Vec<_>>();

    println!("Building {} examples...", examples.len());
    println!();

    let mut success_count = 0;
    let mut fail_count = 0;

    for example in examples {
        let example_name = example.file_name();
        let example_name_str = example_name.to_string_lossy();

        println!("Building {}...", example_name_str);

        // Build the example to WASM
        let status = Command::new("cargo")
            .args(&["build", "--target", "wasm32-unknown-unknown", "--release"])
            .current_dir(example.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        match status {
            Ok(status) if status.success() => {
                // Find the built WASM file
                let target_dir = example.path().join("target/wasm32-unknown-unknown/release");

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
                    // Create game directory
                    let game_dir = games_dir.join(&example_name);
                    fs::create_dir_all(&game_dir)?;

                    // Copy WASM file to rom.wasm
                    let rom_path = game_dir.join("rom.wasm");
                    fs::copy(wasm_file.path(), &rom_path).context("Failed to copy WASM file")?;

                    // Create manifest.json
                    let title = example_name_str
                        .replace('-', " ")
                        .split_whitespace()
                        .map(|word| {
                            let mut chars = word.chars();
                            match chars.next() {
                                None => String::new(),
                                Some(first) => {
                                    first.to_uppercase().collect::<String>() + chars.as_str()
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(" ");

                    let manifest = serde_json::json!({
                        "id": example_name_str,
                        "title": title,
                        "author": "Emberware Examples",
                        "version": "0.1.0",
                        "downloaded_at": chrono::Utc::now().to_rfc3339()
                    });

                    let manifest_path = game_dir.join("manifest.json");
                    fs::write(&manifest_path, serde_json::to_string_pretty(&manifest)?)
                        .context("Failed to write manifest")?;

                    println!("  ✓ Installed to {}", game_dir.display());
                    success_count += 1;
                } else {
                    println!("  ✗ No WASM file found");
                    fail_count += 1;
                }
            }
            _ => {
                println!("  ✗ Build failed");
                fail_count += 1;
            }
        }
    }

    println!();
    println!(
        "Done! Built {} examples ({} failed)",
        success_count, fail_count
    );
    println!("Examples installed to: {}", games_dir.display());
    println!("You can now run 'cargo run' to play them in Emberware Z.");

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
