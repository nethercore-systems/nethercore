//! FFI binding generator CLI

use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "ffi-gen")]
#[command(about = "Generate FFI bindings from Rust source", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate all FFI bindings
    Generate,

    /// Validate that bindings are in sync
    Validate,

    /// Show diff between current and generated
    Diff,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Generate => {
            println!("Generating FFI bindings...");
            ffi_gen::generate_all()?;
            println!("✓ Done!");
        }
        Commands::Validate => {
            println!("Validating FFI bindings...");
            ffi_gen::validate()?;
            println!("✓ All bindings are valid!");
        }
        Commands::Diff => {
            println!("Diff command not yet implemented");
            // TODO: Implement in Phase 2c
        }
    }

    Ok(())
}
