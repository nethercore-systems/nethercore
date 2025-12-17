//! Inspect Emberware ROM files

use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Args;
use zx_common::ZRom;

/// Arguments for inspecting ROM metadata
#[derive(Debug, Args)]
pub struct InfoArgs {
    /// Path to the ROM file (.ewz)
    pub rom_file: PathBuf,
}

/// Execute the info command
pub fn execute(args: InfoArgs) -> Result<()> {
    // Detect ROM type from extension
    let extension = args
        .rom_file
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow::anyhow!("ROM file has no extension"))?;

    match extension {
        "ewz" => inspect_z_rom(&args.rom_file),
        _ => anyhow::bail!("Unsupported ROM extension: .{}", extension),
    }
}

/// Inspect an Emberware Z ROM
fn inspect_z_rom(rom_path: &PathBuf) -> Result<()> {
    // Read ROM file
    let rom_bytes = std::fs::read(rom_path)
        .with_context(|| format!("Failed to read ROM file: {}", rom_path.display()))?;

    let file_size = rom_bytes.len();

    // Deserialize and validate
    let rom = ZRom::from_bytes(&rom_bytes)
        .with_context(|| format!("Failed to load EWZ ROM: {}", rom_path.display()))?;

    // Display metadata
    println!("═══════════════════════════════════════════════════════════");
    println!("Emberware Z ROM: {}", rom_path.display());
    println!("═══════════════════════════════════════════════════════════");
    println!();

    println!("GAME INFORMATION");
    println!("───────────────────────────────────────────────────────────");
    println!("  ID:          {}", rom.metadata.id);
    println!("  Title:       {}", rom.metadata.title);
    println!("  Author:      {}", rom.metadata.author);
    println!("  Version:     {}", rom.metadata.version);
    println!("  Description: {}", rom.metadata.description);

    if !rom.metadata.tags.is_empty() {
        println!("  Tags:        {}", rom.metadata.tags.join(", "));
    }

    println!();
    println!("CREATION INFO");
    println!("───────────────────────────────────────────────────────────");
    println!("  Created:     {}", rom.metadata.created_at);
    println!("  Tool:        xtask v{}", rom.metadata.tool_version);
    println!("  ROM version: {}", rom.version);

    // Platform integration
    if rom.metadata.platform_game_id.is_some() || rom.metadata.platform_author_id.is_some() {
        println!();
        println!("PLATFORM INTEGRATION");
        println!("───────────────────────────────────────────────────────────");
        if let Some(ref game_id) = rom.metadata.platform_game_id {
            println!("  Game UUID:   {}", game_id);
        }
        if let Some(ref author_id) = rom.metadata.platform_author_id {
            println!("  Author UUID: {}", author_id);
        }
    }

    // Z-specific settings
    if rom.metadata.render_mode.is_some()
        || rom.metadata.default_resolution.is_some()
        || rom.metadata.target_fps.is_some()
    {
        println!();
        println!("CONSOLE SETTINGS (Emberware Z)");
        println!("───────────────────────────────────────────────────────────");

        if let Some(mode) = rom.metadata.render_mode {
            let mode_name = match mode {
                0 => "Unlit",
                1 => "Matcap",
                2 => "PBR-lite",
                3 => "Hybrid",
                _ => "Unknown",
            };
            println!("  Render mode: {} ({})", mode, mode_name);
        }

        if let Some(ref res) = rom.metadata.default_resolution {
            println!("  Resolution:  {}", res);
        }

        if let Some(fps) = rom.metadata.target_fps {
            println!("  Target FPS:  {}", fps);
        }
    }

    println!();
    println!("ROM CONTENTS");
    println!("───────────────────────────────────────────────────────────");
    println!("  ROM file:    {} bytes", format_bytes(file_size));
    println!("  WASM code:   {} bytes", format_bytes(rom.code.len()));

    if let Some(ref thumb) = rom.thumbnail {
        println!(
            "  Thumbnail:   {} bytes (extracted locally)",
            format_bytes(thumb.len())
        );
    } else {
        println!("  Thumbnail:   none");
    }

    if !rom.screenshots.is_empty() {
        let total_screenshot_size: usize = rom.screenshots.iter().map(|s| s.len()).sum();
        println!(
            "  Screenshots: {} images, {} bytes (stay in ROM)",
            rom.screenshots.len(),
            format_bytes(total_screenshot_size)
        );
    } else {
        println!("  Screenshots: none");
    }

    println!();
    println!("═══════════════════════════════════════════════════════════");

    Ok(())
}

/// Format byte count with commas
fn format_bytes(bytes: usize) -> String {
    let s = bytes.to_string();
    let mut result = String::new();

    for (count, c) in s.chars().rev().enumerate() {
        if count > 0 && count % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result.chars().rev().collect()
}
