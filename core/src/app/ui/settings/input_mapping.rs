//! Input mapping types and helpers for keyboard control configuration

use winit::keyboard::KeyCode;

use crate::app::input::KeyboardMapping;

/// Input button types for keyboard mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InputButton {
    DPadUp,
    DPadDown,
    DPadLeft,
    DPadRight,
    ButtonA,
    ButtonB,
    ButtonX,
    ButtonY,
    LeftBumper,
    RightBumper,
    Start,
    Select,
}

/// Input axis types for keyboard mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum InputAxis {
    LeftStickUp,
    LeftStickDown,
    LeftStickLeft,
    LeftStickRight,
    RightStickUp,
    RightStickDown,
    RightStickLeft,
    RightStickRight,
    LeftTrigger,
    RightTrigger,
}

/// Represents what input we're waiting for during remapping
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WaitingFor {
    Button(InputButton),
    Axis(InputAxis),
}

impl InputButton {
    pub(super) fn name(&self) -> &'static str {
        match self {
            InputButton::DPadUp => "D-Pad Up",
            InputButton::DPadDown => "D-Pad Down",
            InputButton::DPadLeft => "D-Pad Left",
            InputButton::DPadRight => "D-Pad Right",
            InputButton::ButtonA => "Button A",
            InputButton::ButtonB => "Button B",
            InputButton::ButtonX => "Button X",
            InputButton::ButtonY => "Button Y",
            InputButton::LeftBumper => "Left Bumper",
            InputButton::RightBumper => "Right Bumper",
            InputButton::Start => "Start",
            InputButton::Select => "Select",
        }
    }

    pub(super) fn get_key(&self, mapping: &KeyboardMapping) -> KeyCode {
        match self {
            InputButton::DPadUp => mapping.dpad_up,
            InputButton::DPadDown => mapping.dpad_down,
            InputButton::DPadLeft => mapping.dpad_left,
            InputButton::DPadRight => mapping.dpad_right,
            InputButton::ButtonA => mapping.button_a,
            InputButton::ButtonB => mapping.button_b,
            InputButton::ButtonX => mapping.button_x,
            InputButton::ButtonY => mapping.button_y,
            InputButton::LeftBumper => mapping.left_bumper,
            InputButton::RightBumper => mapping.right_bumper,
            InputButton::Start => mapping.start,
            InputButton::Select => mapping.select,
        }
    }

    pub(super) fn set_key(&self, mapping: &mut KeyboardMapping, key: KeyCode) {
        match self {
            InputButton::DPadUp => mapping.dpad_up = key,
            InputButton::DPadDown => mapping.dpad_down = key,
            InputButton::DPadLeft => mapping.dpad_left = key,
            InputButton::DPadRight => mapping.dpad_right = key,
            InputButton::ButtonA => mapping.button_a = key,
            InputButton::ButtonB => mapping.button_b = key,
            InputButton::ButtonX => mapping.button_x = key,
            InputButton::ButtonY => mapping.button_y = key,
            InputButton::LeftBumper => mapping.left_bumper = key,
            InputButton::RightBumper => mapping.right_bumper = key,
            InputButton::Start => mapping.start = key,
            InputButton::Select => mapping.select = key,
        }
    }
}

impl InputAxis {
    pub(super) fn name(&self) -> &'static str {
        match self {
            InputAxis::LeftStickUp => "Up",
            InputAxis::LeftStickDown => "Down",
            InputAxis::LeftStickLeft => "Left",
            InputAxis::LeftStickRight => "Right",
            InputAxis::RightStickUp => "Up",
            InputAxis::RightStickDown => "Down",
            InputAxis::RightStickLeft => "Left",
            InputAxis::RightStickRight => "Right",
            InputAxis::LeftTrigger => "Left Trigger",
            InputAxis::RightTrigger => "Right Trigger",
        }
    }

    pub(super) fn get_key(&self, mapping: &KeyboardMapping) -> KeyCode {
        match self {
            InputAxis::LeftStickUp => mapping.left_stick_up,
            InputAxis::LeftStickDown => mapping.left_stick_down,
            InputAxis::LeftStickLeft => mapping.left_stick_left,
            InputAxis::LeftStickRight => mapping.left_stick_right,
            InputAxis::RightStickUp => mapping.right_stick_up,
            InputAxis::RightStickDown => mapping.right_stick_down,
            InputAxis::RightStickLeft => mapping.right_stick_left,
            InputAxis::RightStickRight => mapping.right_stick_right,
            InputAxis::LeftTrigger => mapping.left_trigger,
            InputAxis::RightTrigger => mapping.right_trigger,
        }
    }

    pub(super) fn set_key(&self, mapping: &mut KeyboardMapping, key: KeyCode) {
        match self {
            InputAxis::LeftStickUp => mapping.left_stick_up = key,
            InputAxis::LeftStickDown => mapping.left_stick_down = key,
            InputAxis::LeftStickLeft => mapping.left_stick_left = key,
            InputAxis::LeftStickRight => mapping.left_stick_right = key,
            InputAxis::RightStickUp => mapping.right_stick_up = key,
            InputAxis::RightStickDown => mapping.right_stick_down = key,
            InputAxis::RightStickLeft => mapping.right_stick_left = key,
            InputAxis::RightStickRight => mapping.right_stick_right = key,
            InputAxis::LeftTrigger => mapping.left_trigger = key,
            InputAxis::RightTrigger => mapping.right_trigger = key,
        }
    }
}
