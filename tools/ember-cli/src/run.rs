//! Run command - build + launch in emulator
//!
//! Orchestrates: compile → pack → launch

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

use crate::build::{self, BuildArgs};
use crate::manifest::EmberManifest;

/// Arguments for the run command
#[derive(Args)]
pub struct RunArgs {
    /// Path to game project directory (defaults to current directory)
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Path to ember.toml manifest file (relative to project directory)
    #[arg(short, long, default_value = "ember.toml")]
    pub manifest: PathBuf,

    /// Build in debug mode (default is release)
    #[arg(long)]
    pub debug: bool,

    /// Don't rebuild, just launch existing ROM
    #[arg(long)]
    pub no_build: bool,
}

/// Execute the run command
pub fn execute(args: RunArgs) -> Result<()> {
    let project_dir = args
        .project
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let manifest_path = project_dir.join(&args.manifest);

    // Step 1: Build (unless --no-build)
    if !args.no_build {
        build::execute(BuildArgs {
            project: Some(project_dir.clone()),
            manifest: args.manifest.clone(),
            output: None,
            debug: args.debug,
            no_compile: false,
        })?;
        println!();
    }

    // Step 2: Launch
    println!("=== Launching ===");

    // Read manifest to get game ID
    let manifest = EmberManifest::load(&manifest_path)?;
    let rom_path = project_dir.join(format!(
        "{}.{}",
        manifest.game.id,
        emberware_shared::ZX_ROM_FORMAT.extension
    ));

    // Use absolute path for subprocess (working directory may differ)
    let rom_path = rom_path.canonicalize().unwrap_or_else(|_| rom_path.clone());

    if !rom_path.exists() {
        anyhow::bail!(
            "ROM file not found: {}\nRun 'ember build' first.",
            rom_path.display()
        );
    }

    // Find emberware executable
    let emberware_exe = find_emberware_exe()?;

    println!(
        "  Launching: {} {}",
        emberware_exe.display(),
        rom_path.display()
    );

    // Handle special "cargo:run" marker
    let status = if emberware_exe.to_string_lossy() == "cargo:run" {
        Command::new("cargo")
            .args(["run", "-p", "emberware-zx", "--"])
            .arg(&rom_path)
            .status()
            .context("Failed to run 'cargo run'")?
    } else {
        Command::new(&emberware_exe)
            .arg(&rom_path)
            .status()
            .context("Failed to run emberware")?
    };

    if !status.success() {
        anyhow::bail!("Emberware exited with error");
    }

    Ok(())
}

/// Find the emberware-zx player executable
fn find_emberware_exe() -> Result<PathBuf> {
    // Try PATH first - look for the standalone player
    if let Ok(path) = which::which("emberware-zx") {
        return Ok(path);
    }

    // Try cargo run in workspace
    // This is useful during development
    let cargo_exe = PathBuf::from("cargo");

    // Check if we can run cargo
    if Command::new(&cargo_exe).arg("--version").output().is_ok() {
        // Return a special marker that means "use cargo run"
        return Ok(PathBuf::from("cargo:run"));
    }

    anyhow::bail!(
        "Could not find emberware-zx executable. \
        Make sure it's in your PATH or run from the workspace directory."
    )
}
