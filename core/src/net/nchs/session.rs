//! NCHS session implementation

use crate::net::nchs::{
    guest::{GuestEvent, GuestState, GuestStateMachine},
    host::{HostEvent, HostState, HostStateMachine},
    types::{NchsConfig, NchsError, NchsEvent, NchsRole, NchsState},
    LobbyState, NchsSocket, SessionStart,
};

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

impl NchsSession {
    /// Create a new host session
    ///
    /// The host listens for incoming connections and manages the lobby.
    pub fn host(port: u16, config: NchsConfig) -> Result<Self, NchsError> {
        let host_machine = HostStateMachine::new(
            port,
            config.netplay,
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
        let guest_machine =
            GuestStateMachine::new(host_addr, config.netplay, config.player_info.clone())?;

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
                    GuestEvent::Rejected(reject) => NchsEvent::Error(NchsError::Rejected(reject)),
                    GuestEvent::LobbyUpdated(lobby) => NchsEvent::LobbyUpdated(lobby),
                    GuestEvent::SessionStarting(session_start) => {
                        // Still punching, but session is starting
                        self.session_start = Some(session_start.clone());
                        NchsEvent::LobbyUpdated(guest.lobby().cloned().unwrap_or_else(|| {
                            LobbyState {
                                players: vec![],
                                max_players: self.config.netplay.max_players,
                                host_handle: 0,
                            }
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
            SessionInner::Host(_) => Err(NchsError::ProtocolError(
                "Only guest can set ready".to_string(),
            )),
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
            SessionInner::Guest(guest) => guest
                .lobby()
                .map(|l| l.players.iter().filter(|p| p.active).count() as u8)
                .unwrap_or(0),
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
