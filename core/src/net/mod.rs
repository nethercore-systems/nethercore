//! Networking modules for Nethercore
//!
//! This module contains networking protocols used by Nethercore consoles:
//!
//! - [`nchs`] - Nethercore Handshake Protocol (pre-GGRS session setup)
//!
//! # Architecture
//!
//! ```text
//!                    ┌─────────────────────━E
//!                    ━E  Game Connection   ━E
//!                    └──────────┬──────────━E
//!                               ━E
//!                    ┌──────────▼──────────━E
//!                    ━E  NCHS Protocol     ━E
//!                    ━E (Session Setup)    ━E
//!                    └──────────┬──────────━E
//!                               ━E
//!                    ┌──────────▼──────────━E
//!                    ━E  GGRS Protocol     ━E
//!                    ━E (Rollback Netcode) ━E
//!                    └─────────────────────━E
//! ```

pub mod nchs;

// Re-export commonly used NCHS types
pub use nchs::{
    // Constants
    DEFAULT_NCHS_PORT,
    // Message types
    JoinReject,
    JoinRejectReason,
    JoinRequest,
    LobbyState,
    NchsConfig,
    NchsError,
    NchsEvent,
    NchsRole,
    NchsSession,
    NchsSocket,
    NchsSocketError,
    NchsState,
    NetworkConfig,
    PlayerConnectionInfo,
    PlayerInfo,
    SaveConfig,
    SaveMode,
    SessionStart,
};
