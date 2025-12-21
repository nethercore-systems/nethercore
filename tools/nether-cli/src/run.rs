//! Run command - build + launch in emulator
//!
//! Orchestrates: compile → pack → launch

use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use std::process::Command;

use crate::build::{self, BuildArgs};
use crate::manifest::NetherManifest;

/// Arguments for the run command
#[derive(Args)]
pub struct RunArgs {
    /// Path to game project directory (defaults to current directory)
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Path to nether.toml manifest file (relative to project directory)
    #[arg(short, long, default_value = "nether.toml")]
    pub manifest: PathBuf,

    /// Build in debug mode (default is release)
    #[arg(long)]
    pub debug: bool,

    /// Don't rebuild, just launch existing ROM
    #[arg(long)]
    pub no_build: bool,

    // === Multiplayer Options ===
    /// Run in sync-test mode to verify game determinism
    #[arg(long)]
    pub sync_test: bool,

    /// Number of players (1-4)
    #[arg(long, default_value = "1")]
    pub players: usize,

    /// Input delay in frames (0-10, higher = smoother online play)
    #[arg(long, default_value = "0")]
    pub input_delay: usize,

    /// Launch local P2P test (spawns two connected instances)
    #[arg(long)]
    pub p2p_test: bool,
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
    let manifest = NetherManifest::load(&manifest_path)?;
    let rom_path = project_dir.join(format!(
        "{}.{}",
        manifest.game.id,
        nethercore_shared::ZX_ROM_FORMAT.extension
    ));

    // Use absolute path for subprocess (working directory may differ)
    let rom_path = rom_path.canonicalize().unwrap_or_else(|_| rom_path.clone());

    if !rom_path.exists() {
        anyhow::bail!(
            "ROM file not found: {}\nRun 'nether build' first.",
            rom_path.display()
        );
    }

    // Find nethercore executable
    let nethercore_exe = find_nethercore_exe()?;

    // Handle P2P test mode (launches two instances)
    if args.p2p_test {
        return launch_p2p_test(&nethercore_exe, &rom_path, &args);
    }

    // Build common arguments
    let extra_args = build_player_args(&args);

    println!(
        "  Launching: {} {} {}",
        nethercore_exe.display(),
        rom_path.display(),
        extra_args.join(" ")
    );

    // Handle special "cargo:run" marker
    let status = if nethercore_exe.to_string_lossy() == "cargo:run" {
        Command::new("cargo")
            .args(["run", "-p", "nethercore-zx", "--"])
            .arg(&rom_path)
            .args(&extra_args)
            .status()
            .context("Failed to run 'cargo run'")?
    } else {
        Command::new(&nethercore_exe)
            .arg(&rom_path)
            .args(&extra_args)
            .status()
            .context("Failed to run nethercore")?
    };

    if !status.success() {
        anyhow::bail!("Nethercore exited with error");
    }

    Ok(())
}

/// Build extra arguments to pass to the player based on RunArgs
fn build_player_args(args: &RunArgs) -> Vec<String> {
    let mut extra_args = Vec::new();

    if args.sync_test {
        extra_args.push("--sync-test".to_string());
    }

    if args.players > 1 {
        extra_args.push("--players".to_string());
        extra_args.push(args.players.to_string());
    }

    if args.input_delay > 0 {
        extra_args.push("--input-delay".to_string());
        extra_args.push(args.input_delay.to_string());
    }

    extra_args
}

/// Launch a local P2P test with two connected instances
fn launch_p2p_test(nethercore_exe: &PathBuf, rom_path: &PathBuf, args: &RunArgs) -> Result<()> {
    println!("  Launching P2P test...");
    println!("    Player 1: bind=7777, peer=7778, local_player=0");
    println!("    Player 2: bind=7778, peer=7777, local_player=1");
    println!();

    let input_delay = args.input_delay.to_string();

    // Handle special "cargo:run" marker
    let is_cargo_run = nethercore_exe.to_string_lossy() == "cargo:run";

    // Start player 2 in background
    let mut p2_cmd = if is_cargo_run {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-p", "nethercore-zx", "--"]);
        cmd.arg(rom_path);
        cmd
    } else {
        let mut cmd = Command::new(nethercore_exe);
        cmd.arg(rom_path);
        cmd
    };

    p2_cmd
        .args(["--p2p", "--bind", "7778", "--peer", "7777", "--local-player", "1"])
        .args(["--input-delay", &input_delay]);

    let mut p2_child = p2_cmd.spawn().context("Failed to spawn player 2")?;

    // Give player 2 time to bind
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Start player 1 in foreground
    let mut p1_cmd = if is_cargo_run {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-p", "nethercore-zx", "--"]);
        cmd.arg(rom_path);
        cmd
    } else {
        let mut cmd = Command::new(nethercore_exe);
        cmd.arg(rom_path);
        cmd
    };

    p1_cmd
        .args(["--p2p", "--bind", "7777", "--peer", "7778", "--local-player", "0"])
        .args(["--input-delay", &input_delay]);

    let status = p1_cmd.status().context("Failed to run player 1")?;

    // Clean up player 2
    println!();
    println!("  Player 1 exited, cleaning up...");
    let _ = p2_child.kill();
    let _ = p2_child.wait();

    if !status.success() {
        anyhow::bail!("Nethercore exited with error");
    }

    println!("  P2P test complete.");
    Ok(())
}

/// Find the nethercore-zx player executable
fn find_nethercore_exe() -> Result<PathBuf> {
    // Try PATH first - look for the standalone player
    if let Ok(path) = which::which("nethercore-zx") {
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
        "Could not find nethercore-zx executable. \
        Make sure it's in your PATH or run from the workspace directory."
    )
}
