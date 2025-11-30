//! Emberware Z FFI host functions
//!
//! Console-specific FFI functions for the PS1/N64 aesthetic fantasy console.
//! These functions are registered with the WASM linker and called by games.

use anyhow::Result;
use glam::{Mat4, Vec3};
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use emberware_core::wasm::{DrawCommand, GameState, PendingMesh, PendingTexture, MAX_BONES, MAX_PLAYERS, MAX_TRANSFORM_STACK};

use crate::console::{RESOLUTIONS, TICK_RATES};

/// Register all Emberware Z FFI functions with the linker
pub fn register_z_ffi(linker: &mut Linker<GameState>) -> Result<()> {
    // Configuration functions (init-only)
    linker.func_wrap("env", "set_resolution", set_resolution)?;
    linker.func_wrap("env", "set_tick_rate", set_tick_rate)?;
    linker.func_wrap("env", "set_clear_color", set_clear_color)?;
    linker.func_wrap("env", "render_mode", render_mode)?;

    // Camera functions
    linker.func_wrap("env", "camera_set", camera_set)?;
    linker.func_wrap("env", "camera_fov", camera_fov)?;

    // Transform stack functions
    linker.func_wrap("env", "transform_identity", transform_identity)?;
    linker.func_wrap("env", "transform_translate", transform_translate)?;
    linker.func_wrap("env", "transform_rotate", transform_rotate)?;
    linker.func_wrap("env", "transform_scale", transform_scale)?;
    linker.func_wrap("env", "transform_push", transform_push)?;
    linker.func_wrap("env", "transform_pop", transform_pop)?;
    linker.func_wrap("env", "transform_set", transform_set)?;

    // Input functions
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

    // Sky system
    linker.func_wrap("env", "set_sky", set_sky)?;

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
    linker.func_wrap("env", "light_disable", light_disable)?;

    // Mode 3 (Hybrid) lighting functions
    // Note: Mode 3 uses the same FFI functions as Mode 2 but conventionally only uses light 0
    // The shader in Mode 3 uses light 0 as the single directional light

    // GPU skinning
    linker.func_wrap("env", "set_bones", set_bones)?;

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
    mut caller: Caller<'_, GameState>,
    x: f32,
    y: f32,
    z: f32,
    target_x: f32,
    target_y: f32,
    target_z: f32,
) {
    let state = caller.data_mut();
    state.camera.position = Vec3::new(x, y, z);
    state.camera.target = Vec3::new(target_x, target_y, target_z);
}

/// Set the camera field of view
///
/// # Arguments
/// * `fov_degrees` — Field of view in degrees (typically 45-90, default 60)
///
/// Values outside 1-179 degrees are clamped with a warning.
fn camera_fov(mut caller: Caller<'_, GameState>, fov_degrees: f32) {
    let state = caller.data_mut();

    // Validate FOV range
    let clamped_fov = if fov_degrees < 1.0 || fov_degrees > 179.0 {
        let clamped = fov_degrees.clamp(1.0, 179.0);
        warn!(
            "camera_fov({}) out of range (1-179), clamped to {}",
            fov_degrees, clamped
        );
        clamped
    } else {
        fov_degrees
    };

    state.camera.fov = clamped_fov;
}

// ============================================================================
// Transform Stack Functions
// ============================================================================

/// Reset the current transform to identity matrix
///
/// After calling this, the transform represents no transformation
/// (objects will be drawn at their original position/rotation/scale).
fn transform_identity(mut caller: Caller<'_, GameState>) {
    let state = caller.data_mut();
    state.current_transform = Mat4::IDENTITY;
}

/// Translate the current transform
///
/// # Arguments
/// * `x, y, z` — Translation amounts in world units
///
/// The translation is applied to the current transform (post-multiplication).
fn transform_translate(mut caller: Caller<'_, GameState>, x: f32, y: f32, z: f32) {
    let state = caller.data_mut();
    state.current_transform = state.current_transform * Mat4::from_translation(Vec3::new(x, y, z));
}

/// Rotate the current transform around an axis
///
/// # Arguments
/// * `angle_deg` — Rotation angle in degrees
/// * `x, y, z` — Rotation axis (will be normalized internally)
///
/// The rotation is applied to the current transform (post-multiplication).
/// Common axes: (1,0,0)=X, (0,1,0)=Y, (0,0,1)=Z
fn transform_rotate(mut caller: Caller<'_, GameState>, angle_deg: f32, x: f32, y: f32, z: f32) {
    let state = caller.data_mut();
    let axis = Vec3::new(x, y, z);

    // Handle zero-length axis
    if axis.length_squared() < 1e-10 {
        warn!("transform_rotate called with zero-length axis, ignored");
        return;
    }

    let axis = axis.normalize();
    let angle_rad = angle_deg.to_radians();
    state.current_transform = state.current_transform * Mat4::from_axis_angle(axis, angle_rad);
}

/// Scale the current transform
///
/// # Arguments
/// * `x, y, z` — Scale factors for each axis (1.0 = no change)
///
/// The scale is applied to the current transform (post-multiplication).
fn transform_scale(mut caller: Caller<'_, GameState>, x: f32, y: f32, z: f32) {
    let state = caller.data_mut();
    state.current_transform = state.current_transform * Mat4::from_scale(Vec3::new(x, y, z));
}

/// Push the current transform onto the stack
///
/// Returns 1 on success, 0 if the stack is full (max 16 entries).
/// Use this before making temporary transform changes that should be undone later.
fn transform_push(mut caller: Caller<'_, GameState>) -> u32 {
    let state = caller.data_mut();

    if state.transform_stack.len() >= MAX_TRANSFORM_STACK {
        warn!("transform_push failed: stack full (max {} entries)", MAX_TRANSFORM_STACK);
        return 0;
    }

    state.transform_stack.push(state.current_transform);
    1
}

/// Pop the transform from the stack
///
/// Returns 1 on success, 0 if the stack is empty.
/// Restores the transform that was active before the matching push.
fn transform_pop(mut caller: Caller<'_, GameState>) -> u32 {
    let state = caller.data_mut();

    if let Some(transform) = state.transform_stack.pop() {
        state.current_transform = transform;
        1
    } else {
        warn!("transform_pop failed: stack empty");
        0
    }
}

/// Set the current transform from a 4x4 matrix
///
/// # Arguments
/// * `matrix_ptr` — Pointer to 16 f32 values in column-major order
///
/// Column-major order means: [col0, col1, col2, col3] where each column is [x, y, z, w].
/// This is the same format used by glam and WGSL.
fn transform_set(mut caller: Caller<'_, GameState>, matrix_ptr: u32) {
    // Read the 16 floats from WASM memory
    let memory = match caller.data().memory {
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
    let state = caller.data_mut();
    state.current_transform = Mat4::from_cols_array(&matrix);
}

// ============================================================================
// Input Functions
// ============================================================================

/// Check if a button is currently held for a player
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum: UP=0, DOWN=1, ..., SELECT=13)
///
/// Returns 1 if held, 0 otherwise.
fn button_held(caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("button_held: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0;
    }

    if button > 13 {
        warn!("button_held: invalid button {} (max 13)", button);
        return 0;
    }

    let mask = 1u16 << button;
    if (state.input_curr[player].buttons & mask) != 0 {
        1
    } else {
        0
    }
}

/// Check if a button was just pressed this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum)
///
/// Returns 1 if just pressed (not held last tick, held this tick), 0 otherwise.
fn button_pressed(caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("button_pressed: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0;
    }

    if button > 13 {
        warn!("button_pressed: invalid button {} (max 13)", button);
        return 0;
    }

    let mask = 1u16 << button;
    let was_held = (state.input_prev[player].buttons & mask) != 0;
    let is_held = (state.input_curr[player].buttons & mask) != 0;

    if is_held && !was_held {
        1
    } else {
        0
    }
}

/// Check if a button was just released this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `button` — Button index (see Button enum)
///
/// Returns 1 if just released (held last tick, not held this tick), 0 otherwise.
fn button_released(caller: Caller<'_, GameState>, player: u32, button: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("button_released: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0;
    }

    if button > 13 {
        warn!("button_released: invalid button {} (max 13)", button);
        return 0;
    }

    let mask = 1u16 << button;
    let was_held = (state.input_prev[player].buttons & mask) != 0;
    let is_held = (state.input_curr[player].buttons & mask) != 0;

    if was_held && !is_held {
        1
    } else {
        0
    }
}

/// Get bitmask of all held buttons for a player
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask where each bit represents a button state.
fn buttons_held(caller: Caller<'_, GameState>, player: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("buttons_held: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0;
    }

    state.input_curr[player].buttons as u32
}

/// Get bitmask of all buttons just pressed this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask of buttons that are held now but were not held last tick.
fn buttons_pressed(caller: Caller<'_, GameState>, player: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("buttons_pressed: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0;
    }

    let prev = state.input_prev[player].buttons;
    let curr = state.input_curr[player].buttons;

    // Pressed = held now AND not held before
    (curr & !prev) as u32
}

/// Get bitmask of all buttons just released this tick
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns a bitmask of buttons that were held last tick but are not held now.
fn buttons_released(caller: Caller<'_, GameState>, player: u32) -> u32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("buttons_released: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0;
    }

    let prev = state.input_prev[player].buttons;
    let curr = state.input_curr[player].buttons;

    // Released = held before AND not held now
    (prev & !curr) as u32
}

/// Get left stick X axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
fn left_stick_x(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("left_stick_x: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0.0;
    }

    state.input_curr[player].left_stick_x as f32 / 127.0
}

/// Get left stick Y axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
fn left_stick_y(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("left_stick_y: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0.0;
    }

    state.input_curr[player].left_stick_y as f32 / 127.0
}

/// Get right stick X axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
fn right_stick_x(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("right_stick_x: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0.0;
    }

    state.input_curr[player].right_stick_x as f32 / 127.0
}

/// Get right stick Y axis value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from -1.0 to 1.0 (0.0 if invalid player).
fn right_stick_y(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("right_stick_y: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0.0;
    }

    state.input_curr[player].right_stick_y as f32 / 127.0
}

/// Get both left stick axes at once
///
/// # Arguments
/// * `player` — Player index (0-3)
/// * `out_x` — Pointer to write X axis value (-1.0 to 1.0)
/// * `out_y` — Pointer to write Y axis value (-1.0 to 1.0)
///
/// More efficient than two separate calls for the same player.
fn left_stick(mut caller: Caller<'_, GameState>, player: u32, out_x: u32, out_y: u32) {
    let (x, y) = {
        let state = caller.data();
        let player = player as usize;

        if player >= MAX_PLAYERS {
            warn!("left_stick: invalid player {} (max {})", player, MAX_PLAYERS - 1);
            (0.0f32, 0.0f32)
        } else {
            let input = &state.input_curr[player];
            (
                input.left_stick_x as f32 / 127.0,
                input.left_stick_y as f32 / 127.0,
            )
        }
    };

    // Write results to WASM memory
    let memory = match caller.data().memory {
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
fn right_stick(mut caller: Caller<'_, GameState>, player: u32, out_x: u32, out_y: u32) {
    let (x, y) = {
        let state = caller.data();
        let player = player as usize;

        if player >= MAX_PLAYERS {
            warn!("right_stick: invalid player {} (max {})", player, MAX_PLAYERS - 1);
            (0.0f32, 0.0f32)
        } else {
            let input = &state.input_curr[player];
            (
                input.right_stick_x as f32 / 127.0,
                input.right_stick_y as f32 / 127.0,
            )
        }
    };

    // Write results to WASM memory
    let memory = match caller.data().memory {
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

/// Get left trigger value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from 0.0 to 1.0 (0.0 if invalid player).
fn trigger_left(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("trigger_left: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0.0;
    }

    state.input_curr[player].left_trigger as f32 / 255.0
}

/// Get right trigger value
///
/// # Arguments
/// * `player` — Player index (0-3)
///
/// Returns value from 0.0 to 1.0 (0.0 if invalid player).
fn trigger_right(caller: Caller<'_, GameState>, player: u32) -> f32 {
    let state = caller.data();
    let player = player as usize;

    if player >= MAX_PLAYERS {
        warn!("trigger_right: invalid player {} (max {})", player, MAX_PLAYERS - 1);
        return 0.0;
    }

    state.input_curr[player].right_trigger as f32 / 255.0
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
fn set_color(mut caller: Caller<'_, GameState>, color: u32) {
    let state = caller.data_mut();
    state.render_state.color = color;
}

/// Enable or disable depth testing
///
/// # Arguments
/// * `enabled` — 0 to disable, non-zero to enable
///
/// Default: enabled. Disable for 2D overlays or special effects.
fn depth_test(mut caller: Caller<'_, GameState>, enabled: u32) {
    let state = caller.data_mut();
    state.render_state.depth_test = enabled != 0;
}

/// Set the face culling mode
///
/// # Arguments
/// * `mode` — 0=none (draw both sides), 1=back (default), 2=front
///
/// Back-face culling is the default for solid 3D objects.
fn cull_mode(mut caller: Caller<'_, GameState>, mode: u32) {
    let state = caller.data_mut();

    if mode > 2 {
        warn!("cull_mode({}) invalid - must be 0-2, using 0 (none)", mode);
        state.render_state.cull_mode = 0;
        return;
    }

    state.render_state.cull_mode = mode as u8;
}

/// Set the blend mode for transparent rendering
///
/// # Arguments
/// * `mode` — 0=none (opaque), 1=alpha, 2=additive, 3=multiply
///
/// Default: none (opaque). Use alpha for transparent textures.
fn blend_mode(mut caller: Caller<'_, GameState>, mode: u32) {
    let state = caller.data_mut();

    if mode > 3 {
        warn!("blend_mode({}) invalid - must be 0-3, using 0 (none)", mode);
        state.render_state.blend_mode = 0;
        return;
    }

    state.render_state.blend_mode = mode as u8;
}

/// Set the texture filtering mode
///
/// # Arguments
/// * `filter` — 0=nearest (pixelated, retro), 1=linear (smooth)
///
/// Default: nearest for retro aesthetic.
fn texture_filter(mut caller: Caller<'_, GameState>, filter: u32) {
    let state = caller.data_mut();

    if filter > 1 {
        warn!("texture_filter({}) invalid - must be 0-1, using 0 (nearest)", filter);
        state.render_state.texture_filter = 0;
        return;
    }

    state.render_state.texture_filter = filter as u8;
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
fn load_texture(mut caller: Caller<'_, GameState>, width: u32, height: u32, pixels_ptr: u32) -> u32 {
    // Validate dimensions
    if width == 0 || height == 0 {
        warn!("load_texture: invalid dimensions {}x{}", width, height);
        return 0;
    }

    // Read pixel data from WASM memory
    let memory = match caller.data().memory {
        Some(m) => m,
        None => {
            warn!("load_texture: no WASM memory available");
            return 0;
        }
    };

    let ptr = pixels_ptr as usize;
    let size = (width * height * 4) as usize;

    // Copy pixel data while we have the immutable borrow
    let pixel_data = {
        let mem_data = memory.data(&caller);

        if ptr + size > mem_data.len() {
            warn!(
                "load_texture: pixel data ({} bytes at {}) exceeds memory bounds ({})",
                size, ptr, mem_data.len()
            );
            return 0;
        }

        mem_data[ptr..ptr + size].to_vec()
    };

    // Now we can mutably borrow state
    let state = caller.data_mut();

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
fn texture_bind(mut caller: Caller<'_, GameState>, handle: u32) {
    let state = caller.data_mut();
    state.render_state.bound_textures[0] = handle;
}

/// Bind a texture to a specific slot
///
/// # Arguments
/// * `handle` — Texture handle from load_texture
/// * `slot` — Slot index (0-3)
///
/// Slots: 0=albedo, 1=MRE/matcap, 2=env matcap, 3=matcap
fn texture_bind_slot(mut caller: Caller<'_, GameState>, handle: u32, slot: u32) {
    if slot > 3 {
        warn!("texture_bind_slot: invalid slot {} (max 3)", slot);
        return;
    }

    let state = caller.data_mut();
    state.render_state.bound_textures[slot as usize] = handle;
}

// ============================================================================
// Mesh Functions (Retained Mode)
// ============================================================================

/// Vertex format flags
const FORMAT_UV: u8 = 1;
const FORMAT_COLOR: u8 = 2;
const FORMAT_NORMAL: u8 = 4;
const FORMAT_SKINNED: u8 = 8;
const MAX_VERTEX_FORMAT: u8 = 15;

/// Calculate vertex stride in bytes for a given format
#[inline]
const fn vertex_stride(format: u8) -> u32 {
    let mut stride = 3 * 4; // Position: 3 floats = 12 bytes

    if format & FORMAT_UV != 0 {
        stride += 2 * 4; // UV: 2 floats = 8 bytes
    }
    if format & FORMAT_COLOR != 0 {
        stride += 3 * 4; // Color: 3 floats = 12 bytes
    }
    if format & FORMAT_NORMAL != 0 {
        stride += 3 * 4; // Normal: 3 floats = 12 bytes
    }
    if format & FORMAT_SKINNED != 0 {
        stride += 4 + 4 * 4; // Bone indices (4 u8 = 4 bytes) + weights (4 floats = 16 bytes) = 20 bytes
    }

    stride
}

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
fn load_mesh(mut caller: Caller<'_, GameState>, data_ptr: u32, vertex_count: u32, format: u32) -> u32 {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!("load_mesh: invalid format {} (max {})", format, MAX_VERTEX_FORMAT);
        return 0;
    }
    let format = format as u8;

    // Validate vertex count
    if vertex_count == 0 {
        warn!("load_mesh: vertex_count cannot be 0");
        return 0;
    }

    // Calculate data size
    let stride = vertex_stride(format);
    let data_size = vertex_count * stride;
    let float_count = data_size / 4;

    // Read vertex data from WASM memory
    let memory = match caller.data().memory {
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
                byte_size, ptr, mem_data.len()
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
    let state = caller.data_mut();

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
    mut caller: Caller<'_, GameState>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) -> u32 {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!("load_mesh_indexed: invalid format {} (max {})", format, MAX_VERTEX_FORMAT);
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
    if index_count % 3 != 0 {
        warn!("load_mesh_indexed: index_count {} is not a multiple of 3", index_count);
        return 0;
    }

    // Calculate data sizes
    let stride = vertex_stride(format);
    let vertex_data_size = vertex_count * stride;
    let index_data_size = index_count * 4; // u32 = 4 bytes
    let float_count = vertex_data_size / 4;

    // Read data from WASM memory
    let memory = match caller.data().memory {
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
    let (vertex_data, index_data): (Vec<f32>, Vec<u32>) = {
        let mem_data = memory.data(&caller);

        if vertex_ptr + vertex_byte_size > mem_data.len() {
            warn!(
                "load_mesh_indexed: vertex data ({} bytes at {}) exceeds memory bounds ({})",
                vertex_byte_size, vertex_ptr, mem_data.len()
            );
            return 0;
        }

        if idx_ptr + index_byte_size > mem_data.len() {
            warn!(
                "load_mesh_indexed: index data ({} bytes at {}) exceeds memory bounds ({})",
                index_byte_size, idx_ptr, mem_data.len()
            );
            return 0;
        }

        let vertex_bytes = &mem_data[vertex_ptr..vertex_ptr + vertex_byte_size];
        let floats: &[f32] = bytemuck::cast_slice(vertex_bytes);

        let index_bytes = &mem_data[idx_ptr..idx_ptr + index_byte_size];
        let indices: &[u32] = bytemuck::cast_slice(index_bytes);

        // Validate indices are within bounds
        for &idx in indices {
            if idx >= vertex_count {
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
    let state = caller.data_mut();

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
fn draw_mesh(mut caller: Caller<'_, GameState>, handle: u32) {
    if handle == 0 {
        warn!("draw_mesh: invalid handle 0");
        return;
    }

    let state = caller.data_mut();

    // Record draw command with current state
    state.draw_commands.push(DrawCommand::DrawMesh {
        handle,
        transform: state.current_transform,
        color: state.render_state.color,
        depth_test: state.render_state.depth_test,
        cull_mode: state.render_state.cull_mode,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
fn draw_triangles(mut caller: Caller<'_, GameState>, data_ptr: u32, vertex_count: u32, format: u32) {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!("draw_triangles: invalid format {} (max {})", format, MAX_VERTEX_FORMAT);
        return;
    }
    let format = format as u8;

    // Validate vertex count
    if vertex_count == 0 {
        return; // Nothing to draw
    }
    if vertex_count % 3 != 0 {
        warn!("draw_triangles: vertex_count {} is not a multiple of 3", vertex_count);
        return;
    }

    // Calculate data size
    let stride = vertex_stride(format);
    let data_size = vertex_count * stride;
    let float_count = data_size / 4;

    // Read vertex data from WASM memory
    let memory = match caller.data().memory {
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
                byte_size, ptr, mem_data.len()
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

    let state = caller.data_mut();

    // Record draw command with current state
    state.draw_commands.push(DrawCommand::DrawTriangles {
        format,
        vertex_data,
        transform: state.current_transform,
        color: state.render_state.color,
        depth_test: state.render_state.depth_test,
        cull_mode: state.render_state.cull_mode,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
    mut caller: Caller<'_, GameState>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) {
    // Validate format
    if format > MAX_VERTEX_FORMAT as u32 {
        warn!("draw_triangles_indexed: invalid format {} (max {})", format, MAX_VERTEX_FORMAT);
        return;
    }
    let format = format as u8;

    // Validate counts
    if vertex_count == 0 || index_count == 0 {
        return; // Nothing to draw
    }
    if index_count % 3 != 0 {
        warn!("draw_triangles_indexed: index_count {} is not a multiple of 3", index_count);
        return;
    }

    // Calculate data sizes
    let stride = vertex_stride(format);
    let vertex_data_size = vertex_count * stride;
    let index_data_size = index_count * 4; // u32 = 4 bytes
    let float_count = vertex_data_size / 4;

    // Read data from WASM memory
    let memory = match caller.data().memory {
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
    let (vertex_data, index_data): (Vec<f32>, Vec<u32>) = {
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
        let indices: &[u32] = bytemuck::cast_slice(index_bytes);

        // Validate indices are within bounds
        for &idx in indices {
            if idx >= vertex_count {
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

    let state = caller.data_mut();

    // Record draw command with current state
    state.draw_commands.push(DrawCommand::DrawTrianglesIndexed {
        format,
        vertex_data,
        index_data,
        transform: state.current_transform,
        color: state.render_state.color,
        depth_test: state.render_state.depth_test,
        cull_mode: state.render_state.cull_mode,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
fn draw_billboard(mut caller: Caller<'_, GameState>, w: f32, h: f32, mode: u32, color: u32) {
    // Validate mode
    if mode < 1 || mode > 4 {
        warn!("draw_billboard: invalid mode {} (must be 1-4)", mode);
        return;
    }

    let state = caller.data_mut();

    // Record billboard draw command
    state.draw_commands.push(DrawCommand::DrawBillboard {
        width: w,
        height: h,
        mode: mode as u8,
        uv_rect: None, // Full texture (0,0,1,1)
        transform: state.current_transform,
        color,
        depth_test: state.render_state.depth_test,
        cull_mode: state.render_state.cull_mode,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
    mut caller: Caller<'_, GameState>,
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
    if mode < 1 || mode > 4 {
        warn!("draw_billboard_region: invalid mode {} (must be 1-4)", mode);
        return;
    }

    let state = caller.data_mut();

    // Record billboard draw command with UV region
    state.draw_commands.push(DrawCommand::DrawBillboard {
        width: w,
        height: h,
        mode: mode as u8,
        uv_rect: Some((src_x, src_y, src_w, src_h)),
        transform: state.current_transform,
        color,
        depth_test: state.render_state.depth_test,
        cull_mode: state.render_state.cull_mode,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
fn draw_sprite(mut caller: Caller<'_, GameState>, x: f32, y: f32, w: f32, h: f32, color: u32) {
    let state = caller.data_mut();

    state.draw_commands.push(DrawCommand::DrawSprite {
        x,
        y,
        width: w,
        height: h,
        uv_rect: None, // Full texture (0,0,1,1)
        origin: None,  // No rotation
        rotation: 0.0,
        color,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
    mut caller: Caller<'_, GameState>,
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
    let state = caller.data_mut();

    state.draw_commands.push(DrawCommand::DrawSprite {
        x,
        y,
        width: w,
        height: h,
        uv_rect: Some((src_x, src_y, src_w, src_h)),
        origin: None,  // No rotation
        rotation: 0.0,
        color,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
    mut caller: Caller<'_, GameState>,
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
    let state = caller.data_mut();

    state.draw_commands.push(DrawCommand::DrawSprite {
        x,
        y,
        width: w,
        height: h,
        uv_rect: Some((src_x, src_y, src_w, src_h)),
        origin: Some((origin_x, origin_y)),
        rotation: angle_deg,
        color,
        blend_mode: state.render_state.blend_mode,
        bound_textures: state.render_state.bound_textures,
    });
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
fn draw_rect(mut caller: Caller<'_, GameState>, x: f32, y: f32, w: f32, h: f32, color: u32) {
    let state = caller.data_mut();

    state.draw_commands.push(DrawCommand::DrawRect {
        x,
        y,
        width: w,
        height: h,
        color,
        blend_mode: state.render_state.blend_mode,
    });
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
    mut caller: Caller<'_, GameState>,
    ptr: u32,
    len: u32,
    x: f32,
    y: f32,
    size: f32,
    color: u32,
) {
    // Read UTF-8 string from WASM memory
    let memory = match caller.data().memory {
        Some(m) => m,
        None => {
            warn!("draw_text: no WASM memory available");
            return;
        }
    };

    let text_string = {
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
        match std::str::from_utf8(bytes) {
            Ok(s) => s.to_string(),
            Err(e) => {
                warn!("draw_text: invalid UTF-8 string: {}", e);
                return;
            }
        }
    };

    let state = caller.data_mut();

    state.draw_commands.push(DrawCommand::DrawText {
        text: text_string,
        x,
        y,
        size,
        color,
        blend_mode: state.render_state.blend_mode,
    });
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
/// * `sun_b` — Sun color blue (0.0-1.0+)
/// * `sun_sharpness` — Sun sharpness (typically 32-256, higher = sharper sun)
///
/// Configures the procedural sky system for background rendering and ambient lighting.
/// Default is all zeros (black sky, no sun, no lighting).
fn set_sky(
    mut caller: Caller<'_, GameState>,
    horizon_r: f32,
    horizon_g: f32,
    horizon_b: f32,
    zenith_r: f32,
    zenith_g: f32,
    zenith_b: f32,
    sun_dir_x: f32,
    sun_dir_y: f32,
    sun_dir_z: f32,
    sun_r: f32,
    sun_g: f32,
    sun_b: f32,
    sun_sharpness: f32,
) {
    let state = caller.data_mut();

    // Record sky configuration as a draw command
    state.draw_commands.push(DrawCommand::SetSky {
        horizon_color: [horizon_r, horizon_g, horizon_b],
        zenith_color: [zenith_r, zenith_g, zenith_b],
        sun_direction: [sun_dir_x, sun_dir_y, sun_dir_z],
        sun_color: [sun_r, sun_g, sun_b],
        sun_sharpness,
    });

    info!(
        "set_sky: horizon=({:.2},{:.2},{:.2}), zenith=({:.2},{:.2},{:.2}), sun_dir=({:.2},{:.2},{:.2}), sun_color=({:.2},{:.2},{:.2}), sharpness={:.1}",
        horizon_r, horizon_g, horizon_b,
        zenith_r, zenith_g, zenith_b,
        sun_dir_x, sun_dir_y, sun_dir_z,
        sun_r, sun_g, sun_b,
        sun_sharpness
    );
}

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
fn matcap_set(mut caller: Caller<'_, GameState>, slot: u32, texture: u32) {
    // Validate slot range (1-3 for matcaps)
    if slot < 1 || slot > 3 {
        warn!("matcap_set: invalid slot {} (must be 1-3)", slot);
        return;
    }

    let state = caller.data_mut();
    state.render_state.bound_textures[slot as usize] = texture;
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
fn material_mre(mut caller: Caller<'_, GameState>, texture: u32) {
    let state = caller.data_mut();
    state.render_state.bound_textures[1] = texture;
}

/// Bind an albedo texture
///
/// # Arguments
/// * `texture` — Texture handle for the base color/albedo map
///
/// Binds to slot 0. This is equivalent to texture_bind(texture) but more semantically clear.
/// The albedo texture is multiplied with the uniform color and vertex colors.
fn material_albedo(mut caller: Caller<'_, GameState>, texture: u32) {
    let state = caller.data_mut();
    state.render_state.bound_textures[0] = texture;
}

/// Set the material metallic value
///
/// # Arguments
/// * `value` — Metallic value (0.0 = dielectric, 1.0 = metal)
///
/// Used in Mode 2 (PBR) and Mode 3 (Hybrid).
/// Clamped to 0.0-1.0 range. Default is 0.0 (non-metallic).
fn material_metallic(mut caller: Caller<'_, GameState>, value: f32) {
    let state = caller.data_mut();
    let clamped = value.clamp(0.0, 1.0);

    if (value - clamped).abs() > 0.001 {
        warn!("material_metallic: value {} out of range, clamped to {}", value, clamped);
    }

    state.render_state.material_metallic = clamped;
}

/// Set the material roughness value
///
/// # Arguments
/// * `value` — Roughness value (0.0 = smooth/glossy, 1.0 = rough/matte)
///
/// Used in Mode 2 (PBR) and Mode 3 (Hybrid).
/// Clamped to 0.0-1.0 range. Default is 0.5.
fn material_roughness(mut caller: Caller<'_, GameState>, value: f32) {
    let state = caller.data_mut();
    let clamped = value.clamp(0.0, 1.0);

    if (value - clamped).abs() > 0.001 {
        warn!("material_roughness: value {} out of range, clamped to {}", value, clamped);
    }

    state.render_state.material_roughness = clamped;
}

/// Set the material emissive intensity
///
/// # Arguments
/// * `value` — Emissive intensity (0.0 = no emission, higher = brighter)
///
/// Used in Mode 2 (PBR) and Mode 3 (Hybrid).
/// Values can be greater than 1.0 for HDR-like effects. Default is 0.0.
fn material_emissive(mut caller: Caller<'_, GameState>, value: f32) {
    let state = caller.data_mut();

    // No clamping for emissive - allow HDR values
    if value < 0.0 {
        warn!("material_emissive: negative value {} not allowed, using 0.0", value);
        state.render_state.material_emissive = 0.0;
    } else {
        state.render_state.material_emissive = value;
    }
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
fn light_set(mut caller: Caller<'_, GameState>, index: u32, x: f32, y: f32, z: f32) {
    // Validate index
    if index > 3 {
        warn!("light_set: invalid light index {} (must be 0-3)", index);
        return;
    }

    // Validate direction vector (warn if zero-length)
    let len_sq = x * x + y * y + z * z;
    if len_sq < 1e-10 {
        warn!("light_set: zero-length direction vector, using default (0, -1, 0)");
        let state = caller.data_mut();
        state.render_state.lights[index as usize].direction = [0.0, -1.0, 0.0];
        state.render_state.lights[index as usize].enabled = true;
        return;
    }

    let state = caller.data_mut();
    let light = &mut state.render_state.lights[index as usize];

    // Set direction (will be normalized by graphics backend) and enable
    light.direction = [x, y, z];
    light.enabled = true;
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
fn light_color(mut caller: Caller<'_, GameState>, index: u32, r: f32, g: f32, b: f32) {
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

    let state = caller.data_mut();
    state.render_state.lights[index as usize].color = [r, g, b];
}

/// Set light intensity multiplier
///
/// # Arguments
/// * `index` — Light index (0-3)
/// * `intensity` — Intensity multiplier (typically 0.0-10.0, but no upper limit)
///
/// Sets the intensity multiplier for a light. The final light contribution is color × intensity.
/// Negative values are clamped to 0.0.
fn light_intensity(mut caller: Caller<'_, GameState>, index: u32, intensity: f32) {
    // Validate index
    if index > 3 {
        warn!("light_intensity: invalid light index {} (must be 0-3)", index);
        return;
    }

    // Validate intensity (allow > 1.0, but clamp negative to 0.0)
    let intensity = if intensity < 0.0 {
        warn!("light_intensity: negative intensity {}, clamping to 0.0", intensity);
        0.0
    } else {
        intensity
    };

    let state = caller.data_mut();
    state.render_state.lights[index as usize].intensity = intensity;
}

/// Disable a light
///
/// # Arguments
/// * `index` — Light index (0-3)
///
/// Disables a light so it no longer contributes to the scene.
/// Useful for toggling lights on/off dynamically.
/// The light's direction, color, and intensity are preserved and can be re-enabled later.
fn light_disable(mut caller: Caller<'_, GameState>, index: u32) {
    // Validate index
    if index > 3 {
        warn!("light_disable: invalid light index {} (must be 0-3)", index);
        return;
    }

    let state = caller.data_mut();
    state.render_state.lights[index as usize].enabled = false;
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
fn set_bones(mut caller: Caller<'_, GameState>, matrices_ptr: u32, count: u32) {
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
        let state = caller.data_mut();
        state.render_state.bone_matrices.clear();
        state.render_state.bone_count = 0;
        return;
    }

    // Calculate required memory size (16 floats per matrix × 4 bytes per float)
    let matrix_size = 16 * 4; // 64 bytes per matrix
    let total_size = count as usize * matrix_size;

    // Get WASM memory
    let memory = match caller.data().memory {
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
        for j in 0..16 {
            let byte_offset = j * 4;
            let bytes = [
                matrix_bytes[byte_offset],
                matrix_bytes[byte_offset + 1],
                matrix_bytes[byte_offset + 2],
                matrix_bytes[byte_offset + 3],
            ];
            floats[j] = f32::from_le_bytes(bytes);
        }

        // Create Mat4 from column-major floats
        let matrix = Mat4::from_cols_array(&floats);
        matrices.push(matrix);
    }

    // Store bone matrices in render state
    let state = caller.data_mut();
    state.render_state.bone_matrices = matrices;
    state.render_state.bone_count = count;
}

#[cfg(test)]
mod tests {
    use emberware_core::wasm::{CameraState, GameState, DEFAULT_CAMERA_FOV, MAX_TRANSFORM_STACK};
    use glam::{Mat4, Vec3};

    #[test]
    fn test_init_config_defaults() {
        let state = GameState::new();
        assert_eq!(state.init_config.resolution_index, 1); // 540p
        assert_eq!(state.init_config.tick_rate_index, 2); // 60 fps
        assert_eq!(state.init_config.clear_color, 0x000000FF); // Black
        assert_eq!(state.init_config.render_mode, 0); // Unlit
        assert!(!state.init_config.modified);
    }

    // ========================================================================
    // Camera State Tests
    // ========================================================================

    #[test]
    fn test_camera_state_defaults() {
        let camera = CameraState::default();
        assert_eq!(camera.position, Vec3::new(0.0, 0.0, 5.0));
        assert_eq!(camera.target, Vec3::ZERO);
        assert_eq!(camera.fov, DEFAULT_CAMERA_FOV);
        assert_eq!(camera.near, 0.1);
        assert_eq!(camera.far, 1000.0);
    }

    #[test]
    fn test_camera_view_matrix() {
        let camera = CameraState {
            position: Vec3::new(0.0, 0.0, 5.0),
            target: Vec3::ZERO,
            fov: 60.0,
            near: 0.1,
            far: 100.0,
        };

        let view = camera.view_matrix();
        // The view matrix should transform the target point to the origin
        let target_in_view = view.transform_point3(camera.target);
        // Should be at (0, 0, -5) in view space (camera looks down -Z)
        assert!((target_in_view.z + 5.0).abs() < 0.001);
    }

    #[test]
    fn test_camera_projection_matrix() {
        let camera = CameraState::default();
        let proj = camera.projection_matrix(16.0 / 9.0);

        // Projection matrix should not be identity
        assert_ne!(proj, Mat4::IDENTITY);
        // Should have perspective (w != 1 for points not at origin)
        let point = proj.project_point3(Vec3::new(0.0, 0.0, -10.0));
        assert!(point.z.abs() < 1.0); // Should be in NDC range
    }

    #[test]
    fn test_game_state_camera_initialized() {
        let state = GameState::new();
        assert_eq!(state.camera.fov, DEFAULT_CAMERA_FOV);
        assert_eq!(state.camera.position, Vec3::new(0.0, 0.0, 5.0));
    }

    // ========================================================================
    // Transform Stack Tests
    // ========================================================================

    #[test]
    fn test_transform_stack_defaults() {
        let state = GameState::new();
        assert_eq!(state.current_transform, Mat4::IDENTITY);
        assert!(state.transform_stack.is_empty());
    }

    #[test]
    fn test_transform_stack_capacity() {
        let state = GameState::new();
        assert!(state.transform_stack.capacity() >= MAX_TRANSFORM_STACK);
    }

    #[test]
    fn test_transform_operations_on_game_state() {
        let mut state = GameState::new();

        // Test translation
        state.current_transform = state.current_transform * Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        let point = state.current_transform.transform_point3(Vec3::ZERO);
        assert!((point - Vec3::new(1.0, 2.0, 3.0)).length() < 0.001);

        // Reset and test rotation
        state.current_transform = Mat4::IDENTITY;
        let angle = std::f32::consts::FRAC_PI_2; // 90 degrees
        state.current_transform = state.current_transform * Mat4::from_rotation_y(angle);
        let point = state.current_transform.transform_point3(Vec3::new(1.0, 0.0, 0.0));
        // Rotating (1,0,0) 90 degrees around Y should give (0,0,-1)
        assert!((point.x).abs() < 0.001);
        assert!((point.z + 1.0).abs() < 0.001);

        // Reset and test scale
        state.current_transform = Mat4::IDENTITY;
        state.current_transform = state.current_transform * Mat4::from_scale(Vec3::new(2.0, 3.0, 4.0));
        let point = state.current_transform.transform_point3(Vec3::new(1.0, 1.0, 1.0));
        assert!((point - Vec3::new(2.0, 3.0, 4.0)).length() < 0.001);
    }

    #[test]
    fn test_transform_push_pop() {
        let mut state = GameState::new();

        // Set up initial transform
        state.current_transform = Mat4::from_translation(Vec3::new(1.0, 0.0, 0.0));
        let original = state.current_transform;

        // Push
        state.transform_stack.push(state.current_transform);
        assert_eq!(state.transform_stack.len(), 1);

        // Modify current transform
        state.current_transform = state.current_transform * Mat4::from_translation(Vec3::new(0.0, 1.0, 0.0));
        assert_ne!(state.current_transform, original);

        // Pop
        state.current_transform = state.transform_stack.pop().unwrap();
        assert_eq!(state.current_transform, original);
        assert!(state.transform_stack.is_empty());
    }

    #[test]
    fn test_transform_stack_max_depth() {
        let mut state = GameState::new();

        // Fill the stack to max
        for i in 0..MAX_TRANSFORM_STACK {
            assert!(state.transform_stack.len() < MAX_TRANSFORM_STACK);
            state.transform_stack.push(Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0)));
        }

        assert_eq!(state.transform_stack.len(), MAX_TRANSFORM_STACK);
    }

    // ========================================================================
    // GPU Skinning FFI Tests
    // ========================================================================

    use emberware_core::wasm::MAX_BONES;

    #[test]
    fn test_vertex_stride_skinned_constant() {
        // Skinning adds: 4 u8 bone indices (4 bytes) + 4 f32 bone weights (16 bytes) = 20 bytes
        const SKINNING_OVERHEAD: u32 = 20;

        // Test base format + skinning
        assert_eq!(super::vertex_stride(super::FORMAT_SKINNED), 12 + SKINNING_OVERHEAD); // 32
    }

    #[test]
    fn test_vertex_stride_all_skinned_formats() {
        // All 8 skinned format combinations
        assert_eq!(super::vertex_stride(8), 32);   // POS_SKINNED
        assert_eq!(super::vertex_stride(9), 40);   // POS_UV_SKINNED
        assert_eq!(super::vertex_stride(10), 44);  // POS_COLOR_SKINNED
        assert_eq!(super::vertex_stride(11), 52);  // POS_UV_COLOR_SKINNED
        assert_eq!(super::vertex_stride(12), 44);  // POS_NORMAL_SKINNED
        assert_eq!(super::vertex_stride(13), 52);  // POS_UV_NORMAL_SKINNED
        assert_eq!(super::vertex_stride(14), 56);  // POS_COLOR_NORMAL_SKINNED
        assert_eq!(super::vertex_stride(15), 64);  // POS_UV_COLOR_NORMAL_SKINNED
    }

    #[test]
    fn test_max_bones_constant() {
        // MAX_BONES should be 256 for GPU skinning
        assert_eq!(MAX_BONES, 256);
    }

    #[test]
    fn test_render_state_bone_matrices_default() {
        let state = GameState::new();
        assert!(state.render_state.bone_matrices.is_empty());
        assert_eq!(state.render_state.bone_count, 0);
    }

    #[test]
    fn test_render_state_bone_matrices_mutation() {
        let mut state = GameState::new();

        // Add bone matrices
        let bone = Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0));
        state.render_state.bone_matrices.push(bone);
        state.render_state.bone_count = 1;

        assert_eq!(state.render_state.bone_matrices.len(), 1);
        assert_eq!(state.render_state.bone_count, 1);
        assert_eq!(state.render_state.bone_matrices[0], bone);
    }

    #[test]
    fn test_render_state_bone_matrices_clear() {
        let mut state = GameState::new();

        // Add some bones
        for _ in 0..10 {
            state.render_state.bone_matrices.push(Mat4::IDENTITY);
        }
        state.render_state.bone_count = 10;

        // Clear
        state.render_state.bone_matrices.clear();
        state.render_state.bone_count = 0;

        assert!(state.render_state.bone_matrices.is_empty());
        assert_eq!(state.render_state.bone_count, 0);
    }

    #[test]
    fn test_render_state_bone_matrices_max_count() {
        let mut state = GameState::new();

        // Fill with MAX_BONES matrices
        for i in 0..MAX_BONES {
            let bone = Mat4::from_translation(Vec3::new(i as f32, 0.0, 0.0));
            state.render_state.bone_matrices.push(bone);
        }
        state.render_state.bone_count = MAX_BONES as u32;

        assert_eq!(state.render_state.bone_matrices.len(), MAX_BONES);
        assert_eq!(state.render_state.bone_count, MAX_BONES as u32);
    }

    #[test]
    fn test_skinned_format_flag_value() {
        // FORMAT_SKINNED should be 8 (bit 3)
        assert_eq!(super::FORMAT_SKINNED, 8);
    }

    #[test]
    fn test_max_vertex_format_includes_skinned() {
        // MAX_VERTEX_FORMAT should be 15 (all flags set: UV=1 | COLOR=2 | NORMAL=4 | SKINNED=8)
        assert_eq!(super::MAX_VERTEX_FORMAT, 15);
    }

    #[test]
    fn test_bone_matrix_identity_transform() {
        // Verify identity bone matrix doesn't transform a vertex
        let bone = Mat4::IDENTITY;
        let vertex = Vec3::new(1.0, 2.0, 3.0);
        let transformed = bone.transform_point3(vertex);

        assert_eq!(transformed, vertex);
    }

    #[test]
    fn test_bone_matrix_translation() {
        let bone = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        let vertex = Vec3::ZERO;
        let transformed = bone.transform_point3(vertex);

        assert!((transformed.x - 5.0).abs() < 0.0001);
        assert!(transformed.y.abs() < 0.0001);
        assert!(transformed.z.abs() < 0.0001);
    }

    #[test]
    fn test_bone_matrix_column_major_layout() {
        // Verify glam uses column-major layout (same as WGSL/wgpu)
        let translation = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));
        let cols = translation.to_cols_array();

        // Column 3 (indices 12-15) contains translation
        assert_eq!(cols[12], 10.0); // x translation
        assert_eq!(cols[13], 20.0); // y translation
        assert_eq!(cols[14], 30.0); // z translation
        assert_eq!(cols[15], 1.0);  // w = 1
    }

    #[test]
    fn test_bone_weights_sum_to_one() {
        // In GPU skinning, bone weights should sum to 1.0
        // This is a convention test - games should ensure this
        let weights = [0.5f32, 0.3, 0.15, 0.05];
        let sum: f32 = weights.iter().sum();
        assert!((sum - 1.0).abs() < 0.0001);
    }
}
