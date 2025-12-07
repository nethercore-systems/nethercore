//! Emberware Z - Fantasy console runtime

use std::env;
use emberware_core::app::AppMode;

mod app;
mod audio;
mod config;
mod console;
mod deep_link;
mod download;
mod ffi;
mod font;
mod game_resolver;
mod graphics;
mod input;
mod library;
mod resource_manager;
mod settings_ui;
mod shader_gen;
mod state;
mod ui;

fn main() {
    tracing_subscriber::fmt::init();

    let args: Vec<String> = env::args().collect();

    let mode = if let Some(uri) = deep_link::parse(&args) {
        tracing::info!("Launched via deep link: {:?}", uri);
        AppMode::Playing {
            game_id: uri.game_id,
        }
    } else if let Some(game_id) = resolve_cli_arg(&args) {
        tracing::info!("Launched via CLI argument: {}", game_id);
        AppMode::Playing { game_id }
    } else {
        tracing::info!("Launched directly, showing library");
        AppMode::Library
    };

    if let Err(e) = app::run(mode) {
        tracing::error!("Application error: {}", e);
        std::process::exit(1);
    }
}

/// Try to resolve a game from CLI arguments
///
/// Returns None if no valid game argument found (user should see library)
/// Prints error and exits if invalid game specified (user has explicit intent)
fn resolve_cli_arg(args: &[String]) -> Option<String> {
    // Extract first non-flag positional argument
    let query = extract_query_arg(args)?;

    // Load available games
    let games = library::get_local_games();

    // Resolve game ID from query
    match game_resolver::resolve_game_id(&query, &games) {
        Ok(game_id) => Some(game_id),
        Err(err) => {
            // User explicitly requested a game that doesn't exist
            eprintln!("Error: {}", err.message);

            if let Some(suggestions) = &err.suggestion {
                if !suggestions.is_empty() {
                    eprintln!("\nDid you mean:");
                    for suggestion in suggestions {
                        eprintln!("  - {}", suggestion);
                    }
                }
            }

            if games.is_empty() {
                eprintln!("\nNo games found. Check ~/.emberware/games/");
            } else {
                eprintln!("\nAvailable games:");
                for game in &games {
                    eprintln!("  - {} ({})", game.id, game.title);
                }
            }

            std::process::exit(1);
        }
    }
}

/// Extract first positional argument (non-flag)
fn extract_query_arg(args: &[String]) -> Option<String> {
    // args[0] = executable path
    // Skip executable name and any flags
    for arg in args.iter().skip(1) {
        // Skip flags: --foo, -f
        if arg.starts_with("--") || (arg.starts_with('-') && arg.len() > 1) {
            continue;
        }
        return Some(arg.clone());
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_query_single_arg() {
        let args = vec!["app".to_string(), "platformer".to_string()];
        assert_eq!(extract_query_arg(&args), Some("platformer".to_string()));
    }

    #[test]
    fn test_extract_query_no_args() {
        let args = vec!["app".to_string()];
        assert!(extract_query_arg(&args).is_none());
    }

    #[test]
    fn test_extract_query_skip_flags() {
        let args = vec![
            "app".to_string(),
            "--verbose".to_string(),
            "cube".to_string(),
        ];
        assert_eq!(extract_query_arg(&args), Some("cube".to_string()));
    }

    #[test]
    fn test_extract_query_first_non_flag() {
        let args = vec![
            "app".to_string(),
            "-v".to_string(),
            "platformer".to_string(),
        ];
        assert_eq!(extract_query_arg(&args), Some("platformer".to_string()));
    }

    #[test]
    fn test_extract_query_skip_multiple_flags() {
        let args = vec![
            "app".to_string(),
            "--debug".to_string(),
            "-v".to_string(),
            "cube".to_string(),
        ];
        assert_eq!(extract_query_arg(&args), Some("cube".to_string()));
    }

    #[test]
    fn test_extract_query_only_flags() {
        let args = vec!["app".to_string(), "--verbose".to_string(), "-d".to_string()];
        assert!(extract_query_arg(&args).is_none());
    }

    #[test]
    fn test_extract_query_single_dash_not_flag() {
        let args = vec!["app".to_string(), "-".to_string()];
        // Single dash alone is not a flag (length == 1)
        assert_eq!(extract_query_arg(&args), Some("-".to_string()));
    }
}
