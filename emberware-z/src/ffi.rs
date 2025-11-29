//! Emberware Z FFI host functions
//!
//! Console-specific FFI functions for the PS1/N64 aesthetic fantasy console.
//! These functions are registered with the WASM linker and called by games.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameState;

use crate::console::{RESOLUTIONS, TICK_RATES};

/// Register all Emberware Z FFI functions with the linker
pub fn register_z_ffi(linker: &mut Linker<GameState>) -> Result<()> {
    // Configuration functions (init-only)
    linker.func_wrap("env", "set_resolution", set_resolution)?;
    linker.func_wrap("env", "set_tick_rate", set_tick_rate)?;
    linker.func_wrap("env", "set_clear_color", set_clear_color)?;
    linker.func_wrap("env", "render_mode", render_mode)?;

    Ok(())
}

/// Set the render resolution
///
/// Valid indices: 0=360p, 1=540p (default), 2=720p, 3=1080p
///
/// Must be called during `init()`. Calls outside init are ignored with a warning.
fn set_resolution(mut caller: Caller<'_, GameState>, res: u32) {
    let state = caller.data_mut();

    // Check if we're in init phase
    if !state.in_init {
        warn!("set_resolution() called outside init() - ignored");
        return;
    }

    // Validate resolution index
    if res as usize >= RESOLUTIONS.len() {
        warn!(
            "set_resolution({}) invalid - must be 0-{}, using default",
            res,
            RESOLUTIONS.len() - 1
        );
        return;
    }

    state.init_config.resolution_index = res;
    state.init_config.modified = true;

    let (w, h) = RESOLUTIONS[res as usize];
    info!("Resolution set to {}x{} (index {})", w, h, res);
}

/// Set the tick rate (frames per second for update loop)
///
/// Valid indices: 0=24fps, 1=30fps, 2=60fps (default), 3=120fps
///
/// Must be called during `init()`. Calls outside init are ignored with a warning.
fn set_tick_rate(mut caller: Caller<'_, GameState>, rate: u32) {
    let state = caller.data_mut();

    // Check if we're in init phase
    if !state.in_init {
        warn!("set_tick_rate() called outside init() - ignored");
        return;
    }

    // Validate tick rate index
    if rate as usize >= TICK_RATES.len() {
        warn!(
            "set_tick_rate({}) invalid - must be 0-{}, using default",
            rate,
            TICK_RATES.len() - 1
        );
        return;
    }

    state.init_config.tick_rate_index = rate;
    state.init_config.modified = true;

    let fps = TICK_RATES[rate as usize];
    info!("Tick rate set to {} fps (index {})", fps, rate);
}

/// Set the clear/background color
///
/// Color format: 0xRRGGBBAA (red, green, blue, alpha)
/// Default: 0x000000FF (black, fully opaque)
///
/// Must be called during `init()`. Calls outside init are ignored with a warning.
fn set_clear_color(mut caller: Caller<'_, GameState>, color: u32) {
    let state = caller.data_mut();

    // Check if we're in init phase
    if !state.in_init {
        warn!("set_clear_color() called outside init() - ignored");
        return;
    }

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
}

/// Set the render mode
///
/// Valid modes:
/// - 0 = Unlit (no lighting, flat colors)
/// - 1 = Matcap (view-space normal mapped to matcap textures)
/// - 2 = PBR (physically-based rendering with up to 4 lights)
/// - 3 = Hybrid (PBR direct + matcap ambient)
///
/// Must be called during `init()`. Calls outside init are ignored with a warning.
fn render_mode(mut caller: Caller<'_, GameState>, mode: u32) {
    let state = caller.data_mut();

    // Check if we're in init phase
    if !state.in_init {
        warn!("render_mode() called outside init() - ignored");
        return;
    }

    // Validate mode
    if mode > 3 {
        warn!(
            "render_mode({}) invalid - must be 0-3, using default (0=Unlit)",
            mode
        );
        return;
    }

    state.init_config.render_mode = mode as u8;
    state.init_config.modified = true;

    let mode_name = match mode {
        0 => "Unlit",
        1 => "Matcap",
        2 => "PBR",
        3 => "Hybrid",
        _ => "Unknown",
    };
    info!("Render mode set to {} (mode {})", mode_name, mode);
}

#[cfg(test)]
mod tests {
    use emberware_core::wasm::GameState;

    #[test]
    fn test_init_config_defaults() {
        let state = GameState::new();
        assert_eq!(state.init_config.resolution_index, 1); // 540p
        assert_eq!(state.init_config.tick_rate_index, 2); // 60 fps
        assert_eq!(state.init_config.clear_color, 0x000000FF); // Black
        assert_eq!(state.init_config.render_mode, 0); // Unlit
        assert!(!state.init_config.modified);
    }
}
