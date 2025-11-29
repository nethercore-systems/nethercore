//! Emberware Z FFI host functions
//!
//! Console-specific FFI functions for the PS1/N64 aesthetic fantasy console.
//! These functions are registered with the WASM linker and called by games.

use anyhow::Result;
use glam::{Mat4, Vec3};
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use emberware_core::wasm::{GameState, PendingTexture, MAX_PLAYERS, MAX_TRANSFORM_STACK};

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
    let matrix: [f32; 16] = floats.try_into().expect("slice with incorrect length");
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
}
