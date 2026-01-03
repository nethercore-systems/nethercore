//! Immediate mode 3D drawing FFI functions
//!
//! Functions for drawing 3D triangles immediately (buffered on CPU, flushed at frame end).

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;
use super::helpers::{checked_mul, read_wasm_floats, read_wasm_u16s, validate_vertex_format};
use crate::graphics::vertex_stride;

/// Register immediate mode 3D drawing FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "draw_triangles", draw_triangles)?;
    linker.func_wrap("env", "draw_triangles_indexed", draw_triangles_indexed)?;
    Ok(())
}

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
    mut caller: Caller<'_, ZXGameContext>,
    data_ptr: u32,
    vertex_count: u32,
    format: u32,
) {
    const FN_NAME: &str = "draw_triangles";

    // Validate format
    let Some(format) = validate_vertex_format(format, FN_NAME) else {
        return;
    };

    // Validate vertex count
    if vertex_count == 0 {
        return; // Nothing to draw
    }
    if !vertex_count.is_multiple_of(3) {
        warn!(
            "{}: vertex_count {} is not a multiple of 3",
            FN_NAME, vertex_count
        );
        return;
    }

    // Calculate data size with overflow checking
    let stride = vertex_stride(format);
    let Some(data_size) = checked_mul(vertex_count, stride, FN_NAME, "data size") else {
        return;
    };
    let float_count = (data_size / 4) as usize;

    // Read vertex data from WASM memory
    let Some(vertex_data) = read_wasm_floats(&caller, data_ptr, float_count, FN_NAME) else {
        return;
    };

    let state = &mut caller.data_mut().ffi;

    // Capture bound_textures at command creation time (not deferred)
    // They are resolved to TextureHandle at render time via texture_map
    let textures = state.bound_textures;

    let cull_mode = state.cull_mode;

    // Capture current viewport for split-screen rendering
    let viewport = state.current_viewport;

    // Capture current pass_id for render pass ordering
    let pass_id = state.current_pass_id;

    // Allocate combined MVP+shading buffer index (lazy allocation with deduplication)
    let buffer_index = state.add_mvp_shading_state();

    // Record draw command directly
    state.render_pass.record_triangles(
        format,
        &vertex_data,
        buffer_index,
        textures,
        cull_mode,
        viewport,
        pass_id,
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
    mut caller: Caller<'_, ZXGameContext>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) {
    const FN_NAME: &str = "draw_triangles_indexed";

    // Validate format
    let Some(format) = validate_vertex_format(format, FN_NAME) else {
        return;
    };

    // Validate counts
    if vertex_count == 0 || index_count == 0 {
        return; // Nothing to draw
    }
    if !index_count.is_multiple_of(3) {
        warn!(
            "{}: index_count {} is not a multiple of 3",
            FN_NAME, index_count
        );
        return;
    }

    // Calculate data sizes with overflow checking
    let stride = vertex_stride(format);
    let Some(vertex_data_size) = checked_mul(vertex_count, stride, FN_NAME, "vertex data size")
    else {
        return;
    };
    let float_count = (vertex_data_size / 4) as usize;

    // Read vertex data from WASM memory
    let Some(vertex_data) = read_wasm_floats(&caller, data_ptr, float_count, FN_NAME) else {
        return;
    };

    // Read index data from WASM memory
    let Some(index_data) = read_wasm_u16s(&caller, index_ptr, index_count as usize, FN_NAME) else {
        return;
    };

    // Validate indices are within bounds
    for &idx in &index_data {
        if idx as u32 >= vertex_count {
            warn!(
                "{}: index {} out of bounds (vertex_count = {})",
                FN_NAME, idx, vertex_count
            );
            return;
        }
    }

    let state = &mut caller.data_mut().ffi;

    // Capture bound_textures at command creation time (not deferred)
    // They are resolved to TextureHandle at render time via texture_map
    let textures = state.bound_textures;

    let cull_mode = state.cull_mode;

    // Capture current viewport for split-screen rendering
    let viewport = state.current_viewport;

    // Capture current pass_id for render pass ordering
    let pass_id = state.current_pass_id;

    // Allocate combined MVP+shading buffer index (lazy allocation with deduplication)
    let buffer_index = state.add_mvp_shading_state();

    // Record draw command directly
    state.render_pass.record_triangles_indexed(
        format,
        &vertex_data,
        &index_data,
        buffer_index,
        textures,
        cull_mode,
        viewport,
        pass_id,
    );
}
