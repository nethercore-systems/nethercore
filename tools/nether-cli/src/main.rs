//! Nether CLI - Build tool for Nethercore ZX games
//!
//! # Commands
//!
//! - `nether init` - Create a new nether.toml manifest
//! - `nether compile` - Compile WASM from game project (runs build script)
//! - `nether pack` - Bundle WASM + assets into .nczx ROM (no compilation)
//! - `nether build` - Build game: compile + pack (main command)
//! - `nether run` - Build and launch in emulator
//! - `nether preview` - Browse ROM assets without running the game
//!
//! # Usage
//!
//! In a game project directory with nether.toml:
//! ```bash
//! # Build the complete game (compile + pack)
//! nether build
//!
//! # Build and run
//! nether run
//!
//! # Just compile WASM (useful for watch mode)
//! nether compile
//!
//! # Just pack assets (use existing WASM)
//! nether pack
//! ```
//!
//! # Manifest (nether.toml)
//!
//! ```toml
//! [game]
//! id = "my-game"
//! title = "My Game"
//! author = "Developer"
//! version = "1.0.0"
//!
//! # Optional: custom build script (defaults to cargo build for wasm32)
//! [build]
//! script = "cargo build --target wasm32-unknown-unknown --release"
//! wasm = "target/wasm32-unknown-unknown/release/my_game.wasm"
//!
//! # Optional: assets to bundle
//! [[assets.textures]]
//! id = "player"
//! path = "assets/player.png"
//! ```

mod audio_convert;
mod build;
mod compile;
mod init;
mod manifest;
mod pack;
mod preview;
mod run;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Nether CLI - Build tool for Nethercore ZX games
#[derive(Parser)]
#[command(name = "nether")]
#[command(about = "Build tool for Nethercore ZX games")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new nether.toml manifest
    Init(init::InitArgs),

    /// Compile WASM from game project (runs build script from manifest)
    Compile(compile::CompileArgs),

    /// Bundle WASM + assets into .nczx ROM (no compilation)
    Pack(pack::PackArgs),

    /// Build game: compile + pack (main command)
    Build(build::BuildArgs),

    /// Build and launch in emulator
    Run(run::RunArgs),

    /// Browse ROM assets without running the game
    Preview(preview::PreviewArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init(args) => init::execute(args),
        Commands::Compile(args) => {
            compile::execute(args)?;
            Ok(())
        }
        Commands::Pack(args) => pack::execute(args),
        Commands::Build(args) => build::execute(args),
        Commands::Run(args) => run::execute(args),
        Commands::Preview(args) => preview::execute(args),
    }
}
