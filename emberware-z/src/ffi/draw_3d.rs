//! Immediate mode 3D drawing FFI functions
//!
//! Functions for drawing 3D triangles immediately (buffered on CPU, flushed at frame end).

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::graphics::vertex_stride;
use crate::state::ZFFIState;

/// Maximum vertex format value (all flags set: UV | COLOR | NORMAL | SKINNED)
const MAX_VERTEX_FORMAT: u8 = 15;

/// Register immediate mode 3D drawing FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
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

    // Capture bound_textures at command creation time (not deferred)
    // They are resolved to TextureHandle at render time via texture_map
    let textures = state.bound_textures;

    let cull_mode = crate::graphics::CullMode::from_u8(state.cull_mode);
    let blend_mode = crate::graphics::BlendMode::from_u8(state.blend_mode);

    // Allocate combined MVP+shading buffer index (lazy allocation with deduplication)
    let buffer_index = state.add_mvp_shading_state();

    // Record draw command directly
    state.render_pass.record_triangles(
        format,
        &vertex_data,
        buffer_index,
        textures,
        blend_mode,
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

    // Capture bound_textures at command creation time (not deferred)
    // They are resolved to TextureHandle at render time via texture_map
    let textures = state.bound_textures;

    let cull_mode = crate::graphics::CullMode::from_u8(state.cull_mode);
    let blend_mode = crate::graphics::BlendMode::from_u8(state.blend_mode);

    // Allocate combined MVP+shading buffer index (lazy allocation with deduplication)
    let buffer_index = state.add_mvp_shading_state();

    // Record draw command directly
    state.render_pass.record_triangles_indexed(
        format,
        &vertex_data,
        &index_data,
        buffer_index,
        textures,
        blend_mode,
        state.depth_test,
        cull_mode,
    );
}
