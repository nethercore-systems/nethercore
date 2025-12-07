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
        }
    }
}
