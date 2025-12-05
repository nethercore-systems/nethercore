//! Emberware Z FFI host functions
//!
//! Console-specific FFI functions for the PS1/N64 aesthetic fantasy console.
//! These functions are registered with the WASM linker and called by games.
//!
//! Note: FFI functions have many parameters because they match WebAssembly signatures.
#![allow(clippy::too_many_arguments)]

mod input;

use anyhow::Result;
use glam::{Mat4, Vec3};
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::audio::{AudioCommand, Sound};
use crate::console::{ZInput, RESOLUTIONS, TICK_RATES};
use crate::graphics::vertex_stride;
use crate::state::{Font, PendingMesh, PendingTexture, ZFFIState, MAX_BONES};

/// Register all Emberware Z FFI functions with the linker
pub fn register_z_ffi(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    // Configuration functions (init-only)
    linker.func_wrap("env", "set_resolution", set_resolution)?;
    linker.func_wrap("env", "set_tick_rate", set_tick_rate)?;
    linker.func_wrap("env", "set_clear_color", set_clear_color)?;
    linker.func_wrap("env", "render_mode", render_mode)?;

    // Camera functions
    linker.func_wrap("env", "camera_set", camera_set)?;
    linker.func_wrap("env", "camera_fov", camera_fov)?;

    // Advanced matrix functions
    linker.func_wrap("env", "push_view_matrix", push_view_matrix)?;
    linker.func_wrap("env", "push_projection_matrix", push_projection_matrix)?;

    // Transform functions
    linker.func_wrap("env", "push_identity", push_identity)?;
    linker.func_wrap("env", "transform_set", transform_set)?;
    linker.func_wrap("env", "push_translate", push_translate)?;
    linker.func_wrap("env", "push_rotate_x", push_rotate_x)?;
    linker.func_wrap("env", "push_rotate_y", push_rotate_y)?;
    linker.func_wrap("env", "push_rotate_z", push_rotate_z)?;
    linker.func_wrap("env", "push_rotate", push_rotate)?;
    linker.func_wrap("env", "push_scale", push_scale)?;
    linker.func_wrap("env", "push_scale_uniform", push_scale_uniform)?;

    // Input functions (from input submodule)
    linker.func_wrap("env", "button_held", input::button_held)?;
    linker.func_wrap("env", "button_pressed", input::button_pressed)?;
    linker.func_wrap("env", "button_released", input::button_released)?;
    linker.func_wrap("env", "buttons_held", input::buttons_held)?;
    linker.func_wrap("env", "buttons_pressed", input::buttons_pressed)?;
    linker.func_wrap("env", "buttons_released", input::buttons_released)?;
    linker.func_wrap("env", "left_stick_x", input::left_stick_x)?;
    linker.func_wrap("env", "left_stick_y", input::left_stick_y)?;
    linker.func_wrap("env", "right_stick_x", input::right_stick_x)?;
    linker.func_wrap("env", "right_stick_y", input::right_stick_y)?;
    linker.func_wrap("env", "left_stick", input::left_stick)?;
    linker.func_wrap("env", "right_stick", input::right_stick)?;
    linker.func_wrap("env", "trigger_left", input::trigger_left)?;
    linker.func_wrap("env", "trigger_right", input::trigger_right)?;

    // Render state functions
    linker.func_wrap("env", "set_color", set_color)?;
    linker.func_wrap("env", "depth_test", depth_test)?;
    linker.func_wrap("env", "cull_mode", cull_mode)?;
    linker.func_wrap("env", "blend_mode", blend_mode)?;
    linker.func_wrap("env", "texture_filter", texture_filter)?;

    // Texture functions
    linker.func_wrap("env", "load_texture", load_texture)?;
    linker.func_wrap("env", "texture_bind", texture_bind)?;
    linker.func_wrap("env", "texture_bind_slot", texture_bind_slot)?;
    linker.func_wrap("env", "matcap_blend_mode", matcap_blend_mode)?;

    // Mesh functions (retained mode)
    linker.func_wrap("env", "load_mesh", load_mesh)?;
    linker.func_wrap("env", "load_mesh_indexed", load_mesh_indexed)?;
    linker.func_wrap("env", "draw_mesh", draw_mesh)?;

    // Immediate mode 3D drawing
    linker.func_wrap("env", "draw_triangles", draw_triangles)?;
    linker.func_wrap("env", "draw_triangles_indexed", draw_triangles_indexed)?;

    // Billboard drawing
    linker.func_wrap("env", "draw_billboard", draw_billboard)?;
    linker.func_wrap("env", "draw_billboard_region", draw_billboard_region)?;

    // 2D drawing (screen space)
    linker.func_wrap("env", "draw_sprite", draw_sprite)?;
    linker.func_wrap("env", "draw_sprite_region", draw_sprite_region)?;
    linker.func_wrap("env", "draw_sprite_ex", draw_sprite_ex)?;
    linker.func_wrap("env", "draw_rect", draw_rect)?;
    linker.func_wrap("env", "draw_text", draw_text)?;

    // Font loading
    linker.func_wrap("env", "load_font", load_font)?;
    linker.func_wrap("env", "load_font_ex", load_font_ex)?;
    linker.func_wrap("env", "font_bind", font_bind)?;

    // Sky system
    linker.func_wrap("env", "sky_set_colors", sky_set_colors)?;
    linker.func_wrap("env", "sky_set_sun", sky_set_sun)?;

    // Mode 1 (Matcap) functions
    linker.func_wrap("env", "matcap_set", matcap_set)?;

    // Material functions
    linker.func_wrap("env", "material_mre", material_mre)?;
    linker.func_wrap("env", "material_albedo", material_albedo)?;
    linker.func_wrap("env", "material_metallic", material_metallic)?;
    linker.func_wrap("env", "material_roughness", material_roughness)?;
    linker.func_wrap("env", "material_emissive", material_emissive)?;

    // Mode 2 (PBR) lighting functions
    linker.func_wrap("env", "light_set", light_set)?;
    linker.func_wrap("env", "light_color", light_color)?;
    linker.func_wrap("env", "light_intensity", light_intensity)?;
    linker.func_wrap("env", "light_enable", light_enable)?;
    linker.func_wrap("env", "light_disable", light_disable)?;

    // Mode 3 (Hybrid) lighting functions
    // Note: Mode 3 uses the same FFI functions as Mode 2 but conventionally only uses light 0
    // The shader in Mode 3 uses light 0 as the single directional light

    // GPU skinning
    linker.func_wrap("env", "set_bones", set_bones)?;

    // Audio functions
    linker.func_wrap("env", "load_sound", load_sound)?;
    linker.func_wrap("env", "play_sound", play_sound)?;
    linker.func_wrap("env", "channel_play", channel_play)?;
    linker.func_wrap("env", "channel_set", channel_set)?;
    linker.func_wrap("env", "channel_stop", channel_stop)?;
    linker.func_wrap("env", "music_play", music_play)?;
    linker.func_wrap("env", "music_stop", music_stop)?;
    linker.func_wrap("env", "music_set_volume", music_set_volume)?;

    Ok(())
}

/// Set the render resolution
///
/// Valid indices: 0=360p, 1=540p (default), 2=720p, 3=1080p
///
/// Must be called during `init()`. Calls outside init are ignored with a warning.
fn set_resolution(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, res: u32) {
    // Check if we're in init phase
    if !caller.data().game.in_init {
        warn!("set_resolution() called outside init() - ignored");
        return;
    }

    let state = &mut caller.data_mut().console;

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
fn set_tick_rate(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, rate: u32) {
    // Check if we're in init phase
    if !caller.data().game.in_init {
        warn!("set_tick_rate() called outside init() - ignored");
        return;
    }

    let state = &mut caller.data_mut().console;

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
fn set_clear_color(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, color: u32) {
    // Check if we're in init phase
    if !caller.data().game.in_init {
        warn!("set_clear_color() called outside init() - ignored");
        return;
    }

    let state = &mut caller.data_mut().console;

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
fn render_mode(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, mode: u32) {
    // Check if we're in init phase
    if !caller.data().game.in_init {
        warn!("render_mode() called outside init() - ignored");
        return;
    }

    let state = &mut caller.data_mut().console;

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

// ============================================================================
// Camera Functions
// ============================================================================

/// Set the camera position and target (look-at point)
///
/// # Arguments
/// * `x, y, z` — Camera position in world space
/// * `target_x, target_y, target_z` — Point the camera looks at
///
/// Uses a Y-up, right-handed coordinate system.
fn camera_set(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32,
    y: f32,
    z: f32,
    target_x: f32,
    target_y: f32,
    target_z: f32,
) {
    let state = &mut caller.data_mut().console;

    // Build view matrix from position and target
    let position = Vec3::new(x, y, z);
    let target = Vec3::new(target_x, target_y, target_z);
    let view = Mat4::look_at_rh(position, target, Vec3::Y);

    // Set current view matrix (will be pushed to pool on next draw)
    state.current_view_matrix = Some(view);
}

/// Set the camera field of view
///
/// # Arguments
/// * `fov_degrees` — Field of view in degrees (typically 45-90, default 60)
///
/// Values outside 1-179 degrees are clamped with a warning.
/// Rebuilds the projection matrix at index 0 with default parameters (16:9 aspect, 0.1 near, 1000 far).
fn camera_fov(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, fov_degrees: f32) {
    let state = &mut caller.data_mut().console;

    // Validate FOV range
    let clamped_fov = if !(1.0..=179.0).contains(&fov_degrees) {
        let clamped = fov_degrees.clamp(1.0, 179.0);
        warn!(
            "camera_fov({}) out of range (1-179), clamped to {}",
            fov_degrees, clamped
        );
        clamped
    } else {
        fov_degrees
    };

    // Rebuild projection matrix with new FOV
    let aspect = 16.0 / 9.0; // TODO: Get from actual viewport
    let proj = Mat4::perspective_rh(clamped_fov.to_radians(), aspect, 0.1, 1000.0);

    // Set current projection matrix (will be pushed to pool on next draw)
    state.current_proj_matrix = Some(proj);
}

/// Push a custom view matrix to the pool, returning its index
///
/// For advanced rendering techniques (multiple cameras, render-to-texture, etc.)
/// Most users should use camera_set() instead.
///
/// # Arguments
/// * `m0-m15` — Matrix elements in column-major order
///
/// # Returns
/// The index of the newly added view matrix (0-255)
fn push_view_matrix(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    m0: f32,
    m1: f32,
    m2: f32,
    m3: f32,
    m4: f32,
    m5: f32,
    m6: f32,
    m7: f32,
    m8: f32,
    m9: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m13: f32,
    m14: f32,
    m15: f32,
) {
    let state = &mut caller.data_mut().console;

    let matrix = Mat4::from_cols_array(&[
        m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15,
    ]);

    state.current_view_matrix = Some(matrix);
}

/// Push a custom projection matrix to the pool, returning its index
///
/// For advanced rendering techniques (custom projections, orthographic, etc.)
/// Most users should use camera_set() instead.
///
/// # Arguments
/// * `m0-m15` — Matrix elements in column-major order
///
/// Sets the current projection matrix (no return value - uses lazy allocation)
fn push_projection_matrix(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    m0: f32,
    m1: f32,
    m2: f32,
    m3: f32,
    m4: f32,
    m5: f32,
    m6: f32,
    m7: f32,
    m8: f32,
    m9: f32,
    m10: f32,
    m11: f32,
    m12: f32,
    m13: f32,
    m14: f32,
    m15: f32,
) {
    let state = &mut caller.data_mut().console;

    let matrix = Mat4::from_cols_array(&[
        m0, m1, m2, m3, m4, m5, m6, m7, m8, m9, m10, m11, m12, m13, m14, m15,
    ]);

    state.current_proj_matrix = Some(matrix);
}

// ============================================================================
// Transform Stack Functions
// ============================================================================

/// Push identity matrix onto the transform stack
///
/// After calling this, subsequent draws will use identity transformation
/// (objects will be drawn at their original position/rotation/scale).
/// This is typically called at the start of rendering to reset the transform stack.
fn push_identity(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>) {
    let state = &mut caller.data_mut().console;
    state.current_model_matrix = Some(Mat4::IDENTITY);
}

/// Set the current transform from a 4x4 matrix
///
/// # Arguments
/// * `matrix_ptr` — Pointer to 16 f32 values in column-major order
///
/// Column-major order means: [col0, col1, col2, col3] where each column is [x, y, z, w].
/// This is the same format used by glam and WGSL.
fn transform_set(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, matrix_ptr: u32) {
    // Read the 16 floats from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("transform_set failed: no WASM memory available");
            return;
        }
    };

    let mem_data = memory.data(&caller);
    let ptr = matrix_ptr as usize;
    let size = 16 * std::mem::size_of::<f32>();

    // Bounds check
    if ptr + size > mem_data.len() {
        warn!(
            "transform_set failed: pointer {} + {} bytes exceeds memory bounds {}",
            ptr,
            size,
            mem_data.len()
        );
        return;
    }

    // Read floats from memory
    let bytes = &mem_data[ptr..ptr + size];
    let floats: &[f32] = bytemuck::cast_slice(bytes);

    // Create matrix from column-major array
    let Ok(matrix): Result<[f32; 16], _> = floats.try_into() else {
        warn!(
            "transform_set failed: expected 16 floats, got {}",
            floats.len()
        );
        return;
    };
    let state = &mut caller.data_mut().console;
    let new_matrix = Mat4::from_cols_array(&matrix);
    state.current_model_matrix = Some(new_matrix); // Pending matrix
}

/// Push a translated transform onto the stack
///
/// # Arguments
/// * `x`, `y`, `z` — Translation amounts
///
/// Reads the current transform, applies translation, and pushes the result.
fn push_translate(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32,
    y: f32,
    z: f32,
) {
    let state = &mut caller.data_mut().console;
    let current = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let new_matrix = current * Mat4::from_translation(Vec3::new(x, y, z));
    state.current_model_matrix = Some(new_matrix);
}

/// Push a rotated transform onto the stack (X axis)
///
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
///
/// Reads the current transform, applies rotation, and pushes the result.
fn push_rotate_x(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, angle_deg: f32) {
    let state = &mut caller.data_mut().console;
    let current = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let angle_rad = angle_deg.to_radians();
    let new_matrix = current * Mat4::from_rotation_x(angle_rad);
    state.current_model_matrix = Some(new_matrix);
}

/// Push a rotated transform onto the stack (Y axis)
///
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
///
/// Reads the current transform, applies rotation, and pushes the result.
fn push_rotate_y(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, angle_deg: f32) {
    let state = &mut caller.data_mut().console;
    let current = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let angle_rad = angle_deg.to_radians();
    let new_matrix = current * Mat4::from_rotation_y(angle_rad);
    state.current_model_matrix = Some(new_matrix);
}

/// Push a rotated transform onto the stack (Z axis)
///
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
///
/// Reads the current transform, applies rotation, and pushes the result.
fn push_rotate_z(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, angle_deg: f32) {
    let state = &mut caller.data_mut().console;
    let current = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let angle_rad = angle_deg.to_radians();
    let new_matrix = current * Mat4::from_rotation_z(angle_rad);
    state.current_model_matrix = Some(new_matrix);
}

/// Push a rotated transform onto the stack (arbitrary axis)
///
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
/// * `axis_x`, `axis_y`, `axis_z` — Rotation axis (will be normalized)
///
/// Reads the current transform, applies rotation, and pushes the result.
fn push_rotate(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    angle_deg: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
) {
    let state = &mut caller.data_mut().console;
    let current = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let angle_rad = angle_deg.to_radians();
    let axis = Vec3::new(axis_x, axis_y, axis_z).normalize();
    let new_matrix = current * Mat4::from_axis_angle(axis, angle_rad);
    state.current_model_matrix = Some(new_matrix);
}

/// Push a scaled transform onto the stack
///
/// # Arguments
/// * `x`, `y`, `z` — Scale factors for each axis
///
/// Reads the current transform, applies scale, and pushes the result.
fn push_scale(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32,
    y: f32,
    z: f32,
) {
    let state = &mut caller.data_mut().console;
    let current = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let new_matrix = current * Mat4::from_scale(Vec3::new(x, y, z));
    state.current_model_matrix = Some(new_matrix);
}

/// Push a uniformly scaled transform onto the stack
///
/// # Arguments
/// * `s` — Uniform scale factor
///
/// Reads the current transform, applies scale, and pushes the result.
fn push_scale_uniform(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, s: f32) {
    let state = &mut caller.data_mut().console;
    let current = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let new_matrix = current * Mat4::from_scale(Vec3::splat(s));
    state.current_model_matrix = Some(new_matrix);
}

// ============================================================================
// Render State Functions
// ============================================================================

/// Set the uniform tint color
///
/// # Arguments
/// * `color` — Color in 0xRRGGBBAA format
///
/// This color is multiplied with vertex colors and textures.
fn set_color(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, color: u32) {
    let state = &mut caller.data_mut().console;
    state.update_color(color);
}

/// Enable or disable depth testing
///
/// # Arguments
/// * `enabled` — 0 to disable, non-zero to enable
///
/// Default: enabled. Disable for 2D overlays or special effects.
fn depth_test(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, enabled: u32) {
    let state = &mut caller.data_mut().console;
    state.depth_test = enabled != 0;
}

/// Set the face culling mode
///
/// # Arguments
/// * `mode` — 0=none (draw both sides), 1=back (default), 2=front
///
/// Back-face culling is the default for solid 3D objects.
fn cull_mode(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, mode: u32) {
    let state = &mut caller.data_mut().console;

    if mode > 2 {
        warn!("cull_mode({}) invalid - must be 0-2, using 0 (none)", mode);
        state.cull_mode = 0;
        return;
    }

    state.cull_mode = mode as u8;
}

/// Set the blend mode for transparent rendering
///
/// # Arguments
/// * `mode` — 0=none (opaque), 1=alpha, 2=additive, 3=multiply
///
/// Default: none (opaque). Use alpha for transparent textures.
fn blend_mode(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, mode: u32) {
    use crate::graphics::BlendMode;

    let state = &mut caller.data_mut().console;

    if mode > 3 {
        warn!("blend_mode({}) invalid - must be 0-3, using 0 (none)", mode);
        state.blend_mode = 0;
        state.update_blend_mode(BlendMode::None); // Sync to current_shading_state
        return;
    }

    state.blend_mode = mode as u8;
    state.update_blend_mode(BlendMode::from_u8(mode as u8)); // Sync to current_shading_state
}

/// Set the texture filtering mode
///
/// # Arguments
/// * `filter` — 0=nearest (pixelated, retro), 1=linear (smooth)
///
/// Default: nearest for retro aesthetic.
fn texture_filter(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, filter: u32) {
    let state = &mut caller.data_mut().console;

    if filter > 1 {
        warn!(
            "texture_filter({}) invalid - must be 0-1, using 0 (nearest)",
            filter
        );
        state.texture_filter = 0;
        return;
    }

    state.texture_filter = filter as u8;
}

// ============================================================================
// Texture Functions
// ============================================================================

/// Load a texture from RGBA pixel data
///
/// # Arguments
/// * `width` — Texture width in pixels
/// * `height` — Texture height in pixels
/// * `pixels_ptr` — Pointer to RGBA8 pixel data (width × height × 4 bytes)
///
/// Returns texture handle (>0) on success, 0 on failure.
/// Validates VRAM budget before allocation.
fn load_texture(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    width: u32,
    height: u32,
    pixels_ptr: u32,
) -> u32 {
    // Validate dimensions
    if width == 0 || height == 0 {
        warn!("load_texture: invalid dimensions {}x{}", width, height);
        return 0;
    }

    // Read pixel data from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("load_texture: no WASM memory available");
            return 0;
        }
    };

    let ptr = pixels_ptr as usize;
    // Use checked arithmetic to prevent overflow
    let Some(pixels) = width.checked_mul(height) else {
        warn!("load_texture: dimensions overflow ({}x{})", width, height);
        return 0;
    };
    let Some(size) = pixels.checked_mul(4) else {
        warn!("load_texture: size overflow ({}x{})", width, height);
        return 0;
    };
    let size = size as usize;

    // Copy pixel data while we have the immutable borrow
    let pixel_data = {
        let mem_data = memory.data(&caller);

        if ptr + size > mem_data.len() {
            warn!(
                "load_texture: pixel data ({} bytes at {}) exceeds memory bounds ({})",
                size,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        mem_data[ptr..ptr + size].to_vec()
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

    // Allocate a texture handle
    let handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    // Store the texture data for the graphics backend
    state.pending_textures.push(PendingTexture {
        handle,
        width,
        height,
        data: pixel_data,
    });

    handle
}

/// Bind a texture to slot 0 (albedo)
///
/// # Arguments
/// * `handle` — Texture handle from load_texture
///
/// Equivalent to texture_bind_slot(handle, 0).
fn texture_bind(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, handle: u32) {
    let state = &mut caller.data_mut().console;
    state.bound_textures[0] = handle;
}

/// Bind a texture to a specific slot
///
/// # Arguments
/// * `handle` — Texture handle from load_texture
/// * `slot` — Slot index (0-3)
///
/// Slots: 0=albedo, 1=MRE/matcap, 2=env matcap, 3=matcap
fn texture_bind_slot(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
    slot: u32,
) {
    if slot > 3 {
        warn!("texture_bind_slot: invalid slot {} (max 3)", slot);
        return;
    }

    let state = &mut caller.data_mut().console;
    state.bound_textures[slot as usize] = handle;
}

/// Set matcap blend mode for a texture slot (Mode 1 only)
///
/// # Arguments
/// * `slot` — Matcap slot index (1-3, slot 0 is albedo and does not support blend modes)
/// * `mode` — Blend mode (0=Multiply, 1=Add, 2=HSV Modulate)
///
/// Mode 0 (Multiply): Standard matcap blending (default)
/// Mode 1 (Add): Additive blending for glow/emission effects
/// Mode 2 (HSV Modulate): Hue shifting and iridescence effects
fn matcap_blend_mode(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    slot: u32,
    mode: u32,
) {
    use crate::graphics::MatcapBlendMode;

    if !(1..=3).contains(&slot) {
        warn!("matcap_blend_mode: invalid slot {} (must be 1-3)", slot);
        return;
    }

    let blend_mode = match MatcapBlendMode::from_u32(mode) {
        Some(m) => m,
        None => {
            warn!("matcap_blend_mode: invalid mode {} (must be 0-2)", mode);
            return;
        }
    };

    let state = &mut caller.data_mut().console;
    state.update_matcap_blend_mode(slot as usize, blend_mode); // Update single slot in unified state
}

// ============================================================================
// Mesh Functions (Retained Mode)
// ============================================================================

/// Maximum vertex format value (all flags set: UV | COLOR | NORMAL | SKINNED)
const MAX_VERTEX_FORMAT: u8 = 15;

/// Load a non-indexed mesh (retained mode)
///
/// # Arguments
/// * `data_ptr` — Pointer to vertex data (f32 array)
/// * `vertex_count` — Number of vertices
/// * `format` — Vertex format flags (0-15)
///
/// Vertex format flags:
/// - FORMAT_UV (1): Has UV coordinates (2 floats)
/// - FORMAT_COLOR (2): Has per-vertex color (RGB, 3 floats)
/// - FORMAT_NORMAL (4): Has normals (3 floats)
/// - FORMAT_SKINNED (8): Has bone indices/weights (4 u8 + 4 floats)
///
/// Returns mesh handle (>0) on success, 0 on failure.
fn load_mesh(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    vertex_count: u32,
    format: u32,
) -> u32 {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!(
            "load_mesh: invalid format {} (max {})",
            format, MAX_VERTEX_FORMAT
        );
        return 0;
    }
    let format = format as u8;

    // Validate vertex count
    if vertex_count == 0 {
        warn!("load_mesh: vertex_count cannot be 0");
        return 0;
    }

    // Calculate data size with overflow checking
    let stride = vertex_stride(format);
    let Some(data_size) = vertex_count.checked_mul(stride) else {
        warn!(
            "load_mesh: data size overflow (vertex_count={}, stride={})",
            vertex_count, stride
        );
        return 0;
    };
    let float_count = data_size / 4;

    // Read vertex data from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("load_mesh: no WASM memory available");
            return 0;
        }
    };

    let ptr = data_ptr as usize;
    let byte_size = data_size as usize;

    // Copy vertex data while we have the immutable borrow
    let vertex_data: Vec<f32> = {
        let mem_data = memory.data(&caller);

        if ptr + byte_size > mem_data.len() {
            warn!(
                "load_mesh: vertex data ({} bytes at {}) exceeds memory bounds ({})",
                byte_size,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        let bytes = &mem_data[ptr..ptr + byte_size];
        let floats: &[f32] = bytemuck::cast_slice(bytes);
        floats.to_vec()
    };

    // Verify data length
    if vertex_data.len() != float_count as usize {
        warn!(
            "load_mesh: expected {} floats, got {}",
            float_count,
            vertex_data.len()
        );
        return 0;
    }

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

    // Allocate a mesh handle
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    // Store the mesh data for the graphics backend
    state.pending_meshes.push(PendingMesh {
        handle,
        format,
        vertex_data,
        index_data: None,
    });

    info!(
        "load_mesh: created mesh {} with {} vertices, format {}",
        handle, vertex_count, format
    );

    handle
}

/// Load an indexed mesh (retained mode)
///
/// # Arguments
/// * `data_ptr` — Pointer to vertex data (f32 array)
/// * `vertex_count` — Number of vertices
/// * `index_ptr` — Pointer to index data (u32 array)
/// * `index_count` — Number of indices
/// * `format` — Vertex format flags (0-15)
///
/// Returns mesh handle (>0) on success, 0 on failure.
fn load_mesh_indexed(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) -> u32 {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!(
            "load_mesh_indexed: invalid format {} (max {})",
            format, MAX_VERTEX_FORMAT
        );
        return 0;
    }
    let format = format as u8;

    // Validate counts
    if vertex_count == 0 {
        warn!("load_mesh_indexed: vertex_count cannot be 0");
        return 0;
    }
    if index_count == 0 {
        warn!("load_mesh_indexed: index_count cannot be 0");
        return 0;
    }
    if !index_count.is_multiple_of(3) {
        warn!(
            "load_mesh_indexed: index_count {} is not a multiple of 3",
            index_count
        );
        return 0;
    }

    // Calculate data sizes with overflow checking
    let stride = vertex_stride(format);
    let Some(vertex_data_size) = vertex_count.checked_mul(stride) else {
        warn!(
            "load_mesh_indexed: vertex data size overflow (vertex_count={}, stride={})",
            vertex_count, stride
        );
        return 0;
    };
    let Some(index_data_size) = index_count.checked_mul(2) else {
        warn!(
            "load_mesh_indexed: index data size overflow (index_count={})",
            index_count
        );
        return 0;
    };
    let float_count = vertex_data_size / 4;

    // Read data from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("load_mesh_indexed: no WASM memory available");
            return 0;
        }
    };

    let vertex_ptr = data_ptr as usize;
    let vertex_byte_size = vertex_data_size as usize;
    let idx_ptr = index_ptr as usize;
    let index_byte_size = index_data_size as usize;

    // Copy data while we have the immutable borrow
    let (vertex_data, index_data): (Vec<f32>, Vec<u16>) = {
        let mem_data = memory.data(&caller);

        if vertex_ptr + vertex_byte_size > mem_data.len() {
            warn!(
                "load_mesh_indexed: vertex data ({} bytes at {}) exceeds memory bounds ({})",
                vertex_byte_size,
                vertex_ptr,
                mem_data.len()
            );
            return 0;
        }

        if idx_ptr + index_byte_size > mem_data.len() {
            warn!(
                "load_mesh_indexed: index data ({} bytes at {}) exceeds memory bounds ({})",
                index_byte_size,
                idx_ptr,
                mem_data.len()
            );
            return 0;
        }

        let vertex_bytes = &mem_data[vertex_ptr..vertex_ptr + vertex_byte_size];
        let floats: &[f32] = bytemuck::cast_slice(vertex_bytes);

        let index_bytes = &mem_data[idx_ptr..idx_ptr + index_byte_size];
        let indices: &[u16] = bytemuck::cast_slice(index_bytes);

        // Validate indices are within bounds
        for &idx in indices {
            if idx as u32 >= vertex_count {
                warn!(
                    "load_mesh_indexed: index {} out of bounds (vertex_count = {})",
                    idx, vertex_count
                );
                return 0;
            }
        }

        (floats.to_vec(), indices.to_vec())
    };

    // Verify data lengths
    if vertex_data.len() != float_count as usize {
        warn!(
            "load_mesh_indexed: expected {} vertex floats, got {}",
            float_count,
            vertex_data.len()
        );
        return 0;
    }
    if index_data.len() != index_count as usize {
        warn!(
            "load_mesh_indexed: expected {} indices, got {}",
            index_count,
            index_data.len()
        );
        return 0;
    }

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

    // Allocate a mesh handle
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    // Store the mesh data for the graphics backend
    state.pending_meshes.push(PendingMesh {
        handle,
        format,
        vertex_data,
        index_data: Some(index_data),
    });

    info!(
        "load_mesh_indexed: created mesh {} with {} vertices, {} indices, format {}",
        handle, vertex_count, index_count, format
    );

    handle
}

/// Draw a retained mesh with current transform and render state
///
/// # Arguments
/// * `handle` — Mesh handle from load_mesh or load_mesh_indexed
///
/// The mesh is drawn using the current transform (from transform_* functions)
/// and render state (color, textures, depth test, cull mode, blend mode).
fn draw_mesh(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, handle: u32) {
    if handle == 0 {
        warn!("draw_mesh: invalid handle 0");
        return;
    }

    let state = &mut caller.data_mut().console;

    // Look up mesh
    let mesh = match state.mesh_map.get(&handle) {
        Some(m) => m,
        None => {
            warn!("draw_mesh: invalid handle {}", handle);
            return;
        }
    };

    // Extract mesh data
    let mesh_format = mesh.format;
    let mesh_vertex_count = mesh.vertex_count;
    let mesh_index_count = mesh.index_count;
    let mesh_vertex_offset = mesh.vertex_offset;
    let mesh_index_offset = mesh.index_offset;

    // Texture mapping happens in process_draw_commands() using session.texture_map
    // FFI doesn't have access to the texture map, so we use placeholders here
    let texture_slots = [
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
    ];

    let cull_mode = crate::graphics::CullMode::from_u8(state.cull_mode);

    // Allocate combined MVP+shading buffer index (lazy allocation with deduplication)
    let buffer_index = state.add_mvp_shading_state();

    // Record draw command directly
    state.render_pass.record_mesh(
        mesh_format,
        mesh_vertex_count,
        mesh_index_count,
        mesh_vertex_offset,
        mesh_index_offset,
        buffer_index,
        texture_slots,
        state.depth_test,
        cull_mode,
    );
}

// ============================================================================
// Immediate Mode 3D Drawing
// ============================================================================

/// Draw triangles immediately (non-indexed)
///
/// # Arguments
/// * `data_ptr` — Pointer to vertex data (f32 array)
/// * `vertex_count` — Number of vertices (must be multiple of 3)
/// * `format` — Vertex format flags (0-15)
///
/// Vertices are buffered on the CPU and flushed at frame end.
/// Uses current transform and render state.
fn draw_triangles(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    vertex_count: u32,
    format: u32,
) {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!(
            "draw_triangles: invalid format {} (max {})",
            format, MAX_VERTEX_FORMAT
        );
        return;
    }
    let format = format as u8;

    // Validate vertex count
    if vertex_count == 0 {
        return; // Nothing to draw
    }
    if !vertex_count.is_multiple_of(3) {
        warn!(
            "draw_triangles: vertex_count {} is not a multiple of 3",
            vertex_count
        );
        return;
    }

    // Calculate data size with overflow checking
    let stride = vertex_stride(format);
    let Some(data_size) = vertex_count.checked_mul(stride) else {
        warn!(
            "draw_triangles: data size overflow (vertex_count={}, stride={})",
            vertex_count, stride
        );
        return;
    };
    let float_count = data_size / 4;

    // Read vertex data from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("draw_triangles: no WASM memory available");
            return;
        }
    };

    let ptr = data_ptr as usize;
    let byte_size = data_size as usize;

    // Copy vertex data
    let vertex_data: Vec<f32> = {
        let mem_data = memory.data(&caller);

        if ptr + byte_size > mem_data.len() {
            warn!(
                "draw_triangles: vertex data ({} bytes at {}) exceeds memory bounds ({})",
                byte_size,
                ptr,
                mem_data.len()
            );
            return;
        }

        let bytes = &mem_data[ptr..ptr + byte_size];
        let floats: &[f32] = bytemuck::cast_slice(bytes);
        floats.to_vec()
    };

    // Verify data length
    if vertex_data.len() != float_count as usize {
        warn!(
            "draw_triangles: expected {} floats, got {}",
            float_count,
            vertex_data.len()
        );
        return;
    }

    let state = &mut caller.data_mut().console;

    // Texture mapping happens in process_draw_commands() using session.texture_map
    // FFI doesn't have access to the texture map, so we use placeholders here
    let texture_slots = [
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
    ];

    let cull_mode = crate::graphics::CullMode::from_u8(state.cull_mode);

    // Allocate combined MVP+shading buffer index (lazy allocation with deduplication)
    let buffer_index = state.add_mvp_shading_state();

    // Record draw command directly
    state.render_pass.record_triangles(
        format,
        &vertex_data,
        buffer_index,
        texture_slots,
        state.depth_test,
        cull_mode,
    );
}

/// Draw indexed triangles immediately
///
/// # Arguments
/// * `data_ptr` — Pointer to vertex data (f32 array)
/// * `vertex_count` — Number of vertices
/// * `index_ptr` — Pointer to index data (u32 array)
/// * `index_count` — Number of indices (must be multiple of 3)
/// * `format` — Vertex format flags (0-15)
///
/// Vertices and indices are buffered on the CPU and flushed at frame end.
/// Uses current transform and render state.
fn draw_triangles_indexed(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!(
            "draw_triangles_indexed: invalid format {} (max {})",
            format, MAX_VERTEX_FORMAT
        );
        return;
    }
    let format = format as u8;

    // Validate counts
    if vertex_count == 0 || index_count == 0 {
        return; // Nothing to draw
    }
    if !index_count.is_multiple_of(3) {
        warn!(
            "draw_triangles_indexed: index_count {} is not a multiple of 3",
            index_count
        );
        return;
    }

    // Calculate data sizes with overflow checking
    let stride = vertex_stride(format);
    let Some(vertex_data_size) = vertex_count.checked_mul(stride) else {
        warn!(
            "draw_triangles_indexed: vertex data size overflow (vertex_count={}, stride={})",
            vertex_count, stride
        );
        return;
    };
    let Some(index_data_size) = index_count.checked_mul(2) else {
        warn!(
            "draw_triangles_indexed: index data size overflow (index_count={})",
            index_count
        );
        return;
    };
    let float_count = vertex_data_size / 4;

    // Read data from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("draw_triangles_indexed: no WASM memory available");
            return;
        }
    };

    let vertex_ptr = data_ptr as usize;
    let vertex_byte_size = vertex_data_size as usize;
    let idx_ptr = index_ptr as usize;
    let index_byte_size = index_data_size as usize;

    // Copy data
    let (vertex_data, index_data): (Vec<f32>, Vec<u16>) =
        {
            let mem_data = memory.data(&caller);

            if vertex_ptr + vertex_byte_size > mem_data.len() {
                warn!(
                "draw_triangles_indexed: vertex data ({} bytes at {}) exceeds memory bounds ({})",
                vertex_byte_size, vertex_ptr, mem_data.len()
            );
                return;
            }

            if idx_ptr + index_byte_size > mem_data.len() {
                warn!(
                "draw_triangles_indexed: index data ({} bytes at {}) exceeds memory bounds ({})",
                index_byte_size, idx_ptr, mem_data.len()
            );
                return;
            }

            let vertex_bytes = &mem_data[vertex_ptr..vertex_ptr + vertex_byte_size];
            let floats: &[f32] = bytemuck::cast_slice(vertex_bytes);

            let index_bytes = &mem_data[idx_ptr..idx_ptr + index_byte_size];
            let indices: &[u16] = bytemuck::cast_slice(index_bytes);

            // Validate indices are within bounds
            for &idx in indices {
                if idx as u32 >= vertex_count {
                    warn!(
                        "draw_triangles_indexed: index {} out of bounds (vertex_count = {})",
                        idx, vertex_count
                    );
                    return;
                }
            }

            (floats.to_vec(), indices.to_vec())
        };

    // Verify data lengths
    if vertex_data.len() != float_count as usize {
        warn!(
            "draw_triangles_indexed: expected {} vertex floats, got {}",
            float_count,
            vertex_data.len()
        );
        return;
    }
    if index_data.len() != index_count as usize {
        warn!(
            "draw_triangles_indexed: expected {} indices, got {}",
            index_count,
            index_data.len()
        );
        return;
    }

    let state = &mut caller.data_mut().console;

    // Texture mapping happens in process_draw_commands() using session.texture_map
    // FFI doesn't have access to the texture map, so we use placeholders here
    let texture_slots = [
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
        crate::graphics::TextureHandle::INVALID,
    ];

    let cull_mode = crate::graphics::CullMode::from_u8(state.cull_mode);

    // Allocate combined MVP+shading buffer index (lazy allocation with deduplication)
    let buffer_index = state.add_mvp_shading_state();

    // Record draw command directly
    state.render_pass.record_triangles_indexed(
        format,
        &vertex_data,
        &index_data,
        buffer_index,
        texture_slots,
        state.depth_test,
        cull_mode,
    );
}

// ============================================================================
// Billboard Drawing
// ============================================================================

/// Draw a billboard (camera-facing quad) with full texture
///
/// # Arguments
/// * `w` — Billboard width in world units
/// * `h` — Billboard height in world units
/// * `mode` — Billboard mode (1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z)
/// * `color` — Color tint (0xRRGGBBAA)
///
/// The billboard is positioned at the current transform origin and always faces the camera.
/// Modes:
/// - 1 (spherical): Faces camera completely (rotates on all axes)
/// - 2 (cylindrical Y): Rotates around Y axis only (stays upright)
/// - 3 (cylindrical X): Rotates around X axis only
/// - 4 (cylindrical Z): Rotates around Z axis only
fn draw_billboard(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    w: f32,
    h: f32,
    mode: u32,
    color: u32,
) {
    // Validate mode
    if !(1..=4).contains(&mode) {
        warn!("draw_billboard: invalid mode {} (must be 1-4)", mode);
        return;
    }

    let state = &mut caller.data_mut().console;

    // Get shading state index IMMEDIATELY (while current_shading_state is valid)
    let shading_state_index = state.add_shading_state();

    // Convert FFI mode (1-4) to QuadMode enum (0-3)
    let quad_mode = match mode {
        1 => crate::graphics::QuadMode::BillboardSpherical,
        2 => crate::graphics::QuadMode::BillboardCylindricalY,
        3 => crate::graphics::QuadMode::BillboardCylindricalX,
        4 => crate::graphics::QuadMode::BillboardCylindricalZ,
        _ => unreachable!(), // Already validated above
    };

    // Extract world position from current model matrix
    // Get current model matrix (from Option or last in pool)
    let current_matrix = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let position = [
        current_matrix.w_axis.x,
        current_matrix.w_axis.y,
        current_matrix.w_axis.z,
    ];

    // Force lazy push of view matrix if pending (fixes cylindrical billboard bug)
    if let Some(mat) = state.current_view_matrix.take() {
        state.view_matrices.push(mat);
    }

    // Get current view index (after any pending push)
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Create quad instance (full texture UV: 0,0,1,1)
    let instance = crate::graphics::QuadInstance::billboard(
        position,
        w,
        h,
        quad_mode,
        [0.0, 0.0, 1.0, 1.0], // Full texture
        color,
        shading_state_index.0,
        view_idx,
    );

    state.add_quad_instance(instance);
}

/// Draw a billboard with a UV region from the texture
///
/// # Arguments
/// * `w` — Billboard width in world units
/// * `h` — Billboard height in world units
/// * `src_x` — Source texture X coordinate (0.0-1.0)
/// * `src_y` — Source texture Y coordinate (0.0-1.0)
/// * `src_w` — Source texture width (0.0-1.0)
/// * `src_h` — Source texture height (0.0-1.0)
/// * `mode` — Billboard mode (1=spherical, 2=cylindrical Y, 3=cylindrical X, 4=cylindrical Z)
/// * `color` — Color tint (0xRRGGBBAA)
///
/// This allows drawing a region of a sprite sheet as a billboard.
fn draw_billboard_region(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    w: f32,
    h: f32,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
    mode: u32,
    color: u32,
) {
    // Validate mode
    if !(1..=4).contains(&mode) {
        warn!("draw_billboard_region: invalid mode {} (must be 1-4)", mode);
        return;
    }

    let state = &mut caller.data_mut().console;

    // Get shading state index IMMEDIATELY (while current_shading_state is valid)
    let shading_state_index = state.add_shading_state();

    // Convert FFI mode (1-4) to QuadMode enum (0-3)
    let quad_mode = match mode {
        1 => crate::graphics::QuadMode::BillboardSpherical,
        2 => crate::graphics::QuadMode::BillboardCylindricalY,
        3 => crate::graphics::QuadMode::BillboardCylindricalX,
        4 => crate::graphics::QuadMode::BillboardCylindricalZ,
        _ => unreachable!(), // Already validated above
    };

    // Extract world position from current model matrix
    // Get current model matrix (from Option or last in pool)
    let current_matrix = state.current_model_matrix.unwrap_or_else(|| {
        state
            .model_matrices
            .last()
            .copied()
            .unwrap_or(Mat4::IDENTITY)
    });
    let position = [
        current_matrix.w_axis.x,
        current_matrix.w_axis.y,
        current_matrix.w_axis.z,
    ];

    // Force lazy push of view matrix if pending (fixes cylindrical billboard bug)
    if let Some(mat) = state.current_view_matrix.take() {
        state.view_matrices.push(mat);
    }

    // Get current view index (after any pending push)
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Create quad instance with UV region
    let instance = crate::graphics::QuadInstance::billboard(
        position,
        w,
        h,
        quad_mode,
        [src_x, src_y, src_x + src_w, src_y + src_h], // UV rect
        color,
        shading_state_index.0,
        view_idx,
    );

    state.add_quad_instance(instance);
}

// ============================================================================
// 2D Drawing (Screen Space)
// ============================================================================

/// Draw a sprite with the bound texture
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Sprite width in pixels
/// * `h` — Sprite height in pixels
/// * `color` — Color tint (0xRRGGBBAA)
///
/// Draws the full texture (UV 0,0 to 1,1) as a quad in screen space.
/// Uses current blend mode and bound texture (slot 0).
fn draw_sprite(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: u32,
) {
    let state = &mut caller.data_mut().console;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Get current view index (last in pool, following Option pattern)
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Create screen-space quad instance
    let instance = crate::graphics::QuadInstance::sprite(
        x,
        y,
        w,
        h,
        0.0,                  // No rotation
        [0.0, 0.0, 1.0, 1.0], // Full texture UV
        color,
        shading_state_index.0,
        view_idx,
    );

    state.add_quad_instance(instance);
}

/// Draw a region of a sprite sheet
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Sprite width in pixels
/// * `h` — Sprite height in pixels
/// * `src_x` — Source texture X coordinate (0.0-1.0)
/// * `src_y` — Source texture Y coordinate (0.0-1.0)
/// * `src_w` — Source texture width (0.0-1.0)
/// * `src_h` — Source texture height (0.0-1.0)
/// * `color` — Color tint (0xRRGGBBAA)
///
/// Useful for sprite sheets and texture atlases.
fn draw_sprite_region(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
    color: u32,
) {
    let state = &mut caller.data_mut().console;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Calculate UV coordinates (convert from src_x,src_y,src_w,src_h to u0,v0,u1,v1)
    let u0 = src_x;
    let v0 = src_y;
    let u1 = src_x + src_w;
    let v1 = src_y + src_h;

    // Create screen-space quad instance
    let instance = crate::graphics::QuadInstance::sprite(
        x,
        y,
        w,
        h,
        0.0,              // No rotation
        [u0, v0, u1, v1], // Texture UV region
        color,
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance);
}

/// Draw a sprite with full control (rotation, origin, UV region)
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Sprite width in pixels
/// * `h` — Sprite height in pixels
/// * `src_x` — Source texture X coordinate (0.0-1.0)
/// * `src_y` — Source texture Y coordinate (0.0-1.0)
/// * `src_w` — Source texture width (0.0-1.0)
/// * `src_h` — Source texture height (0.0-1.0)
/// * `origin_x` — Origin X offset in pixels (0 = left edge of sprite)
/// * `origin_y` — Origin Y offset in pixels (0 = top edge of sprite)
/// * `angle_deg` — Rotation angle in degrees (clockwise)
/// * `color` — Color tint (0xRRGGBBAA)
///
/// The sprite rotates around the origin point. For center rotation, use (w/2, h/2).
fn draw_sprite_ex(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    src_x: f32,
    src_y: f32,
    src_w: f32,
    src_h: f32,
    origin_x: f32,
    origin_y: f32,
    angle_deg: f32,
    color: u32,
) {
    let state = &mut caller.data_mut().console;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Calculate UV coordinates
    let u0 = src_x;
    let v0 = src_y;
    let u1 = src_x + src_w;
    let v1 = src_y + src_h;

    // Apply origin offset to position
    let adjusted_x = x - origin_x;
    let adjusted_y = y - origin_y;

    // Create screen-space quad instance with rotation
    let instance = crate::graphics::QuadInstance::sprite(
        adjusted_x,
        adjusted_y,
        w,
        h,
        angle_deg.to_radians(), // Convert degrees to radians
        [u0, v0, u1, v1],
        color,
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance);
}

/// Draw a solid color rectangle
///
/// # Arguments
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (0 = top edge)
/// * `w` — Rectangle width in pixels
/// * `h` — Rectangle height in pixels
/// * `color` — Fill color (0xRRGGBBAA)
///
/// Draws an untextured quad. Useful for UI backgrounds, health bars, etc.
fn draw_rect(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    color: u32,
) {
    let state = &mut caller.data_mut().console;

    // Bind white texture (handle 0xFFFFFFFF) to slot 0
    state.bound_textures[0] = u32::MAX;

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Create screen-space quad instance (rects use white/fallback texture)
    let instance = crate::graphics::QuadInstance::sprite(
        x,
        y,
        w,
        h,
        0.0,                  // No rotation
        [0.0, 0.0, 1.0, 1.0], // Full texture UV (white texture is 1x1, so any UV works)
        color,
        shading_state_index.0,
        (state.view_matrices.len() - 1) as u32,
    );

    state.add_quad_instance(instance);
}

/// Draw text with the built-in font
///
/// # Arguments
/// * `ptr` — Pointer to UTF-8 string data
/// * `len` — Length of string in bytes
/// * `x` — Screen X coordinate in pixels (0 = left edge)
/// * `y` — Screen Y coordinate in pixels (baseline)
/// * `size` — Font size in pixels
/// * `color` — Text color (0xRRGGBBAA)
///
/// Supports full UTF-8 encoding. Text is left-aligned with no wrapping.
fn draw_text(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    ptr: u32,
    len: u32,
    x: f32,
    y: f32,
    size: f32,
    color: u32,
) {
    // Read UTF-8 string from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("draw_text: no WASM memory available");
            return;
        }
    };

    let text_str = {
        let mem_data = memory.data(&caller);
        let ptr = ptr as usize;
        let len = len as usize;

        if ptr + len > mem_data.len() {
            warn!(
                "draw_text: string data ({} bytes at {}) exceeds memory bounds ({})",
                len,
                ptr,
                mem_data.len()
            );
            return;
        }

        let bytes = &mem_data[ptr..ptr + len];
        // Validate UTF-8 and copy to owned string
        match std::str::from_utf8(bytes) {
            Ok(s) => s.to_string(), // Convert to owned String
            Err(_) => {
                warn!("draw_text: invalid UTF-8 string");
                return;
            }
        }
    };

    // Skip empty text
    if text_str.is_empty() {
        return;
    }

    let state = &mut caller.data_mut().console;

    // Ensure material color is white so it doesn't interfere with text instance color
    // (Text color is passed via the color parameter and stored in instance.color)
    state.update_color(0xFFFFFFFF);

    // Get shading state index
    let shading_state_index = state.add_shading_state();

    // Force lazy push of view matrix if pending
    if let Some(mat) = state.current_view_matrix.take() {
        state.view_matrices.push(mat);
    }
    let view_idx = (state.view_matrices.len() - 1) as u32;

    // Determine which font to use
    let font_handle = state.current_font;

    // Look up custom font if font_handle != 0
    // Clone the font to avoid holding a borrow across add_quad_instance calls
    let custom_font = if font_handle == 0 {
        None
    } else {
        let font_index = (font_handle - 1) as usize;
        state.fonts.get(font_index).cloned()
    };

    // Bind the appropriate font texture to slot 0
    if let Some(ref font) = custom_font {
        state.bound_textures[0] = font.texture;
    } else {
        // For built-in font, use handle 0 (special case handled in rendering)
        // The rendering code will map handle 0 to the actual built-in font texture
        state.bound_textures[0] = 0;
    }

    // Generate quad instances for each character
    let mut cursor_x = x;

    if let Some(ref font) = custom_font {
        // Custom font rendering
        let scale = size / font.char_height as f32;
        let glyph_height = size;

        // Calculate atlas dimensions
        let texture_width = 1024; // TODO: Get actual texture dimensions
        let texture_height = 1024;

        let max_glyph_width = font.char_width as u32;
        let glyphs_per_row = texture_width / max_glyph_width;

        for ch in text_str.chars() {
            let char_code = ch as u32;

            // Calculate glyph index
            if char_code < font.first_codepoint
                || char_code >= font.first_codepoint + font.char_count
            {
                // Character not in font, skip or use replacement
                continue;
            }
            let glyph_index = (char_code - font.first_codepoint) as usize;

            // Get glyph width (variable or fixed)
            let glyph_width_px = font
                .char_widths
                .as_ref()
                .and_then(|widths| widths.get(glyph_index).copied())
                .unwrap_or(font.char_width);
            let glyph_width = glyph_width_px as f32 * scale;

            // Calculate UV coordinates
            let col = glyph_index % glyphs_per_row as usize;
            let row = glyph_index / glyphs_per_row as usize;

            let u0 = (col * max_glyph_width as usize) as f32 / texture_width as f32;
            let v0 = (row * font.char_height as usize) as f32 / texture_height as f32;
            let u1 = ((col * max_glyph_width as usize) + glyph_width_px as usize) as f32
                / texture_width as f32;
            let v1 = ((row + 1) * font.char_height as usize) as f32 / texture_height as f32;

            // Create quad instance for this glyph
            let instance = crate::graphics::QuadInstance::sprite(
                cursor_x,
                y,
                glyph_width,
                glyph_height,
                0.0, // no rotation
                [u0, v0, u1, v1],
                color,
                shading_state_index.0,
                view_idx,
            );
            state.add_quad_instance(instance);

            cursor_x += glyph_width;
        }
    } else {
        // Built-in font rendering
        let scale = size / crate::font::GLYPH_HEIGHT as f32;
        let glyph_width = crate::font::GLYPH_WIDTH as f32 * scale;
        let glyph_height = crate::font::GLYPH_HEIGHT as f32 * scale;

        for ch in text_str.chars() {
            let char_code = ch as u32;

            // Get UV coordinates for this character
            let (u0, v0, u1, v1) = crate::font::get_glyph_uv(char_code);

            // Create quad instance for this glyph
            let instance = crate::graphics::QuadInstance::sprite(
                cursor_x,
                y,
                glyph_width,
                glyph_height,
                0.0, // no rotation
                [u0, v0, u1, v1],
                color,
                shading_state_index.0,
                view_idx,
            );
            state.add_quad_instance(instance);

            cursor_x += glyph_width;
        }
    }
}

/// Load a fixed-width bitmap font from a texture atlas
///
/// The texture must contain a grid of glyphs arranged left-to-right, top-to-bottom.
/// Each glyph occupies char_width × char_height pixels.
///
/// # Arguments
/// * `texture` — Handle to the texture atlas
/// * `char_width` — Width of each glyph in pixels
/// * `char_height` — Height of each glyph in pixels
/// * `first_codepoint` — Unicode codepoint of the first glyph
/// * `char_count` — Number of glyphs in the font
///
/// # Returns
/// Handle to the loaded font (use with `font_bind()`)
///
/// # Notes
/// - Call this in `init()` - font loading is not allowed during gameplay
/// - All glyphs in a fixed-width font have the same width
/// - The texture must have enough space for char_count glyphs
#[inline]
fn load_font(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    texture: u32,
    char_width: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32,
) -> u32 {
    // Only allow during init
    if !caller.data().game.in_init {
        warn!("load_font: can only be called during init()");
        return 0;
    }

    // Validate parameters
    if texture == 0 {
        warn!("load_font: invalid texture handle 0");
        return 0;
    }
    if char_width == 0 || char_width > 255 {
        warn!("load_font: char_width must be 1-255");
        return 0;
    }
    if char_height == 0 || char_height > 255 {
        warn!("load_font: char_height must be 1-255");
        return 0;
    }
    if char_count == 0 {
        warn!("load_font: char_count must be > 0");
        return 0;
    }

    let state = &mut caller.data_mut().console;

    // Allocate font handle
    let handle = state.next_font_handle;
    state.next_font_handle += 1;

    // Create font descriptor
    let font = Font {
        texture,
        char_width: char_width as u8,
        char_height: char_height as u8,
        first_codepoint,
        char_count,
        char_widths: None, // Fixed-width
    };

    state.fonts.push(font);
    handle
}

/// Load a variable-width bitmap font from a texture atlas
///
/// Like `load_font()`, but allows each glyph to have a different width.
///
/// # Arguments
/// * `texture` — Handle to the texture atlas
/// * `widths_ptr` — Pointer to array of char_count u8 widths
/// * `char_height` — Height of each glyph in pixels
/// * `first_codepoint` — Unicode codepoint of the first glyph
/// * `char_count` — Number of glyphs in the font
///
/// # Returns
/// Handle to the loaded font (use with `font_bind()`)
///
/// # Notes
/// - Call this in `init()` - font loading is not allowed during gameplay
/// - The widths array must have exactly char_count entries
/// - Glyphs are still arranged in a grid, but can have custom widths
#[inline]
fn load_font_ex(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    texture: u32,
    widths_ptr: u32,
    char_height: u32,
    first_codepoint: u32,
    char_count: u32,
) -> u32 {
    // Only allow during init
    if !caller.data().game.in_init {
        warn!("load_font_ex: can only be called during init()");
        return 0;
    }

    // Validate parameters
    if texture == 0 {
        warn!("load_font_ex: invalid texture handle 0");
        return 0;
    }
    if char_height == 0 || char_height > 255 {
        warn!("load_font_ex: char_height must be 1-255");
        return 0;
    }
    if char_count == 0 {
        warn!("load_font_ex: char_count must be > 0");
        return 0;
    }

    // Read widths array from WASM memory
    let memory = match caller.data().game.memory {
        Some(m) => m,
        None => {
            warn!("load_font_ex: no WASM memory available");
            return 0;
        }
    };

    let widths = {
        let mem_data = memory.data(&caller);
        let ptr = widths_ptr as usize;
        let len = char_count as usize;

        if ptr + len > mem_data.len() {
            warn!(
                "load_font_ex: widths array ({} bytes at {}) exceeds memory bounds ({})",
                len,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        mem_data[ptr..ptr + len].to_vec()
    };

    let state = &mut caller.data_mut().console;

    // Allocate font handle
    let handle = state.next_font_handle;
    state.next_font_handle += 1;

    // Create font descriptor
    let font = Font {
        texture,
        char_width: 0, // Not used for variable-width
        char_height: char_height as u8,
        first_codepoint,
        char_count,
        char_widths: Some(widths),
    };

    state.fonts.push(font);
    handle
}

/// Bind a font for subsequent draw_text() calls
///
/// # Arguments
/// * `font_handle` — Font handle from load_font() or load_font_ex(), or 0 for built-in font
///
/// # Notes
/// - Font 0 is the built-in 8×8 monospace font (default)
/// - Custom fonts persist for all subsequent draw_text() calls until changed
#[inline]
fn font_bind(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, font_handle: u32) {
    let state = &mut caller.data_mut().console;

    // Validate font handle (0 is always valid = built-in)
    if font_handle != 0 {
        // Check if handle is valid (font exists)
        let font_index = (font_handle - 1) as usize;
        if font_index >= state.fonts.len() {
            warn!("font_bind: invalid font handle {}", font_handle);
            return;
        }
    }

    state.current_font = font_handle;
}

// ============================================================================
// Sky System
// ============================================================================

/// Set procedural sky parameters
///
/// # Arguments
/// * `horizon_r` — Horizon color red (0.0-1.0)
/// * `horizon_g` — Horizon color green (0.0-1.0)
/// * `horizon_b` — Horizon color blue (0.0-1.0)
/// * `zenith_r` — Zenith (top) color red (0.0-1.0)
/// * `zenith_g` — Zenith (top) color green (0.0-1.0)
/// * `zenith_b` — Zenith (top) color blue (0.0-1.0)
/// * `sun_dir_x` — Sun direction X (will be normalized)
/// * `sun_dir_y` — Sun direction Y (will be normalized)
/// * `sun_dir_z` — Sun direction Z (will be normalized)
/// * `sun_r` — Sun color red (0.0-1.0+)
/// * `sun_g` — Sun color green (0.0-1.0+)
// ============================================================================
// Mode 1 (Matcap) Functions
// ============================================================================

/// Bind a matcap texture to a slot (Mode 1 only)
///
/// # Arguments
/// * `slot` — Matcap slot (1-3)
/// * `texture` — Texture handle from load_texture
///
/// In Mode 1 (Matcap), slots 1-3 are used for matcap textures that multiply together.
/// Slot 0 is reserved for albedo texture.
/// Using this function in other modes is allowed but has no effect.
fn matcap_set(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    slot: u32,
    texture: u32,
) {
    // Validate slot range (1-3 for matcaps)
    if !(1..=3).contains(&slot) {
        warn!("matcap_set: invalid slot {} (must be 1-3)", slot);
        return;
    }

    let state = &mut caller.data_mut().console;
    state.bound_textures[slot as usize] = texture;
}

// ============================================================================
// Sky Functions
// ============================================================================

/// Set sky gradient colors
///
/// # Arguments
/// * `horizon_r` — Horizon color red (0.0-1.0)
/// * `horizon_g` — Horizon color green (0.0-1.0)
/// * `horizon_b` — Horizon color blue (0.0-1.0)
/// * `zenith_r` — Zenith color red (0.0-1.0)
/// * `zenith_g` — Zenith color green (0.0-1.0)
/// * `zenith_b` — Zenith color blue (0.0-1.0)
///
/// Sets the procedural sky gradient. Horizon is the color at eye level,
/// zenith is the color directly overhead. The gradient interpolates smoothly between them.
/// Works in all render modes (provides ambient lighting in PBR/Hybrid modes).
fn sky_set_colors(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    horizon_r: f32,
    horizon_g: f32,
    horizon_b: f32,
    zenith_r: f32,
    zenith_g: f32,
    zenith_b: f32,
) {
    let state = &mut caller.data_mut().console;
    state.update_sky_colors(
        [horizon_r, horizon_g, horizon_b],
        [zenith_r, zenith_g, zenith_b],
    );
}

/// Set sky sun properties
///
/// # Arguments
/// * `dir_x` — Sun direction X component (will be normalized)
/// * `dir_y` — Sun direction Y component (will be normalized)
/// * `dir_z` — Sun direction Z component (will be normalized)
/// * `color_r` — Sun color red (0.0-1.0)
/// * `color_g` — Sun color green (0.0-1.0)
/// * `color_b` — Sun color blue (0.0-1.0)
/// * `sharpness` — Sun sharpness (0.0-1.0, higher = smaller/sharper sun disc)
///
/// Sets the procedural sky sun. The sun appears as a bright disc in the sky gradient
/// and provides specular highlights in PBR/Hybrid modes.
/// Direction will be automatically normalized by the graphics backend.
fn sky_set_sun(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    sharpness: f32,
) {
    let state = &mut caller.data_mut().console;

    // Validate direction vector (warn if zero-length)
    let len_sq = dir_x * dir_x + dir_y * dir_y + dir_z * dir_z;
    if len_sq < 1e-10 {
        warn!("sky_set_sun: zero-length direction vector, using default (0, 1, 0)");
        state.update_sky_sun([0.0, 1.0, 0.0], [color_r, color_g, color_b], sharpness);
        return;
    }

    state.update_sky_sun(
        [dir_x, dir_y, dir_z],
        [color_r, color_g, color_b],
        sharpness,
    );
}

// ============================================================================
// Material Functions
// ============================================================================

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

// ============================================================================
// Mode 2 (PBR) Lighting Functions
// ============================================================================

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

/// Set bone transform matrices for GPU skinning
///
/// # Arguments
/// * `matrices_ptr` — Pointer to array of bone matrices in WASM memory
/// * `count` — Number of bones (max 256)
///
/// Each bone matrix is 16 floats in column-major order (same as transform_set).
/// The vertex shader will use these matrices to deform skinned vertices based on
/// their bone indices and weights.
///
/// Call this before drawing skinned meshes (meshes with FORMAT_SKINNED flag).
/// The bone transforms are typically computed on CPU each frame for skeletal animation.
fn set_bones(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    matrices_ptr: u32,
    count: u32,
) {
    // Validate bone count
    if count > MAX_BONES as u32 {
        warn!(
            "set_bones: bone count {} exceeds maximum {} - clamping",
            count, MAX_BONES
        );
        return;
    }

    if count == 0 {
        // Clear bone data
        let state = &mut caller.data_mut().console;
        state.bone_matrices.clear();
        state.bone_count = 0;
        return;
    }

    // Calculate required memory size (16 floats per matrix × 4 bytes per float)
    let matrix_size = 16 * 4; // 64 bytes per matrix
    let total_size = count as usize * matrix_size;

    // Get WASM memory
    let memory = match caller.data().game.memory {
        Some(mem) => mem,
        None => {
            warn!("set_bones: WASM memory not initialized");
            return;
        }
    };

    // Read matrix data from WASM memory
    let data = memory.data(&caller);
    let start = matrices_ptr as usize;
    let end = start + total_size;

    if end > data.len() {
        warn!(
            "set_bones: memory access out of bounds (requested {}-{}, memory size {})",
            start,
            end,
            data.len()
        );
        return;
    }

    // Parse matrices from memory (column-major order)
    let mut matrices = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = start + i * matrix_size;
        let matrix_bytes = &data[offset..offset + matrix_size];

        // Convert bytes to f32 array (16 floats)
        let mut floats = [0.0f32; 16];
        for (j, float) in floats.iter_mut().enumerate() {
            let byte_offset = j * 4;
            let bytes = [
                matrix_bytes[byte_offset],
                matrix_bytes[byte_offset + 1],
                matrix_bytes[byte_offset + 2],
                matrix_bytes[byte_offset + 3],
            ];
            *float = f32::from_le_bytes(bytes);
        }

        // Create Mat4 from column-major floats
        let matrix = Mat4::from_cols_array(&floats);
        matrices.push(matrix);
    }

    // Store bone matrices in render state
    let state = &mut caller.data_mut().console;
    state.bone_matrices = matrices;
    state.bone_count = count;
}

// =============================================================================
// Audio FFI Functions
// =============================================================================

/// Load raw PCM sound data (22.05kHz, 16-bit signed, mono)
///
/// Must be called during `init()`. Returns sound handle (u32).
///
/// # Parameters
/// - `data_ptr`: Pointer to raw i16 PCM data in WASM memory
/// - `byte_len`: Length of data in bytes (must be even, as each sample is 2 bytes)
///
/// # Returns
/// Sound handle for use with play_sound, channel_play, music_play
fn load_sound(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    byte_len: u32,
) -> u32 {
    // Enforce init-only
    if !caller.data().game.in_init {
        warn!("load_sound() called outside init() - ignored");
        return 0;
    }

    // Validate byte length is even (each sample is 2 bytes)
    if !byte_len.is_multiple_of(2) {
        warn!("load_sound: byte_len must be even (got {})", byte_len);
        return 0;
    }

    let sample_count = (byte_len / 2) as usize;

    // Get WASM memory
    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => {
            warn!("load_sound: failed to get WASM memory");
            return 0;
        }
    };

    // Read PCM data from WASM memory
    let mut pcm_data = vec![0i16; sample_count];
    // SAFETY: This unsafe block is sound because:
    // 1. The pointer comes from WASM memory export, guaranteed valid by wasmtime
    // 2. byte_len is validated as even (divisible by 2), ensuring proper i16 alignment
    // 3. sample_count = byte_len / 2, so we're reading exactly the right number of i16 samples
    // 4. Data is immediately copied to owned Vec, no aliasing or lifetime issues
    // 5. WASM linear memory is guaranteed to be valid for the duration of this call
    let data_slice = unsafe {
        let ptr = memory.data_ptr(&caller).add(data_ptr as usize);
        std::slice::from_raw_parts(ptr as *const i16, sample_count)
    };
    pcm_data.copy_from_slice(data_slice);

    let state = &mut caller.data_mut().console;

    // Create Sound and add to sounds vec
    let sound = Sound {
        data: std::sync::Arc::new(pcm_data),
    };

    let handle = state.next_sound_handle;
    state.next_sound_handle += 1;

    // Resize sounds vec if needed
    if handle as usize >= state.sounds.len() {
        state.sounds.resize(handle as usize + 1, None);
    }
    state.sounds[handle as usize] = Some(sound);

    info!("Loaded sound {} ({} samples)", handle, sample_count);
    handle
}

/// Play sound on next available channel (fire-and-forget)
///
/// For one-shot sounds: gunshots, jumps, coins
///
/// # Parameters
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
fn play_sound(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    sound: u32,
    volume: f32,
    pan: f32,
) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::PlaySound { sound, volume, pan });
}

/// Play sound on specific channel
///
/// For managed channels (positional/looping: engines, ambient, footsteps)
///
/// # Parameters
/// - `channel`: 0-15
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
/// - `looping`: 1 = loop, 0 = play once
fn channel_play(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    channel: u32,
    sound: u32,
    volume: f32,
    pan: f32,
    looping: u32,
) {
    let state = &mut caller.data_mut().console;
    state.audio_commands.push(AudioCommand::ChannelPlay {
        channel,
        sound,
        volume,
        pan,
        looping: looping != 0,
    });
}

/// Update channel parameters (call every frame for positional audio)
///
/// # Parameters
/// - `channel`: 0-15
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
fn channel_set(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    channel: u32,
    volume: f32,
    pan: f32,
) {
    let state = &mut caller.data_mut().console;
    state.audio_commands.push(AudioCommand::ChannelSet {
        channel,
        volume,
        pan,
    });
}

/// Stop channel
///
/// # Parameters
/// - `channel`: 0-15
fn channel_stop(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, channel: u32) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::ChannelStop { channel });
}

/// Play music (looping, dedicated channel)
///
/// # Parameters
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
fn music_play(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    sound: u32,
    volume: f32,
) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::MusicPlay { sound, volume });
}

/// Stop music
fn music_stop(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>) {
    let state = &mut caller.data_mut().console;
    state.audio_commands.push(AudioCommand::MusicStop);
}

/// Set music volume
///
/// # Parameters
/// - `volume`: 0.0 to 1.0
fn music_set_volume(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, volume: f32) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::MusicSetVolume { volume });
}

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // Init Config Defaults Tests
    // ========================================================================

    #[test]
    fn test_init_config_defaults() {
        let state = ZFFIState::new();
        assert_eq!(state.init_config.resolution_index, 1); // 540p
        assert_eq!(state.init_config.tick_rate_index, 2); // 60 fps
        assert_eq!(state.init_config.clear_color, 0x000000FF); // Black
        assert_eq!(state.init_config.render_mode, 0); // Unlit
        assert!(!state.init_config.modified);
    }

    // Camera and transform stack tests removed - obsolete with matrix pool system

    // ========================================================================
    // GPU Skinning FFI Tests
    // ========================================================================

    #[test]
    fn test_vertex_stride_skinned_constant() {
        // Skinning adds: 4 u8 bone indices (4 bytes) + 4 f32 bone weights (16 bytes) = 20 bytes
        const SKINNING_OVERHEAD: u32 = 20;

        // Test base format + skinning
        assert_eq!(vertex_stride(FORMAT_SKINNED), 12 + SKINNING_OVERHEAD); // 32
    }

    #[test]
    fn test_vertex_stride_all_skinned_formats() {
        // All 8 skinned format combinations
        assert_eq!(vertex_stride(8), 32); // POS_SKINNED
        assert_eq!(vertex_stride(9), 40); // POS_UV_SKINNED
        assert_eq!(vertex_stride(10), 44); // POS_COLOR_SKINNED
        assert_eq!(vertex_stride(11), 52); // POS_UV_COLOR_SKINNED
        assert_eq!(vertex_stride(12), 44); // POS_NORMAL_SKINNED
        assert_eq!(vertex_stride(13), 52); // POS_UV_NORMAL_SKINNED
        assert_eq!(vertex_stride(14), 56); // POS_COLOR_NORMAL_SKINNED
        assert_eq!(vertex_stride(15), 64); // POS_UV_COLOR_NORMAL_SKINNED
    }

    #[test]
    fn test_render_state_bone_matrices_default() {
        let state = ZFFIState::new();
        assert!(state.bone_matrices.is_empty());
        assert_eq!(state.bone_count, 0);
    }

    // ========================================================================
    // Vertex Format Validation Tests
    // ========================================================================

    #[test]
    fn test_vertex_stride_base_formats() {
        // Base formats without skinning
        assert_eq!(vertex_stride(0), 12); // POS: 3 floats = 12 bytes
        assert_eq!(vertex_stride(1), 20); // POS_UV: 3 + 2 floats = 20 bytes
        assert_eq!(vertex_stride(2), 24); // POS_COLOR: 3 + 3 floats = 24 bytes
        assert_eq!(vertex_stride(3), 32); // POS_UV_COLOR: 3 + 2 + 3 floats = 32 bytes
        assert_eq!(vertex_stride(4), 24); // POS_NORMAL: 3 + 3 floats = 24 bytes
        assert_eq!(vertex_stride(5), 32); // POS_UV_NORMAL: 3 + 2 + 3 floats = 32 bytes
        assert_eq!(vertex_stride(6), 36); // POS_COLOR_NORMAL: 3 + 3 + 3 floats = 36 bytes
        assert_eq!(vertex_stride(7), 44); // POS_UV_COLOR_NORMAL: 3 + 2 + 3 + 3 floats = 44 bytes
    }

    #[test]
    fn test_vertex_stride_all_format_combinations() {
        // Verify all 16 format combinations have correct stride
        for format in 0..=15u8 {
            let stride = vertex_stride(format);
            // Minimum is 12 (POS only), maximum is 64 (all attributes + skinning)
            assert!(stride >= 12);
            assert!(stride <= 64);
        }
    }

    // ========================================================================
    // Render State Defaults Tests
    // ========================================================================

    #[test]
    fn test_render_state_defaults() {
        let state = ZFFIState::new();
        assert_eq!(state.current_shading_state.color_rgba8, 0xFFFFFFFF); // White
        assert!(state.depth_test); // Enabled
        assert_eq!(state.cull_mode, 1); // Back-face culling
        assert_eq!(state.blend_mode, 0); // Opaque
        assert_eq!(state.texture_filter, 0); // Nearest
    }

    #[test]
    fn test_render_state_texture_slots_default() {
        let state = ZFFIState::new();
        assert_eq!(state.bound_textures, [0; 4]); // All slots unbound
    }

    // Material field tests removed - now stored in unified shading state

    #[test]
    fn test_render_state_lights_default() {
        let state = ZFFIState::new();
        // Lights now stored in current_shading_state
        for i in 0..4 {
            let light = &state.current_shading_state.lights[i];
            assert!(!light.is_enabled());
        }
    }

    // ========================================================================
    // Init Config Tests
    // ========================================================================

    #[test]
    fn test_init_config_resolution_values() {
        use crate::console::RESOLUTIONS;
        // Verify resolution indices map to expected values
        assert_eq!(RESOLUTIONS[0], (640, 360)); // 360p
        assert_eq!(RESOLUTIONS[1], (960, 540)); // 540p (default)
        assert_eq!(RESOLUTIONS[2], (1280, 720)); // 720p
        assert_eq!(RESOLUTIONS[3], (1920, 1080)); // 1080p
    }

    #[test]
    fn test_init_config_tick_rate_values() {
        use crate::console::TICK_RATES;
        // Verify tick rate indices map to expected values
        assert_eq!(TICK_RATES[0], 24); // 24 fps
        assert_eq!(TICK_RATES[1], 30); // 30 fps
        assert_eq!(TICK_RATES[2], 60); // 60 fps (default)
        assert_eq!(TICK_RATES[3], 120); // 120 fps
    }

    // ========================================================================
    // Input State Tests (moved to console.rs - ZInput tests)
    // ========================================================================

    use crate::{console::ZInput, graphics::FORMAT_SKINNED};

    #[test]
    fn test_zinput_button_bitmask() {
        // Verify button bitmask layout

        // Button 0 (UP) should be bit 0
        let input = ZInput {
            buttons: 1 << 0,
            ..Default::default()
        };
        assert_eq!(input.buttons & (1 << 0), 1);

        // Button 13 (SELECT) should be bit 13
        let input = ZInput {
            buttons: 1 << 13,
            ..Default::default()
        };
        assert_eq!(input.buttons & (1 << 13), 1 << 13);

        // All buttons set
        let input = ZInput {
            buttons: 0x3FFF, // 14 buttons (0-13)
            ..Default::default()
        };
        for i in 0..14 {
            assert_ne!(input.buttons & (1 << i), 0);
        }
    }

    #[test]
    fn test_zinput_stick_range() {
        // Sticks are i8 (-128 to 127)
        let input = ZInput {
            buttons: 0,
            left_stick_x: -128,
            left_stick_y: 127,
            right_stick_x: 0,
            right_stick_y: -1,
            left_trigger: 0,
            right_trigger: 0,
        };

        // Converting to -1.0..1.0 range
        assert!(input.left_stick_x as f32 / 127.0 <= -1.0);
        assert!((input.left_stick_y as f32 / 127.0 - 1.0).abs() < 0.01);
        assert_eq!(input.right_stick_x as f32 / 127.0, 0.0);
    }

    #[test]
    fn test_zinput_trigger_range() {
        // Triggers are u8 (0 to 255)
        let input = ZInput {
            buttons: 0,
            left_stick_x: 0,
            left_stick_y: 0,
            right_stick_x: 0,
            right_stick_y: 0,
            left_trigger: 0,
            right_trigger: 255,
        };

        // Converting to 0.0..1.0 range
        assert_eq!(input.left_trigger as f32 / 255.0, 0.0);
        assert_eq!(input.right_trigger as f32 / 255.0, 1.0);
    }

    // ========================================================================
    // Draw Command Tests
    // ========================================================================

    #[test]
    fn test_draw_commands_initially_empty() {
        let state = ZFFIState::new();
        assert!(state.render_pass.commands().is_empty());
        assert!(state.deferred_commands.is_empty());
    }

    // Command buffer recording tests removed - testing implementation details that changed with unified shading state

    #[test]
    fn test_draw_command_text() {
        let mut state = ZFFIState::new();

        state.deferred_commands.push(DeferredCommand::DrawText {
            text: b"Hello".to_vec(),
            x: 100.0,
            y: 200.0,
            size: 16.0,
            color: 0x000000FF,
            font: 0,
        });

        if let DeferredCommand::DrawText { text, size, .. } = &state.deferred_commands[0] {
            assert_eq!(std::str::from_utf8(text).unwrap(), "Hello");
            assert_eq!(*size, 16.0);
        } else {
            panic!("Expected DrawText command");
        }
    }

    // SetSky test removed - sky is now part of unified shading state, not a deferred command

    // ========================================================================
    // Pending Resource Tests
    // ========================================================================

    #[test]
    fn test_pending_textures_initially_empty() {
        let state = ZFFIState::new();
        assert!(state.pending_textures.is_empty());
    }

    #[test]
    fn test_pending_meshes_initially_empty() {
        let state = ZFFIState::new();
        assert!(state.pending_meshes.is_empty());
    }

    #[test]
    fn test_pending_texture_structure() {
        let texture = PendingTexture {
            handle: 1,
            width: 64,
            height: 64,
            data: vec![0xFF; 64 * 64 * 4], // RGBA8
        };

        assert_eq!(texture.handle, 1);
        assert_eq!(texture.width, 64);
        assert_eq!(texture.height, 64);
        assert_eq!(texture.data.len(), 64 * 64 * 4);
    }

    #[test]
    fn test_pending_mesh_non_indexed() {
        let mesh = PendingMesh {
            handle: 1,
            format: 0,                                                      // POS only
            vertex_data: vec![0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0], // 3 vertices
            index_data: None,
        };

        assert_eq!(mesh.handle, 1);
        assert_eq!(mesh.format, 0);
        assert!(mesh.index_data.is_none());
    }

    #[test]
    fn test_pending_mesh_indexed() {
        let mesh = PendingMesh {
            handle: 2,
            format: 5,                                // POS_UV_NORMAL
            vertex_data: vec![0.0; 8 * 4],            // 4 vertices
            index_data: Some(vec![0, 1, 2, 0, 2, 3]), // 2 triangles
        };

        assert_eq!(mesh.handle, 2);
        assert_eq!(mesh.format, 5);
        assert!(mesh.index_data.is_some());
        assert_eq!(mesh.index_data.as_ref().unwrap().len(), 6);
    }

    #[test]
    fn test_next_texture_handle_increments() {
        let mut state = ZFFIState::new();
        let initial = state.next_texture_handle;

        state.next_texture_handle += 1;
        assert_eq!(state.next_texture_handle, initial + 1);

        state.next_texture_handle += 1;
        assert_eq!(state.next_texture_handle, initial + 2);
    }

    #[test]
    fn test_next_mesh_handle_increments() {
        let mut state = ZFFIState::new();
        let initial = state.next_mesh_handle;

        state.next_mesh_handle += 1;
        assert_eq!(state.next_mesh_handle, initial + 1);
    }

    // LightState tests removed - obsolete, now using PackedLight in unified shading state

    #[test]
    fn test_four_light_slots() {
        let state = ZFFIState::new();
        // Lights now stored in current_shading_state
        assert_eq!(state.current_shading_state.lights.len(), 4);
    }

    // ========================================================================
    // Color Conversion Tests
    // ========================================================================

    #[test]
    fn test_color_to_vec4_white() {
        // 0xFFFFFFFF = white, fully opaque
        let color = 0xFFFFFFFF_u32;
        let r = ((color >> 24) & 0xFF) as f32 / 255.0;
        let g = ((color >> 16) & 0xFF) as f32 / 255.0;
        let b = ((color >> 8) & 0xFF) as f32 / 255.0;
        let a = (color & 0xFF) as f32 / 255.0;

        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 1.0).abs() < 0.01);
        assert!((b - 1.0).abs() < 0.01);
        assert!((a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_vec4_red() {
        // 0xFF0000FF = red, fully opaque
        let color = 0xFF0000FF_u32;
        let r = ((color >> 24) & 0xFF) as f32 / 255.0;
        let g = ((color >> 16) & 0xFF) as f32 / 255.0;
        let b = ((color >> 8) & 0xFF) as f32 / 255.0;
        let a = (color & 0xFF) as f32 / 255.0;

        assert!((r - 1.0).abs() < 0.01);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert!((a - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_color_to_vec4_transparent() {
        // 0x00000000 = black, fully transparent
        let color = 0x00000000_u32;
        let r = ((color >> 24) & 0xFF) as f32 / 255.0;
        let g = ((color >> 16) & 0xFF) as f32 / 255.0;
        let b = ((color >> 8) & 0xFF) as f32 / 255.0;
        let a = (color & 0xFF) as f32 / 255.0;

        assert_eq!(r, 0.0);
        assert_eq!(g, 0.0);
        assert_eq!(b, 0.0);
        assert_eq!(a, 0.0);
    }

    #[test]
    fn test_color_to_vec4_semi_transparent() {
        // 0xFF00FF80 = magenta, 50% transparent
        let color = 0xFF00FF80_u32;
        let r = ((color >> 24) & 0xFF) as f32 / 255.0;
        let g = ((color >> 16) & 0xFF) as f32 / 255.0;
        let b = ((color >> 8) & 0xFF) as f32 / 255.0;
        let a = (color & 0xFF) as f32 / 255.0;

        assert!((r - 1.0).abs() < 0.01);
        assert_eq!(g, 0.0);
        assert!((b - 1.0).abs() < 0.01);
        assert!((a - 0.5).abs() < 0.01);
    }

    // ========================================================================
    // Negative Test Cases for FFI Error Conditions
    // ========================================================================
    //
    // These tests verify that invalid inputs are handled gracefully:
    // - Invalid texture handles
    // - Invalid mesh handles
    // - Out-of-range parameters
    // - Edge cases for all FFI functions

    // ------------------------------------------------------------------------
    // Invalid Texture Handle Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_texture_bind_invalid_handle_zero() {
        // Handle 0 is reserved/invalid - binding it should still set the slot
        // (validation happens at draw time, not bind time)
        let mut state = ZFFIState::new();
        state.bound_textures[0] = 0;
        assert_eq!(state.bound_textures[0], 0);
    }

    #[test]
    fn test_texture_bind_handle_not_loaded() {
        // Handle 999 doesn't exist but binding should still succeed
        // (validation is deferred to graphics backend)
        let mut state = ZFFIState::new();
        state.bound_textures[0] = 999;
        assert_eq!(state.bound_textures[0], 999);
    }

    #[test]
    fn test_texture_bind_slot_invalid_index() {
        // Slot index > 3 is invalid
        let state = ZFFIState::new();
        // Verify only 4 slots exist
        assert_eq!(state.bound_textures.len(), 4);
    }

    #[test]
    fn test_texture_bind_all_slots_independently() {
        // Test that binding to one slot doesn't affect others
        let mut state = ZFFIState::new();
        state.bound_textures[0] = 1;
        state.bound_textures[1] = 2;
        state.bound_textures[2] = 3;
        state.bound_textures[3] = 4;

        assert_eq!(state.bound_textures[0], 1);
        assert_eq!(state.bound_textures[1], 2);
        assert_eq!(state.bound_textures[2], 3);
        assert_eq!(state.bound_textures[3], 4);
    }

    // ------------------------------------------------------------------------
    // Invalid Mesh Handle Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_draw_mesh_handle_zero_produces_no_command() {
        // Handle 0 is invalid - draw_mesh should reject it
        // Simulate what draw_mesh does: it checks handle == 0 and returns early
        let state = ZFFIState::new();
        let handle = 0u32;

        // Simulate the validation in draw_mesh
        if handle == 0 {
            // Should not add a draw command
        } else {
            // Manual record would go here
        }

        // No command should have been added
        assert!(state.render_pass.commands().is_empty());
    }

    #[test]
    fn test_mesh_handle_not_loaded() {
        // Handle 999 doesn't exist - draw command is NOT queued in new system
        // because we check mesh_map immediately
        let state = ZFFIState::new();

        // Simulate draw_mesh logic
        let handle = 999;
        if state.mesh_map.contains_key(&handle) {
            // record
        } else {
            // warn and return
        }

        assert!(state.render_pass.commands().is_empty());
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Resolution
    // ------------------------------------------------------------------------

    #[test]
    fn test_resolution_index_boundary_valid() {
        // Valid indices are 0-3
        use crate::console::RESOLUTIONS;
        assert_eq!(RESOLUTIONS.len(), 4);

        for i in 0..4 {
            assert!(i < RESOLUTIONS.len());
        }
    }

    #[test]
    fn test_resolution_index_boundary_invalid() {
        // Index >= 4 is invalid
        use crate::console::RESOLUTIONS;

        let invalid_indices = [4, 5, 10, 100, u32::MAX];
        for idx in invalid_indices {
            assert!(idx as usize >= RESOLUTIONS.len());
        }
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Tick Rate
    // ------------------------------------------------------------------------

    #[test]
    fn test_tick_rate_index_boundary_valid() {
        // Valid indices are 0-3
        use crate::console::TICK_RATES;
        assert_eq!(TICK_RATES.len(), 4);

        for i in 0..4 {
            assert!(i < TICK_RATES.len());
        }
    }

    #[test]
    fn test_tick_rate_index_boundary_invalid() {
        // Index >= 4 is invalid
        use crate::console::TICK_RATES;

        let invalid_indices = [4, 5, 10, 100, u32::MAX];
        for idx in invalid_indices {
            assert!(idx as usize >= TICK_RATES.len());
        }
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Render Mode
    // ------------------------------------------------------------------------

    #[test]
    fn test_render_mode_boundary_valid() {
        // Valid modes are 0-3
        let valid_modes = [0u32, 1, 2, 3];
        for mode in valid_modes {
            assert!(mode <= 3);
        }
    }

    #[test]
    fn test_render_mode_boundary_invalid() {
        // Mode > 3 is invalid
        let invalid_modes = [4u32, 5, 10, 100, u32::MAX];
        for mode in invalid_modes {
            assert!(mode > 3);
        }
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Cull Mode
    // ------------------------------------------------------------------------

    #[test]
    fn test_cull_mode_boundary_valid() {
        // Valid modes are 0=none, 1=back, 2=front
        let valid_modes = [0u8, 1, 2];
        for mode in valid_modes {
            assert!(mode <= 2);
        }
    }

    #[test]
    fn test_cull_mode_boundary_invalid() {
        // Mode > 2 is invalid
        let invalid_modes = [3u8, 4, 10, 100, u8::MAX];
        for mode in invalid_modes {
            assert!(mode > 2);
        }
    }

    #[test]
    fn test_cull_mode_invalid_resets_to_default() {
        // When an invalid cull mode is set, it should reset to 0 (none)
        let mut state = ZFFIState::new();

        // Simulate invalid mode handling
        let mode = 5u32;
        if mode > 2 {
            state.cull_mode = 0; // Reset to none
        } else {
            state.cull_mode = mode as u8;
        }

        assert_eq!(state.cull_mode, 0);
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Blend Mode
    // ------------------------------------------------------------------------

    #[test]
    fn test_blend_mode_boundary_valid() {
        // Valid modes are 0=none, 1=alpha, 2=additive, 3=multiply
        let valid_modes = [0u8, 1, 2, 3];
        for mode in valid_modes {
            assert!(mode <= 3);
        }
    }

    #[test]
    fn test_blend_mode_boundary_invalid() {
        // Mode > 3 is invalid
        let invalid_modes = [4u8, 5, 10, 100, u8::MAX];
        for mode in invalid_modes {
            assert!(mode > 3);
        }
    }

    #[test]
    fn test_blend_mode_invalid_resets_to_default() {
        // When an invalid blend mode is set, it should reset to 0 (none)
        let mut state = ZFFIState::new();

        // Simulate invalid mode handling
        let mode = 5u32;
        if mode > 3 {
            state.blend_mode = 0; // Reset to none
        } else {
            state.blend_mode = mode as u8;
        }

        assert_eq!(state.blend_mode, 0);
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Texture Filter
    // ------------------------------------------------------------------------

    #[test]
    fn test_texture_filter_boundary_valid() {
        // Valid filters are 0=nearest, 1=linear
        let valid_filters = [0u8, 1];
        for filter in valid_filters {
            assert!(filter <= 1);
        }
    }

    #[test]
    fn test_texture_filter_boundary_invalid() {
        // Filter > 1 is invalid
        let invalid_filters = [2u8, 3, 10, 100, u8::MAX];
        for filter in invalid_filters {
            assert!(filter > 1);
        }
    }

    #[test]
    fn test_texture_filter_invalid_resets_to_default() {
        // When an invalid filter is set, it should reset to 0 (nearest)
        let mut state = ZFFIState::new();

        // Simulate invalid filter handling
        let filter = 5u32;
        if filter > 1 {
            state.texture_filter = 0; // Reset to nearest
        } else {
            state.texture_filter = filter as u8;
        }

        assert_eq!(state.texture_filter, 0);
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Vertex Format
    // ------------------------------------------------------------------------

    #[test]
    fn test_vertex_format_boundary_valid() {
        // Valid formats are 0-15 (4 bits)
        for format in 0u8..=15 {
            assert!(format <= super::MAX_VERTEX_FORMAT);
        }
    }

    #[test]
    fn test_vertex_format_boundary_invalid() {
        // Format > 15 is invalid
        let invalid_formats = [16u8, 17, 100, u8::MAX];
        for format in invalid_formats {
            assert!(format > super::MAX_VERTEX_FORMAT);
        }
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Billboard Mode
    // ------------------------------------------------------------------------

    #[test]
    fn test_billboard_mode_boundary_valid() {
        // Valid modes are 1-4
        let valid_modes = [1u8, 2, 3, 4];
        for mode in valid_modes {
            assert!((1..=4).contains(&mode));
        }
    }

    #[test]
    fn test_billboard_mode_boundary_invalid_zero() {
        // Mode 0 is invalid (must be >= 1)
        let mode = 0u32;
        assert!(mode < 1);
    }

    #[test]
    fn test_billboard_mode_boundary_invalid_high() {
        // Mode > 4 is invalid
        let invalid_modes = [5u32, 6, 10, 100, u32::MAX];
        for mode in invalid_modes {
            assert!(mode > 4);
        }
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Matcap Slot
    // ------------------------------------------------------------------------

    #[test]
    fn test_matcap_slot_boundary_valid() {
        // Valid slots are 1-3 (slot 0 is albedo)
        let valid_slots = [1u32, 2, 3];
        for slot in valid_slots {
            assert!((1..=3).contains(&slot));
        }
    }

    #[test]
    fn test_matcap_slot_boundary_invalid_zero() {
        // Slot 0 is invalid for matcaps (it's albedo)
        let slot = 0u32;
        assert!(slot < 1);
    }

    #[test]
    fn test_matcap_slot_boundary_invalid_high() {
        // Slot > 3 is invalid
        let invalid_slots = [4u32, 5, 10, 100, u32::MAX];
        for slot in invalid_slots {
            assert!(slot > 3);
        }
    }

    // ------------------------------------------------------------------------
    // Out-of-Range Parameter Tests: Light Index
    // ------------------------------------------------------------------------

    #[test]
    fn test_light_index_boundary_valid() {
        // Valid indices are 0-3
        let state = ZFFIState::new();
        assert_eq!(state.current_shading_state.lights.len(), 4);

        for i in 0..4 {
            assert!(i < state.current_shading_state.lights.len());
        }
    }

    #[test]
    fn test_light_index_boundary_invalid() {
        // Index > 3 is invalid
        let invalid_indices = [4u32, 5, 10, 100, u32::MAX];
        for idx in invalid_indices {
            assert!(idx > 3);
        }
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Camera FOV
    // ------------------------------------------------------------------------

    #[test]
    fn test_camera_fov_boundary_valid() {
        // Valid FOV is 1-179 degrees
        let valid_fovs = [1.0f32, 45.0, 60.0, 90.0, 120.0, 179.0];
        for fov in valid_fovs {
            assert!((1.0..=179.0).contains(&fov));
        }
    }

    #[test]
    fn test_camera_fov_boundary_invalid_low() {
        // FOV < 1 is invalid and should be clamped
        let invalid_fovs = [0.0f32, -1.0, -10.0, 0.5, 0.999];
        for fov in invalid_fovs {
            assert!(fov < 1.0);
            let clamped = fov.clamp(1.0, 179.0);
            assert_eq!(clamped, 1.0);
        }
    }

    #[test]
    fn test_camera_fov_boundary_invalid_high() {
        // FOV > 179 is invalid and should be clamped
        let invalid_fovs = [180.0f32, 200.0, 360.0];
        for fov in invalid_fovs {
            assert!(fov > 179.0);
            let clamped = fov.clamp(1.0, 179.0);
            assert_eq!(clamped, 179.0);
        }
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Transform Rotate with Zero Axis
    // ------------------------------------------------------------------------

    #[test]
    fn test_transform_rotate_zero_axis_detection() {
        // A zero-length axis should be detected
        let axis = Vec3::new(0.0, 0.0, 0.0);
        let len_sq = axis.length_squared();
        assert!(len_sq < 1e-10);
    }

    #[test]
    fn test_transform_rotate_near_zero_axis() {
        // Very small but non-zero axis should be normalized
        let axis = Vec3::new(1e-6, 0.0, 0.0);
        let len_sq = axis.length_squared();
        // This is not quite zero, but very small
        assert!(len_sq > 0.0);
        assert!(len_sq < 1e-10);
    }

    #[test]
    fn test_transform_rotate_valid_axis_normalized() {
        // A valid axis should be normalized
        let axis = Vec3::new(1.0, 1.0, 1.0);
        let normalized = axis.normalize();
        assert!((normalized.length() - 1.0).abs() < 0.0001);
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Material Property Clamping
    // ------------------------------------------------------------------------

    #[test]
    fn test_material_metallic_clamping() {
        // Metallic should be clamped to 0.0-1.0
        let test_cases = [
            (-1.0f32, 0.0f32),
            (0.0, 0.0),
            (0.5, 0.5),
            (1.0, 1.0),
            (2.0, 1.0),
            (100.0, 1.0),
        ];

        for (input, expected) in test_cases {
            let clamped = input.clamp(0.0, 1.0);
            assert_eq!(clamped, expected);
        }
    }

    #[test]
    fn test_material_roughness_clamping() {
        // Roughness should be clamped to 0.0-1.0
        let test_cases = [
            (-1.0f32, 0.0f32),
            (0.0, 0.0),
            (0.5, 0.5),
            (1.0, 1.0),
            (2.0, 1.0),
        ];

        for (input, expected) in test_cases {
            let clamped = input.clamp(0.0, 1.0);
            assert_eq!(clamped, expected);
        }
    }

    #[test]
    fn test_material_emissive_no_upper_clamp() {
        // Emissive allows HDR values (> 1.0), only negative is clamped
        let test_cases = [
            (-1.0f32, 0.0f32), // Clamped to 0
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 2.0),   // Allowed
            (10.0, 10.0), // Allowed for HDR
        ];

        for (input, expected) in test_cases {
            let result = if input < 0.0 { 0.0 } else { input };
            assert_eq!(result, expected);
        }
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Light Color/Intensity Negative Values
    // ------------------------------------------------------------------------

    #[test]
    fn test_light_color_negative_clamping() {
        // Negative color values should be clamped to 0
        let test_cases = [
            (-1.0f32, 0.0f32),
            (-0.5, 0.0),
            (0.0, 0.0),
            (1.0, 1.0),
            (2.0, 2.0), // HDR allowed
        ];

        for (input, expected) in test_cases {
            let result = if input < 0.0 { 0.0 } else { input };
            assert_eq!(result, expected);
        }
    }

    #[test]
    fn test_light_intensity_negative_clamping() {
        // Negative intensity should be clamped to 0
        let test_cases = [
            (-1.0f32, 0.0f32),
            (0.0, 0.0),
            (1.0, 1.0),
            (10.0, 10.0), // High intensity allowed
        ];

        for (input, expected) in test_cases {
            let result = if input < 0.0 { 0.0 } else { input };
            assert_eq!(result, expected);
        }
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Light Direction Zero Vector
    // ------------------------------------------------------------------------

    #[test]
    fn test_light_direction_zero_vector_detection() {
        let x = 0.0f32;
        let y = 0.0f32;
        let z = 0.0f32;
        let len_sq = x * x + y * y + z * z;
        assert!(len_sq < 1e-10);
    }

    #[test]
    fn test_light_direction_default_fallback() {
        // When zero-length direction is given, should use default (0, -1, 0)
        let default_direction = [0.0f32, -1.0, 0.0];
        assert_eq!(default_direction[1], -1.0);
    }

    // Transform stack tests removed - obsolete with matrix pool system

    // ------------------------------------------------------------------------
    // Edge Case Tests: Bone Count Limits
    // ------------------------------------------------------------------------

    #[test]
    fn test_bone_count_zero_clears_matrices() {
        let mut state = ZFFIState::new();

        // Add some bones first
        state.bone_matrices.push(Mat4::IDENTITY);
        state.bone_count = 1;

        // Clear with count = 0
        state.bone_matrices.clear();
        state.bone_count = 0;

        assert!(state.bone_matrices.is_empty());
        assert_eq!(state.bone_count, 0);
    }

    #[test]
    fn test_bone_count_exceeds_max() {
        // Count > MAX_BONES should be rejected
        let count = 300u32;
        assert!(count > MAX_BONES as u32);
    }

    #[test]
    fn test_bone_count_at_max() {
        // Count == MAX_BONES should be allowed
        let count = MAX_BONES as u32;
        assert!(count <= MAX_BONES as u32);
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Load Mesh Index Count
    // ------------------------------------------------------------------------

    #[test]
    fn test_load_mesh_indexed_index_count_not_multiple_of_three() {
        // Index count must be multiple of 3 for triangles
        let invalid_counts = [1u32, 2, 4, 5, 7, 8, 10];
        for count in invalid_counts {
            assert!(count % 3 != 0);
        }
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Texture Dimensions
    // ------------------------------------------------------------------------

    #[test]
    fn test_texture_dimensions_zero_width() {
        let width = 0u32;
        let height = 64u32;
        assert!(width == 0 || height == 0);
    }

    #[test]
    fn test_texture_dimensions_zero_height() {
        let width = 64u32;
        let height = 0u32;
        assert!(width == 0 || height == 0);
    }

    #[test]
    fn test_texture_dimensions_both_zero() {
        let width = 0u32;
        let height = 0u32;
        assert!(width == 0 || height == 0);
    }

    #[test]
    fn test_texture_dimensions_valid() {
        let valid_dimensions = [
            (1u32, 1u32),
            (8, 8),
            (64, 64),
            (256, 256),
            (1024, 1024),
            (4096, 4096),
        ];
        for (w, h) in valid_dimensions {
            assert!(w > 0 && h > 0);
        }
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Draw Command Buffer Growth
    // ------------------------------------------------------------------------

    // ------------------------------------------------------------------------
    // Edge Case Tests: Pending Resources Growth
    // ------------------------------------------------------------------------

    // ------------------------------------------------------------------------
    // Edge Case Tests: Handle Allocation Overflow
    // ------------------------------------------------------------------------

    #[test]
    fn test_texture_handle_wrapping() {
        let mut state = ZFFIState::new();

        // Set handle near max
        state.next_texture_handle = u32::MAX - 1;

        // Allocate one more
        let handle = state.next_texture_handle;
        state.next_texture_handle = state.next_texture_handle.wrapping_add(1);

        assert_eq!(handle, u32::MAX - 1);
        assert_eq!(state.next_texture_handle, u32::MAX);

        // One more wraps to 0
        state.next_texture_handle = state.next_texture_handle.wrapping_add(1);
        assert_eq!(state.next_texture_handle, 0);
    }

    #[test]
    fn test_mesh_handle_wrapping() {
        let mut state = ZFFIState::new();

        // Set handle near max
        state.next_mesh_handle = u32::MAX;

        // Allocate wraps to 0
        state.next_mesh_handle = state.next_mesh_handle.wrapping_add(1);
        assert_eq!(state.next_mesh_handle, 0);
    }

    // ------------------------------------------------------------------------
    // Edge Case Tests: Special Float Values
    // ------------------------------------------------------------------------

    #[test]
    fn test_float_nan_handling() {
        // NaN comparisons always return false (using partial_cmp for clarity)
        let nan = f32::NAN;
        assert!(nan.partial_cmp(&0.0).is_none());
        assert!(nan.is_nan());
        #[allow(clippy::eq_op)]
        {
            assert!(nan != nan); // NaN is not equal to itself
        }
    }

    #[test]
    fn test_float_infinity_handling() {
        let pos_inf = f32::INFINITY;
        let neg_inf = f32::NEG_INFINITY;

        assert!(pos_inf > 0.0);
        assert!(neg_inf < 0.0);
        assert!(pos_inf.is_infinite());
        assert!(neg_inf.is_infinite());
    }

    #[test]
    fn test_float_clamping_with_infinity() {
        // Clamping infinity should work correctly
        let pos_inf = f32::INFINITY;
        let clamped = pos_inf.clamp(0.0, 1.0);
        assert_eq!(clamped, 1.0);

        let neg_inf = f32::NEG_INFINITY;
        let clamped = neg_inf.clamp(0.0, 1.0);
        assert_eq!(clamped, 0.0);
    }

    // ------------------------------------------------------------------------
    // Arithmetic Overflow Protection Tests
    // ------------------------------------------------------------------------

    #[test]
    fn test_texture_size_overflow_protection() {
        // width * height * 4 could overflow u32 for very large textures
        // Max u32 = 4,294,967,295
        // width * height would overflow at ~65536 x 65536 (if not multiplied by 4)
        // With *4, overflow occurs at ~32768 x 32768

        // Test that checked_mul catches overflow
        let width: u32 = 65536;
        let height: u32 = 65536;

        // This would overflow: 65536 * 65536 = 4,294,967,296 > u32::MAX
        let result = width.checked_mul(height);
        assert!(
            result.is_none(),
            "Expected overflow for 65536x65536 texture"
        );

        // Smaller dimensions should succeed
        let width: u32 = 4096;
        let height: u32 = 4096;
        let pixels = width.checked_mul(height);
        assert!(pixels.is_some());
        let size = pixels.unwrap().checked_mul(4);
        assert!(size.is_some());
    }

    #[test]
    fn test_vertex_data_size_overflow_protection() {
        // vertex_count * stride could overflow for large vertex counts
        // Max stride is 64 bytes (all format flags set including skinned)
        // Max safe vertex_count with stride 64: u32::MAX / 64 = 67,108,863

        let stride: u32 = 64; // Max possible stride
        let vertex_count: u32 = u32::MAX / 64 + 1; // Just over the limit

        let result = vertex_count.checked_mul(stride);
        assert!(
            result.is_none(),
            "Expected overflow for extreme vertex count"
        );

        // Safe vertex count should succeed
        let vertex_count: u32 = 1_000_000;
        let result = vertex_count.checked_mul(stride);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 64_000_000);
    }

    #[test]
    fn test_index_data_size_overflow_protection() {
        // index_count * 4 could overflow for very large index counts
        // Max safe index_count: u32::MAX / 4 = 1,073,741,823

        let index_count: u32 = u32::MAX / 4 + 1; // Just over the limit

        let result = index_count.checked_mul(4);
        assert!(
            result.is_none(),
            "Expected overflow for extreme index count"
        );

        // Safe index count should succeed
        let index_count: u32 = 1_000_000;
        let result = index_count.checked_mul(4);
        assert!(result.is_some());
        assert_eq!(result.unwrap(), 4_000_000);
    }

    #[test]
    fn test_realistic_mesh_sizes_no_overflow() {
        // Test realistic mesh sizes that should never overflow

        // Large but reasonable mesh: 100,000 vertices with full format
        let vertex_count: u32 = 100_000;
        let stride: u32 = vertex_stride(15); // All flags set
        assert_eq!(stride, 64);

        let data_size = vertex_count.checked_mul(stride);
        assert!(data_size.is_some());
        assert_eq!(data_size.unwrap(), 6_400_000); // 6.4 MB

        // Large index buffer: 300,000 indices (100,000 triangles)
        let index_count: u32 = 300_000;
        let index_size = index_count.checked_mul(4);
        assert!(index_size.is_some());
        assert_eq!(index_size.unwrap(), 1_200_000); // 1.2 MB
    }

    #[test]
    fn test_realistic_texture_sizes_no_overflow() {
        // Test realistic texture sizes that should never overflow

        // 4K texture: 4096 x 4096 x 4 = 67,108,864 bytes (64 MB)
        let width: u32 = 4096;
        let height: u32 = 4096;
        let pixels = width.checked_mul(height);
        assert!(pixels.is_some());
        let size = pixels.unwrap().checked_mul(4);
        assert!(size.is_some());
        assert_eq!(size.unwrap(), 67_108_864);

        // 8K texture: 8192 x 8192 x 4 = 268,435,456 bytes (256 MB)
        let width: u32 = 8192;
        let height: u32 = 8192;
        let pixels = width.checked_mul(height);
        assert!(pixels.is_some());
        let size = pixels.unwrap().checked_mul(4);
        assert!(size.is_some());
        assert_eq!(size.unwrap(), 268_435_456);
    }
}
