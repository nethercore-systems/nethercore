//! ROM data pack FFI functions
//!
//! Functions for loading assets from the bundled ROM data pack.
//! Assets loaded via `rom_*` go directly to VRAM/audio memory on the host,
//! bypassing WASM linear memory for efficient rollback.
//!
//! **All `rom_*` functions are init-only** — they can only be called during `init()`.

use std::sync::Arc;

use anyhow::{Result, bail};
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::{ZXGameContext, guards::check_init_only};
use crate::audio::Sound;
use crate::state::{MAX_SKELETONS, PendingMeshPacked, PendingSkeleton, PendingTexture};
use zx_common::TextureFormat;

/// Register ROM data pack FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // GPU resources (return handles, uploaded to VRAM)
    linker.func_wrap("env", "rom_texture", rom_texture)?;
    linker.func_wrap("env", "rom_mesh", rom_mesh)?;
    linker.func_wrap("env", "rom_skeleton", rom_skeleton)?;
    linker.func_wrap("env", "rom_font", rom_font)?;
    linker.func_wrap("env", "rom_sound", rom_sound)?;

    // Raw data (copies into WASM linear memory)
    linker.func_wrap("env", "rom_data_len", rom_data_len)?;
    linker.func_wrap("env", "rom_data", rom_data)?;

    Ok(())
}

/// Read a string ID from WASM memory
fn read_string_id(caller: &Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> Option<String> {
    let memory = caller.data().game.memory?;
    let data = memory.data(caller);

    let start = id_ptr as usize;
    let len = id_len as usize;
    let end = start.checked_add(len)?;

    if end > data.len() {
        warn!(
            "rom: string ID access out of bounds ({}-{}, memory size {})",
            start,
            end,
            data.len()
        );
        return None;
    }

    let bytes = &data[start..end];
    String::from_utf8(bytes.to_vec()).ok()
}

// ═══════════════════════════════════════════════════════════════════════════
// GPU RESOURCES (return handles, uploaded to VRAM)
// ═══════════════════════════════════════════════════════════════════════════

/// Load a texture from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Texture handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
fn rom_texture(mut caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> Result<u32> {
    check_init_only(&caller, "rom_texture")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_texture: failed to read asset ID at ptr=0x{:08X}, len={}", id_ptr, id_len))?;

    // Extract texture data from data pack (read-only access)
    let (width, height, format, data) = {
        let state = &caller.data().ffi;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_texture: no data pack loaded"))?;
        let texture = data_pack.find_texture(&id).ok_or_else(|| {
            anyhow::anyhow!("rom_texture: texture '{}' not found in data pack", id)
        })?;
        (
            texture.width as u32,
            texture.height as u32,
            texture.format,
            texture.data.clone(),
        )
    };

    // Now mutate state to allocate handle and queue upload
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    state.pending_textures.push(PendingTexture {
        handle,
        width,
        height,
        format,
        data,
    });

    Ok(handle)
}

/// Load a mesh from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Mesh handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
fn rom_mesh(mut caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> Result<u32> {
    check_init_only(&caller, "rom_mesh")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_mesh: failed to read asset ID at ptr=0x{:08X}, len={}", id_ptr, id_len))?;

    // Extract mesh data from data pack (read-only access)
    let (format, vertex_data, index_data) = {
        let state = &caller.data().ffi;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_mesh: no data pack loaded"))?;
        let mesh = data_pack
            .find_mesh(&id)
            .ok_or_else(|| anyhow::anyhow!("rom_mesh: mesh '{}' not found in data pack", id))?;
        (
            mesh.format,
            mesh.vertex_data.clone(),
            mesh.index_data.clone(),
        )
    };

    // Now mutate state to allocate handle and queue upload
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format,
        vertex_data,
        index_data: Some(index_data),
    });

    Ok(handle)
}

/// Load skeleton inverse bind matrices from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Skeleton handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
///
/// **Note:** This uploads ONLY the inverse bind matrices to GPU. Bone names,
/// hierarchy, and rest pose should be in WASM memory (generated by nether-export).
fn rom_skeleton(mut caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> Result<u32> {
    check_init_only(&caller, "rom_skeleton")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_skeleton: failed to read asset ID at ptr=0x{:08X}, len={}", id_ptr, id_len))?;

    // Extract skeleton data from data pack (read-only access)
    let (bone_count, inverse_bind) = {
        let state = &caller.data().ffi;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_skeleton: no data pack loaded"))?;
        let skeleton = data_pack.find_skeleton(&id).ok_or_else(|| {
            anyhow::anyhow!("rom_skeleton: skeleton '{}' not found in data pack", id)
        })?;

        // Check skeleton limit
        let total_skeletons = state.skeletons.len() + state.pending_skeletons.len();
        if total_skeletons >= MAX_SKELETONS {
            bail!(
                "rom_skeleton: maximum skeleton count {} exceeded",
                MAX_SKELETONS
            );
        }

        (skeleton.bone_count, skeleton.inverse_bind_matrices.clone())
    };

    // Now mutate state to allocate handle and queue upload
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_skeleton_handle;
    state.next_skeleton_handle += 1;

    state.pending_skeletons.push(PendingSkeleton {
        handle,
        inverse_bind,
        bone_count,
    });

    Ok(handle)
}

/// Load a font from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Texture handle for font atlas (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
///
/// **Note:** This uploads the font atlas as a texture. The returned handle
/// can be used with text rendering functions. Full BMFont-style variable-width
/// font support will use the glyph metrics stored in the packed font.
fn rom_font(mut caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> Result<u32> {
    check_init_only(&caller, "rom_font")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_font: failed to read asset ID at ptr=0x{:08X}, len={}", id_ptr, id_len))?;

    // Extract font atlas data from data pack (read-only access)
    let (atlas_width, atlas_height, atlas_data) = {
        let state = &caller.data().ffi;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_font: no data pack loaded"))?;
        let packed_font = data_pack
            .find_font(&id)
            .ok_or_else(|| anyhow::anyhow!("rom_font: font '{}' not found in data pack", id))?;
        (
            packed_font.atlas_width,
            packed_font.atlas_height,
            packed_font.atlas_data.clone(),
        )
    };

    // Now mutate state to allocate handle and queue upload
    let state = &mut caller.data_mut().ffi;
    let atlas_handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    // Font atlases are always RGBA8 (uncompressed for crisp text)
    state.pending_textures.push(PendingTexture {
        handle: atlas_handle,
        width: atlas_width,
        height: atlas_height,
        format: TextureFormat::Rgba8,
        data: atlas_data,
    });

    // Return the atlas texture handle
    // Games can use this with sprite drawing for custom text rendering,
    // or the built-in text_draw() system once BMFont support is added
    Ok(atlas_handle)
}

/// Load a sound from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Sound handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
fn rom_sound(mut caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> Result<u32> {
    check_init_only(&caller, "rom_sound")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_sound: failed to read asset ID at ptr=0x{:08X}, len={}", id_ptr, id_len))?;

    // Extract sound data from data pack (read-only access)
    let sound_data = {
        let state = &caller.data().ffi;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_sound: no data pack loaded"))?;
        let sound = data_pack
            .find_sound(&id)
            .ok_or_else(|| anyhow::anyhow!("rom_sound: sound '{}' not found in data pack", id))?;
        sound.data.clone()
    };

    // Now mutate state to allocate handle and register sound
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_sound_handle;
    state.next_sound_handle += 1;

    // Create Sound resource (PCM data wrapped in Arc for efficient cloning during playback)
    let sound_resource = Sound {
        data: Arc::new(sound_data),
    };

    // Ensure sounds vector is large enough
    while state.sounds.len() <= handle as usize {
        state.sounds.push(None);
    }
    state.sounds[handle as usize] = Some(sound_resource);

    // Store ID -> handle mapping for tracker sample resolution
    state.sound_id_to_handle.insert(id, handle);

    Ok(handle)
}

// ═══════════════════════════════════════════════════════════════════════════
// RAW DATA (copies into WASM linear memory)
// ═══════════════════════════════════════════════════════════════════════════

/// Get the byte size of raw data in the ROM data pack
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Byte count on success. Traps if not found.
///
/// Use this to allocate a buffer before calling `rom_data()`.
fn rom_data_len(caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> Result<u32> {
    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_data_len: failed to read asset ID at ptr=0x{:08X}, len={}", id_ptr, id_len))?;

    let state = &caller.data().ffi;

    // Get data pack
    let data_pack = state
        .data_pack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("rom_data_len: no data pack loaded"))?;

    // Find data in data pack
    let data = data_pack
        .find_data(&id)
        .ok_or_else(|| anyhow::anyhow!("rom_data_len: data '{}' not found in data pack", id))?;

    Ok(data.data.len() as u32)
}

/// Copy raw data from ROM data pack into WASM linear memory
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
/// * `dst_ptr` — Pointer to destination buffer in WASM memory
/// * `max_len` — Maximum bytes to copy (size of destination buffer)
///
/// # Returns
/// Bytes written on success. Traps on failure.
///
/// The game must allocate the destination buffer before calling this function.
/// Use `rom_data_len()` to determine the required buffer size.
fn rom_data(
    mut caller: Caller<'_, ZXGameContext>,
    id_ptr: u32,
    id_len: u32,
    dst_ptr: u32,
    max_len: u32,
) -> Result<u32> {
    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_data: failed to read asset ID at ptr=0x{:08X}, len={}", id_ptr, id_len))?;

    // Get data from data pack (clone the bytes we need)
    let data_bytes = {
        let state = &caller.data().ffi;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_data: no data pack loaded"))?;

        data_pack
            .find_data(&id)
            .ok_or_else(|| anyhow::anyhow!("rom_data: data '{}' not found in data pack", id))?
            .data
            .clone()
    };

    // Calculate how many bytes to copy
    let bytes_to_copy = (data_bytes.len() as u32).min(max_len) as usize;

    // Get WASM memory and copy data
    let memory = caller
        .data()
        .game
        .memory
        .ok_or_else(|| anyhow::anyhow!("rom_data: no WASM memory available"))?;

    let dst = dst_ptr as usize;
    let mem_data = memory.data_mut(&mut caller);

    if dst + bytes_to_copy > mem_data.len() {
        bail!(
            "rom_data: destination buffer ({} bytes at {}) exceeds memory bounds ({})",
            bytes_to_copy,
            dst,
            mem_data.len()
        );
    }

    mem_data[dst..dst + bytes_to_copy].copy_from_slice(&data_bytes[..bytes_to_copy]);

    Ok(bytes_to_copy as u32)
}
