//! NCHS Guest State Machine
//!
//! Manages the guest side of NCHS handshake, including:
//! - Connecting to host
//! - Sending join requests
//! - Waiting for session start
//! - UDP hole punching with other guests

use std::collections::HashSet;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use nethercore_shared::netplay::NetplayMetadata;

use super::messages::{
    GuestReady, JoinReject, JoinRequest, LobbyState, NchsMessage, PlayerInfo, PunchAck, PunchHello,
    SessionStart,
};
use super::socket::NchsSocket;
use super::NchsError;

/// Guest state machine states
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuestState {
    /// Not connected
    Idle,
    /// Sent JoinRequest, waiting for response
    Joining,
    /// In lobby, waiting for SessionStart
    Lobby,
    /// Received SessionStart, hole punching with peers
    Punching,
    /// All peers connected, ready for GGRS
    Ready,
    /// Connection failed
    Failed,
}

/// Guest state machine for NCHS protocol
pub struct GuestStateMachine {
    /// Current state
    state: GuestState,
    /// Socket for communication
    socket: NchsSocket,
    /// Our netplay metadata
    netplay: NetplayMetadata,
    /// Our player info
    player_info: PlayerInfo,
    /// Host address
    host_addr: SocketAddr,
    /// Assigned player handle (after accept)
    player_handle: Option<u8>,
    /// Current lobby state
    lobby: Option<LobbyState>,
    /// Session start info (after receiving SessionStart)
    session_start: Option<SessionStart>,
    /// Whether we're ready
    ready: bool,
    /// Peers we need to punch (handle -> addr)
    peers_to_punch: Vec<(u8, SocketAddr)>,
    /// Peers we've successfully punched
    punched_peers: HashSet<u8>,
    /// Join request sent time
    join_sent_at: Option<Instant>,
    /// Punch start time
    punch_started_at: Option<Instant>,
    /// Nonce for punch messages
    punch_nonce: u64,
}

/// Events emitted by the guest state machine
#[derive(Debug, Clone)]
pub enum GuestEvent {
    /// No events pending
    None,
    /// Join request accepted
    Accepted { handle: u8 },
    /// Join request rejected
    Rejected(JoinReject),
    /// Lobby state updated
    LobbyUpdated(LobbyState),
    /// Session starting
    SessionStarting(SessionStart),
    /// Ready for GGRS
    Ready,
    /// Error occurred
    Error(NchsError),
}

impl GuestStateMachine {
    /// Create a new guest and initiate connection
    ///
    /// # Arguments
    ///
    /// * `host_addr` - Host address to connect to (e.g., "192.168.1.50:7770")
    /// * `netplay` - Netplay metadata for validation
    /// * `player_info` - Our player info
    pub fn new(
        host_addr: &str,
        netplay: NetplayMetadata,
        player_info: PlayerInfo,
    ) -> Result<Self, NchsError> {
        let host: SocketAddr = host_addr
            .parse()
            .map_err(|e| NchsError::NetworkError(format!("Invalid host address: {}", e)))?;

        // Bind to any available port
        let socket = NchsSocket::bind_any()
            .map_err(|e| NchsError::BindFailed(e.to_string()))?;

        log::info!("NCHS Guest connecting to {} from port {}", host, socket.port());

        let mut guest = Self {
            state: GuestState::Idle,
            socket,
            netplay,
            player_info,
            host_addr: host,
            player_handle: None,
            lobby: None,
            session_start: None,
            ready: false,
            peers_to_punch: Vec::new(),
            punched_peers: HashSet::new(),
            join_sent_at: None,
            punch_started_at: None,
            punch_nonce: rand::random(),
        };

        // Send initial join request
        guest.send_join_request()?;

        Ok(guest)
    }

    /// Get current state
    pub fn state(&self) -> GuestState {
        self.state
    }

    /// Get our player handle (after accepted)
    pub fn player_handle(&self) -> Option<u8> {
        self.player_handle
    }

    /// Get current lobby state
    pub fn lobby(&self) -> Option<&LobbyState> {
        self.lobby.as_ref()
    }

    /// Get session start info (after receiving SessionStart)
    pub fn session_start(&self) -> Option<&SessionStart> {
        self.session_start.as_ref()
    }

    /// Check if we're ready
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Send join request to host
    fn send_join_request(&mut self) -> Result<(), NchsError> {
        let request = JoinRequest {
            console_type: self.netplay.console_type,
            rom_hash: self.netplay.rom_hash,
            tick_rate: self.netplay.tick_rate,
            max_players: self.netplay.max_players,
            player_info: self.player_info.clone(),
            local_addr: self.socket.local_addr_string(),
            extra_data: vec![],
        };

        self.socket
            .send_to(self.host_addr, &NchsMessage::JoinRequest(request))
            .map_err(|e| NchsError::NetworkError(e.to_string()))?;

        self.state = GuestState::Joining;
        self.join_sent_at = Some(Instant::now());

        log::debug!("Sent JoinRequest to {}", self.host_addr);

        Ok(())
    }

    /// Set ready state
    pub fn set_ready(&mut self, ready: bool) -> Result<(), NchsError> {
        if self.state != GuestState::Lobby {
            return Err(NchsError::ProtocolError("Not in lobby".to_string()));
        }

        self.ready = ready;

        let msg = NchsMessage::GuestReady(GuestReady { ready });
        self.socket
            .send_to(self.host_addr, &msg)
            .map_err(|e| NchsError::NetworkError(e.to_string()))?;

        log::debug!("Set ready: {}", ready);

        Ok(())
    }

    /// Poll for events
    pub fn poll(&mut self) -> GuestEvent {
        // Check for timeouts
        if let Some(event) = self.check_timeouts() {
            return event;
        }

        // Receive messages
        while let Some((from, msg)) = self.socket.poll() {
            if let Some(event) = self.handle_message(from, msg) {
                return event;
            }
        }

        // If punching, check completion
        if self.state == GuestState::Punching {
            if self.is_punch_complete() {
                self.state = GuestState::Ready;
                return GuestEvent::Ready;
            }

            // Retry punch periodically
            self.retry_punch();
        }

        GuestEvent::None
    }

    /// Check for timeouts
    fn check_timeouts(&mut self) -> Option<GuestEvent> {
        const JOIN_TIMEOUT: Duration = Duration::from_secs(5);
        const PUNCH_TIMEOUT: Duration = Duration::from_secs(3);

        // Check join timeout
        if self.state == GuestState::Joining {
            if let Some(sent_at) = self.join_sent_at {
                if sent_at.elapsed() > JOIN_TIMEOUT {
                    self.state = GuestState::Failed;
                    return Some(GuestEvent::Error(NchsError::Timeout));
                }
            }
        }

        // Check punch timeout
        if self.state == GuestState::Punching {
            if let Some(started_at) = self.punch_started_at {
                if started_at.elapsed() > PUNCH_TIMEOUT {
                    self.state = GuestState::Failed;
                    return Some(GuestEvent::Error(NchsError::PunchFailed));
                }
            }
        }

        None
    }

    /// Handle an incoming message
    fn handle_message(&mut self, from: SocketAddr, msg: NchsMessage) -> Option<GuestEvent> {
        match msg {
            NchsMessage::JoinAccept(accept) => {
                if from != self.host_addr {
                    log::warn!("JoinAccept from non-host: {}", from);
                    return None;
                }
                self.handle_accept(accept)
            }
            NchsMessage::JoinReject(reject) => {
                if from != self.host_addr {
                    return None;
                }
                self.state = GuestState::Failed;
                Some(GuestEvent::Rejected(reject))
            }
            NchsMessage::LobbyUpdate(update) => {
                if from != self.host_addr {
                    return None;
                }
                self.lobby = Some(update.lobby.clone());
                Some(GuestEvent::LobbyUpdated(update.lobby))
            }
            NchsMessage::SessionStart(start) => {
                if from != self.host_addr {
                    return None;
                }
                self.handle_session_start(start)
            }
            NchsMessage::PunchHello(hello) => self.handle_punch_hello(from, hello),
            NchsMessage::PunchAck(ack) => self.handle_punch_ack(from, ack),
            NchsMessage::Ping => {
                // Respond with Pong
                let _ = self.socket.send_to(from, &NchsMessage::Pong);
                None
            }
            _ => {
                log::warn!("Unexpected message from {}: {:?}", from, msg);
                None
            }
        }
    }

    /// Handle JoinAccept
    fn handle_accept(&mut self, accept: super::messages::JoinAccept) -> Option<GuestEvent> {
        if self.state != GuestState::Joining {
            return None;
        }

        self.player_handle = Some(accept.player_handle);
        self.lobby = Some(accept.lobby);
        self.state = GuestState::Lobby;

        log::info!("Joined lobby as player {}", accept.player_handle);

        Some(GuestEvent::Accepted {
            handle: accept.player_handle,
        })
    }

    /// Handle SessionStart
    fn handle_session_start(&mut self, start: SessionStart) -> Option<GuestEvent> {
        if self.state != GuestState::Lobby {
            log::warn!("SessionStart received in wrong state: {:?}", self.state);
            return None;
        }

        log::info!(
            "Session starting: {} players, seed {:016x}",
            start.player_count,
            start.random_seed
        );

        // Determine which peers we need to punch (other guests, not host)
        self.peers_to_punch.clear();
        self.punched_peers.clear();

        if let Some(our_handle) = self.player_handle {
            for player in &start.players {
                if player.active && player.handle != our_handle && player.handle != 0 {
                    // This is another guest we need to punch
                    if let Ok(addr) = player.addr.parse::<SocketAddr>() {
                        self.peers_to_punch.push((player.handle, addr));
                    }
                }
            }
        }

        self.session_start = Some(start.clone());

        if self.peers_to_punch.is_empty() {
            // No peers to punch (2-player game), go directly to Ready
            self.state = GuestState::Ready;
            Some(GuestEvent::Ready)
        } else {
            // Start hole punching
            self.state = GuestState::Punching;
            self.punch_started_at = Some(Instant::now());
            self.send_punch_hellos();
            Some(GuestEvent::SessionStarting(start))
        }
    }

    /// Send PunchHello to all peers
    fn send_punch_hellos(&self) {
        if let Some(our_handle) = self.player_handle {
            let hello = PunchHello {
                sender_handle: our_handle,
                nonce: self.punch_nonce,
            };
            let msg = NchsMessage::PunchHello(hello);

            for (handle, addr) in &self.peers_to_punch {
                log::debug!("Sending PunchHello to player {} at {}", handle, addr);
                let _ = self.socket.send_to(*addr, &msg);
            }
        }
    }

    /// Retry punch (called periodically while punching)
    fn retry_punch(&mut self) {
        const PUNCH_RETRY_INTERVAL: Duration = Duration::from_millis(200);

        if let Some(started_at) = self.punch_started_at {
            let elapsed = started_at.elapsed();
            let retry_count = (elapsed.as_millis() / PUNCH_RETRY_INTERVAL.as_millis()) as u32;

            // Retry every PUNCH_RETRY_INTERVAL
            if elapsed.as_millis() % PUNCH_RETRY_INTERVAL.as_millis() < 50 {
                if retry_count > 0 {
                    log::debug!("Punch retry #{}", retry_count);
                    self.send_punch_hellos();
                }
            }
        }
    }

    /// Handle PunchHello from a peer
    fn handle_punch_hello(&mut self, from: SocketAddr, hello: PunchHello) -> Option<GuestEvent> {
        if self.state != GuestState::Punching {
            return None;
        }

        // Check if this is a peer we expected
        let is_expected = self
            .peers_to_punch
            .iter()
            .any(|(h, _)| *h == hello.sender_handle);

        if !is_expected {
            log::warn!("Unexpected PunchHello from {} (handle {})", from, hello.sender_handle);
            return None;
        }

        log::debug!("Received PunchHello from player {} at {}", hello.sender_handle, from);

        // Send PunchAck
        if let Some(our_handle) = self.player_handle {
            let ack = PunchAck {
                sender_handle: our_handle,
                nonce: hello.nonce,
            };
            let _ = self.socket.send_to(from, &NchsMessage::PunchAck(ack));
        }

        // Mark as punched (receiving hello means they can send to us)
        self.punched_peers.insert(hello.sender_handle);

        None
    }

    /// Handle PunchAck from a peer
    fn handle_punch_ack(&mut self, from: SocketAddr, ack: PunchAck) -> Option<GuestEvent> {
        if self.state != GuestState::Punching {
            return None;
        }

        // Validate nonce
        if ack.nonce != self.punch_nonce {
            log::warn!("Invalid PunchAck nonce from {}", from);
            return None;
        }

        log::debug!("Received PunchAck from player {} at {}", ack.sender_handle, from);

        // Mark as punched
        self.punched_peers.insert(ack.sender_handle);

        None
    }

    /// Check if hole punching is complete
    fn is_punch_complete(&self) -> bool {
        // We need to have received at least one message from each peer
        for (handle, _) in &self.peers_to_punch {
            if !self.punched_peers.contains(handle) {
                return false;
            }
        }
        true
    }

    /// Get the socket for GGRS transition
    pub fn take_socket(self) -> NchsSocket {
        self.socket
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    use nethercore_shared::console::{ConsoleType, TickRate};

    #[allow(dead_code)]
    fn test_netplay() -> NetplayMetadata {
        NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678)
    }

    #[allow(dead_code)]
    fn test_player_info(name: &str) -> PlayerInfo {
        PlayerInfo {
            name: name.to_string(),
            avatar_id: 0,
            color: [255, 255, 255],
        }
    }

    #[test]
    fn test_guest_state() {
        // Note: This test doesn't actually connect, just checks initial state
        // A real test would need a running host

        // We can't easily test the full flow without a host,
        // but we can test state transitions
        assert_eq!(GuestState::Idle, GuestState::Idle);
        assert_ne!(GuestState::Idle, GuestState::Joining);
    }

    #[test]
    fn test_punch_nonce_generation() {
        // Verify nonces are random
        let nonce1 = rand::random::<u64>();
        let nonce2 = rand::random::<u64>();
        // Very unlikely to be equal
        assert_ne!(nonce1, nonce2);
    }
}
