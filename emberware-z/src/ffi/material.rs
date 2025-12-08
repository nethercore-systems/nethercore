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

/// Set rim lighting parameters (all lit modes)
///
/// # Arguments
/// * `intensity` — Rim intensity 0.0-1.0 (stored in uniform_set_0 byte 3)
/// * `power` — Rim falloff power 0.0-32.0 (stored in uniform_set_1 byte 3)
///
/// Rim lighting creates edge highlights. Intensity controls brightness,
/// power controls falloff sharpness (higher = tighter highlight).
/// Rim lighting uses the sky color from behind the object for coherent scene lighting.
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

    // Clamp and warn for power (0-32 range, normalized to 0-1 for storage)
    let power_clamped = power.clamp(0.0, 32.0);
    if (power - power_clamped).abs() > 0.001 {
        warn!(
            "material_rim: power {} out of range, clamped to {}",
            power, power_clamped
        );
    }

    // Set rim_intensity in uniform_set_0 byte 3
    state.update_material_rim_intensity(intensity_clamped);

    // Set rim_power in uniform_set_1 byte 3 (normalized: 0-32 → 0-1 → 0-255)
    state.update_material_rim_power(power_clamped);
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

/// Set specular color (Mode 3 only)
/// color: RGBA8 packed u32 (RGB used, A ignored)
/// Format: 0xRRGGBBAA (R in highest byte, A in lowest)
/// 0xFFFFFFFF = white (neutral specular - highlights match light color)
/// Tinted values create colored highlights (e.g., warm gold, cool silver)
fn material_specular(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, color: u32) {
    let state = &mut caller.data_mut().console;

    // Extract RGB bytes from input and convert to normalized floats
    // Format: 0xRRGGBBAA (R in highest byte, A in lowest)
    let r = ((color >> 24) & 0xFF) as f32 / 255.0;
    let g = ((color >> 16) & 0xFF) as f32 / 255.0;
    let b = ((color >> 8) & 0xFF) as f32 / 255.0;

    // Specular RGB stored in uniform_set_1 bytes 0-2 (byte 3 = rim_power)
    state.update_specular_color(r, g, b);
}
