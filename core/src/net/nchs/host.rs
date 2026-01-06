//! NCHS Host State Machine
//!
//! Manages the host side of NCHS handshake, including:
//! - Listening for incoming connections
//! - Validating join requests
//! - Managing lobby state
//! - Initiating game start and distributing session info

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use nethercore_shared::netplay::NetplayMetadata;

use super::messages::{
    JoinAccept, JoinReject, JoinRejectReason, JoinRequest, LobbyState, LobbyUpdate, NchsMessage,
    NetworkConfig, PlayerConnectionInfo, PlayerInfo, PlayerSlot, SessionStart,
};
use super::socket::NchsSocket;
use super::NchsError;

/// Host state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostState {
    /// Not started
    Idle,
    /// Listening for connections
    Listening,
    /// Players in lobby, waiting for ready
    Lobby,
    /// SessionStart sent, waiting for punch completion
    Starting,
    /// All peers connected, ready for GGRS
    Ready,
}

/// Connected player tracking
#[derive(Debug, Clone)]
#[allow(dead_code)] // handle field used for debugging
struct ConnectedPlayer {
    /// Player handle (0-3)
    handle: u8,
    /// Player info from JoinRequest
    info: PlayerInfo,
    /// Network address
    addr: SocketAddr,
    /// Whether player is ready
    ready: bool,
    /// Last message time (for timeout detection)
    last_seen: Instant,
}

/// Host state machine for NCHS protocol
pub struct HostStateMachine {
    /// Current state
    state: HostState,
    /// Socket for communication
    socket: NchsSocket,
    /// Our netplay metadata for validation
    netplay: NetplayMetadata,
    /// Host's player info
    host_info: PlayerInfo,
    /// Connected players (by handle)
    players: HashMap<u8, ConnectedPlayer>,
    /// Address to handle mapping
    addr_to_handle: HashMap<SocketAddr, u8>,
    /// Next available player handle
    next_handle: u8,
    /// Network configuration
    network_config: NetworkConfig,
    /// Random seed for session (generated on start)
    random_seed: Option<u64>,
    /// Session start sent time
    start_time: Option<Instant>,
    /// Public address for sharing with peers (real IP, not 0.0.0.0)
    public_addr: String,
}

/// Events emitted by the host state machine
#[derive(Debug, Clone)]
pub enum HostEvent {
    /// No events pending
    None,
    /// Now listening on port
    Listening { port: u16 },
    /// Player joined
    PlayerJoined { handle: u8, info: PlayerInfo },
    /// Player left
    PlayerLeft { handle: u8 },
    /// Player ready state changed
    PlayerReadyChanged { handle: u8, ready: bool },
    /// All players ready, can start
    AllReady,
    /// Session started, transitioning to GGRS
    Ready(SessionStart),
    /// Error occurred
    Error(NchsError),
}

impl HostStateMachine {
    /// Create a new host state machine
    ///
    /// # Arguments
    ///
    /// * `port` - Port to listen on
    /// * `netplay` - Netplay metadata for validation
    /// * `host_info` - Host's player info
    /// * `network_config` - Network configuration for GGRS
    pub fn new(
        port: u16,
        netplay: NetplayMetadata,
        host_info: PlayerInfo,
        network_config: NetworkConfig,
    ) -> Result<Self, NchsError> {
        let socket = NchsSocket::bind(&format!("0.0.0.0:{}", port))
            .map_err(|e| NchsError::BindFailed(e.to_string()))?;

        // Determine real IP address for sharing with peers
        // Prefer non-loopback address, fall back to localhost
        let real_ip = NchsSocket::get_local_ips()
            .into_iter()
            .find(|ip| ip != "127.0.0.1")
            .unwrap_or_else(|| "127.0.0.1".to_string());
        let public_addr = format!("{}:{}", real_ip, socket.port());

        tracing::info!(port = socket.port(), "NCHS Host listening");

        Ok(Self {
            state: HostState::Listening,
            socket,
            netplay,
            host_info,
            players: HashMap::new(),
            addr_to_handle: HashMap::new(),
            next_handle: 1, // Host is handle 0
            network_config,
            random_seed: None,
            start_time: None,
            public_addr,
        })
    }

    /// Get current state
    pub fn state(&self) -> HostState {
        self.state
    }

    /// Get the socket port
    pub fn port(&self) -> u16 {
        self.socket.port()
    }

    /// Get current lobby state
    pub fn lobby_state(&self) -> LobbyState {
        let mut slots = Vec::with_capacity(self.netplay.max_players as usize);

        // Add host as player 0
        slots.push(PlayerSlot {
            handle: 0,
            active: true,
            info: Some(self.host_info.clone()),
            ready: true, // Host is always ready
            addr: Some(self.public_addr.clone()),
        });

        // Add other players
        for handle in 1..self.netplay.max_players {
            if let Some(player) = self.players.get(&handle) {
                slots.push(PlayerSlot {
                    handle,
                    active: true,
                    info: Some(player.info.clone()),
                    ready: player.ready,
                    addr: Some(player.addr.to_string()),
                });
            } else {
                slots.push(PlayerSlot {
                    handle,
                    active: false,
                    info: None,
                    ready: false,
                    addr: None,
                });
            }
        }

        LobbyState {
            players: slots,
            max_players: self.netplay.max_players,
            host_handle: 0,
        }
    }

    /// Get number of connected players (including host)
    pub fn player_count(&self) -> u8 {
        1 + self.players.len() as u8
    }

    /// Check if all players are ready
    pub fn all_ready(&self) -> bool {
        self.players.values().all(|p| p.ready)
    }

    /// Check if lobby is full
    pub fn is_full(&self) -> bool {
        self.player_count() >= self.netplay.max_players
    }

    /// Poll for events
    pub fn poll(&mut self) -> HostEvent {
        // Host in Starting state should immediately transition to Ready.
        // Unlike guests who need to punch each other, the host doesn't
        // participate in hole punching - it just waits for GGRS connections.
        if self.state == HostState::Starting {
            self.state = HostState::Ready;
            if let Some(session_start) = self.session_start() {
                return HostEvent::Ready(session_start);
            }
        }

        // Only check for timeouts during Listening/Lobby states
        // After SessionStart, guests send GGRS packets, not NCHS pings
        if self.state == HostState::Listening || self.state == HostState::Lobby {
            let timed_out = self.check_timeouts(Duration::from_secs(5));
            if let Some(&handle) = timed_out.first() {
                return HostEvent::PlayerLeft { handle };
            }
        }

        // Receive messages
        while let Some((from, msg)) = self.socket.poll() {
            if let Some(event) = self.handle_message(from, msg) {
                return event;
            }
        }

        HostEvent::None
    }

    /// Handle an incoming message
    fn handle_message(&mut self, from: SocketAddr, msg: NchsMessage) -> Option<HostEvent> {
        match msg {
            NchsMessage::JoinRequest(req) => self.handle_join_request(from, req),
            NchsMessage::GuestReady(ready) => self.handle_guest_ready(from, ready.ready),
            NchsMessage::Ping => {
                // Respond with Pong
                let _ = self.socket.send_to(from, &NchsMessage::Pong);
                // Update last_seen if this is a known player
                if let Some(handle) = self.addr_to_handle.get(&from) {
                    if let Some(player) = self.players.get_mut(handle) {
                        player.last_seen = Instant::now();
                    }
                }
                None
            }
            NchsMessage::PunchAck(_) => {
                // Ignore punch acks from guests to host
                None
            }
            _ => {
                tracing::warn!(?msg, "Unexpected message from peer");
                None
            }
        }
    }

    /// Handle a join request
    fn handle_join_request(&mut self, from: SocketAddr, req: JoinRequest) -> Option<HostEvent> {
        // Check if already connected
        if self.addr_to_handle.contains_key(&from) {
            // Resend accept with existing handle
            if let Some(&handle) = self.addr_to_handle.get(&from) {
                let accept = JoinAccept {
                    player_handle: handle,
                    lobby: self.lobby_state(),
                };
                let _ = self.socket.send_to(from, &NchsMessage::JoinAccept(accept));
            }
            return None;
        }

        // Validate request
        if let Some(reject) = self.validate_join_request(&req) {
            let _ = self.socket.send_to(from, &NchsMessage::JoinReject(reject.clone()));
            return Some(HostEvent::Error(NchsError::ValidationFailed(
                format!("{:?}", reject.reason),
            )));
        }

        // Check if lobby is full
        if self.is_full() {
            let reject = JoinReject {
                reason: JoinRejectReason::LobbyFull,
                message: None,
            };
            let _ = self.socket.send_to(from, &NchsMessage::JoinReject(reject));
            return None;
        }

        // Assign handle and add player
        let handle = self.next_handle;
        self.next_handle += 1;

        let player = ConnectedPlayer {
            handle,
            info: req.player_info.clone(),
            addr: from,
            ready: false,
            last_seen: Instant::now(),
        };

        self.players.insert(handle, player);
        self.addr_to_handle.insert(from, handle);

        tracing::info!(player = handle, "Player joined");

        // Send accept
        let accept = JoinAccept {
            player_handle: handle,
            lobby: self.lobby_state(),
        };
        let _ = self.socket.send_to(from, &NchsMessage::JoinAccept(accept));

        // Broadcast lobby update to all other players
        self.broadcast_lobby_update();

        // Update state if we were just listening
        if self.state == HostState::Listening {
            self.state = HostState::Lobby;
        }

        Some(HostEvent::PlayerJoined {
            handle,
            info: req.player_info,
        })
    }

    /// Validate a join request
    fn validate_join_request(&self, req: &JoinRequest) -> Option<JoinReject> {
        // Check console type
        if req.console_type != self.netplay.console_type {
            return Some(JoinReject {
                reason: JoinRejectReason::ConsoleTypeMismatch,
                message: Some(format!(
                    "Expected {:?}, got {:?}",
                    self.netplay.console_type, req.console_type
                )),
            });
        }

        // Check ROM hash
        if req.rom_hash != self.netplay.rom_hash {
            return Some(JoinReject {
                reason: JoinRejectReason::RomHashMismatch,
                message: Some("Different game version".to_string()),
            });
        }

        // Check tick rate
        if req.tick_rate != self.netplay.tick_rate {
            return Some(JoinReject {
                reason: JoinRejectReason::TickRateMismatch,
                message: Some(format!(
                    "Expected {}Hz, got {}Hz",
                    self.netplay.tick_rate.as_hz(),
                    req.tick_rate.as_hz()
                )),
            });
        }

        // Check if game already started
        if self.state == HostState::Starting || self.state == HostState::Ready {
            return Some(JoinReject {
                reason: JoinRejectReason::GameInProgress,
                message: None,
            });
        }

        None
    }

    /// Handle guest ready state change
    fn handle_guest_ready(&mut self, from: SocketAddr, ready: bool) -> Option<HostEvent> {
        let handle = *self.addr_to_handle.get(&from)?;
        let player = self.players.get_mut(&handle)?;

        if player.ready != ready {
            player.ready = ready;
            player.last_seen = Instant::now();

            tracing::info!(player = handle, ready, "Player ready");

            // Broadcast lobby update
            self.broadcast_lobby_update();

            // Check if all ready
            if self.all_ready() && self.player_count() > 1 {
                return Some(HostEvent::AllReady);
            }

            return Some(HostEvent::PlayerReadyChanged { handle, ready });
        }

        None
    }

    /// Broadcast lobby update to all connected players
    fn broadcast_lobby_update(&self) {
        let update = LobbyUpdate {
            lobby: self.lobby_state(),
        };
        let msg = NchsMessage::LobbyUpdate(update);

        for player in self.players.values() {
            let _ = self.socket.send_to(player.addr, &msg);
        }
    }

    /// Start the game session
    ///
    /// Call this when all players are ready and the host wants to start.
    /// Returns the SessionStart that will be sent to all players.
    pub fn start(&mut self) -> Result<SessionStart, NchsError> {
        if !self.all_ready() {
            return Err(NchsError::ProtocolError("Not all players ready".to_string()));
        }

        if self.player_count() < 2 {
            return Err(NchsError::ProtocolError("Need at least 2 players".to_string()));
        }

        // Generate random seed
        let random_seed = rand::random::<u64>();
        self.random_seed = Some(random_seed);

        // Build player connection info
        let mut players = Vec::with_capacity(self.netplay.max_players as usize);

        // Add host
        players.push(PlayerConnectionInfo {
            handle: 0,
            active: true,
            info: self.host_info.clone(),
            addr: self.public_addr.clone(),
            ggrs_port: self.socket.port() + 1, // GGRS uses port + 1
        });

        // Add other players
        for handle in 1..self.netplay.max_players {
            if let Some(player) = self.players.get(&handle) {
                players.push(PlayerConnectionInfo {
                    handle,
                    active: true,
                    info: player.info.clone(),
                    addr: player.addr.to_string(),
                    ggrs_port: player.addr.port() + 1,
                });
            } else {
                players.push(PlayerConnectionInfo {
                    handle,
                    active: false,
                    info: PlayerInfo::default(),
                    addr: String::new(),
                    ggrs_port: 0,
                });
            }
        }

        let session_start = SessionStart {
            local_player_handle: 0, // Will be set per-process by library when serializing
            random_seed,
            start_frame: 0,
            tick_rate: self.netplay.tick_rate,
            players,
            player_count: self.player_count(),
            network_config: self.network_config.clone(),
            save_config: None, // TODO: Add save sync support
            extra_data: vec![],
        };

        // Send SessionStart to all guests
        let msg = NchsMessage::SessionStart(session_start.clone());
        for player in self.players.values() {
            let _ = self.socket.send_to(player.addr, &msg);
        }

        self.state = HostState::Starting;
        self.start_time = Some(Instant::now());

        tracing::info!(
            "Session started with {} players, seed: {:016x}",
            self.player_count(),
            random_seed
        );

        Ok(session_start)
    }

    /// Mark session as ready (after punch completion)
    pub fn mark_ready(&mut self) {
        self.state = HostState::Ready;
    }

    /// Remove a player by handle
    pub fn remove_player(&mut self, handle: u8) -> Option<PlayerInfo> {
        if let Some(player) = self.players.remove(&handle) {
            self.addr_to_handle.remove(&player.addr);
            self.broadcast_lobby_update();

            // If we're back to just the host, go back to Listening
            if self.players.is_empty() {
                self.state = HostState::Listening;
            }

            return Some(player.info);
        }
        None
    }

    /// Check for timed out players
    pub fn check_timeouts(&mut self, timeout: Duration) -> Vec<u8> {
        let now = Instant::now();
        let timed_out: Vec<u8> = self
            .players
            .iter()
            .filter(|(_, p)| now.duration_since(p.last_seen) > timeout)
            .map(|(h, _)| *h)
            .collect();

        for handle in &timed_out {
            tracing::warn!(player = handle, "Player timed out");
            self.remove_player(*handle);
        }

        timed_out
    }

    /// Get the socket for GGRS transition
    pub fn take_socket(self) -> NchsSocket {
        self.socket
    }

    /// Get session start info (only valid after start())
    pub fn session_start(&self) -> Option<SessionStart> {
        self.random_seed.map(|seed| SessionStart {
            local_player_handle: 0, // Will be set per-process by library when serializing
            random_seed: seed,
            start_frame: 0,
            tick_rate: self.netplay.tick_rate,
            players: self.build_player_connection_info(),
            player_count: self.player_count(),
            network_config: self.network_config.clone(),
            save_config: None,
            extra_data: vec![],
        })
    }

    /// Build player connection info list
    fn build_player_connection_info(&self) -> Vec<PlayerConnectionInfo> {
        let mut players = Vec::with_capacity(self.netplay.max_players as usize);

        // Add host
        players.push(PlayerConnectionInfo {
            handle: 0,
            active: true,
            info: self.host_info.clone(),
            addr: self.public_addr.clone(),
            ggrs_port: self.socket.port() + 1,
        });

        // Add other players
        for handle in 1..self.netplay.max_players {
            if let Some(player) = self.players.get(&handle) {
                players.push(PlayerConnectionInfo {
                    handle,
                    active: true,
                    info: player.info.clone(),
                    addr: player.addr.to_string(),
                    ggrs_port: player.addr.port() + 1,
                });
            } else {
                players.push(PlayerConnectionInfo {
                    handle,
                    active: false,
                    info: PlayerInfo::default(),
                    addr: String::new(),
                    ggrs_port: 0,
                });
            }
        }

        players
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nethercore_shared::console::{ConsoleType, TickRate};

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

    #[test]
    fn test_host_create() {
        let host = HostStateMachine::new(
            0, // Let OS assign port
            test_netplay(),
            test_player_info("Host"),
            NetworkConfig::default(),
        )
        .unwrap();

        assert_eq!(host.state(), HostState::Listening);
        assert!(host.port() > 0);
        assert_eq!(host.player_count(), 1); // Just the host
    }

    #[test]
    fn test_host_lobby_state() {
        let host = HostStateMachine::new(
            0,
            test_netplay(),
            test_player_info("Host"),
            NetworkConfig::default(),
        )
        .unwrap();

        let lobby = host.lobby_state();
        assert_eq!(lobby.players.len(), 4); // max_players slots
        assert!(lobby.players[0].active); // Host
        assert!(!lobby.players[1].active); // Empty slot
        assert_eq!(lobby.host_handle, 0);
    }

    #[test]
    fn test_host_all_ready_empty() {
        let host = HostStateMachine::new(
            0,
            test_netplay(),
            test_player_info("Host"),
            NetworkConfig::default(),
        )
        .unwrap();

        // With no other players, "all ready" is true (vacuously)
        assert!(host.all_ready());
    }

    #[test]
    fn test_host_is_full() {
        let mut netplay = test_netplay();
        netplay.max_players = 1; // Only host

        let host = HostStateMachine::new(
            0,
            netplay,
            test_player_info("Host"),
            NetworkConfig::default(),
        )
        .unwrap();

        assert!(host.is_full());
    }

    #[test]
    fn test_host_public_addr_not_zero() {
        let host = HostStateMachine::new(
            0,
            test_netplay(),
            test_player_info("Host"),
            NetworkConfig::default(),
        )
        .unwrap();

        // Public address should not start with 0.0.0.0
        assert!(
            !host.public_addr.starts_with("0.0.0.0"),
            "public_addr should not be 0.0.0.0, got: {}",
            host.public_addr
        );
    }

    #[test]
    fn test_host_lobby_state_has_real_ip() {
        let host = HostStateMachine::new(
            0,
            test_netplay(),
            test_player_info("Host"),
            NetworkConfig::default(),
        )
        .unwrap();

        let lobby = host.lobby_state();
        let host_slot = &lobby.players[0];

        // Host slot should have a real IP address, not 0.0.0.0
        assert!(host_slot.addr.is_some());
        let addr = host_slot.addr.as_ref().unwrap();
        assert!(
            !addr.starts_with("0.0.0.0"),
            "Host address in lobby should not be 0.0.0.0, got: {}",
            addr
        );
    }
}
