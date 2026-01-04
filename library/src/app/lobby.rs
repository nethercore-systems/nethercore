//! Lobby session state management
//!
//! Holds the NchsSession during lobby phase and tracks UI state.

use nethercore_core::library::LocalGame;
use nethercore_core::net::nchs::{
    JoinReject, JoinRejectReason, LobbyState, NchsError, NchsEvent, NchsRole, NchsSession,
    SessionStart,
};

/// Active lobby session state
pub struct LobbySession {
    /// The NCHS session being managed
    pub session: NchsSession,
    /// Game being played
    pub game: LocalGame,
    /// Current UI phase
    pub phase: LobbyPhase,
    /// Guest's local ready state (for UI checkbox)
    pub local_ready: bool,
    /// Last error message
    pub error: Option<String>,
    /// Cached local IPs for host display
    pub local_ips: Vec<String>,
}

/// Lobby UI phases
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LobbyPhase {
    /// Host: listening for connections
    Listening,
    /// Guest: connecting to host
    Connecting,
    /// Both: in lobby, showing players
    InLobby,
    /// Both: session starting, hole punching
    Starting,
    /// Both: ready to spawn player
    Ready,
    /// Failed with error
    Failed,
}

impl LobbySession {
    /// Create a new host lobby session
    pub fn new_host(session: NchsSession, game: LocalGame) -> Self {
        Self {
            session,
            game,
            phase: LobbyPhase::Listening,
            local_ready: false,
            error: None,
            local_ips: get_local_ips(),
        }
    }

    /// Create a new guest lobby session
    pub fn new_guest(session: NchsSession, game: LocalGame) -> Self {
        Self {
            session,
            game,
            phase: LobbyPhase::Connecting,
            local_ready: false,
            error: None,
            local_ips: Vec::new(),
        }
    }

    /// Get the session role (Host or Guest)
    pub fn role(&self) -> NchsRole {
        self.session.role()
    }

    /// Check if this is the host
    pub fn is_host(&self) -> bool {
        self.session.role() == NchsRole::Host
    }

    /// Get the current lobby state
    pub fn lobby(&self) -> Option<LobbyState> {
        self.session.lobby()
    }

    /// Get the port (for host address display)
    pub fn port(&self) -> u16 {
        self.session.port()
    }

    /// Check if all players are ready (host only)
    pub fn all_ready(&self) -> bool {
        self.session.all_ready()
    }

    /// Get player count
    pub fn player_count(&self) -> u8 {
        self.session.player_count()
    }

    /// Check if host can start the game
    pub fn can_start(&self) -> bool {
        self.is_host() && self.all_ready() && self.player_count() >= 2
    }

    /// Get the session config (only after Ready)
    pub fn session_config(&self) -> Option<&SessionStart> {
        self.session.session_config()
    }

    /// Poll the session and update phase
    ///
    /// Returns true if the UI should be refreshed
    pub fn poll(&mut self) -> bool {
        match self.session.poll() {
            NchsEvent::Pending => false,

            NchsEvent::Listening { port } => {
                tracing::info!("Lobby: Listening on port {}", port);
                self.phase = LobbyPhase::Listening;
                true
            }

            NchsEvent::LobbyUpdated(lobby) => {
                tracing::debug!("Lobby: Updated, {} players", lobby.players.len());
                self.phase = LobbyPhase::InLobby;
                true
            }

            NchsEvent::PlayerJoined { handle, ref info } => {
                tracing::info!("Lobby: Player {} joined ({})", handle, info.name);
                self.phase = LobbyPhase::InLobby;
                true
            }

            NchsEvent::PlayerLeft { handle } => {
                tracing::info!("Lobby: Player {} left", handle);
                true
            }

            NchsEvent::Ready(ref session_start) => {
                tracing::info!(
                    "Lobby: Ready! seed={} players={}",
                    session_start.random_seed,
                    session_start.player_count
                );
                self.phase = LobbyPhase::Ready;
                true
            }

            NchsEvent::Error(ref e) => {
                tracing::error!("Lobby: Error - {:?}", e);
                self.phase = LobbyPhase::Failed;
                self.error = Some(format_nchs_error(e));
                true
            }
        }
    }

    /// Set ready state (guest only)
    pub fn set_ready(&mut self, ready: bool) -> Result<(), NchsError> {
        self.local_ready = ready;
        self.session.set_ready(ready)
    }

    /// Start the game (host only)
    pub fn start(&mut self) -> Result<SessionStart, NchsError> {
        self.phase = LobbyPhase::Starting;
        self.session.start()
    }
}

/// Format an NCHS error for display to the user
pub fn format_nchs_error(error: &NchsError) -> String {
    match error {
        NchsError::BindFailed(e) => format!("Could not use port: {}", e),
        NchsError::Timeout => "Connection timed out. Check the address and try again.".into(),
        NchsError::Rejected(reject) => format_reject_reason(reject),
        NchsError::ValidationFailed(e) => format!("Validation failed: {}", e),
        NchsError::PunchFailed => "Failed to connect to other players.".into(),
        NchsError::NetworkError(e) => format!("Network error: {}", e),
        NchsError::ProtocolError(e) => format!("Protocol error: {}", e),
    }
}

/// Format a join rejection reason for display to the user
fn format_reject_reason(reject: &JoinReject) -> String {
    match reject.reason {
        JoinRejectReason::LobbyFull => "The lobby is full.".into(),
        JoinRejectReason::ConsoleTypeMismatch => "Console type mismatch.".into(),
        JoinRejectReason::RomHashMismatch => {
            "ROM version mismatch. Make sure you have the same game version.".into()
        }
        JoinRejectReason::TickRateMismatch => "Game tick rate mismatch.".into(),
        JoinRejectReason::GameInProgress => "Game has already started.".into(),
        JoinRejectReason::HostRejected => "Host rejected the connection.".into(),
        JoinRejectReason::VersionMismatch => "Protocol version mismatch. Update Nethercore.".into(),
        JoinRejectReason::Other => reject
            .message
            .clone()
            .unwrap_or_else(|| "Unknown error.".into()),
    }
}

/// Get local IP addresses for display to the user
fn get_local_ips() -> Vec<String> {
    let mut ips = Vec::new();

    #[cfg(not(target_os = "windows"))]
    {
        if let Ok(output) = std::process::Command::new("hostname").arg("-I").output() {
            if let Ok(stdout) = String::from_utf8(output.stdout) {
                for ip in stdout.split_whitespace() {
                    if !ip.starts_with("127.") && !ip.contains(':') {
                        ips.push(ip.to_string());
                    }
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("ipconfig").output()
            && let Ok(stdout) = String::from_utf8(output.stdout)
        {
            for line in stdout.lines() {
                if line.contains("IPv4")
                    && let Some(ip) = line.split(':').nth(1)
                {
                    let ip = ip.trim();
                    if !ip.starts_with("127.") {
                        ips.push(ip.to_string());
                    }
                }
            }
        }
    }

    if ips.is_empty() {
        ips.push("127.0.0.1".to_string());
    }

    ips
}
