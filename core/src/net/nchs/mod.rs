//! NCHS (Nethercore Handshake) Protocol
//!
//! The NCHS protocol establishes multiplayer sessions before GGRS takes over.
//! It handles:
//!
//! - Player discovery and connection (direct IP:port for now)
//! - ROM validation (console type, rom_hash, tick_rate must match)
//! - Lobby management (join, ready, start)
//! - Deterministic session setup (random seed, player handles)
//! - UDP hole punching for peer mesh
//!
//! # Protocol Flow
//!
//! ```text
//! Guest                          Host
//!   |                              |
//!   |--- JoinRequest ------------->|  (ROM validation)
//!   |<-- JoinAccept/JoinReject ----|
//!   |                              |
//!   |<-- LobbyUpdate --------------|  (player joined/left/ready)
//!   |                              |
//!   |--- GuestReady -------------->|  (toggle ready)
//!   |                              |
//!   |<-- SessionStart -------------|  (host starts game)
//!   |                              |
//!   |<-- PunchHello (peer) ------->|  (UDP hole punch)
//!   |<-- PunchAck (peer) --------->|
//!   |                              |
//!   +======= GGRS Session =========+
//! ```
//!
//! # Usage
//!
//! ```rust,ignore
//! use nethercore_core::net::nchs::{NchsSession, NchsConfig, NchsEvent};
//!
//! // Host a game
//! let mut session = NchsSession::host(7770, config)?;
//!
//! // Guest joins
//! let mut session = NchsSession::join("192.168.1.50:7770", config)?;
//!
//! // Poll for events
//! match session.poll() {
//!     NchsEvent::LobbyUpdated(lobby) => { /* update UI */ }
//!     NchsEvent::Ready(session_start) => { /* transition to GGRS */ }
//!     _ => {}
//! }
//!
//! // Convert to GGRS session when ready
//! let ggrs = session.into_ggrs(max_state_size)?;
//! ```

pub mod guest;
pub mod host;
pub mod messages;
pub mod session;
pub mod socket;
pub mod types;
pub mod validation;

#[cfg(test)]
mod tests;

// Re-export state machine types
pub use guest::{GuestEvent, GuestState, GuestStateMachine};
pub use host::{HostEvent, HostState, HostStateMachine};
pub use validation::{is_netplay_compatible, validate_join_request};

// Re-export message types
pub use messages::{
    GuestReady, JoinAccept, JoinReject, JoinRejectReason, JoinRequest, LobbyState, LobbyUpdate,
    NCHS_HEADER_SIZE, NCHS_MAGIC, NCHS_VERSION, NchsDecodeError, NchsMessage, NetworkConfig,
    PlayerConnectionInfo, PlayerInfo, PlayerSlot, PunchAck, PunchHello, SaveConfig, SaveMode,
    SessionStart,
};

// Re-export socket types
pub use socket::{DEFAULT_NCHS_PORT, NchsSocket, NchsSocketError};

// Re-export session types
pub use session::NchsSession;
pub use types::{NchsConfig, NchsError, NchsEvent, NchsRole, NchsState};
