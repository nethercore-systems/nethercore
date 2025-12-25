//! Human-readable display strings for keyboard keys

use winit::keyboard::KeyCode;

/// Convert KeyCode to a human-readable display string
pub fn keycode_to_display_string(key: KeyCode) -> String {
    match key {
        // Letters
        KeyCode::KeyA => "A",
        KeyCode::KeyB => "B",
        KeyCode::KeyC => "C",
        KeyCode::KeyD => "D",
        KeyCode::KeyE => "E",
        KeyCode::KeyF => "F",
        KeyCode::KeyG => "G",
        KeyCode::KeyH => "H",
        KeyCode::KeyI => "I",
        KeyCode::KeyJ => "J",
        KeyCode::KeyK => "K",
        KeyCode::KeyL => "L",
        KeyCode::KeyM => "M",
        KeyCode::KeyN => "N",
        KeyCode::KeyO => "O",
        KeyCode::KeyP => "P",
        KeyCode::KeyQ => "Q",
        KeyCode::KeyR => "R",
        KeyCode::KeyS => "S",
        KeyCode::KeyT => "T",
        KeyCode::KeyU => "U",
        KeyCode::KeyV => "V",
        KeyCode::KeyW => "W",
        KeyCode::KeyX => "X",
        KeyCode::KeyY => "Y",
        KeyCode::KeyZ => "Z",

        // Numbers
        KeyCode::Digit1 => "1",
        KeyCode::Digit2 => "2",
        KeyCode::Digit3 => "3",
        KeyCode::Digit4 => "4",
        KeyCode::Digit5 => "5",
        KeyCode::Digit6 => "6",
        KeyCode::Digit7 => "7",
        KeyCode::Digit8 => "8",
        KeyCode::Digit9 => "9",
        KeyCode::Digit0 => "0",

        // Function keys
        KeyCode::F1 => "F1",
        KeyCode::F2 => "F2",
        KeyCode::F3 => "F3",
        KeyCode::F4 => "F4",
        KeyCode::F5 => "F5",
        KeyCode::F6 => "F6",
        KeyCode::F7 => "F7",
        KeyCode::F8 => "F8",
        KeyCode::F9 => "F9",
        KeyCode::F10 => "F10",
        KeyCode::F11 => "F11",
        KeyCode::F12 => "F12",

        // Arrow keys
        KeyCode::ArrowUp => "Up",
        KeyCode::ArrowDown => "Down",
        KeyCode::ArrowLeft => "Left",
        KeyCode::ArrowRight => "Right",

        // Special keys
        KeyCode::Space => "Space",
        KeyCode::Enter => "Enter",
        KeyCode::Escape => "Esc",
        KeyCode::Backspace => "Backspace",
        KeyCode::Tab => "Tab",
        KeyCode::ShiftLeft | KeyCode::ShiftRight => "Shift",
        KeyCode::ControlLeft | KeyCode::ControlRight => "Ctrl",
        KeyCode::AltLeft | KeyCode::AltRight => "Alt",
        KeyCode::CapsLock => "Caps Lock",

        // Punctuation
        KeyCode::Comma => ",",
        KeyCode::Period => ".",
        KeyCode::Slash => "/",
        KeyCode::Semicolon => ";",
        KeyCode::Quote => "'",
        KeyCode::BracketLeft => "[",
        KeyCode::BracketRight => "]",
        KeyCode::Backslash => "\\",
        KeyCode::Minus => "-",
        KeyCode::Equal => "=",
        KeyCode::Backquote => "`",

        // Other
        _ => return format!("{:?}", key),
    }
    .to_string()
}
