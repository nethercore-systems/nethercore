//! ember-export - Emberware asset export tool
//!
//! Converts raw assets (glTF, PNG, WAV, TTF) to GPU-ready binary formats
//! (.ewzmesh, .ewztex, .ewzsnd)

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod codegen;
mod formats;
mod manifest;
mod mesh;
mod texture;
mod audio;
// mod font;  // Deferred

// Re-export packing functions and vertex format constants from z-common
pub use z_common::{
    pack_bone_weights_unorm8, pack_color_rgba_unorm8, pack_normal_octahedral, pack_normal_snorm16,
    pack_position_f16, pack_uv_f16, pack_uv_unorm16, pack_vertex_data, unpack_octahedral_u32,
    vertex_stride, vertex_stride_packed, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV,
};

// Re-export file extension constants
pub use emberware_shared::formats::{EWZ_MESH_EXT, EWZ_SOUND_EXT, EWZ_TEXTURE_EXT};

#[derive(Parser)]
#[command(name = "ember-export")]
#[command(about = "Emberware asset export tool")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build assets from a manifest file
    Build {
        /// Path to assets.toml manifest
        #[arg(default_value = "assets.toml")]
        manifest: PathBuf,

        /// Output directory (overrides manifest)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },

    /// Validate manifest without building
    Check {
        /// Path to assets.toml manifest
        #[arg(default_value = "assets.toml")]
        manifest: PathBuf,
    },

    /// Export a single mesh file
    Mesh {
        /// Input mesh file (glTF/GLB/OBJ)
        input: PathBuf,

        /// Output .ewzmesh file
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Vertex format (e.g., POS_UV_NORMAL)
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Export a single texture file
    Texture {
        /// Input PNG/JPG file
        input: PathBuf,

        /// Output .ewztex file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Export a single audio file
    Audio {
        /// Input WAV file
        input: PathBuf,

        /// Output .ewzsnd file
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Build { manifest, output, verbose } => {
            if verbose {
                tracing::info!("Building assets from {:?}", manifest);
            }
            let config = manifest::load_manifest(&manifest)?;
            manifest::build_all(&config, output.as_deref())?;
            tracing::info!("Build complete!");
        }

        Commands::Check { manifest } => {
            tracing::info!("Checking manifest {:?}", manifest);
            let config = manifest::load_manifest(&manifest)?;
            manifest::validate(&config)?;
            tracing::info!("Manifest is valid!");
        }

        Commands::Mesh { input, output, format } => {
            let output = output.unwrap_or_else(|| input.with_extension(EWZ_MESH_EXT));
            tracing::info!("Converting {:?} -> {:?}", input, output);

            // Detect format by extension
            let ext = input.extension()
                .and_then(|e| e.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            match ext.as_str() {
                "obj" => mesh::convert_obj(&input, &output, format.as_deref())?,
                "gltf" | "glb" => mesh::convert_gltf(&input, &output, format.as_deref())?,
                _ => anyhow::bail!("Unsupported mesh format: {:?} (use .obj, .gltf, or .glb)", input),
            }
            tracing::info!("Done!");
        }

        Commands::Texture { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension(EWZ_TEXTURE_EXT));
            tracing::info!("Converting {:?} -> {:?}", input, output);
            texture::convert_image(&input, &output)?;
            tracing::info!("Done!");
        }

        Commands::Audio { input, output } => {
            let output = output.unwrap_or_else(|| input.with_extension(EWZ_SOUND_EXT));
            tracing::info!("Converting {:?} -> {:?}", input, output);
            audio::convert_wav(&input, &output)?;
            tracing::info!("Done!");
        }
    }

    Ok(())
}
