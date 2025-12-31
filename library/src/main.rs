//! Unified Nethercore launcher
//!
//! This binary can launch games from any supported console type (Z, Classic, etc.)
//! by detecting the console type from the game manifest.
//!
//! # URL Scheme
//!
//! Supports `nethercore://` deep links:
//! - `nethercore://play/{game_id}` - Play a local game
//! - `nethercore://download/{game_id}` - Download and play (opens browser if not installed)
//! - `nethercore://host/{game_id}?port=7777&players=2` - Host a multiplayer game
//! - `nethercore://join/{ip}:{port}/{game_id}` - Join a hosted multiplayer game

use anyhow::Result;
use nethercore_core::library::{DataDirProvider, LocalGame, get_local_games, resolve_game_id};
use nethercore_library::registry::{ConnectionMode, ConsoleRegistry, PlayerOptions};
use nethercore_library::update::check_and_prompt_for_update;
use std::env;
use std::path::PathBuf;

/// Data directory provider for the unified launcher.
struct LauncherDataDirProvider;

impl DataDirProvider for LauncherDataDirProvider {
    fn data_dir(&self) -> Option<PathBuf> {
        directories::ProjectDirs::from("io.nethercore", "", "Nethercore")
            .map(|dirs| dirs.data_dir().to_path_buf())
    }
}

/// Actions that can be triggered by deep links
#[derive(Debug, Clone)]
pub enum DeepLinkAction {
    /// Play a local game
    Play { game_id: String },
    /// Download a game (and play if already installed)
    Download { game_id: String },
    /// Host a multiplayer game
    Host {
        game_id: String,
        port: u16,
        players: usize,
    },
    /// Join a hosted multiplayer game
    Join {
        game_id: String,
        host_ip: String,
        port: u16,
    },
}

/// Parse deep link from command line args
///
/// Supports:
/// - `nethercore://play/{game_id}`
/// - `nethercore://download/{game_id}`
/// - `nethercore://host/{game_id}?port=7777&players=2`
/// - `nethercore://join/{ip}:{port}/{game_id}`
fn parse_deep_link(args: &[String]) -> Option<DeepLinkAction> {
    for arg in args.iter().skip(1) {
        if let Some(rest) = arg.strip_prefix("nethercore://") {
            return parse_nethercore_url(rest);
        }
    }
    None
}

/// Parse a nethercore:// URL path into an action
fn parse_nethercore_url(path: &str) -> Option<DeepLinkAction> {
    // Split into action and rest: "play/game_id" -> ("play", "game_id")
    let (action, rest) = path.split_once('/')?;

    match action {
        "play" => {
            let game_id = rest.trim_end_matches('/').to_string();
            if game_id.is_empty() {
                return None;
            }
            Some(DeepLinkAction::Play { game_id })
        }
        "download" => {
            let game_id = rest.trim_end_matches('/').to_string();
            if game_id.is_empty() {
                return None;
            }
            Some(DeepLinkAction::Download { game_id })
        }
        "host" => {
            // Parse: {game_id}?port=7777&players=2
            let (game_part, query) = rest.split_once('?').unwrap_or((rest, ""));
            let game_id = game_part.trim_end_matches('/').to_string();
            if game_id.is_empty() {
                return None;
            }

            let port = parse_query_param(query, "port").unwrap_or(7777);
            let players = parse_query_param(query, "players").unwrap_or(2);

            Some(DeepLinkAction::Host {
                game_id,
                port,
                players,
            })
        }
        "join" => {
            // Parse: {ip}:{port}/{game_id}
            // e.g., "192.168.1.100:7777/paddle-demo" or "[::1]:7777/paddle-demo"
            let (addr, game_id) = rest.rsplit_once('/')?;
            let game_id = game_id.trim_end_matches('/').to_string();
            if game_id.is_empty() {
                return None;
            }

            // Handle IPv6 addresses in brackets: [::1]:7777
            let (host_ip, port) = if addr.starts_with('[') {
                // IPv6: [::1]:7777
                let (ip_bracket, port_str) = addr.rsplit_once("]:")?;
                let ip = ip_bracket.trim_start_matches('[').to_string();
                let port: u16 = port_str.parse().ok()?;
                (ip, port)
            } else {
                // IPv4: 192.168.1.100:7777
                let (ip, port_str) = addr.rsplit_once(':')?;
                let port: u16 = port_str.parse().ok()?;
                (ip.to_string(), port)
            };

            Some(DeepLinkAction::Join {
                game_id,
                host_ip,
                port,
            })
        }
        _ => None,
    }
}

/// Parse a query parameter value from a query string
fn parse_query_param<T: std::str::FromStr>(query: &str, key: &str) -> Option<T> {
    for pair in query.split('&') {
        if let Some((k, v)) = pair.split_once('=')
            && k == key {
                return v.parse().ok();
            }
    }
    None
}

/// Check if a string looks like a file path to a ROM file
fn is_rom_path(arg: &str) -> Option<PathBuf> {
    let path = PathBuf::from(arg);

    // Check if it has a ROM extension
    let ext = path.extension().and_then(|e| e.to_str());
    let is_rom_ext = matches!(ext, Some("nczx") | Some("wasm"));

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

    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-f" | "--fullscreen" => options.fullscreen = true,
            "-d" | "--debug" => options.debug = true,
            "--preview" => options.preview = true,
            "--asset" => {
                if let Some(asset_name) = iter.next() {
                    options.preview_asset = Some(asset_name.clone());
                }
            }
            _ => {}
        }
    }

    options
}

/// Run a game with update checking
///
/// Checks for updates before launching. If an update is available and the user
/// accepts, downloads and installs it, then reloads the game data before launching.
fn run_game_with_update_check(
    game: &LocalGame,
    registry: &ConsoleRegistry,
    provider: &impl DataDirProvider,
    options: &PlayerOptions,
) -> Result<()> {
    // Get data directory for update operations
    if let Some(data_dir) = provider.data_dir() {
        // Check for updates (has 3-second timeout, won't block if offline)
        let updated = check_and_prompt_for_update(game, &data_dir);

        if updated {
            // Reload game data to get updated version
            let games = get_local_games(provider);
            if let Some(updated_game) = games.iter().find(|g| g.id == game.id) {
                tracing::info!(
                    "Launching updated game: {} v{}",
                    updated_game.title,
                    updated_game.version
                );
                return registry.run_game_with_options(updated_game, options);
            }
        }
    }

    // Launch with original game data (no update or update check failed)
    registry.run_game_with_options(game, options)
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    // Register protocol handler on first launch (idempotent, non-fatal if fails)
    if let Err(e) = nethercore_library::protocol::register() {
        tracing::warn!("Failed to register protocol handler: {}", e);
    }

    let registry = ConsoleRegistry::new();
    let provider = LauncherDataDirProvider;

    // Check for CLI game argument or deep link
    let args: Vec<String> = env::args().collect();

    // Parse player options from CLI flags
    let options = parse_player_options(&args);

    // Try deep link first (nethercore://...)
    if let Some(action) = parse_deep_link(&args) {
        tracing::info!("Deep link detected: {:?}", action);
        return handle_deep_link_action(action, &registry, &provider, &options);
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
            eprintln!("Tip: You can also pass a .nczx file path directly.");
            std::process::exit(1);
        }

        // Resolve game ID
        match resolve_game_id(query, &games) {
            Ok(game_id) => {
                // Find the game to get its console type
                if let Some(game) = games.iter().find(|g| g.id == game_id) {
                    tracing::info!("Running '{}' (console: {})", game.title, game.console_type);

                    // Check for updates and run
                    run_game_with_update_check(game, &registry, &provider, &options)?;
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
        tracing::info!("Launching Nethercore Library");

        registry.launch_library()?;
    }

    Ok(())
}

/// Handle a deep link action
fn handle_deep_link_action(
    action: DeepLinkAction,
    registry: &ConsoleRegistry,
    provider: &impl DataDirProvider,
    base_options: &PlayerOptions,
) -> Result<()> {
    let games = get_local_games(provider);

    match action {
        DeepLinkAction::Play { game_id } => {
            tracing::info!("Playing game: {}", game_id);
            if let Some(game) = games.iter().find(|g| g.id == game_id) {
                run_game_with_update_check(game, registry, provider, base_options)?;
            } else {
                eprintln!(
                    "Game '{}' not found locally. Install it from nethercore.systems",
                    game_id
                );
                std::process::exit(1);
            }
        }
        DeepLinkAction::Download { game_id } => {
            tracing::info!("Download request for game: {}", game_id);
            // If already installed, just play it
            if let Some(game) = games.iter().find(|g| g.id == game_id) {
                tracing::info!("Game already installed, launching");
                run_game_with_update_check(game, registry, provider, base_options)?;
            } else {
                // Not installed - open browser to download page
                let url = format!("https://nethercore.systems/game/{}", game_id);
                tracing::info!("Game not installed, opening browser: {}", url);
                if let Err(e) = open::that(&url) {
                    eprintln!("Failed to open browser: {}", e);
                    eprintln!("Please visit: {}", url);
                }
            }
        }
        DeepLinkAction::Host {
            game_id,
            port,
            players,
        } => {
            tracing::info!(
                "Hosting game: {} on port {} with {} players",
                game_id,
                port,
                players
            );
            if let Some(game) = games.iter().find(|g| g.id == game_id) {
                let options = PlayerOptions {
                    players: Some(players),
                    connection: Some(ConnectionMode::Host { port }),
                    ..base_options.clone()
                };
                run_game_with_update_check(game, registry, provider, &options)?;
            } else {
                eprintln!("Game '{}' not found. Install it first.", game_id);
                std::process::exit(1);
            }
        }
        DeepLinkAction::Join {
            game_id,
            host_ip,
            port,
        } => {
            tracing::info!("Joining game: {} at {}:{}", game_id, host_ip, port);
            if let Some(game) = games.iter().find(|g| g.id == game_id) {
                let options = PlayerOptions {
                    connection: Some(ConnectionMode::Join { host_ip, port }),
                    ..base_options.clone()
                };
                run_game_with_update_check(game, registry, provider, &options)?;
            } else {
                eprintln!("Game '{}' not found. Install it first.", game_id);
                // Could offer to download then join, but for now just exit
                std::process::exit(1);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_play_url() {
        let action = parse_nethercore_url("play/paddle-demo");
        assert!(
            matches!(action, Some(DeepLinkAction::Play { game_id }) if game_id == "paddle-demo")
        );
    }

    #[test]
    fn test_parse_play_url_with_trailing_slash() {
        let action = parse_nethercore_url("play/paddle-demo/");
        assert!(
            matches!(action, Some(DeepLinkAction::Play { game_id }) if game_id == "paddle-demo")
        );
    }

    #[test]
    fn test_parse_download_url() {
        let action = parse_nethercore_url("download/my-game");
        assert!(
            matches!(action, Some(DeepLinkAction::Download { game_id }) if game_id == "my-game")
        );
    }

    #[test]
    fn test_parse_host_url_defaults() {
        let action = parse_nethercore_url("host/paddle-demo");
        assert!(matches!(
            action,
            Some(DeepLinkAction::Host { game_id, port, players })
            if game_id == "paddle-demo" && port == 7777 && players == 2
        ));
    }

    #[test]
    fn test_parse_host_url_with_params() {
        let action = parse_nethercore_url("host/my-game?port=8888&players=4");
        assert!(matches!(
            action,
            Some(DeepLinkAction::Host { game_id, port, players })
            if game_id == "my-game" && port == 8888 && players == 4
        ));
    }

    #[test]
    fn test_parse_join_url_ipv4() {
        let action = parse_nethercore_url("join/192.168.1.100:7777/paddle-demo");
        assert!(matches!(
            action,
            Some(DeepLinkAction::Join { game_id, host_ip, port })
            if game_id == "paddle-demo" && host_ip == "192.168.1.100" && port == 7777
        ));
    }

    #[test]
    fn test_parse_join_url_ipv6() {
        let action = parse_nethercore_url("join/[::1]:7777/paddle-demo");
        assert!(matches!(
            action,
            Some(DeepLinkAction::Join { game_id, host_ip, port })
            if game_id == "paddle-demo" && host_ip == "::1" && port == 7777
        ));
    }

    #[test]
    fn test_parse_invalid_url() {
        assert!(parse_nethercore_url("invalid/").is_none());
        assert!(parse_nethercore_url("play/").is_none());
        assert!(parse_nethercore_url("").is_none());
    }

    #[test]
    fn test_parse_query_param() {
        assert_eq!(
            parse_query_param::<u16>("port=8080&host=foo", "port"),
            Some(8080)
        );
        assert_eq!(parse_query_param::<usize>("players=4", "players"), Some(4));
        assert_eq!(parse_query_param::<u16>("foo=bar", "port"), None);
    }
}
