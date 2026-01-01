//! Execute a replay script and generate a report

use anyhow::{Context, Result};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;

use nethercore_core::replay::{HeadlessConfig, HeadlessRunner, InputLayout, StructuredInput};

/// ZX Input Layout (shared from compile.rs - TODO: deduplicate)
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

        // left stick x: i8 (1 byte)
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

        // left trigger: u8 (1 byte)
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
