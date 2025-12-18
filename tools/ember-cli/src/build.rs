//! Build command - compile WASM and pack into cartridge
//!
//! This is the main command for game developers. It orchestrates:
//! 1. Compile: Run build script → WASM
//! 2. Pack: Bundle WASM + assets → .ewzx cartridge
//!
//! # Future optimization: Concurrent compile + asset loading
//!
//! Currently, compilation must complete before asset packing begins (we need the
//! WASM to analyze render_mode for texture compression format). However, we could
//! overlap compilation with asset pre-loading:
//!
//! ```ignore
//! // 1. Read manifest
//! let manifest = EmberManifest::load(&manifest_path)?;
//!
//! // 2. Start compile in background thread
//! let compile_handle = std::thread::spawn(move || {
//!     compile::execute(compile_args)
//! });
//!
//! // 3. Pre-load assets (textures as RGBA8, defer BC7 compression)
//! let preloaded = preload_assets(&manifest.assets)?;
//! // preloaded.textures: Vec<(String, RgbaImage)> - not yet compressed
//! // preloaded.meshes, sounds, etc: fully loaded
//!
//! // 4. Wait for compile
//! let wasm_path = compile_handle.join()??;
//!
//! // 5. Analyze WASM for render_mode
//! let analysis = analyze_wasm(&std::fs::read(&wasm_path)?)?;
//!
//! // 6. Compress textures based on render_mode (parallel with rayon)
//! let textures = preloaded.textures.par_iter()
//!     .map(|(id, img)| compress_texture(id, img, analysis.render_mode))
//!     .collect()?;
//!
//! // 7. Bundle and write .ewzx
//! ```
//!
//! This would overlap I/O-bound compilation with CPU-bound asset loading,
//! potentially reducing total build time for projects with many assets.

use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::compile::{self, CompileArgs};
use crate::manifest::EmberManifest;
use crate::pack::{self, PackArgs};

/// Arguments for the build command
#[derive(Args)]
pub struct BuildArgs {
    /// Path to game project directory (defaults to current directory)
    #[arg(short, long)]
    pub project: Option<PathBuf>,

    /// Path to ember.toml manifest file (relative to project directory)
    #[arg(short, long, default_value = "ember.toml")]
    pub manifest: PathBuf,

    /// Output .ewzx file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Build in debug mode (default is release)
    #[arg(long)]
    pub debug: bool,

    /// Skip compilation, just pack (use existing WASM)
    #[arg(long)]
    pub no_compile: bool,
}

/// Execute the build command
pub fn execute(args: BuildArgs) -> Result<()> {
    let project_dir = args
        .project
        .clone()
        .unwrap_or_else(|| std::env::current_dir().unwrap());

    let manifest_path = project_dir.join(&args.manifest);

    // Step 1: Compile (unless --no-compile)
    let wasm_path = if !args.no_compile {
        println!("=== Compiling ===");
        compile::execute(CompileArgs {
            project: Some(project_dir.clone()),
            manifest: args.manifest.clone(),
            debug: args.debug,
        })?
    } else {
        println!("=== Skipping compilation ===");
        // Find existing WASM
        let manifest = EmberManifest::load(&manifest_path)?;
        manifest.find_wasm(&project_dir, args.debug)?
    };

    println!();

    // Step 2: Pack
    println!("=== Packing ===");
    pack::execute(PackArgs {
        manifest: manifest_path,
        output: args.output,
        wasm: Some(wasm_path),
    })?;

    Ok(())
}
