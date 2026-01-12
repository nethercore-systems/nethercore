//! Replay player
//!
//! Plays back recorded replays.

use crate::replay::types::{Checkpoint, Replay};

/// Playback configuration
#[derive(Debug, Clone)]
pub struct PlayerConfig {
    /// Playback speed multiplier (1.0 = normal)
    pub speed: f32,
    /// Loop playback when reaching the end
    pub loop_playback: bool,
    /// Show debug overlay
    pub show_debug: bool,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self {
            speed: 1.0,
            loop_playback: false,
            show_debug: false,
        }
    }
}

/// Replay player state
pub struct Player {
    replay: Replay,
    config: PlayerConfig,
    current_frame: u64,
    playing: bool,
    paused: bool,
}

impl Player {
    /// Create a new player for the given replay
    pub fn new(replay: Replay, config: PlayerConfig) -> Self {
        Self {
            replay,
            config,
            current_frame: 0,
            playing: false,
            paused: false,
        }
    }

    /// Start playback
    pub fn play(&mut self) {
        self.playing = true;
        self.paused = false;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        self.paused = true;
    }

    /// Resume playback
    pub fn resume(&mut self) {
        self.paused = false;
    }

    /// Stop playback and reset to beginning
    pub fn stop(&mut self) {
        self.playing = false;
        self.paused = false;
        self.current_frame = 0;
    }

    /// Check if playback is active
    pub fn is_playing(&self) -> bool {
        self.playing && !self.paused
    }

    /// Check if paused
    pub fn is_paused(&self) -> bool {
        self.paused
    }

    /// Check if playback is complete
    pub fn is_complete(&self) -> bool {
        self.current_frame >= self.replay.header.frame_count
    }

    /// Get the current frame number
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Get the total frame count
    pub fn frame_count(&self) -> u64 {
        self.replay.header.frame_count
    }

    /// Get playback progress (0.0 to 1.0)
    pub fn progress(&self) -> f32 {
        if self.replay.header.frame_count == 0 {
            return 0.0;
        }
        self.current_frame as f32 / self.replay.header.frame_count as f32
    }

    /// Get the inputs for the current frame
    pub fn current_inputs(&self) -> Option<&Vec<Vec<u8>>> {
        self.replay.inputs.get_frame(self.current_frame)
    }

    /// Advance to the next frame
    pub fn advance_frame(&mut self) -> bool {
        if self.is_complete() {
            if self.config.loop_playback {
                self.current_frame = 0;
                true
            } else {
                self.playing = false;
                false
            }
        } else {
            self.current_frame += 1;
            true
        }
    }

    /// Seek to a specific frame
    pub fn seek(&mut self, frame: u64) -> SeekResult {
        if frame >= self.replay.header.frame_count {
            return SeekResult::OutOfRange;
        }

        // Check if we can use a checkpoint
        if let Some(checkpoint) = self.find_nearest_checkpoint(frame).cloned() {
            self.current_frame = checkpoint.frame;
            SeekResult::NeedsStateLoad(checkpoint)
        } else if frame < self.current_frame {
            // Need to rewind from beginning
            self.current_frame = 0;
            SeekResult::NeedsRewind { target: frame }
        } else {
            // Can fast-forward
            self.current_frame = frame;
            SeekResult::Immediate
        }
    }

    /// Find the nearest checkpoint before the target frame
    fn find_nearest_checkpoint(&self, target_frame: u64) -> Option<&Checkpoint> {
        self.replay
            .checkpoints
            .iter()
            .filter(|c| c.frame <= target_frame)
            .max_by_key(|c| c.frame)
    }

    /// Get the replay header
    pub fn header(&self) -> &crate::replay::types::ReplayHeader {
        &self.replay.header
    }

    /// Get the player configuration
    pub fn config(&self) -> &PlayerConfig {
        &self.config
    }

    /// Set the playback speed
    pub fn set_speed(&mut self, speed: f32) {
        self.config.speed = speed.clamp(0.1, 10.0);
    }

    /// Get the effective frame duration based on speed
    pub fn frame_duration_ms(&self, base_tick_rate: u32) -> f32 {
        (1000.0 / base_tick_rate as f32) / self.config.speed
    }
}

/// Result of a seek operation
#[derive(Debug, Clone)]
pub enum SeekResult {
    /// Seek completed immediately (no state load needed)
    Immediate,
    /// Need to load state from checkpoint first
    NeedsStateLoad(Checkpoint),
    /// Need to rewind and replay from beginning
    NeedsRewind { target: u64 },
    /// Target frame is out of range
    OutOfRange,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::types::{InputSequence, ReplayFlags, ReplayHeader};

    fn create_test_replay(frames: u64) -> Replay {
        let mut inputs = InputSequence::new();
        for i in 0..frames {
            inputs.push_frame(vec![vec![i as u8]]);
        }

        Replay {
            header: ReplayHeader {
                console_id: 1,
                player_count: 1,
                input_size: 1,
                flags: ReplayFlags::empty(),
                reserved: [0; 4],
                seed: 0,
                frame_count: frames,
            },
            inputs,
            checkpoints: Vec::new(),
            assertions: Vec::new(),
        }
    }

    #[test]
    fn test_player_basic() {
        let replay = create_test_replay(10);
        let mut player = Player::new(replay, PlayerConfig::default());

        assert!(!player.is_playing());
        assert_eq!(player.current_frame(), 0);
        assert_eq!(player.frame_count(), 10);

        player.play();
        assert!(player.is_playing());

        // Advance through all frames
        for _ in 0..10 {
            assert!(player.advance_frame());
        }

        assert!(player.is_complete());
    }

    #[test]
    fn test_player_seek() {
        let replay = create_test_replay(100);
        let mut player = Player::new(replay, PlayerConfig::default());

        // Seek forward
        let result = player.seek(50);
        assert!(matches!(result, SeekResult::Immediate));
        assert_eq!(player.current_frame(), 50);

        // Seek backward (needs rewind)
        let result = player.seek(25);
        assert!(matches!(result, SeekResult::NeedsRewind { target: 25 }));

        // Seek out of range
        let result = player.seek(200);
        assert!(matches!(result, SeekResult::OutOfRange));
    }

    #[test]
    fn test_player_with_checkpoints() {
        let mut replay = create_test_replay(100);
        replay.checkpoints = vec![
            Checkpoint {
                frame: 30,
                state: vec![0xAA],
            },
            Checkpoint {
                frame: 60,
                state: vec![0xBB],
            },
        ];

        let mut player = Player::new(replay, PlayerConfig::default());

        // Seek to frame 50 - should use checkpoint at 30
        let result = player.seek(50);
        match result {
            SeekResult::NeedsStateLoad(checkpoint) => {
                assert_eq!(checkpoint.frame, 30);
            }
            _ => panic!("Expected NeedsStateLoad"),
        }

        // Seek to frame 70 - should use checkpoint at 60
        player.current_frame = 0;
        let result = player.seek(70);
        match result {
            SeekResult::NeedsStateLoad(checkpoint) => {
                assert_eq!(checkpoint.frame, 60);
            }
            _ => panic!("Expected NeedsStateLoad"),
        }
    }

    #[test]
    fn test_player_loop() {
        let replay = create_test_replay(5);
        let config = PlayerConfig {
            loop_playback: true,
            ..Default::default()
        };
        let mut player = Player::new(replay, config);
        player.play();

        // Advance past end
        for _ in 0..6 {
            player.advance_frame();
        }

        // Should loop back to beginning
        assert!(!player.is_complete());
        assert!(player.current_frame() < 5);
    }

    #[test]
    fn test_player_speed() {
        let replay = create_test_replay(10);
        let mut player = Player::new(replay, PlayerConfig::default());

        player.set_speed(2.0);
        assert_eq!(player.frame_duration_ms(60), 1000.0 / 60.0 / 2.0);

        player.set_speed(0.5);
        assert_eq!(player.frame_duration_ms(60), 1000.0 / 60.0 / 0.5);
    }
}
