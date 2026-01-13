//! Replay recorder
//!
//! Records gameplay inputs for later playback.

use crate::replay::types::{Checkpoint, InputSequence, Replay, ReplayFlags, ReplayHeader};

/// Configuration for the recorder
#[derive(Debug, Clone)]
pub struct RecorderConfig {
    /// Console ID
    pub console_id: u8,
    /// Number of players
    pub player_count: u8,
    /// Input size in bytes per player
    pub input_size: u8,
    /// Random seed
    pub seed: u64,
    /// Interval for checkpoints (0 = no checkpoints)
    pub checkpoint_interval: u64,
    /// Whether to compress inputs
    pub compress: bool,
}

impl Default for RecorderConfig {
    fn default() -> Self {
        Self {
            console_id: 1,
            player_count: 1,
            input_size: 8,
            seed: 0,
            checkpoint_interval: 300, // Every 5 seconds at 60fps
            compress: true,
        }
    }
}

/// Replay recorder state
pub struct Recorder {
    config: RecorderConfig,
    inputs: InputSequence,
    checkpoints: Vec<Checkpoint>,
    frame_count: u64,
    recording: bool,
}

impl Recorder {
    /// Create a new recorder with the given configuration
    pub fn new(config: RecorderConfig) -> Self {
        Self {
            config,
            inputs: InputSequence::new(),
            checkpoints: Vec::new(),
            frame_count: 0,
            recording: false,
        }
    }

    /// Start recording
    pub fn start(&mut self) {
        self.recording = true;
        self.inputs = InputSequence::new();
        self.checkpoints = Vec::new();
        self.frame_count = 0;
    }

    /// Stop recording and finalize the replay
    pub fn stop(&mut self) -> Replay {
        self.recording = false;

        let mut flags = ReplayFlags::empty();
        if self.config.compress {
            flags |= ReplayFlags::COMPRESSED_INPUTS;
        }
        if !self.checkpoints.is_empty() {
            flags |= ReplayFlags::HAS_CHECKPOINTS;
        }

        Replay {
            header: ReplayHeader {
                console_id: self.config.console_id,
                player_count: self.config.player_count,
                input_size: self.config.input_size,
                flags,
                reserved: [0; 4],
                seed: self.config.seed,
                frame_count: self.frame_count,
            },
            inputs: std::mem::take(&mut self.inputs),
            checkpoints: std::mem::take(&mut self.checkpoints),
            assertions: Vec::new(),
        }
    }

    /// Check if recording is active
    pub fn is_recording(&self) -> bool {
        self.recording
    }

    /// Record a frame of inputs
    pub fn record_frame(&mut self, player_inputs: Vec<Vec<u8>>) {
        if !self.recording {
            return;
        }

        self.inputs.push_frame(player_inputs);
        self.frame_count += 1;
    }

    /// Record a checkpoint (state snapshot)
    pub fn record_checkpoint(&mut self, state: Vec<u8>) {
        if !self.recording {
            return;
        }

        self.checkpoints.push(Checkpoint {
            frame: self.frame_count,
            state,
        });
    }

    /// Check if a checkpoint should be recorded at the current frame
    pub fn should_checkpoint(&self) -> bool {
        if !self.recording || self.config.checkpoint_interval == 0 {
            return false;
        }
        self.frame_count > 0
            && self
                .frame_count
                .is_multiple_of(self.config.checkpoint_interval)
    }

    /// Get the current frame count
    pub fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the configuration
    pub fn config(&self) -> &RecorderConfig {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recorder_basic() {
        let config = RecorderConfig {
            console_id: 1,
            player_count: 1,
            input_size: 1,
            seed: 12345,
            checkpoint_interval: 0,
            compress: false,
        };

        let mut recorder = Recorder::new(config);
        assert!(!recorder.is_recording());

        recorder.start();
        assert!(recorder.is_recording());

        recorder.record_frame(vec![vec![0x00]]);
        recorder.record_frame(vec![vec![0x10]]);
        recorder.record_frame(vec![vec![0x00]]);

        assert_eq!(recorder.frame_count(), 3);

        let replay = recorder.stop();
        assert!(!recorder.is_recording());
        assert_eq!(replay.header.frame_count, 3);
        assert_eq!(replay.header.seed, 12345);
        assert_eq!(replay.inputs.frame_count(), 3);
    }

    #[test]
    fn test_recorder_with_checkpoints() {
        let config = RecorderConfig {
            console_id: 1,
            player_count: 1,
            input_size: 1,
            seed: 0,
            checkpoint_interval: 3,
            compress: true,
        };

        let mut recorder = Recorder::new(config);
        recorder.start();

        for i in 0..10 {
            recorder.record_frame(vec![vec![i as u8]]);
            if recorder.should_checkpoint() {
                recorder.record_checkpoint(vec![0xAA, 0xBB]);
            }
        }

        let replay = recorder.stop();

        assert!(replay.header.flags.contains(ReplayFlags::COMPRESSED_INPUTS));
        assert!(replay.header.flags.contains(ReplayFlags::HAS_CHECKPOINTS));
        assert_eq!(replay.checkpoints.len(), 3); // At frames 3, 6, 9
    }

    #[test]
    fn test_recorder_multiplayer() {
        let config = RecorderConfig {
            console_id: 1,
            player_count: 2,
            input_size: 2,
            seed: 0,
            checkpoint_interval: 0,
            compress: false,
        };

        let mut recorder = Recorder::new(config);
        recorder.start();

        recorder.record_frame(vec![vec![0x00, 0x00], vec![0x10, 0x01]]);
        recorder.record_frame(vec![vec![0x08, 0x00], vec![0x00, 0x00]]);

        let replay = recorder.stop();

        assert_eq!(replay.header.player_count, 2);
        assert_eq!(replay.header.input_size, 2);

        let frame0 = replay.inputs.get_frame(0).unwrap();
        assert_eq!(frame0.len(), 2); // 2 players
        assert_eq!(frame0[0], vec![0x00, 0x00]);
        assert_eq!(frame0[1], vec![0x10, 0x01]);
    }
}
