//! CLI command definitions using clap

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "tracker-debug")]
#[command(about = "Debug tool for testing tracker music pipeline")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Play a tracker module file with debug display
    Play {
        /// Path to the XM or IT file
        file: PathBuf,

        /// Test via NCXM packed format (XM only)
        #[arg(long)]
        via_ncxm: bool,

        /// Test via NCIT packed format (IT only)
        #[arg(long)]
        via_ncit: bool,

        /// Show per-tick output instead of per-row
        #[arg(long, short)]
        verbose: bool,
    },
}
