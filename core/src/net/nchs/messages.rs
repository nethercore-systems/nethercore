//! NCHS (Nethercore Handshake) Protocol Messages
//!
//! This module defines all message types used in the NCHS protocol for
//! pre-GGRS multiplayer setup. Messages are serialized using bitcode for
//! efficient binary encoding with extensibility via Vec and Option fields.
//!
//! # Wire Format
//!
//! ```text
//! [NCHS][version:u16][length:u32][bitcode payload...]
//! ```

use bitcode::{Decode, Encode};
use nethercore_shared::console::{ConsoleType, TickRate};

/// NCHS protocol magic bytes
pub const NCHS_MAGIC: [u8; 4] = *b"NCHS";

/// Current NCHS protocol version
pub const NCHS_VERSION: u16 = 1;

/// Maximum player name length
pub const MAX_PLAYER_NAME_LEN: usize = 32;

/// Header size: magic (4) + version (2) + length (4)
pub const NCHS_HEADER_SIZE: usize = 10;

// ============================================================================
// Core Message Enum
// ============================================================================

/// Top-level NCHS message enum
///
/// All NCHS communication uses this enum, serialized with bitcode.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub enum NchsMessage {
    // Guest -> Host
    /// Request to join a game session
    JoinRequest(JoinRequest),
    /// Guest signals ready to start
    GuestReady(GuestReady),

    // Host -> Guest
    /// Accept a join request
    JoinAccept(JoinAccept),
    /// Reject a join request
    JoinReject(JoinReject),
    /// Lobby state update (player joined/left/ready changed)
    LobbyUpdate(LobbyUpdate),
    /// Game session is starting
    SessionStart(SessionStart),

    // Peer <-> Peer
    /// UDP hole punch initiation
    PunchHello(PunchHello),
    /// UDP hole punch acknowledgement
    PunchAck(PunchAck),

    // Any direction
    /// Keepalive ping
    Ping,
    /// Keepalive pong
    Pong,
}

// ============================================================================
// Guest -> Host Messages
// ============================================================================

/// Request to join a game session
///
/// Sent by a guest when connecting to a host. Contains all information
/// needed for validation and lobby display.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct JoinRequest {
    /// Console type (must match host)
    pub console_type: ConsoleType,
    /// xxHash3 of ROM WASM bytecode (must match host)
    pub rom_hash: u64,
    /// Tick rate in Hz (must match host)
    pub tick_rate: TickRate,
    /// Maximum players supported by guest's ROM
    pub max_players: u8,
    /// Guest's player info for lobby display
    pub player_info: PlayerInfo,
    /// Guest's local address for peer connections (e.g., "192.168.1.50:7770")
    pub local_addr: String,
    /// Future expansion data (ignored if unknown)
    pub extra_data: Vec<u8>,
}

/// Guest signals ready to start
///
/// Sent when a guest toggles their ready state.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct GuestReady {
    /// Whether the guest is ready
    pub ready: bool,
}

// ============================================================================
// Host -> Guest Messages
// ============================================================================

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

// ============================================================================
// Peer <-> Peer Messages
// ============================================================================

/// UDP hole punch initiation
///
/// Sent by guests to each other to establish peer connections
/// after receiving SessionStart from host.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct PunchHello {
    /// Sender's player handle
    pub sender_handle: u8,
    /// Nonce for matching hello/ack pairs
    pub nonce: u64,
}

/// UDP hole punch acknowledgement
///
/// Response to PunchHello confirming connection established.
#[derive(Debug, Clone, PartialEq, Encode, Decode)]
pub struct PunchAck {
    /// Sender's player handle
    pub sender_handle: u8,
    /// Nonce from corresponding PunchHello
    pub nonce: u64,
}

// ============================================================================
// Shared Types
// ============================================================================

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

// ============================================================================
// Serialization
// ============================================================================

impl NchsMessage {
    /// Serialize message to bytes with NCHS framing
    ///
    /// Returns wire format: [NCHS][version:u16][length:u32][payload...]
    pub fn to_bytes(&self) -> Vec<u8> {
        let payload = bitcode::encode(self);
        let mut bytes = Vec::with_capacity(NCHS_HEADER_SIZE + payload.len());

        // Magic
        bytes.extend_from_slice(&NCHS_MAGIC);
        // Version
        bytes.extend_from_slice(&NCHS_VERSION.to_le_bytes());
        // Length
        bytes.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        // Payload
        bytes.extend_from_slice(&payload);

        bytes
    }

    /// Deserialize message from bytes with NCHS framing
    ///
    /// Validates magic, version, and length before decoding payload.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, NchsDecodeError> {
        if bytes.len() < NCHS_HEADER_SIZE {
            return Err(NchsDecodeError::TooShort);
        }

        // Check magic
        if bytes[0..4] != NCHS_MAGIC {
            return Err(NchsDecodeError::InvalidMagic);
        }

        // Check version
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != NCHS_VERSION {
            return Err(NchsDecodeError::VersionMismatch {
                expected: NCHS_VERSION,
                got: version,
            });
        }

        // Get length
        let length = u32::from_le_bytes([bytes[6], bytes[7], bytes[8], bytes[9]]) as usize;

        // Validate length
        if bytes.len() < NCHS_HEADER_SIZE + length {
            return Err(NchsDecodeError::IncompletePayload {
                expected: length,
                got: bytes.len() - NCHS_HEADER_SIZE,
            });
        }

        // Decode payload
        let payload = &bytes[NCHS_HEADER_SIZE..NCHS_HEADER_SIZE + length];
        bitcode::decode(payload).map_err(|e| NchsDecodeError::DecodeFailed(e.to_string()))
    }
}

/// Errors that can occur when decoding NCHS messages
#[derive(Debug, Clone, PartialEq)]
pub enum NchsDecodeError {
    /// Message too short for header
    TooShort,
    /// Invalid magic bytes
    InvalidMagic,
    /// Protocol version mismatch
    VersionMismatch { expected: u16, got: u16 },
    /// Payload incomplete
    IncompletePayload { expected: usize, got: usize },
    /// Bitcode decode failed
    DecodeFailed(String),
}

impl std::fmt::Display for NchsDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "Message too short for NCHS header"),
            Self::InvalidMagic => write!(f, "Invalid NCHS magic bytes"),
            Self::VersionMismatch { expected, got } => {
                write!(
                    f,
                    "NCHS version mismatch: expected {}, got {}",
                    expected, got
                )
            }
            Self::IncompletePayload { expected, got } => {
                write!(
                    f,
                    "Incomplete payload: expected {} bytes, got {}",
                    expected, got
                )
            }
            Self::DecodeFailed(e) => write!(f, "Failed to decode NCHS message: {}", e),
        }
    }
}

impl std::error::Error for NchsDecodeError {}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_request_roundtrip() {
        let msg = NchsMessage::JoinRequest(JoinRequest {
            console_type: ConsoleType::ZX,
            rom_hash: 0xDEADBEEF12345678,
            tick_rate: TickRate::Fixed60,
            max_players: 4,
            player_info: PlayerInfo {
                name: "TestPlayer".to_string(),
                avatar_id: 42,
                color: [255, 128, 0],
            },
            local_addr: "192.168.1.50:7770".to_string(),
            extra_data: vec![1, 2, 3],
        });

        let bytes = msg.to_bytes();
        let decoded = NchsMessage::from_bytes(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_session_start_roundtrip() {
        let msg = NchsMessage::SessionStart(SessionStart {
            local_player_handle: 0,
            random_seed: 0x123456789ABCDEF0,
            start_frame: 0,
            tick_rate: TickRate::Fixed60,
            players: vec![
                PlayerConnectionInfo {
                    handle: 0,
                    active: true,
                    info: PlayerInfo::default(),
                    addr: "192.168.1.50:7770".to_string(),
                    ggrs_port: 7771,
                },
                PlayerConnectionInfo {
                    handle: 1,
                    active: true,
                    info: PlayerInfo {
                        name: "Player2".to_string(),
                        avatar_id: 1,
                        color: [0, 255, 0],
                    },
                    addr: "192.168.1.51:7770".to_string(),
                    ggrs_port: 7771,
                },
            ],
            player_count: 2,
            network_config: NetworkConfig::default(),
            save_config: Some(SaveConfig {
                slot_index: 0,
                mode: SaveMode::Synchronized,
                synchronized_save: Some(vec![1, 2, 3, 4]),
            }),
            extra_data: vec![],
        });

        let bytes = msg.to_bytes();
        let decoded = NchsMessage::from_bytes(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_ping_pong_roundtrip() {
        let ping = NchsMessage::Ping;
        let pong = NchsMessage::Pong;

        let ping_bytes = ping.to_bytes();
        let pong_bytes = pong.to_bytes();

        assert_eq!(NchsMessage::from_bytes(&ping_bytes).unwrap(), ping);
        assert_eq!(NchsMessage::from_bytes(&pong_bytes).unwrap(), pong);
    }

    #[test]
    fn test_invalid_magic() {
        let bytes = [b'X', b'X', b'X', b'X', 0, 0, 0, 0, 0, 0];
        let result = NchsMessage::from_bytes(&bytes);
        assert!(matches!(result, Err(NchsDecodeError::InvalidMagic)));
    }

    #[test]
    fn test_version_mismatch() {
        let mut bytes = NchsMessage::Ping.to_bytes();
        bytes[4] = 99; // Set version to 99
        bytes[5] = 0;
        let result = NchsMessage::from_bytes(&bytes);
        assert!(matches!(
            result,
            Err(NchsDecodeError::VersionMismatch {
                expected: 1,
                got: 99
            })
        ));
    }

    #[test]
    fn test_too_short() {
        let bytes = [b'N', b'C', b'H', b'S'];
        let result = NchsMessage::from_bytes(&bytes);
        assert!(matches!(result, Err(NchsDecodeError::TooShort)));
    }

    #[test]
    fn test_join_reject_roundtrip() {
        let msg = NchsMessage::JoinReject(JoinReject {
            reason: JoinRejectReason::RomHashMismatch,
            message: Some("You have a different version of the game".to_string()),
        });

        let bytes = msg.to_bytes();
        let decoded = NchsMessage::from_bytes(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_lobby_state_roundtrip() {
        let msg = NchsMessage::LobbyUpdate(LobbyUpdate {
            lobby: LobbyState {
                players: vec![
                    PlayerSlot {
                        handle: 0,
                        active: true,
                        info: Some(PlayerInfo::default()),
                        ready: true,
                        addr: Some("192.168.1.50:7770".to_string()),
                    },
                    PlayerSlot {
                        handle: 1,
                        active: false,
                        info: None,
                        ready: false,
                        addr: None,
                    },
                ],
                max_players: 4,
                host_handle: 0,
            },
        });

        let bytes = msg.to_bytes();
        let decoded = NchsMessage::from_bytes(&bytes).unwrap();
        assert_eq!(msg, decoded);
    }

    #[test]
    fn test_punch_messages_roundtrip() {
        let hello = NchsMessage::PunchHello(PunchHello {
            sender_handle: 1,
            nonce: 0xCAFEBABE,
        });
        let ack = NchsMessage::PunchAck(PunchAck {
            sender_handle: 2,
            nonce: 0xCAFEBABE,
        });

        let hello_bytes = hello.to_bytes();
        let ack_bytes = ack.to_bytes();

        assert_eq!(NchsMessage::from_bytes(&hello_bytes).unwrap(), hello);
        assert_eq!(NchsMessage::from_bytes(&ack_bytes).unwrap(), ack);
    }
}
