//! Input handling for keyboard and gamepad

mod keyboard_mapping;
pub(crate) mod keycode_serde; // Made pub(crate) for tests
mod manager;

pub use keyboard_mapping::KeyboardMapping;
pub use manager::InputManager;

use serde::{Deserialize, Serialize};

/// Keyboard mappings for all 4 player slots.
/// Each player can have their own keyboard mapping or be disabled (None).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyboardsConfig {
    /// Player 1 keyboard mapping (enabled by default)
    #[serde(
        default = "default_keyboard_p1",
        skip_serializing_if = "Option::is_none"
    )]
    pub p1: Option<KeyboardMapping>,
    /// Player 2 keyboard mapping (disabled by default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p2: Option<KeyboardMapping>,
    /// Player 3 keyboard mapping (disabled by default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p3: Option<KeyboardMapping>,
    /// Player 4 keyboard mapping (disabled by default)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub p4: Option<KeyboardMapping>,
}

fn default_keyboard_p1() -> Option<KeyboardMapping> {
    Some(KeyboardMapping::default())
}

impl Default for KeyboardsConfig {
    fn default() -> Self {
        Self {
            p1: default_keyboard_p1(),
            p2: None,
            p3: None,
            p4: None,
        }
    }
}

impl KeyboardsConfig {
    /// Get the keyboard mapping for a player (0-3)
    pub fn get(&self, player: usize) -> Option<&KeyboardMapping> {
        match player {
            0 => self.p1.as_ref(),
            1 => self.p2.as_ref(),
            2 => self.p3.as_ref(),
            3 => self.p4.as_ref(),
            _ => None,
        }
    }

    /// Get a mutable reference to the keyboard mapping for a player (0-3)
    pub fn get_mut(&mut self, player: usize) -> Option<&mut KeyboardMapping> {
        match player {
            0 => self.p1.as_mut(),
            1 => self.p2.as_mut(),
            2 => self.p3.as_mut(),
            3 => self.p4.as_mut(),
            _ => None,
        }
    }

    /// Set the keyboard mapping for a player (0-3)
    pub fn set(&mut self, player: usize, mapping: Option<KeyboardMapping>) {
        match player {
            0 => self.p1 = mapping,
            1 => self.p2 = mapping,
            2 => self.p3 = mapping,
            3 => self.p4 = mapping,
            _ => {}
        }
    }

    /// Check if a player has keyboard enabled
    pub fn is_enabled(&self, player: usize) -> bool {
        self.get(player).is_some()
    }

    /// Iterate over all (player_index, mapping) pairs for enabled players
    pub fn iter_enabled(&self) -> impl Iterator<Item = (usize, &KeyboardMapping)> {
        [(0, &self.p1), (1, &self.p2), (2, &self.p3), (3, &self.p4)]
            .into_iter()
            .filter_map(|(i, opt)| opt.as_ref().map(|m| (i, m)))
    }
}

/// Input configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InputConfig {
    /// Keyboard mappings per player slot (0-3).
    #[serde(default)]
    pub keyboards: KeyboardsConfig,

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
            keyboards: KeyboardsConfig::default(),
            stick_deadzone: default_deadzone(),
            trigger_deadzone: default_trigger_deadzone(),
        }
    }
}
