//! Session start and game flow logic for host

use std::time::Instant;

use crate::net::nchs::NchsError;
use crate::net::nchs::messages::{NchsMessage, PlayerConnectionInfo, PlayerInfo, SessionStart};

use super::state::HostStateMachine;

impl HostStateMachine {
    /// Start the game session
    ///
    /// Call this when all players are ready and the host wants to start.
    /// Returns the SessionStart that will be sent to all players.
    pub fn start(&mut self) -> Result<SessionStart, NchsError> {
        if !self.all_ready() {
            return Err(NchsError::ProtocolError(
                "Not all players ready".to_string(),
            ));
        }

        if self.player_count() < 2 {
            return Err(NchsError::ProtocolError(
                "Need at least 2 players".to_string(),
            ));
        }

        // Generate random seed
        let random_seed = rand::random::<u64>();
        self.random_seed = Some(random_seed);

        // Build player connection info
        let mut players = Vec::with_capacity(self.netplay.max_players as usize);

        // Add host
        let host_ggrs_port = self.socket.port().checked_add(1).ok_or_else(|| {
            NchsError::ProtocolError("Host port too high for GGRS (max 65534)".to_string())
        })?;
        players.push(PlayerConnectionInfo {
            handle: 0,
            active: true,
            info: self.host_info.clone(),
            addr: self.public_addr.clone(),
            ggrs_port: host_ggrs_port, // GGRS uses port + 1
        });

        // Add other players
        for handle in 1..self.netplay.max_players {
            if let Some(player) = self.players.get(&handle) {
                let ggrs_port = player.addr.port().checked_add(1).ok_or_else(|| {
                    NchsError::ProtocolError(format!(
                        "Player {} port too high for GGRS (max 65534)",
                        handle
                    ))
                })?;
                players.push(PlayerConnectionInfo {
                    handle,
                    active: true,
                    info: player.info.clone(),
                    addr: player.addr.to_string(),
                    ggrs_port,
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
            save_config: self.save_config.clone(),
            extra_data: vec![],
        };

        // Send SessionStart to all guests
        let msg = NchsMessage::SessionStart(session_start.clone());
        for player in self.players.values() {
            if let Err(e) = self.socket.send_to(player.addr, &msg) {
                tracing::warn!(
                    error = %e,
                    player = player.handle,
                    "Failed to send SessionStart"
                );
            }
        }

        self.state = super::state::HostState::Starting;
        self.start_time = Some(Instant::now());

        tracing::info!(
            "Session started with {} players, seed: {:016x}",
            self.player_count(),
            random_seed
        );

        Ok(session_start)
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
            save_config: self.save_config.clone(),
            extra_data: vec![],
        })
    }

    /// Build player connection info list
    fn build_player_connection_info(&self) -> Vec<PlayerConnectionInfo> {
        let mut players = Vec::with_capacity(self.netplay.max_players as usize);

        // Add host
        let host_ggrs_port = self.socket.port().checked_add(1).unwrap_or_else(|| {
            tracing::warn!("Host port too high for GGRS (max 65534)");
            0
        });
        players.push(PlayerConnectionInfo {
            handle: 0,
            active: true,
            info: self.host_info.clone(),
            addr: self.public_addr.clone(),
            ggrs_port: host_ggrs_port,
        });

        // Add other players
        for handle in 1..self.netplay.max_players {
            if let Some(player) = self.players.get(&handle) {
                let ggrs_port = player.addr.port().checked_add(1).unwrap_or_else(|| {
                    tracing::warn!(player = handle, "Player port too high for GGRS (max 65534)");
                    0
                });
                players.push(PlayerConnectionInfo {
                    handle,
                    active: true,
                    info: player.info.clone(),
                    addr: player.addr.to_string(),
                    ggrs_port,
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
