//! Keyframe loading functions (init-only)

use anyhow::{Result, bail};
use wasmtime::Caller;

use zx_common::formats::NetherZXAnimationHeader;

use crate::ffi::{ZXGameContext, guards::check_init_only};
use crate::state::{MAX_BONES, MAX_KEYFRAME_COLLECTIONS, PendingKeyframes};

/// Load keyframes from WASM memory
///
/// # Arguments
/// * `data_ptr` — Pointer to .nczxanim data in WASM memory
/// * `byte_size` — Total size of the data in bytes
///
/// # Returns
/// Keyframe collection handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
pub(super) fn keyframes_load(
    mut caller: Caller<'_, ZXGameContext>,
    data_ptr: u32,
    byte_size: u32,
) -> Result<u32> {
    check_init_only(&caller, "keyframes_load")?;

    // Check keyframe collection limit
    let state = &caller.data().ffi;
    let total_keyframes = state.keyframes.len() + state.pending_keyframes.len();
    if total_keyframes >= MAX_KEYFRAME_COLLECTIONS {
        bail!(
            "keyframes_load: maximum keyframe collection count {} exceeded",
            MAX_KEYFRAME_COLLECTIONS
        );
    }

    // Get WASM memory
    let memory = caller
        .data()
        .game
        .memory
        .ok_or_else(|| anyhow::anyhow!("keyframes_load: no WASM memory available"))?;

    let data = memory.data(&caller);
    let start = data_ptr as usize;
    let size = byte_size as usize;

    if start + size > data.len() {
        bail!(
            "keyframes_load: memory access out of bounds ({} + {} > {})",
            start,
            size,
            data.len()
        );
    }

    // Parse header
    let header_bytes = &data[start..start + NetherZXAnimationHeader::SIZE.min(size)];
    let header = NetherZXAnimationHeader::from_bytes(header_bytes)
        .ok_or_else(|| anyhow::anyhow!("keyframes_load: invalid header"))?;

    if !header.validate() {
        bail!(
            "keyframes_load: invalid header (bone_count={}, frame_count={}, flags={})",
            header.bone_count,
            header.frame_count,
            header.flags
        );
    }

    // Validate data size
    let expected_size = header.file_size();
    if size < expected_size {
        bail!(
            "keyframes_load: data too small ({} bytes, expected {})",
            size,
            expected_size
        );
    }

    // Check bone count against limit
    if header.bone_count as usize > MAX_BONES {
        bail!(
            "keyframes_load: bone_count {} exceeds MAX_BONES {}",
            header.bone_count,
            MAX_BONES
        );
    }

    // Copy keyframe data (skip header)
    let data_start = start + NetherZXAnimationHeader::SIZE;
    let data_len = header.data_size();
    let keyframe_data = data[data_start..data_start + data_len].to_vec();

    // Allocate handle and queue pending load
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_keyframe_handle;
    state.next_keyframe_handle += 1;

    state.pending_keyframes.push(PendingKeyframes {
        handle,
        bone_count: header.bone_count,
        frame_count: header.frame_count,
        data: keyframe_data,
    });

    tracing::info!(
        "keyframes_load: queued handle {} ({} bones, {} frames)",
        handle,
        header.bone_count,
        header.frame_count
    );

    Ok(handle)
}

/// Load keyframes from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Keyframe collection handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
pub(super) fn rom_keyframes(
    mut caller: Caller<'_, ZXGameContext>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    check_init_only(&caller, "rom_keyframes")?;

    // Read asset ID from WASM memory
    let id = {
        let memory = caller
            .data()
            .game
            .memory
            .ok_or_else(|| anyhow::anyhow!("rom_keyframes: no WASM memory available"))?;
        let data = memory.data(&caller);
        let start = id_ptr as usize;
        let len = id_len as usize;

        if start + len > data.len() {
            bail!("rom_keyframes: string ID access out of bounds");
        }

        String::from_utf8(data[start..start + len].to_vec())
            .map_err(|_| anyhow::anyhow!("rom_keyframes: invalid UTF-8 in asset ID"))?
    };

    // Check keyframe collection limit
    let state = &caller.data().ffi;
    let total_keyframes = state.keyframes.len() + state.pending_keyframes.len();
    if total_keyframes >= MAX_KEYFRAME_COLLECTIONS {
        bail!(
            "rom_keyframes: maximum keyframe collection count {} exceeded",
            MAX_KEYFRAME_COLLECTIONS
        );
    }

    // Get keyframe data from data pack
    let (bone_count, frame_count, keyframe_data) = {
        let state = &caller.data().ffi;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_keyframes: no data pack loaded"))?;

        let packed = data_pack.find_keyframes(&id).ok_or_else(|| {
            anyhow::anyhow!("rom_keyframes: keyframes '{}' not found in data pack", id)
        })?;

        // Validate
        if !packed.validate() {
            bail!("rom_keyframes: invalid keyframes '{}' in data pack", id);
        }

        (packed.bone_count, packed.frame_count, packed.data.clone())
    };

    // Allocate handle and queue pending load
    let state = &mut caller.data_mut().ffi;
    let handle = state.next_keyframe_handle;
    state.next_keyframe_handle += 1;

    state.pending_keyframes.push(PendingKeyframes {
        handle,
        bone_count,
        frame_count,
        data: keyframe_data,
    });

    tracing::info!(
        "rom_keyframes: queued '{}' as handle {} ({} bones, {} frames)",
        id,
        handle,
        bone_count,
        frame_count
    );

    Ok(handle)
}
