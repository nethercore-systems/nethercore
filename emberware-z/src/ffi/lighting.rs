//! Lighting FFI functions (Mode 2 PBR)
//!
//! Functions for configuring directional and point lights in PBR mode.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZGameContext;
use crate::graphics::LightType;

/// Register lighting FFI functions
pub fn register(linker: &mut Linker<ZGameContext>) -> Result<()> {
    // Directional light functions
    linker.func_wrap("env", "light_set", light_set)?;
    linker.func_wrap("env", "light_color", light_color)?;
    linker.func_wrap("env", "light_intensity", light_intensity)?;
    linker.func_wrap("env", "light_enable", light_enable)?;
    linker.func_wrap("env", "light_disable", light_disable)?;

    // Point light functions
    linker.func_wrap("env", "light_set_point", light_set_point)?;
    linker.func_wrap("env", "light_range", light_range)?;
    Ok(())
}

/// Set light parameters (position/direction)
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `x` — Light ray direction X component (will be normalized)
/// * `y` — Light ray direction Y component (will be normalized)
/// * `z` — Light ray direction Z component (will be normalized)
///
/// **Direction convention:** The direction is where light rays travel (from light toward surface).
/// For a light from directly above, use `(0, -1, 0)` (rays going down).
/// This matches the convention used by `sky_set_sun()`.
///
/// This function sets the light direction and enables the light.
/// The direction vector will be automatically normalized by the graphics backend.
/// For Mode 2 (PBR), all lights are directional.
/// Use `light_color()` and `light_intensity()` to set color and brightness.
fn light_set(
    mut caller: Caller<'_, ZGameContext>,
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
    let state = &mut caller.data_mut().ffi;

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
/// * `color` — Light color (0xRRGGBBAA)
///
/// Sets the color for a light using packed u32 format.
/// The RGB values are converted to 0.0-1.0 range for lighting calculations.
/// Alpha channel is ignored for lights.
///
/// **Examples:**
/// - `0xFF0000FF` — Red light
/// - `0xFFFFFFFF` — White light
/// - `0xFFA500FF` — Orange light
fn light_color(
    mut caller: Caller<'_, ZGameContext>,
    index: u32,
    color: u32,
) {
    // Validate index
    if index > 3 {
        warn!("light_color: invalid light index {} (must be 0-3)", index);
        return;
    }

    // Unpack color from 0xRRGGBBAA to 0.0-1.0 range
    let [r, g, b] = super::unpack_rgb(color);

    let state = &mut caller.data_mut().ffi;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let light_type = light.get_type();
    let intensity = light.get_intensity();
    let enabled = light.is_enabled();

    // Update with new color (preserve type)
    if light_type == LightType::Point {
        let position = light.get_position();
        let range = light.get_range();
        state.update_point_light(
            index as usize,
            position,
            [r, g, b],
            intensity,
            range,
            enabled,
        );
    } else {
        let direction = light.get_direction();
        state.update_light(index as usize, direction, [r, g, b], intensity, enabled);
    }
}

/// Set light intensity multiplier
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `intensity` — Intensity multiplier (typically 0.0-8.0, clamped to 8.0 max)
///
/// Sets the intensity multiplier for a light. The final light contribution is color × intensity.
/// Negative values are clamped to 0.0, values above 8.0 are clamped to 8.0.
fn light_intensity(
    mut caller: Caller<'_, ZGameContext>,
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

    // Validate intensity (allow 0.0-8.0 range)
    let intensity = if intensity < 0.0 {
        warn!(
            "light_intensity: negative intensity {}, clamping to 0.0",
            intensity
        );
        0.0
    } else if intensity > 8.0 {
        warn!(
            "light_intensity: intensity {} exceeds max 8.0, clamping",
            intensity
        );
        8.0
    } else {
        intensity
    };

    let state = &mut caller.data_mut().ffi;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let light_type = light.get_type();
    let color = light.get_color();

    // Setting non-zero intensity automatically enables the light
    let enabled = intensity > 0.0;

    // Update with new intensity (preserve type)
    if light_type == LightType::Point {
        let position = light.get_position();
        let range = light.get_range();
        state.update_point_light(index as usize, position, color, intensity, range, enabled);
    } else {
        let direction = light.get_direction();
        state.update_light(index as usize, direction, color, intensity, enabled);
    }
}

/// Enable a light
///
/// # Arguments
/// * `index` — Light index (0-3)
///
/// Enables a previously disabled light so it contributes to the scene.
/// The light will use its current direction, color, and intensity settings.
fn light_enable(mut caller: Caller<'_, ZGameContext>, index: u32) {
    // Validate index
    if index > 3 {
        warn!("light_enable: invalid light index {} (must be 0-3)", index);
        return;
    }

    let state = &mut caller.data_mut().ffi;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let light_type = light.get_type();
    let color = light.get_color();
    let mut intensity = light.get_intensity();

    // If intensity is 0, set to default so light is actually visible when enabled
    if intensity == 0.0 {
        intensity = 1.0;
    }

    // Enable light (preserve type)
    if light_type == LightType::Point {
        let position = light.get_position();
        let range = light.get_range();
        state.update_point_light(index as usize, position, color, intensity, range, true);
    } else {
        let direction = light.get_direction();
        state.update_light(index as usize, direction, color, intensity, true);
    }
}

/// Disable a light
///
/// # Arguments
/// * `index` — Light index (0-3)
///
/// Disables a light so it no longer contributes to the scene.
/// Useful for toggling lights on/off dynamically.
/// The light's direction, color, and intensity are preserved and can be re-enabled later.
fn light_disable(mut caller: Caller<'_, ZGameContext>, index: u32) {
    // Validate index
    if index > 3 {
        warn!("light_disable: invalid light index {} (must be 0-3)", index);
        return;
    }

    let state = &mut caller.data_mut().ffi;

    // Extract current light state
    let light = &state.current_shading_state.lights[index as usize];
    let light_type = light.get_type();
    let color = light.get_color();
    let intensity = light.get_intensity();

    // Disable light (preserve type)
    if light_type == LightType::Point {
        let position = light.get_position();
        let range = light.get_range();
        state.update_point_light(index as usize, position, color, intensity, range, false);
    } else {
        let direction = light.get_direction();
        state.update_light(index as usize, direction, color, intensity, false);
    }
}

/// Set light as point light with position
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `x` — World-space X position
/// * `y` — World-space Y position
/// * `z` — World-space Z position
///
/// Converts the light to a point light and sets its position.
/// Use `light_range()` to set the falloff distance.
/// Use `light_color()` and `light_intensity()` for color/brightness.
fn light_set_point(
    mut caller: Caller<'_, ZGameContext>,
    index: u32,
    x: f32,
    y: f32,
    z: f32,
) {
    if index > 3 {
        warn!(
            "light_set_point: invalid light index {} (must be 0-3)",
            index
        );
        return;
    }

    let state = &mut caller.data_mut().ffi;
    let light = &state.current_shading_state.lights[index as usize];
    let color = light.get_color();
    let intensity = light.get_intensity();
    let range = if light.get_type() == LightType::Point {
        light.get_range()
    } else {
        10.0 // Default range for new point lights
    };

    state.update_point_light(index as usize, [x, y, z], color, intensity, range, true);
}

/// Set point light range (falloff distance)
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `range` — Distance at which light reaches zero intensity
///
/// Only affects point lights. Directional lights ignore this.
fn light_range(
    mut caller: Caller<'_, ZGameContext>,
    index: u32,
    range: f32,
) {
    if index > 3 {
        warn!("light_range: invalid light index {} (must be 0-3)", index);
        return;
    }

    let range = range.max(0.0); // Clamp negative to 0

    let state = &mut caller.data_mut().ffi;
    let light = &state.current_shading_state.lights[index as usize];

    // Only valid for point lights
    if light.get_type() != LightType::Point {
        warn!("light_range: light {} is directional, not point", index);
        return;
    }

    let position = light.get_position();
    let color = light.get_color();
    let intensity = light.get_intensity();
    let enabled = light.is_enabled();

    state.update_point_light(index as usize, position, color, intensity, range, enabled);
}
