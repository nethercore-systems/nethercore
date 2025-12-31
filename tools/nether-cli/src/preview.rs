//! Preview command - browse ROM assets without running the game
//!
//! Supports smart ROM lookup:
//! - Direct path: `nether preview game.nczx`
//! - Game ID lookup: `nether preview paddle` (finds paddle.nczx)
//! - Prefix matching: `nether preview pad` (matches paddle.nczx if unique)

use anyhow::{Context, Result};
use clap::Args;
use nethercore_core::library::resolve_id;
use std::path::PathBuf;
use std::process::Command;
use walkdir::WalkDir;

/// Arguments for the preview command
#[derive(Args)]
pub struct PreviewArgs {
    /// ROM file or game ID to preview (.nczx)
    ///
    /// Can be:
    /// - A direct path: game.nczx, ./build/game.nczx
    /// - A game ID: paddle (finds paddle.nczx in current directory)
    /// - A prefix: pad (matches paddle.nczx if unique)
    pub rom: String,

    /// Initial asset to focus (e.g., "player_texture")
    #[arg(long)]
    pub asset: Option<String>,
}

/// Execute the preview command
pub fn execute(args: PreviewArgs) -> Result<()> {
    // Resolve ROM path (supports direct path or game ID lookup)
    let rom_path = resolve_rom_path(&args.rom)?;

    println!("=== Preview Mode ===");
    println!("  ROM: {}", rom_path.display());

    // Find nethercore-zx executable (reuse logic from run.rs)
    let (nethercore_exe, workspace_dir) = find_nethercore_exe()?;

    // Build arguments
    let mut player_args = vec!["--preview".to_string()];
    if let Some(asset) = &args.asset {
        player_args.push("--asset".to_string());
        player_args.push(asset.clone());
    }

    println!("  Launching preview...");

    // Handle special "cargo:run" marker
    let status = if nethercore_exe.to_string_lossy() == "cargo:run" {
        let mut cmd = Command::new("cargo");
        cmd.args(["run", "-p", "nethercore-zx", "--"])
            .arg(&rom_path)
            .args(&player_args);
        if let Some(ref ws) = workspace_dir {
            cmd.current_dir(ws);
        }
        cmd.status().context("Failed to run 'cargo run'")?
    } else {
        Command::new(&nethercore_exe)
            .arg(&rom_path)
            .args(&player_args)
            .status()
            .context("Failed to run nethercore-zx")?
    };

    if !status.success() {
        anyhow::bail!("Preview mode exited with error");
    }

    Ok(())
}

/// Find the nethercore-zx player executable (copied from run.rs)
fn find_nethercore_exe() -> Result<(PathBuf, Option<PathBuf>)> {
    let exe_name = if cfg!(windows) {
        "nethercore-zx.exe"
    } else {
        "nethercore-zx"
    };

    // 1. Try PATH first
    if let Ok(path) = which::which("nethercore-zx") {
        return Ok((path, None));
    }

    // 2. Try sibling binary
    if let Ok(current_exe) = std::env::current_exe() {
        if let Some(exe_dir) = current_exe.parent() {
            let sibling = exe_dir.join(exe_name);
            if sibling.exists() {
                return Ok((sibling, None));
            }
        }
    }

    // 3. Fall back to cargo run
    let cargo_exe = PathBuf::from("cargo");
    if Command::new(&cargo_exe).arg("--version").output().is_ok() {
        let cli_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let workspace = cli_dir
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_path_buf());
        return Ok((PathBuf::from("cargo:run"), workspace));
    }

    anyhow::bail!(
        "Could not find nethercore-zx player.\n\
        Install it to PATH or run from workspace."
    )
}

/// Resolve a ROM path from user input.
///
/// Supports:
/// 1. Direct path (file exists)
/// 2. Game ID lookup (finds {id}.nczx in current dir or subdirs)
/// 3. Prefix matching (unique prefix match)
fn resolve_rom_path(input: &str) -> Result<PathBuf> {
    let path = PathBuf::from(input);

    // 1. Direct path - if it exists, use it
    if path.exists() {
        return path.canonicalize().context("Failed to resolve path");
    }

    // 2. If input has path separators or .nczx extension, it's meant to be a path
    if input.contains('/') || input.contains('\\') || input.ends_with(".nczx") {
        anyhow::bail!("ROM file not found: {}", input);
    }

    // 3. Search for matching .nczx files using the shared resolver
    let available_roms = find_local_roms()?;

    if available_roms.is_empty() {
        anyhow::bail!(
            "No .nczx ROM files found in current directory.\n\
            Build a game first with 'nether build' or provide a direct path."
        );
    }

    // Use the shared resolver with a closure to extract ROM name
    match resolve_id(
        input,
        &available_roms,
        |p| p.file_stem().and_then(|s| s.to_str()).unwrap_or(""),
        "ROM",
    ) {
        Ok(rom) => Ok(rom.clone()),
        Err(err) => {
            let mut msg = err.message;
            if let Some(suggestions) = err.suggestion {
                msg.push_str("\n\nDid you mean:\n  ");
                msg.push_str(&suggestions.join("\n  "));
            }
            msg.push_str(
                "\n\nTip: Use prefix matching, e.g., 'nether preview pad' for 'paddle.nczx'",
            );
            anyhow::bail!(msg);
        }
    }
}

/// Find all .nczx ROM files in the current directory and immediate subdirectories.
fn find_local_roms() -> Result<Vec<PathBuf>> {
    let current_dir = std::env::current_dir()?;
    let mut roms = Vec::new();

    // Search current directory and up to 2 levels deep
    for entry in WalkDir::new(&current_dir)
        .max_depth(3)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some("nczx") {
            // Skip target directories (build artifacts from other projects)
            let path_str = path.to_string_lossy();
            if !path_str.contains("target")
                || path_str.contains(&current_dir.to_string_lossy().to_string())
            {
                roms.push(path.to_path_buf());
            }
        }
    }

    // Sort by path length (prefer closer files) then alphabetically
    roms.sort_by(|a, b| {
        let depth_a = a.components().count();
        let depth_b = b.components().count();
        depth_a.cmp(&depth_b).then_with(|| a.cmp(b))
    });

    Ok(roms)
}
