//! Rollback session events and statistics
//!
//! Contains high-level session events, connection quality assessment,
//! network statistics, and session error types.

use ggrs::GgrsError;

/// High-level session events for the application layer
///
/// These are translated from raw GGRS events into actionable events
/// that the game/UI can respond to.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// Connection synchronized with a peer
    Synchronized {
        /// Player handle that synchronized
        player_handle: usize,
    },
    /// A peer disconnected
    Disconnected {
        /// Player handle that disconnected
        player_handle: usize,
    },
    /// Desync detected between clients
    ///
    /// This is a critical error - game state has diverged and cannot be recovered.
    /// The session should be terminated.
    Desync {
        /// Frame where desync was detected
        frame: i32,
        /// Local checksum
        local_checksum: u64,
        /// Remote checksum
        remote_checksum: u64,
    },
    /// Network interrupted with a peer
    NetworkInterrupted {
        /// Player handle with network issues
        player_handle: usize,
        /// How long the connection has been interrupted (ms)
        disconnect_timeout_ms: u64,
    },
    /// Network resumed with a peer
    NetworkResumed {
        /// Player handle whose connection resumed
        player_handle: usize,
    },
    /// Frame advantage warning - local client is too far ahead
    ///
    /// This indicates potential network issues. Consider showing
    /// a "waiting for opponent" message if this persists.
    FrameAdvantageWarning {
        /// How many frames ahead of the slowest peer
        frames_ahead: i32,
    },
    /// Timesync event (internal GGRS timing adjustment)
    TimeSync {
        /// Frames to skip for synchronization
        frames_to_skip: usize,
    },
    /// Waiting for remote players (not enough input yet)
    WaitingForPlayers,
}

/// Connection quality assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionQuality {
    /// Excellent connection (< 50ms RTT, < 2 frames ahead)
    #[default]
    Excellent,
    /// Good connection (< 100ms RTT, < 4 frames ahead)
    Good,
    /// Fair connection (< 150ms RTT, < 6 frames ahead)
    Fair,
    /// Poor connection (>= 150ms RTT or >= 6 frames ahead)
    Poor,
    /// Connection interrupted
    Disconnected,
}

/// Network statistics for a player
#[derive(Debug, Clone, Default)]
pub struct PlayerNetworkStats {
    /// Round-trip time in milliseconds
    pub ping_ms: u32,
    /// Packet loss percentage (0-100)
    pub packet_loss: u8,
    /// Local frames ahead of this player
    pub local_frames_ahead: i32,
    /// Remote frames ahead of local
    pub remote_frames_ahead: i32,
    /// Number of rollback frames in last second
    pub rollback_frames: u32,
    /// Connection quality assessment
    pub quality: ConnectionQuality,
    /// Whether this player is currently connected
    pub connected: bool,
}

impl PlayerNetworkStats {
    /// Update quality assessment based on current stats
    pub fn assess_quality(&mut self) {
        if !self.connected {
            self.quality = ConnectionQuality::Disconnected;
        } else if self.ping_ms < 50 && self.local_frames_ahead.abs() < 2 {
            self.quality = ConnectionQuality::Excellent;
        } else if self.ping_ms < 100 && self.local_frames_ahead.abs() < 4 {
            self.quality = ConnectionQuality::Good;
        } else if self.ping_ms < 150 && self.local_frames_ahead.abs() < 6 {
            self.quality = ConnectionQuality::Fair;
        } else {
            self.quality = ConnectionQuality::Poor;
        }
    }
}

/// Session errors
#[derive(Debug, Clone)]
pub enum SessionError {
    /// Error during state save
    SaveState(String),
    /// Error during state load
    LoadState(String),
    /// GGRS error
    Ggrs(String),
    /// Desync detected
    Desync {
        frame: i32,
        local_checksum: u64,
        remote_checksum: u64,
    },
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SaveState(e) => write!(f, "Failed to save state: {}", e),
            Self::LoadState(e) => write!(f, "Failed to load state: {}", e),
            Self::Ggrs(e) => write!(f, "GGRS error: {}", e),
            Self::Desync {
                frame,
                local_checksum,
                remote_checksum,
            } => write!(
                f,
                "Desync detected at frame {}: local={:#x}, remote={:#x}",
                frame, local_checksum, remote_checksum
            ),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<GgrsError> for SessionError {
    fn from(e: GgrsError) -> Self {
        Self::Ggrs(e.to_string())
    }
}
