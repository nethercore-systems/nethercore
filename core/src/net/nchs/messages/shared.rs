//! Shared types used across NCHS protocol messages

use bitcode::{Decode, Encode};

/// Maximum player name length
pub const MAX_PLAYER_NAME_LEN: usize = 32;

/// Player display information
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct PlayerInfo {
    /// Player name (up to 32 characters)
    pub name: String,
    /// Avatar ID (game-specific)
    pub avatar_id: u16,
    /// Player color (RGB)
    pub color: [u8; 3],
}

impl Default for PlayerInfo {
    fn default() -> Self {
        Self {
            name: "Player".to_string(),
            avatar_id: 0,
            color: [255, 255, 255],
        }
    }
}

/// Network configuration for GGRS session
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct NetworkConfig {
    /// Input delay in frames (0-10)
    pub input_delay: u8,
    /// Maximum rollback frames (typically 8)
    pub max_rollback: u8,
    /// Disconnect timeout in milliseconds
    pub disconnect_timeout_ms: u32,
    /// Whether to enable desync detection
    pub desync_detection: bool,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            input_delay: 2,
            max_rollback: 8,
            disconnect_timeout_ms: 5000,
            desync_detection: true,
        }
    }
}

/// Save slot synchronization configuration
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct SaveConfig {
    /// Save slot index to use
    pub slot_index: u8,
    /// Save synchronization mode
    pub mode: SaveMode,
    /// Host's save data (for synchronized mode)
    pub synchronized_save: Option<Vec<u8>>,
}

/// Save synchronization mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum SaveMode {
    /// Each player uses their own save slot
    PerPlayer,
    /// All players use host's save data
    Synchronized,
    /// Fresh start, no save data
    NewGame,
}
