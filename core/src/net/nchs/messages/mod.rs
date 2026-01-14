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

// Submodules
mod guest;
mod host;
mod peer;
mod shared;

#[cfg(test)]
mod tests;

// Re-export all public types
pub use guest::{GuestReady, JoinRequest};
pub use host::{
    JoinAccept, JoinReject, JoinRejectReason, LobbyState, LobbyUpdate, PlayerConnectionInfo,
    PlayerSlot, SessionStart,
};
pub use peer::{PunchAck, PunchHello};
pub use shared::{NetworkConfig, PlayerInfo, SaveConfig, SaveMode, MAX_PLAYER_NAME_LEN};

/// NCHS protocol magic bytes
pub const NCHS_MAGIC: [u8; 4] = *b"NCHS";

/// Current NCHS protocol version
pub const NCHS_VERSION: u16 = 1;

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
