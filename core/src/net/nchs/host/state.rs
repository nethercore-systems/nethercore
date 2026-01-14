//! Host state machine core types and management

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use nethercore_shared::netplay::NetplayMetadata;

use crate::net::nchs::messages::{
    NetworkConfig, PlayerInfo, PlayerSlot, LobbyState, SessionStart,
};
use crate::net::nchs::socket::NchsSocket;
use crate::net::nchs::NchsError;

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
pub struct ConnectedPlayer {
    /// Player handle (0-3)
    pub(super) handle: u8,
    /// Player info from JoinRequest
    pub(super) info: PlayerInfo,
    /// Network address
    pub(super) addr: SocketAddr,
    /// Whether player is ready
    pub(super) ready: bool,
    /// Last message time (for timeout detection)
    pub(super) last_seen: Instant,
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

/// Host state machine for NCHS protocol
pub struct HostStateMachine {
    /// Current state
    pub(super) state: HostState,
    /// Socket for communication
    pub(super) socket: NchsSocket,
    /// Our netplay metadata for validation
    pub(super) netplay: NetplayMetadata,
    /// Host's player info
    pub(super) host_info: PlayerInfo,
    /// Connected players (by handle)
    pub(super) players: HashMap<u8, ConnectedPlayer>,
    /// Address to handle mapping
    pub(super) addr_to_handle: HashMap<SocketAddr, u8>,
    /// Next available player handle
    pub(super) next_handle: u8,
    /// Network configuration
    pub(super) network_config: NetworkConfig,
    /// Random seed for session (generated on start)
    pub(super) random_seed: Option<u64>,
    /// Session start sent time
    pub(super) start_time: Option<Instant>,
    /// Public address for sharing with peers (real IP, not 0.0.0.0)
    pub(super) public_addr: String,
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
        if socket.port() == u16::MAX {
            return Err(NchsError::BindFailed(
                "Port 65535 not supported (GGRS uses port+1)".to_string(),
            ));
        }

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

    /// Mark session as ready (after punch completion)
    pub fn mark_ready(&mut self) {
        self.state = HostState::Ready;
    }

    /// Broadcast lobby update to all connected players
    pub(super) fn broadcast_lobby_update(&self) {
        use crate::net::nchs::messages::{LobbyUpdate, NchsMessage};

        let update = LobbyUpdate {
            lobby: self.lobby_state(),
        };
        let msg = NchsMessage::LobbyUpdate(update);

        for player in self.players.values() {
            if let Err(e) = self.socket.send_to(player.addr, &msg) {
                tracing::warn!(
                    error = %e,
                    player = player.handle,
                    "Failed to send LobbyUpdate"
                );
            }
        }
    }
}
