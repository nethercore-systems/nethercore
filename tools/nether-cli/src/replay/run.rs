//! Execute a replay script and generate a report

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use nethercore_core::replay::ReplayScript;

/// Execute a replay script
pub fn execute(
    script: PathBuf,
    report: Option<PathBuf>,
    headless: bool,
    fail_fast: bool,
    timeout: u64,
) -> Result<()> {
    println!("Executing script: {}", script.display());
    println!("  Headless: {}", headless);
    println!("  Fail-fast: {}", fail_fast);
    println!("  Timeout: {}s", timeout);

    // Parse the script
    let script_content = ReplayScript::from_file(&script)
        .with_context(|| format!("Failed to parse script: {}", script.display()))?;

    println!();
    println!("=== Script Loaded ===");
    println!("Console: {}", script_content.console);
    println!("Seed: {}", script_content.seed);
    println!("Players: {}", script_content.players);
    println!("Frames: {}", script_content.frames.len());

    // Count snap frames and assertions
    let snap_count = script_content.frames.iter().filter(|f| f.snap).count();
    let assert_count = script_content
        .frames
        .iter()
        .filter(|f| f.assert.is_some())
        .count();

    println!();
    println!("Snap frames: {}", snap_count);
    println!("Assertions: {}", assert_count);

    // TODO: Actual execution requires a loaded console instance
    // For now, we just validate the script and report what would happen
    println!();
    println!("=== Execution Simulation ===");
    println!(
        "Would execute {} frames with {} assertions",
        script_content.max_frame() + 1,
        assert_count
    );

    if headless {
        println!("Running in headless mode (no rendering)");
    }

    if fail_fast && assert_count > 0 {
        println!("Would stop on first assertion failure");
    }

    // Generate report if requested
    if let Some(report_path) = report {
        let report_json = serde_json::json!({
            "script": script.display().to_string(),
            "console": script_content.console,
            "seed": script_content.seed,
            "players": script_content.players,
            "frame_count": script_content.max_frame() + 1,
            "snap_count": snap_count,
            "assertion_count": assert_count,
            "status": "simulated",
            "note": "Full execution requires console integration"
        });

        let mut file = File::create(&report_path)
            .with_context(|| format!("Failed to create report file: {}", report_path.display()))?;
        file.write_all(serde_json::to_string_pretty(&report_json)?.as_bytes())
            .with_context(|| "Failed to write report")?;

        println!();
        println!("Report written to: {}", report_path.display());
    }

    println!();
    println!("=== Execution Complete (Simulated) ===");
    println!("Note: Full execution requires console integration.");
    println!("      Use 'nether run --replay <script>' with a loaded game.");

    Ok(())
}
