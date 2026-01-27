//! Script decompiler
//!
//! Converts binary replays back to TOML script format.

use super::ast::{FrameEntry, InputValue, ReplayScript};
use super::compiler::InputLayout;
use crate::replay::types::Replay;

/// Decompile a binary replay to a script
pub fn decompile(replay: &Replay, layout: &dyn InputLayout) -> ReplayScript {
    let mut frames = Vec::new();

    // Convert each frame to a FrameEntry
    for (frame_idx, frame_inputs) in replay.inputs.iter().enumerate() {
        // Get player 1-4 inputs
        let p1 = frame_inputs
            .first()
            .map(|bytes| bytes_to_input(bytes, layout));
        let p2 = frame_inputs
            .get(1)
            .map(|bytes| bytes_to_input(bytes, layout));
        let p3 = frame_inputs
            .get(2)
            .map(|bytes| bytes_to_input(bytes, layout));
        let p4 = frame_inputs
            .get(3)
            .map(|bytes| bytes_to_input(bytes, layout));

        // Check if this frame has an assertion
        let assert = replay
            .assertions
            .iter()
            .find(|a| a.frame == frame_idx as u64)
            .map(format_assertion);

        frames.push(FrameEntry {
            f: frame_idx as u64,
            p1,
            p2,
            p3,
            p4,
            snap: false, // Binary format doesn't preserve snap flags
            screenshot: false, // Binary format doesn't preserve screenshot flags
            assert,
            action: None, // Binary format doesn't preserve actions
            action_params: None,
        });
    }

    // Optimize: remove trailing idle frames with no assertions
    while frames.len() > 1 {
        let last = frames.last().unwrap();
        if last.assert.is_none() && is_all_idle(&[&last.p1, &last.p2, &last.p3, &last.p4]) {
            frames.pop();
        } else {
            break;
        }
    }

    ReplayScript {
        console: console_name(replay.header.console_id),
        seed: replay.header.seed,
        players: replay.header.player_count,
        frames,
    }
}

/// Convert raw bytes to an InputValue
fn bytes_to_input(bytes: &[u8], layout: &dyn InputLayout) -> InputValue {
    let structured = layout.decode_input(bytes);

    // If only buttons and no analog, use symbolic format
    if structured.lstick.is_none()
        && structured.rstick.is_none()
        && structured.lt.is_none()
        && structured.rt.is_none()
    {
        if structured.buttons.is_empty() {
            InputValue::Symbolic("idle".to_string())
        } else {
            InputValue::Symbolic(structured.buttons.join("+"))
        }
    } else {
        // Use structured format for analog inputs
        InputValue::Structured(structured)
    }
}

/// Format an assertion for output
fn format_assertion(assertion: &crate::replay::types::Assertion) -> String {
    use crate::replay::types::AssertExpr;

    match &assertion.expression {
        AssertExpr::VarEq { name, value } => format!("{} == {}", name, value),
        AssertExpr::VarNe { name, value } => format!("{} != {}", name, value),
        AssertExpr::VarGt { name, value } => format!("{} > {}", name, value),
        AssertExpr::VarLt { name, value } => format!("{} < {}", name, value),
        AssertExpr::VarGe { name, value } => format!("{} >= {}", name, value),
        AssertExpr::VarLe { name, value } => format!("{} <= {}", name, value),
        AssertExpr::MemEq { addr, value } => format!("mem[0x{:x}] == {}", addr, value),
        AssertExpr::MemNe { addr, value } => format!("mem[0x{:x}] != {}", addr, value),
        AssertExpr::MemGt { addr, value } => format!("mem[0x{:x}] > {}", addr, value),
        AssertExpr::MemLt { addr, value } => format!("mem[0x{:x}] < {}", addr, value),
        AssertExpr::MemGe { addr, value } => format!("mem[0x{:x}] >= {}", addr, value),
        AssertExpr::MemLe { addr, value } => format!("mem[0x{:x}] <= {}", addr, value),
        AssertExpr::MemApprox {
            addr,
            value,
            tolerance,
        } => {
            format!("mem[0x{:x}] ~= {} ± {}", addr, value, tolerance)
        }
        AssertExpr::MemIncreased { addr } => format!("mem[0x{:x}] increased", addr),
        AssertExpr::MemDecreased { addr } => format!("mem[0x{:x}] decreased", addr),
        AssertExpr::TickGt { value } => format!("tick > {}", value),
    }
}

/// Check if all inputs are idle
fn is_all_idle(inputs: &[&Option<InputValue>]) -> bool {
    inputs
        .iter()
        .all(|input| input.as_ref().map(|v| v.is_idle()).unwrap_or(true))
}

/// Get console name from ID
fn console_name(id: u8) -> String {
    match id {
        1 => "zx".to_string(),
        _ => format!("console_{}", id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::script::StructuredInput;
    use crate::replay::types::{InputSequence, ReplayFlags, ReplayHeader};

    /// Mock input layout for testing
    struct MockLayout;

    impl InputLayout for MockLayout {
        fn encode_input(&self, input: &StructuredInput) -> Vec<u8> {
            let mut byte = 0u8;
            for button in &input.buttons {
                match button.as_str() {
                    "up" => byte |= 0x01,
                    "down" => byte |= 0x02,
                    "left" => byte |= 0x04,
                    "right" => byte |= 0x08,
                    "a" => byte |= 0x10,
                    "b" => byte |= 0x20,
                    _ => {}
                }
            }
            vec![byte]
        }

        fn decode_input(&self, bytes: &[u8]) -> StructuredInput {
            let byte = bytes.first().copied().unwrap_or(0);
            let mut buttons = Vec::new();
            if byte & 0x01 != 0 {
                buttons.push("up".to_string());
            }
            if byte & 0x02 != 0 {
                buttons.push("down".to_string());
            }
            if byte & 0x04 != 0 {
                buttons.push("left".to_string());
            }
            if byte & 0x08 != 0 {
                buttons.push("right".to_string());
            }
            if byte & 0x10 != 0 {
                buttons.push("a".to_string());
            }
            if byte & 0x20 != 0 {
                buttons.push("b".to_string());
            }
            StructuredInput {
                buttons,
                ..Default::default()
            }
        }

        fn input_size(&self) -> usize {
            1
        }

        fn console_id(&self) -> u8 {
            1
        }

        fn button_names(&self) -> &[&str] {
            &["up", "down", "left", "right", "a", "b"]
        }
    }

    #[test]
    fn test_decompile_basic() {
        let mut inputs = InputSequence::new();
        inputs.push_frame(vec![vec![0x00]]); // idle
        inputs.push_frame(vec![vec![0x18]]); // right+a
        inputs.push_frame(vec![vec![0x00]]); // idle

        let replay = Replay {
            header: ReplayHeader {
                console_id: 1,
                player_count: 1,
                input_size: 1,
                flags: ReplayFlags::empty(),
                reserved: [0; 4],
                seed: 12345,
                frame_count: 3,
            },
            inputs,
            checkpoints: Vec::new(),
            assertions: Vec::new(),
        };

        let layout = MockLayout;
        let script = decompile(&replay, &layout);

        assert_eq!(script.console, "zx");
        assert_eq!(script.seed, 12345);
        assert_eq!(script.players, 1);

        // Trailing idle should be trimmed
        assert_eq!(script.frames.len(), 2);

        // Check frame 0 is idle
        assert!(matches!(
            &script.frames[0].p1,
            Some(InputValue::Symbolic(s)) if s == "idle"
        ));

        // Check frame 1 is right+a
        assert!(matches!(
            &script.frames[1].p1,
            Some(InputValue::Symbolic(s)) if s.contains("right") && s.contains("a")
        ));
    }
}
