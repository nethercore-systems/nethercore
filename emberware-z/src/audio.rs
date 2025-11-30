//! Emberware Z audio backend
//!
//! PS1/N64-style audio system with:
//! - 22,050 Hz sample rate (authentic retro)
//! - 16-bit signed PCM mono format
//! - 16 managed channels for sound effects
//! - Dedicated music channel
//! - Rollback-aware audio command buffering
//!
//! TODO: Full rodio integration requires thread-safe audio handling.
//! This stub implementation provides the API structure and command buffering.
//! Actual audio playback will be implemented via audio server thread + message passing.

use std::sync::Arc;
use tracing::warn;

/// Maximum number of sound effect channels
#[allow(dead_code)] // Used in full audio implementation
pub const MAX_CHANNELS: usize = 16;

/// Audio sample rate (22.05 kHz - PS1/N64 authentic)
#[allow(dead_code)] // Used in full audio implementation
pub const SAMPLE_RATE: u32 = 22_050;

/// Sound data (raw PCM)
#[derive(Clone, Debug)]
pub struct Sound {
    /// Raw PCM data (16-bit signed, mono, 22.05kHz)
    #[allow(dead_code)] // Used in full audio implementation
    pub data: Arc<Vec<i16>>,
}

/// Audio commands buffered per frame
#[derive(Debug, Clone)]
#[allow(dead_code)] // Used in full audio implementation
pub enum AudioCommand {
    /// Play sound on next available channel
    PlaySound {
        sound: u32,
        volume: f32,
        pan: f32,
    },
    /// Play sound on specific channel
    ChannelPlay {
        channel: u32,
        sound: u32,
        volume: f32,
        pan: f32,
        looping: bool,
    },
    /// Update channel parameters
    ChannelSet {
        channel: u32,
        volume: f32,
        pan: f32,
    },
    /// Stop channel
    ChannelStop {
        channel: u32,
    },
    /// Play music (looping)
    MusicPlay {
        sound: u32,
        volume: f32,
    },
    /// Stop music
    MusicStop,
    /// Set music volume
    MusicSetVolume {
        volume: f32,
    },
}

/// Emberware Z audio backend (stub implementation)
pub struct ZAudio {
    /// Whether audio is in rollback mode (commands discarded)
    rollback_mode: bool,
}

impl ZAudio {
    /// Create new audio backend
    pub fn new() -> Result<Self, String> {
        warn!("Audio backend initialized (stub - no audio playback yet)");
        Ok(Self {
            rollback_mode: false,
        })
    }

    /// Set rollback mode
    pub fn set_rollback_mode(&mut self, rolling_back: bool) {
        self.rollback_mode = rolling_back;
    }

    /// Process buffered audio commands
    #[allow(dead_code)] // Used in full audio implementation
    pub fn process_commands(&mut self, commands: &[AudioCommand], _sounds: &[Option<Sound>]) {
        if self.rollback_mode {
            // Discard playback commands during rollback
            return;
        }

        // TODO: Implement actual audio playback
        // For now, just count commands for debugging
        if !commands.is_empty() {
            tracing::trace!("Processing {} audio commands (stub)", commands.len());
        }
    }
}
