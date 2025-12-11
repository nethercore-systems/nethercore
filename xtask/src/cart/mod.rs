//! ROM cart management commands
//!
//! This module provides CLI commands for creating, inspecting, and managing
//! Emberware ROM files (.ewz, .ewc, etc.).

pub mod create_z;
pub mod info;
pub mod install;

use anyhow::Result;
use clap::Subcommand;

/// Cart management subcommands
#[derive(Debug, Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum CartCommand {
    /// Create an Emberware Z ROM (.ewz) from a WASM file
    #[command(name = "create-z")]
    CreateZ(create_z::CreateZArgs),

    /// Display ROM metadata and information
    Info(info::InfoArgs),

    /// Install a ROM file to the local game library
    Install(install::InstallArgs),
}

/// Execute a cart command
pub fn execute(cmd: CartCommand) -> Result<()> {
    match cmd {
        CartCommand::CreateZ(args) => create_z::execute(args),
        CartCommand::Info(args) => info::execute(args),
        CartCommand::Install(args) => install::execute(args),
    }
}
