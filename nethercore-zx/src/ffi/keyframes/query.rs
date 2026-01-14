//! Keyframe query functions

use tracing::warn;
use wasmtime::Caller;

use crate::ffi::ZXGameContext;

/// Get the bone count for a keyframe collection
///
/// # Arguments
/// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
///
/// # Returns
/// Bone count (0 on invalid handle)
///
/// # Note
/// Works during init() by also checking pending_keyframes.
pub(super) fn keyframes_bone_count(caller: Caller<'_, ZXGameContext>, handle: u32) -> u32 {
    if handle == 0 {
        warn!("keyframes_bone_count: invalid handle 0");
        return 0;
    }

    let state = &caller.data().ffi;
    let index = handle as usize - 1;

    // First check finalized keyframes
    if let Some(kf) = state.keyframes.get(index) {
        return kf.bone_count as u32;
    }

    // During init(), keyframes may still be in pending_keyframes
    // Search by handle since indices don't match during pending state
    for pending in &state.pending_keyframes {
        if pending.handle == handle {
            return pending.bone_count as u32;
        }
    }

    warn!(
        "keyframes_bone_count: handle {} not found (only {} loaded, {} pending)",
        handle,
        state.keyframes.len(),
        state.pending_keyframes.len()
    );
    0
}

/// Get the frame count for a keyframe collection
///
/// # Arguments
/// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
///
/// # Returns
/// Frame count (0 on invalid handle)
///
/// # Note
/// Works during init() by also checking pending_keyframes.
pub(super) fn keyframes_frame_count(caller: Caller<'_, ZXGameContext>, handle: u32) -> u32 {
    if handle == 0 {
        warn!("keyframes_frame_count: invalid handle 0");
        return 0;
    }

    let state = &caller.data().ffi;
    let index = handle as usize - 1;

    // First check finalized keyframes
    if let Some(kf) = state.keyframes.get(index) {
        return kf.frame_count as u32;
    }

    // During init(), keyframes may still be in pending_keyframes
    // Search by handle since indices don't match during pending state
    for pending in &state.pending_keyframes {
        if pending.handle == handle {
            return pending.frame_count as u32;
        }
    }

    warn!(
        "keyframes_frame_count: handle {} not found (only {} loaded, {} pending)",
        handle,
        state.keyframes.len(),
        state.pending_keyframes.len()
    );
    0
}
