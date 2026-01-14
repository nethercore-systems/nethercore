//! Tracker engine implementation
//!
//! Core playback logic for the tracker engine including:
//! - State synchronization for rollback
//! - Row and tick processing
//! - Effect processing
//! - Audio rendering

// Re-export all public APIs from submodules
mod effects;
mod mixing;
mod nna;
mod render;
mod row_processing;
mod sync;
mod tick;

#[cfg(test)]
mod tests;

// ============================================================================
// Tracker Audio Constants
// ============================================================================

/// Maximum volume level for volume envelopes (XM/IT spec)
pub(crate) const VOLUME_ENVELOPE_MAX: f32 = 64.0;

/// Maximum volume fadeout value (16-bit)
pub(crate) const VOLUME_FADEOUT_MAX: f32 = 65535.0;

/// Maximum tracker volume (8-bit state volume)
pub(crate) const TRACKER_VOLUME_MAX: f32 = 256.0;

/// Maximum channel volume (IT-style, 0-64)
pub(crate) const CHANNEL_VOLUME_MAX: f32 = 64.0;

/// Maximum global volume (IT-style, 0-128)
pub(crate) const GLOBAL_VOLUME_MAX: f32 = 128.0;

/// Panning envelope center value
pub(crate) const PAN_ENVELOPE_CENTER: f32 = 32.0;

/// Maximum panning note range
pub(crate) const PAN_NOTE_RANGE: f32 = 256.0;

/// Minimum BPM value
pub(crate) const MIN_BPM: i16 = 32;

/// Maximum BPM value
pub(crate) const MAX_BPM: i16 = 255;
