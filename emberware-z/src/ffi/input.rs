//! Input FFI host functions for Emberware Z
//!
//! This module contains all input-related FFI functions:
//! - Button queries: held, pressed, released (individual and bulk)
//! - Analog stick queries: X/Y axes, bulk read
//! - Trigger queries: left and right analog triggers

use tracing::warn;
use wasmtime::Caller;

use super::ZGameContext;
use crate::console::{MAX_BUTTON_INDEX, STICK_SCALE, TRIGGER_SCALE};
use emberware_core::wasm::MAX_PLAYERS;

// ============================================================================
// Validation Helpers (reduce DRY violations)
// ============================================================================

/// Validate player index, returning Some(player_idx) if valid
#[inline]
fn validate_player(player: u32, func_name: &str) -> Option<usize> {
    let player_idx = player as usize;
    if player_idx >= MAX_PLAYERS {
        warn!(
            "{}: invalid player {} (max {})",
            func_name,
            player,
            MAX_PLAYERS - 1
        );
        None
    } else {
        Some(player_idx)
    }
}

/// Validate button index, returning true if valid
#[inline]
fn validate_button(button: u32, func_name: &str) -> bool {
    if button > MAX_BUTTON_INDEX {
        warn!(
            "{}: invalid button {} (max {})",
            func_name, button, MAX_BUTTON_INDEX
        );
        false
    } else {
        true
    }
}

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
#[inline]
pub fn button_held(caller: Caller<'_, ZGameContext>, player: u32, button: u32) -> u32 {
    let Some(player_idx) = validate_player(player, "button_held") else {
        return 0;
    };
    if !validate_button(button, "button_held") {
        return 0;
    }

    let state = &caller.data().game;
    let mask = 1u16 << button;
    u32::from((state.input_curr[player_idx].buttons & mask) != 0)
}

/// Check if a button was just pressed this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum)
///
/// Returns 1 if just pressed (not held last tick, held this tick), 0 otherwise.
#[inline]
pub fn button_pressed(caller: Caller<'_, ZGameContext>, player: u32, button: u32) -> u32 {
    let Some(player_idx) = validate_player(player, "button_pressed") else {
        return 0;
    };
    if !validate_button(button, "button_pressed") {
        return 0;
    }

    let state = &caller.data().game;
    let mask = 1u16 << button;
    let was_held = (state.input_prev[player_idx].buttons & mask) != 0;
    let is_held = (state.input_curr[player_idx].buttons & mask) != 0;

    u32::from(is_held && !was_held)
}

/// Check if a button was just released this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum)
///
/// Returns 1 if just released (held last tick, not held this tick), 0 otherwise.
#[inline]
pub fn button_released(caller: Caller<'_, ZGameContext>, player: u32, button: u32) -> u32 {
    let Some(player_idx) = validate_player(player, "button_released") else {
        return 0;
    };
    if !validate_button(button, "button_released") {
        return 0;
    }

    let state = &caller.data().game;
    let mask = 1u16 << button;
    let was_held = (state.input_prev[player_idx].buttons & mask) != 0;
    let is_held = (state.input_curr[player_idx].buttons & mask) != 0;

    u32::from(was_held && !is_held)
}

/// Get bitmask of all held buttons for a player
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask where each bit represents a button state.
#[inline]
pub fn buttons_held(caller: Caller<'_, ZGameContext>, player: u32) -> u32 {
    let Some(player_idx) = validate_player(player, "buttons_held") else {
        return 0;
    };
    caller.data().game.input_curr[player_idx].buttons as u32
}

/// Get bitmask of all buttons just pressed this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask of buttons that are held now but were not held last tick.
#[inline]
pub fn buttons_pressed(caller: Caller<'_, ZGameContext>, player: u32) -> u32 {
    let Some(player_idx) = validate_player(player, "buttons_pressed") else {
        return 0;
    };
    let state = &caller.data().game;
    let prev = state.input_prev[player_idx].buttons;
    let curr = state.input_curr[player_idx].buttons;

    // Pressed = held now AND not held before
    (curr & !prev) as u32
}

/// Get bitmask of all buttons just released this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask of buttons that were held last tick but are not held now.
#[inline]
pub fn buttons_released(caller: Caller<'_, ZGameContext>, player: u32) -> u32 {
    let Some(player_idx) = validate_player(player, "buttons_released") else {
        return 0;
    };
    let state = &caller.data().game;
    let prev = state.input_prev[player_idx].buttons;
    let curr = state.input_curr[player_idx].buttons;

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
#[inline]
pub fn left_stick_x(caller: Caller<'_, ZGameContext>, player: u32) -> f32 {
    let Some(player_idx) = validate_player(player, "left_stick_x") else {
        return 0.0;
    };
    caller.data().game.input_curr[player_idx].left_stick_x as f32 / STICK_SCALE
}

/// Get left stick Y axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
#[inline]
pub fn left_stick_y(caller: Caller<'_, ZGameContext>, player: u32) -> f32 {
    let Some(player_idx) = validate_player(player, "left_stick_y") else {
        return 0.0;
    };
    caller.data().game.input_curr[player_idx].left_stick_y as f32 / STICK_SCALE
}

/// Get right stick X axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
#[inline]
pub fn right_stick_x(caller: Caller<'_, ZGameContext>, player: u32) -> f32 {
    let Some(player_idx) = validate_player(player, "right_stick_x") else {
        return 0.0;
    };
    caller.data().game.input_curr[player_idx].right_stick_x as f32 / STICK_SCALE
}

/// Get right stick Y axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
#[inline]
pub fn right_stick_y(caller: Caller<'_, ZGameContext>, player: u32) -> f32 {
    let Some(player_idx) = validate_player(player, "right_stick_y") else {
        return 0.0;
    };
    caller.data().game.input_curr[player_idx].right_stick_y as f32 / STICK_SCALE
}

/// Get both left stick axes at once
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `out_x` — Pointer to write X axis value (-1.0 to 1.0)
/// * `out_y` — Pointer to write Y axis value (-1.0 to 1.0)
///
/// More efficient than two separate calls for the same player.
#[inline]
pub fn left_stick(mut caller: Caller<'_, ZGameContext>, player: u32, out_x: u32, out_y: u32) {
    let (x, y) = match validate_player(player, "left_stick") {
        Some(player_idx) => {
            let input = &caller.data().game.input_curr[player_idx];
            (
                input.left_stick_x as f32 / STICK_SCALE,
                input.left_stick_y as f32 / STICK_SCALE,
            )
        }
        None => (0.0f32, 0.0f32),
    };

    // Write results to WASM memory
    let memory = match caller.data().game.memory {
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
#[inline]
pub fn right_stick(mut caller: Caller<'_, ZGameContext>, player: u32, out_x: u32, out_y: u32) {
    let (x, y) = match validate_player(player, "right_stick") {
        Some(player_idx) => {
            let input = &caller.data().game.input_curr[player_idx];
            (
                input.right_stick_x as f32 / STICK_SCALE,
                input.right_stick_y as f32 / STICK_SCALE,
            )
        }
        None => (0.0f32, 0.0f32),
    };

    // Write results to WASM memory
    let memory = match caller.data().game.memory {
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
#[inline]
pub fn trigger_left(caller: Caller<'_, ZGameContext>, player: u32) -> f32 {
    let Some(player_idx) = validate_player(player, "trigger_left") else {
        return 0.0;
    };
    caller.data().game.input_curr[player_idx].left_trigger as f32 / TRIGGER_SCALE
}

/// Get right trigger value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from 0.0 to 1.0 (0.0 if invalid player).
#[inline]
pub fn trigger_right(caller: Caller<'_, ZGameContext>, player: u32) -> f32 {
    let Some(player_idx) = validate_player(player, "trigger_right") else {
        return 0.0;
    };
    caller.data().game.input_curr[player_idx].right_trigger as f32 / TRIGGER_SCALE
}

// ============================================================================
// Registration
// ============================================================================

use anyhow::Result;
use wasmtime::Linker;

/// Register input FFI functions with the linker
pub fn register(linker: &mut Linker<ZGameContext>) -> Result<()> {
    linker.func_wrap("env", "button_held", button_held)?;
    linker.func_wrap("env", "button_pressed", button_pressed)?;
    linker.func_wrap("env", "button_released", button_released)?;
    linker.func_wrap("env", "buttons_held", buttons_held)?;
    linker.func_wrap("env", "buttons_pressed", buttons_pressed)?;
    linker.func_wrap("env", "buttons_released", buttons_released)?;
    linker.func_wrap("env", "left_stick_x", left_stick_x)?;
    linker.func_wrap("env", "left_stick_y", left_stick_y)?;
    linker.func_wrap("env", "right_stick_x", right_stick_x)?;
    linker.func_wrap("env", "right_stick_y", right_stick_y)?;
    linker.func_wrap("env", "left_stick", left_stick)?;
    linker.func_wrap("env", "right_stick", right_stick)?;
    linker.func_wrap("env", "trigger_left", trigger_left)?;
    linker.func_wrap("env", "trigger_right", trigger_right)?;
    Ok(())
}
