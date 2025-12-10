//! Input manager handling keyboard and gamepad

use crate::console::RawInput;
use gilrs::{Axis, Button, Gilrs};
use hashbrown::HashMap;
use winit::keyboard::KeyCode;

use super::InputConfig;

#[cfg(test)]
use super::keycode_serde::{keycode_to_string, string_to_keycode};
#[cfg(test)]
use super::KeyboardMapping;

pub struct InputManager {
    /// Gilrs context for gamepad handling (None if initialization failed)
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
    gamepad_to_player: HashMap<gilrs::GamepadId, usize>,
}

impl InputManager {
    /// Create a new input manager
    pub fn new(config: InputConfig) -> Self {
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
        // Collect gilrs events first (if gamepad support is available)
        let events: Vec<_> = if let Some(ref mut gilrs) = self.gilrs {
            std::iter::from_fn(|| gilrs.next_event())
                .map(|e| (e.id, e.event))
                .collect()
        } else {
            Vec::new()
        };

        // Process collected events
        for (id, event) in events {
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

        // Update gamepad inputs (if gamepad support is available)
        if let Some(ref gilrs) = self.gilrs {
            for (gamepad_id, &player_slot) in &self.gamepad_to_player {
                let gamepad = gilrs.gamepad(*gamepad_id);
                self.player_inputs[player_slot] = self.read_gamepad_input(&gamepad);
            }
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
    #[allow(dead_code)] // Public API for rollback netcode, not yet wired up
    pub fn get_all_inputs(&self) -> [RawInput; 4] {
        self.player_inputs
    }

    /// Find the next free player slot (0-3)
    fn find_free_player_slot(&self) -> Option<usize> {
        (0..4).find(|&slot| !self.gamepad_to_player.values().any(|&s| s == slot))
    }

    /// Read keyboard input and map to RawInput
    fn read_keyboard_input(&self) -> RawInput {
        let is_pressed =
            |key: KeyCode| -> bool { self.keyboard_state.get(&key).copied().unwrap_or(false) };

        let mapping = &self.config.keyboard;

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

            left_stick_button: false,
            right_stick_button: false,

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

    /// Read gamepad input and map to RawInput
    fn read_gamepad_input(&self, gamepad: &gilrs::Gamepad) -> RawInput {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keycode_to_string_letters() {
        assert_eq!(keycode_to_string(&KeyCode::KeyA), "A");
        assert_eq!(keycode_to_string(&KeyCode::KeyZ), "Z");
    }

    #[test]
    fn test_keycode_to_string_arrows() {
        assert_eq!(keycode_to_string(&KeyCode::ArrowUp), "ArrowUp");
        assert_eq!(keycode_to_string(&KeyCode::ArrowDown), "ArrowDown");
        assert_eq!(keycode_to_string(&KeyCode::ArrowLeft), "ArrowLeft");
        assert_eq!(keycode_to_string(&KeyCode::ArrowRight), "ArrowRight");
    }

    #[test]
    fn test_keycode_to_string_special() {
        assert_eq!(keycode_to_string(&KeyCode::Enter), "Enter");
        assert_eq!(keycode_to_string(&KeyCode::ShiftLeft), "ShiftLeft");
        assert_eq!(keycode_to_string(&KeyCode::Space), "Space");
    }

    #[test]
    fn test_string_to_keycode_letters() {
        assert_eq!(string_to_keycode("A"), Some(KeyCode::KeyA));
        assert_eq!(string_to_keycode("Z"), Some(KeyCode::KeyZ));
    }

    #[test]
    fn test_string_to_keycode_arrows() {
        assert_eq!(string_to_keycode("ArrowUp"), Some(KeyCode::ArrowUp));
        assert_eq!(string_to_keycode("ArrowDown"), Some(KeyCode::ArrowDown));
    }

    #[test]
    fn test_string_to_keycode_unknown() {
        assert_eq!(string_to_keycode("InvalidKey"), None);
        assert_eq!(string_to_keycode(""), None);
    }

    #[test]
    fn test_keyboard_mapping_roundtrip() {
        let mapping = KeyboardMapping::default();

        // Serialize to TOML
        let toml_str = toml::to_string(&mapping).expect("serialize");

        // Should contain human-readable key names
        assert!(toml_str.contains("ArrowUp"));
        assert!(toml_str.contains("ArrowDown"));
        assert!(toml_str.contains("Enter"));
        assert!(toml_str.contains("ShiftLeft"));

        // Deserialize back
        let mapping2: KeyboardMapping = toml::from_str(&toml_str).expect("deserialize");

        // Verify roundtrip
        assert_eq!(mapping.dpad_up, mapping2.dpad_up);
        assert_eq!(mapping.dpad_down, mapping2.dpad_down);
        assert_eq!(mapping.button_a, mapping2.button_a);
        assert_eq!(mapping.button_b, mapping2.button_b);
        assert_eq!(mapping.start, mapping2.start);
        assert_eq!(mapping.select, mapping2.select);
    }

    #[test]
    fn test_keyboard_mapping_custom_keys() {
        let toml_str = r#"
            dpad_up = "W"
            dpad_down = "S"
            dpad_left = "A"
            dpad_right = "D"
            button_a = "J"
            button_b = "K"
            button_x = "L"
            button_y = "I"
            left_bumper = "U"
            right_bumper = "O"
            start = "Enter"
            select = "Backspace"
        "#;

        let mapping: KeyboardMapping = toml::from_str(toml_str).expect("deserialize");

        assert_eq!(mapping.dpad_up, KeyCode::KeyW);
        assert_eq!(mapping.dpad_down, KeyCode::KeyS);
        assert_eq!(mapping.dpad_left, KeyCode::KeyA);
        assert_eq!(mapping.dpad_right, KeyCode::KeyD);
        assert_eq!(mapping.button_a, KeyCode::KeyJ);
        assert_eq!(mapping.button_b, KeyCode::KeyK);
        assert_eq!(mapping.button_x, KeyCode::KeyL);
        assert_eq!(mapping.button_y, KeyCode::KeyI);
        assert_eq!(mapping.select, KeyCode::Backspace);
    }

    #[test]
    fn test_input_config_roundtrip() {
        let config = InputConfig::default();

        // Serialize to TOML
        let toml_str = toml::to_string(&config).expect("serialize");

        // Should contain keyboard section with human-readable keys
        assert!(toml_str.contains("[keyboard]"));

        // Deserialize back
        let config2: InputConfig = toml::from_str(&toml_str).expect("deserialize");

        // Verify keyboard mapping preserved
        assert_eq!(config.keyboard.dpad_up, config2.keyboard.dpad_up);
        assert_eq!(config.keyboard.button_a, config2.keyboard.button_a);
        assert_eq!(config.stick_deadzone, config2.stick_deadzone);
        assert_eq!(config.trigger_deadzone, config2.trigger_deadzone);
    }

    #[test]
    fn test_deadzone_application() {
        let config = InputConfig {
            stick_deadzone: 0.2,
            trigger_deadzone: 0.1,
            ..Default::default()
        };

        // Create a minimal manager to test deadzone
        let manager = InputManager::new(config);

        // Values within deadzone should return 0
        assert_eq!(manager.apply_stick_deadzone(0.1), 0.0);
        assert_eq!(manager.apply_stick_deadzone(-0.1), 0.0);

        // Values at deadzone boundary
        assert_eq!(manager.apply_stick_deadzone(0.2), 0.0);

        // Values outside deadzone should be scaled
        let result = manager.apply_stick_deadzone(0.6);
        assert!(result > 0.0 && result <= 1.0);

        // Trigger deadzone
        assert_eq!(manager.apply_trigger_deadzone(0.05), 0.0);
        let trigger_result = manager.apply_trigger_deadzone(0.5);
        assert!(trigger_result > 0.0 && trigger_result <= 1.0);
    }

    // === Player Slot Assignment Tests ===
    //
    // Note: gilrs::GamepadId is opaque and cannot be constructed directly,
    // so we test the slot-finding logic by examining the occupied_slots behavior.

    /// Helper to test find_free_player_slot logic
    /// Since GamepadId is opaque, we extract the pure slot-finding logic for testing
    fn find_free_slot_from_occupied(occupied_slots: &[usize]) -> Option<usize> {
        (0..4).find(|slot| !occupied_slots.contains(slot))
    }

    #[test]
    fn test_find_free_player_slot_all_empty() {
        let manager = InputManager::new(InputConfig::default());
        // All slots should be free initially, first free is 0
        assert_eq!(manager.find_free_player_slot(), Some(0));
    }

    #[test]
    fn test_find_free_slot_logic_sequential() {
        // Test the slot-finding logic directly
        assert_eq!(find_free_slot_from_occupied(&[]), Some(0));
        assert_eq!(find_free_slot_from_occupied(&[0]), Some(1));
        assert_eq!(find_free_slot_from_occupied(&[0, 1]), Some(2));
        assert_eq!(find_free_slot_from_occupied(&[0, 1, 2]), Some(3));
    }

    #[test]
    fn test_find_free_slot_logic_all_full() {
        // All 4 slots occupied
        assert_eq!(find_free_slot_from_occupied(&[0, 1, 2, 3]), None);
    }

    #[test]
    fn test_find_free_slot_logic_gap_in_middle() {
        // Slots 0, 2, 3 occupied (skip 1)
        assert_eq!(find_free_slot_from_occupied(&[0, 2, 3]), Some(1));
    }

    #[test]
    fn test_find_free_slot_logic_disconnect_frees_slot() {
        // Initially 0 and 1 occupied
        assert_eq!(find_free_slot_from_occupied(&[0, 1]), Some(2));

        // After 0 disconnects, slot 0 is free again
        assert_eq!(find_free_slot_from_occupied(&[1]), Some(0));
    }

    #[test]
    fn test_find_free_slot_logic_out_of_order_assignment() {
        // Slots assigned out of order: 1, 3
        assert_eq!(find_free_slot_from_occupied(&[1, 3]), Some(0));

        // Add slot 0
        assert_eq!(find_free_slot_from_occupied(&[0, 1, 3]), Some(2));
    }

    // === Deadzone Edge Cases ===

    #[test]
    fn test_stick_deadzone_negative_values() {
        let config = InputConfig {
            stick_deadzone: 0.15,
            ..Default::default()
        };
        let manager = InputManager::new(config);

        // Negative values within deadzone
        assert_eq!(manager.apply_stick_deadzone(-0.1), 0.0);

        // Negative values outside deadzone should be scaled and negative
        let result = manager.apply_stick_deadzone(-0.5);
        assert!(result < 0.0, "Expected negative result, got {}", result);
        assert!(result >= -1.0, "Result should be >= -1.0, got {}", result);
    }

    #[test]
    fn test_stick_deadzone_max_value() {
        let config = InputConfig {
            stick_deadzone: 0.15,
            ..Default::default()
        };
        let manager = InputManager::new(config);

        // At max input (1.0), should get 1.0 output
        let result = manager.apply_stick_deadzone(1.0);
        assert!(
            (result - 1.0).abs() < 0.001,
            "Expected ~1.0, got {}",
            result
        );

        // At max negative input (-1.0), should get -1.0 output
        let result = manager.apply_stick_deadzone(-1.0);
        assert!(
            (result - (-1.0)).abs() < 0.001,
            "Expected ~-1.0, got {}",
            result
        );
    }

    #[test]
    fn test_stick_deadzone_zero_deadzone() {
        let config = InputConfig {
            stick_deadzone: 0.0,
            ..Default::default()
        };
        let manager = InputManager::new(config);

        // With 0 deadzone, small values should pass through
        assert_eq!(manager.apply_stick_deadzone(0.01), 0.01);
        assert_eq!(manager.apply_stick_deadzone(-0.01), -0.01);
    }

    #[test]
    fn test_trigger_deadzone_max_value() {
        let config = InputConfig {
            trigger_deadzone: 0.1,
            ..Default::default()
        };
        let manager = InputManager::new(config);

        // At max input (1.0), should get 1.0 output
        let result = manager.apply_trigger_deadzone(1.0);
        assert!(
            (result - 1.0).abs() < 0.001,
            "Expected ~1.0, got {}",
            result
        );
    }

    #[test]
    fn test_trigger_deadzone_at_boundary() {
        let config = InputConfig {
            trigger_deadzone: 0.1,
            ..Default::default()
        };
        let manager = InputManager::new(config);

        // At exactly the deadzone, should return 0
        assert_eq!(manager.apply_trigger_deadzone(0.1), 0.0);

        // Just above the deadzone should return a small positive value
        let result = manager.apply_trigger_deadzone(0.11);
        assert!(
            result > 0.0 && result < 0.1,
            "Expected small positive, got {}",
            result
        );
    }

    // === Get Player Input Tests ===

    #[test]
    fn test_get_player_input_valid_range() {
        let manager = InputManager::new(InputConfig::default());

        // Players 0-3 should return valid (default) inputs
        for i in 0..4 {
            let input = manager.get_player_input(i);
            // All buttons should be false by default
            assert!(!input.button_a);
            assert!(!input.dpad_up);
        }
    }

    #[test]
    fn test_get_player_input_out_of_range() {
        let manager = InputManager::new(InputConfig::default());

        // Players >= 4 should return default input
        let input = manager.get_player_input(4);
        assert!(!input.button_a);
        assert!(!input.dpad_up);

        // Large values should also be handled
        let input = manager.get_player_input(100);
        assert!(!input.button_a);
    }

    #[test]
    fn test_get_all_inputs_returns_four_players() {
        let manager = InputManager::new(InputConfig::default());
        let inputs = manager.get_all_inputs();
        assert_eq!(inputs.len(), 4);
    }

    // === Keyboard Input Tests ===

    #[test]
    fn test_keyboard_input_dpad() {
        let mut manager = InputManager::new(InputConfig::default());

        // Initially all buttons are not pressed
        let input = manager.read_keyboard_input();
        assert!(!input.dpad_up);
        assert!(!input.dpad_down);

        // Press up arrow
        manager.update_keyboard(KeyCode::ArrowUp, true);
        let input = manager.read_keyboard_input();
        assert!(input.dpad_up);
        assert!(!input.dpad_down);

        // Release up, press down
        manager.update_keyboard(KeyCode::ArrowUp, false);
        manager.update_keyboard(KeyCode::ArrowDown, true);
        let input = manager.read_keyboard_input();
        assert!(!input.dpad_up);
        assert!(input.dpad_down);
    }

    #[test]
    fn test_keyboard_input_face_buttons() {
        let mut manager = InputManager::new(InputConfig::default());

        // Press all face buttons
        manager.update_keyboard(KeyCode::KeyZ, true); // A
        manager.update_keyboard(KeyCode::KeyX, true); // B
        manager.update_keyboard(KeyCode::KeyC, true); // X
        manager.update_keyboard(KeyCode::KeyV, true); // Y

        let input = manager.read_keyboard_input();
        assert!(input.button_a);
        assert!(input.button_b);
        assert!(input.button_x);
        assert!(input.button_y);
    }

    #[test]
    fn test_keyboard_input_start_select() {
        let mut manager = InputManager::new(InputConfig::default());

        manager.update_keyboard(KeyCode::Enter, true);
        manager.update_keyboard(KeyCode::ShiftLeft, true);

        let input = manager.read_keyboard_input();
        assert!(input.start);
        assert!(input.select);
    }

    #[test]
    fn test_keyboard_analog_is_zero_when_no_keys_pressed() {
        let manager = InputManager::new(InputConfig::default());

        // When no axis keys are pressed, analog values should be zero
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_stick_x, 0.0);
        assert_eq!(input.left_stick_y, 0.0);
        assert_eq!(input.right_stick_x, 0.0);
        assert_eq!(input.right_stick_y, 0.0);
        assert_eq!(input.left_trigger, 0.0);
        assert_eq!(input.right_trigger, 0.0);
    }

    #[test]
    fn test_keyboard_axis_left_stick() {
        let mut manager = InputManager::new(InputConfig::default());

        // Press W for up - should give positive Y
        manager.update_keyboard(KeyCode::KeyW, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_stick_y, 1.0);
        assert_eq!(input.left_stick_x, 0.0);

        // Release W, press S for down - should give negative Y
        manager.update_keyboard(KeyCode::KeyW, false);
        manager.update_keyboard(KeyCode::KeyS, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_stick_y, -1.0);

        // Press both W and S - should cancel out to 0
        manager.update_keyboard(KeyCode::KeyW, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_stick_y, 0.0);

        // Test X axis with A/D
        manager.update_keyboard(KeyCode::KeyW, false);
        manager.update_keyboard(KeyCode::KeyS, false);
        manager.update_keyboard(KeyCode::KeyA, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_stick_x, -1.0);

        manager.update_keyboard(KeyCode::KeyA, false);
        manager.update_keyboard(KeyCode::KeyD, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_stick_x, 1.0);
    }

    #[test]
    fn test_keyboard_axis_right_stick() {
        let mut manager = InputManager::new(InputConfig::default());

        // Press I for up - should give positive Y
        manager.update_keyboard(KeyCode::KeyI, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.right_stick_y, 1.0);

        // Press J for left - should give negative X
        manager.update_keyboard(KeyCode::KeyJ, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.right_stick_x, -1.0);
        assert_eq!(input.right_stick_y, 1.0);
    }

    #[test]
    fn test_keyboard_triggers() {
        let mut manager = InputManager::new(InputConfig::default());

        // Press U for left trigger
        manager.update_keyboard(KeyCode::KeyU, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_trigger, 1.0);
        assert_eq!(input.right_trigger, 0.0);

        // Press O for right trigger
        manager.update_keyboard(KeyCode::KeyO, true);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_trigger, 1.0);
        assert_eq!(input.right_trigger, 1.0);

        // Release left trigger
        manager.update_keyboard(KeyCode::KeyU, false);
        let input = manager.read_keyboard_input();
        assert_eq!(input.left_trigger, 0.0);
        assert_eq!(input.right_trigger, 1.0);
    }

    #[test]
    fn test_keyboard_custom_mapping() {
        let custom_mapping = KeyboardMapping {
            dpad_up: KeyCode::KeyW,
            dpad_down: KeyCode::KeyS,
            dpad_left: KeyCode::KeyA,
            dpad_right: KeyCode::KeyD,
            button_a: KeyCode::KeyJ,
            button_b: KeyCode::KeyK,
            button_x: KeyCode::KeyL,
            button_y: KeyCode::KeyI,
            left_bumper: KeyCode::KeyU,
            right_bumper: KeyCode::KeyO,
            start: KeyCode::Space,
            select: KeyCode::Tab,
            // Use different keys for axis bindings in this custom mapping
            left_stick_up: KeyCode::Numpad8,
            left_stick_down: KeyCode::Numpad2,
            left_stick_left: KeyCode::Numpad4,
            left_stick_right: KeyCode::Numpad6,
            right_stick_up: KeyCode::ArrowUp,
            right_stick_down: KeyCode::ArrowDown,
            right_stick_left: KeyCode::ArrowLeft,
            right_stick_right: KeyCode::ArrowRight,
            left_trigger: KeyCode::KeyQ,
            right_trigger: KeyCode::KeyE,
        };

        let config = InputConfig {
            keyboard: custom_mapping,
            ..Default::default()
        };

        let mut manager = InputManager::new(config);

        // Default D-pad keys (arrow keys) should NOT work for D-pad
        manager.update_keyboard(KeyCode::ArrowUp, true);
        let input = manager.read_keyboard_input();
        assert!(!input.dpad_up);

        // Custom keys SHOULD work
        manager.update_keyboard(KeyCode::KeyW, true);
        let input = manager.read_keyboard_input();
        assert!(input.dpad_up);
    }

    // === InputConfig Tests ===

    #[test]
    fn test_input_config_default_values() {
        let config = InputConfig::default();

        assert!((config.stick_deadzone - 0.15).abs() < 0.001);
        assert!((config.trigger_deadzone - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_input_config_deserialize_partial() {
        // Should be able to deserialize a config with only some fields
        let toml_str = r#"
            stick_deadzone = 0.25
        "#;

        let config: InputConfig = toml::from_str(toml_str).expect("deserialize");

        // Specified value
        assert!((config.stick_deadzone - 0.25).abs() < 0.001);
        // Default values for unspecified
        assert!((config.trigger_deadzone - 0.1).abs() < 0.001);
    }

    #[test]
    fn test_keycode_roundtrip_all_supported() {
        // Test a representative sample of all key categories
        let keys_to_test = vec![
            // Letters
            KeyCode::KeyA,
            KeyCode::KeyZ,
            // Numbers
            KeyCode::Digit0,
            KeyCode::Digit9,
            // Arrows
            KeyCode::ArrowUp,
            KeyCode::ArrowDown,
            KeyCode::ArrowLeft,
            KeyCode::ArrowRight,
            // Function keys
            KeyCode::F1,
            KeyCode::F12,
            // Modifiers
            KeyCode::ShiftLeft,
            KeyCode::ShiftRight,
            KeyCode::ControlLeft,
            KeyCode::AltLeft,
            // Special
            KeyCode::Space,
            KeyCode::Enter,
            KeyCode::Escape,
            KeyCode::Tab,
            KeyCode::Backspace,
            // Punctuation
            KeyCode::Comma,
            KeyCode::Period,
            KeyCode::Slash,
            // Numpad
            KeyCode::Numpad0,
            KeyCode::NumpadAdd,
            KeyCode::NumpadEnter,
        ];

        for key in keys_to_test {
            let str_repr = keycode_to_string(&key);
            assert_ne!(
                str_repr, "Unknown",
                "Key {:?} should have a string representation",
                key
            );

            let parsed = string_to_keycode(str_repr);
            assert_eq!(
                parsed,
                Some(key),
                "Key {:?} -> '{}' should roundtrip",
                key,
                str_repr
            );
        }
    }
}
