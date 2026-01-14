//! Core types for NCHS sessions

use nethercore_shared::netplay::NetplayMetadata;

use crate::net::nchs::{JoinReject, NetworkConfig, PlayerInfo, SaveConfig, SessionStart};

/// NCHS session configuration
#[derive(Debug, Clone)]
pub struct NchsConfig {
    /// ROM netplay metadata (from ROM header)
    pub netplay: NetplayMetadata,
    /// Local player info
    pub player_info: PlayerInfo,
    /// Network configuration for GGRS
    pub network_config: NetworkConfig,
    /// Save slot configuration (optional)
    pub save_config: Option<SaveConfig>,
}

/// Session role
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NchsRole {
    Host,
    Guest,
}

/// Session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NchsState {
    /// Not connected
    Idle,
    /// Host: listening for connections, Guest: connecting
    Connecting,
    /// In lobby, waiting for ready/start
    Lobby,
    /// SessionStart received, doing hole punch
    Punching,
    /// All peers connected, ready for GGRS
    Ready,
    /// Session failed
    Failed,
}

/// Events emitted by NCHS session
#[derive(Debug, Clone)]
pub enum NchsEvent {
    /// No events pending
    Pending,
    /// Host is listening on port
    Listening { port: u16 },
    /// Lobby state changed
    LobbyUpdated(crate::net::nchs::LobbyState),
    /// Player joined the lobby
    PlayerJoined { handle: u8, info: PlayerInfo },
    /// Player left the lobby
    PlayerLeft { handle: u8 },
    /// All players ready, session starting
    Ready(SessionStart),
    /// Error occurred
    Error(NchsError),
}

/// NCHS session errors
#[derive(Debug, Clone)]
pub enum NchsError {
    /// Failed to bind to port
    BindFailed(String),
    /// Connection timed out
    Timeout,
    /// Join was rejected
    Rejected(JoinReject),
    /// ROM validation failed
    ValidationFailed(String),
    /// UDP hole punch failed
    PunchFailed,
    /// Network error
    NetworkError(String),
    /// Protocol error
    ProtocolError(String),
}

impl std::fmt::Display for NchsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BindFailed(e) => write!(f, "Failed to bind: {}", e),
            Self::Timeout => write!(f, "Connection timed out"),
            Self::Rejected(r) => write!(f, "Join rejected: {:?}", r.reason),
            Self::ValidationFailed(e) => write!(f, "Validation failed: {}", e),
            Self::PunchFailed => write!(f, "UDP hole punch failed"),
            Self::NetworkError(e) => write!(f, "Network error: {}", e),
            Self::ProtocolError(e) => write!(f, "Protocol error: {}", e),
        }
    }
}

impl std::error::Error for NchsError {}
