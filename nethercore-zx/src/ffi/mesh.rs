//! Mesh FFI functions (retained mode)
//!
//! Functions for loading and drawing retained meshes.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::helpers::{
    checked_mul, read_wasm_bytes, read_wasm_floats, read_wasm_u16s, validate_count_nonzero,
    validate_vertex_format,
};
use super::{ZXGameContext, guards::check_init_only};
use crate::graphics::{vertex_stride, vertex_stride_packed};
use crate::state::{PendingMesh, PendingMeshPacked};

/// Register mesh FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // Unpacked mesh loading (user convenience API)
    linker.func_wrap("env", "load_mesh", load_mesh)?;
    linker.func_wrap("env", "load_mesh_indexed", load_mesh_indexed)?;

    // Packed mesh loading (power user API, used by exporter)
    linker.func_wrap("env", "load_mesh_packed", load_mesh_packed)?;
    linker.func_wrap("env", "load_mesh_indexed_packed", load_mesh_indexed_packed)?;

    // Mesh drawing
    linker.func_wrap("env", "draw_mesh", draw_mesh)?;
    Ok(())
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
fn load_mesh(
    mut caller: Caller<'_, ZXGameContext>,
    data_ptr: u32,
    vertex_count: u32,
    format: u32,
) -> u32 {
    const FN_NAME: &str = "load_mesh";

    // Guard: init-only
    if let Err(e) = check_init_only(&caller, FN_NAME) {
        warn!("{}", e);
        return 0;
    }

    // Validate format and vertex count
    let Some(format) = validate_vertex_format(format, FN_NAME) else {
        return 0;
    };
    if !validate_count_nonzero(vertex_count, FN_NAME, "vertex_count") {
        return 0;
    }

    // Calculate data size with overflow checking
    let stride = vertex_stride(format);
    let Some(data_size) = checked_mul(vertex_count, stride, FN_NAME, "data size") else {
        return 0;
    };
    let float_count = (data_size / 4) as usize;

    // Read vertex data from WASM memory
    let Some(vertex_data) = read_wasm_floats(&caller, data_ptr, float_count, FN_NAME) else {
        return 0;
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().ffi;

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
    mut caller: Caller<'_, ZXGameContext>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) -> u32 {
    const FN_NAME: &str = "load_mesh_indexed";

    // Guard: init-only
    if let Err(e) = check_init_only(&caller, FN_NAME) {
        warn!("{}", e);
        return 0;
    }

    // Validate format
    let Some(format) = validate_vertex_format(format, FN_NAME) else {
        return 0;
    };

    // Validate counts
    if !validate_count_nonzero(vertex_count, FN_NAME, "vertex_count") {
        return 0;
    }
    if !validate_count_nonzero(index_count, FN_NAME, "index_count") {
        return 0;
    }
    if !index_count.is_multiple_of(3) {
        warn!(
            "{}: index_count {} is not a multiple of 3",
            FN_NAME, index_count
        );
        return 0;
    }

    // Calculate data sizes with overflow checking
    let stride = vertex_stride(format);
    let Some(vertex_data_size) = checked_mul(vertex_count, stride, FN_NAME, "vertex data size")
    else {
        return 0;
    };
    let float_count = (vertex_data_size / 4) as usize;

    // Read vertex data from WASM memory
    let Some(vertex_data) = read_wasm_floats(&caller, data_ptr, float_count, FN_NAME) else {
        return 0;
    };

    // Read index data from WASM memory
    let Some(index_data) = read_wasm_u16s(&caller, index_ptr, index_count as usize, FN_NAME) else {
        return 0;
    };

    // Validate indices are within bounds
    for &idx in &index_data {
        if idx as u32 >= vertex_count {
            warn!(
                "{}: index {} out of bounds (vertex_count = {})",
                FN_NAME, idx, vertex_count
            );
            return 0;
        }
    }

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().ffi;

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

    handle
}

/// Load packed mesh data (power user API)
///
/// # Arguments
/// * `data_ptr` - Pointer to packed vertex data (f16/snorm16/unorm8)
/// * `vertex_count` - Number of vertices
/// * `format` - Vertex format (0-15, base format without FORMAT_PACKED flag)
///
/// Returns mesh handle (>0) on success, 0 on failure.
fn load_mesh_packed(
    mut caller: Caller<'_, ZXGameContext>,
    data_ptr: u32,
    vertex_count: u32,
    format: u32,
) -> u32 {
    const FN_NAME: &str = "load_mesh_packed";

    // Guard: init-only
    if let Err(e) = check_init_only(&caller, FN_NAME) {
        warn!("{}", e);
        return 0;
    }

    // Validate format (0-15 only, no FORMAT_PACKED)
    let Some(format) = validate_vertex_format(format, FN_NAME) else {
        return 0;
    };

    // Validate vertex count
    if !validate_count_nonzero(vertex_count, FN_NAME, "vertex_count") {
        return 0;
    }

    // Calculate packed stride
    let stride = vertex_stride_packed(format) as usize;
    let byte_size = vertex_count as usize * stride;

    // Read packed bytes from WASM memory
    let Some(vertex_data) = read_wasm_bytes(&caller, data_ptr, byte_size, FN_NAME) else {
        return 0;
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().ffi;

    // Allocate a mesh handle
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    // Store packed mesh data for the graphics backend
    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format,
        vertex_data,
        index_data: None,
    });

    handle
}

/// Load an indexed packed mesh (power user API)
///
/// # Arguments
/// * `data_ptr` - Pointer to packed vertex data (f16/snorm16/unorm8)
/// * `vertex_count` - Number of vertices
/// * `index_ptr` - Pointer to index data (u16 array)
/// * `index_count` - Number of indices
/// * `format` - Vertex format (0-15, base format without FORMAT_PACKED flag)
///
/// Returns mesh handle (>0) on success, 0 on failure.
fn load_mesh_indexed_packed(
    mut caller: Caller<'_, ZXGameContext>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) -> u32 {
    const FN_NAME: &str = "load_mesh_indexed_packed";

    // Guard: init-only
    if let Err(e) = check_init_only(&caller, FN_NAME) {
        warn!("{}", e);
        return 0;
    }

    // Validate format (0-15 only, no FORMAT_PACKED)
    let Some(format) = validate_vertex_format(format, FN_NAME) else {
        return 0;
    };

    // Validate counts
    if !validate_count_nonzero(vertex_count, FN_NAME, "vertex_count") {
        return 0;
    }
    if !validate_count_nonzero(index_count, FN_NAME, "index_count") {
        return 0;
    }
    if !index_count.is_multiple_of(3) {
        warn!(
            "{}: index_count {} is not a multiple of 3",
            FN_NAME, index_count
        );
        return 0;
    }

    // Calculate sizes
    let stride = vertex_stride_packed(format) as usize;
    let vertex_byte_size = vertex_count as usize * stride;

    // Read packed vertex data from WASM memory
    let Some(vertex_data) = read_wasm_bytes(&caller, data_ptr, vertex_byte_size, FN_NAME) else {
        return 0;
    };

    // Read index data from WASM memory
    let Some(index_data) = read_wasm_u16s(&caller, index_ptr, index_count as usize, FN_NAME) else {
        return 0;
    };

    // Validate indices are within bounds
    for &idx in &index_data {
        if idx as u32 >= vertex_count {
            warn!(
                "{}: index {} out of bounds (vertex_count = {})",
                FN_NAME, idx, vertex_count
            );
            return 0;
        }
    }

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().ffi;

    // Allocate a mesh handle
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    // Store packed mesh data for the graphics backend
    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format,
        vertex_data,
        index_data: Some(index_data),
    });

    handle
}

/// Draw a retained mesh with current transform and render state
///
/// # Arguments
/// * `handle` — Mesh handle from load_mesh or load_mesh_indexed
///
/// The mesh is drawn using the current transform (from transform_* functions)
/// and render state (color, textures, depth test, cull mode, blend mode).
fn draw_mesh(mut caller: Caller<'_, ZXGameContext>, handle: u32) {
    if handle == 0 {
        warn!("draw_mesh: invalid handle 0");
        return;
    }

    let state = &mut caller.data_mut().ffi;

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
    state.render_pass.record_mesh(
        mesh_format,
        mesh_vertex_count,
        mesh_index_count,
        mesh_vertex_offset,
        mesh_index_offset,
        buffer_index,
        textures,
        cull_mode,
        viewport,
        pass_id,
    );
}
