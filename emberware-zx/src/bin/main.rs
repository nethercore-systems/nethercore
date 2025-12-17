//! Emberware Z - Standalone Player
//!
//! A minimal player for running .ewz ROM files without the library UI.
//!
//! # Usage
//!
//! ```bash
//! emberware-z path/to/game.ewz
//! emberware-z game.ewz --fullscreen
//! emberware-z game.ewz --debug
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

use emberware_zx::player::{PlayerConfig, run};

#[derive(Parser)]
#[command(name = "emberware-z")]
#[command(
    author,
    version,
    about = "Emberware Z - PS1/N64 aesthetic fantasy console"
)]
struct Args {
    /// ROM file to play (.ewz or .wasm)
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
}

fn main() -> Result<()> {
    let args = Args::parse();

    // Validate ROM path exists
    if !args.rom.exists() {
        anyhow::bail!("ROM file not found: {}", args.rom.display());
    }

    let config = PlayerConfig {
        rom_path: args.rom,
        fullscreen: args.fullscreen,
        scale: args.scale,
        debug: args.debug,
    };

    run(config)
}
