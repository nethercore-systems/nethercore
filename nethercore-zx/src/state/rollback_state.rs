//! Nethercore ZX rollback state
//!
//! This module contains console-specific state that needs to be rolled back
//! during netcode rollback. Unlike FFI state (which is rebuilt each frame),
//! rollback state persists and must be serialized/deserialized with WASM memory.
//!
//! All types are POD (Plain Old Data) using bytemuck for zero-copy serialization.

use bytemuck::{Pod, Zeroable};
use nethercore_core::console::ConsoleRollbackState;

/// Maximum number of sound effect channels
pub const MAX_CHANNELS: usize = 16;

/// State for a single audio channel (20 bytes, POD)
///
/// Position is stored in 24.8 fixed-point format:
/// - Upper 24 bits: integer sample position (0..16,777,215 ≈ 761 seconds at 22050 Hz)
/// - Lower 8 bits: fractional position (1/256 sample precision ≈ 0.02ms)
///
/// This fixed-point encoding enables smooth sub-sample interpolation while maintaining
/// deterministic rollback behavior and binary compatibility (same 4-byte u32 size).
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ChannelState {
    /// Sound handle (0 = silent/no sound)
    pub sound: u32,
    /// Playhead position in samples (24.8 fixed-point)
    pub position: u32,
    /// Whether the channel is looping (0 = no, 1 = yes)
    pub looping: u32,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Pan (-1.0 = left, 0.0 = center, 1.0 = right)
    pub pan: f32,
}

impl ChannelState {
    /// Number of fractional bits in fixed-point position encoding
    pub const FRAC_BITS: u32 = 8;
    /// Mask for extracting fractional part
    pub const FRAC_MASK: u32 = (1 << Self::FRAC_BITS) - 1;
    /// Fixed-point value representing 1.0
    pub const FRAC_ONE: u32 = 1 << Self::FRAC_BITS;

    /// Get position as (integer_part, fractional_part)
    ///
    /// Returns the sample position split into integer and fractional components.
    /// The fractional part is in the range [0.0, 1.0).
    #[inline]
    pub fn get_position(&self) -> (usize, f32) {
        let int_part = (self.position >> Self::FRAC_BITS) as usize;
        let frac_part = (self.position & Self::FRAC_MASK) as f32 / Self::FRAC_ONE as f32;
        (int_part, frac_part)
    }

    /// Set position from floating-point value
    ///
    /// Converts a floating-point position to 24.8 fixed-point format.
    #[inline]
    pub fn set_position(&mut self, pos: f32) {
        self.position = (pos * Self::FRAC_ONE as f32) as u32;
    }

    /// Advance position by fractional delta
    ///
    /// Adds a floating-point delta to the current position, maintaining
    /// sub-sample precision. Uses wrapping addition for determinism.
    #[inline]
    pub fn advance_position(&mut self, delta: f32) {
        let delta_fixed = (delta * Self::FRAC_ONE as f32) as u32;
        self.position = self.position.wrapping_add(delta_fixed);
    }

    /// Reset position to zero
    #[inline]
    pub fn reset_position(&mut self) {
        self.position = 0;
    }
}

/// Audio playback state (340 bytes total)
///
/// Contains the state of all audio channels including the dedicated music channel.
/// This entire structure is rolled back during netcode rollback, which means
/// audio playback automatically syncs with game state.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct AudioPlaybackState {
    /// Sound effect channels (16 channels)
    pub channels: [ChannelState; MAX_CHANNELS],
    /// Dedicated music channel
    pub music: ChannelState,
}

/// Tracker playback state flags
pub mod tracker_flags {
    /// Tracker is currently playing
    pub const PLAYING: u32 = 1 << 0;
    /// Tracker should loop when reaching the end
    pub const LOOPING: u32 = 1 << 1;
    /// Tracker playback is paused
    pub const PAUSED: u32 = 1 << 2;
}

/// Tracker music playback state (64 bytes, POD)
///
/// Minimal state for XM tracker playback that gets rolled back during netcode.
/// The full channel state is reconstructed from this by seeking to the position
/// and replaying ticks. This keeps the rollback snapshot small while still
/// enabling perfect audio synchronization.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct TrackerState {
    /// Tracker module handle (0 = no tracker playing)
    pub handle: u32,
    /// Current position in the pattern order table
    pub order_position: u16,
    /// Current row within the pattern (0-255)
    pub row: u16,
    /// Current tick within the row
    pub tick: u16,
    /// Ticks per row (from Fxx speed command, default 6)
    pub speed: u16,
    /// Beats per minute (from Fxx tempo command, default 125)
    pub bpm: u16,
    /// Volume multiplier (0-256, where 256 = 1.0)
    pub volume: u16,
    /// Playback flags (see tracker_flags module)
    pub flags: u32,
    /// Sample-accurate position within the current tick
    pub tick_sample_pos: u32,
    /// Reserved for future use (maintains 64-byte alignment)
    /// Using [u32; 10] instead of [u8; 40] because Default is only impl'd for arrays <= 32
    pub _reserved: [u32; 10],
}

/// Nethercore ZX rollback state (404 bytes total)
///
/// This is the console-specific state that gets rolled back along with
/// WASM memory during netcode rollback. It contains audio playback state
/// so that sounds automatically stay in sync with game state.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ZRollbackState {
    /// Audio playback state (channels + music) - 340 bytes
    pub audio: AudioPlaybackState,
    /// Tracker music playback state - 64 bytes
    pub tracker: TrackerState,
}

impl ConsoleRollbackState for ZRollbackState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_state_size() {
        assert_eq!(std::mem::size_of::<ChannelState>(), 20);
    }

    #[test]
    fn test_audio_playback_state_size() {
        // 16 channels * 20 bytes + 1 music channel * 20 bytes = 340 bytes
        assert_eq!(
            std::mem::size_of::<AudioPlaybackState>(),
            MAX_CHANNELS * 20 + 20
        );
    }

    #[test]
    fn test_tracker_state_size() {
        // TrackerState must be exactly 64 bytes for efficient rollback
        assert_eq!(std::mem::size_of::<TrackerState>(), 64);
    }

    #[test]
    fn test_z_rollback_state_size() {
        // 340 bytes audio + 64 bytes tracker = 404 bytes
        assert_eq!(std::mem::size_of::<ZRollbackState>(), 404);
    }

    #[test]
    fn test_z_rollback_state_is_pod() {
        // This compiles only if ZRollbackState is Pod
        let state = ZRollbackState::default();
        let bytes: &[u8] = bytemuck::bytes_of(&state);
        assert_eq!(bytes.len(), std::mem::size_of::<ZRollbackState>());
    }

    #[test]
    fn test_channel_state_defaults() {
        let channel = ChannelState::default();
        assert_eq!(channel.sound, 0);
        assert_eq!(channel.position, 0);
        assert_eq!(channel.looping, 0);
        assert_eq!(channel.volume, 0.0);
        assert_eq!(channel.pan, 0.0);
    }

    #[test]
    fn test_tracker_state_defaults() {
        let tracker = TrackerState::default();
        assert_eq!(tracker.handle, 0);
        assert_eq!(tracker.order_position, 0);
        assert_eq!(tracker.row, 0);
        assert_eq!(tracker.tick, 0);
        assert_eq!(tracker.speed, 0);
        assert_eq!(tracker.bpm, 0);
        assert_eq!(tracker.volume, 0);
        assert_eq!(tracker.flags, 0);
        assert_eq!(tracker.tick_sample_pos, 0);
    }

    #[test]
    fn test_channel_position_fixed_point_encoding() {
        let mut channel = ChannelState::default();

        // Test integer positions
        channel.set_position(0.0);
        let (int_pos, frac_pos) = channel.get_position();
        assert_eq!(int_pos, 0);
        assert!((frac_pos - 0.0).abs() < 0.001);

        channel.set_position(100.0);
        let (int_pos, frac_pos) = channel.get_position();
        assert_eq!(int_pos, 100);
        assert!((frac_pos - 0.0).abs() < 0.001);

        // Test fractional positions
        channel.set_position(100.5);
        let (int_pos, frac_pos) = channel.get_position();
        assert_eq!(int_pos, 100);
        assert!((frac_pos - 0.5).abs() < 0.01, "Expected ~0.5, got {}", frac_pos);

        channel.set_position(42.25);
        let (int_pos, frac_pos) = channel.get_position();
        assert_eq!(int_pos, 42);
        assert!((frac_pos - 0.25).abs() < 0.01, "Expected ~0.25, got {}", frac_pos);

        channel.set_position(99.75);
        let (int_pos, frac_pos) = channel.get_position();
        assert_eq!(int_pos, 99);
        assert!((frac_pos - 0.75).abs() < 0.01, "Expected ~0.75, got {}", frac_pos);
    }

    #[test]
    fn test_channel_position_advance() {
        let mut channel = ChannelState::default();
        channel.set_position(0.0);

        // Advance by 0.5 repeatedly (simulating 44.1kHz output from 22.05kHz source)
        for _ in 0..512 {
            channel.advance_position(0.5);
        }

        let (int_pos, frac_pos) = channel.get_position();
        assert_eq!(int_pos, 256, "After 512 advances of 0.5, position should be 256");
        assert!(frac_pos.abs() < 0.01, "Fractional part should be ~0, got {}", frac_pos);
    }

    #[test]
    fn test_channel_position_sub_sample_precision() {
        let mut channel = ChannelState::default();
        channel.set_position(0.0);

        // Test precise fractional advancement
        for _ in 0..100 {
            channel.advance_position(0.3);
        }

        let (int_pos, _frac_pos) = channel.get_position();
        // 100 * 0.3 = 30.0 (but with fixed-point rounding, may be 29 or 30)
        assert!(
            int_pos == 29 || int_pos == 30,
            "Expected ~30 (with rounding tolerance), got {}",
            int_pos
        );
    }

    #[test]
    fn test_channel_reset_position() {
        let mut channel = ChannelState::default();
        channel.set_position(1234.567);
        channel.reset_position();

        let (int_pos, frac_pos) = channel.get_position();
        assert_eq!(int_pos, 0);
        assert!((frac_pos - 0.0).abs() < 0.001);
        assert_eq!(channel.position, 0);
    }

    #[test]
    fn test_channel_position_determinism() {
        // Two channels with same operations should have identical position
        let mut ch1 = ChannelState::default();
        let mut ch2 = ChannelState::default();

        for _ in 0..1000 {
            ch1.advance_position(0.5);
            ch2.advance_position(0.5);
        }

        assert_eq!(ch1.position, ch2.position, "Determinism: positions must be bit-identical");
    }

    #[test]
    fn test_fixed_point_constants() {
        assert_eq!(ChannelState::FRAC_BITS, 8);
        assert_eq!(ChannelState::FRAC_ONE, 256);
        assert_eq!(ChannelState::FRAC_MASK, 255);
    }
}
