//! Replay system support for Nethercore ZX
//!
//! Provides the `ZxInputLayout` implementation of the `InputLayout` trait
//! for encoding/decoding ZX input in replay scripts.

use std::borrow::Cow;

use nethercore_core::replay::{InputLayout, StructuredInput};

use crate::console::{Button, ZInput};

/// Input layout for Nethercore ZX console
///
/// Handles encoding/decoding between structured input (symbolic button names,
/// analog values) and raw ZX input bytes.
///
/// # Input Format (8 bytes)
///
/// | Offset | Size | Field           | Description                    |
/// |--------|------|-----------------|--------------------------------|
/// | 0      | 2    | buttons         | u16 bitmask (little-endian)    |
/// | 2      | 1    | left_stick_x    | i8 (-128 to 127)               |
/// | 3      | 1    | left_stick_y    | i8 (-128 to 127)               |
/// | 4      | 1    | right_stick_x   | i8 (-128 to 127)               |
/// | 5      | 1    | right_stick_y   | i8 (-128 to 127)               |
/// | 6      | 1    | left_trigger    | u8 (0 to 255)                  |
/// | 7      | 1    | right_trigger   | u8 (0 to 255)                  |
///
/// # Button Mapping
///
/// | Bit | Button | Script Name |
/// |-----|--------|-------------|
/// | 0   | Up     | up          |
/// | 1   | Down   | down        |
/// | 2   | Left   | left        |
/// | 3   | Right  | right       |
/// | 4   | A      | a           |
/// | 5   | B      | b           |
/// | 6   | X      | x           |
/// | 7   | Y      | y           |
/// | 8   | LB     | l / lb      |
/// | 9   | RB     | r / rb      |
/// | 10  | L3     | l3          |
/// | 11  | R3     | r3          |
/// | 12  | Start  | start       |
/// | 13  | Select | select      |
#[derive(Debug, Clone, Copy, Default)]
pub struct ZxInputLayout;

impl ZxInputLayout {
    /// Create a new ZX input layout
    pub fn new() -> Self {
        Self
    }

    /// Convert a ZInput struct to raw bytes
    pub fn zinput_to_bytes(input: &ZInput) -> [u8; 8] {
        let mut bytes = [0u8; 8];
        bytes[0] = (input.buttons & 0xFF) as u8;
        bytes[1] = ((input.buttons >> 8) & 0xFF) as u8;
        bytes[2] = input.left_stick_x as u8;
        bytes[3] = input.left_stick_y as u8;
        bytes[4] = input.right_stick_x as u8;
        bytes[5] = input.right_stick_y as u8;
        bytes[6] = input.left_trigger;
        bytes[7] = input.right_trigger;
        bytes
    }

    /// Convert raw bytes to a ZInput struct
    pub fn bytes_to_zinput(bytes: &[u8]) -> ZInput {
        let mut input = ZInput::default();

        if bytes.len() >= 2 {
            input.buttons = u16::from_le_bytes([bytes[0], bytes[1]]);
        }
        if bytes.len() >= 4 {
            input.left_stick_x = bytes[2] as i8;
            input.left_stick_y = bytes[3] as i8;
        }
        if bytes.len() >= 6 {
            input.right_stick_x = bytes[4] as i8;
            input.right_stick_y = bytes[5] as i8;
        }
        if bytes.len() >= 7 {
            input.left_trigger = bytes[6];
        }
        if bytes.len() >= 8 {
            input.right_trigger = bytes[7];
        }

        input
    }
}

impl InputLayout for ZxInputLayout {
    fn encode_input(&self, input: &StructuredInput) -> Vec<u8> {
        let mut buttons: u16 = 0;

        for button in &input.buttons {
            match button.to_lowercase().as_str() {
                "up" => buttons |= Button::Up.mask(),
                "down" => buttons |= Button::Down.mask(),
                "left" => buttons |= Button::Left.mask(),
                "right" => buttons |= Button::Right.mask(),
                "a" => buttons |= Button::A.mask(),
                "b" => buttons |= Button::B.mask(),
                "x" => buttons |= Button::X.mask(),
                "y" => buttons |= Button::Y.mask(),
                "l" | "lb" | "l1" => buttons |= Button::LeftBumper.mask(),
                "r" | "rb" | "r1" => buttons |= Button::RightBumper.mask(),
                "l3" | "ls" => buttons |= Button::LeftStick.mask(),
                "r3" | "rs" => buttons |= Button::RightStick.mask(),
                "start" => buttons |= Button::Start.mask(),
                "select" | "back" => buttons |= Button::Select.mask(),
                _ => {}
            }
        }

        let mut bytes = vec![0u8; 8];
        bytes[0] = (buttons & 0xFF) as u8;
        bytes[1] = ((buttons >> 8) & 0xFF) as u8;

        // Left stick: convert -1.0..1.0 to -128..127
        if let Some([x, _]) = input.lstick {
            bytes[2] = (x.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }
        if let Some([_, y]) = input.lstick {
            bytes[3] = (y.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }

        // Right stick: convert -1.0..1.0 to -128..127
        if let Some([x, _]) = input.rstick {
            bytes[4] = (x.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }
        if let Some([_, y]) = input.rstick {
            bytes[5] = (y.clamp(-1.0, 1.0) * 127.0) as i8 as u8;
        }

        // Triggers: convert 0.0..1.0 to 0..255
        if let Some(lt) = input.lt {
            bytes[6] = (lt.clamp(0.0, 1.0) * 255.0) as u8;
        }
        if let Some(rt) = input.rt {
            bytes[7] = (rt.clamp(0.0, 1.0) * 255.0) as u8;
        }

        bytes
    }

    fn decode_input(&self, bytes: &[u8]) -> StructuredInput {
        let mut input = StructuredInput::default();

        if bytes.len() >= 2 {
            let buttons = u16::from_le_bytes([bytes[0], bytes[1]]);

            if buttons & Button::Up.mask() != 0 {
                input.buttons.push(Cow::Borrowed("up"));
            }
            if buttons & Button::Down.mask() != 0 {
                input.buttons.push(Cow::Borrowed("down"));
            }
            if buttons & Button::Left.mask() != 0 {
                input.buttons.push(Cow::Borrowed("left"));
            }
            if buttons & Button::Right.mask() != 0 {
                input.buttons.push(Cow::Borrowed("right"));
            }
            if buttons & Button::A.mask() != 0 {
                input.buttons.push(Cow::Borrowed("a"));
            }
            if buttons & Button::B.mask() != 0 {
                input.buttons.push(Cow::Borrowed("b"));
            }
            if buttons & Button::X.mask() != 0 {
                input.buttons.push(Cow::Borrowed("x"));
            }
            if buttons & Button::Y.mask() != 0 {
                input.buttons.push(Cow::Borrowed("y"));
            }
            if buttons & Button::LeftBumper.mask() != 0 {
                input.buttons.push(Cow::Borrowed("l"));
            }
            if buttons & Button::RightBumper.mask() != 0 {
                input.buttons.push(Cow::Borrowed("r"));
            }
            if buttons & Button::LeftStick.mask() != 0 {
                input.buttons.push(Cow::Borrowed("l3"));
            }
            if buttons & Button::RightStick.mask() != 0 {
                input.buttons.push(Cow::Borrowed("r3"));
            }
            if buttons & Button::Start.mask() != 0 {
                input.buttons.push(Cow::Borrowed("start"));
            }
            if buttons & Button::Select.mask() != 0 {
                input.buttons.push(Cow::Borrowed("select"));
            }
        }

        // Left stick: convert -128..127 to -1.0..1.0
        if bytes.len() >= 4 {
            let lx = bytes[2] as i8 as f32 / 127.0;
            let ly = bytes[3] as i8 as f32 / 127.0;
            if lx.abs() > 0.01 || ly.abs() > 0.01 {
                input.lstick = Some([lx, ly]);
            }
        }

        // Right stick: convert -128..127 to -1.0..1.0
        if bytes.len() >= 6 {
            let rx = bytes[4] as i8 as f32 / 127.0;
            let ry = bytes[5] as i8 as f32 / 127.0;
            if rx.abs() > 0.01 || ry.abs() > 0.01 {
                input.rstick = Some([rx, ry]);
            }
        }

        // Triggers: convert 0..255 to 0.0..1.0
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
        1 // ZX console ID
    }

    fn button_names(&self) -> &[&str] {
        &[
            "up", "down", "left", "right", "a", "b", "x", "y", "l", "r", "l3", "r3", "start",
            "select",
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_button_encoding() {
        let layout = ZxInputLayout;

        // Test single button
        let input = StructuredInput {
            buttons: vec![Cow::Borrowed("a")],
            ..Default::default()
        };
        let bytes = layout.encode_input(&input);
        assert_eq!(bytes[0] & 0x10, 0x10); // A button is bit 4

        // Test multiple buttons
        let input = StructuredInput {
            buttons: vec![Cow::Borrowed("up"), Cow::Borrowed("a")],
            ..Default::default()
        };
        let bytes = layout.encode_input(&input);
        assert_eq!(bytes[0] & 0x11, 0x11); // Up (bit 0) + A (bit 4)
    }

    #[test]
    fn test_button_decoding() {
        let layout = ZxInputLayout;

        // A button pressed
        let bytes = [0x10, 0x00, 0, 0, 0, 0, 0, 0];
        let input = layout.decode_input(&bytes);
        assert_eq!(input.buttons, vec![Cow::Borrowed("a")]);

        // Up + Start
        let bytes = [0x01, 0x10, 0, 0, 0, 0, 0, 0];
        let input = layout.decode_input(&bytes);
        assert!(input.buttons.contains(&Cow::Borrowed("up")));
        assert!(input.buttons.contains(&Cow::Borrowed("start")));
    }

    #[test]
    fn test_analog_roundtrip() {
        let layout = ZxInputLayout;

        let input = StructuredInput {
            buttons: Vec::new(),
            lstick: Some([0.5, -0.5]),
            rstick: Some([-1.0, 1.0]),
            lt: Some(0.75),
            rt: Some(0.25),
        };

        let bytes = layout.encode_input(&input);
        let decoded = layout.decode_input(&bytes);

        // Check lstick (with some tolerance for quantization)
        let lstick = decoded.lstick.unwrap();
        assert!((lstick[0] - 0.5).abs() < 0.02);
        assert!((lstick[1] - (-0.5)).abs() < 0.02);

        // Check rstick
        let rstick = decoded.rstick.unwrap();
        assert!((rstick[0] - (-1.0)).abs() < 0.02);
        assert!((rstick[1] - 1.0).abs() < 0.02);

        // Check triggers
        assert!((decoded.lt.unwrap() - 0.75).abs() < 0.01);
        assert!((decoded.rt.unwrap() - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_zinput_conversion() {
        let input = ZInput {
            buttons: 0x0011, // Up + A
            left_stick_x: 64,
            left_stick_y: -64,
            right_stick_x: 0,
            right_stick_y: 0,
            left_trigger: 128,
            right_trigger: 0,
        };

        let bytes = ZxInputLayout::zinput_to_bytes(&input);
        let decoded = ZxInputLayout::bytes_to_zinput(&bytes);

        assert_eq!(decoded.buttons, input.buttons);
        assert_eq!(decoded.left_stick_x, input.left_stick_x);
        assert_eq!(decoded.left_stick_y, input.left_stick_y);
        assert_eq!(decoded.left_trigger, input.left_trigger);
    }
}
