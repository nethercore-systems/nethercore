//! Execute a replay script and generate a report

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use nethercore_core::replay::{HeadlessConfig, HeadlessRunner};

use super::layout::ZxInputLayout;

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
    println!();

    // Create headless runner configuration
    let config = HeadlessConfig {
        fail_fast,
        timeout_secs: timeout,
        script_path: Some(script.display().to_string()),
    };

    // Create runner with ZX input layout
    let layout = ZxInputLayout;
    let mut runner = HeadlessRunner::from_file(&script, &layout, config)
        .with_context(|| "Failed to create headless runner")?;

    println!("=== Executing Replay ===");
    println!("Note: This is a simplified headless execution.");
    println!("      For full game execution with actual game logic,");
    println!("      the game WASM would need to be loaded and run.");
    println!();

    // Execute the replay
    let execution_report = runner.execute()
        .with_context(|| "Failed to execute replay")?;

    // Print summary
    println!("=== Execution Complete ===");
    println!("Frames executed: {}/{}", execution_report.frames_executed, execution_report.total_frames);
    println!("Snapshots captured: {}", execution_report.snapshots.len());
    println!("Assertions: {} passed, {} failed",
        execution_report.summary.assertions_passed,
        execution_report.summary.assertions_failed
    );
    println!("Status: {}", execution_report.summary.status);

    if let Some(duration) = execution_report.duration_ms {
        println!("Duration: {}ms", duration);
    }

    // Write report if requested
    if let Some(report_path) = report {
        let json = execution_report.to_json()
            .with_context(|| "Failed to serialize report to JSON")?;

        let mut file = File::create(&report_path)
            .with_context(|| format!("Failed to create report file: {}", report_path.display()))?;
        file.write_all(json.as_bytes())
            .with_context(|| "Failed to write report")?;

        println!();
        println!("Report written to: {}", report_path.display());
    }

    // Exit with appropriate code
    let exit_code = if execution_report.summary.assertions_failed > 0 {
        1
    } else {
        0
    };

    if exit_code != 0 {
        std::process::exit(exit_code);
    }

    Ok(())
}
