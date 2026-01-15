//! RollbackSession core implementation

use ggrs::{GgrsError, GgrsEvent, GgrsRequest, InputStatus, SessionState};

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::GameInstance;

use super::super::config::{NethercoreConfig, SessionConfig};
use super::super::events::{PlayerNetworkStats, SessionError, SessionEvent};
use super::super::player::PlayerSessionConfig;
use super::super::state::{GameStateSnapshot, LoadStateError, RollbackStateManager, SaveStateError};
use super::types::{SessionInner, SessionType};

/// Frame advantage threshold for warning events
const FRAME_ADVANTAGE_WARNING_THRESHOLD: i32 = 4;

/// Rollback session manager
///
/// Wraps GGRS session types and provides a unified interface for
/// local, sync-test, and P2P sessions. Handles state management
/// and input processing.
pub struct RollbackSession<
    I: ConsoleInput,
    S: Send + Default + 'static,
    R: ConsoleRollbackState = (),
> {
    pub(super) inner: SessionInner<I>,
    pub(super) session_type: SessionType,
    pub(super) config: SessionConfig,
    /// Player session configuration (local/remote player assignments)
    pub(super) player_config: PlayerSessionConfig,
    pub(super) state_manager: RollbackStateManager,
    /// Whether we're currently in rollback mode (used to mute audio)
    pub(super) rolling_back: bool,
    /// Local player handles for this session
    pub(super) local_players: Vec<usize>,
    /// Network statistics per remote player
    pub(super) network_stats: Vec<PlayerNetworkStats>,
    /// Number of rollback frames this session
    pub(super) total_rollback_frames: u64,
    /// Last frame advantage (for warning detection)
    pub(super) last_frame_advantage: i32,
    /// Whether a desync has been detected
    pub(super) desync_detected: bool,
    pub(super) _phantom: std::marker::PhantomData<(S, R)>,
}

impl<I: ConsoleInput, S: Send + Default + 'static, R: ConsoleRollbackState>
    RollbackSession<I, S, R>
{
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
            SessionInner::Local { stored_inputs, .. } => {
                // Store input for use in advance_frame
                if player_handle < stored_inputs.len() {
                    stored_inputs[player_handle] = input;
                }
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
    /// with the stored inputs (set via `add_local_input`).
    pub fn advance_frame(&mut self) -> Result<Vec<GgrsRequest<NethercoreConfig<I>>>, GgrsError> {
        match &mut self.inner {
            SessionInner::Local {
                current_frame,
                stored_inputs,
                ..
            } => {
                // Local sessions advance immediately with stored inputs
                *current_frame += 1;
                let inputs: Vec<(I, InputStatus)> = stored_inputs
                    .iter()
                    .map(|input| (*input, InputStatus::Confirmed))
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
    pub fn events(&mut self) -> Vec<GgrsEvent<NethercoreConfig<I>>> {
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
                    tracing::debug!("Synchronizing: {}/{}", count, total);
                }
                GgrsEvent::Synchronized { addr: _ } => {
                    // Find the player handle for this address
                    // For now, we emit a generic synchronized event
                    tracing::info!("Peer synchronized");
                    // We don't have a direct mapping from address to player handle
                    // in the current design, so we use a placeholder
                    session_events.push(SessionEvent::Synchronized { player_handle: 0 });
                }
                GgrsEvent::Disconnected { addr: _ } => {
                    tracing::warn!("Peer disconnected");
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
                    tracing::warn!("Network interrupted, disconnect in {}ms", timeout_ms);
                    session_events.push(SessionEvent::NetworkInterrupted {
                        player_handle: 0,
                        disconnect_timeout_ms: timeout_ms,
                    });
                }
                GgrsEvent::NetworkResumed { addr: _ } => {
                    tracing::info!("Network resumed");
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
                    tracing::debug!("Wait recommendation: skip {} frames", skip_frames);
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
                    tracing::error!(
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
            tracing::debug!("Frame advantage warning: {} frames ahead", frames_ahead);
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
        game: &mut GameInstance<I, S, R>,
        requests: Vec<GgrsRequest<NethercoreConfig<I>>>,
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
                stats.rollback_frames = stats
                    .rollback_frames
                    .saturating_add(rollback_frames_this_call);
            }
        }

        Ok(advance_inputs)
    }

    /// Save game state (convenience wrapper)
    pub fn save_game_state(
        &mut self,
        game: &mut GameInstance<I, S, R>,
        frame: i32,
    ) -> Result<GameStateSnapshot, SaveStateError> {
        self.state_manager.save_state(game, frame)
    }

    /// Load game state (convenience wrapper)
    pub fn load_game_state(
        &mut self,
        game: &mut GameInstance<I, S, R>,
        snapshot: &GameStateSnapshot,
    ) -> Result<(), LoadStateError> {
        self.state_manager.load_state(game, snapshot)
    }
}
