//! Tracker module loading and position/control
//!
//! Handles XM/IT tracker module loading and playback control.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use zx_common::TrackerFormat;

use crate::audio::Sound;
use crate::state::tracker_flags;
use crate::tracker::is_tracker_handle;

use super::super::{ZXGameContext, get_wasm_memory, guards::guard_init_only};

/// Register tracker FFI functions
pub(super) fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // Tracker loading
    linker.func_wrap("env", "rom_tracker", rom_tracker)?;
    linker.func_wrap("env", "load_tracker", load_tracker)?;

    // Position/control functions
    linker.func_wrap("env", "music_jump", music_jump)?;
    linker.func_wrap("env", "music_position", music_position)?;
    linker.func_wrap("env", "music_length", music_length)?;
    linker.func_wrap("env", "music_set_speed", music_set_speed)?;
    linker.func_wrap("env", "music_set_tempo", music_set_tempo)?;
    linker.func_wrap("env", "music_info", music_info)?;
    linker.func_wrap("env", "music_name", music_name)?;

    Ok(())
}

/// Load a tracker module from ROM data pack
///
/// Must be called during `init()`. Returns tracker handle (u32).
/// The tracker's instruments are mapped to ROM sound IDs by name.
///
/// Samples referenced by the tracker are automatically loaded from the data pack
/// if not already loaded. If a sample is not found in the data pack, a warning
/// is logged and the instrument will be silent.
///
/// # Parameters
/// - `id_ptr`: Pointer to tracker ID string in WASM memory
/// - `id_len`: Length of tracker ID string
///
/// # Returns
/// Tracker handle for use with tracker_play (0 = error)
fn rom_tracker(mut caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> u32 {
    guard_init_only!(caller, "rom_tracker");

    // Read tracker ID from WASM memory
    let id = {
        let mem = match get_wasm_memory(&mut caller) {
            Some(m) => m,
            None => {
                warn!("rom_tracker: failed to get WASM memory");
                return 0;
            }
        };
        let data = mem.data(&caller);
        let start = id_ptr as usize;
        let end = start + id_len as usize;
        if end > data.len() {
            warn!("rom_tracker: ID out of bounds");
            return 0;
        }
        match std::str::from_utf8(&data[start..end]) {
            Ok(s) => s.to_string(),
            Err(_) => {
                warn!("rom_tracker: invalid UTF-8 in ID");
                return 0;
            }
        }
    };

    let ctx = caller.data_mut();

    // Get data pack
    let data_pack = match &ctx.ffi.data_pack {
        Some(dp) => dp.clone(),
        None => {
            warn!("rom_tracker: no data pack loaded");
            return 0;
        }
    };

    // Find tracker in data pack
    let packed_tracker = match data_pack.find_tracker(&id) {
        Some(t) => t,
        None => {
            warn!("rom_tracker: tracker '{}' not found in data pack", id);
            return 0;
        }
    };

    // Resolve instrument names to sound handles
    // Auto-load samples from data pack if they're not already loaded
    let mut sound_handles = Vec::new();
    for sample_id in &packed_tracker.sample_ids {
        // Check if already loaded
        let sound_handle = if let Some(&existing_handle) = ctx.ffi.sound_id_to_handle.get(sample_id)
        {
            existing_handle
        } else {
            // Try to auto-load from data pack
            match data_pack.find_sound(sample_id) {
                Some(sound) => {
                    // Allocate new handle
                    let handle = ctx.ffi.next_sound_handle;
                    ctx.ffi.next_sound_handle += 1;

                    // Create Sound resource
                    let sound_resource = Sound {
                        data: std::sync::Arc::new(sound.data.clone()),
                    };

                    // Ensure sounds vector is large enough
                    while ctx.ffi.sounds.len() <= handle as usize {
                        ctx.ffi.sounds.push(None);
                    }
                    ctx.ffi.sounds[handle as usize] = Some(sound_resource);

                    // Store ID -> handle mapping
                    ctx.ffi.sound_id_to_handle.insert(sample_id.clone(), handle);

                    info!(
                        "rom_tracker: auto-loaded sample '{}' as handle {}",
                        sample_id, handle
                    );
                    handle
                }
                None => {
                    warn!(
                        "rom_tracker: sample '{}' not found in data pack, tracker instrument will be silent",
                        sample_id
                    );
                    0
                }
            }
        };
        sound_handles.push(sound_handle);
    }

    // Parse and load the tracker based on format
    let handle = match packed_tracker.format {
        TrackerFormat::Xm => {
            // Parse XM/NCXM format
            let module = match nether_xm::parse_xm_minimal(&packed_tracker.pattern_data) {
                Ok(m) => m,
                Err(e) => {
                    warn!(
                        "rom_tracker: failed to parse XM tracker data for '{}': {:?}",
                        id, e
                    );
                    return 0;
                }
            };
            ctx.ffi.tracker_engine.load_xm_module(module, sound_handles)
        }
        TrackerFormat::It => {
            // Parse IT/NCIT format
            let module = match nether_it::parse_it_minimal(&packed_tracker.pattern_data) {
                Ok(m) => m,
                Err(e) => {
                    warn!(
                        "rom_tracker: failed to parse IT tracker data for '{}': {:?}",
                        id, e
                    );
                    return 0;
                }
            };
            ctx.ffi.tracker_engine.load_it_module(module, sound_handles)
        }
    };

    info!("Loaded tracker '{}' as handle {}", id, handle);
    handle
}

/// Load a tracker module from raw XM data
///
/// Must be called during `init()`. Returns tracker handle (u32).
/// Note: Instruments must be pre-loaded as sounds and passed via sound handles.
///
/// # Parameters
/// - `data_ptr`: Pointer to raw XM data in WASM memory
/// - `data_len`: Length of XM data in bytes
///
/// # Returns
/// Tracker handle for use with tracker_play (0 = error)
fn load_tracker(mut caller: Caller<'_, ZXGameContext>, data_ptr: u32, data_len: u32) -> u32 {
    guard_init_only!(caller, "load_tracker");

    // Read XM data from WASM memory
    let xm_data = {
        let mem = match get_wasm_memory(&mut caller) {
            Some(m) => m,
            None => {
                warn!("load_tracker: failed to get WASM memory");
                return 0;
            }
        };
        let data = mem.data(&caller);
        let start = data_ptr as usize;
        let end = start + data_len as usize;
        if end > data.len() {
            warn!("load_tracker: data out of bounds");
            return 0;
        }
        data[start..end].to_vec()
    };

    // Parse the tracker data (auto-detects NCXM minimal or standard XM format)
    let module = match nether_xm::parse_xm_minimal(&xm_data) {
        Ok(m) => m,
        Err(e) => {
            warn!("load_tracker: failed to parse tracker data: {:?}", e);
            return 0;
        }
    };

    // Capture info before moving module
    let num_patterns = module.num_patterns;
    let num_instruments = module.num_instruments;

    let ctx = caller.data_mut();

    // For raw XM loading, we don't have instrument -> sound mapping
    // The game would need to provide this separately or use named samples
    let sound_handles = vec![0u32; num_instruments as usize];

    // Load the module into the tracker engine
    let handle = ctx.ffi.tracker_engine.load_xm_module(module, sound_handles);

    info!(
        "Loaded tracker with {} patterns, {} instruments as handle {}",
        num_patterns, num_instruments, handle
    );
    handle
}

// ============================================================================
// Music Position/Control Functions (tracker-specific, no-op for PCM)
// ============================================================================

/// Jump to a specific position (tracker only, no-op for PCM)
///
/// # Parameters
/// - `order`: Order position (0-based)
/// - `row`: Row within the pattern (0-based)
fn music_jump(mut caller: Caller<'_, ZXGameContext>, order: u32, row: u32) {
    let ctx = caller.data_mut();
    let tracker = &mut ctx.rollback.tracker;

    tracker.order_position = order as u16;
    tracker.row = row as u16;
    tracker.tick = 0;
    tracker.tick_sample_pos = 0;
}

/// Get current music position
///
/// For tracker: (order << 16) | row
/// For PCM: sample position
///
/// # Returns
/// Position value
fn music_position(caller: Caller<'_, ZXGameContext>) -> u32 {
    let ctx = caller.data();

    // Check if tracker is playing
    let tracker = &ctx.rollback.tracker;
    if tracker.handle != 0 && (tracker.flags & tracker_flags::PLAYING) != 0 {
        return ((tracker.order_position as u32) << 16) | (tracker.row as u32);
    }

    // Return PCM position
    ctx.rollback.audio.music.position
}

/// Get music length
///
/// For tracker: number of orders
/// For PCM: number of samples (if known, otherwise 0)
///
/// # Parameters
/// - `handle`: Music handle (PCM or tracker)
///
/// # Returns
/// Length value
fn music_length(caller: Caller<'_, ZXGameContext>, handle: u32) -> u32 {
    let ctx = caller.data();

    if is_tracker_handle(handle) {
        // Tracker length in orders
        if let Some(module) = ctx.ffi.tracker_engine.get_module(handle) {
            return module.order_table.len() as u32;
        }
    } else {
        // PCM length in samples
        if let Some(sound) = ctx.ffi.sounds.get(handle as usize).and_then(|s| s.as_ref()) {
            return sound.data.len() as u32;
        }
    }

    0
}

/// Set music speed (ticks per row, tracker only)
///
/// # Parameters
/// - `speed`: 1-31 (XM default is 6)
fn music_set_speed(mut caller: Caller<'_, ZXGameContext>, speed: u32) {
    let ctx = caller.data_mut();
    ctx.rollback.tracker.speed = (speed.clamp(1, 31)) as u16;
}

/// Set music tempo (BPM, tracker only)
///
/// # Parameters
/// - `bpm`: 32-255 (XM default is 125)
fn music_set_tempo(mut caller: Caller<'_, ZXGameContext>, bpm: u32) {
    let ctx = caller.data_mut();
    ctx.rollback.tracker.bpm = (bpm.clamp(32, 255)) as u16;
}

/// Get music info
///
/// For tracker: (num_channels << 24) | (num_patterns << 16) | (num_instruments << 8) | song_length
/// For PCM: (sample_rate << 16) | (1 << 8) | 0 (1 channel, mono)
///
/// # Parameters
/// - `handle`: Music handle (PCM or tracker)
///
/// # Returns
/// Packed info value
fn music_info(caller: Caller<'_, ZXGameContext>, handle: u32) -> u32 {
    let ctx = caller.data();

    if is_tracker_handle(handle) {
        if let Some(module) = ctx.ffi.tracker_engine.get_module(handle) {
            return ((module.num_channels as u32) << 24)
                | ((module.patterns.len() as u32) << 16)
                | ((module.instruments.len() as u32) << 8)
                | (module.order_table.len() as u32);
        }
    } else {
        // PCM info: sample_rate=22050, channels=1, bits=16
        if ctx
            .ffi
            .sounds
            .get(handle as usize)
            .and_then(|s| s.as_ref())
            .is_some()
        {
            return (22050 << 16) | (1 << 8) | 16;
        }
    }

    0
}

/// Get music name (tracker only, returns 0 for PCM)
///
/// # Parameters
/// - `handle`: Music handle
/// - `out_ptr`: Pointer to output buffer in WASM memory
/// - `max_len`: Maximum bytes to write
///
/// # Returns
/// Actual length written (0 if handle invalid or PCM)
fn music_name(
    mut caller: Caller<'_, ZXGameContext>,
    handle: u32,
    out_ptr: u32,
    max_len: u32,
) -> u32 {
    // Only tracker handles have names
    if !is_tracker_handle(handle) {
        return 0;
    }

    let name = {
        let ctx = caller.data();
        if let Some(module) = ctx.ffi.tracker_engine.get_module(handle) {
            module.name.clone()
        } else {
            return 0;
        }
    };

    let bytes = name.as_bytes();
    let write_len = bytes.len().min(max_len as usize);

    // Write to WASM memory
    let mem = match get_wasm_memory(&mut caller) {
        Some(m) => m,
        None => return 0,
    };

    let data = mem.data_mut(&mut caller);
    let start = out_ptr as usize;
    let end = start + write_len;

    if end > data.len() {
        return 0;
    }

    data[start..end].copy_from_slice(&bytes[..write_len]);
    write_len as u32
}
