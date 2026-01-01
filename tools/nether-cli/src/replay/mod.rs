//! Replay CLI commands
//!
//! Commands for recording, playing, and executing replay scripts.

mod compile;
mod decompile;
mod run;
mod validate;

use anyhow::Result;
use clap::Subcommand;
use std::path::PathBuf;

/// Replay subcommands
#[derive(Subcommand)]
pub enum ReplayAction {
    /// Execute a replay script and generate a report
    Run {
        /// Script file (.ncrs)
        script: PathBuf,

        /// Output report file (JSON)
        #[arg(short, long)]
        report: Option<PathBuf>,

        /// Run without rendering
        #[arg(long)]
        headless: bool,

        /// Stop on first assertion failure
        #[arg(long)]
        fail_fast: bool,

        /// Maximum execution time in seconds
        #[arg(long, default_value = "300")]
        timeout: u64,
    },

    /// Compile script to binary
    Compile {
        /// Input script (.ncrs)
        input: PathBuf,

        /// Output binary (.ncrp)
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Decompile binary to script
    Decompile {
        /// Input binary (.ncrp)
        input: PathBuf,

        /// Output script (.ncrs)
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Validate a script without running
    Validate {
        /// Script file (.ncrs)
        script: PathBuf,
    },
}

/// Execute a replay action
pub fn execute(action: ReplayAction) -> Result<()> {
    match action {
        ReplayAction::Run {
            script,
            report,
            headless,
            fail_fast,
            timeout,
        } => run::execute(script, report, headless, fail_fast, timeout),

        ReplayAction::Compile { input, output } => compile::execute(input, output),

        ReplayAction::Decompile { input, output } => decompile::execute(input, output),

        ReplayAction::Validate { script } => validate::execute(script),
    }
}
