//! Billboard drawing FFI functions
//!
//! Functions for drawing camera-facing quads (billboards) in 3D space.

use anyhow::Result;
use glam::Mat4;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;

/// Register billboard drawing FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "draw_billboard", draw_billboard)?;
    linker.func_wrap("env", "draw_billboard_region", draw_billboard_region)?;
    Ok(())
}

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
fn draw_billboard(mut caller: Caller<'_, ZXGameContext>, w: f32, h: f32, mode: u32, color: u32) {
    // Validate mode
    if !(1..=4).contains(&mode) {
        warn!("draw_billboard: invalid mode {} (must be 1-4)", mode);
        return;
    }

    let state = &mut caller.data_mut().ffi;

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
    mut caller: Caller<'_, ZXGameContext>,
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

    let state = &mut caller.data_mut().ffi;

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
