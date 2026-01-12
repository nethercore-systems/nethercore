use anyhow::{Context, Result};
use clap::Args;
use nethercore_shared::{read_file_with_limit, MAX_WASM_BYTES};
use std::path::PathBuf;
use xxhash_rust::xxh3::xxh3_64;

use nethercore_shared::netplay::NetplayMetadata;
use nethercore_shared::ConsoleType;

use super::{assets, manifest, output, validation};

/// Arguments for the pack command
#[derive(Args)]
pub struct PackArgs {
    /// Path to nether.toml manifest file
    #[arg(short, long, default_value = "nether.toml")]
    pub manifest: PathBuf,

    /// Output .nczx file path
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Path to WASM file (overrides auto-detection)
    #[arg(long)]
    pub wasm: Option<PathBuf>,
}

/// Execute the pack command
pub fn execute(args: PackArgs) -> Result<()> {
    let ctx = manifest::load_manifest(&args.manifest)?;

    println!(
        "Packing game: {} ({})",
        ctx.manifest.game.title, ctx.manifest.game.id
    );

    let wasm_path = manifest::resolve_wasm_path(&ctx, args.wasm)?;

    let code = read_file_with_limit(&wasm_path, MAX_WASM_BYTES)
        .with_context(|| format!("Failed to read WASM file: {}", wasm_path.display()))?;
    println!("  WASM: {} ({} bytes)", wasm_path.display(), code.len());

    let rom_hash = xxh3_64(&code);
    println!("  ROM hash: {:016x}", rom_hash);

    let render_mode = ctx.manifest.game.render_mode;
    let mode_name = validation::render_mode_name(render_mode);
    println!("  Render mode: {} ({})", render_mode, mode_name);

    let texture_format = validation::select_texture_format(ctx.manifest.game.compress_textures);
    validation::warn_compression_mismatch(render_mode, ctx.manifest.game.compress_textures);

    let data_pack = assets::load_assets(&ctx.project_dir, &ctx.manifest.assets, texture_format)?;

    let max_players = if ctx.manifest.netplay.enabled {
        ctx.manifest.game.max_players
    } else {
        1
    };
    let netplay = NetplayMetadata::new(
        ConsoleType::ZX,
        ctx.manifest.tick_rate(),
        max_players,
        rom_hash,
    );

    if ctx.manifest.netplay.enabled {
        println!(
            "  Netplay: enabled ({}Hz, {} players)",
            ctx.manifest.game.tick_rate, ctx.manifest.game.max_players
        );
    } else {
        println!("  Netplay: disabled");
    }

    let rom = output::build_rom(&ctx.manifest, code, data_pack, render_mode, netplay);

    rom.validate().context("ROM validation failed")?;

    let output_path = output::default_output_path(&ctx.project_dir, &ctx.manifest, args.output);
    let rom_bytes = output::serialize_rom(&rom)?;
    output::write_rom(&output_path, &rom_bytes)?;
    output::print_summary(&output_path, rom_bytes.len(), &ctx.manifest);

    Ok(())
}
