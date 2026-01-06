//! Compile a replay script to binary format

use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use nethercore_core::replay::{BinaryWriter, ReplayFlags, ReplayScript};

use super::layout::ZxInputLayout;

/// Compile a script to binary format
pub fn execute(input: PathBuf, output: PathBuf) -> Result<()> {
    println!("Compiling: {} -> {}", input.display(), output.display());

    // Parse the script
    let script = ReplayScript::from_file(&input)
        .with_context(|| format!("Failed to parse script: {}", input.display()))?;

    // Use default ZX layout
    let layout = ZxInputLayout;

    // Compile the script
    let compiler = nethercore_core::replay::Compiler::new(&layout);
    let compiled = compiler
        .compile(&script)
        .with_context(|| "Failed to compile script")?;

    // Build replay structure
    let mut flags = ReplayFlags::COMPRESSED_INPUTS;
    if !compiled.assertions.is_empty() {
        flags |= ReplayFlags::HAS_ASSERTIONS;
    }

    let replay = nethercore_core::replay::Replay {
        header: nethercore_core::replay::ReplayHeader {
            console_id: compiled.console_id,
            player_count: compiled.player_count,
            input_size: compiled.input_size,
            flags,
            reserved: [0; 4],
            seed: compiled.seed,
            frame_count: compiled.frame_count,
        },
        inputs: compiled.inputs,
        checkpoints: Vec::new(),
        assertions: Vec::new(), // Assertions are stored separately in script format
    };

    // Write binary file
    let file = File::create(&output)
        .with_context(|| format!("Failed to create output file: {}", output.display()))?;
    let mut writer = BinaryWriter::new(BufWriter::new(file));
    writer
        .write_replay(&replay)
        .with_context(|| "Failed to write binary replay")?;

    println!();
    println!("=== Compilation Complete ===");
    println!("Frames: {}", replay.header.frame_count);
    println!("Players: {}", replay.header.player_count);
    println!("Seed: {}", replay.header.seed);

    Ok(())
}
