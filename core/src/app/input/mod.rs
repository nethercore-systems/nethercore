//! Input handling for keyboard and gamepad

mod keyboard_mapping;
mod keycode_serde;
mod manager;

pub use keyboard_mapping::KeyboardMapping;
pub use manager::InputManager;

use serde::{Deserialize, Serialize};

/// Input configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputConfig {
    /// Keyboard mapping for player 1
    #[serde(default)]
    pub keyboard: KeyboardMapping,

    /// Deadzone for analog sticks (0.0-1.0)
    #[serde(default = "default_deadzone")]
    pub stick_deadzone: f32,

    /// Deadzone for analog triggers (0.0-1.0)
    #[serde(default = "default_trigger_deadzone")]
    pub trigger_deadzone: f32,
}

fn default_deadzone() -> f32 {
    0.15
}
fn default_trigger_deadzone() -> f32 {
    0.1
}

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            keyboard: KeyboardMapping::default(),
            stick_deadzone: default_deadzone(),
            trigger_deadzone: default_trigger_deadzone(),
        }
    }
}
