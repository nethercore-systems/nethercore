//! Nethercore ZX - Standalone Player
//!
//! A minimal player for running Nethercore ZX ROM files without the library UI.
//!
//! # Usage
//!
//! ```bash
//! nethercore-zx path/to/game.nczx
//! nethercore-zx game.nczx --fullscreen
//! nethercore-zx game.nczx --debug
//! nethercore-zx game.nczx --preview
//! nethercore-zx game.nczx --preview --asset textures/player
//! ```
//!
//! # Keyboard Shortcuts
//!
//! - ESC: Quit
//! - F3: Toggle debug overlay
//! - F5: Pause/Resume
//! - F6: Frame step (when paused)
//! - F11: Toggle fullscreen

use std::path::PathBuf;

use anyhow::Result;
use clap::Parser;

use nethercore_core::rollback::ConnectionMode;
use nethercore_zx::player::{PlayerConfig, run};
use nethercore_zx::preview::{PreviewConfig, run as run_preview};

#[derive(Parser)]
#[command(name = "nethercore-zx")]
#[command(
    author,
    version,
    about = "Nethercore ZX - PS1/N64 aesthetic fantasy console"
)]
struct Args {
    /// ROM file to play (.nczx or .wasm)
    rom: PathBuf,

    /// Start in fullscreen mode (borderless window, scales to fit)
    #[arg(long, short = 'f')]
    fullscreen: bool,

    /// Integer scaling factor (default: 2, only affects windowed mode)
    #[arg(long, short = 's', default_value = "2")]
    scale: u32,

    /// Enable debug overlay on startup
    #[arg(long, short = 'd')]
    debug: bool,

    // === Multiplayer Options ===
    /// Number of players (1-4)
    #[arg(long, short = 'p', default_value = "1")]
    players: usize,

    /// Input delay in frames (0-10, higher = smoother online play)
    #[arg(long, default_value = "0")]
    input_delay: usize,

    /// Run in sync-test mode to verify game determinism
    #[arg(long)]
    sync_test: bool,

    /// Sync-test check distance (frames between state checksums)
    #[arg(long, default_value = "2")]
    check_distance: usize,

    // === P2P Testing (Local Development) ===
    /// Enable P2P mode for local testing
    #[arg(long)]
    p2p: bool,

    /// Local port to bind for P2P/Host mode
    #[arg(long, default_value = "7777")]
    bind: u16,

    /// Peer port to connect to in P2P mode
    #[arg(long, default_value = "7778")]
    peer: u16,

    /// Which player this instance controls (0 or 1) in P2P mode
    #[arg(long, default_value = "0")]
    local_player: usize,

    // === Network Play ===
    /// Host a multiplayer game on this port
    #[arg(long)]
    host: Option<u16>,

    /// Join a multiplayer game at this address (ip:port)
    #[arg(long)]
    join: Option<String>,

    /// Session config file from library lobby (NCHS pre-negotiated session)
    #[arg(long, value_name = "FILE")]
    session: Option<PathBuf>,

    // === Preview Mode ===
    /// Run in preview mode to inspect ROM assets
    #[arg(long)]
    preview: bool,

    /// Specific asset to focus on in preview mode (e.g., "textures/player")
    #[arg(long)]
    asset: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate ROM path exists
    if !args.rom.exists() {
        anyhow::bail!("ROM file not found: {}", args.rom.display());
    }

    // Handle preview mode
    if args.preview {
        let config = PreviewConfig {
            rom_path: args.rom,
            asset_path: args.asset,
            scale: args.scale,
        };
        return run_preview(config);
    }

    // Validate player count
    if args.players == 0 || args.players > 4 {
        anyhow::bail!("Player count must be between 1 and 4");
    }

    // Validate input delay
    if args.input_delay > 10 {
        anyhow::bail!("Input delay must be between 0 and 10");
    }

    // Determine connection mode from arguments
    // Priority: session > join > host > p2p > sync_test > local
    let connection_mode = if let Some(session_file) = args.session {
        ConnectionMode::Session { session_file }
    } else if let Some(ref address) = args.join {
        ConnectionMode::Join {
            address: address.clone(),
        }
    } else if let Some(port) = args.host {
        ConnectionMode::Host { port }
    } else if args.p2p {
        ConnectionMode::P2P {
            bind_port: args.bind,
            peer_port: args.peer,
            local_player: args.local_player,
        }
    } else if args.sync_test {
        ConnectionMode::SyncTest {
            check_distance: args.check_distance,
        }
    } else {
        ConnectionMode::Local
    };

    let config = PlayerConfig {
        rom_path: args.rom,
        fullscreen: args.fullscreen,
        scale: args.scale,
        debug: args.debug,
        num_players: args.players,
        input_delay: args.input_delay,
        connection_mode,
    };

    run(config)
}
