//! Run command - build + pack + launch in emulator

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

use crate::build::{self, BuildArgs};
use crate::pack::{self, PackArgs};

/// Arguments for the run command
#[derive(Args)]
pub struct RunArgs {
    /// Path to game project directory (defaults to current directory)
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Path to ember.toml manifest file
    #[arg(short, long, default_value = "ember.toml")]
    pub manifest: PathBuf,

    /// Build in debug mode (default is release)
    #[arg(long)]
    pub debug: bool,

    /// Don't rebuild, just repack and run
    #[arg(long)]
    pub no_build: bool,
}

/// Execute the run command
pub fn execute(args: RunArgs) -> Result<()> {
    let project_dir = args
        .project
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    // Step 1: Build (unless --no-build)
    if !args.no_build {
        println!("=== Building ===");
        build::execute(BuildArgs {
            project: Some(project_dir.clone()),
            debug: args.debug,
        })?;
        println!();
    }

    // Step 2: Pack
    println!("=== Packing ===");
    let manifest_path = project_dir.join(&args.manifest);
    pack::execute(PackArgs {
        manifest: manifest_path.clone(),
        output: None,
        wasm: None,
    })?;
    println!();

    // Step 3: Run
    println!("=== Running ===");

    // Read manifest to get game ID
    let manifest_content =
        std::fs::read_to_string(&manifest_path).context("Failed to read ember.toml")?;
    let manifest: toml::Value =
        toml::from_str(&manifest_content).context("Failed to parse ember.toml")?;

    let game_id = manifest
        .get("game")
        .and_then(|g| g.get("id"))
        .and_then(|id| id.as_str())
        .context("Missing game.id in ember.toml")?;

    let rom_path = project_dir.join(format!("{}.ewz", game_id));

    // Find emberware executable
    // Try several locations:
    // 1. In PATH
    // 2. In workspace target directory
    // 3. Relative to ember-cli
    let emberware_exe = find_emberware_exe()?;

    println!(
        "Launching: {} {}",
        emberware_exe.display(),
        rom_path.display()
    );

    let status = Command::new(&emberware_exe)
        .arg(&rom_path)
        .status()
        .context("Failed to run emberware")?;

    if !status.success() {
        anyhow::bail!("Emberware exited with error");
    }

    Ok(())
}

/// Find the emberware executable
fn find_emberware_exe() -> Result<PathBuf> {
    // Try PATH first
    if let Ok(path) = which::which("emberware") {
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
        "Could not find emberware executable. \
        Make sure it's in your PATH or run from the workspace directory."
    )
}
