//! Unified asset generator for Nethercore showcase games
//!
//! Generates procedural 3D meshes for:
//! - PRISM SURVIVORS (top-down shooter) - Mode 3
//! - NEON DRIFT (arcade racer) - Mode 2
//! - LUMINA DEPTHS (underwater exploration) - Mode 3
//!
//! Also auto-generates the proc-gen preview viewers for each render mode.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod lumina_depths;
mod neon_drift;
mod prism_survivors;

/// Asset definition: (id, display_name)
type AssetDef = (&'static str, &'static str);

/// Showcase definition with render mode and assets
struct ShowcaseDef {
    name: &'static str,
    render_mode: u8,
    folder: &'static str,
    meshes: &'static [AssetDef],
}

/// All showcase definitions - single source of truth
const SHOWCASES: &[ShowcaseDef] = &[
    ShowcaseDef {
        name: "NEON DRIFT",
        render_mode: 2,
        folder: "neon-drift",
        meshes: &[
            // Vehicles
            ("speedster", "Speedster"),
            ("muscle", "Muscle Car"),
            ("racer", "Racer"),
            ("drift", "Drift Car"),
            // Track segments
            ("track_straight", "Track: Straight"),
            ("track_curve_left", "Track: Curve Left"),
            ("track_tunnel", "Track: Tunnel"),
            ("track_jump", "Track: Jump Ramp"),
            // Props
            ("prop_barrier", "Prop: Barrier"),
            ("prop_boost_pad", "Prop: Boost Pad"),
            ("prop_billboard", "Prop: Billboard"),
            ("prop_building", "Prop: Building"),
        ],
    },
    ShowcaseDef {
        name: "PRISM SURVIVORS",
        render_mode: 3,
        folder: "prism-survivors",
        meshes: &[
            // Heroes
            ("knight", "Knight"),
            ("mage", "Mage"),
            ("ranger", "Ranger"),
            ("cleric", "Cleric"),
            // Enemies
            ("golem", "Golem"),
            ("crawler", "Crawler"),
            ("wisp", "Wisp"),
            ("skeleton", "Skeleton"),
        ],
    },
    ShowcaseDef {
        name: "LUMINA DEPTHS",
        render_mode: 3,
        folder: "lumina-depths",
        meshes: &[
            // Submersible
            ("submersible", "Submersible"),
            // Creatures
            ("reef_fish", "Reef Fish"),
            ("sea_turtle", "Sea Turtle"),
            ("manta_ray", "Manta Ray"),
            ("moon_jelly", "Moon Jelly"),
            ("anglerfish", "Anglerfish"),
            ("blue_whale", "Blue Whale"),
            // Flora
            ("coral_brain", "Brain Coral"),
            ("kelp", "Kelp"),
            ("anemone", "Anemone"),
            // Terrain
            ("vent_chimney", "Vent Chimney"),
            ("tube_worms", "Tube Worms"),
        ],
    },
];

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

            // Auto-generate preview viewers
            println!("\n=== Generating Preview Viewers ===");
            generate_preview_viewers(&output);

            println!("\n=== All assets generated successfully ===");
        }
    }
}

/// Auto-generate preview viewer manifests and code for each render mode
fn generate_preview_viewers(examples_dir: &PathBuf) {
    // Group showcases by render mode
    let mode2_showcases: Vec<_> = SHOWCASES.iter().filter(|s| s.render_mode == 2).collect();
    let mode3_showcases: Vec<_> = SHOWCASES.iter().filter(|s| s.render_mode == 3).collect();

    if !mode2_showcases.is_empty() {
        generate_mode_preview(examples_dir, 2, &mode2_showcases);
    }

    if !mode3_showcases.is_empty() {
        generate_mode_preview(examples_dir, 3, &mode3_showcases);
    }
}

/// Generate nether.toml and lib.rs for a specific render mode
fn generate_mode_preview(examples_dir: &PathBuf, mode: u8, showcases: &[&ShowcaseDef]) {
    let preview_dir = examples_dir.join(format!("proc-gen-mode{}", mode));

    // Collect showcase names for title
    let showcase_names: Vec<_> = showcases.iter().map(|s| s.name).collect();
    let title = format!("Mode {} Preview ({})", mode, showcase_names.join(" + "));

    println!("  Generating proc-gen-mode{}/", mode);

    // Generate nether.toml
    let mut toml = String::new();
    toml.push_str(&format!(
        r#"[game]
id = "mode{mode}-preview"
title = "{title}"
author = "Nethercore Examples"
version = "0.1.0"

# Auto-generated by: cargo run -p gen-assets -- all
# Showcases: {showcase_names}

"#,
        mode = mode,
        title = title,
        showcase_names = showcase_names.join(", ")
    ));

    // Add mesh entries for each showcase
    for showcase in showcases {
        toml.push_str(&format!("# {} assets\n", showcase.name));
        for (id, _name) in showcase.meshes {
            toml.push_str(&format!(
                r#"[[assets.meshes]]
id = "{id}"
path = "../{folder}/assets/models/meshes/{id}.obj"

"#,
                id = id,
                folder = showcase.folder
            ));
        }
    }

    let toml_path = preview_dir.join("nether.toml");
    fs::write(&toml_path, &toml).expect("Failed to write nether.toml");
    println!("    -> {}", toml_path.display());

    // Generate lib.rs
    let mut lib_rs = String::new();
    lib_rs.push_str(&format!(
        r#"//! Mode {mode} Asset Preview ({showcase_names})
//!
//! Auto-generated by: cargo run -p gen-assets -- all
//! Uses render_mode({mode}) for viewing generated assets.

#![no_std]
#![no_main]

proc_gen_common::asset_viewer!({mode}, "{title}", [
"#,
        mode = mode,
        title = title,
        showcase_names = showcase_names.join(" + ")
    ));

    // Add asset entries
    for showcase in showcases {
        lib_rs.push_str(&format!("    // {}\n", showcase.name));
        for (id, name) in showcase.meshes {
            lib_rs.push_str(&format!("    (\"{}\", \"{}\"),\n", id, name));
        }
    }

    lib_rs.push_str("]);\n");

    let lib_path = preview_dir.join("src/lib.rs");
    fs::write(&lib_path, &lib_rs).expect("Failed to write lib.rs");
    println!("    -> {}", lib_path.display());
}
