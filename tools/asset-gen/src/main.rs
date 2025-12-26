//! Unified asset generator for Nethercore showcase games
//!
//! Generates procedural 3D meshes for:
//! - PRISM SURVIVORS (top-down shooter)
//! - NEON DRIFT (arcade racer)
//! - LUMINA DEPTHS (underwater exploration)

use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod prism_survivors;
mod neon_drift;
mod lumina_depths;

#[derive(Parser)]
#[command(name = "asset-gen")]
#[command(about = "Generate procedural assets for Nethercore showcase games")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate PRISM SURVIVORS assets (player, enemies, projectiles, pickups)
    PrismSurvivors {
        /// Output directory for generated assets
        #[arg(short, long, default_value = "examples/prism-survivors/assets/models")]
        output: PathBuf,
    },
    /// Generate NEON DRIFT assets (cars, track elements)
    NeonDrift {
        /// Output directory for generated assets
        #[arg(short, long, default_value = "examples/neon-drift/assets/models")]
        output: PathBuf,
    },
    /// Generate LUMINA DEPTHS assets (submersible, creatures, flora, terrain)
    LuminaDepths {
        /// Output directory for generated assets
        #[arg(short, long, default_value = "examples/lumina-depths/assets/models")]
        output: PathBuf,
    },
    /// Generate all showcase game assets
    All {
        /// Base output directory
        #[arg(short, long, default_value = "examples")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::PrismSurvivors { output } => {
            println!("=== PRISM SURVIVORS Asset Generation ===");
            prism_survivors::generate_all(&output);
        }
        Commands::NeonDrift { output } => {
            println!("=== NEON DRIFT Asset Generation ===");
            neon_drift::generate_all(&output);
        }
        Commands::LuminaDepths { output } => {
            println!("=== LUMINA DEPTHS Asset Generation ===");
            lumina_depths::generate_all(&output);
        }
        Commands::All { output } => {
            println!("=== Generating ALL Showcase Assets ===\n");

            println!("--- PRISM SURVIVORS ---");
            prism_survivors::generate_all(&output.join("prism-survivors/assets/models"));

            println!("\n--- NEON DRIFT ---");
            neon_drift::generate_all(&output.join("neon-drift/assets/models"));

            println!("\n--- LUMINA DEPTHS ---");
            lumina_depths::generate_all(&output.join("lumina-depths/assets/models"));

            println!("\n=== All assets generated successfully ===");
        }
    }
}
