//! Camera FFI functions
//!
//! Functions for setting camera position and field of view.

use anyhow::Result;
use glam::{Mat4, Vec3};
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::ZFFIState;

/// Register camera FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "camera_set", camera_set)?;
    linker.func_wrap("env", "camera_fov", camera_fov)?;
    linker.func_wrap("env", "push_view_matrix", push_view_matrix)?;
    linker.func_wrap("env", "push_projection_matrix", push_projection_matrix)?;
    Ok(())
}

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
