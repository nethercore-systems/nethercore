//! Run command - build + launch in emulator
//!
//! Orchestrates: compile → pack → launch
//!
//! With `--watch` flag, enters dev mode:
//! - Watches source and asset files for changes
//! - Rebuilds ROM on change
//! - Relaunches player automatically

use anyhow::{Context, Result};
use clap::Args;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

use crate::build::{self, BuildArgs};
use crate::manifest::NetherManifest;
use crate::watch::{self, WatchEvent};

/// Create a successful exit status (platform-specific workaround)
#[cfg(unix)]
fn status_success() -> std::process::ExitStatus {
    use std::os::unix::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(0)
}

#[cfg(windows)]
fn status_success() -> std::process::ExitStatus {
    use std::os::windows::process::ExitStatusExt;
    std::process::ExitStatus::from_raw(0)
}

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

    /// Watch for file changes and automatically rebuild/relaunch
    #[arg(long)]
    pub watch: bool,
}

/// Execute the run command
pub fn execute(args: RunArgs) -> Result<()> {
    let project_dir = args
        .project
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let manifest_path = project_dir.join(&args.manifest);

    // Handle watch mode separately
    if args.watch {
        return execute_watch_mode(&args, &project_dir, &manifest_path);
    }

    // Normal (non-watch) execution
    execute_single_run(&args, &project_dir, &manifest_path)
}

/// Execute a single build + launch cycle
fn execute_single_run(
    args: &RunArgs,
    project_dir: &Path,
    manifest_path: &Path,
) -> Result<()> {
    // Step 1: Build (unless --no-build)
    if !args.no_build {
        build::execute(BuildArgs {
            project: Some(project_dir.to_path_buf()),
            manifest: args.manifest.clone(),
            output: None,
            debug: args.debug,
            no_compile: false,
        })?;
        println!();
    }

    // Step 2: Launch and wait
    let status = launch_player(args, project_dir, manifest_path)?;

    if !status.success() {
        anyhow::bail!("Nethercore exited with error");
    }

    Ok(())
}

/// Launch the player and wait for it to exit
fn launch_player(
    args: &RunArgs,
    project_dir: &Path,
    manifest_path: &Path,
) -> Result<std::process::ExitStatus> {
    println!("=== Launching ===");

    // Read manifest to get game ID
    let manifest = NetherManifest::load(manifest_path)?;
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
    let (nethercore_exe, workspace_dir) = find_nethercore_exe()?;

    // Handle P2P test mode (launches two instances)
    if args.p2p_test {
        // P2P test handles its own process management
        return launch_p2p_test(&nethercore_exe, &workspace_dir, &rom_path, args)
            .map(|()| status_success());
    }

    // Build common arguments
    let extra_args = build_player_args(args);

    println!(
        "  Launching: {} {} {}",
        nethercore_exe.display(),
        rom_path.display(),
        extra_args.join(" ")
    );

    // Handle special "cargo:run" marker
    let status = if nethercore_exe.to_string_lossy() == "cargo:run" {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-p", "nethercore-zx", "--"])
            .arg(&rom_path)
            .args(&extra_args);
        if let Some(ref ws) = workspace_dir {
            cmd.current_dir(ws);
        }
        cmd.status().context("Failed to run 'cargo run'")?
    } else {
        Command::new(&nethercore_exe)
            .arg(&rom_path)
            .args(&extra_args)
            .status()
            .context("Failed to run nethercore")?
    };

    Ok(status)
}

/// Spawn the player as a background process (doesn't wait for exit)
fn spawn_player(
    args: &RunArgs,
    project_dir: &Path,
    manifest_path: &Path,
) -> Result<Child> {
    // Read manifest to get game ID
    let manifest = NetherManifest::load(manifest_path)?;
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
    let (nethercore_exe, workspace_dir) = find_nethercore_exe()?;

    // Build common arguments
    let extra_args = build_player_args(args);

    println!(
        "  Launching: {} {} {}",
        nethercore_exe.display(),
        rom_path.display(),
        extra_args.join(" ")
    );

    // Handle special "cargo:run" marker
    let child = if nethercore_exe.to_string_lossy() == "cargo:run" {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-p", "nethercore-zx", "--"])
            .arg(&rom_path)
            .args(&extra_args);
        if let Some(ref ws) = workspace_dir {
            cmd.current_dir(ws);
        }
        cmd.spawn().context("Failed to spawn 'cargo run'")?
    } else {
        Command::new(&nethercore_exe)
            .arg(&rom_path)
            .args(&extra_args)
            .spawn()
            .context("Failed to spawn nethercore")?
    };

    Ok(child)
}

/// Execute watch mode: rebuild and relaunch on file changes
fn execute_watch_mode(
    args: &RunArgs,
    project_dir: &Path,
    manifest_path: &Path,
) -> Result<()> {
    println!("=== Dev Mode (--watch) ===");
    println!("  Watching for changes. Press Ctrl+C to exit.");
    println!();

    // Initial build
    if !args.no_build {
        if let Err(e) = build::execute(BuildArgs {
            project: Some(project_dir.to_path_buf()),
            manifest: args.manifest.clone(),
            output: None,
            debug: args.debug,
            no_compile: false,
        }) {
            eprintln!("Build failed: {}", e);
            eprintln!("Watching for changes to retry...");
        } else {
            println!();
        }
    }

    // Setup file watcher
    let manifest = NetherManifest::load(manifest_path)?;
    let watch_paths = watch::collect_watch_paths(project_dir, &manifest);
    println!(
        "  Watching {} paths ({} source dirs, {} asset files)",
        watch_paths.count(),
        watch_paths.source_dirs.len(),
        watch_paths.asset_files.len()
    );

    let watcher = watch::FileWatcher::new(&watch_paths)?;

    // Launch player (spawn, don't wait)
    let mut player: Option<Child> = match spawn_player(args, project_dir, manifest_path) {
        Ok(child) => Some(child),
        Err(e) => {
            eprintln!("Failed to launch player: {}", e);
            None
        }
    };

    // Watch loop
    loop {
        // Check if player has exited
        if let Some(ref mut child) = player {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Player exited
                    if status.success() {
                        println!();
                        println!("  Player closed normally. Exiting watch mode.");
                        return Ok(());
                    } else {
                        println!();
                        println!("  Player exited with error. Watching for changes...");
                        player = None;
                    }
                }
                Ok(None) => {
                    // Still running
                }
                Err(e) => {
                    eprintln!("Error checking player status: {}", e);
                    player = None;
                }
            }
        }

        // Wait for file changes (with timeout to check player status periodically)
        match watcher.try_recv() {
            Some(WatchEvent::FilesChanged(files)) => {
                println!();
                println!("=== Files changed ===");
                for file in files.iter().take(5) {
                    println!("  {}", file.display());
                }
                if files.len() > 5 {
                    println!("  ... and {} more", files.len() - 5);
                }
                println!();

                // Kill existing player
                if let Some(ref mut child) = player {
                    println!("  Stopping player...");
                    let _ = child.kill();
                    let _ = child.wait();
                }
                player = None;

                // Rebuild
                println!("=== Rebuilding ===");
                if let Err(e) = build::execute(BuildArgs {
                    project: Some(project_dir.to_path_buf()),
                    manifest: args.manifest.clone(),
                    output: None,
                    debug: args.debug,
                    no_compile: false,
                }) {
                    eprintln!("Build failed: {}", e);
                    eprintln!("Watching for changes to retry...");
                    continue;
                }
                println!();

                // Relaunch
                println!("=== Relaunching ===");
                match spawn_player(args, project_dir, manifest_path) {
                    Ok(child) => player = Some(child),
                    Err(e) => {
                        eprintln!("Failed to launch player: {}", e);
                    }
                }
            }
            Some(WatchEvent::ManifestChanged) => {
                println!();
                println!("=== Manifest changed ===");
                println!("  Reloading watch paths and rebuilding...");
                println!();

                // Kill existing player
                if let Some(ref mut child) = player {
                    println!("  Stopping player...");
                    let _ = child.kill();
                    let _ = child.wait();
                }
                player = None;

                // Reload manifest and recreate watcher
                // For simplicity in Phase 1, we just rebuild and relaunch
                // A full implementation would recreate the watcher with new paths

                // Rebuild
                println!("=== Rebuilding ===");
                if let Err(e) = build::execute(BuildArgs {
                    project: Some(project_dir.to_path_buf()),
                    manifest: args.manifest.clone(),
                    output: None,
                    debug: args.debug,
                    no_compile: false,
                }) {
                    eprintln!("Build failed: {}", e);
                    eprintln!("Watching for changes to retry...");
                    continue;
                }
                println!();

                // Relaunch
                println!("=== Relaunching ===");
                match spawn_player(args, project_dir, manifest_path) {
                    Ok(child) => player = Some(child),
                    Err(e) => {
                        eprintln!("Failed to launch player: {}", e);
                    }
                }
            }
            Some(WatchEvent::Error(msg)) => {
                eprintln!("Watch error: {}", msg);
            }
            None => {
                // No events, sleep briefly before checking again
                std::thread::sleep(std::time::Duration::from_millis(100));
            }
        }
    }
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
fn launch_p2p_test(
    nethercore_exe: &PathBuf,
    workspace_dir: &Option<PathBuf>,
    rom_path: &PathBuf,
    args: &RunArgs,
) -> Result<()> {
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
        if let Some(ref ws) = workspace_dir {
            cmd.current_dir(ws);
        }
        cmd
    } else {
        let mut cmd = Command::new(nethercore_exe);
        cmd.arg(rom_path);
        cmd
    };

    p2_cmd
        .args([
            "--p2p",
            "--bind",
            "7778",
            "--peer",
            "7777",
            "--local-player",
            "1",
        ])
        .args(["--input-delay", &input_delay]);

    let mut p2_child = p2_cmd.spawn().context("Failed to spawn player 2")?;

    // Give player 2 time to bind
    std::thread::sleep(std::time::Duration::from_millis(500));

    // Start player 1 in foreground
    let mut p1_cmd = if is_cargo_run {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-p", "nethercore-zx", "--"]);
        cmd.arg(rom_path);
        if let Some(ref ws) = workspace_dir {
            cmd.current_dir(ws);
        }
        cmd
    } else {
        let mut cmd = Command::new(nethercore_exe);
        cmd.arg(rom_path);
        cmd
    };

    p1_cmd
        .args([
            "--p2p",
            "--bind",
            "7777",
            "--peer",
            "7778",
            "--local-player",
            "0",
        ])
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
/// Returns (exe_path, optional_workspace_dir for cargo:run fallback)
fn find_nethercore_exe() -> Result<(PathBuf, Option<PathBuf>)> {
    let exe_name = if cfg!(windows) {
        "nethercore-zx.exe"
    } else {
        "nethercore-zx"
    };

    // 1. Try PATH first (installed globally)
    if let Ok(path) = which::which("nethercore-zx") {
        return Ok((path, None));
    }

    // 2. Try sibling binary (distributed bundle)
    // Look for nethercore-zx next to the nether CLI binary
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let sibling = exe_dir.join(exe_name);
            if sibling.exists() {
                return Ok((sibling, None));
            }
        }
    }

    // 3. Fall back to cargo run (developers only)
    let cargo_exe = PathBuf::from("cargo");
    if Command::new(&cargo_exe).arg("--version").output().is_ok() {
        // CARGO_MANIFEST_DIR points to tools/nether-cli at compile time
        let cli_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace = cli_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        return Ok((PathBuf::from("cargo:run"), workspace));
    }

    anyhow::bail!(
        "Could not find nethercore-zx player.\n\
        Options:\n\
        - Install it to PATH: cargo install --path nethercore-zx\n\
        - Place nethercore-zx binary next to nether CLI\n\
        - Run from nethercore workspace (developer mode)"
    )
}
