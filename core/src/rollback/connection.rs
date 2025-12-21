//! Connection mode types for multiplayer sessions
//!
//! This module defines the different connection modes supported by the player:
//!
//! - `Local`: Single machine, couch co-op (no rollback)
//! - `SyncTest`: Determinism testing (simulates rollback every frame)
//! - `Host`: Host a P2P session, wait for peers to connect
//! - `Join`: Join an existing P2P session
//! - `P2P`: Direct P2P connection with explicit ports (for local testing)

/// Connection mode for multiplayer sessions
///
/// Determines how the session is created and connected.
#[derive(Debug, Clone, Default)]
pub enum ConnectionMode {
    /// Local single-player or couch co-op (no rollback)
    ///
    /// All players use the same machine. Input is processed immediately
    /// without network delay or rollback.
    #[default]
    Local,

    /// Sync test mode for determinism validation
    ///
    /// Simulates rollback every frame to catch non-determinism bugs.
    /// Used during development before releasing a game for online play.
    SyncTest {
        /// Number of frames between state checksums (default: 2)
        check_distance: usize,
    },

    /// Host a P2P session, waiting for peers to connect
    ///
    /// Binds to all interfaces on the specified port and waits for
    /// incoming connections. Share your IP with friends to let them join.
    Host {
        /// Port to listen on (default: 7777)
        port: u16,
    },

    /// Join an existing P2P session
    ///
    /// Connects to a host at the specified address.
    Join {
        /// Host address in "ip:port" format (e.g., "192.168.1.100:7777")
        address: String,
    },

    /// Direct P2P connection with explicit ports
    ///
    /// For local testing where both instances run on the same machine.
    /// Each instance binds to its own port and connects to the peer's port.
    P2P {
        /// Local port to bind to
        bind_port: u16,
        /// Peer port to connect to
        peer_port: u16,
        /// Which player this instance controls (0 or 1)
        local_player: usize,
    },
}

impl ConnectionMode {
    /// Create a sync test connection mode with default check distance
    pub fn sync_test() -> Self {
        Self::SyncTest { check_distance: 2 }
    }

    /// Create a sync test connection mode with custom check distance
    pub fn sync_test_with_distance(check_distance: usize) -> Self {
        Self::SyncTest { check_distance }
    }

    /// Create a host connection mode with default port (7777)
    pub fn host() -> Self {
        Self::Host { port: 7777 }
    }

    /// Create a host connection mode with custom port
    pub fn host_on_port(port: u16) -> Self {
        Self::Host { port }
    }

    /// Create a join connection mode
    pub fn join(address: impl Into<String>) -> Self {
        Self::Join {
            address: address.into(),
        }
    }

    /// Create a P2P connection mode for local testing
    pub fn p2p(bind_port: u16, peer_port: u16, local_player: usize) -> Self {
        Self::P2P {
            bind_port,
            peer_port,
            local_player,
        }
    }

    /// Check if this mode requires network connectivity
    pub fn is_networked(&self) -> bool {
        matches!(self, Self::Host { .. } | Self::Join { .. } | Self::P2P { .. })
    }

    /// Check if this mode uses rollback
    pub fn uses_rollback(&self) -> bool {
        !matches!(self, Self::Local)
    }
}

/// Connection state for UI feedback
///
/// Used to display connection progress to the user.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionState {
    /// Not in a networked session
    #[default]
    Disconnected,

    /// Binding to local port
    Binding,

    /// Waiting for peer to connect (host mode)
    WaitingForPeer,

    /// Attempting to connect to peer (join mode)
    Connecting,

    /// Connection established, synchronizing with GGRS
    Synchronizing {
        /// Current synchronization progress (0-100)
        progress: u8,
    },

    /// Fully connected and ready to play
    Connected,

    /// Connection failed
    Failed,
}

impl ConnectionState {
    /// Check if we're in a connecting/waiting state
    pub fn is_pending(&self) -> bool {
        matches!(
            self,
            Self::Binding | Self::WaitingForPeer | Self::Connecting | Self::Synchronizing { .. }
        )
    }

    /// Check if we're fully connected
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    /// Check if connection failed
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> &'static str {
        match self {
            Self::Disconnected => "Disconnected",
            Self::Binding => "Binding to port...",
            Self::WaitingForPeer => "Waiting for player to connect...",
            Self::Connecting => "Connecting to host...",
            Self::Synchronizing { .. } => "Synchronizing...",
            Self::Connected => "Connected",
            Self::Failed => "Connection failed",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connection_mode_default() {
        let mode = ConnectionMode::default();
        assert!(matches!(mode, ConnectionMode::Local));
    }

    #[test]
    fn test_connection_mode_sync_test() {
        let mode = ConnectionMode::sync_test();
        assert!(matches!(mode, ConnectionMode::SyncTest { check_distance: 2 }));
    }

    #[test]
    fn test_connection_mode_is_networked() {
        assert!(!ConnectionMode::Local.is_networked());
        assert!(!ConnectionMode::sync_test().is_networked());
        assert!(ConnectionMode::host().is_networked());
        assert!(ConnectionMode::join("127.0.0.1:7777").is_networked());
        assert!(ConnectionMode::p2p(7777, 7778, 0).is_networked());
    }

    #[test]
    fn test_connection_mode_uses_rollback() {
        assert!(!ConnectionMode::Local.uses_rollback());
        assert!(ConnectionMode::sync_test().uses_rollback());
        assert!(ConnectionMode::host().uses_rollback());
    }

    #[test]
    fn test_connection_state_status() {
        assert!(ConnectionState::Connecting.is_pending());
        assert!(ConnectionState::Connected.is_connected());
        assert!(ConnectionState::Failed.is_failed());
    }

    #[test]
    fn test_connection_mode_sync_test_with_distance() {
        let mode = ConnectionMode::sync_test_with_distance(5);
        match mode {
            ConnectionMode::SyncTest { check_distance } => assert_eq!(check_distance, 5),
            _ => panic!("Expected SyncTest mode"),
        }
    }

    #[test]
    fn test_connection_mode_host_on_port() {
        let mode = ConnectionMode::host_on_port(8888);
        match mode {
            ConnectionMode::Host { port } => assert_eq!(port, 8888),
            _ => panic!("Expected Host mode"),
        }
    }

    #[test]
    fn test_connection_mode_host_default_port() {
        let mode = ConnectionMode::host();
        match mode {
            ConnectionMode::Host { port } => assert_eq!(port, 7777),
            _ => panic!("Expected Host mode"),
        }
    }

    #[test]
    fn test_connection_mode_join() {
        let mode = ConnectionMode::join("192.168.1.100:7777");
        match mode {
            ConnectionMode::Join { address } => assert_eq!(address, "192.168.1.100:7777"),
            _ => panic!("Expected Join mode"),
        }
    }

    #[test]
    fn test_connection_mode_p2p() {
        let mode = ConnectionMode::p2p(7777, 7778, 1);
        match mode {
            ConnectionMode::P2P {
                bind_port,
                peer_port,
                local_player,
            } => {
                assert_eq!(bind_port, 7777);
                assert_eq!(peer_port, 7778);
                assert_eq!(local_player, 1);
            }
            _ => panic!("Expected P2P mode"),
        }
    }

    #[test]
    fn test_connection_state_default() {
        let state = ConnectionState::default();
        assert!(matches!(state, ConnectionState::Disconnected));
    }

    #[test]
    fn test_connection_state_is_pending() {
        // All pending states
        assert!(ConnectionState::Binding.is_pending());
        assert!(ConnectionState::WaitingForPeer.is_pending());
        assert!(ConnectionState::Connecting.is_pending());
        assert!(ConnectionState::Synchronizing { progress: 50 }.is_pending());

        // Non-pending states
        assert!(!ConnectionState::Disconnected.is_pending());
        assert!(!ConnectionState::Connected.is_pending());
        assert!(!ConnectionState::Failed.is_pending());
    }

    #[test]
    fn test_connection_state_status_messages() {
        assert_eq!(ConnectionState::Disconnected.status_message(), "Disconnected");
        assert_eq!(ConnectionState::Binding.status_message(), "Binding to port...");
        assert_eq!(
            ConnectionState::WaitingForPeer.status_message(),
            "Waiting for player to connect..."
        );
        assert_eq!(
            ConnectionState::Connecting.status_message(),
            "Connecting to host..."
        );
        assert_eq!(
            ConnectionState::Synchronizing { progress: 75 }.status_message(),
            "Synchronizing..."
        );
        assert_eq!(ConnectionState::Connected.status_message(), "Connected");
        assert_eq!(ConnectionState::Failed.status_message(), "Connection failed");
    }
}
