//! Emberware Z rollback state
//!
//! This state is saved/restored during GGRS rollback alongside WASM memory.
//! It contains host-side deterministic state like audio playback positions.
//!
//! All fields must be POD (Plain Old Data) for zero-copy serialization via bytemuck.

use bytemuck::{Pod, Zeroable};
use emberware_core::ConsoleRollbackState;

use crate::audio::AudioPlaybackState;

/// Emberware Z rollback state
///
/// This POD struct is saved/restored during GGRS rollback to maintain
/// deterministic audio playback across network peers.
///
/// Size: 272 bytes (16 channels × 16 bytes + 16 bytes music)
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct ZRollbackState {
    /// Audio playback state (channels, music, playhead positions)
    pub audio: AudioPlaybackState,
}

impl Default for ZRollbackState {
    fn default() -> Self {
        Self {
            audio: AudioPlaybackState::new(),
        }
    }
}

impl ZRollbackState {
    /// Create a new rollback state
    pub fn new() -> Self {
        Self::default()
    }
}

// Implement ConsoleRollbackState trait for zero-copy serialization
impl ConsoleRollbackState for ZRollbackState {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_z_rollback_state_is_pod() {
        // Verify we can serialize/deserialize via bytemuck
        let state = ZRollbackState::new();
        let bytes: &[u8] = bytemuck::bytes_of(&state);
        let _restored: ZRollbackState = *bytemuck::from_bytes(bytes);
    }

    #[test]
    fn test_z_rollback_state_size() {
        // AudioPlaybackState should be exactly the size we expect
        // 16 channels × 16 bytes + 16 bytes music = 272 bytes
        let expected_size = std::mem::size_of::<crate::audio::ChannelState>()
            * crate::audio::MAX_SFX_CHANNELS
            + std::mem::size_of::<crate::audio::MusicState>();
        assert_eq!(
            std::mem::size_of::<ZRollbackState>(),
            expected_size,
            "ZRollbackState should match AudioPlaybackState size"
        );
    }
}
