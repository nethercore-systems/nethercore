//! GGRS session management
//!
//! Provides the RollbackSession wrapper for GGRS local, sync-test, and P2P sessions.

use std::time::Duration;

use bytemuck::{Pod, Zeroable};
use ggrs::{
    GgrsError, GgrsEvent, GgrsRequest, InputStatus, NonBlockingSocket, P2PSession, PlayerType,
    SessionBuilder, SessionState, SyncTestSession,
};

use crate::console::ConsoleInput;
use crate::wasm::GameInstance;

use super::config::{EmberwareConfig, SessionConfig};
use super::player::{PlayerSessionConfig, MAX_PLAYERS};
use super::state::{
    GameStateSnapshot, LoadStateError, RollbackStateManager, SaveStateError,
};

// ============================================================================
// Session Types
// ============================================================================

/// Session type for GGRS
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SessionType {
    /// Local session (no rollback, single machine)
    Local,
    /// Sync test session (local with rollback for testing determinism)
    SyncTest,
    /// P2P session with rollback netcode
    P2P,
}

// ============================================================================
// Network Input Wrapper
// ============================================================================

/// Wrapper type to implement Pod + Zeroable for generic inputs
///
/// GGRS requires inputs to be POD (Plain Old Data) for network serialization.
/// This wrapper ensures the generic input type satisfies those requirements.
#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct NetworkInput<I: ConsoleInput> {
    /// The console-specific input data
    pub input: I,
}

impl<I: ConsoleInput> NetworkInput<I> {
    /// Create a new network input wrapper
    pub fn new(input: I) -> Self {
        Self { input }
    }
}

// SAFETY: I is required to be Pod + Zeroable by ConsoleInput trait bounds.
// NetworkInput is a #[repr(transparent)] wrapper, so it has the same layout as I.
unsafe impl<I: ConsoleInput> Pod for NetworkInput<I> {}
unsafe impl<I: ConsoleInput> Zeroable for NetworkInput<I> {}

// ============================================================================
// GGRS Session Wrapper
// ============================================================================

/// Inner session types for different modes
///
/// Note: P2P variant is boxed to reduce overall enum size, as P2PSession is
/// significantly larger than other variants (~440 bytes vs ~228 bytes).
enum SessionInner<I: ConsoleInput> {
    /// Local session - no GGRS, just direct execution
    Local {
        num_players: usize,
        current_frame: i32,
    },
    /// Sync test session for determinism testing
    SyncTest {
        session: SyncTestSession<EmberwareConfig<I>>,
        current_frame: i32,
    },
    /// P2P session with rollback (boxed to reduce enum size)
    P2P(Box<P2PSession<EmberwareConfig<I>>>),
}

/// Frame advantage threshold for warning events
const FRAME_ADVANTAGE_WARNING_THRESHOLD: i32 = 4;

/// Rollback session manager
///
/// Wraps GGRS session types and provides a unified interface for
/// local, sync-test, and P2P sessions. Handles state management
/// and input processing.
pub struct RollbackSession<I: ConsoleInput> {
    inner: SessionInner<I>,
    session_type: SessionType,
    config: SessionConfig,
    /// Player session configuration (local/remote player assignments)
    player_config: PlayerSessionConfig,
    state_manager: RollbackStateManager,
    /// Whether we're currently in rollback mode (used to mute audio)
    rolling_back: bool,
    /// Local player handles for this session
    local_players: Vec<usize>,
    /// Network statistics per remote player
    network_stats: Vec<PlayerNetworkStats>,
    /// Number of rollback frames this session
    total_rollback_frames: u64,
    /// Last frame advantage (for warning detection)
    last_frame_advantage: i32,
    /// Whether a desync has been detected
    desync_detected: bool,
}

impl<I: ConsoleInput> RollbackSession<I> {
    /// Create a new local session (no rollback)
    ///
    /// Local sessions run without GGRS - updates execute immediately
    /// without any rollback support. Useful for single player games
    /// or local multiplayer on the same machine.
    ///
    /// All players are assumed to be local.
    pub fn new_local(num_players: usize) -> Self {
        let player_config = PlayerSessionConfig::all_local(num_players as u32);
        Self::new_local_with_config(player_config)
    }

    /// Create a new local session with explicit player configuration
    ///
    /// This allows specifying which players are local vs remote.
    /// For local sessions, all players should typically be local,
    /// but this method allows flexibility for testing or special scenarios.
    pub fn new_local_with_config(player_config: PlayerSessionConfig) -> Self {
        let num_players = player_config.num_players() as usize;
        let local_players = player_config.local_player_indices();

        Self {
            inner: SessionInner::Local {
                num_players,
                current_frame: 0,
            },
            session_type: SessionType::Local,
            config: SessionConfig::local(num_players),
            player_config,
            state_manager: RollbackStateManager::new(),
            rolling_back: false,
            local_players,
            network_stats: Vec::new(), // No network stats for local
            total_rollback_frames: 0,
            last_frame_advantage: 0,
            desync_detected: false,
        }
    }

    /// Create a new sync test session (for testing determinism)
    ///
    /// Sync test sessions simulate rollback every frame to verify
    /// the game state is deterministic. Use this during development
    /// to catch non-determinism bugs.
    pub fn new_sync_test(config: SessionConfig) -> Result<Self, GgrsError> {
        let player_config = PlayerSessionConfig::all_local(config.num_players as u32);
        Self::new_sync_test_with_config(config, player_config)
    }

    /// Create a new sync test session with explicit player configuration
    pub fn new_sync_test_with_config(
        config: SessionConfig,
        player_config: PlayerSessionConfig,
    ) -> Result<Self, GgrsError> {
        let session = SessionBuilder::<EmberwareConfig<I>>::new()
            .with_num_players(config.num_players)
            .with_max_prediction_window(config.max_prediction_frames)?
            .with_input_delay(config.input_delay)
            .with_check_distance(2)
            .start_synctest_session()?;

        let local_players = player_config.local_player_indices();

        Ok(Self {
            inner: SessionInner::SyncTest {
                session,
                current_frame: 0,
            },
            session_type: SessionType::SyncTest,
            config,
            player_config,
            state_manager: RollbackStateManager::new(),
            rolling_back: false,
            local_players,
            network_stats: Vec::new(), // No network stats for sync test
            total_rollback_frames: 0,
            last_frame_advantage: 0,
            desync_detected: false,
        })
    }

    /// Create a new P2P session with the given socket
    ///
    /// P2P sessions use GGRS for rollback netcode. Players must be
    /// added via the session builder before starting.
    ///
    /// The player configuration is derived from the `players` parameter:
    /// - Local players are those with `PlayerType::Local`
    /// - Remote players are those with `PlayerType::Remote`
    pub fn new_p2p<S>(
        config: SessionConfig,
        socket: S,
        players: Vec<(usize, PlayerType<String>)>,
    ) -> Result<Self, GgrsError>
    where
        S: NonBlockingSocket<String> + 'static,
    {
        // Build player config from the players list
        let mut local_mask = 0u32;
        for (handle, player_type) in &players {
            if matches!(player_type, PlayerType::Local) && *handle < MAX_PLAYERS {
                local_mask |= 1u32 << handle;
            }
        }
        let player_config = PlayerSessionConfig::new(config.num_players as u32, local_mask);

        Self::new_p2p_with_config(config, player_config, socket, players)
    }

    /// Create a new P2P session with explicit player configuration
    ///
    /// This allows full control over the player session configuration.
    /// The `players` parameter still specifies the GGRS player types.
    pub fn new_p2p_with_config<S>(
        config: SessionConfig,
        player_config: PlayerSessionConfig,
        socket: S,
        players: Vec<(usize, PlayerType<String>)>,
    ) -> Result<Self, GgrsError>
    where
        S: NonBlockingSocket<String> + 'static,
    {
        let mut builder = SessionBuilder::<EmberwareConfig<I>>::new()
            .with_num_players(config.num_players)
            .with_max_prediction_window(config.max_prediction_frames)?
            .with_input_delay(config.input_delay)
            .with_fps(config.fps)?
            .with_disconnect_timeout(Duration::from_millis(config.disconnect_timeout))
            .with_disconnect_notify_delay(Duration::from_millis(config.disconnect_notify_start));

        let mut local_players = Vec::new();

        for (handle, player_type) in players {
            if matches!(player_type, PlayerType::Local) {
                local_players.push(handle);
            }
            builder = builder.add_player(player_type, handle)?;
        }

        let session = builder.start_p2p_session(socket)?;

        // Initialize network stats for all players
        let network_stats: Vec<PlayerNetworkStats> = (0..config.num_players)
            .map(|_| PlayerNetworkStats {
                connected: true,
                ..Default::default()
            })
            .collect();

        Ok(Self {
            inner: SessionInner::P2P(Box::new(session)),
            session_type: SessionType::P2P,
            config,
            player_config,
            state_manager: RollbackStateManager::new(),
            rolling_back: false,
            local_players,
            network_stats,
            total_rollback_frames: 0,
            last_frame_advantage: 0,
            desync_detected: false,
        })
    }

    /// Get the session type
    pub fn session_type(&self) -> SessionType {
        self.session_type
    }

    /// Get the session configuration
    pub fn config(&self) -> &SessionConfig {
        &self.config
    }

    /// Get the player session configuration
    ///
    /// This provides information about which players are local vs remote.
    pub fn player_config(&self) -> &PlayerSessionConfig {
        &self.player_config
    }

    /// Get mutable access to the state manager
    pub fn state_manager_mut(&mut self) -> &mut RollbackStateManager {
        &mut self.state_manager
    }

    /// Check if currently rolling back
    pub fn is_rolling_back(&self) -> bool {
        self.rolling_back
    }

    /// Get local player handles
    pub fn local_players(&self) -> &[usize] {
        &self.local_players
    }

    /// Get current frame number
    pub fn current_frame(&self) -> i32 {
        match &self.inner {
            SessionInner::Local { current_frame, .. } => *current_frame,
            SessionInner::SyncTest { current_frame, .. } => *current_frame,
            SessionInner::P2P(session) => session.current_frame(),
        }
    }

    /// Get the current session state (for P2P sessions)
    pub fn session_state(&self) -> Option<SessionState> {
        match &self.inner {
            SessionInner::P2P(session) => Some(session.current_state()),
            _ => None,
        }
    }

    /// Add local input for a player
    ///
    /// For Local sessions, input is stored immediately.
    /// For GGRS sessions, input is passed to GGRS for synchronization.
    pub fn add_local_input(&mut self, player_handle: usize, input: I) -> Result<(), GgrsError> {
        match &mut self.inner {
            SessionInner::Local { .. } => {
                // Local sessions don't need GGRS input handling
                // Input is set directly on GameInstance
                Ok(())
            }
            SessionInner::SyncTest { session, .. } => session.add_local_input(player_handle, input),
            SessionInner::P2P(session) => session.add_local_input(player_handle, input),
        }
    }

    /// Poll remote clients (P2P only)
    ///
    /// Must be called regularly to receive network messages.
    pub fn poll_remote_clients(&mut self) {
        if let SessionInner::P2P(session) = &mut self.inner {
            session.poll_remote_clients();
        }
    }

    /// Advance the frame and get GGRS requests
    ///
    /// Returns a list of requests that must be handled by the game:
    /// - SaveGameState: Save current state for rollback
    /// - LoadGameState: Restore to a previous state
    /// - AdvanceFrame: Execute one tick with the given inputs
    ///
    /// For Local sessions, this returns a simple AdvanceFrame request
    /// with default inputs for all players.
    pub fn advance_frame(&mut self) -> Result<Vec<GgrsRequest<EmberwareConfig<I>>>, GgrsError> {
        match &mut self.inner {
            SessionInner::Local {
                num_players,
                current_frame,
            } => {
                // Local sessions just advance immediately with default inputs
                *current_frame += 1;
                let inputs: Vec<(I, InputStatus)> = (0..*num_players)
                    .map(|_| (I::default(), InputStatus::Confirmed))
                    .collect();
                Ok(vec![GgrsRequest::AdvanceFrame { inputs }])
            }
            SessionInner::SyncTest {
                session,
                current_frame,
            } => {
                let requests = session.advance_frame()?;
                // Track frame count - increment for each AdvanceFrame request
                for req in &requests {
                    if matches!(req, GgrsRequest::AdvanceFrame { .. }) {
                        *current_frame += 1;
                    }
                }
                Ok(requests)
            }
            SessionInner::P2P(session) => session.advance_frame(),
        }
    }

    /// Drain events from the session (P2P only)
    pub fn events(&mut self) -> Vec<GgrsEvent<EmberwareConfig<I>>> {
        match &mut self.inner {
            SessionInner::P2P(session) => session.events().collect(),
            _ => Vec::new(),
        }
    }

    /// Get network stats for a player (P2P only)
    pub fn network_stats(&self, player_handle: usize) -> Option<ggrs::NetworkStats> {
        match &self.inner {
            SessionInner::P2P(session) => session.network_stats(player_handle).ok(),
            _ => None,
        }
    }

    /// Get frames ahead (P2P only)
    pub fn frames_ahead(&self) -> i32 {
        match &self.inner {
            SessionInner::P2P(session) => session.frames_ahead(),
            _ => 0,
        }
    }

    /// Get player network statistics
    pub fn player_stats(&self, player_handle: usize) -> Option<&PlayerNetworkStats> {
        self.network_stats.get(player_handle)
    }

    /// Get all player network statistics
    pub fn all_player_stats(&self) -> &[PlayerNetworkStats] {
        &self.network_stats
    }

    /// Get total rollback frames this session
    pub fn total_rollback_frames(&self) -> u64 {
        self.total_rollback_frames
    }

    /// Check if a desync has been detected
    pub fn has_desync(&self) -> bool {
        self.desync_detected
    }

    /// Process and handle GGRS events
    ///
    /// Converts raw GGRS events to application-level `SessionEvent`s,
    /// updates internal network statistics, and logs relevant information.
    ///
    /// Returns a list of events for the application to respond to.
    /// Critical events like desync should trigger session termination.
    pub fn handle_events(&mut self) -> Vec<SessionEvent> {
        let raw_events = self.events();
        let mut session_events = Vec::new();

        for event in raw_events {
            match event {
                GgrsEvent::Synchronizing {
                    addr: _,
                    total,
                    count,
                } => {
                    log::debug!("Synchronizing: {}/{}", count, total);
                }
                GgrsEvent::Synchronized { addr: _ } => {
                    // Find the player handle for this address
                    // For now, we emit a generic synchronized event
                    log::info!("Peer synchronized");
                    // We don't have a direct mapping from address to player handle
                    // in the current design, so we use a placeholder
                    session_events.push(SessionEvent::Synchronized { player_handle: 0 });
                }
                GgrsEvent::Disconnected { addr: _ } => {
                    log::warn!("Peer disconnected");
                    // Mark all remote players as disconnected (conservative approach)
                    for (i, stats) in self.network_stats.iter_mut().enumerate() {
                        if !self.local_players.contains(&i) {
                            stats.connected = false;
                            stats.assess_quality();
                        }
                    }
                    session_events.push(SessionEvent::Disconnected { player_handle: 0 });
                }
                GgrsEvent::NetworkInterrupted {
                    addr: _,
                    disconnect_timeout,
                } => {
                    // disconnect_timeout is u128 (milliseconds)
                    let timeout_ms = disconnect_timeout as u64;
                    log::warn!("Network interrupted, disconnect in {}ms", timeout_ms);
                    session_events.push(SessionEvent::NetworkInterrupted {
                        player_handle: 0,
                        disconnect_timeout_ms: timeout_ms,
                    });
                }
                GgrsEvent::NetworkResumed { addr: _ } => {
                    log::info!("Network resumed");
                    // Mark remote players as connected again
                    for (i, stats) in self.network_stats.iter_mut().enumerate() {
                        if !self.local_players.contains(&i) {
                            stats.connected = true;
                            stats.assess_quality();
                        }
                    }
                    session_events.push(SessionEvent::NetworkResumed { player_handle: 0 });
                }
                GgrsEvent::WaitRecommendation { skip_frames } => {
                    log::debug!("Wait recommendation: skip {} frames", skip_frames);
                    session_events.push(SessionEvent::TimeSync {
                        frames_to_skip: skip_frames as usize,
                    });
                }
                GgrsEvent::DesyncDetected {
                    frame,
                    local_checksum,
                    remote_checksum,
                    addr: _,
                } => {
                    log::error!(
                        "DESYNC at frame {}: local={:#x}, remote={:#x}",
                        frame,
                        local_checksum,
                        remote_checksum
                    );
                    self.desync_detected = true;
                    session_events.push(SessionEvent::Desync {
                        frame,
                        local_checksum: local_checksum as u64,
                        remote_checksum: remote_checksum as u64,
                    });
                }
            }
        }

        // Update network statistics from GGRS
        self.update_network_stats();

        // Check frame advantage and emit warning if needed
        let frames_ahead = self.frames_ahead();
        if frames_ahead >= FRAME_ADVANTAGE_WARNING_THRESHOLD
            && self.last_frame_advantage < FRAME_ADVANTAGE_WARNING_THRESHOLD
        {
            log::debug!("Frame advantage warning: {} frames ahead", frames_ahead);
            session_events.push(SessionEvent::FrameAdvantageWarning { frames_ahead });
        }
        self.last_frame_advantage = frames_ahead;

        session_events
    }

    /// Update network statistics from GGRS
    fn update_network_stats(&mut self) {
        // First, collect all GGRS stats (to avoid borrow issues)
        let ggrs_stats: Vec<Option<ggrs::NetworkStats>> = match &self.inner {
            SessionInner::P2P(session) => (0..self.network_stats.len())
                .map(|player_handle| session.network_stats(player_handle).ok())
                .collect(),
            _ => return,
        };

        // Then update our stats
        for (player_handle, stats) in self.network_stats.iter_mut().enumerate() {
            // Skip local players
            if self.local_players.contains(&player_handle) {
                continue;
            }

            // Get GGRS network stats for this player
            if let Some(ggrs_stat) = ggrs_stats.get(player_handle).and_then(|s| s.as_ref()) {
                stats.ping_ms = ggrs_stat.ping as u32;
                stats.local_frames_ahead = ggrs_stat.local_frames_behind;
                stats.remote_frames_ahead = ggrs_stat.remote_frames_behind;
                stats.assess_quality();
            }
        }
    }

    /// Handle all GGRS requests for a frame
    ///
    /// Processes SaveGameState, LoadGameState, and AdvanceFrame requests.
    /// Returns the inputs for each AdvanceFrame request so the caller
    /// can update the game.
    ///
    /// During rollback (LoadGameState followed by AdvanceFrame), audio should
    /// be muted. Check `is_rolling_back()` before playing sounds.
    pub fn handle_requests(
        &mut self,
        game: &mut GameInstance,
        requests: Vec<GgrsRequest<EmberwareConfig<I>>>,
    ) -> Result<Vec<Vec<(I, InputStatus)>>, SessionError> {
        let mut advance_inputs = Vec::new();
        let mut rollback_frames_this_call = 0u32;

        for request in requests {
            match request {
                GgrsRequest::SaveGameState { cell, frame } => {
                    let snapshot = self
                        .state_manager
                        .save_state(game, frame)
                        .map_err(|e| SessionError::SaveState(e.to_string()))?;
                    let checksum = snapshot.checksum as u128;
                    cell.save(frame, Some(snapshot), Some(checksum));
                }
                GgrsRequest::LoadGameState { cell, frame: _ } => {
                    self.rolling_back = true;
                    if let Some(snapshot) = cell.load() {
                        self.state_manager
                            .load_state(game, &snapshot)
                            .map_err(|e| SessionError::LoadState(e.to_string()))?;
                    }
                }
                GgrsRequest::AdvanceFrame { inputs } => {
                    // Track rollback frames
                    if self.rolling_back {
                        rollback_frames_this_call += 1;
                    }
                    self.rolling_back = false;
                    advance_inputs.push(inputs);
                }
            }
        }

        // Update rollback frame counter
        self.total_rollback_frames += rollback_frames_this_call as u64;

        // Update rollback stats for all players
        if rollback_frames_this_call > 0 {
            for stats in &mut self.network_stats {
                stats.rollback_frames = stats.rollback_frames.saturating_add(rollback_frames_this_call);
            }
        }

        Ok(advance_inputs)
    }

    /// Save game state (convenience wrapper)
    pub fn save_game_state(
        &mut self,
        game: &mut GameInstance,
        frame: i32,
    ) -> Result<GameStateSnapshot, SaveStateError> {
        self.state_manager.save_state(game, frame)
    }

    /// Load game state (convenience wrapper)
    pub fn load_game_state(
        &mut self,
        game: &mut GameInstance,
        snapshot: &GameStateSnapshot,
    ) -> Result<(), LoadStateError> {
        self.state_manager.load_state(game, snapshot)
    }
}

// ============================================================================
// Session Events (Application-Level)
// ============================================================================

/// High-level session events for the application layer
///
/// These are translated from raw GGRS events into actionable events
/// that the game/UI can respond to.
#[derive(Debug, Clone)]
pub enum SessionEvent {
    /// Connection synchronized with a peer
    Synchronized {
        /// Player handle that synchronized
        player_handle: usize,
    },
    /// A peer disconnected
    Disconnected {
        /// Player handle that disconnected
        player_handle: usize,
    },
    /// Desync detected between clients
    ///
    /// This is a critical error - game state has diverged and cannot be recovered.
    /// The session should be terminated.
    Desync {
        /// Frame where desync was detected
        frame: i32,
        /// Local checksum
        local_checksum: u64,
        /// Remote checksum
        remote_checksum: u64,
    },
    /// Network interrupted with a peer
    NetworkInterrupted {
        /// Player handle with network issues
        player_handle: usize,
        /// How long the connection has been interrupted (ms)
        disconnect_timeout_ms: u64,
    },
    /// Network resumed with a peer
    NetworkResumed {
        /// Player handle whose connection resumed
        player_handle: usize,
    },
    /// Frame advantage warning - local client is too far ahead
    ///
    /// This indicates potential network issues. Consider showing
    /// a "waiting for opponent" message if this persists.
    FrameAdvantageWarning {
        /// How many frames ahead of the slowest peer
        frames_ahead: i32,
    },
    /// Timesync event (internal GGRS timing adjustment)
    TimeSync {
        /// Frames to skip for synchronization
        frames_to_skip: usize,
    },
    /// Waiting for remote players (not enough input yet)
    WaitingForPlayers,
}

/// Connection quality assessment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConnectionQuality {
    /// Excellent connection (< 50ms RTT, < 2 frames ahead)
    #[default]
    Excellent,
    /// Good connection (< 100ms RTT, < 4 frames ahead)
    Good,
    /// Fair connection (< 150ms RTT, < 6 frames ahead)
    Fair,
    /// Poor connection (>= 150ms RTT or >= 6 frames ahead)
    Poor,
    /// Connection interrupted
    Disconnected,
}

/// Network statistics for a player
#[derive(Debug, Clone, Default)]
pub struct PlayerNetworkStats {
    /// Round-trip time in milliseconds
    pub ping_ms: u32,
    /// Packet loss percentage (0-100)
    pub packet_loss: u8,
    /// Local frames ahead of this player
    pub local_frames_ahead: i32,
    /// Remote frames ahead of local
    pub remote_frames_ahead: i32,
    /// Number of rollback frames in last second
    pub rollback_frames: u32,
    /// Connection quality assessment
    pub quality: ConnectionQuality,
    /// Whether this player is currently connected
    pub connected: bool,
}

impl PlayerNetworkStats {
    /// Update quality assessment based on current stats
    pub fn assess_quality(&mut self) {
        if !self.connected {
            self.quality = ConnectionQuality::Disconnected;
        } else if self.ping_ms < 50 && self.local_frames_ahead.abs() < 2 {
            self.quality = ConnectionQuality::Excellent;
        } else if self.ping_ms < 100 && self.local_frames_ahead.abs() < 4 {
            self.quality = ConnectionQuality::Good;
        } else if self.ping_ms < 150 && self.local_frames_ahead.abs() < 6 {
            self.quality = ConnectionQuality::Fair;
        } else {
            self.quality = ConnectionQuality::Poor;
        }
    }
}

/// Session errors
#[derive(Debug, Clone)]
pub enum SessionError {
    /// Error during state save
    SaveState(String),
    /// Error during state load
    LoadState(String),
    /// GGRS error
    Ggrs(String),
    /// Desync detected
    Desync {
        frame: i32,
        local_checksum: u64,
        remote_checksum: u64,
    },
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SaveState(e) => write!(f, "Failed to save state: {}", e),
            Self::LoadState(e) => write!(f, "Failed to load state: {}", e),
            Self::Ggrs(e) => write!(f, "GGRS error: {}", e),
            Self::Desync {
                frame,
                local_checksum,
                remote_checksum,
            } => write!(
                f,
                "Desync detected at frame {}: local={:#x}, remote={:#x}",
                frame, local_checksum, remote_checksum
            ),
        }
    }
}

impl std::error::Error for SessionError {}

impl From<GgrsError> for SessionError {
    fn from(e: GgrsError) -> Self {
        Self::Ggrs(e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasm::InputState;

    // Test input type for unit tests
    #[repr(C)]
    #[derive(Clone, Copy, Default, PartialEq, Debug)]
    struct TestInput {
        buttons: u16,
        x: i8,
        y: i8,
    }

    // SAFETY: TestInput is #[repr(C)] with only primitive types (u16, i8, i8).
    // All bit patterns are valid for these types, satisfying Pod and Zeroable requirements.
    unsafe impl Pod for TestInput {}
    unsafe impl Zeroable for TestInput {}
    impl ConsoleInput for TestInput {
        fn to_input_state(&self) -> InputState {
            InputState {
                buttons: self.buttons,
                left_stick_x: self.x,
                left_stick_y: self.y,
                ..Default::default()
            }
        }
    }

    #[test]
    fn test_rollback_session_local() {
        let session = RollbackSession::<TestInput>::new_local(2);
        assert_eq!(session.session_type(), SessionType::Local);
        assert_eq!(session.config().num_players, 2);
        assert_eq!(session.current_frame(), 0);
        assert_eq!(session.local_players(), &[0, 1]);
    }

    #[test]
    fn test_rollback_session_sync_test() {
        let config = SessionConfig::sync_test();
        let session = RollbackSession::<TestInput>::new_sync_test(config).unwrap();
        assert_eq!(session.session_type(), SessionType::SyncTest);
    }

    #[test]
    fn test_local_session_advance() {
        let mut session = RollbackSession::<TestInput>::new_local(2);
        assert_eq!(session.current_frame(), 0);

        let requests = session.advance_frame().unwrap();
        assert_eq!(requests.len(), 1);

        match &requests[0] {
            GgrsRequest::AdvanceFrame { inputs } => {
                assert_eq!(inputs.len(), 2);
                for (input, status) in inputs {
                    assert_eq!(*input, TestInput::default());
                    assert_eq!(*status, InputStatus::Confirmed);
                }
            }
            _ => panic!("Expected AdvanceFrame request"),
        }

        assert_eq!(session.current_frame(), 1);
    }

    #[test]
    fn test_network_input_wrapper() {
        let input = TestInput {
            buttons: 0xFF,
            x: 100,
            y: -50,
        };
        let network_input = NetworkInput::new(input);
        assert_eq!(network_input.input, input);
    }

    #[test]
    fn test_network_input_pod_zeroable() {
        // Verify NetworkInput satisfies Pod + Zeroable requirements
        let zeroed: NetworkInput<TestInput> = bytemuck::Zeroable::zeroed();
        assert_eq!(zeroed.input.buttons, 0);
        assert_eq!(zeroed.input.x, 0);
        assert_eq!(zeroed.input.y, 0);

        // Verify we can cast to/from bytes
        let input = NetworkInput::new(TestInput {
            buttons: 0x1234,
            x: 10,
            y: -20,
        });
        let bytes: &[u8] = bytemuck::bytes_of(&input);
        let restored: &NetworkInput<TestInput> = bytemuck::from_bytes(bytes);
        assert_eq!(restored.input, input.input);
    }

    #[test]
    fn test_connection_quality_assessment() {
        let mut stats = PlayerNetworkStats {
            connected: true,
            ping_ms: 30,
            local_frames_ahead: 1,
            ..Default::default()
        };
        stats.assess_quality();
        assert_eq!(stats.quality, ConnectionQuality::Excellent);

        // Test good quality
        stats.ping_ms = 75;
        stats.local_frames_ahead = 3;
        stats.assess_quality();
        assert_eq!(stats.quality, ConnectionQuality::Good);

        // Test fair quality
        stats.ping_ms = 120;
        stats.local_frames_ahead = 5;
        stats.assess_quality();
        assert_eq!(stats.quality, ConnectionQuality::Fair);

        // Test poor quality
        stats.ping_ms = 200;
        stats.local_frames_ahead = 8;
        stats.assess_quality();
        assert_eq!(stats.quality, ConnectionQuality::Poor);

        // Test disconnected
        stats.connected = false;
        stats.assess_quality();
        assert_eq!(stats.quality, ConnectionQuality::Disconnected);
    }

    #[test]
    fn test_player_network_stats_default() {
        let stats = PlayerNetworkStats::default();
        assert_eq!(stats.ping_ms, 0);
        assert_eq!(stats.packet_loss, 0);
        assert_eq!(stats.local_frames_ahead, 0);
        assert_eq!(stats.remote_frames_ahead, 0);
        assert_eq!(stats.rollback_frames, 0);
        assert!(!stats.connected);
    }

    #[test]
    fn test_session_error_display() {
        let save_err = SessionError::SaveState("memory full".to_string());
        assert!(save_err.to_string().contains("memory full"));

        let load_err = SessionError::LoadState("corrupted".to_string());
        assert!(load_err.to_string().contains("corrupted"));

        let desync_err = SessionError::Desync {
            frame: 100,
            local_checksum: 0xDEAD,
            remote_checksum: 0xBEEF,
        };
        let msg = desync_err.to_string();
        assert!(msg.contains("100"));
        assert!(msg.contains("0xdead"));
        assert!(msg.contains("0xbeef"));
    }

    #[test]
    fn test_local_session_has_no_network_stats() {
        let session = RollbackSession::<TestInput>::new_local(2);
        assert!(session.all_player_stats().is_empty());
        assert!(session.player_stats(0).is_none());
    }

    #[test]
    fn test_local_session_no_desync() {
        let session = RollbackSession::<TestInput>::new_local(2);
        assert!(!session.has_desync());
    }

    #[test]
    fn test_local_session_total_rollback_frames() {
        let session = RollbackSession::<TestInput>::new_local(2);
        assert_eq!(session.total_rollback_frames(), 0);
    }

    #[test]
    fn test_local_session_handle_events_empty() {
        let mut session = RollbackSession::<TestInput>::new_local(2);
        let events = session.handle_events();
        // Local sessions don't produce events
        assert!(events.is_empty());
    }

    #[test]
    fn test_session_event_variants() {
        // Test that all event variants can be created
        let sync = SessionEvent::Synchronized { player_handle: 0 };
        let disc = SessionEvent::Disconnected { player_handle: 1 };
        let desync = SessionEvent::Desync {
            frame: 50,
            local_checksum: 123,
            remote_checksum: 456,
        };
        let interrupted = SessionEvent::NetworkInterrupted {
            player_handle: 0,
            disconnect_timeout_ms: 3000,
        };
        let resumed = SessionEvent::NetworkResumed { player_handle: 0 };
        let advantage = SessionEvent::FrameAdvantageWarning { frames_ahead: 5 };
        let timesync = SessionEvent::TimeSync { frames_to_skip: 2 };
        let waiting = SessionEvent::WaitingForPlayers;

        // Verify Debug trait works
        assert!(!format!("{:?}", sync).is_empty());
        assert!(!format!("{:?}", disc).is_empty());
        assert!(!format!("{:?}", desync).is_empty());
        assert!(!format!("{:?}", interrupted).is_empty());
        assert!(!format!("{:?}", resumed).is_empty());
        assert!(!format!("{:?}", advantage).is_empty());
        assert!(!format!("{:?}", timesync).is_empty());
        assert!(!format!("{:?}", waiting).is_empty());
    }

    #[test]
    fn test_rollback_session_local_has_player_config() {
        let session = RollbackSession::<TestInput>::new_local(2);
        let player_config = session.player_config();
        assert_eq!(player_config.num_players(), 2);
        assert_eq!(player_config.local_player_count(), 2);
        assert!(player_config.is_local_player(0));
        assert!(player_config.is_local_player(1));
    }

    #[test]
    fn test_rollback_session_local_with_config() {
        // Create a local session with custom player config
        let player_config = PlayerSessionConfig::new(4, 0b0011); // Only players 0, 1 local
        let session = RollbackSession::<TestInput>::new_local_with_config(player_config);

        assert_eq!(session.player_config().num_players(), 4);
        assert_eq!(session.player_config().local_player_mask(), 0b0011);
        assert_eq!(session.local_players(), &[0, 1]);
    }

    #[test]
    fn test_rollback_session_sync_test_has_player_config() {
        let config = SessionConfig::sync_test();
        let session = RollbackSession::<TestInput>::new_sync_test(config).unwrap();
        let player_config = session.player_config();
        assert_eq!(player_config.num_players(), 1);
        assert!(player_config.is_local_player(0));
    }
}
