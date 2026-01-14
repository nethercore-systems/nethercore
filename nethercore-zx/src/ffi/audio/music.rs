//! Unified music API
//!
//! Provides a unified interface for playing both PCM sounds and tracker modules as music.

use anyhow::Result;
use wasmtime::{Caller, Linker};

use crate::state::tracker_flags;
use crate::tracker::is_tracker_handle;

use super::super::ZXGameContext;
use super::{clamp_safe, music_type};

/// Register music FFI functions
pub(super) fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "music_play", music_play)?;
    linker.func_wrap("env", "music_stop", music_stop)?;
    linker.func_wrap("env", "music_pause", music_pause)?;
    linker.func_wrap("env", "music_set_volume", music_set_volume)?;
    linker.func_wrap("env", "music_is_playing", music_is_playing)?;
    linker.func_wrap("env", "music_type", music_type_fn)?;
    Ok(())
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
        ctx.rollback.audio.music.reset_position();

        // Set up tracker state with raw handle (strip flag)
        let raw_handle = crate::tracker::raw_tracker_handle(handle);

        // Get initial tempo/speed from the loaded module (fall back to defaults if not found)
        let (initial_speed, initial_tempo) = ctx
            .ffi
            .tracker_engine
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .map(|m| (m.module.initial_speed as u16, m.module.initial_tempo as u16))
            .unwrap_or((crate::tracker::DEFAULT_SPEED, crate::tracker::DEFAULT_BPM));

        let tracker = &mut ctx.rollback.tracker;
        tracker.handle = raw_handle;
        tracker.order_position = 0;
        tracker.row = 0;
        tracker.tick = 0;
        tracker.speed = initial_speed;
        tracker.bpm = initial_tempo;
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
        music.reset_position();
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
    music.reset_position();
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
