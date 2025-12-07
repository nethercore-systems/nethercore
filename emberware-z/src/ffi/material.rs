//! Material FFI functions
//!
//! Functions for setting PBR and Blinn-Phong material properties.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::ZFFIState;

/// Register material FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "material_mre", material_mre)?;
    linker.func_wrap("env", "material_albedo", material_albedo)?;
    linker.func_wrap("env", "material_metallic", material_metallic)?;
    linker.func_wrap("env", "material_roughness", material_roughness)?;
    linker.func_wrap("env", "material_emissive", material_emissive)?;
    linker.func_wrap("env", "material_rim", material_rim)?;
    linker.func_wrap("env", "material_shininess", material_shininess)?;
    linker.func_wrap("env", "material_specular", material_specular)?;
    linker.func_wrap(
        "env",
        "material_specular_intensity",
        material_specular_intensity,
    )?;
    Ok(())
}

/// Bind an MRE texture (Metallic-Roughness-Emissive)
///
/// # Arguments
/// * `texture` — Texture handle where R=Metallic, G=Roughness, B=Emissive
///
/// Binds to slot 1. Used in Mode 2 (PBR) and Mode 3 (Hybrid).
/// In Mode 2/3, slot 1 is interpreted as an MRE texture instead of a matcap.
fn material_mre(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, texture: u32) {
    let state = &mut caller.data_mut().console;
    state.bound_textures[1] = texture;
}

/// Bind an albedo texture
///
/// # Arguments
/// * `texture` — Texture handle for the base color/albedo map
///
/// Binds to slot 0. This is equivalent to texture_bind(texture) but more semantically clear.
/// The albedo texture is multiplied with the uniform color and vertex colors.
fn material_albedo(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, texture: u32) {
    let state = &mut caller.data_mut().console;
    state.bound_textures[0] = texture;
}

/// Set the material metallic value
///
/// # Arguments
/// * `value` — Metallic value (0.0 = dielectric, 1.0 = metal)
///
/// Used in Mode 2 (PBR) and Mode 3 (Hybrid).
/// Clamped to 0.0-1.0 range. Default is 0.0 (non-metallic).
fn material_metallic(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, value: f32) {
    let state = &mut caller.data_mut().console;
    let clamped = value.clamp(0.0, 1.0);

    if (value - clamped).abs() > 0.001 {
        warn!(
            "material_metallic: value {} out of range, clamped to {}",
            value, clamped
        );
    }

    // Quantize and store only in current_shading_state
    state.update_material_metallic(clamped);
}

/// Set the material roughness value
///
/// # Arguments
/// * `value` — Roughness value (0.0 = smooth, 1.0 = rough)
///
/// Used in Mode 2 (PBR) and Mode 3 (Hybrid).
/// Clamped to 0.0-1.0 range. Default is 0.5.
fn material_roughness(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, value: f32) {
    let state = &mut caller.data_mut().console;
    let clamped = value.clamp(0.0, 1.0);

    if (value - clamped).abs() > 0.001 {
        warn!(
            "material_roughness: value {} out of range, clamped to {}",
            value, clamped
        );
    }

    // Quantize and store only in current_shading_state
    state.update_material_roughness(clamped);
}

/// Set the material emissive intensity
///
/// # Arguments
/// * `value` — Emissive intensity (0.0 = no emission, higher = brighter)
///
/// Used in Mode 2 (PBR) and Mode 3 (Hybrid).
/// Values can be greater than 1.0 for HDR-like effects. Default is 0.0.
fn material_emissive(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, value: f32) {
    let state = &mut caller.data_mut().console;

    // No clamping for emissive - allow HDR values
    let clamped = if value < 0.0 {
        warn!(
            "material_emissive: negative value {} not allowed, using 0.0",
            value
        );
        0.0
    } else {
        value
    };

    // Quantize and store only in current_shading_state
    state.update_material_emissive(clamped);
}

/// Set rim lighting parameters (Mode 3 only)
///
/// # Arguments
/// * `intensity` — Rim intensity 0.0-1.0 (uniform fallback for Slot 1.R)
/// * `power` — Rim falloff power 0.0-1.0 (mapped to 0-32 internally, uniform-only, no texture)
///
/// Used in Mode 3 (Blinn-Phong) for edge lighting effects.
/// Rim lighting uses the sun color for coherent scene lighting.
/// Intensity is clamped to 0.0-1.0 range. Power is clamped to 0.0-1.0 and mapped to 0-32.
fn material_rim(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    intensity: f32,
    power: f32,
) {
    let state = &mut caller.data_mut().console;

    // Clamp and warn for intensity
    let intensity_clamped = intensity.clamp(0.0, 1.0);
    if (intensity - intensity_clamped).abs() > 0.001 {
        warn!(
            "material_rim: intensity {} out of range, clamped to {}",
            intensity, intensity_clamped
        );
    }

    // Clamp and warn for power
    let power_clamped = power.clamp(0.0, 1.0);
    if (power - power_clamped).abs() > 0.001 {
        warn!(
            "material_rim: power {} out of range, clamped to {}",
            power, power_clamped
        );
    }

    // Set rim_intensity in the rim_intensity field (byte 3 of packed_values)
    // This keeps metallic free for PBR metallic (Mode 2) or specular_intensity (Mode 3)
    state.update_rim_intensity(intensity_clamped);

    // Pack rim_power into matcap_blend_modes byte 3 (maps 0-1 to 0-255)
    let power_u8 = (power_clamped * 255.0) as u8;
    let new_value =
        (state.current_shading_state.matcap_blend_modes & 0x00FFFFFF) | ((power_u8 as u32) << 24);

    // Only update if changed
    if state.current_shading_state.matcap_blend_modes != new_value {
        state.current_shading_state.matcap_blend_modes = new_value;
        state.shading_state_dirty = true;
    }
}

/// Set shininess (Mode 3 only, alias for material_roughness)
///
/// # Arguments
/// * `value` — Shininess value 0.0-1.0 (mapped to 1-256 range internally)
///
/// This is an alias for material_roughness() for clarity when using Mode 3.
/// Clamped to 0.0-1.0 range. Default is 0.5 (maps to shininess ~128).
fn material_shininess(caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, value: f32) {
    // Simply calls material_roughness - field reinterpretation happens in the shader
    material_roughness(caller, value);
}

/// Set specular intensity (Mode 3 only)
///
/// # Arguments
/// * `value` — Specular intensity 0.0-1.0 (multiplies specular color)
///
/// Controls the brightness of specular highlights in Mode 3.
/// This is an alias for material_metallic() - the metallic field is reinterpreted
/// as specular_intensity in Mode 3.
///
/// Default is 0.0 - **call this with 1.0 for full specular intensity in Mode 3!**
fn material_specular_intensity(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    value: f32,
) {
    // Simply calls material_metallic - field reinterpretation happens in the shader
    material_metallic(caller, value);
}

/// Set specular color (Mode 3 only)
/// color: RGBA8 packed u32 (RGB used, A ignored)
/// 0xFFFFFFFF = white (neutral specular - highlights match light color)
/// Tinted values create colored highlights (e.g., warm gold, cool silver)
fn material_specular(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, color: u32) {
    let state = &mut caller.data_mut().console;

    // Extract RGB bytes from input (ignore alpha)
    let r_u8 = (color & 0xFF) as u8;
    let g_u8 = ((color >> 8) & 0xFF) as u8;
    let b_u8 = ((color >> 16) & 0xFF) as u8;

    // Pack RGB into matcap_blend_modes bytes 0-2 (preserve byte 3 = rim_power)
    let new_value = (state.current_shading_state.matcap_blend_modes & 0xFF000000)
        | (r_u8 as u32)
        | ((g_u8 as u32) << 8)
        | ((b_u8 as u32) << 16);

    // Only update if changed
    if state.current_shading_state.matcap_blend_modes != new_value {
        state.current_shading_state.matcap_blend_modes = new_value;
        state.shading_state_dirty = true;
    }
}
