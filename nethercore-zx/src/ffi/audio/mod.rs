//! Audio FFI functions
//!
//! Functions for loading sounds and controlling playback via channels and music.
//!
//! Audio state is stored in ZRollbackState.audio, which is automatically rolled back
//! during netcode rollback. FFI functions directly modify this state rather than
//! buffering commands, ensuring audio stays perfectly in sync with game state.

use anyhow::Result;
use wasmtime::Linker;

use super::ZXGameContext;

pub mod music;
pub mod sound;
pub mod tracker;

/// Music type constants for music_type() return value
pub mod music_type {
    pub const NONE: u32 = 0;
    pub const PCM: u32 = 1;
    pub const TRACKER: u32 = 2;
}

/// Clamp a float value, treating NaN as the minimum value
#[inline]
pub(crate) fn clamp_safe(value: f32, min: f32, max: f32) -> f32 {
    if value.is_nan() {
        min
    } else {
        value.clamp(min, max)
    }
}

/// Register audio FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    sound::register(linker)?;
    music::register(linker)?;
    tracker::register(linker)?;
    Ok(())
}
