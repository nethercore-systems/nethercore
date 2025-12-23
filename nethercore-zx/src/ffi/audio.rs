//! Audio FFI functions
//!
//! Functions for loading sounds and controlling playback via channels and music.
//!
//! Audio state is stored in ZRollbackState.audio, which is automatically rolled back
//! during netcode rollback. FFI functions directly modify this state rather than
//! buffering commands, ensuring audio stays perfectly in sync with game state.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use crate::audio::Sound;
use crate::state::{MAX_CHANNELS, tracker_flags};
use crate::tracker::{is_tracker_handle, raw_tracker_handle};

use super::{ZXGameContext, get_wasm_memory, guards::check_init_only, helpers::read_wasm_i16s};

/// Music type constants for music_type() return value
pub mod music_type {
    pub const NONE: u32 = 0;
    pub const PCM: u32 = 1;
    pub const TRACKER: u32 = 2;
}

/// Clamp a float value, treating NaN as the minimum value
#[inline]
fn clamp_safe(value: f32, min: f32, max: f32) -> f32 {
    if value.is_nan() {
        min
    } else {
        value.clamp(min, max)
    }
}

/// Register audio FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // Sound loading and playback
    linker.func_wrap("env", "load_sound", load_sound)?;
    linker.func_wrap("env", "play_sound", play_sound)?;
    linker.func_wrap("env", "channel_play", channel_play)?;
    linker.func_wrap("env", "channel_set", channel_set)?;
    linker.func_wrap("env", "channel_stop", channel_stop)?;

    // Tracker loading (returns flagged handles for unified music API)
    linker.func_wrap("env", "rom_tracker", rom_tracker)?;
    linker.func_wrap("env", "load_tracker", load_tracker)?;

    // Unified Music API (works with both PCM and tracker handles)
    linker.func_wrap("env", "music_play", music_play)?;
    linker.func_wrap("env", "music_stop", music_stop)?;
    linker.func_wrap("env", "music_pause", music_pause)?;
    linker.func_wrap("env", "music_set_volume", music_set_volume)?;
    linker.func_wrap("env", "music_is_playing", music_is_playing)?;
    linker.func_wrap("env", "music_type", music_type_fn)?;
    linker.func_wrap("env", "music_jump", music_jump)?;
    linker.func_wrap("env", "music_position", music_position)?;
    linker.func_wrap("env", "music_length", music_length)?;
    linker.func_wrap("env", "music_set_speed", music_set_speed)?;
    linker.func_wrap("env", "music_set_tempo", music_set_tempo)?;
    linker.func_wrap("env", "music_info", music_info)?;
    linker.func_wrap("env", "music_name", music_name)?;

    Ok(())
}

/// Load raw PCM sound data (22.05kHz, 16-bit signed, mono)
///
/// Must be called during `init()`. Returns sound handle (u32).
///
/// # Parameters
/// - `data_ptr`: Pointer to raw i16 PCM data in WASM memory
/// - `byte_len`: Length of data in bytes (must be even, as each sample is 2 bytes)
///
/// # Returns
/// Sound handle for use with play_sound, channel_play, music_play
fn load_sound(mut caller: Caller<'_, ZXGameContext>, data_ptr: u32, byte_len: u32) -> u32 {
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "load_sound") {
        warn!("{}", e);
        return 0;
    }

    // Validate byte length is even (each sample is 2 bytes)
    if !byte_len.is_multiple_of(2) {
        warn!("load_sound: byte_len must be even (got {})", byte_len);
        return 0;
    }

    let sample_count = (byte_len / 2) as usize;

    // Read PCM data from WASM memory using helper (handles bounds checking + bytemuck cast)
    let Some(pcm_data) = read_wasm_i16s(&caller, data_ptr, sample_count, "load_sound") else {
        return 0;
    };

    let state = &mut caller.data_mut().ffi;

    // Create Sound and add to sounds vec
    let sound = Sound {
        data: std::sync::Arc::new(pcm_data),
    };

    let handle = state.next_sound_handle;
    state.next_sound_handle += 1;

    // Resize sounds vec if needed
    if handle as usize >= state.sounds.len() {
        state.sounds.resize(handle as usize + 1, None);
    }
    state.sounds[handle as usize] = Some(sound);

    info!("Loaded sound {} ({} samples)", handle, sample_count);
    handle
}

/// Play sound on next available channel (fire-and-forget)
///
/// For one-shot sounds: gunshots, jumps, coins
///
/// # Parameters
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
fn play_sound(mut caller: Caller<'_, ZXGameContext>, sound: u32, volume: f32, pan: f32) {
    let ctx = caller.data_mut();

    // Find first free channel (sound == 0 means channel is available)
    for channel in ctx.rollback.audio.channels.iter_mut() {
        if channel.sound == 0 {
            channel.sound = sound;
            channel.position = 0;
            channel.looping = 0;
            channel.volume = clamp_safe(volume, 0.0, 1.0);
            channel.pan = clamp_safe(pan, -1.0, 1.0);
            return;
        }
    }

    // All channels busy - sound is dropped
    warn!("play_sound: all channels busy, sound {} dropped", sound);
}

/// Play sound on specific channel
///
/// For managed channels (positional/looping: engines, ambient, footsteps)
///
/// # Parameters
/// - `channel`: 0-15
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
/// - `looping`: 1 = loop, 0 = play once
fn channel_play(
    mut caller: Caller<'_, ZXGameContext>,
    channel: u32,
    sound: u32,
    volume: f32,
    pan: f32,
    looping: u32,
) {
    let channel_idx = channel as usize;
    if channel_idx >= MAX_CHANNELS {
        warn!("channel_play: invalid channel {}", channel);
        return;
    }

    let ctx = caller.data_mut();
    let ch = &mut ctx.rollback.audio.channels[channel_idx];

    // If same sound is already playing and looping matches, just update volume/pan
    if ch.sound == sound && ch.looping == looping && ch.sound != 0 {
        ch.volume = clamp_safe(volume, 0.0, 1.0);
        ch.pan = clamp_safe(pan, -1.0, 1.0);
        return;
    }

    // Start new sound
    ch.sound = sound;
    ch.position = 0;
    ch.looping = looping;
    ch.volume = clamp_safe(volume, 0.0, 1.0);
    ch.pan = clamp_safe(pan, -1.0, 1.0);
}

/// Update channel parameters (call every frame for positional audio)
///
/// # Parameters
/// - `channel`: 0-15
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
fn channel_set(mut caller: Caller<'_, ZXGameContext>, channel: u32, volume: f32, pan: f32) {
    let channel_idx = channel as usize;
    if channel_idx >= MAX_CHANNELS {
        warn!("channel_set: invalid channel {}", channel);
        return;
    }

    let ctx = caller.data_mut();
    let ch = &mut ctx.rollback.audio.channels[channel_idx];
    ch.volume = clamp_safe(volume, 0.0, 1.0);
    ch.pan = clamp_safe(pan, -1.0, 1.0);
}

/// Stop channel
///
/// # Parameters
/// - `channel`: 0-15
fn channel_stop(mut caller: Caller<'_, ZXGameContext>, channel: u32) {
    let channel_idx = channel as usize;
    if channel_idx >= MAX_CHANNELS {
        warn!("channel_stop: invalid channel {}", channel);
        return;
    }

    let ctx = caller.data_mut();
    let ch = &mut ctx.rollback.audio.channels[channel_idx];
    ch.sound = 0;
    ch.position = 0;
    ch.looping = 0;
}

/// Play music (unified API for PCM sounds and tracker modules)
///
/// Automatically stops any currently playing music of either type.
/// Detects handle type by checking bit 31 (set for tracker handles).
///
/// # Parameters
/// - `handle`: Sound handle from load_sound() or tracker handle from rom_tracker()/load_tracker()
/// - `volume`: 0.0 to 1.0
/// - `looping`: 1 = loop, 0 = play once
fn music_play(mut caller: Caller<'_, ZXGameContext>, handle: u32, volume: f32, looping: u32) {
    let ctx = caller.data_mut();

    if is_tracker_handle(handle) {
        // Tracker music - stop PCM music first
        ctx.rollback.audio.music.sound = 0;
        ctx.rollback.audio.music.position = 0;

        // Set up tracker state with raw handle (strip flag)
        let raw_handle = raw_tracker_handle(handle);
        let tracker = &mut ctx.rollback.tracker;
        tracker.handle = raw_handle;
        tracker.order_position = 0;
        tracker.row = 0;
        tracker.tick = 0;
        tracker.speed = crate::tracker::DEFAULT_SPEED;
        tracker.bpm = crate::tracker::DEFAULT_BPM;
        tracker.volume = (clamp_safe(volume, 0.0, 1.0) * 256.0) as u16;
        tracker.tick_sample_pos = 0;

        let mut flags = tracker_flags::PLAYING;
        if looping != 0 {
            flags |= tracker_flags::LOOPING;
        }
        tracker.flags = flags;

        // Reset the tracker engine
        ctx.ffi.tracker_engine.reset();
    } else {
        // PCM music - stop tracker first
        ctx.rollback.tracker.handle = 0;
        ctx.rollback.tracker.flags = 0;

        let music = &mut ctx.rollback.audio.music;

        // If same music is already playing with same looping, just update volume
        if music.sound == handle && music.looping == looping && music.sound != 0 {
            music.volume = clamp_safe(volume, 0.0, 1.0);
            return;
        }

        // Start new PCM music
        music.sound = handle;
        music.position = 0;
        music.looping = looping;
        music.volume = clamp_safe(volume, 0.0, 1.0);
        music.pan = 0.0; // Music is always centered
    }
}

/// Stop music (unified - stops both PCM and tracker)
fn music_stop(mut caller: Caller<'_, ZXGameContext>) {
    let ctx = caller.data_mut();

    // Stop PCM music
    let music = &mut ctx.rollback.audio.music;
    music.sound = 0;
    music.position = 0;
    music.looping = 0;

    // Stop tracker music
    let tracker = &mut ctx.rollback.tracker;
    tracker.handle = 0;
    tracker.flags = 0;
    tracker.order_position = 0;
    tracker.row = 0;
    tracker.tick = 0;
}

/// Pause or resume music (tracker only, no-op for PCM)
///
/// # Parameters
/// - `paused`: 1 = pause, 0 = resume
fn music_pause(mut caller: Caller<'_, ZXGameContext>, paused: u32) {
    let ctx = caller.data_mut();
    let tracker = &mut ctx.rollback.tracker;

    if paused != 0 {
        tracker.flags |= tracker_flags::PAUSED;
    } else {
        tracker.flags &= !tracker_flags::PAUSED;
    }
}

/// Set music volume (works for both PCM and tracker)
///
/// # Parameters
/// - `volume`: 0.0 to 1.0
fn music_set_volume(mut caller: Caller<'_, ZXGameContext>, volume: f32) {
    let ctx = caller.data_mut();

    // Set PCM music volume
    ctx.rollback.audio.music.volume = clamp_safe(volume, 0.0, 1.0);

    // Set tracker volume
    ctx.rollback.tracker.volume = (clamp_safe(volume, 0.0, 1.0) * 256.0) as u16;
}

/// Check if music is currently playing
///
/// # Returns
/// 1 if playing (and not paused), 0 otherwise
fn music_is_playing(caller: Caller<'_, ZXGameContext>) -> u32 {
    let ctx = caller.data();

    // Check tracker
    let tracker = &ctx.rollback.tracker;
    if tracker.handle != 0
        && (tracker.flags & tracker_flags::PLAYING) != 0
        && (tracker.flags & tracker_flags::PAUSED) == 0
    {
        return 1;
    }

    // Check PCM music
    if ctx.rollback.audio.music.sound != 0 {
        return 1;
    }

    0
}

/// Get current music type
///
/// # Returns
/// 0 = none, 1 = PCM, 2 = tracker
fn music_type_fn(caller: Caller<'_, ZXGameContext>) -> u32 {
    let ctx = caller.data();

    // Check tracker first (higher priority)
    let tracker = &ctx.rollback.tracker;
    if tracker.handle != 0 && (tracker.flags & tracker_flags::PLAYING) != 0 {
        return music_type::TRACKER;
    }

    // Check PCM music
    if ctx.rollback.audio.music.sound != 0 {
        return music_type::PCM;
    }

    music_type::NONE
}

// ============================================================================
// Tracker FFI Functions (XM Module Playback)
// ============================================================================

/// Load a tracker module from ROM data pack
///
/// Must be called during `init()`. Returns tracker handle (u32).
/// The tracker's instruments are mapped to ROM sound IDs by name.
///
/// # Parameters
/// - `id_ptr`: Pointer to tracker ID string in WASM memory
/// - `id_len`: Length of tracker ID string
///
/// # Returns
/// Tracker handle for use with tracker_play (0 = error)
fn rom_tracker(mut caller: Caller<'_, ZXGameContext>, id_ptr: u32, id_len: u32) -> u32 {
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "rom_tracker") {
        warn!("{}", e);
        return 0;
    }

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

    // Parse the XM data
    let module = match nether_xm::parse_xm(&packed_tracker.pattern_data) {
        Ok(m) => m,
        Err(e) => {
            warn!("rom_tracker: failed to parse XM for '{}': {:?}", id, e);
            return 0;
        }
    };

    // Resolve instrument names to sound handles
    // The game must load samples via rom_sound() before loading the tracker
    let mut sound_handles = Vec::new();
    for sample_id in &packed_tracker.sample_ids {
        // Look up the sound handle from the ID -> handle mapping
        let sound_handle = ctx
            .ffi
            .sound_id_to_handle
            .get(sample_id)
            .copied()
            .unwrap_or_else(|| {
                warn!(
                    "rom_tracker: sample '{}' not loaded, tracker instrument will be silent",
                    sample_id
                );
                0
            });
        sound_handles.push(sound_handle);
    }

    // Load the module into the tracker engine
    let handle = ctx.ffi.tracker_engine.load_module(module, sound_handles);

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
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "load_tracker") {
        warn!("{}", e);
        return 0;
    }

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

    // Parse the XM data
    let module = match nether_xm::parse_xm(&xm_data) {
        Ok(m) => m,
        Err(e) => {
            warn!("load_tracker: failed to parse XM: {:?}", e);
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
    let handle = ctx.ffi.tracker_engine.load_module(module, sound_handles);

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
            return module.song_length as u32;
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
                | ((module.num_patterns as u32) << 16)
                | ((module.num_instruments as u32) << 8)
                | (module.song_length as u32);
        }
    } else {
        // PCM info: sample_rate=22050, channels=1, bits=16
        if ctx.ffi.sounds.get(handle as usize).and_then(|s| s.as_ref()).is_some() {
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

