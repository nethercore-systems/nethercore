//! Input handling for keyboard and gamepad

use emberware_core::console::RawInput;
use gilrs::{Axis, Button, Gilrs};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use winit::keyboard::KeyCode;

/// Input configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
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

fn default_deadzone() -> f32 { 0.15 }
fn default_trigger_deadzone() -> f32 { 0.1 }

impl Default for InputConfig {
    fn default() -> Self {
        Self {
            keyboard: KeyboardMapping::default(),
            stick_deadzone: 0.15,
            trigger_deadzone: 0.1,
        }
    }
}

/// Keyboard to virtual controller mapping
/// Note: We don't serialize the actual mapping yet, just use defaults.
/// TODO: Add proper serialization with string-based key names
#[derive(Debug, Clone)]
pub struct KeyboardMapping {
    pub dpad_up: KeyCode,
    pub dpad_down: KeyCode,
    pub dpad_left: KeyCode,
    pub dpad_right: KeyCode,

    pub button_a: KeyCode,
    pub button_b: KeyCode,
    pub button_x: KeyCode,
    pub button_y: KeyCode,

    pub left_bumper: KeyCode,
    pub right_bumper: KeyCode,

    pub start: KeyCode,
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

// Custom serialization for KeyboardMapping (for now, just skip it)
impl Serialize for KeyboardMapping {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Skip serialization for now - use defaults on load
        use serde::ser::SerializeStruct;
        let state = serializer.serialize_struct("KeyboardMapping", 0)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for KeyboardMapping {
    fn deserialize<D>(_deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Always use default mapping for now
        Ok(KeyboardMapping::default())
    }
}

/// Input manager handling keyboard and gamepad
pub struct InputManager {
    /// Gilrs context for gamepad handling
    gilrs: Gilrs,

    /// Current keyboard state (key -> pressed)
    keyboard_state: HashMap<KeyCode, bool>,

    /// Input configuration
    config: InputConfig,

    /// Input state for up to 4 players
    /// Player 0 = keyboard (if no gamepad for player 1)
    /// Players 1-3 = gamepads
    player_inputs: [RawInput; 4],

    /// Gamepad ID to player slot mapping
    gamepad_to_player: HashMap<gilrs::GamepadId, usize>,
}

impl InputManager {
    /// Create a new input manager
    pub fn new(config: InputConfig) -> Self {
        let gilrs = Gilrs::new().unwrap_or_else(|e| {
            tracing::warn!("Failed to initialize gamepad support: {}", e);
            // Create a dummy Gilrs context
            // This will work but won't detect any gamepads
            Gilrs::new().unwrap()
        });

        Self {
            gilrs,
            keyboard_state: HashMap::new(),
            config,
            player_inputs: [RawInput::default(); 4],
            gamepad_to_player: HashMap::new(),
        }
    }

    /// Update keyboard state
    pub fn update_keyboard(&mut self, key: KeyCode, pressed: bool) {
        self.keyboard_state.insert(key, pressed);
    }

    /// Poll gamepad events and update input state
    pub fn update(&mut self) {
        // Poll gilrs events
        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            match event {
                gilrs::EventType::Connected => {
                    // Assign to next available player slot
                    if let Some(slot) = self.find_free_player_slot() {
                        self.gamepad_to_player.insert(id, slot);
                        tracing::info!("Gamepad {} connected as player {}", id, slot);
                    } else {
                        tracing::warn!("Gamepad {} connected but no free player slots", id);
                    }
                }
                gilrs::EventType::Disconnected => {
                    if let Some(slot) = self.gamepad_to_player.remove(&id) {
                        tracing::info!("Gamepad {} (player {}) disconnected", id, slot);
                        self.player_inputs[slot] = RawInput::default();
                    }
                }
                _ => {}
            }
        }

        // Update keyboard input (player 0 if no gamepad is assigned to player 0)
        if !self.gamepad_to_player.values().any(|&slot| slot == 0) {
            self.player_inputs[0] = self.read_keyboard_input();
        }

        // Update gamepad inputs
        for (gamepad_id, &player_slot) in &self.gamepad_to_player {
            let gamepad = self.gilrs.gamepad(*gamepad_id);
            self.player_inputs[player_slot] = self.read_gamepad_input(&gamepad);
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

    /// Find the next free player slot (0-3)
    fn find_free_player_slot(&self) -> Option<usize> {
        for slot in 0..4 {
            if !self.gamepad_to_player.values().any(|&s| s == slot) {
                return Some(slot);
            }
        }
        None
    }

    /// Read keyboard input and map to RawInput
    fn read_keyboard_input(&self) -> RawInput {
        let is_pressed = |key: KeyCode| -> bool {
            self.keyboard_state.get(&key).copied().unwrap_or(false)
        };

        let mapping = &self.config.keyboard;

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

            left_stick_button: false,
            right_stick_button: false,

            start: is_pressed(mapping.start),
            select: is_pressed(mapping.select),

            // Keyboard has no analog input
            left_stick_x: 0.0,
            left_stick_y: 0.0,
            right_stick_x: 0.0,
            right_stick_y: 0.0,
            left_trigger: 0.0,
            right_trigger: 0.0,
        }
    }

    /// Read gamepad input and map to RawInput
    fn read_gamepad_input(&self, gamepad: &gilrs::Gamepad) -> RawInput {
        // Read buttons
        let btn = |button: Button| -> bool {
            gamepad.is_pressed(button)
        };

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

    /// Apply deadzone to analog stick input
    fn apply_stick_deadzone(&self, value: f32) -> f32 {
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
    fn apply_trigger_deadzone(&self, value: f32) -> f32 {
        let deadzone = self.config.trigger_deadzone;
        if value < deadzone {
            0.0
        } else {
            // Scale to full range after deadzone
            (value - deadzone) / (1.0 - deadzone)
        }
    }
}
