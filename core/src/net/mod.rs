//! Networking modules for Nethercore
//!
//! This module contains networking protocols used by Nethercore consoles:
//!
//! - [`nchs`] - Nethercore Handshake Protocol (pre-GGRS session setup)
//!
//! # Architecture
//!
//! ```text
//!                    ┌─────────────────────┐
//!                    │   Game Connection   │
//!                    └──────────┬──────────┘
//!                               │
//!                    ┌──────────▼──────────┐
//!                    │   NCHS Protocol     │
//!                    │  (Session Setup)    │
//!                    └──────────┬──────────┘
//!                               │
//!                    ┌──────────▼──────────┐
//!                    │   GGRS Protocol     │
//!                    │  (Rollback Netcode) │
//!                    └─────────────────────┘
//! ```

pub mod nchs;

// Re-export commonly used NCHS types
pub use nchs::{
    NchsConfig, NchsError, NchsEvent, NchsRole, NchsSession, NchsSocket, NchsSocketError, NchsState,
    // Message types
    JoinReject, JoinRejectReason, JoinRequest, LobbyState, NetworkConfig, PlayerConnectionInfo,
    PlayerInfo, SaveConfig, SaveMode, SessionStart,
    // Constants
    DEFAULT_NCHS_PORT,
};
