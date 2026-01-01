//! Validate a replay script without running

use anyhow::{Context, Result};
use std::path::PathBuf;

use nethercore_core::replay::ReplayScript;

/// Validate a replay script
pub fn execute(script: PathBuf) -> Result<()> {
    println!("Validating script: {}", script.display());

    // Parse the script
    let script_content = ReplayScript::from_file(&script)
        .with_context(|| format!("Failed to parse script: {}", script.display()))?;

    // Report statistics
    println!();
    println!("=== Script Valid ===");
    println!("Console: {}", script_content.console);
    println!("Seed: {}", script_content.seed);
    println!("Players: {}", script_content.players);
    println!("Frames: {}", script_content.frames.len());
    println!(
        "Max frame: {}",
        script_content.frames.iter().map(|f| f.f).max().unwrap_or(0)
    );

    let snap_count = script_content.frames.iter().filter(|f| f.snap).count();
    let assert_count = script_content
        .frames
        .iter()
        .filter(|f| f.assert.is_some())
        .count();

    println!();
    println!("Snap frames: {}", snap_count);
    println!("Assertions: {}", assert_count);

    // Validate assertions parse correctly
    let mut errors = Vec::new();
    for frame in &script_content.frames {
        if let Some(ref assert_str) = frame.assert {
            if let Err(e) = nethercore_core::replay::AssertCondition::parse(assert_str) {
                errors.push(format!("Frame {}: {}", frame.f, e));
            }
        }
    }

    if errors.is_empty() {
        println!();
        println!("All assertions parse correctly.");
    } else {
        println!();
        println!("=== Assertion Parse Errors ===");
        for error in &errors {
            println!("  {}", error);
        }
        anyhow::bail!("{} assertion parse error(s)", errors.len());
    }

    Ok(())
}
