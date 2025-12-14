//! Audio FFI functions
//!
//! Functions for loading sounds and controlling playback via channels and music.
//!
//! In the per-frame audio architecture, these functions directly modify the
//! `AudioPlaybackState` in `ctx.rollback.audio`, which is saved/restored during
//! GGRS rollback for deterministic audio.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use super::{get_wasm_memory, guards::check_init_only, ZContext};
use crate::audio::MAX_SFX_CHANNELS;
use crate::state::PendingSound;

/// Register audio FFI functions
pub fn register(linker: &mut Linker<ZContext>) -> Result<()> {
    linker.func_wrap("env", "load_sound", load_sound)?;
    linker.func_wrap("env", "play_sound", play_sound)?;
    linker.func_wrap("env", "channel_play", channel_play)?;
    linker.func_wrap("env", "channel_set", channel_set)?;
    linker.func_wrap("env", "channel_stop", channel_stop)?;
    linker.func_wrap("env", "music_play", music_play)?;
    linker.func_wrap("env", "music_stop", music_stop)?;
    linker.func_wrap("env", "music_set_volume", music_set_volume)?;
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
fn load_sound(mut caller: Caller<'_, ZContext>, data_ptr: u32, byte_len: u32) -> u32 {
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

    // Get WASM memory
    let Some(memory) = get_wasm_memory(&mut caller) else {
        warn!("load_sound: failed to get WASM memory");
        return 0;
    };

    // Read PCM data from WASM memory
    let mut pcm_data = vec![0i16; sample_count];
    // SAFETY: This unsafe block is sound because:
    // 1. The pointer comes from WASM memory export, guaranteed valid by wasmtime
    // 2. byte_len is validated as even (divisible by 2), ensuring proper i16 alignment
    // 3. sample_count = byte_len / 2, so we're reading exactly the right number of i16 samples
    // 4. Data is immediately copied to owned Vec, no aliasing or lifetime issues
    // 5. WASM linear memory is guaranteed to be valid for the duration of this call
    let data_slice = unsafe {
        let ptr = memory.data_ptr(&caller).add(data_ptr as usize);
        std::slice::from_raw_parts(ptr as *const i16, sample_count)
    };
    pcm_data.copy_from_slice(data_slice);

    let state = &mut caller.data_mut().ffi;

    // Add to pending sounds (will be moved to resource manager during process_pending_resources)
    let handle = state.next_sound_handle;
    state.next_sound_handle += 1;

    state.pending_sounds.push(PendingSound {
        handle,
        data: pcm_data,
    });

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
fn play_sound(mut caller: Caller<'_, ZContext>, sound: u32, volume: f32, pan: f32) {
    let ctx = caller.data_mut();
    let audio_state = &mut ctx.rollback.audio;

    // Find first free channel
    if let Some(channel_idx) = audio_state.find_free_channel() {
        audio_state.channels[channel_idx].play(sound, volume, pan, false);
    } else {
        // All channels busy, sound dropped (this is fine for fire-and-forget)
        tracing::trace!("play_sound: all channels busy, sound {} dropped", sound);
    }
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
    mut caller: Caller<'_, ZContext>,
    channel: u32,
    sound: u32,
    volume: f32,
    pan: f32,
    looping: u32,
) {
    let channel_idx = channel as usize;
    if channel_idx >= MAX_SFX_CHANNELS {
        warn!("channel_play: invalid channel {}", channel);
        return;
    }

    let audio_state = &mut caller.data_mut().rollback.audio;
    audio_state.channels[channel_idx].play(sound, volume, pan, looping != 0);
}

/// Update channel parameters (call every frame for positional audio)
///
/// # Parameters
/// - `channel`: 0-15
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
fn channel_set(mut caller: Caller<'_, ZContext>, channel: u32, volume: f32, pan: f32) {
    let channel_idx = channel as usize;
    if channel_idx >= MAX_SFX_CHANNELS {
        warn!("channel_set: invalid channel {}", channel);
        return;
    }

    let audio_state = &mut caller.data_mut().rollback.audio;
    let ch = &mut audio_state.channels[channel_idx];

    // Only update if playing (ignore if stopped)
    if ch.is_playing() {
        ch.set_volume(volume);
        ch.set_pan(pan);
    }
}

/// Stop channel
///
/// # Parameters
/// - `channel`: 0-15
fn channel_stop(mut caller: Caller<'_, ZContext>, channel: u32) {
    let channel_idx = channel as usize;
    if channel_idx >= MAX_SFX_CHANNELS {
        warn!("channel_stop: invalid channel {}", channel);
        return;
    }

    let audio_state = &mut caller.data_mut().rollback.audio;
    audio_state.channels[channel_idx].stop();
}

/// Play music (looping, dedicated channel)
///
/// # Parameters
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
fn music_play(mut caller: Caller<'_, ZContext>, sound: u32, volume: f32) {
    let audio_state = &mut caller.data_mut().rollback.audio;
    audio_state.music.play(sound, volume);
}

/// Stop music
fn music_stop(mut caller: Caller<'_, ZContext>) {
    let audio_state = &mut caller.data_mut().rollback.audio;
    audio_state.music.stop();
}

/// Set music volume
///
/// # Parameters
/// - `volume`: 0.0 to 1.0
fn music_set_volume(mut caller: Caller<'_, ZContext>, volume: f32) {
    let audio_state = &mut caller.data_mut().rollback.audio;
    audio_state.music.set_volume(volume);
}
