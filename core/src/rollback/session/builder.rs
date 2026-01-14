//! Session construction and initialization

use std::time::Duration;

use ggrs::{GgrsError, NonBlockingSocket, PlayerType, SessionBuilder};

use crate::console::{ConsoleInput, ConsoleRollbackState};

use super::super::config::{NethercoreConfig, SessionConfig};
use super::super::events::PlayerNetworkStats;
use super::super::player::{MAX_PLAYERS, PlayerSessionConfig};
use super::super::state::RollbackStateManager;
use super::session::RollbackSession;
use super::types::{SessionInner, SessionType};

impl<I: ConsoleInput, S: Send + Default + 'static, R: ConsoleRollbackState>
    RollbackSession<I, S, R>
{
    /// Create a new local session (no rollback)
    ///
    /// Local sessions run without GGRS - updates execute immediately
    /// without any rollback support. Useful for single player games
    /// or local multiplayer on the same machine.
    ///
    /// All players are assumed to be local.
    ///
    /// `max_state_size` should match the console's RAM limit (e.g., `console.specs().ram_limit`).
    pub fn new_local(num_players: usize, max_state_size: usize) -> Self {
        let player_config = PlayerSessionConfig::all_local(num_players as u32);
        Self::new_local_with_config(player_config, max_state_size)
    }

    /// Create a new local session with explicit player configuration
    ///
    /// This allows specifying which players are local vs remote.
    /// For local sessions, all players should typically be local,
    /// but this method allows flexibility for testing or special scenarios.
    ///
    /// `max_state_size` should match the console's RAM limit (e.g., `console.specs().ram_limit`).
    pub fn new_local_with_config(
        player_config: PlayerSessionConfig,
        max_state_size: usize,
    ) -> Self {
        let num_players = player_config.num_players() as usize;
        let local_players = player_config.local_player_indices();

        Self {
            inner: SessionInner::Local {
                current_frame: 0,
                stored_inputs: vec![I::default(); num_players],
            },
            session_type: SessionType::Local,
            config: SessionConfig::local(num_players),
            player_config,
            state_manager: RollbackStateManager::new(max_state_size),
            rolling_back: false,
            local_players,
            network_stats: Vec::new(), // No network stats for local
            total_rollback_frames: 0,
            last_frame_advantage: 0,
            desync_detected: false,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Create a new sync test session (for testing determinism)
    ///
    /// Sync test sessions simulate rollback every frame to verify
    /// the game state is deterministic. Use this during development
    /// to catch non-determinism bugs.
    ///
    /// `max_state_size` should match the console's RAM limit (e.g., `console.specs().ram_limit`).
    pub fn new_sync_test(config: SessionConfig, max_state_size: usize) -> Result<Self, GgrsError> {
        let player_config = PlayerSessionConfig::all_local(config.num_players as u32);
        Self::new_sync_test_with_config(config, player_config, max_state_size)
    }

    /// Create a new sync test session with explicit player configuration
    ///
    /// `max_state_size` should match the console's RAM limit (e.g., `console.specs().ram_limit`).
    pub fn new_sync_test_with_config(
        config: SessionConfig,
        player_config: PlayerSessionConfig,
        max_state_size: usize,
    ) -> Result<Self, GgrsError> {
        let session = SessionBuilder::<NethercoreConfig<I>>::new()
            .with_num_players(config.num_players)
            .with_max_prediction_window(config.max_prediction_frames)
            .with_input_delay(config.input_delay)
            .with_check_distance(2)
            .start_synctest_session()?;

        let local_players = player_config.local_player_indices();

        Ok(Self {
            inner: SessionInner::SyncTest {
                session: Box::new(session),
                current_frame: 0,
            },
            session_type: SessionType::SyncTest,
            config,
            player_config,
            state_manager: RollbackStateManager::new(max_state_size),
            rolling_back: false,
            local_players,
            network_stats: Vec::new(), // No network stats for sync test
            total_rollback_frames: 0,
            last_frame_advantage: 0,
            desync_detected: false,
            _phantom: std::marker::PhantomData,
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
    ///
    /// `max_state_size` should match the console's RAM limit (e.g., `console.specs().ram_limit`).
    pub fn new_p2p<Sock>(
        config: SessionConfig,
        socket: Sock,
        players: Vec<(usize, PlayerType<String>)>,
        max_state_size: usize,
    ) -> Result<Self, GgrsError>
    where
        Sock: NonBlockingSocket<String> + 'static,
    {
        // Build player config from the players list
        let mut local_mask = 0u32;
        for (handle, player_type) in &players {
            if matches!(player_type, PlayerType::Local) && *handle < MAX_PLAYERS {
                local_mask |= 1u32 << handle;
            }
        }
        let player_config = PlayerSessionConfig::new(config.num_players as u32, local_mask);

        Self::new_p2p_with_config(config, player_config, socket, players, max_state_size)
    }

    /// Create a new P2P session with explicit player configuration
    ///
    /// This allows full control over the player session configuration.
    /// The `players` parameter still specifies the GGRS player types.
    ///
    /// `max_state_size` should match the console's RAM limit (e.g., `console.specs().ram_limit`).
    pub fn new_p2p_with_config<Sock>(
        config: SessionConfig,
        player_config: PlayerSessionConfig,
        socket: Sock,
        players: Vec<(usize, PlayerType<String>)>,
        max_state_size: usize,
    ) -> Result<Self, GgrsError>
    where
        Sock: NonBlockingSocket<String> + 'static,
    {
        let mut builder = SessionBuilder::<NethercoreConfig<I>>::new()
            .with_num_players(config.num_players)
            .with_max_prediction_window(config.max_prediction_frames)
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
            state_manager: RollbackStateManager::new(max_state_size),
            rolling_back: false,
            local_players,
            network_stats,
            total_rollback_frames: 0,
            last_frame_advantage: 0,
            desync_detected: false,
            _phantom: std::marker::PhantomData,
        })
    }

    /// Create a P2P session from NCHS session configuration
    ///
    /// This is the preferred way to create a session after NCHS handshake completes.
    /// The `SessionStart` contains all determinism-critical configuration from the host.
    ///
    /// # Arguments
    ///
    /// * `session_start` - SessionStart received from NCHS handshake
    /// * `local_handle` - Our local player handle (0-3)
    /// * `socket` - Network socket for GGRS communication
    /// * `max_state_size` - Maximum state size for rollback (usually console's RAM limit)
    ///
    /// # Example
    ///
    /// ```rust,ignore
    /// let session = RollbackSession::from_nchs_session(
    ///     &nchs_session.session_config().unwrap(),
    ///     nchs_session.local_handle().unwrap(),
    ///     nchs_socket,
    ///     console.specs().ram_limit,
    /// )?;
    /// ```
    pub fn from_nchs_session<Sock>(
        session_start: &crate::net::nchs::SessionStart,
        local_handle: u8,
        socket: Sock,
        max_state_size: usize,
    ) -> Result<Self, GgrsError>
    where
        Sock: NonBlockingSocket<String> + 'static,
    {
        // Build session config from NCHS
        let nchs_config = &session_start.network_config;
        let config = SessionConfig {
            num_players: session_start.player_count as usize,
            max_prediction_frames: nchs_config.max_rollback as usize,
            input_delay: nchs_config.input_delay as usize,
            fps: session_start.tick_rate.as_hz() as usize,
            disconnect_timeout: nchs_config.disconnect_timeout_ms as u64,
            disconnect_notify_start: nchs_config.disconnect_timeout_ms as u64 / 2,
        };

        // Build player types from NCHS player list
        let mut players = Vec::new();
        for player in &session_start.players {
            if !player.active {
                continue;
            }

            let player_type = if player.handle == local_handle {
                PlayerType::Local
            } else {
                // Use the GGRS port for remote players
                let addr = format!(
                    "{}:{}",
                    player.addr.split(':').next().unwrap_or("127.0.0.1"),
                    player.ggrs_port
                );
                PlayerType::Remote(addr)
            };

            players.push((player.handle as usize, player_type));
        }

        Self::new_p2p(config, socket, players, max_state_size)
    }
}
