use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

use nethercore_shared::netplay::NetplayMetadata;
use nethercore_shared::ZX_ROM_FORMAT;
use zx_common::{ZXDataPack, ZXMetadata, ZXRom};

use crate::manifest::NetherManifest;

pub fn build_metadata(
    manifest: &NetherManifest,
    render_mode: u8,
    netplay: NetplayMetadata,
) -> ZXMetadata {
    ZXMetadata {
        id: manifest.game.id.clone(),
        title: manifest.game.title.clone(),
        author: manifest.game.author.clone(),
        version: manifest.game.version.clone(),
        description: manifest.game.description.clone(),
        tags: manifest.game.tags.clone(),
        platform_game_id: None,
        platform_author_id: None,
        created_at: chrono::Utc::now().to_rfc3339(),
        tool_version: env!("CARGO_PKG_VERSION").to_string(),
        render_mode: Some(render_mode as u32),
        default_resolution: None,
        target_fps: None,
        netplay,
    }
}

pub fn build_rom(
    manifest: &NetherManifest,
    code: Vec<u8>,
    data_pack: ZXDataPack,
    render_mode: u8,
    netplay: NetplayMetadata,
) -> ZXRom {
    let metadata = build_metadata(manifest, render_mode, netplay);

    ZXRom {
        version: ZX_ROM_FORMAT.version,
        metadata,
        code,
        data_pack: if data_pack.is_empty() {
            None
        } else {
            Some(data_pack)
        },
        thumbnail: None,
        screenshots: vec![],
    }
}

pub fn default_output_path(
    project_dir: &Path,
    manifest: &NetherManifest,
    override_path: Option<PathBuf>,
) -> PathBuf {
    override_path.unwrap_or_else(|| {
        project_dir.join(format!(
            "{}.{}",
            manifest.game.id, ZX_ROM_FORMAT.extension
        ))
    })
}

pub fn serialize_rom(rom: &ZXRom) -> Result<Vec<u8>> {
    rom.to_bytes().context("Failed to serialize ROM")
}

pub fn write_rom(output_path: &Path, rom_bytes: &[u8]) -> Result<()> {
    std::fs::write(output_path, rom_bytes)
        .with_context(|| format!("Failed to write ROM: {}", output_path.display()))?;
    Ok(())
}

pub fn print_summary(output_path: &Path, rom_size: usize, manifest: &NetherManifest) {
    println!();
    println!("Created: {} ({} bytes)", output_path.display(), rom_size);
    println!("  Game ID: {}", manifest.game.id);
    println!("  Title: {}", manifest.game.title);
    println!("  Version: {}", manifest.game.version);
}
