//! Decompile a binary replay to script format

use anyhow::{Context, Result};
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use nethercore_core::replay::{BinaryReader, decompile};

use super::layout::ZxInputLayout;

/// Decompile a binary replay to script format
pub fn execute(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("Decompiling: {} -> {}", input.display(), output.display());

    // Read the binary file
    let file = File::open(&input)
        .with_context(|| format!("Failed to open input file: {}", input.display()))?;
    let mut reader = BinaryReader::new(BufReader::new(file));
    let replay = reader
        .read_replay()
        .with_context(|| "Failed to read binary replay")?;

    // Decompile to script
    let layout = ZxInputLayout;
    let script = decompile(&replay, &layout);

    // Write TOML output
    let toml_str = script
        .to_toml()
        .with_context(|| "Failed to serialize to TOML")?;

    let mut output_file = File::create(&output)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;
    output_file
        .write_all(toml_str.as_bytes())
        .with_context(|| "Failed to write output file")?;

    println!();
    println!("=== Decompilation Complete ===");
    println!("Console: {}", script.console);
    println!("Seed: {}", script.seed);
    println!("Players: {}", script.players);
    println!("Frames: {}", script.frames.len());

    Ok(())
}
