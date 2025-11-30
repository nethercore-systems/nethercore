//! Input FFI host functions for Emberware Z
//!
//! This module contains all input-related FFI functions:
//! - Button queries: held, pressed, released (individual and bulk)
//! - Analog stick queries: X/Y axes, bulk read
//! - Trigger queries: left and right analog triggers

use tracing::warn;
use wasmtime::Caller;

use emberware_core::wasm::{GameState, MAX_PLAYERS};

// ============================================================================
// Button Functions
// ============================================================================

/// Check if a button is currently held for a player
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum: UP=0, DOWN=1, ..., SELECT=13)
///
/// Returns 1 if held, 0 otherwise.
pub fn button_held(caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "button_held: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0;
    }

    if button > 13 {
        warn!("button_held: invalid button {} (max 13)", button);
        return 0;
    }

    let mask = 1u16 << button;
    if (state.input_curr[player].buttons & mask) != 0 {
        1
    } else {
        0
    }
}

/// Check if a button was just pressed this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum)
///
/// Returns 1 if just pressed (not held last tick, held this tick), 0 otherwise.
pub fn button_pressed(caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "button_pressed: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0;
    }

    if button > 13 {
        warn!("button_pressed: invalid button {} (max 13)", button);
        return 0;
    }

    let mask = 1u16 << button;
    let was_held = (state.input_prev[player].buttons & mask) != 0;
    let is_held = (state.input_curr[player].buttons & mask) != 0;

    if is_held && !was_held {
        1
    } else {
        0
    }
}

/// Check if a button was just released this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum)
///
/// Returns 1 if just released (held last tick, not held this tick), 0 otherwise.
pub fn button_released(caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "button_released: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0;
    }

    if button > 13 {
        warn!("button_released: invalid button {} (max 13)", button);
        return 0;
    }

    let mask = 1u16 << button;
    let was_held = (state.input_prev[player].buttons & mask) != 0;
    let is_held = (state.input_curr[player].buttons & mask) != 0;

    if was_held && !is_held {
        1
    } else {
        0
    }
}

/// Get bitmask of all held buttons for a player
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask where each bit represents a button state.
pub fn buttons_held(caller: Caller<'_, GameState>, player: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "buttons_held: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0;
    }

    state.input_curr[player].buttons as u32
}

/// Get bitmask of all buttons just pressed this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask of buttons that are held now but were not held last tick.
pub fn buttons_pressed(caller: Caller<'_, GameState>, player: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "buttons_pressed: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0;
    }

    let prev = state.input_prev[player].buttons;
    let curr = state.input_curr[player].buttons;

    // Pressed = held now AND not held before
    (curr & !prev) as u32
}

/// Get bitmask of all buttons just released this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask of buttons that were held last tick but are not held now.
pub fn buttons_released(caller: Caller<'_, GameState>, player: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "buttons_released: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0;
    }

    let prev = state.input_prev[player].buttons;
    let curr = state.input_curr[player].buttons;

    // Released = held before AND not held now
    (prev & !curr) as u32
}

// ============================================================================
// Analog Stick Functions
// ============================================================================

/// Get left stick X axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
pub fn left_stick_x(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "left_stick_x: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0.0;
    }

    state.input_curr[player].left_stick_x as f32 / 127.0
}

/// Get left stick Y axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
pub fn left_stick_y(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "left_stick_y: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0.0;
    }

    state.input_curr[player].left_stick_y as f32 / 127.0
}

/// Get right stick X axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
pub fn right_stick_x(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "right_stick_x: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0.0;
    }

    state.input_curr[player].right_stick_x as f32 / 127.0
}

/// Get right stick Y axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
pub fn right_stick_y(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "right_stick_y: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0.0;
    }

    state.input_curr[player].right_stick_y as f32 / 127.0
}

/// Get both left stick axes at once
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `out_x` — Pointer to write X axis value (-1.0 to 1.0)
/// * `out_y` — Pointer to write Y axis value (-1.0 to 1.0)
///
/// More efficient than two separate calls for the same player.
pub fn left_stick(mut caller: Caller<'_, GameState>, player: u32, out_x: u32, out_y: u32) {
    let (x, y) = {
        let state = caller.data();
        let player = player as usize;

        if player >= MAX_PLAYERS {
            warn!(
                "left_stick: invalid player {} (max {})",
                player,
                MAX_PLAYERS - 1
            );
            (0.0f32, 0.0f32)
        } else {
            let input = &state.input_curr[player];
            (
                input.left_stick_x as f32 / 127.0,
                input.left_stick_y as f32 / 127.0,
            )
        }
    };

    // Write results to WASM memory
    let memory = match caller.data().memory {
        Some(m) => m,
        None => {
            warn!("left_stick: no WASM memory available");
            return;
        }
    };

    let mem_data = memory.data_mut(&mut caller);
    let x_ptr = out_x as usize;
    let y_ptr = out_y as usize;

    if x_ptr + 4 > mem_data.len() || y_ptr + 4 > mem_data.len() {
        warn!("left_stick: output pointers out of bounds");
        return;
    }

    mem_data[x_ptr..x_ptr + 4].copy_from_slice(&x.to_le_bytes());
    mem_data[y_ptr..y_ptr + 4].copy_from_slice(&y.to_le_bytes());
}

/// Get both right stick axes at once
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `out_x` — Pointer to write X axis value (-1.0 to 1.0)
/// * `out_y` — Pointer to write Y axis value (-1.0 to 1.0)
///
/// More efficient than two separate calls for the same player.
pub fn right_stick(mut caller: Caller<'_, GameState>, player: u32, out_x: u32, out_y: u32) {
    let (x, y) = {
        let state = caller.data();
        let player = player as usize;

        if player >= MAX_PLAYERS {
            warn!(
                "right_stick: invalid player {} (max {})",
                player,
                MAX_PLAYERS - 1
            );
            (0.0f32, 0.0f32)
        } else {
            let input = &state.input_curr[player];
            (
                input.right_stick_x as f32 / 127.0,
                input.right_stick_y as f32 / 127.0,
            )
        }
    };

    // Write results to WASM memory
    let memory = match caller.data().memory {
        Some(m) => m,
        None => {
            warn!("right_stick: no WASM memory available");
            return;
        }
    };

    let mem_data = memory.data_mut(&mut caller);
    let x_ptr = out_x as usize;
    let y_ptr = out_y as usize;

    if x_ptr + 4 > mem_data.len() || y_ptr + 4 > mem_data.len() {
        warn!("right_stick: output pointers out of bounds");
        return;
    }

    mem_data[x_ptr..x_ptr + 4].copy_from_slice(&x.to_le_bytes());
    mem_data[y_ptr..y_ptr + 4].copy_from_slice(&y.to_le_bytes());
}

// ============================================================================
// Trigger Functions
// ============================================================================

/// Get left trigger value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from 0.0 to 1.0 (0.0 if invalid player).
pub fn trigger_left(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "trigger_left: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0.0;
    }

    state.input_curr[player].left_trigger as f32 / 255.0
}

/// Get right trigger value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from 0.0 to 1.0 (0.0 if invalid player).
pub fn trigger_right(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!(
            "trigger_right: invalid player {} (max {})",
            player,
            MAX_PLAYERS - 1
        );
        return 0.0;
    }

    state.input_curr[player].right_trigger as f32 / 255.0
}
