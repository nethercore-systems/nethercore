//! Asset loading FFI for EmberZ binary formats (.ewzmesh, .ewztex, .ewzsnd, .ewzskel)
//!
//! These functions load assets from POD EmberZ binary formats.
//! No magic bytes - format is determined by which function is called.
//!
//! Usage from game code:
//! ```text
//! static MESH_DATA: &[u8] = include_bytes!("player.ewzmesh");
//! let handle = load_zmesh(MESH_DATA.as_ptr() as u32, MESH_DATA.len() as u32);
//! ```
//!
//! Note: `rom_*` functions are reserved for Phase B datapack API where assets
//! are loaded by string ID from a host-side datapack.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use super::{get_wasm_memory, guards::check_init_only};

use emberware_core::wasm::GameStateWithConsole;
use z_common::TextureFormat;
use z_common::formats::{
    EmberZMeshHeader, EmberZSkeletonHeader, EmberZSoundHeader, EmberZTextureHeader,
};

use crate::audio::Sound;
use crate::console::ZInput;
use crate::graphics::vertex_stride_packed;
use crate::state::{
    BoneMatrix3x4, MAX_BONES, MAX_SKELETONS, PendingMeshPacked, PendingSkeleton, PendingTexture,
    ZFFIState,
};

/// Register asset loading FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    // Load from EmberZ binary formats (Phase A - include_bytes! workflow)
    linker.func_wrap("env", "load_zmesh", load_zmesh)?;
    linker.func_wrap("env", "load_ztex", load_ztex)?;
    linker.func_wrap("env", "load_zsound", load_zsound)?;
    linker.func_wrap("env", "load_zskeleton", load_zskeleton)?;
    Ok(())
}

/// Load a mesh from EmberZ mesh (.ewzmesh) binary format
///
/// # Arguments
/// * `data_ptr` — Pointer to .ewzmesh binary data
/// * `data_len` — Length of the data in bytes
///
/// Returns mesh handle (>0) on success, 0 on failure.
fn load_zmesh(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    data_len: u32,
) -> u32 {
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "load_zmesh") {
        warn!("{}", e);
        return 0;
    }

    let data_len = data_len as usize;

    // Validate minimum size
    if data_len < EmberZMeshHeader::SIZE {
        warn!(
            "load_zmesh: data too small ({} bytes, need at least {})",
            data_len,
            EmberZMeshHeader::SIZE
        );
        return 0;
    }

    // Get memory reference
    let Some(memory) = get_wasm_memory(&mut caller) else {
        warn!("load_zmesh: failed to get WASM memory");
        return 0;
    };

    let ptr = data_ptr as usize;

    // Read and parse header, then copy vertex/index data
    let (format, vertex_count, index_count, vertex_data, index_data) = {
        let mem_data = memory.data(&caller);

        if ptr + data_len > mem_data.len() {
            warn!(
                "load_zmesh: data ({} bytes at {}) exceeds memory bounds ({})",
                data_len,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        let data = &mem_data[ptr..ptr + data_len];

        // Parse header
        let Some(header) = EmberZMeshHeader::from_bytes(data) else {
            warn!("load_zmesh: failed to parse header");
            return 0;
        };

        // Validate format
        if header.format > 15 {
            warn!("load_zmesh: invalid format {} (max 15)", header.format);
            return 0;
        }

        // Calculate sizes
        let stride = vertex_stride_packed(header.format) as usize;
        let vertex_size = header.vertex_count as usize * stride;
        let index_size = header.index_count as usize * 2; // u16 indices

        let expected_size = EmberZMeshHeader::SIZE + vertex_size + index_size;
        if data_len < expected_size {
            warn!(
                "load_zmesh: data too small ({} bytes, need {} for {} verts + {} indices)",
                data_len, expected_size, header.vertex_count, header.index_count
            );
            return 0;
        }

        // Copy vertex data
        let vertex_start = EmberZMeshHeader::SIZE;
        let vertex_data = data[vertex_start..vertex_start + vertex_size].to_vec();

        // Copy index data if present
        let index_data = if header.index_count > 0 {
            let index_start = vertex_start + vertex_size;
            let index_bytes = &data[index_start..index_start + index_size];
            let indices: &[u16] = bytemuck::cast_slice(index_bytes);

            // Validate indices are within bounds
            for &idx in indices {
                if idx as u32 >= header.vertex_count {
                    warn!(
                        "load_zmesh: index {} out of bounds (vertex_count = {})",
                        idx, header.vertex_count
                    );
                    return 0;
                }
            }

            Some(indices.to_vec())
        } else {
            None
        };

        (
            header.format,
            header.vertex_count,
            header.index_count,
            vertex_data,
            index_data,
        )
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
        index_data,
    });

    info!(
        "load_zmesh: created mesh {} with {} vertices, {} indices, format {}",
        handle, vertex_count, index_count, format
    );

    handle
}

/// Load a texture from EmberZ texture (.ewztex) binary format
///
/// # Arguments
/// * `data_ptr` — Pointer to .ewztex binary data
/// * `data_len` — Length of the data in bytes
///
/// Returns texture handle (>0) on success, 0 on failure.
fn load_ztex(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    data_len: u32,
) -> u32 {
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "load_ztex") {
        warn!("{}", e);
        return 0;
    }

    let data_len = data_len as usize;

    // Validate minimum size
    if data_len < EmberZTextureHeader::SIZE {
        warn!(
            "load_ztex: data too small ({} bytes, need at least {})",
            data_len,
            EmberZTextureHeader::SIZE
        );
        return 0;
    }

    // Get memory reference
    let Some(memory) = get_wasm_memory(&mut caller) else {
        warn!("load_ztex: failed to get WASM memory");
        return 0;
    };

    let ptr = data_ptr as usize;

    // Read and parse header, then copy pixel data
    let (width, height, pixel_data) = {
        let mem_data = memory.data(&caller);

        if ptr + data_len > mem_data.len() {
            warn!(
                "load_ztex: data ({} bytes at {}) exceeds memory bounds ({})",
                data_len,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        let data = &mem_data[ptr..ptr + data_len];

        // Parse header
        let Some(header) = EmberZTextureHeader::from_bytes(data) else {
            warn!("load_ztex: failed to parse header");
            return 0;
        };

        // Validate dimensions
        if header.width == 0 || header.height == 0 {
            warn!(
                "load_ztex: invalid dimensions {}x{}",
                header.width, header.height
            );
            return 0;
        }

        // Calculate pixel data size (RGBA8 = 4 bytes per pixel)
        let width = header.width as u32;
        let height = header.height as u32;
        let Some(pixels) = width.checked_mul(height) else {
            warn!(
                "load_ztex: dimensions overflow ({}x{})",
                header.width, header.height
            );
            return 0;
        };
        let Some(pixel_size) = pixels.checked_mul(4) else {
            warn!(
                "load_ztex: pixel size overflow ({}x{})",
                header.width, header.height
            );
            return 0;
        };
        let pixel_size = pixel_size as usize;

        let expected_size = EmberZTextureHeader::SIZE + pixel_size;
        if data_len < expected_size {
            warn!(
                "load_ztex: data too small ({} bytes, need {} for {}x{})",
                data_len, expected_size, header.width, header.height
            );
            return 0;
        }

        // Copy pixel data
        let pixel_data =
            data[EmberZTextureHeader::SIZE..EmberZTextureHeader::SIZE + pixel_size].to_vec();

        (width, height, pixel_data)
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

    // Allocate a texture handle
    let handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    // Store texture data for the graphics backend
    // load_ztex() always uses RGBA8 format (embedded binary format)
    state.pending_textures.push(PendingTexture {
        handle,
        width,
        height,
        format: TextureFormat::Rgba8,
        data: pixel_data,
    });

    info!(
        "load_ztex: created texture {} ({}x{})",
        handle, width, height
    );

    handle
}

/// Load audio from EmberZ sound (.ewzsnd) binary format
///
/// # Arguments
/// * `data_ptr` — Pointer to .ewzsnd binary data
/// * `data_len` — Length of the data in bytes
///
/// Returns sound handle (>0) on success, 0 on failure.
fn load_zsound(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    data_len: u32,
) -> u32 {
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "load_zsound") {
        warn!("{}", e);
        return 0;
    }

    let data_len = data_len as usize;

    // Validate minimum size
    if data_len < EmberZSoundHeader::SIZE {
        warn!(
            "load_zsound: data too small ({} bytes, need at least {})",
            data_len,
            EmberZSoundHeader::SIZE
        );
        return 0;
    }

    // Get memory reference
    let Some(memory) = get_wasm_memory(&mut caller) else {
        warn!("load_zsound: failed to get WASM memory");
        return 0;
    };

    let ptr = data_ptr as usize;

    // Read and parse header, then copy sample data
    let (sample_count, samples) = {
        let mem_data = memory.data(&caller);

        if ptr + data_len > mem_data.len() {
            warn!(
                "load_zsound: data ({} bytes at {}) exceeds memory bounds ({})",
                data_len,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        let data = &mem_data[ptr..ptr + data_len];

        // Parse header
        let Some(header) = EmberZSoundHeader::from_bytes(data) else {
            warn!("load_zsound: failed to parse header");
            return 0;
        };

        // Calculate sample data size (PCM16 = 2 bytes per sample)
        let sample_size = header.sample_count as usize * 2;

        let expected_size = EmberZSoundHeader::SIZE + sample_size;
        if data_len < expected_size {
            warn!(
                "load_zsound: data too small ({} bytes, need {} for {} samples)",
                data_len, expected_size, header.sample_count
            );
            return 0;
        }

        // Copy sample data
        let sample_bytes = &data[EmberZSoundHeader::SIZE..EmberZSoundHeader::SIZE + sample_size];
        let samples: &[i16] = bytemuck::cast_slice(sample_bytes);

        (header.sample_count, samples.to_vec())
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

    // Create Sound and add to sounds vec (matches load_sound pattern)
    let sound = Sound {
        data: std::sync::Arc::new(samples),
    };

    let handle = state.next_sound_handle;
    state.next_sound_handle += 1;

    // Resize sounds vec if needed
    if handle as usize >= state.sounds.len() {
        state.sounds.resize(handle as usize + 1, None);
    }
    state.sounds[handle as usize] = Some(sound);

    info!(
        "load_zsound: created sound {} ({} samples)",
        handle, sample_count
    );

    handle
}

/// Load a skeleton from EmberZ skeleton (.ewzskel) binary format
///
/// # Arguments
/// * `data_ptr` — Pointer to .ewzskel binary data
/// * `data_len` — Length of the data in bytes
///
/// Returns skeleton handle (>0) on success, 0 on failure.
///
/// # Binary Format
/// ```text
/// 0x00: bone_count u32
/// 0x04: reserved u32
/// 0x08: inverse_bind_matrices (bone_count × 48 bytes, 3×4 column-major)
/// ```
fn load_zskeleton(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    data_len: u32,
) -> u32 {
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "load_zskeleton") {
        warn!("{}", e);
        return 0;
    }

    let data_len = data_len as usize;

    // Validate minimum size
    if data_len < EmberZSkeletonHeader::SIZE {
        warn!(
            "load_zskeleton: data too small ({} bytes, need at least {})",
            data_len,
            EmberZSkeletonHeader::SIZE
        );
        return 0;
    }

    // Get memory reference
    let Some(memory) = get_wasm_memory(&mut caller) else {
        warn!("load_zskeleton: failed to get WASM memory");
        return 0;
    };

    let ptr = data_ptr as usize;

    // Read and parse header, then copy inverse bind matrices
    let (bone_count, inverse_bind) = {
        let mem_data = memory.data(&caller);

        if ptr + data_len > mem_data.len() {
            warn!(
                "load_zskeleton: data ({} bytes at {}) exceeds memory bounds ({})",
                data_len,
                ptr,
                mem_data.len()
            );
            return 0;
        }

        let data = &mem_data[ptr..ptr + data_len];

        // Parse header
        let Some(header) = EmberZSkeletonHeader::from_bytes(data) else {
            warn!("load_zskeleton: failed to parse header");
            return 0;
        };

        // Validate bone count
        if header.bone_count == 0 {
            warn!("load_zskeleton: bone_count is 0");
            return 0;
        }
        if header.bone_count > MAX_BONES as u32 {
            warn!(
                "load_zskeleton: bone_count {} exceeds maximum {}",
                header.bone_count, MAX_BONES
            );
            return 0;
        }

        // Calculate expected data size (48 bytes per bone matrix)
        let matrix_size = 48usize;
        let matrices_size = header.bone_count as usize * matrix_size;
        let expected_size = EmberZSkeletonHeader::SIZE + matrices_size;

        if data_len < expected_size {
            warn!(
                "load_zskeleton: data too small ({} bytes, need {} for {} bones)",
                data_len, expected_size, header.bone_count
            );
            return 0;
        }

        // Parse inverse bind matrices (column-major input, row-major output for GPU)
        let mut inverse_bind = Vec::with_capacity(header.bone_count as usize);
        for i in 0..header.bone_count as usize {
            let offset = EmberZSkeletonHeader::SIZE + i * matrix_size;
            let matrix_bytes = &data[offset..offset + matrix_size];

            // Convert bytes to f32 array (12 floats in column-major order)
            let mut floats = [0.0f32; 12];
            for (j, float) in floats.iter_mut().enumerate() {
                let byte_offset = j * 4;
                let bytes = [
                    matrix_bytes[byte_offset],
                    matrix_bytes[byte_offset + 1],
                    matrix_bytes[byte_offset + 2],
                    matrix_bytes[byte_offset + 3],
                ];
                *float = f32::from_le_bytes(bytes);
            }

            // Transpose column-major input to row-major storage (for GPU alignment)
            // Input:  [col0.x, col0.y, col0.z, col1.x, col1.y, col1.z, col2.x, col2.y, col2.z, tx, ty, tz]
            // Output: row0 = [col0.x, col1.x, col2.x, tx]
            //         row1 = [col0.y, col1.y, col2.y, ty]
            //         row2 = [col0.z, col1.z, col2.z, tz]
            let matrix = BoneMatrix3x4 {
                row0: [floats[0], floats[3], floats[6], floats[9]],
                row1: [floats[1], floats[4], floats[7], floats[10]],
                row2: [floats[2], floats[5], floats[8], floats[11]],
            };
            inverse_bind.push(matrix);
        }

        (header.bone_count, inverse_bind)
    };

    // Check skeleton limit before allocating
    let state = &caller.data().console;
    let total_skeletons = state.skeletons.len() + state.pending_skeletons.len();
    if total_skeletons >= MAX_SKELETONS {
        warn!(
            "load_zskeleton: maximum skeleton count {} exceeded",
            MAX_SKELETONS
        );
        return 0;
    }

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

    // Allocate a skeleton handle
    let handle = state.next_skeleton_handle;
    state.next_skeleton_handle += 1;

    // Store pending skeleton for GPU upload
    state.pending_skeletons.push(PendingSkeleton {
        handle,
        inverse_bind,
        bone_count,
    });

    info!(
        "load_zskeleton: created skeleton {} with {} bones",
        handle, bone_count
    );

    handle
}
