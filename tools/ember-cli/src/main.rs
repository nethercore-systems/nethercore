//! Ember CLI - Build tool for Emberware Z games
//!
//! # Commands
//!
//! - `ember init` - Create a new ember.toml manifest
//! - `ember compile` - Compile WASM from game project (runs build script)
//! - `ember pack` - Bundle WASM + assets into .ewz ROM (no compilation)
//! - `ember build` - Build game: compile + pack (main command)
//! - `ember run` - Build and launch in emulator
//!
//! # Usage
//!
//! In a game project directory with ember.toml:
//! ```bash
//! # Build the complete game (compile + pack)
//! ember build
//!
//! # Build and run
//! ember run
//!
//! # Just compile WASM (useful for watch mode)
//! ember compile
//!
//! # Just pack assets (use existing WASM)
//! ember pack
//! ```
//!
//! # Manifest (ember.toml)
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

mod build;
mod compile;
mod init;
mod manifest;
mod pack;
mod run;

use anyhow::Result;
use clap::{Parser, Subcommand};

/// Ember CLI - Build tool for Emberware Z games
#[derive(Parser)]
#[command(name = "ember")]
#[command(about = "Build tool for Emberware Z games")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new ember.toml manifest
    Init(init::InitArgs),

    /// Compile WASM from game project (runs build script from manifest)
    Compile(compile::CompileArgs),

    /// Bundle WASM + assets into .ewz ROM (no compilation)
    Pack(pack::PackArgs),

    /// Build game: compile + pack (main command)
    Build(build::BuildArgs),

    /// Build and launch in emulator
    Run(run::RunArgs),
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
    }
}
