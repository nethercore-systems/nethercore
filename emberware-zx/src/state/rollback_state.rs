//! Emberware Z rollback state
//!
//! This module contains console-specific state that needs to be rolled back
//! during netcode rollback. Unlike FFI state (which is rebuilt each frame),
//! rollback state persists and must be serialized/deserialized with WASM memory.
//!
//! All types are POD (Plain Old Data) using bytemuck for zero-copy serialization.

use bytemuck::{Pod, Zeroable};
use emberware_core::console::ConsoleRollbackState;

/// Maximum number of sound effect channels
pub const MAX_CHANNELS: usize = 16;

/// State for a single audio channel (20 bytes, POD)
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ChannelState {
    /// Sound handle (0 = silent/no sound)
    pub sound: u32,
    /// Playhead position in samples
    pub position: u32,
    /// Whether the channel is looping (0 = no, 1 = yes)
    pub looping: u32,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Pan (-1.0 = left, 0.0 = center, 1.0 = right)
    pub pan: f32,
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

/// Emberware Z rollback state
///
/// This is the console-specific state that gets rolled back along with
/// WASM memory during netcode rollback. It contains audio playback state
/// so that sounds automatically stay in sync with game state.
#[repr(C)]
#[derive(Clone, Copy, Default, Debug, Pod, Zeroable)]
pub struct ZRollbackState {
    /// Audio playback state (channels + music)
    pub audio: AudioPlaybackState,
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
}
