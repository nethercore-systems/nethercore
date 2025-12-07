//! Audio FFI functions
//!
//! Functions for loading sounds and controlling playback via channels and music.

use anyhow::Result;
use tracing::{info, warn};
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::audio::{AudioCommand, Sound};
use crate::console::ZInput;
use crate::state::ZFFIState;

/// Register audio FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
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
fn load_sound(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    byte_len: u32,
) -> u32 {
    // Enforce init-only
    if !caller.data().game.in_init {
        warn!("load_sound() called outside init() - ignored");
        return 0;
    }

    // Validate byte length is even (each sample is 2 bytes)
    if !byte_len.is_multiple_of(2) {
        warn!("load_sound: byte_len must be even (got {})", byte_len);
        return 0;
    }

    let sample_count = (byte_len / 2) as usize;

    // Get WASM memory
    let memory = match caller.get_export("memory") {
        Some(wasmtime::Extern::Memory(mem)) => mem,
        _ => {
            warn!("load_sound: failed to get WASM memory");
            return 0;
        }
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

    let state = &mut caller.data_mut().console;

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
fn play_sound(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    sound: u32,
    volume: f32,
    pan: f32,
) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::PlaySound { sound, volume, pan });
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
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    channel: u32,
    sound: u32,
    volume: f32,
    pan: f32,
    looping: u32,
) {
    let state = &mut caller.data_mut().console;
    state.audio_commands.push(AudioCommand::ChannelPlay {
        channel,
        sound,
        volume,
        pan,
        looping: looping != 0,
    });
}

/// Update channel parameters (call every frame for positional audio)
///
/// # Parameters
/// - `channel`: 0-15
/// - `volume`: 0.0 to 1.0
/// - `pan`: -1.0 (left) to 1.0 (right), 0.0 = center
fn channel_set(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    channel: u32,
    volume: f32,
    pan: f32,
) {
    let state = &mut caller.data_mut().console;
    state.audio_commands.push(AudioCommand::ChannelSet {
        channel,
        volume,
        pan,
    });
}

/// Stop channel
///
/// # Parameters
/// - `channel`: 0-15
fn channel_stop(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, channel: u32) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::ChannelStop { channel });
}

/// Play music (looping, dedicated channel)
///
/// # Parameters
/// - `sound`: Sound handle from load_sound()
/// - `volume`: 0.0 to 1.0
fn music_play(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    sound: u32,
    volume: f32,
) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::MusicPlay { sound, volume });
}

/// Stop music
fn music_stop(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>) {
    let state = &mut caller.data_mut().console;
    state.audio_commands.push(AudioCommand::MusicStop);
}

/// Set music volume
///
/// # Parameters
/// - `volume`: 0.0 to 1.0
fn music_set_volume(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, volume: f32) {
    let state = &mut caller.data_mut().console;
    state
        .audio_commands
        .push(AudioCommand::MusicSetVolume { volume });
}
