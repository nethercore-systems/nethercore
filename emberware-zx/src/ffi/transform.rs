//! Transform stack FFI functions
//!
//! Functions for managing the model transform matrix stack.

use anyhow::Result;
use glam::{Mat4, Vec3};
use wasmtime::{Caller, Linker};

use super::ZXGameContext;
use super::helpers::read_wasm_matrix4x4;

/// Register transform FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "push_identity", push_identity)?;
    linker.func_wrap("env", "transform_set", transform_set)?;
    linker.func_wrap("env", "push_translate", push_translate)?;
    linker.func_wrap("env", "push_rotate_x", push_rotate_x)?;
    linker.func_wrap("env", "push_rotate_y", push_rotate_y)?;
    linker.func_wrap("env", "push_rotate_z", push_rotate_z)?;
    linker.func_wrap("env", "push_rotate", push_rotate)?;
    linker.func_wrap("env", "push_scale", push_scale)?;
    linker.func_wrap("env", "push_scale_uniform", push_scale_uniform)?;
    Ok(())
}

/// Push identity matrix onto the transform stack
///
/// After calling this, subsequent draws will use identity transformation
/// (objects will be drawn at their original position/rotation/scale).
/// This is typically called at the start of rendering to reset the transform stack.
fn push_identity(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;
    state.current_model_matrix = Some(Mat4::IDENTITY);
}

/// Set the current transform from a 4x4 matrix
///
/// # Arguments
/// * `matrix_ptr` — Pointer to 16 f32 values in column-major order
///
/// Column-major order means: [col0, col1, col2, col3] where each column is [x, y, z, w].
/// This is the same format used by glam and WGSL.
fn transform_set(mut caller: Caller<'_, ZXGameContext>, matrix_ptr: u32) {
    // Read the 16 floats from WASM memory using helper
    let Some(matrix) = read_wasm_matrix4x4(&caller, matrix_ptr, "transform_set") else {
        return;
    };

    let state = &mut caller.data_mut().ffi;
    let new_matrix = Mat4::from_cols_array(&matrix);
    state.current_model_matrix = Some(new_matrix);
}

/// Push a translated transform onto the stack
///
/// # Arguments
/// * `x`, `y`, `z` — Translation amounts
///
/// Reads the current transform, applies translation, and pushes the result.
fn push_translate(mut caller: Caller<'_, ZXGameContext>, x: f32, y: f32, z: f32) {
    let state = &mut caller.data_mut().ffi;
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
fn push_rotate_x(mut caller: Caller<'_, ZXGameContext>, angle_deg: f32) {
    let state = &mut caller.data_mut().ffi;
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
fn push_rotate_y(mut caller: Caller<'_, ZXGameContext>, angle_deg: f32) {
    let state = &mut caller.data_mut().ffi;
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
fn push_rotate_z(mut caller: Caller<'_, ZXGameContext>, angle_deg: f32) {
    let state = &mut caller.data_mut().ffi;
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
    mut caller: Caller<'_, ZXGameContext>,
    angle_deg: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
) {
    let state = &mut caller.data_mut().ffi;
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
fn push_scale(mut caller: Caller<'_, ZXGameContext>, x: f32, y: f32, z: f32) {
    let state = &mut caller.data_mut().ffi;
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
fn push_scale_uniform(mut caller: Caller<'_, ZXGameContext>, s: f32) {
    let state = &mut caller.data_mut().ffi;
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
