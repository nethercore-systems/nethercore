//! Gamepad input handling

use crate::console::RawInput;
use gilrs::{Axis, Button};

use super::InputManager;

impl InputManager {
    /// Read gamepad input and map to RawInput
    pub(super) fn read_gamepad_input(&self, gamepad: &gilrs::Gamepad) -> RawInput {
        // Read buttons
        let btn = |button: Button| -> bool { gamepad.is_pressed(button) };

        // Read axes with deadzone
        let axis = |axis: Axis| -> f32 {
            let value = gamepad.value(axis);
            self.apply_stick_deadzone(value)
        };

        // Read trigger axes with deadzone
        let trigger = |axis: Axis| -> f32 {
            let value = gamepad.value(axis);
            // Triggers are typically 0.0 to 1.0, but some report -1.0 to 1.0
            let normalized = (value + 1.0) / 2.0; // Convert -1..1 to 0..1
            self.apply_trigger_deadzone(normalized).clamp(0.0, 1.0)
        };

        RawInput {
            // D-pad
            dpad_up: btn(Button::DPadUp),
            dpad_down: btn(Button::DPadDown),
            dpad_left: btn(Button::DPadLeft),
            dpad_right: btn(Button::DPadRight),

            // Face buttons (South=A, East=B, West=X, North=Y in Xbox layout)
            button_a: btn(Button::South),
            button_b: btn(Button::East),
            button_x: btn(Button::West),
            button_y: btn(Button::North),

            // Shoulder buttons
            left_bumper: btn(Button::LeftTrigger),
            right_bumper: btn(Button::RightTrigger),

            // Stick buttons
            left_stick_button: btn(Button::LeftThumb),
            right_stick_button: btn(Button::RightThumb),

            // Start/Select
            start: btn(Button::Start),
            select: btn(Button::Select),

            // Analog sticks
            left_stick_x: axis(Axis::LeftStickX),
            left_stick_y: -axis(Axis::LeftStickY), // Invert Y (up = positive)
            right_stick_x: axis(Axis::RightStickX),
            right_stick_y: -axis(Axis::RightStickY), // Invert Y

            // Analog triggers
            left_trigger: trigger(Axis::LeftZ),
            right_trigger: trigger(Axis::RightZ),
        }
    }
}
