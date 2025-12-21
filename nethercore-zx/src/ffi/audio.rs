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
use crate::state::MAX_CHANNELS;

use super::{ZXGameContext, guards::check_init_only, helpers::read_wasm_i16s};

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

/// Play music (looping, dedicated channel)
///
/// # Parameters
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
fn music_play(mut caller: Caller<'_, ZXGameContext>, sound: u32, volume: f32) {
    let ctx = caller.data_mut();
    let music = &mut ctx.rollback.audio.music;

    // If same music is already playing, just update volume
    if music.sound == sound && music.sound != 0 {
        music.volume = clamp_safe(volume, 0.0, 1.0);
        return;
    }

    // Start new music
    music.sound = sound;
    music.position = 0;
    music.looping = 1; // Music always loops
    music.volume = clamp_safe(volume, 0.0, 1.0);
    music.pan = 0.0; // Music is always centered
}

/// Stop music
fn music_stop(mut caller: Caller<'_, ZXGameContext>) {
    let ctx = caller.data_mut();
    let music = &mut ctx.rollback.audio.music;
    music.sound = 0;
    music.position = 0;
    music.looping = 0;
}

/// Set music volume
///
/// # Parameters
/// - `volume`: 0.0 to 1.0
fn music_set_volume(mut caller: Caller<'_, ZXGameContext>, volume: f32) {
    let ctx = caller.data_mut();
    ctx.rollback.audio.music.volume = clamp_safe(volume, 0.0, 1.0);
}
