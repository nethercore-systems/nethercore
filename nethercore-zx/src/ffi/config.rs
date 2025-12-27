//! Configuration FFI functions (init-only, single-call)
//!
//! These functions configure the console during game initialization.
//!
//! **Init-only:** Calls outside of init() are ignored with a warning.
//! **Single-call:** Each function can only be called once during init().
//!                  Calling the same function twice traps with an error.

use anyhow::{Result, bail};
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use super::ZXGameContext;

use crate::console::TICK_RATES;

/// Register configuration FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "set_tick_rate", set_tick_rate)?;
    linker.func_wrap("env", "set_clear_color", set_clear_color)?;
    linker.func_wrap("env", "render_mode", render_mode)?;
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

    let fps = TICK_RATES[rate as usize];
    info!("Tick rate set to {} fps (index {})", fps, rate);
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

    let r = (color >> 24) & 0xFF;
    let g = (color >> 16) & 0xFF;
    let b = (color >> 8) & 0xFF;
    let a = color & 0xFF;
    info!(
        "Clear color set to rgba({}, {}, {}, {})",
        r,
        g,
        b,
        a as f32 / 255.0
    );
    Ok(())
}

/// Set the render mode
///
/// Valid modes:
/// - 0 = Lambert (simple Lambert diffuse shading with/without normals)
/// - 1 = Matcap (view-space normal mapped to matcap textures)
/// - 2 = PBR (physically-based rendering with up to 4 lights)
/// - 3 = Hybrid (PBR direct + matcap ambient)
///
/// **Init-only:** Must be called during `init()`. Calls outside init are ignored.
/// **Single-call:** Can only be called once. Second call traps with an error.
fn render_mode(mut caller: Caller<'_, ZXGameContext>, mode: u32) -> Result<()> {
    // Check if we're in init phase
    if !caller.data().game.in_init {
        warn!("render_mode() called outside init() - ignored");
        return Ok(());
    }

    let state = &mut caller.data_mut().ffi;

    // Check for duplicate call
    if state.init_config.render_mode_set {
        bail!(
            "render_mode() called twice - each config function can only be called once during init()"
        );
    }
    state.init_config.render_mode_set = true;

    // Validate mode
    if mode > 3 {
        bail!("render_mode({}) invalid - must be 0-3", mode);
    }

    state.init_config.render_mode = mode as u8;
    state.init_config.modified = true;

    let mode_name = match mode {
        0 => "Lambert",
        1 => "Matcap",
        2 => "PBR",
        3 => "Hybrid",
        _ => "Unknown",
    };
    info!("Render mode set to {} (mode {})", mode_name, mode);
    Ok(())
}
