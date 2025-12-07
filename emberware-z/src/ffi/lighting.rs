//! Lighting FFI functions (Mode 2 PBR)
//!
//! Functions for configuring directional lights in PBR mode.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::ZFFIState;

/// Register lighting FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "light_set", light_set)?;
    linker.func_wrap("env", "light_color", light_color)?;
    linker.func_wrap("env", "light_intensity", light_intensity)?;
    linker.func_wrap("env", "light_enable", light_enable)?;
    linker.func_wrap("env", "light_disable", light_disable)?;
    Ok(())
}

/// Set light parameters (position/direction)
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `x` — Direction X component (will be normalized)
/// * `y` — Direction Y component (will be normalized)
/// * `z` — Direction Z component (will be normalized)
///
/// This function sets the light direction and enables the light.
/// The direction vector will be automatically normalized by the graphics backend.
/// For Mode 2 (PBR), all lights are directional.
/// Use `light_color()` and `light_intensity()` to set color and brightness.
fn light_set(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    index: u32,
    x: f32,
    y: f32,
    z: f32,
) {
    // Validate index
    if index > 3 {
        warn!("light_set: invalid light index {} (must be 0-3)", index);
        return;
    }

    // Validate direction vector (warn if zero-length)
    let len_sq = x * x + y * y + z * z;
    let state = &mut caller.data_mut().console;

    if len_sq < 1e-10 {
        warn!("light_set: zero-length direction vector, using default (0, -1, 0)");

        // Extract current light state
        let light = &state.current_shading_state.lights[index as usize];
        let color = light.get_color();
        let intensity = light.get_intensity();

        // Update with default direction
        state.update_light(index as usize, [0.0, -1.0, 0.0], color, intensity, true);
        return;
    }

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let color = light.get_color();
    let intensity = light.get_intensity();

    // Update with new direction
    state.update_light(index as usize, [x, y, z], color, intensity, true);
}

/// Set light color
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `r` — Red component (0.0-1.0+, can be > 1.0 for HDR-like effects)
/// * `g` — Green component (0.0-1.0+)
/// * `b` — Blue component (0.0-1.0+)
///
/// Sets the color for a light.
/// Colors can exceed 1.0 for brighter lights (HDR-like effects).
/// Negative values are clamped to 0.0.
fn light_color(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    index: u32,
    r: f32,
    g: f32,
    b: f32,
) {
    // Validate index
    if index > 3 {
        warn!("light_color: invalid light index {} (must be 0-3)", index);
        return;
    }

    // Validate color values (allow > 1.0 for HDR effects, but clamp negative to 0.0)
    let r = if r < 0.0 {
        warn!("light_color: negative red value {}, clamping to 0.0", r);
        0.0
    } else {
        r
    };
    let g = if g < 0.0 {
        warn!("light_color: negative green value {}, clamping to 0.0", g);
        0.0
    } else {
        g
    };
    let b = if b < 0.0 {
        warn!("light_color: negative blue value {}, clamping to 0.0", b);
        0.0
    } else {
        b
    };

    let state = &mut caller.data_mut().console;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let direction = light.get_direction();
    let intensity = light.get_intensity();
    let enabled = light.is_enabled();

    // Update with new color
    state.update_light(index as usize, direction, [r, g, b], intensity, enabled);
}

/// Set light intensity multiplier
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `intensity` — Intensity multiplier (typically 0.0-10.0, but no upper limit)
///
/// Sets the intensity multiplier for a light. The final light contribution is color × intensity.
/// Negative values are clamped to 0.0.
fn light_intensity(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    index: u32,
    intensity: f32,
) {
    // Validate index
    if index > 3 {
        warn!(
            "light_intensity: invalid light index {} (must be 0-3)",
            index
        );
        return;
    }

    // Validate intensity (allow > 1.0, but clamp negative to 0.0)
    let intensity = if intensity < 0.0 {
        warn!(
            "light_intensity: negative intensity {}, clamping to 0.0",
            intensity
        );
        0.0
    } else {
        intensity
    };

    let state = &mut caller.data_mut().console;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let direction = light.get_direction();
    let color = light.get_color();

    // Setting non-zero intensity automatically enables the light
    let enabled = intensity > 0.0;

    // Update with new intensity
    state.update_light(index as usize, direction, color, intensity, enabled);
}

/// Enable a light
///
/// # Arguments
/// * `index` — Light index (0-3)
///
/// Enables a previously disabled light so it contributes to the scene.
/// The light will use its current direction, color, and intensity settings.
fn light_enable(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, index: u32) {
    // Validate index
    if index > 3 {
        warn!("light_enable: invalid light index {} (must be 0-3)", index);
        return;
    }

    let state = &mut caller.data_mut().console;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let direction = light.get_direction();
    let color = light.get_color();
    let mut intensity = light.get_intensity();

    // If intensity is 0, set to default so light is actually visible when enabled
    if intensity == 0.0 {
        intensity = 1.0;
    }

    // Enable light
    state.update_light(index as usize, direction, color, intensity, true);
}

/// Disable a light
///
/// # Arguments
/// * `index` — Light index (0-3)
///
/// Disables a light so it no longer contributes to the scene.
/// Useful for toggling lights on/off dynamically.
/// The light's direction, color, and intensity are preserved and can be re-enabled later.
fn light_disable(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, index: u32) {
    // Validate index
    if index > 3 {
        warn!("light_disable: invalid light index {} (must be 0-3)", index);
        return;
    }

    let state = &mut caller.data_mut().console;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let direction = light.get_direction();
    let color = light.get_color();
    let intensity = light.get_intensity();

    // Disable light
    state.update_light(index as usize, direction, color, intensity, false);
}
