//! Deadzone application for analog inputs

use super::InputManager;

impl InputManager {
    /// Apply deadzone to analog stick input
    #[cfg_attr(not(feature = "gamepad"), allow(dead_code))]
    pub(super) fn apply_stick_deadzone(&self, value: f32) -> f32 {
        let deadzone = self.config.stick_deadzone;
        if value.abs() < deadzone {
            0.0
        } else {
            // Scale to full range after deadzone
            let sign = value.signum();
            let magnitude = (value.abs() - deadzone) / (1.0 - deadzone);
            sign * magnitude.clamp(0.0, 1.0)
        }
    }

    /// Apply deadzone to trigger input
    pub(super) fn apply_trigger_deadzone(&self, value: f32) -> f32 {
        let deadzone = self.config.trigger_deadzone;
        if value < deadzone {
            0.0
        } else {
            // Scale to full range after deadzone
            (value - deadzone) / (1.0 - deadzone)
        }
    }
}
