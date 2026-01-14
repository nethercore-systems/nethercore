//! Host -> Guest NCHS protocol messages

use bitcode::{Decode, Encode};
use nethercore_shared::console::TickRate;

use super::shared::{NetworkConfig, PlayerInfo, SaveConfig};

/// Accept a join request
///
/// Sent by host when a guest's JoinRequest passes validation.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct JoinAccept {
    /// Assigned player handle (0-3)
    pub player_handle: u8,
    /// Current lobby state
    pub lobby: LobbyState,
}

/// Reject a join request
///
/// Sent by host when a guest's JoinRequest fails validation.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct JoinReject {
    /// Reason for rejection
    pub reason: JoinRejectReason,
    /// Optional human-readable message
    pub message: Option<String>,
}

/// Reasons why a join request was rejected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
pub enum JoinRejectReason {
    /// Lobby is full
    LobbyFull,
    /// Console type mismatch
    ConsoleTypeMismatch,
    /// ROM hash mismatch (different game/version)
    RomHashMismatch,
    /// Tick rate mismatch
    TickRateMismatch,
    /// Game is already in progress
    GameInProgress,
    /// Host rejected manually
    HostRejected,
    /// Protocol version mismatch
    VersionMismatch,
    /// Other/unknown reason
    Other,
}

/// Lobby state update
///
/// Sent whenever the lobby state changes (player joined/left/ready).
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct LobbyUpdate {
    /// Updated lobby state
    pub lobby: LobbyState,
}

/// Complete lobby state
///
/// Contains all information about current players and lobby configuration.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct LobbyState {
    /// All player slots (up to max_players)
    pub players: Vec<PlayerSlot>,
    /// Maximum players allowed
    pub max_players: u8,
    /// Host player handle
    pub host_handle: u8,
}

/// A player slot in the lobby
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct PlayerSlot {
    /// Player handle (0-3)
    pub handle: u8,
    /// Whether slot is occupied
    pub active: bool,
    /// Player info (only valid if active)
    pub info: Option<PlayerInfo>,
    /// Whether player is ready
    pub ready: bool,
    /// Player's network address
    pub addr: Option<String>,
}

/// Game session start notification
///
/// Sent by host when all players are ready and host initiates game start.
/// Contains all determinism-critical configuration.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct SessionStart {
    // === Local player info (set by library when serializing for player process) ===
    /// Which player handle this process controls (0-3)
    /// Set by library when creating session file, not sent over network.
    pub local_player_handle: u8,

    // === Determinism-critical fields ===
    /// Random seed for deterministic RNG (all players use same seed)
    pub random_seed: u64,
    /// Starting frame number (usually 0)
    pub start_frame: u32,
    /// Tick rate for the session (must match host)
    pub tick_rate: TickRate,

    // === Network topology ===
    /// All player connection info (for peer mesh)
    pub players: Vec<PlayerConnectionInfo>,
    /// Number of active players
    pub player_count: u8,

    // === Configuration ===
    /// Network configuration (input delay, rollback settings)
    pub network_config: NetworkConfig,

    // === Save synchronization ===
    /// Save slot configuration (for synchronized saves)
    pub save_config: Option<SaveConfig>,

    // === Future expansion ===
    /// Additional data for future protocol extensions
    pub extra_data: Vec<u8>,
}

/// Player connection info for peer mesh setup
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct PlayerConnectionInfo {
    /// Player handle (0-3)
    pub handle: u8,
    /// Whether this player is active
    pub active: bool,
    /// Player info (name, avatar, color)
    pub info: PlayerInfo,
    /// Network address (e.g., "192.168.1.50:7770")
    pub addr: String,
    /// Port for GGRS traffic (may differ from NCHS port)
    pub ggrs_port: u16,
}
