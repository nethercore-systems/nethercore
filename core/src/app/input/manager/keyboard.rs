//! Keyboard input handling

use crate::app::input::KeyboardMapping;
use crate::console::RawInput;
use winit::keyboard::KeyCode;

use super::InputManager;

impl InputManager {
    /// Read keyboard input for a specific player slot.
    /// Returns None if keyboard is disabled for that player.
    pub(super) fn read_keyboard_input_for_player(&self, player: usize) -> Option<RawInput> {
        let mapping = self.config.keyboards.get(player)?;
        Some(self.read_keyboard_with_mapping(mapping))
    }

    /// Read keyboard input using the given mapping
    fn read_keyboard_with_mapping(&self, mapping: &KeyboardMapping) -> RawInput {
        let is_pressed =
            |key: KeyCode| -> bool { self.keyboard_state.get(&key).copied().unwrap_or(false) };

        // Compute analog stick values from axis keys
        // Opposite keys cancel out (both pressed = 0)
        let left_stick_x = match (
            is_pressed(mapping.left_stick_left),
            is_pressed(mapping.left_stick_right),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let left_stick_y = match (
            is_pressed(mapping.left_stick_down),
            is_pressed(mapping.left_stick_up),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let right_stick_x = match (
            is_pressed(mapping.right_stick_left),
            is_pressed(mapping.right_stick_right),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };
        let right_stick_y = match (
            is_pressed(mapping.right_stick_down),
            is_pressed(mapping.right_stick_up),
        ) {
            (true, false) => -1.0,
            (false, true) => 1.0,
            _ => 0.0,
        };

        // Triggers are simple on/off (0.0 or 1.0)
        let left_trigger = if is_pressed(mapping.left_trigger) {
            1.0
        } else {
            0.0
        };
        let right_trigger = if is_pressed(mapping.right_trigger) {
            1.0
        } else {
            0.0
        };

        RawInput {
            dpad_up: is_pressed(mapping.dpad_up),
            dpad_down: is_pressed(mapping.dpad_down),
            dpad_left: is_pressed(mapping.dpad_left),
            dpad_right: is_pressed(mapping.dpad_right),

            button_a: is_pressed(mapping.button_a),
            button_b: is_pressed(mapping.button_b),
            button_x: is_pressed(mapping.button_x),
            button_y: is_pressed(mapping.button_y),

            left_bumper: is_pressed(mapping.left_bumper),
            right_bumper: is_pressed(mapping.right_bumper),

            left_stick_button: is_pressed(mapping.left_stick_button),
            right_stick_button: is_pressed(mapping.right_stick_button),

            start: is_pressed(mapping.start),
            select: is_pressed(mapping.select),

            left_stick_x,
            left_stick_y,
            right_stick_x,
            right_stick_y,
            left_trigger,
            right_trigger,
        }
    }
}
