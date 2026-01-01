//! Compile a replay script to binary format

use anyhow::{Context, Result};
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use nethercore_core::replay::{
    BinaryWriter, InputLayout, ReplayFlags, ReplayScript, StructuredInput,
};

/// Default input layout for ZX (used when no game is loaded)
struct ZxInputLayout;

impl InputLayout for ZxInputLayout {
    fn encode_input(&self, input: &StructuredInput) -> Vec<u8> {
        let mut buttons: u16 = 0;

        for button in &input.buttons {
            match button.to_lowercase().as_str() {
                "up" => buttons |= 0x0001,
                "down" => buttons |= 0x0002,
                "left" => buttons |= 0x0004,
                "right" => buttons |= 0x0008,
                "a" => buttons |= 0x0010,
                "b" => buttons |= 0x0020,
                "x" => buttons |= 0x0040,
                "y" => buttons |= 0x0080,
                "l" => buttons |= 0x0100,
                "r" => buttons |= 0x0200,
                "start" => buttons |= 0x0400,
                "select" => buttons |= 0x0800,
                _ => {}
            }
        }

        let mut bytes = vec![0u8; 8];

        // buttons: u16 (2 bytes)
        bytes[0] = (buttons & 0xFF) as u8;
        bytes[1] = ((buttons >> 8) & 0xFF) as u8;

        // left stick x: i8 (1 byte) - convert from -1.0..1.0 to -128..127
        if let Some([x, _]) = input.lstick {
            bytes[2] = (x.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }

        // left stick y: i8 (1 byte)
        if let Some([_, y]) = input.lstick {
            bytes[3] = (y.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }

        // right stick x: i8 (1 byte)
        if let Some([x, _]) = input.rstick {
            bytes[4] = (x.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }

        // right stick y: i8 (1 byte)
        if let Some([_, y]) = input.rstick {
            bytes[5] = (y.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }

        // left trigger: u8 (1 byte) - convert from 0.0..1.0 to 0..255
        if let Some(lt) = input.lt {
            bytes[6] = (lt.clamp(0.0, 1.0) * 255.0) as u8;
        }

        // right trigger: u8 (1 byte)
        if let Some(rt) = input.rt {
            bytes[7] = (rt.clamp(0.0, 1.0) * 255.0) as u8;
        }

        bytes
    }

    fn decode_input(&self, bytes: &[u8]) -> StructuredInput {
        let mut input = StructuredInput::default();

        if bytes.len() >= 2 {
            let buttons = u16::from_le_bytes([bytes[0], bytes[1]]);

            if buttons & 0x0001 != 0 {
                input.buttons.push("up".to_string());
            }
            if buttons & 0x0002 != 0 {
                input.buttons.push("down".to_string());
            }
            if buttons & 0x0004 != 0 {
                input.buttons.push("left".to_string());
            }
            if buttons & 0x0008 != 0 {
                input.buttons.push("right".to_string());
            }
            if buttons & 0x0010 != 0 {
                input.buttons.push("a".to_string());
            }
            if buttons & 0x0020 != 0 {
                input.buttons.push("b".to_string());
            }
            if buttons & 0x0040 != 0 {
                input.buttons.push("x".to_string());
            }
            if buttons & 0x0080 != 0 {
                input.buttons.push("y".to_string());
            }
            if buttons & 0x0100 != 0 {
                input.buttons.push("l".to_string());
            }
            if buttons & 0x0200 != 0 {
                input.buttons.push("r".to_string());
            }
            if buttons & 0x0400 != 0 {
                input.buttons.push("start".to_string());
            }
            if buttons & 0x0800 != 0 {
                input.buttons.push("select".to_string());
            }
        }

        if bytes.len() >= 4 {
            let lx = bytes[2] as i8 as f32 / 127.0;
            let ly = bytes[3] as i8 as f32 / 127.0;
            if lx.abs() > 0.01 || ly.abs() > 0.01 {
                input.lstick = Some([lx, ly]);
            }
        }

        if bytes.len() >= 6 {
            let rx = bytes[4] as i8 as f32 / 127.0;
            let ry = bytes[5] as i8 as f32 / 127.0;
            if rx.abs() > 0.01 || ry.abs() > 0.01 {
                input.rstick = Some([rx, ry]);
            }
        }

        if bytes.len() >= 7 && bytes[6] > 0 {
            input.lt = Some(bytes[6] as f32 / 255.0);
        }

        if bytes.len() >= 8 && bytes[7] > 0 {
            input.rt = Some(bytes[7] as f32 / 255.0);
        }

        input
    }

    fn input_size(&self) -> usize {
        8
    }

    fn console_id(&self) -> u8 {
        1 // ZX
    }

    fn button_names(&self) -> &[&str] {
        &[
            "up", "down", "left", "right", "a", "b", "x", "y", "l", "r", "start", "select",
        ]
    }
}

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
