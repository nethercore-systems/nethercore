//! Emberware Z FFI host functions
//!
//! Console-specific FFI functions for the PS1/N64 aesthetic fantasy console.
//! These functions are registered with the WASM linker and called by games.

use anyhow::Result;
use glam::{Mat4, Vec3};
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use emberware_core::wasm::{GameState, MAX_TRANSFORM_STACK};

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
