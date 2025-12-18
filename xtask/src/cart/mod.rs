//! ROM cart management commands
//!
//! This module provides CLI commands for creating, inspecting, and managing
//! Emberware ROM files (.ewzx, .ewc, etc.).

pub mod create_zx;
pub mod info;
pub mod install;

use anyhow::Result;
use clap::Subcommand;

/// Cart management subcommands
#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum CartCommand {
    /// Create an Emberware ZX ROM (.ewzx) from a WASM file
    #[command(name = "create-zx")]
    CreateZx(create_zx::CreateZxArgs),

    /// Display ROM metadata and information
    Info(info::InfoArgs),

    /// Install a ROM file to the local game library
    Install(install::InstallArgs),
}

/// Execute a cart command
pub fn execute(cmd: CartCommand) -> Result<()> {
    match cmd {
        CartCommand::CreateZx(args) => create_zx::execute(args),
        CartCommand::Info(args) => info::execute(args),
        CartCommand::Install(args) => install::execute(args),
    }
}
