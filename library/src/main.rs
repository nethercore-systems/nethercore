//! Unified Emberware launcher
//!
//! This binary can launch games from any supported console type (Z, Classic, etc.)
//! by detecting the console type from the game manifest.

use anyhow::Result;
use emberware_core::library::{DataDirProvider, get_local_games, resolve_game_id};
use emberware_library::registry::{ConsoleRegistry, PlayerOptions};
use std::env;
use std::path::PathBuf;

/// Data directory provider for the unified launcher.
struct LauncherDataDirProvider;

impl DataDirProvider for LauncherDataDirProvider {
    fn data_dir(&self) -> Option<PathBuf> {
        directories::ProjectDirs::from("io.emberware", "", "Emberware")
            .map(|dirs| dirs.data_dir().to_path_buf())
    }
}

/// Parse deep link from command line args (emberware://play/game_id)
fn parse_deep_link(args: &[String]) -> Option<String> {
    for arg in args.iter().skip(1) {
        if let Some(rest) = arg.strip_prefix("emberware://play/") {
            let game_id = rest.trim_end_matches('/').to_string();
            if !game_id.is_empty() {
                return Some(game_id);
            }
        }
    }
    None
}

/// Check if a string looks like a file path to a ROM file
fn is_rom_path(arg: &str) -> Option<PathBuf> {
    let path = PathBuf::from(arg);

    // Check if it has a ROM extension
    let ext = path.extension().and_then(|e| e.to_str());
    let is_rom_ext = matches!(ext, Some("ewzx") | Some("wasm"));

    // If it has a ROM extension and the file exists, treat it as a path
    if is_rom_ext && path.exists() {
        return Some(path);
    }

    // If it contains path separators and exists, treat it as a path
    if (arg.contains('/') || arg.contains('\\')) && path.exists() {
        return Some(path);
    }

    None
}

/// Parse player options from command line args
fn parse_player_options(args: &[String]) -> PlayerOptions {
    let mut options = PlayerOptions::default();

    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-f" | "--fullscreen" => options.fullscreen = true,
            "-d" | "--debug" => options.debug = true,
            _ => {}
        }
    }

    options
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let registry = ConsoleRegistry::new();
    let provider = LauncherDataDirProvider;

    // Check for CLI game argument or deep link
    let args: Vec<String> = env::args().collect();

    // Parse player options from CLI flags
    let options = parse_player_options(&args);

    // Try deep link first (emberware://play/game_id)
    if let Some(game_id) = parse_deep_link(&args) {
        tracing::info!("Deep link detected: {}", game_id);

        let games = get_local_games(&provider);
        if let Some(game) = games.iter().find(|g| g.id == game_id) {
            // Run and wait (no library UI)
            registry.run_game_with_options(game, &options)?;
        } else {
            eprintln!("Game '{}' not found", game_id);
            std::process::exit(1);
        }
        return Ok(());
    }

    // Check for file path argument (for development)
    if args.len() > 1
        && let Some(path) = is_rom_path(&args[1])
    {
        tracing::info!("Running from file path: {}", path.display());
        // Run and wait (no library UI)
        registry.run_from_path_with_options(path, &options)?;
        return Ok(());
    }

    // Check for game ID argument - find first non-flag argument
    let game_query = args.iter().skip(1).find(|arg| !arg.starts_with('-'));

    if let Some(query) = game_query {
        let games = get_local_games(&provider);

        if games.is_empty() {
            eprintln!("No games installed. Download games from the library UI.");
            eprintln!("Tip: You can also pass a .ewzx file path directly.");
            std::process::exit(1);
        }

        // Resolve game ID
        match resolve_game_id(query, &games) {
            Ok(game_id) => {
                // Find the game to get its console type
                if let Some(game) = games.iter().find(|g| g.id == game_id) {
                    tracing::info!("Running '{}' (console: {})", game.title, game.console_type);

                    // Run and wait (no library UI)
                    registry.run_game_with_options(game, &options)?;
                } else {
                    eprintln!("Game '{}' not found", game_id);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("{}", e.message);
                if let Some(suggestions) = e.suggestion {
                    eprintln!("\nDid you mean:");
                    for suggestion in suggestions {
                        eprintln!("  - {}", suggestion);
                    }
                    eprintln!("\nAvailable games:");
                    for game in games.iter() {
                        eprintln!("  - {} [{}]", game.id, game.console_type);
                    }
                }
                std::process::exit(1);
            }
        }
    } else {
        // No argument - show unified library UI
        tracing::info!("Launching Emberware Library");

        registry.launch_library()?;
    }

    Ok(())
}
