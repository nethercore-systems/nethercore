//! Local UDP socket for P2P testing without signaling server
//!
//! This module provides a simple UDP socket wrapper that implements GGRS's
//! `NonBlockingSocket` trait, allowing local network testing without the
//! complexity of WebRTC signaling.
//!
//! # Usage
//!
//! For 2-player local testing, run two instances of the application:
//!
//! ```ignore
//! // Instance 1 (host):
//! let socket = LocalSocket::bind("127.0.0.1:7777")?;
//! socket.connect("127.0.0.1:7778")?;
//!
//! // Instance 2 (client):
//! let socket = LocalSocket::bind("127.0.0.1:7778")?;
//! socket.connect("127.0.0.1:7777")?;
//!
//! // Create P2P session (same for both):
//! let players = vec![
//!     (0, PlayerType::Local),
//!     (1, PlayerType::Remote(peer_addr)),
//! ];
//! let session = RollbackSession::<ZInput>::new_p2p(config, socket, players)?;
//! ```
//!
//! # Limitations
//!
//! - No NAT traversal (localhost only)
//! - No ICE/STUN/TURN
//! - Simple point-to-point (no mesh networking for >2 players without manual port assignment)

mod error;
mod ggrs_impl;
mod socket;

#[cfg(test)]
mod tests;

// Re-export public API
pub use error::LocalSocketError;
pub use socket::{LocalSocket, DEFAULT_LOCAL_PORT};
