//! Build command - compile Rust game to WASM

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

/// Arguments for the build command
#[derive(Args)]
pub struct BuildArgs {
    /// Path to game project directory (defaults to current directory)
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Build in debug mode (default is release)
    #[arg(long)]
    pub debug: bool,
}

/// Execute the build command
pub fn execute(args: BuildArgs) -> Result<()> {
    let project_dir = args
        .project
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    println!("Building WASM from {}...", project_dir.display());

    // Build arguments
    let mut cargo_args = vec!["build", "--target", "wasm32-unknown-unknown"];

    if !args.debug {
        cargo_args.push("--release");
    }

    // Run cargo build
    let status = Command::new("cargo")
        .args(&cargo_args)
        .current_dir(&project_dir)
        .status()
        .context("Failed to run cargo")?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }

    // Find the built WASM file
    let profile = if args.debug { "debug" } else { "release" };
    let target_dir = project_dir.join(format!(
        "target/wasm32-unknown-unknown/{}/",
        profile
    ));

    // Find .wasm file
    let wasm_file = std::fs::read_dir(&target_dir)
        .context("Failed to read target directory")?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path()
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| ext == "wasm")
                .unwrap_or(false)
        })
        .map(|e| e.path());

    match wasm_file {
        Some(path) => {
            let size = std::fs::metadata(&path)?.len();
            println!("Built: {} ({} bytes)", path.display(), size);
            Ok(())
        }
        None => {
            anyhow::bail!("No WASM file found in {}", target_dir.display());
        }
    }
}
