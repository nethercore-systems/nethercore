//! ROM data pack FFI functions
//!
//! Functions for loading assets from the bundled ROM data pack.
//! Assets loaded via `rom_*` go directly to VRAM/audio memory on the host,
//! bypassing WASM linear memory for efficient rollback.
//!
//! **All `rom_*` functions are init-only** — they can only be called during `init()`.

use std::sync::Arc;

use anyhow::{bail, Result};
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::audio::Sound;
use crate::console::ZInput;
use crate::state::{PendingMeshPacked, PendingTexture, ZFFIState};

/// Register ROM data pack FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
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
fn read_string_id(
    caller: &Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Option<String> {
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

/// Check if we're in init phase (init-only function guard)
fn check_init_only(caller: &Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, fn_name: &str) -> Result<()> {
    if !caller.data().game.in_init {
        bail!("{}: can only be called during init()", fn_name);
    }
    Ok(())
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
fn rom_texture(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    check_init_only(&caller, "rom_texture")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_texture: failed to read asset ID"))?;

    let state = &mut caller.data_mut().console;

    // Get data pack
    let data_pack = state
        .data_pack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("rom_texture: no data pack loaded"))?
        .clone();

    // Find texture in data pack
    let texture = data_pack
        .find_texture(&id)
        .ok_or_else(|| anyhow::anyhow!("rom_texture: texture '{}' not found in data pack", id))?;

    // Allocate handle and queue for upload
    let handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    state.pending_textures.push(PendingTexture {
        handle,
        width: texture.width,
        height: texture.height,
        data: texture.data.clone(),
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
fn rom_mesh(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    check_init_only(&caller, "rom_mesh")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_mesh: failed to read asset ID"))?;

    let state = &mut caller.data_mut().console;

    // Get data pack
    let data_pack = state
        .data_pack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("rom_mesh: no data pack loaded"))?
        .clone();

    // Find mesh in data pack
    let mesh = data_pack
        .find_mesh(&id)
        .ok_or_else(|| anyhow::anyhow!("rom_mesh: mesh '{}' not found in data pack", id))?;

    // Allocate handle and queue for upload
    let handle = state.next_mesh_handle;
    state.next_mesh_handle += 1;

    state.pending_meshes_packed.push(PendingMeshPacked {
        handle,
        format: mesh.format,
        vertex_data: mesh.vertex_data.clone(),
        index_data: Some(mesh.index_data.clone()),
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
/// hierarchy, and rest pose should be in WASM memory (generated by ember-export).
fn rom_skeleton(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    check_init_only(&caller, "rom_skeleton")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_skeleton: failed to read asset ID"))?;

    let state = &mut caller.data_mut().console;

    // Get data pack
    let data_pack = state
        .data_pack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("rom_skeleton: no data pack loaded"))?
        .clone();

    // Find skeleton in data pack
    let skeleton = data_pack
        .find_skeleton(&id)
        .ok_or_else(|| anyhow::anyhow!("rom_skeleton: skeleton '{}' not found in data pack", id))?;

    // For now, return the bone count as the handle
    // The game uses set_bones() to upload the animated matrices each frame
    // The IBMs from the skeleton are used in the animation system
    // TODO: Implement proper skeleton resource management when animation system is ready
    Ok(skeleton.bone_count)
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
fn rom_font(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    check_init_only(&caller, "rom_font")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_font: failed to read asset ID"))?;

    let state = &mut caller.data_mut().console;

    // Get data pack
    let data_pack = state
        .data_pack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("rom_font: no data pack loaded"))?
        .clone();

    // Find font in data pack
    let packed_font = data_pack
        .find_font(&id)
        .ok_or_else(|| anyhow::anyhow!("rom_font: font '{}' not found in data pack", id))?;

    // Upload font atlas as a texture
    let atlas_handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    state.pending_textures.push(PendingTexture {
        handle: atlas_handle,
        width: packed_font.atlas_width,
        height: packed_font.atlas_height,
        data: packed_font.atlas_data.clone(),
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
fn rom_sound(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    check_init_only(&caller, "rom_sound")?;

    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_sound: failed to read asset ID"))?;

    let state = &mut caller.data_mut().console;

    // Get data pack
    let data_pack = state
        .data_pack
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("rom_sound: no data pack loaded"))?
        .clone();

    // Find sound in data pack
    let sound = data_pack
        .find_sound(&id)
        .ok_or_else(|| anyhow::anyhow!("rom_sound: sound '{}' not found in data pack", id))?;

    // Allocate handle and register sound
    let handle = state.next_sound_handle;
    state.next_sound_handle += 1;

    // Create Sound resource (PCM data wrapped in Arc for efficient cloning during playback)
    let sound_resource = Sound {
        data: Arc::new(sound.data.clone()),
    };

    // Ensure sounds vector is large enough
    while state.sounds.len() <= handle as usize {
        state.sounds.push(None);
    }
    state.sounds[handle as usize] = Some(sound_resource);

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
fn rom_data_len(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_data_len: failed to read asset ID"))?;

    let state = &caller.data().console;

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
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
    dst_ptr: u32,
    max_len: u32,
) -> Result<u32> {
    let id = read_string_id(&caller, id_ptr, id_len)
        .ok_or_else(|| anyhow::anyhow!("rom_data: failed to read asset ID"))?;

    // Get data from data pack (clone the bytes we need)
    let data_bytes = {
        let state = &caller.data().console;
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
