//! Asset loading FFI for EmberZ binary formats (.ewzmesh, .ewztex, .ewzsnd)
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

use super::get_wasm_memory;

use emberware_core::wasm::GameStateWithConsole;
use emberware_shared::formats::{EmberZMeshHeader, EmberZSoundHeader, EmberZTextureHeader};

use crate::audio::Sound;
use crate::console::ZInput;
use crate::graphics::vertex_stride_packed;
use crate::state::{PendingMeshPacked, PendingTexture, ZFFIState};

/// Register asset loading FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    // Load from EmberZ binary formats (Phase A - include_bytes! workflow)
    linker.func_wrap("env", "load_zmesh", load_zmesh)?;
    linker.func_wrap("env", "load_ztex", load_ztex)?;
    linker.func_wrap("env", "load_zsound", load_zsound)?;
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
        let Some(pixels) = header.width.checked_mul(header.height) else {
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

        (header.width, header.height, pixel_data)
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().console;

    // Allocate a texture handle
    let handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    // Store texture data for the graphics backend
    state.pending_textures.push(PendingTexture {
        handle,
        width,
        height,
        data: pixel_data,
    });

    info!("load_ztex: created texture {} ({}x{})", handle, width, height);

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
        let sample_bytes =
            &data[EmberZSoundHeader::SIZE..EmberZSoundHeader::SIZE + sample_size];
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

    info!("load_zsound: created sound {} ({} samples)", handle, sample_count);

    handle
}
