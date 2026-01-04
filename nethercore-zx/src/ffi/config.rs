//! Configuration FFI functions (init-only, single-call)
//!
//! These functions configure the console during game initialization.
//!
//! **Init-only:** Calls outside of init() are ignored with a warning.
//! **Single-call:** Each function can only be called once during init().
//!                  Calling the same function twice traps with an error.

use anyhow::{Result, bail};
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;

use crate::console::TICK_RATES;

/// Register configuration FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "set_tick_rate", set_tick_rate)?;
    linker.func_wrap("env", "set_clear_color", set_clear_color)?;
    Ok(())
}

/// Set the tick rate (frames per second for update loop)
///
/// Valid indices: 0=24fps, 1=30fps, 2=60fps (default), 3=120fps
///
/// **Init-only:** Must be called during `init()`. Calls outside init are ignored.
/// **Single-call:** Can only be called once. Second call traps with an error.
fn set_tick_rate(mut caller: Caller<'_, ZXGameContext>, rate: u32) -> Result<()> {
    // Check if we're in init phase
    if !caller.data().game.in_init {
        warn!("set_tick_rate() called outside init() - ignored");
        return Ok(());
    }

    let state = &mut caller.data_mut().ffi;

    // Check for duplicate call
    if state.init_config.tick_rate_set {
        bail!(
            "set_tick_rate() called twice - each config function can only be called once during init()"
        );
    }
    state.init_config.tick_rate_set = true;

    // Validate tick rate index
    if rate as usize >= TICK_RATES.len() {
        bail!(
            "set_tick_rate({}) invalid - must be 0-{}",
            rate,
            TICK_RATES.len() - 1
        );
    }

    state.init_config.tick_rate_index = rate;
    state.init_config.modified = true;

    Ok(())
}

/// Set the clear/background color
///
/// Color format: 0xRRGGBBAA (red, green, blue, alpha)
/// Default: 0x000000FF (black, fully opaque)
///
/// **Init-only:** Must be called during `init()`. Calls outside init are ignored.
/// **Single-call:** Can only be called once. Second call traps with an error.
fn set_clear_color(mut caller: Caller<'_, ZXGameContext>, color: u32) -> Result<()> {
    // Check if we're in init phase
    if !caller.data().game.in_init {
        warn!("set_clear_color() called outside init() - ignored");
        return Ok(());
    }

    let state = &mut caller.data_mut().ffi;

    // Check for duplicate call
    if state.init_config.clear_color_set {
        bail!(
            "set_clear_color() called twice - each config function can only be called once during init()"
        );
    }
    state.init_config.clear_color_set = true;

    state.init_config.clear_color = color;
    state.init_config.modified = true;

    Ok(())
}
