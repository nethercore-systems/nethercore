//! Mesh FFI functions (retained mode)
//!
//! Functions for loading and drawing retained meshes.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Extern, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::graphics::{vertex_stride, vertex_stride_packed};
use crate::state::{PendingMesh, PendingMeshPacked, ZFFIState};

/// Maximum vertex format value (all flags set: UV | COLOR | NORMAL | SKINNED)
const MAX_VERTEX_FORMAT: u8 = 15;

/// Register mesh FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
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

/// Load packed mesh data (power user API)
///
/// # Arguments
/// * `data_ptr` - Pointer to packed vertex data (f16/snorm16/unorm8)
/// * `vertex_count` - Number of vertices
/// * `format` - Vertex format (0-15, base format without FORMAT_PACKED flag)
///
/// Returns mesh handle (>0) on success, 0 on failure.
fn load_mesh_packed(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    vertex_count: u32,
    format: u32,
) -> u32 {
    // Validate format (0-15 only, no FORMAT_PACKED)
    if format >= 16 {
        warn!("load_mesh_packed: format must be 0-15 (got {})", format);
        return 0;
    }
    let format = format as u8;

    // Validate vertex count
    if vertex_count == 0 {
        warn!("load_mesh_packed: vertex_count cannot be 0");
        return 0;
    }

    // Calculate packed stride
    let stride = vertex_stride_packed(format) as usize;
    let byte_size = vertex_count as usize * stride;

    // Get memory reference
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => {
            warn!("load_mesh_packed: failed to get WASM memory");
            return 0;
        }
    };

    let ptr = data_ptr as usize;

    // Copy packed bytes from WASM memory
    let vertex_data: Vec<u8> = {
        let mem_data = memory.data(&caller);

        if ptr + byte_size > mem_data.len() {
            warn!(
                "load_mesh_packed: vertex data ({} bytes at {}) exceeds memory bounds ({})",
                byte_size,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        mem_data[ptr..ptr + byte_size].to_vec()
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

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

    info!(
        "load_mesh_packed: created mesh {} with {} vertices, format {}, {} bytes",
        handle, vertex_count, format, byte_size
    );

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
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    vertex_count: u32,
    index_ptr: u32,
    index_count: u32,
    format: u32,
) -> u32 {
    // Validate format (0-15 only, no FORMAT_PACKED)
    if format >= 16 {
        warn!(
            "load_mesh_indexed_packed: format must be 0-15 (got {})",
            format
        );
        return 0;
    }
    let format = format as u8;

    // Validate counts
    if vertex_count == 0 {
        warn!("load_mesh_indexed_packed: vertex_count cannot be 0");
        return 0;
    }
    if index_count == 0 {
        warn!("load_mesh_indexed_packed: index_count cannot be 0");
        return 0;
    }
    if !index_count.is_multiple_of(3) {
        warn!(
            "load_mesh_indexed_packed: index_count {} is not a multiple of 3",
            index_count
        );
        return 0;
    }

    // Calculate sizes
    let stride = vertex_stride_packed(format) as usize;
    let vertex_byte_size = vertex_count as usize * stride;
    let index_byte_size = index_count as usize * 2; // u16 indices

    // Get memory reference
    let memory = match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => mem,
        _ => {
            warn!("load_mesh_indexed_packed: failed to get WASM memory");
            return 0;
        }
    };

    let vertex_ptr = data_ptr as usize;
    let idx_ptr = index_ptr as usize;

    // Copy packed data from WASM memory
    let (vertex_data, index_data): (Vec<u8>, Vec<u16>) = {
        let mem_data = memory.data(&caller);

        if vertex_ptr + vertex_byte_size > mem_data.len() {
            warn!(
                "load_mesh_indexed_packed: vertex data ({} bytes at {}) exceeds memory bounds ({})",
                vertex_byte_size,
                vertex_ptr,
                mem_data.len()
            );
            return 0;
        }

        if idx_ptr + index_byte_size > mem_data.len() {
            warn!(
                "load_mesh_indexed_packed: index data ({} bytes at {}) exceeds memory bounds ({})",
                index_byte_size,
                idx_ptr,
                mem_data.len()
            );
            return 0;
        }

        let vertex_bytes = mem_data[vertex_ptr..vertex_ptr + vertex_byte_size].to_vec();

        let index_bytes = &mem_data[idx_ptr..idx_ptr + index_byte_size];
        let indices: &[u16] = bytemuck::cast_slice(index_bytes);

        // Validate indices are within bounds
        for &idx in indices {
            if idx as u32 >= vertex_count {
                warn!(
                    "load_mesh_indexed_packed: index {} out of bounds (vertex_count = {})",
                    idx, vertex_count
                );
                return 0;
            }
        }

        (vertex_bytes, indices.to_vec())
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

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

    info!(
        "load_mesh_indexed_packed: created mesh {} with {} vertices, {} indices, format {}, {} bytes",
        handle, vertex_count, index_count, format, vertex_byte_size
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
    let blend_mode = crate::graphics::BlendMode::from_u8(state.blend_mode);

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
        blend_mode,
        state.depth_test,
        cull_mode,
    );
}
