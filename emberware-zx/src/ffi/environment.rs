//! Environment system FFI functions (Multi-Environment v3)
//!
//! Functions for setting procedural environment rendering parameters.
//! Environments support 8 modes with layering (base + overlay) and blend modes.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use crate::graphics::{blend_mode, env_mode};

use super::ZXGameContext;

/// Register environment system FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "env_gradient_set", env_gradient_set)?;
    linker.func_wrap("env", "env_select_pair", env_select_pair)?;
    linker.func_wrap("env", "env_blend_mode", env_blend_mode)?;
    Ok(())
}

/// Set gradient parameters for Mode 0 (Gradient)
///
/// # Arguments
/// * `zenith` — Color directly overhead (0xRRGGBBAA)
/// * `sky_horizon` — Sky color at horizon level (0xRRGGBBAA)
/// * `ground_horizon` — Ground color at horizon level (0xRRGGBBAA)
/// * `nadir` — Color directly below (0xRRGGBBAA)
/// * `rotation` — Rotation around Y axis in radians (for non-symmetric gradients)
/// * `shift` — Horizon vertical shift (-1.0 to 1.0, 0.0 = equator)
///
/// Sets the 4-color gradient for environment rendering. The gradient interpolates:
/// - zenith → sky_horizon (Y > 0)
/// - sky_horizon → ground_horizon (at Y = 0 + shift)
/// - ground_horizon → nadir (Y < 0)
///
/// This affects the BASE layer gradient. To set overlay gradient, call env_select_pair
/// first to configure overlay mode, then call this function again.
///
/// **Examples:**
/// - Blue sky: `env_gradient_set(0x191970FF, 0x87CEEBFF, 0x228B22FF, 0x2F4F4FFF, 0.0, 0.0)`
/// - Sunset: `env_gradient_set(0x4A00E0FF, 0xFF6B6BFF, 0x8B4513FF, 0x2F2F2FFF, 0.0, 0.1)`
fn env_gradient_set(
    mut caller: Caller<'_, ZXGameContext>,
    zenith: u32,
    sky_horizon: u32,
    ground_horizon: u32,
    nadir: u32,
    rotation: f32,
    shift: f32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack gradient into current environment state's base layer (offset 0)
    state.current_environment_state.pack_gradient(
        0, // Base layer offset
        zenith,
        sky_horizon,
        ground_horizon,
        nadir,
        rotation,
        shift,
    );

    // Ensure base mode is set to Gradient
    state
        .current_environment_state
        .set_base_mode(env_mode::GRADIENT);

    state.environment_dirty = true;
}

/// Select base and overlay modes for environment layering
///
/// # Arguments
/// * `base_mode` — Base layer mode (0-7)
/// * `overlay_mode` — Overlay layer mode (0-7)
///
/// # Mode Values
/// - 0: Gradient — 4-color procedural sky/ground gradient
/// - 1: Scatter — Random particles (stars, rain, hyperspace) [Phase 2]
/// - 2: Lines — Procedural line patterns (synthwave grid) [Phase 3]
/// - 3: Silhouette — Layered silhouettes (mountains, cityscape) [Phase 5]
/// - 4: Rectangles — Random rectangles (city windows) [Phase 7]
/// - 5: Room — Interior box environment [Phase 6]
/// - 6: Curtains — Vertical strips (forest, pillars) [Phase 8]
/// - 7: Rings — Concentric rings (portals, tunnels) [Phase 4]
///
/// When base_mode == overlay_mode, only the base layer is rendered (no layering).
/// Use env_blend_mode() to control how layers combine.
///
/// **Note:** Phase 1 only supports Mode 0 (Gradient). Other modes will display
/// the default gradient until implemented.
fn env_select_pair(
    mut caller: Caller<'_, ZXGameContext>,
    base_mode: u32,
    overlay_mode: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Validate modes
    if base_mode > 7 {
        warn!(
            "env_select_pair: invalid base_mode {} (must be 0-7), clamping",
            base_mode
        );
    }
    if overlay_mode > 7 {
        warn!(
            "env_select_pair: invalid overlay_mode {} (must be 0-7), clamping",
            overlay_mode
        );
    }

    state
        .current_environment_state
        .set_base_mode(base_mode.min(7));
    state
        .current_environment_state
        .set_overlay_mode(overlay_mode.min(7));

    state.environment_dirty = true;
}

/// Set the blend mode for combining base and overlay layers
///
/// # Arguments
/// * `mode` — Blend mode (0-3)
///
/// # Blend Modes
/// - 0: Alpha — Standard alpha blending: lerp(base, overlay, overlay.a)
/// - 1: Add — Additive blending: base + overlay
/// - 2: Multiply — Multiplicative: base * overlay
/// - 3: Screen — Screen blending: 1 - (1-base) * (1-overlay)
///
/// Only applies when base_mode != overlay_mode (layering enabled).
///
/// **Examples:**
/// - Alpha (default): Overlay covers base based on alpha
/// - Add: Bright overlays add to base (good for glow effects)
/// - Multiply: Dark overlays darken base (good for vignettes)
/// - Screen: Light overlays brighten base (good for fog/haze)
fn env_blend_mode(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
    let state = &mut caller.data_mut().ffi;

    // Validate mode
    if mode > 3 {
        warn!(
            "env_blend_mode: invalid mode {} (must be 0-3), clamping",
            mode
        );
    }

    state
        .current_environment_state
        .set_blend_mode(mode.min(3));

    state.environment_dirty = true;
}
