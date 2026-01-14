//! Tests for InputManager

use super::super::{InputConfig, KeyboardMapping};
use super::InputManager;
use crate::app::input::keycode_serde::{keycode_to_string, string_to_keycode};
use winit::keyboard::KeyCode;

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
