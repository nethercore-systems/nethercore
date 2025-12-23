//! Install ROM files to the local game library

use anyhow::{Context, Result};
use clap::Args;
use nethercore_core::library::RomLoader;
use std::path::PathBuf;
use zx_common::ZXRomLoader;

#[derive(Debug, Args)]
pub struct InstallArgs {
    /// Path to the ROM file (.nczx)
    #[arg(value_name = "ROM_FILE")]
    pub rom_path: PathBuf,

    /// Override data directory (default: ~/.nethercore)
    #[arg(long, value_name = "DIR")]
    pub data_dir: Option<PathBuf>,
}

/// Execute the install command
pub fn execute(args: InstallArgs) -> Result<()> {
    let rom_path = &args.rom_path;

    // Validate ROM file exists
    if !rom_path.exists() {
        anyhow::bail!("ROM file not found: {}", rom_path.display());
    }

    // Determine ROM type from extension
    let extension = rom_path
        .extension()
        .and_then(|ext| ext.to_str())
        .context("ROM file has no extension")?;

    println!("Installing ROM: {}", rom_path.display());
    println!();

    // Get data directory provider
    let provider = DataDirProviderImpl {
        override_dir: args.data_dir.clone(),
    };

    // Install based on extension
    let game = match extension {
        "nczx" => {
            println!("Detected Nethercore ZX ROM (.nczx)");
            let loader = ZXRomLoader;
            loader
                .install(rom_path, &provider)
                .context("Failed to install Nethercore ZX ROM")?
        }
        _ => {
            anyhow::bail!(
                "Unsupported ROM extension: .{}\nSupported formats: .nczx",
                extension
            );
        }
    };

    println!();
    println!("✓ Successfully installed!");
    println!();
    println!("  Game ID:  {}", game.id);
    println!("  Title:    {}", game.title);
    println!("  Author:   {}", game.author);
    println!("  Version:  {}", game.version);
    println!("  Console:  {}", game.console_type);
    println!();
    println!("Run 'cargo run -- {}' to play the game.", game.id);

    Ok(())
}

/// Data directory provider for installation
struct DataDirProviderImpl {
    override_dir: Option<PathBuf>,
}

impl nethercore_core::library::DataDirProvider for DataDirProviderImpl {
    fn data_dir(&self) -> Option<PathBuf> {
        if let Some(ref dir) = self.override_dir {
            Some(dir.clone())
        } else {
            directories::ProjectDirs::from("io.nethercore", "", "Nethercore")
                .map(|dirs| dirs.data_dir().to_path_buf())
        }
    }
}
