//! Input manager handling keyboard and gamepad

mod deadzone;
mod keyboard;
#[cfg(feature = "gamepad")]
mod gamepad;
#[cfg(test)]
mod tests;

use crate::console::RawInput;
#[cfg(feature = "gamepad")]
use gilrs::Gilrs;
use hashbrown::HashMap;
use winit::keyboard::KeyCode;

use super::InputConfig;

pub struct InputManager {
    /// Gilrs context for gamepad handling (None if initialization failed or gamepad feature disabled)
    #[cfg(feature = "gamepad")]
    gilrs: Option<Gilrs>,

    /// Current keyboard state (key -> pressed)
    keyboard_state: HashMap<KeyCode, bool>,

    /// Input configuration
    config: InputConfig,

    /// Input state for up to 4 players.
    /// Each player can have keyboard enabled (via config.keyboards[i]).
    /// Gamepads are dynamically assigned to the first available slot.
    /// If both keyboard and gamepad are present for a slot, inputs are merged.
    player_inputs: [RawInput; 4],

    /// Gamepad ID to player slot mapping
    #[cfg(feature = "gamepad")]
    gamepad_to_player: HashMap<gilrs::GamepadId, usize>,
}

impl InputManager {
    /// Create a new input manager
    pub fn new(config: InputConfig) -> Self {
        #[cfg(feature = "gamepad")]
        let gilrs = match Gilrs::new() {
            Ok(g) => Some(g),
            Err(e) => {
                tracing::warn!(
                    "Failed to initialize gamepad support: {}. Gamepads will not be available.",
                    e
                );
                None
            }
        };

        Self {
            #[cfg(feature = "gamepad")]
            gilrs,
            keyboard_state: HashMap::new(),
            config,
            player_inputs: [RawInput::default(); 4],
            #[cfg(feature = "gamepad")]
            gamepad_to_player: HashMap::new(),
        }
    }

    /// Update keyboard state
    pub fn update_keyboard(&mut self, key: KeyCode, pressed: bool) {
        self.keyboard_state.insert(key, pressed);
    }

    /// Poll gamepad events and update input state
    #[cfg(feature = "gamepad")]
    pub fn update(&mut self) {
        // Process gilrs events directly without collecting into a Vec
        // This avoids per-frame heap allocation
        if let Some(ref mut gilrs) = self.gilrs {
            while let Some(event) = gilrs.next_event() {
                match event.event {
                    gilrs::EventType::Connected => {
                        // Find next free player slot (inlined to avoid borrow conflict)
                        let free_slot = (0..4)
                            .find(|&slot| !self.gamepad_to_player.values().any(|&s| s == slot));
                        if let Some(slot) = free_slot {
                            self.gamepad_to_player.insert(event.id, slot);
                            tracing::info!("Gamepad {} connected as player {}", event.id, slot);
                        } else {
                            tracing::warn!(
                                "Gamepad {} connected but no free player slots",
                                event.id
                            );
                        }
                    }
                    gilrs::EventType::Disconnected => {
                        if let Some(slot) = self.gamepad_to_player.remove(&event.id) {
                            tracing::info!("Gamepad {} (player {}) disconnected", event.id, slot);
                            self.player_inputs[slot] = RawInput::default();
                        }
                    }
                    _ => {}
                }
            }
        }

        // First, read gamepad inputs for assigned slots
        if let Some(ref gilrs) = self.gilrs {
            for (gamepad_id, &player_slot) in &self.gamepad_to_player {
                let gamepad = gilrs.gamepad(*gamepad_id);
                self.player_inputs[player_slot] = self.read_gamepad_input(&gamepad);
            }
        }

        // Then, process keyboard input for all players with keyboard enabled
        // Keyboard merges with gamepad if both are present for the same slot
        for player in 0..4 {
            if let Some(keyboard_input) = self.read_keyboard_input_for_player(player) {
                let has_gamepad = self.gamepad_to_player.values().any(|&slot| slot == player);
                if has_gamepad {
                    // Merge keyboard with existing gamepad input
                    self.player_inputs[player] =
                        merge_inputs(self.player_inputs[player], keyboard_input);
                } else {
                    // No gamepad, use keyboard directly
                    self.player_inputs[player] = keyboard_input;
                }
            }
            // If no keyboard mapping for this player and no gamepad, leave as default
        }
    }

    /// Poll events and update input state (keyboard only when gamepad feature is disabled)
    #[cfg(not(feature = "gamepad"))]
    pub fn update(&mut self) {
        // Process keyboard input for all players with keyboard enabled
        for player in 0..4 {
            if let Some(keyboard_input) = self.read_keyboard_input_for_player(player) {
                self.player_inputs[player] = keyboard_input;
            }
            // If no keyboard mapping for this player, leave as default
        }
    }

    /// Get input state for a specific player
    pub fn get_player_input(&self, player: usize) -> RawInput {
        if player < 4 {
            self.player_inputs[player]
        } else {
            RawInput::default()
        }
    }

    /// Get all player inputs
    pub fn get_all_inputs(&self) -> [RawInput; 4] {
        self.player_inputs
    }

    /// Update the input configuration (keyboard mappings, deadzones, etc.)
    pub fn update_config(&mut self, config: InputConfig) {
        self.config = config;
    }
}

/// Merge two RawInput sources (keyboard + gamepad for the same player).
/// Digital buttons: OR (either source can trigger)
/// Analog: use the value with the larger absolute magnitude
fn merge_inputs(a: RawInput, b: RawInput) -> RawInput {
    RawInput {
        // Digital buttons: OR
        dpad_up: a.dpad_up || b.dpad_up,
        dpad_down: a.dpad_down || b.dpad_down,
        dpad_left: a.dpad_left || b.dpad_left,
        dpad_right: a.dpad_right || b.dpad_right,
        button_a: a.button_a || b.button_a,
        button_b: a.button_b || b.button_b,
        button_x: a.button_x || b.button_x,
        button_y: a.button_y || b.button_y,
        left_bumper: a.left_bumper || b.left_bumper,
        right_bumper: a.right_bumper || b.right_bumper,
        left_stick_button: a.left_stick_button || b.left_stick_button,
        right_stick_button: a.right_stick_button || b.right_stick_button,
        start: a.start || b.start,
        select: a.select || b.select,

        // Analog sticks: use the one with larger absolute value
        left_stick_x: if a.left_stick_x.abs() > b.left_stick_x.abs() {
            a.left_stick_x
        } else {
            b.left_stick_x
        },
        left_stick_y: if a.left_stick_y.abs() > b.left_stick_y.abs() {
            a.left_stick_y
        } else {
            b.left_stick_y
        },
        right_stick_x: if a.right_stick_x.abs() > b.right_stick_x.abs() {
            a.right_stick_x
        } else {
            b.right_stick_x
        },
        right_stick_y: if a.right_stick_y.abs() > b.right_stick_y.abs() {
            a.right_stick_y
        } else {
            b.right_stick_y
        },

        // Triggers: use the larger value
        left_trigger: a.left_trigger.max(b.left_trigger),
        right_trigger: a.right_trigger.max(b.right_trigger),
    }
}
