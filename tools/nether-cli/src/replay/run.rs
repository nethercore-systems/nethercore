//! Execute a replay script and generate a report

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

/// Execute a replay script
pub fn execute(
    script: PathBuf,
    rom: PathBuf,
    report: Option<PathBuf>,
    headless: bool,
    fail_fast: bool,
    timeout: u64,
) -> Result<()> {
    // Resolve against the caller before the cargo fallback changes cwd.
    let cwd = std::env::current_dir()?;
    let script = cwd.join(script);
    let rom = cwd.join(rom);
    let report = report.map(|path| cwd.join(path));
    let (player, workspace) = crate::run::find_nethercore_exe()?;
    let mut command = if player.to_string_lossy() == "cargo:run" {
        let mut command = Command::new("cargo");
        command.args(["run", "-p", "nethercore-zx", "--"]);
        if let Some(workspace) = workspace {
            command.current_dir(workspace);
        }
        command
    } else {
        Command::new(player)
    };
    command.arg(&rom).arg("--replay").arg(&script);
    if headless {
        command.arg("--headless");
        command.arg("--timeout").arg(timeout.to_string());
        if fail_fast {
            command.arg("--fail-fast");
        }
        if let Some(report) = report {
            command.arg("--report").arg(report);
        }
    } else {
        anyhow::ensure!(report.is_none(), "--report requires --headless");
        anyhow::ensure!(!fail_fast, "--fail-fast requires --headless");
    }

    let status = command.status().context("Failed to launch nethercore-zx")?;
    anyhow::ensure!(
        status.success(),
        "replay execution failed with status {status}"
    );
    Ok(())
}
