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
pub mod socket;
pub mod validation;

// Re-export state machine types
pub use guest::{GuestEvent, GuestState, GuestStateMachine};
pub use host::{HostEvent, HostState, HostStateMachine};
pub use validation::{is_netplay_compatible, validate_join_request};

// Re-export message types
pub use messages::{
    GuestReady, JoinAccept, JoinReject, JoinRejectReason, JoinRequest, LobbyState, LobbyUpdate,
    NchsDecodeError, NchsMessage, NetworkConfig, PlayerConnectionInfo, PlayerInfo, PlayerSlot,
    PunchAck, PunchHello, SaveConfig, SaveMode, SessionStart, NCHS_HEADER_SIZE, NCHS_MAGIC,
    NCHS_VERSION,
};

// Re-export socket types
pub use socket::{NchsSocket, NchsSocketError, DEFAULT_NCHS_PORT};

use nethercore_shared::netplay::NetplayMetadata;

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

/// Internal state machine wrapper
enum SessionInner {
    Host(HostStateMachine),
    Guest(GuestStateMachine),
}

/// NCHS session - unified wrapper for host and guest state machines
pub struct NchsSession {
    /// Internal state machine (host or guest)
    inner: SessionInner,
    /// Configuration
    config: NchsConfig,
    /// Cached session start info (when ready)
    session_start: Option<SessionStart>,
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
    LobbyUpdated(LobbyState),
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

impl NchsSession {
    /// Create a new host session
    ///
    /// The host listens for incoming connections and manages the lobby.
    pub fn host(port: u16, config: NchsConfig) -> Result<Self, NchsError> {
        let host_machine = HostStateMachine::new(
            port,
            config.netplay.clone(),
            config.player_info.clone(),
            config.network_config.clone(),
        )?;

        Ok(Self {
            inner: SessionInner::Host(host_machine),
            config,
            session_start: None,
        })
    }

    /// Create a new guest session and connect to host
    ///
    /// The guest connects to the host and waits for lobby updates.
    pub fn join(host_addr: &str, config: NchsConfig) -> Result<Self, NchsError> {
        let guest_machine = GuestStateMachine::new(
            host_addr,
            config.netplay.clone(),
            config.player_info.clone(),
        )?;

        Ok(Self {
            inner: SessionInner::Guest(guest_machine),
            config,
            session_start: None,
        })
    }

    /// Poll for events (non-blocking)
    pub fn poll(&mut self) -> NchsEvent {
        match &mut self.inner {
            SessionInner::Host(host) => {
                match host.poll() {
                    HostEvent::None => NchsEvent::Pending,
                    HostEvent::Listening { port } => NchsEvent::Listening { port },
                    HostEvent::PlayerJoined { handle, info } => {
                        NchsEvent::PlayerJoined { handle, info }
                    }
                    HostEvent::PlayerLeft { handle } => NchsEvent::PlayerLeft { handle },
                    HostEvent::PlayerReadyChanged { .. } => {
                        // Emit lobby update when ready state changes
                        NchsEvent::LobbyUpdated(host.lobby_state())
                    }
                    HostEvent::AllReady => {
                        // Just emit lobby update, host needs to call start()
                        NchsEvent::LobbyUpdated(host.lobby_state())
                    }
                    HostEvent::Ready(session_start) => {
                        self.session_start = Some(session_start.clone());
                        NchsEvent::Ready(session_start)
                    }
                    HostEvent::Error(e) => NchsEvent::Error(e),
                }
            }
            SessionInner::Guest(guest) => {
                match guest.poll() {
                    GuestEvent::None => NchsEvent::Pending,
                    GuestEvent::Accepted { handle } => {
                        // Guest got accepted, emit the lobby state
                        if let Some(lobby) = guest.lobby() {
                            NchsEvent::LobbyUpdated(lobby.clone())
                        } else {
                            NchsEvent::PlayerJoined {
                                handle,
                                info: self.config.player_info.clone(),
                            }
                        }
                    }
                    GuestEvent::Rejected(reject) => {
                        NchsEvent::Error(NchsError::Rejected(reject))
                    }
                    GuestEvent::LobbyUpdated(lobby) => NchsEvent::LobbyUpdated(lobby),
                    GuestEvent::SessionStarting(session_start) => {
                        // Still punching, but session is starting
                        self.session_start = Some(session_start.clone());
                        NchsEvent::LobbyUpdated(guest.lobby().cloned().unwrap_or_else(|| LobbyState {
                            players: vec![],
                            max_players: self.config.netplay.max_players,
                            host_handle: 0,
                        }))
                    }
                    GuestEvent::Ready => {
                        if let Some(session_start) = self.session_start.clone() {
                            NchsEvent::Ready(session_start)
                        } else if let Some(ss) = guest.session_start() {
                            self.session_start = Some(ss.clone());
                            NchsEvent::Ready(ss.clone())
                        } else {
                            NchsEvent::Pending
                        }
                    }
                    GuestEvent::Error(e) => NchsEvent::Error(e),
                }
            }
        }
    }

    /// Host: Start the game session
    ///
    /// Only valid when all players are ready.
    pub fn start(&mut self) -> Result<SessionStart, NchsError> {
        match &mut self.inner {
            SessionInner::Host(host) => {
                let session_start = host.start()?;
                self.session_start = Some(session_start.clone());
                Ok(session_start)
            }
            SessionInner::Guest(_) => {
                Err(NchsError::ProtocolError("Only host can start".to_string()))
            }
        }
    }

    /// Guest: Set ready state
    pub fn set_ready(&mut self, ready: bool) -> Result<(), NchsError> {
        match &mut self.inner {
            SessionInner::Guest(guest) => guest.set_ready(ready),
            SessionInner::Host(_) => {
                Err(NchsError::ProtocolError("Only guest can set ready".to_string()))
            }
        }
    }

    /// Get current lobby state
    pub fn lobby(&self) -> Option<LobbyState> {
        match &self.inner {
            SessionInner::Host(host) => Some(host.lobby_state()),
            SessionInner::Guest(guest) => guest.lobby().cloned(),
        }
    }

    /// Get session start config (only after Ready)
    pub fn session_config(&self) -> Option<&SessionStart> {
        self.session_start.as_ref()
    }

    /// Get local player handle
    pub fn local_handle(&self) -> Option<u8> {
        match &self.inner {
            SessionInner::Host(_) => Some(0), // Host is always player 0
            SessionInner::Guest(guest) => guest.player_handle(),
        }
    }

    /// Get current state
    pub fn state(&self) -> NchsState {
        match &self.inner {
            SessionInner::Host(host) => match host.state() {
                HostState::Idle => NchsState::Idle,
                HostState::Listening => NchsState::Connecting,
                HostState::Lobby => NchsState::Lobby,
                HostState::Starting => NchsState::Punching,
                HostState::Ready => NchsState::Ready,
            },
            SessionInner::Guest(guest) => match guest.state() {
                GuestState::Idle => NchsState::Idle,
                GuestState::Joining => NchsState::Connecting,
                GuestState::Lobby => NchsState::Lobby,
                GuestState::Punching => NchsState::Punching,
                GuestState::Ready => NchsState::Ready,
                GuestState::Failed => NchsState::Failed,
            },
        }
    }

    /// Get role
    pub fn role(&self) -> NchsRole {
        match &self.inner {
            SessionInner::Host(_) => NchsRole::Host,
            SessionInner::Guest(_) => NchsRole::Guest,
        }
    }

    /// Check if session is ready for GGRS transition
    pub fn is_ready(&self) -> bool {
        self.state() == NchsState::Ready
    }

    /// Get the socket port
    pub fn port(&self) -> u16 {
        match &self.inner {
            SessionInner::Host(host) => host.port(),
            SessionInner::Guest(_) => 0, // Guest doesn't expose port directly
        }
    }

    /// Check if all players are ready (host only)
    pub fn all_ready(&self) -> bool {
        match &self.inner {
            SessionInner::Host(host) => host.all_ready(),
            SessionInner::Guest(_) => false,
        }
    }

    /// Get number of connected players
    pub fn player_count(&self) -> u8 {
        match &self.inner {
            SessionInner::Host(host) => host.player_count(),
            SessionInner::Guest(guest) => {
                guest.lobby().map(|l| l.players.iter().filter(|p| p.active).count() as u8).unwrap_or(0)
            }
        }
    }

    /// Take the socket for GGRS transition (consumes self)
    pub fn take_socket(self) -> NchsSocket {
        match self.inner {
            SessionInner::Host(host) => host.take_socket(),
            SessionInner::Guest(guest) => guest.take_socket(),
        }
    }

    /// Host: Mark session as ready (after punch completion)
    pub fn mark_ready(&mut self) {
        if let SessionInner::Host(host) = &mut self.inner {
            host.mark_ready();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_shared::console::{ConsoleType, TickRate};
    use std::thread;
    use std::time::Duration;

    fn test_netplay() -> NetplayMetadata {
        NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678)
    }

    fn test_player_info(name: &str) -> PlayerInfo {
        PlayerInfo {
            name: name.to_string(),
            avatar_id: 0,
            color: [255, 255, 255],
        }
    }

    fn test_config(name: &str) -> NchsConfig {
        NchsConfig {
            netplay: test_netplay(),
            player_info: test_player_info(name),
            network_config: NetworkConfig::default(),
            save_config: None,
        }
    }

    #[test]
    fn test_host_session_create() {
        let config = test_config("Host");
        let session = NchsSession::host(0, config).unwrap();

        assert_eq!(session.role(), NchsRole::Host);
        assert_eq!(session.local_handle(), Some(0));
        assert!(session.port() > 0);
    }

    #[test]
    fn test_guest_session_create() {
        // First create a host so we have a valid port
        let host_config = test_config("Host");
        let host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Now create a guest connecting to that host
        let guest_config = test_config("Guest");
        let guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        assert_eq!(guest.role(), NchsRole::Guest);
        assert_eq!(guest.state(), NchsState::Connecting);
    }

    #[test]
    fn test_host_guest_handshake() {
        // Create host
        let host_config = test_config("Host");
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Create guest connecting to host
        let guest_config = test_config("Guest");
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        // Poll both until guest is accepted or timeout
        let mut guest_accepted = false;
        for _ in 0..100 {
            // Poll host
            match host.poll() {
                NchsEvent::PlayerJoined { handle, .. } => {
                    log::info!("Host: Player {} joined", handle);
                }
                _ => {}
            }

            // Poll guest
            match guest.poll() {
                NchsEvent::LobbyUpdated(lobby) => {
                    log::info!("Guest: Lobby updated, {} players", lobby.players.len());
                    guest_accepted = true;
                    break;
                }
                NchsEvent::PlayerJoined { handle, .. } => {
                    log::info!("Guest: Accepted as player {}", handle);
                    guest_accepted = true;
                    break;
                }
                NchsEvent::Error(e) => {
                    panic!("Guest error: {:?}", e);
                }
                _ => {}
            }

            thread::sleep(Duration::from_millis(10));
        }

        assert!(guest_accepted, "Guest should have been accepted");
        assert!(guest.local_handle().is_some(), "Guest should have a handle");
        assert_eq!(host.player_count(), 2, "Should have 2 players");
    }

    #[test]
    fn test_host_guest_ready_and_start() {
        // Create host
        let host_config = test_config("Host");
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Create guest
        let guest_config = test_config("Guest");
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        // Wait for guest to join
        let mut joined = false;
        for _ in 0..100 {
            host.poll();
            match guest.poll() {
                NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => {
                    joined = true;
                    break;
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(joined, "Guest should have joined");

        // Guest sets ready
        guest.set_ready(true).unwrap();

        // Wait for host to see guest ready
        let mut all_ready = false;
        for _ in 0..100 {
            match host.poll() {
                NchsEvent::LobbyUpdated(lobby) => {
                    let guests_ready = lobby.players.iter()
                        .filter(|p| p.active && p.handle != 0)
                        .all(|p| p.ready);
                    if guests_ready {
                        all_ready = true;
                        break;
                    }
                }
                _ => {}
            }
            guest.poll(); // Keep guest alive
            thread::sleep(Duration::from_millis(10));
        }
        assert!(all_ready, "All players should be ready");

        // Host starts the session
        let session_start = host.start().expect("Host should be able to start");
        assert!(session_start.random_seed != 0, "Should have random seed");
        assert_eq!(session_start.player_count, 2, "Should have 2 players");

        // Guest should receive session start
        let mut guest_ready = false;
        for _ in 0..100 {
            match guest.poll() {
                NchsEvent::Ready(ss) => {
                    assert_eq!(ss.random_seed, session_start.random_seed, "Seeds should match");
                    guest_ready = true;
                    break;
                }
                NchsEvent::LobbyUpdated(_) => {
                    // Session starting, keep polling
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(guest_ready, "Guest should receive Ready event");
    }

    #[test]
    fn test_host_cannot_set_ready() {
        let config = test_config("Host");
        let mut host = NchsSession::host(0, config).unwrap();

        let result = host.set_ready(true);
        assert!(result.is_err());
    }

    #[test]
    fn test_guest_cannot_start() {
        let host_config = test_config("Host");
        let host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        let guest_config = test_config("Guest");
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        let result = guest.start();
        assert!(result.is_err());
    }

    #[test]
    fn test_rom_hash_mismatch_rejected() {
        // Host with one hash
        let host_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xAAAAAAAA),
            player_info: test_player_info("Host"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Guest with different hash
        let guest_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xBBBBBBBB),
            player_info: test_player_info("Guest"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        // Poll until rejection or timeout
        let mut rejected = false;
        let mut reject_reason = None;
        for _ in 0..100 {
            host.poll();
            match guest.poll() {
                NchsEvent::Error(NchsError::Rejected(reject)) => {
                    rejected = true;
                    reject_reason = Some(reject.reason);
                    break;
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(rejected, "Guest should be rejected for ROM hash mismatch");
        assert_eq!(reject_reason, Some(JoinRejectReason::RomHashMismatch));
    }

    #[test]
    fn test_console_type_mismatch_rejected() {
        // Host with ZX
        let host_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678),
            player_info: test_player_info("Host"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Guest with Chroma
        let guest_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::Chroma, TickRate::Fixed60, 4, 0x12345678),
            player_info: test_player_info("Guest"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        // Poll until rejection or timeout
        let mut rejected = false;
        let mut reject_reason = None;
        for _ in 0..100 {
            host.poll();
            match guest.poll() {
                NchsEvent::Error(NchsError::Rejected(reject)) => {
                    rejected = true;
                    reject_reason = Some(reject.reason);
                    break;
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(rejected, "Guest should be rejected for console type mismatch");
        assert_eq!(reject_reason, Some(JoinRejectReason::ConsoleTypeMismatch));
    }

    #[test]
    fn test_tick_rate_mismatch_rejected() {
        // Host with 60Hz
        let host_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678),
            player_info: test_player_info("Host"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Guest with 120Hz
        let guest_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed120, 4, 0x12345678),
            player_info: test_player_info("Guest"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        // Poll until rejection or timeout
        let mut rejected = false;
        let mut reject_reason = None;
        for _ in 0..100 {
            host.poll();
            match guest.poll() {
                NchsEvent::Error(NchsError::Rejected(reject)) => {
                    rejected = true;
                    reject_reason = Some(reject.reason);
                    break;
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(rejected, "Guest should be rejected for tick rate mismatch");
        assert_eq!(reject_reason, Some(JoinRejectReason::TickRateMismatch));
    }

    #[test]
    fn test_lobby_full_rejected() {
        // Host with max 2 players
        let host_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 2, 0x12345678),
            player_info: test_player_info("Host"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // First guest joins successfully
        let guest1_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 2, 0x12345678),
            player_info: test_player_info("Guest1"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut guest1 = NchsSession::join(&format!("127.0.0.1:{}", port), guest1_config).unwrap();

        // Wait for guest1 to join
        let mut guest1_joined = false;
        for _ in 0..100 {
            host.poll();
            match guest1.poll() {
                NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => {
                    guest1_joined = true;
                    break;
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }
        assert!(guest1_joined, "Guest1 should join");

        // Second guest should be rejected (lobby full: host + guest1 = 2)
        let guest2_config = NchsConfig {
            netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 2, 0x12345678),
            player_info: test_player_info("Guest2"),
            network_config: NetworkConfig::default(),
            save_config: None,
        };
        let mut guest2 = NchsSession::join(&format!("127.0.0.1:{}", port), guest2_config).unwrap();

        let mut rejected = false;
        let mut reject_reason = None;
        for _ in 0..100 {
            host.poll();
            guest1.poll(); // Keep guest1 alive
            match guest2.poll() {
                NchsEvent::Error(NchsError::Rejected(reject)) => {
                    rejected = true;
                    reject_reason = Some(reject.reason);
                    break;
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(rejected, "Guest2 should be rejected because lobby is full");
        assert_eq!(reject_reason, Some(JoinRejectReason::LobbyFull));
    }

    #[test]
    fn test_join_while_game_in_progress_rejected() {
        // Create host and first guest
        let host_config = test_config("Host");
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        let guest1_config = test_config("Guest1");
        let mut guest1 = NchsSession::join(&format!("127.0.0.1:{}", port), guest1_config).unwrap();

        // Wait for guest1 to join
        for _ in 0..100 {
            host.poll();
            match guest1.poll() {
                NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => break,
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Guest1 sets ready
        guest1.set_ready(true).unwrap();

        // Wait for host to see ready
        for _ in 0..100 {
            if host.all_ready() && host.player_count() > 1 {
                break;
            }
            host.poll();
            guest1.poll();
            thread::sleep(Duration::from_millis(10));
        }

        // Host starts the game
        host.start().expect("Should be able to start");

        // Now try to join with a new guest
        let guest2_config = test_config("Guest2");
        let mut guest2 = NchsSession::join(&format!("127.0.0.1:{}", port), guest2_config).unwrap();

        let mut rejected = false;
        let mut reject_reason = None;
        for _ in 0..100 {
            host.poll();
            guest1.poll();
            match guest2.poll() {
                NchsEvent::Error(NchsError::Rejected(reject)) => {
                    rejected = true;
                    reject_reason = Some(reject.reason);
                    break;
                }
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(rejected, "Guest2 should be rejected because game is in progress");
        assert_eq!(reject_reason, Some(JoinRejectReason::GameInProgress));
    }

    #[test]
    fn test_session_start_has_real_ip() {
        // Create host
        let host_config = test_config("Host");
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Create guest
        let guest_config = test_config("Guest");
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        // Wait for guest to join
        for _ in 0..100 {
            host.poll();
            match guest.poll() {
                NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => break,
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Guest sets ready
        guest.set_ready(true).unwrap();

        // Wait for host to see guest ready
        for _ in 0..100 {
            if host.all_ready() && host.player_count() > 1 {
                break;
            }
            host.poll();
            guest.poll();
            thread::sleep(Duration::from_millis(10));
        }

        // Host starts the session
        let session_start = host.start().expect("Host should be able to start");

        // Verify host's address in SessionStart is not 0.0.0.0
        let host_player = &session_start.players[0];
        assert!(host_player.active, "Host should be active");
        assert!(
            !host_player.addr.starts_with("0.0.0.0"),
            "Host address in SessionStart should not be 0.0.0.0, got: {}",
            host_player.addr
        );
        // Should be localhost for local test
        assert!(
            host_player.addr.starts_with("127.0.0.1") || !host_player.addr.is_empty(),
            "Host address should be a valid IP, got: {}",
            host_player.addr
        );
    }

    #[test]
    fn test_lobby_state_has_real_host_ip() {
        // Create host
        let host_config = test_config("Host");
        let host = NchsSession::host(0, host_config).unwrap();

        // Get lobby state
        let lobby = host.lobby().expect("Host should have lobby state");

        // Verify host's address is not 0.0.0.0
        let host_slot = &lobby.players[0];
        assert!(host_slot.active, "Host slot should be active");
        let addr = host_slot.addr.as_ref().expect("Host should have an address");
        assert!(
            !addr.starts_with("0.0.0.0"),
            "Host address in lobby should not be 0.0.0.0, got: {}",
            addr
        );
    }

    #[test]
    fn test_host_emits_ready_after_start() {
        // This test verifies that the host emits NchsEvent::Ready after start() is called.
        // Bug: Previously, the host never emitted Ready, causing the library to never
        // spawn the player process for the host.

        // Create host
        let host_config = test_config("Host");
        let mut host = NchsSession::host(0, host_config).unwrap();
        let port = host.port();

        // Create guest
        let guest_config = test_config("Guest");
        let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

        // Wait for guest to join
        for _ in 0..100 {
            host.poll();
            match guest.poll() {
                NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => break,
                _ => {}
            }
            thread::sleep(Duration::from_millis(10));
        }

        // Guest sets ready
        guest.set_ready(true).unwrap();

        // Wait for host to see guest ready
        for _ in 0..100 {
            if host.all_ready() && host.player_count() > 1 {
                break;
            }
            host.poll();
            guest.poll();
            thread::sleep(Duration::from_millis(10));
        }

        assert!(host.all_ready(), "All players should be ready before start");
        assert!(host.player_count() >= 2, "Should have at least 2 players");

        // Host starts the session
        let _session_start = host.start().expect("Host should be able to start");

        // CRITICAL: Host should emit Ready event on the next poll
        // This is the bug - previously the host never emitted Ready
        let mut host_ready = false;
        for _ in 0..10 {
            match host.poll() {
                NchsEvent::Ready(ss) => {
                    // Verify the session start info is correct
                    assert!(ss.random_seed != 0, "Should have random seed");
                    assert_eq!(ss.player_count, 2, "Should have 2 players");
                    host_ready = true;
                    break;
                }
                NchsEvent::Pending => {
                    // Give it a few more tries
                }
                other => {
                    panic!("Unexpected event from host after start: {:?}", other);
                }
            }
            thread::sleep(Duration::from_millis(10));
        }

        assert!(
            host_ready,
            "Host should emit NchsEvent::Ready after start() - this is required for the library to spawn the player process"
        );

        // Also verify state is Ready
        assert_eq!(
            host.state(),
            NchsState::Ready,
            "Host state should be Ready after emitting Ready event"
        );
    }
}
