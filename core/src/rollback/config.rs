//! GGRS session configuration
//!
//! Provides configuration types and constants for GGRS rollback sessions.

use std::marker::PhantomData;

use ggrs::Config;

use crate::console::ConsoleInput;

use super::state::GameStateSnapshot;

/// Maximum rollback frames (how far GGRS can rewind)
pub const MAX_ROLLBACK_FRAMES: usize = 8;

/// Maximum input delay frames (trade latency for fewer rollbacks)
pub const MAX_INPUT_DELAY: usize = 10;

/// Default input delay for local play
pub const DEFAULT_INPUT_DELAY: usize = 0;

/// Default input delay for online play (balance between responsiveness and rollbacks)
pub const DEFAULT_ONLINE_INPUT_DELAY: usize = 2;

/// Maximum state buffer size (16MB - full WASM linear memory snapshot)
/// WASM games typically use 64KB-16MB of memory. This limit accommodates
/// the largest games while preventing excessive memory usage.
pub const MAX_STATE_SIZE: usize = 16 * 1024 * 1024;

/// Number of pre-allocated state buffers in the pool
pub const STATE_POOL_SIZE: usize = MAX_ROLLBACK_FRAMES + 2;

/// GGRS configuration for Emberware
///
/// Parameterized by the console's input type (e.g., `ZInput` for Emberware Z).
/// This allows different consoles to use different input layouts while sharing
/// the rollback infrastructure.
pub struct EmberwareConfig<I: ConsoleInput> {
    _phantom: PhantomData<I>,
}

impl<I: ConsoleInput> Config for EmberwareConfig<I> {
    type Input = I;
    type State = GameStateSnapshot;
    type Address = String; // WebRTC peer address (e.g., "peer_id")
}

/// Settings for creating a GGRS session
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Number of players in the session
    pub num_players: usize,
    /// Local input delay in frames (0 = responsive, higher = fewer rollbacks)
    pub input_delay: usize,
    /// Maximum prediction frames (how far ahead we can simulate without confirmed input)
    pub max_prediction_frames: usize,
    /// Disconnect timeout in milliseconds
    pub disconnect_timeout: u64,
    /// Disconnect notify start in milliseconds
    pub disconnect_notify_start: u64,
    /// Frame rate for time sync
    pub fps: usize,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            num_players: 2,
            input_delay: DEFAULT_INPUT_DELAY,
            max_prediction_frames: MAX_ROLLBACK_FRAMES,
            disconnect_timeout: 5000,
            disconnect_notify_start: 3000,
            fps: 60,
        }
    }
}

impl SessionConfig {
    /// Create config for local play (single machine, no network)
    pub fn local(num_players: usize) -> Self {
        Self {
            num_players,
            input_delay: 0,
            ..Default::default()
        }
    }

    /// Create config for online play
    pub fn online(num_players: usize) -> Self {
        Self {
            num_players,
            input_delay: DEFAULT_ONLINE_INPUT_DELAY,
            ..Default::default()
        }
    }

    /// Create config for sync test (determinism testing)
    pub fn sync_test() -> Self {
        Self {
            num_players: 1,
            input_delay: 0,
            max_prediction_frames: MAX_ROLLBACK_FRAMES,
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_config_default() {
        let config = SessionConfig::default();
        assert_eq!(config.num_players, 2);
        assert_eq!(config.input_delay, DEFAULT_INPUT_DELAY);
        assert_eq!(config.max_prediction_frames, MAX_ROLLBACK_FRAMES);
    }

    #[test]
    fn test_session_config_local() {
        let config = SessionConfig::local(4);
        assert_eq!(config.num_players, 4);
        assert_eq!(config.input_delay, 0);
    }

    #[test]
    fn test_session_config_online() {
        let config = SessionConfig::online(2);
        assert_eq!(config.num_players, 2);
        assert_eq!(config.input_delay, DEFAULT_ONLINE_INPUT_DELAY);
    }
}
