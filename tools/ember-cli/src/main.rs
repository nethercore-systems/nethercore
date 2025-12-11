//! Ember CLI - Build tool for Emberware Z games
//!
//! # Commands
//!
//! - `ember build` - Build WASM from Rust game project
//! - `ember pack` - Create .ewz ROM from WASM + assets
//! - `ember run` - Build + pack + launch in emulator
//!
//! # Usage
//!
//! In a game project directory:
//! ```bash
//! # Build WASM
//! ember build
//!
//! # Create ROM with assets
//! ember pack --manifest ember.toml -o game.ewz
//!
//! # Build + pack + run
//! ember run
//! ```

mod build;
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
    /// Build WASM from Rust game project
    Build(build::BuildArgs),

    /// Create .ewz ROM from WASM + assets
    Pack(pack::PackArgs),

    /// Build + pack + launch in emulator
    Run(run::RunArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build(args) => build::execute(args),
        Commands::Pack(args) => pack::execute(args),
        Commands::Run(args) => run::execute(args),
    }
}
