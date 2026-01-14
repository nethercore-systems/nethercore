//! Player process launching and command building.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};

use nethercore_core::library::LocalGame;
use nethercore_shared::ConsoleType;

use super::helpers::{console_type_from_extension, console_type_from_str, player_binary_name, supported_console_types, supported_extension_list};
use super::launcher::{ConnectionMode, PlayerOptions};

/// Find the player binary for a console type.
///
/// Searches in order:
/// 1. Same directory as the library executable
/// 2. System PATH
///
/// Returns the full path to the player binary, or just the binary name
/// if it should be found in PATH.
pub fn find_player_binary(console_type: ConsoleType) -> PathBuf {
    let binary_name = player_binary_name(console_type);
    let exe_name = if cfg!(windows) {
        format!("{}.exe", binary_name)
    } else {
        binary_name.to_string()
    };

    // Try same directory as library executable
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        let player_path = dir.join(&exe_name);
        if player_path.exists() {
            return player_path;
        }
    }

    // Fall back to PATH
    PathBuf::from(exe_name)
}

/// Build player command with options
pub(crate) fn build_player_command(
    rom_path: &Path,
    console_type: ConsoleType,
    options: &PlayerOptions,
) -> Command {
    let player = find_player_binary(console_type);

    let mut cmd = Command::new(&player);
    cmd.arg(rom_path);

    if options.fullscreen {
        cmd.arg("--fullscreen");
    }
    if options.debug {
        cmd.arg("--debug");
    }
    if let Some(players) = options.players {
        cmd.arg("--players");
        cmd.arg(players.to_string());
    }

    // Add multiplayer connection args
    if let Some(ref connection) = options.connection {
        match connection {
            ConnectionMode::Host { port } => {
                cmd.arg("--host");
                cmd.arg(port.to_string());
            }
            ConnectionMode::Join { host_ip, port } => {
                cmd.arg("--join");
                cmd.arg(format!("{}:{}", host_ip, port));
            }
            ConnectionMode::Session { file } => {
                cmd.arg("--session");
                cmd.arg(file);
            }
        }
    }

    // Add preview mode args
    if options.preview {
        cmd.arg("--preview");
        if let Some(ref asset) = options.preview_asset {
            cmd.arg("--asset");
            cmd.arg(asset);
        }
    }

    cmd
}

/// Launch a game using the appropriate player process.
///
/// This spawns a new process for the player and returns immediately.
/// The library continues running while the game plays.
/// Use `run_player` if you want to wait for the player to finish.
pub fn launch_player(rom_path: &Path, console_type: ConsoleType) -> Result<()> {
    let player = find_player_binary(console_type);

    tracing::info!(
        "Launching player: {} {}",
        player.display(),
        rom_path.display()
    );

    Command::new(&player)
        .arg(rom_path)
        .spawn()
        .with_context(|| {
            format!(
                "Failed to launch player '{}'. Make sure it exists in the same directory as the library or in your PATH.",
                player.display()
            )
        })?;

    Ok(())
}

/// Run a game using the appropriate player process and wait for it to finish.
///
/// This is used when launching from CLI - the launcher process waits for the
/// player to exit before returning. No library UI is shown.
pub fn run_player(rom_path: &Path, console_type: ConsoleType) -> Result<()> {
    run_player_with_options(rom_path, console_type, &PlayerOptions::default())
}

/// Launch a game with player options (spawns and returns immediately).
///
/// This is used when launching from the library UI with multiplayer options.
pub fn launch_player_with_options(
    rom_path: &Path,
    console_type: ConsoleType,
    options: &PlayerOptions,
) -> Result<()> {
    let mut cmd = build_player_command(rom_path, console_type, options);

    tracing::info!("Launching player with options: {:?}", cmd);

    cmd.spawn().with_context(|| {
        "Failed to launch player. Make sure it exists in the same directory as the library or in your PATH.".to_string()
    })?;

    Ok(())
}

/// Run a game with player options and wait for it to finish.
///
/// This is used when launching from CLI with flags like --fullscreen.
pub fn run_player_with_options(
    rom_path: &Path,
    console_type: ConsoleType,
    options: &PlayerOptions,
) -> Result<()> {
    let player = find_player_binary(console_type);

    tracing::info!(
        "Running player: {} {}{}{}",
        player.display(),
        rom_path.display(),
        if options.fullscreen {
            " --fullscreen"
        } else {
            ""
        },
        if options.debug { " --debug" } else { "" },
    );

    let mut cmd = build_player_command(rom_path, console_type, options);

    let status = cmd.status().with_context(|| {
        format!(
            "Failed to run player '{}'. Make sure it exists in the same directory as the library or in your PATH.",
            player.display()
        )
    })?;

    if !status.success()
        && let Some(code) = status.code()
        && code != 0
    {
        // Exit code 0 is success, anything else is an error
        // But some exit codes are normal (e.g., user pressed ESC)
        tracing::debug!("Player exited with code: {}", code);
    }

    Ok(())
}

/// Launch a game by ID (spawns and returns immediately).
///
/// Looks up the game in the local games list and launches the appropriate player.
/// Used by the library UI when the user clicks Play.
pub fn launch_game_by_id(game: &LocalGame) -> Result<()> {
    launch_game_by_id_with_options(game, &PlayerOptions::default())
}

/// Launch a game by ID with options (spawns and returns immediately).
///
/// Used by the library UI for multiplayer games.
pub fn launch_game_by_id_with_options(game: &LocalGame, options: &PlayerOptions) -> Result<()> {
    let console_type = console_type_from_str(&game.console_type).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown console type: '{}'. Supported consoles: {}",
            game.console_type,
            supported_console_types()
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    launch_player_with_options(&game.rom_path, console_type, options)
}

/// Run a game by ID and wait for it to finish.
///
/// Used when launching from CLI with a game ID argument.
pub fn run_game_by_id(game: &LocalGame) -> Result<()> {
    run_game_by_id_with_options(game, &PlayerOptions::default())
}

/// Run a game by ID with options and wait for it to finish.
pub fn run_game_by_id_with_options(game: &LocalGame, options: &PlayerOptions) -> Result<()> {
    let console_type = console_type_from_str(&game.console_type).ok_or_else(|| {
        anyhow::anyhow!(
            "Unknown console type: '{}'. Supported consoles: {}",
            game.console_type,
            supported_console_types()
                .iter()
                .map(|c| c.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        )
    })?;

    run_player_with_options(&game.rom_path, console_type, options)
}

/// Launch a game from a file path (spawns and returns immediately).
///
/// Detects the console type from the file extension.
/// Used by the library UI.
pub fn launch_game_from_path(path: &Path) -> Result<()> {
    let console_type = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(console_type_from_extension)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown ROM file type: {}. Supported extensions: {}",
                path.display(),
                supported_extension_list()
            )
        })?;

    launch_player(path, console_type)
}

/// Run a game from a file path and wait for it to finish.
///
/// Detects the console type from the file extension.
/// Used when launching from CLI with a file path argument.
pub fn run_game_from_path(path: &Path) -> Result<()> {
    run_game_from_path_with_options(path, &PlayerOptions::default())
}

/// Run a game from a file path with options and wait for it to finish.
pub fn run_game_from_path_with_options(path: &Path, options: &PlayerOptions) -> Result<()> {
    let console_type = path
        .extension()
        .and_then(|e| e.to_str())
        .and_then(console_type_from_extension)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Unknown ROM file type: {}. Supported extensions: {}",
                path.display(),
                supported_extension_list()
            )
        })?;

    run_player_with_options(path, console_type, options)
}
