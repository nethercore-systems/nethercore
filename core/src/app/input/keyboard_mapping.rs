//! Keyboard to virtual controller mapping

use serde::{Deserialize, Serialize};
use winit::keyboard::KeyCode;

use super::keycode_serde::{deserialize_keycode, serialize_keycode};

/// Keyboard to virtual controller mapping with string-based serialization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardMapping {
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub dpad_up: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub dpad_down: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub dpad_left: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub dpad_right: KeyCode,

    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub button_a: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub button_b: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub button_x: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub button_y: KeyCode,

    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub left_bumper: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub right_bumper: KeyCode,

    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub start: KeyCode,
    #[serde(
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub select: KeyCode,

    // Left stick axis keys
    #[serde(
        default = "default_left_stick_up",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub left_stick_up: KeyCode,
    #[serde(
        default = "default_left_stick_down",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub left_stick_down: KeyCode,
    #[serde(
        default = "default_left_stick_left",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub left_stick_left: KeyCode,
    #[serde(
        default = "default_left_stick_right",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub left_stick_right: KeyCode,

    // Right stick axis keys
    #[serde(
        default = "default_right_stick_up",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub right_stick_up: KeyCode,
    #[serde(
        default = "default_right_stick_down",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub right_stick_down: KeyCode,
    #[serde(
        default = "default_right_stick_left",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub right_stick_left: KeyCode,
    #[serde(
        default = "default_right_stick_right",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub right_stick_right: KeyCode,

    // Trigger keys
    #[serde(
        default = "default_left_trigger",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub left_trigger: KeyCode,
    #[serde(
        default = "default_right_trigger",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub right_trigger: KeyCode,

    // Stick button keys (L3/R3)
    #[serde(
        default = "default_left_stick_button",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub left_stick_button: KeyCode,
    #[serde(
        default = "default_right_stick_button",
        serialize_with = "serialize_keycode",
        deserialize_with = "deserialize_keycode"
    )]
    pub right_stick_button: KeyCode,
}

// Default functions for serde (enables backwards compatibility with old configs)
fn default_left_stick_up() -> KeyCode {
    KeyCode::KeyW
}
fn default_left_stick_down() -> KeyCode {
    KeyCode::KeyS
}
fn default_left_stick_left() -> KeyCode {
    KeyCode::KeyA
}
fn default_left_stick_right() -> KeyCode {
    KeyCode::KeyD
}
fn default_right_stick_up() -> KeyCode {
    KeyCode::KeyI
}
fn default_right_stick_down() -> KeyCode {
    KeyCode::KeyK
}
fn default_right_stick_left() -> KeyCode {
    KeyCode::KeyJ
}
fn default_right_stick_right() -> KeyCode {
    KeyCode::KeyL
}
fn default_left_trigger() -> KeyCode {
    KeyCode::KeyU
}
fn default_right_trigger() -> KeyCode {
    KeyCode::KeyO
}
fn default_left_stick_button() -> KeyCode {
    KeyCode::KeyR
}
fn default_right_stick_button() -> KeyCode {
    KeyCode::KeyY
}

impl Default for KeyboardMapping {
    fn default() -> Self {
        Self {
            // Arrow keys for D-pad
            dpad_up: KeyCode::ArrowUp,
            dpad_down: KeyCode::ArrowDown,
            dpad_left: KeyCode::ArrowLeft,
            dpad_right: KeyCode::ArrowRight,

            // ZXCV for face buttons (matches common emulator layouts)
            button_a: KeyCode::KeyZ,
            button_b: KeyCode::KeyX,
            button_x: KeyCode::KeyC,
            button_y: KeyCode::KeyV,

            // Q/E for bumpers
            left_bumper: KeyCode::KeyQ,
            right_bumper: KeyCode::KeyE,

            // Enter/Shift for Start/Select
            start: KeyCode::Enter,
            select: KeyCode::ShiftLeft,

            // WASD for left stick
            left_stick_up: KeyCode::KeyW,
            left_stick_down: KeyCode::KeyS,
            left_stick_left: KeyCode::KeyA,
            left_stick_right: KeyCode::KeyD,

            // IJKL for right stick
            right_stick_up: KeyCode::KeyI,
            right_stick_down: KeyCode::KeyK,
            right_stick_left: KeyCode::KeyJ,
            right_stick_right: KeyCode::KeyL,

            // U/O for triggers
            left_trigger: KeyCode::KeyU,
            right_trigger: KeyCode::KeyO,

            // R/Y for stick buttons (L3/R3)
            left_stick_button: KeyCode::KeyR,
            right_stick_button: KeyCode::KeyY,
        }
    }
}

impl KeyboardMapping {
    /// Returns all keys bound in this mapping (for conflict detection)
    pub fn all_keys(&self) -> Vec<KeyCode> {
        vec![
            self.dpad_up,
            self.dpad_down,
            self.dpad_left,
            self.dpad_right,
            self.button_a,
            self.button_b,
            self.button_x,
            self.button_y,
            self.left_bumper,
            self.right_bumper,
            self.start,
            self.select,
            self.left_stick_up,
            self.left_stick_down,
            self.left_stick_left,
            self.left_stick_right,
            self.right_stick_up,
            self.right_stick_down,
            self.right_stick_left,
            self.right_stick_right,
            self.left_trigger,
            self.right_trigger,
            self.left_stick_button,
            self.right_stick_button,
        ]
    }
}
