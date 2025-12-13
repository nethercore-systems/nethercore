//! Unified Emberware launcher
//!
//! This binary can launch games from any supported console type (Z, Classic, etc.)
//! by detecting the console type from the game manifest.

mod registry;

use anyhow::Result;
use emberware_core::library::{DataDirProvider, get_local_games, resolve_game_id};
use registry::ConsoleRegistry;
use std::env;
use std::path::PathBuf;

/// Data directory provider for the unified launcher.
struct LauncherDataDirProvider;

impl DataDirProvider for LauncherDataDirProvider {
    fn data_dir(&self) -> Option<PathBuf> {
        directories::ProjectDirs::from("io", "emberware", "emberware")
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

    // Try deep link first (emberware://play/game_id)
    let query = if let Some(game_id) = parse_deep_link(&args) {
        tracing::info!("Deep link detected: {}", game_id);
        Some(game_id)
    } else if args.len() > 1 {
        // Regular CLI argument
        Some(args[1].clone())
    } else {
        None
    };

    if let Some(query) = query {
        // Get all available games
        let games = get_local_games(&provider);

        if games.is_empty() {
            eprintln!("No games installed. Download games from the library UI.");
            std::process::exit(1);
        }

        // Resolve game ID
        match resolve_game_id(&query, &games) {
            Ok(game_id) => {
                // Find the game to get its console type
                if let Some(game) = games.iter().find(|g| g.id == game_id) {
                    tracing::info!(
                        "Launching '{}' (console: {})",
                        game.title,
                        game.console_type
                    );

                    // Launch via registry
                    registry.launch_game(game)?;
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
        // For now, default to Z's library UI
        // TODO: Create unified library UI that shows all console types
        tracing::info!("Launching unified library (defaulting to Z for now)");

        registry.launch_library(None)?;
    }

    Ok(())
}
