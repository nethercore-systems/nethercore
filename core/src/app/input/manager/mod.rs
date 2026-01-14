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

    /// Input state for up to 4 players
    /// Player 0 = keyboard (if no gamepad for player 1)
    /// Players 1-3 = gamepads
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

        // Update keyboard input (player 0 if no gamepad is assigned to player 0)
        if !self.gamepad_to_player.values().any(|&slot| slot == 0) {
            self.player_inputs[0] = self.read_keyboard_input();
        }

        // Update gamepad inputs (if gamepad support is available)
        if let Some(ref gilrs) = self.gilrs {
            for (gamepad_id, &player_slot) in &self.gamepad_to_player {
                let gamepad = gilrs.gamepad(*gamepad_id);
                self.player_inputs[player_slot] = self.read_gamepad_input(&gamepad);
            }
        }
    }

    /// Poll events and update input state (keyboard only when gamepad feature is disabled)
    #[cfg(not(feature = "gamepad"))]
    pub fn update(&mut self) {
        // Keyboard input only when gamepad feature is disabled
        self.player_inputs[0] = self.read_keyboard_input();
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
    #[allow(dead_code)] // Public API for rollback netcode, not yet wired up
    pub fn get_all_inputs(&self) -> [RawInput; 4] {
        self.player_inputs
    }

    /// Update the input configuration (keyboard mappings, deadzones, etc.)
    pub fn update_config(&mut self, config: InputConfig) {
        self.config = config;
    }
}
