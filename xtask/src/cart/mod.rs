//! ROM cart management commands
//!
//! This module provides CLI commands for creating, inspecting, and managing
//! Emberware ROM files (.ewz, .ewc, etc.).

pub mod create_z;
pub mod info;

use anyhow::Result;
use clap::Subcommand;

/// Cart management subcommands
#[derive(Debug, Subcommand)]
pub enum CartCommand {
    /// Create an Emberware Z ROM (.ewz) from a WASM file
    #[command(name = "create-z")]
    CreateZ(create_z::CreateZArgs),

    /// Display ROM metadata and information
    Info(info::InfoArgs),
}

/// Execute a cart command
pub fn execute(cmd: CartCommand) -> Result<()> {
    match cmd {
        CartCommand::CreateZ(args) => create_z::execute(args),
        CartCommand::Info(args) => info::execute(args),
    }
}
