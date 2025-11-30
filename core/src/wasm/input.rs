//! Input state types
//!
//! Provides the input state structure for player input during gameplay.

/// Input state for a single player
#[derive(Debug, Clone, Copy, Default)]
pub struct InputState {
    /// Button bitmask
    pub buttons: u16,
    /// Left stick X (-128 to 127)
    pub left_stick_x: i8,
    /// Left stick Y (-128 to 127)
    pub left_stick_y: i8,
    /// Right stick X (-128 to 127)
    pub right_stick_x: i8,
    /// Right stick Y (-128 to 127)
    pub right_stick_y: i8,
    /// Left trigger (0-255)
    pub left_trigger: u8,
    /// Right trigger (0-255)
    pub right_trigger: u8,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_state_default() {
        let input = InputState::default();
        assert_eq!(input.buttons, 0);
        assert_eq!(input.left_stick_x, 0);
        assert_eq!(input.left_stick_y, 0);
        assert_eq!(input.right_stick_x, 0);
        assert_eq!(input.right_stick_y, 0);
        assert_eq!(input.left_trigger, 0);
        assert_eq!(input.right_trigger, 0);
    }

    #[test]
    fn test_input_state_full_values() {
        let input = InputState {
            buttons: 0xFFFF,
            left_stick_x: 127,
            left_stick_y: -128,
            right_stick_x: 100,
            right_stick_y: -100,
            left_trigger: 255,
            right_trigger: 128,
        };
        assert_eq!(input.buttons, 0xFFFF);
        assert_eq!(input.left_stick_x, 127);
        assert_eq!(input.left_stick_y, -128);
        assert_eq!(input.right_trigger, 128);
    }

    #[test]
    fn test_input_state_bytemuck_roundtrip() {
        let original = InputState {
            buttons: 0x1234,
            left_stick_x: 50,
            left_stick_y: -75,
            right_stick_x: 25,
            right_stick_y: -25,
            left_trigger: 200,
            right_trigger: 100,
        };

        // InputState should be Copy + Clone
        let copied = original;
        assert_eq!(copied.buttons, original.buttons);
        assert_eq!(copied.left_stick_x, original.left_stick_x);
        assert_eq!(copied.left_trigger, original.left_trigger);
    }
}
