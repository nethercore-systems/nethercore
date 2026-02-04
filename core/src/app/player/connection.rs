//! Network connection and NCHS protocol handling

use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use ggrs::PlayerType;

use crate::console::{Audio, Console};
use crate::net::nchs::SaveConfig;
use crate::net::nchs::SessionStart;
use crate::rollback::{LocalSocket, RollbackSession, SessionConfig};

use super::super::GameErrorPhase;
use super::StandaloneApp;
use super::types::{RomLoader, StandaloneConfig};

/// Formats a GGRS address from an address string and port
pub(super) fn format_ggrs_addr(addr: &str, port: u16) -> String {
    if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
        return SocketAddr::new(socket_addr.ip(), port).to_string();
    }

    if let Ok(ip) = addr.parse::<std::net::IpAddr>() {
        return SocketAddr::new(ip, port).to_string();
    }

    if let Some(stripped) = addr.strip_prefix('[').and_then(|s| s.strip_suffix(']'))
        && let Ok(ip) = stripped.parse::<std::net::IpAddr>()
    {
        return SocketAddr::new(ip, port).to_string();
    }

    // Fallback: split on the last ':' to preserve IPv6 host parts without brackets.
    let mut parts = addr.rsplitn(2, ':');
    let _port_part = parts.next();
    if let Some(host) = parts.next() {
        if host.contains(':') && !(host.starts_with('[') && host.ends_with(']')) {
            return format!("[{}]:{}", host, port);
        }
        return format!("{}:{}", host, port);
    }

    format!("{}:{}", addr, port)
}

/// Performs NCHS handshake and creates P2P session from a session file
pub struct SessionFileResult<C: Console> {
    pub session: RollbackSession<C::Input, C::State, C::RollbackState>,
    pub save_config: Option<SaveConfig>,
}

pub(super) fn decode_session_file(session_file: &std::path::Path) -> Result<SessionStart> {
    const MAX_SESSION_FILE_BYTES: u64 = 1024 * 1024; // 1 MiB
    let session_file_len = std::fs::metadata(session_file)
        .with_context(|| format!("Failed to stat session file: {}", session_file.display()))?
        .len();
    anyhow::ensure!(
        session_file_len <= MAX_SESSION_FILE_BYTES,
        "Session file is too large ({} bytes, max {} bytes): {}",
        session_file_len,
        MAX_SESSION_FILE_BYTES,
        session_file.display()
    );

    let bytes = std::fs::read(session_file).context("Failed to read session file")?;
    let session_start: SessionStart =
        bitcode::decode(&bytes).map_err(|e| anyhow::anyhow!("Failed to decode session: {}", e))?;
    Ok(session_start)
}

pub(super) fn create_session_from_file<C>(
    session_file: &std::path::Path,
    _config: &StandaloneConfig,
    specs: &crate::console::ConsoleSpecs,
) -> Result<SessionFileResult<C>>
where
    C: Console + Clone,
{
    let mut session_start = decode_session_file(session_file)?;
    let save_config = session_start.save_config.take();

    tracing::info!(
        "Session mode: loading pre-negotiated session (local_player={}, player_count={}, seed={})",
        session_start.local_player_handle,
        session_start.player_count,
        session_start.random_seed
    );

    let local_handle = session_start.local_player_handle as usize;
    let is_host = session_start.local_player_handle == 0;

    // Get our own ggrs_port
    let own_ggrs_port = session_start
        .players
        .iter()
        .find(|p| p.handle == session_start.local_player_handle)
        .map(|p| p.ggrs_port)
        .unwrap_or(0);

    tracing::info!(
        "Session mode: binding to ggrs_port {} (handle {}, is_host={})",
        own_ggrs_port,
        session_start.local_player_handle,
        is_host
    );

    // Bind to our GGRS port
    let socket = LocalSocket::bind(&format!("0.0.0.0:{}", own_ggrs_port))
        .context("Failed to bind GGRS socket")?;

    // Handshake magic bytes to identify our packets
    const HANDSHAKE_HELLO: &[u8] = b"NCHS_HELLO";
    const HANDSHAKE_READY: &[u8] = b"NCHS_READY";
    const HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);

    // Perform handshake to ensure all peers are ready before creating GGRS session
    // This prevents the race condition where one side starts sending GGRS packets
    // before the other side has bound to its port.
    let peer_addresses: HashMap<u8, SocketAddr> = if is_host {
        // HOST: Wait for all guests to send HELLO, then send READY to each
        let expected_guests: Vec<u8> = session_start
            .players
            .iter()
            .filter(|p| p.active && p.handle != 0)
            .map(|p| p.handle)
            .collect();

        tracing::info!(
            "Session mode: host waiting for {} guest(s) to connect",
            expected_guests.len()
        );

        let mut received_from: HashMap<u8, SocketAddr> = HashMap::new();
        let start = Instant::now();

        while received_from.len() < expected_guests.len() {
            if start.elapsed() > HANDSHAKE_TIMEOUT {
                anyhow::bail!("Timeout waiting for guests to connect");
            }

            // Try to receive HELLO from guests
            let mut buf = [0u8; 64];
            match socket.socket().recv_from(&mut buf) {
                Ok((len, from)) => {
                    if len >= HANDSHAKE_HELLO.len()
                        && &buf[..HANDSHAKE_HELLO.len()] == HANDSHAKE_HELLO
                    {
                        // Extract handle from after HELLO
                        if len > HANDSHAKE_HELLO.len() {
                            let handle = buf[HANDSHAKE_HELLO.len()];
                            if expected_guests.contains(&handle)
                                && !received_from.contains_key(&handle)
                            {
                                tracing::info!(
                                    "Session mode: received HELLO from guest {} at {}",
                                    handle,
                                    from
                                );
                                received_from.insert(handle, from);

                                // Send READY back immediately
                                let mut ready_msg = HANDSHAKE_READY.to_vec();
                                ready_msg.push(session_start.local_player_handle);
                                let _ = socket.socket().send_to(&ready_msg, from);
                            }
                        }
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
        }

        tracing::info!(
            "Session mode: all {} guest(s) connected",
            received_from.len()
        );
        received_from
    } else {
        // GUEST: Send HELLO to host, wait for READY
        let host_player = session_start
            .players
            .iter()
            .find(|p| p.handle == 0)
            .ok_or_else(|| anyhow::anyhow!("Session has no host player"))?;

        let host_addr: SocketAddr = format!(
            "{}:{}",
            host_player.addr.split(':').next().unwrap_or("127.0.0.1"),
            host_player.ggrs_port
        )
        .parse()
        .context("Invalid host address")?;

        tracing::info!("Session mode: guest sending HELLO to host at {}", host_addr);

        let start = Instant::now();
        let mut received_ready = false;

        while !received_ready {
            if start.elapsed() > HANDSHAKE_TIMEOUT {
                anyhow::bail!("Timeout waiting for host READY");
            }

            // Send HELLO
            let mut hello_msg = HANDSHAKE_HELLO.to_vec();
            hello_msg.push(session_start.local_player_handle);
            let _ = socket.socket().send_to(&hello_msg, host_addr);

            // Wait a bit for READY
            std::thread::sleep(Duration::from_millis(50));

            // Check for READY
            let mut buf = [0u8; 64];
            match socket.socket().recv_from(&mut buf) {
                Ok((len, _from)) => {
                    if len >= HANDSHAKE_READY.len()
                        && &buf[..HANDSHAKE_READY.len()] == HANDSHAKE_READY
                    {
                        tracing::info!("Session mode: received READY from host");
                        received_ready = true;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
                Err(_) => {}
            }
        }

        // Return host address
        let mut addrs = HashMap::new();
        addrs.insert(0, host_addr);
        addrs
    };

    // Now build the player list using actual discovered addresses for guests (host only)
    // Guests use pre-specified host address
    let players: Vec<(usize, PlayerType<String>)> = session_start
        .players
        .iter()
        .filter(|p| p.active)
        .map(|p| {
            let handle = p.handle as usize;
            if handle == local_handle {
                (handle, PlayerType::Local)
            } else if let Some(actual_addr) = peer_addresses.get(&p.handle) {
                // Use actual discovered address (from handshake)
                (handle, PlayerType::Remote(actual_addr.to_string()))
            } else {
                // Fallback to pre-specified address
                let ggrs_addr = format_ggrs_addr(&p.addr, p.ggrs_port);
                (handle, PlayerType::Remote(ggrs_addr))
            }
        })
        .collect();

    tracing::info!("Session mode: players (after handshake) = {:?}", players);

    let mut session_config = SessionConfig::online(session_start.player_count as usize)
        .with_input_delay(session_start.network_config.input_delay as usize);
    session_config.fps = session_start.tick_rate.as_hz() as usize;

    let session = RollbackSession::new_p2p(session_config, socket, players, specs.ram_limit)
        .context("Failed to create session from NCHS config")?;

    tracing::info!(
        "Session mode: session created, local_players = {:?}",
        session.local_players()
    );

    Ok(SessionFileResult {
        session,
        save_config,
    })
}

/// Action from the join connection UI
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinConnectionAction {
    /// No action needed
    None,
    /// User requested to retry the connection
    Retry,
    /// User requested to cancel and quit
    Cancel,
}

impl<C, L> StandaloneApp<C, L>
where
    C: Console + Clone,
    C::Graphics: super::types::StandaloneGraphicsSupport,
    L: RomLoader<Console = C>,
{
    /// Polls for join connection progress and creates session when connected
    pub(super) fn poll_for_join_connection(&mut self) -> bool {
        use super::error_ui::JoinConnectionState;

        let joining = match &mut self.joining_peer {
            Some(j) => j,
            None => return false,
        };

        // Check for timeout
        if joining.is_timed_out() && joining.state == JoinConnectionState::Connecting {
            tracing::warn!("Join connection timed out after {:?}", joining.timeout);
            joining.mark_timed_out();
            self.needs_redraw = true;
            return false;
        }

        // Send probe packets periodically while connecting
        if joining.should_send_probe() {
            let elapsed_ms = joining.elapsed().as_millis() as u64;
            let probe_interval_ms = super::error_ui::JoiningPeer::PROBE_INTERVAL.as_millis() as u64;

            if elapsed_ms / probe_interval_ms > joining.attempt_count as u64 {
                joining.attempt_count += 1;

                // Send probe packet to host
                if let Some(peer_addr) = joining.socket.peer_addr() {
                    let probe_data = super::error_ui::JoiningPeer::PROBE_MAGIC;
                    if let Err(e) = joining.socket.socket().send_to(probe_data, peer_addr) {
                        tracing::debug!("Failed to send probe packet: {}", e);
                    } else {
                        tracing::debug!(
                            "Sent probe packet #{} to {}",
                            joining.attempt_count,
                            peer_addr
                        );
                    }
                }
            }
        }

        // Check for response from host
        if joining.state == JoinConnectionState::Connecting {
            let mut buf = [0u8; 128];
            match joining.socket.socket().recv_from(&mut buf) {
                Ok((len, from)) => {
                    // Any response from the host means they're listening
                    tracing::info!(
                        "Received response from host {} ({} bytes), connection established",
                        from,
                        len
                    );
                    joining.mark_connected();
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // No data yet, keep waiting
                }
                Err(e) => {
                    tracing::debug!("Recv error (may be expected): {}", e);
                }
            }
        }

        // If connected, create the session and start the game
        if joining.state == JoinConnectionState::Connected {
            tracing::info!("Join connection established, creating P2P session");

            // Take ownership of the joining state
            let joining = self.joining_peer.take().unwrap();
            let address = joining.address.clone();

            // Create the P2P session
            if let (Some(rom), Some(runner)) = (&self.loaded_rom, &mut self.runner) {
                let specs = C::specs();
                let session_config =
                    SessionConfig::online(2).with_input_delay(self.config.input_delay);

                // Joiner is player 1, host is player 0
                let players = vec![
                    (0, PlayerType::Remote(address.clone())),
                    (1, PlayerType::Local),
                ];
                tracing::info!(
                    "Join mode: creating P2P session (host=remote p0, local=p1)"
                );

                match RollbackSession::new_p2p(
                    session_config,
                    joining.socket,
                    players,
                    specs.ram_limit,
                ) {
                    Ok(session) => {
                        tracing::info!(
                            "Join mode: session created, local_players = {:?}",
                            session.local_players()
                        );
                        if let Err(e) = runner.load_game_with_session(
                            rom.console.clone(),
                            &rom.code,
                            session,
                            None,
                            &rom.game_id,
                        ) {
                            tracing::error!("Failed to load game with P2P session: {}", e);
                            self.error_state = Some(super::super::GameError {
                                summary: "Connection Error".to_string(),
                                details: format!("Failed to start game: {}", e),
                                stack_trace: None,
                                tick: None,
                                phase: GameErrorPhase::Update,
                                suggestions: vec![],
                            });
                        } else {
                            tracing::info!("Join mode: game started with host at {}", address);
                            // Set audio volume
                            if let Some(session) = runner.session_mut()
                                && let Some(audio) = session.runtime.audio_mut()
                            {
                                let config = super::super::config::load();
                                audio.set_master_volume(config.audio.master_volume);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create P2P session: {}", e);
                        self.error_state = Some(super::super::GameError {
                            summary: "Connection Error".to_string(),
                            details: format!("Failed to create session: {}", e),
                            stack_trace: None,
                            tick: None,
                            phase: GameErrorPhase::Update,
                            suggestions: vec![],
                        });
                    }
                }
            }

            self.needs_redraw = true;
            return true;
        }

        false
    }

    /// Handle retry action for join connection
    pub(super) fn retry_join_connection(&mut self) {
        if let Some(ref mut joining) = self.joining_peer {
            tracing::info!("Retrying join connection to {}", joining.address);
            joining.reset_for_retry();
            self.needs_redraw = true;
        }
    }

    /// Polls for peer connection in Host mode and creates session when connected
    pub(super) fn poll_for_peer_connection(&mut self) -> bool {
        if let Some(ref mut waiting) = self.waiting_for_peer
            && let Some(peer_addr) = waiting.socket.poll_for_peer()
        {
            tracing::info!("Peer connected from {}", peer_addr);

            // Take the waiting state to get ownership of the socket
            let waiting = self.waiting_for_peer.take().unwrap();

            // Create the P2P session now that we have a peer
            if let (Some(rom), Some(runner)) = (&self.loaded_rom, &mut self.runner) {
                let specs = C::specs();
                let session_config =
                    SessionConfig::online(2).with_input_delay(self.config.input_delay);

                // Host is player 0, peer is player 1
                let players = vec![
                    (0, PlayerType::Local),
                    (1, PlayerType::Remote(peer_addr.clone())),
                ];
                tracing::info!("Host mode: creating P2P session (host=local p0, peer=remote p1)");

                match RollbackSession::new_p2p(
                    session_config,
                    waiting.socket,
                    players,
                    specs.ram_limit,
                ) {
                    Ok(session) => {
                        tracing::info!(
                            "Host mode: session created, local_players = {:?}",
                            session.local_players()
                        );
                        if let Err(e) = runner.load_game_with_session(
                            rom.console.clone(),
                            &rom.code,
                            session,
                            None,
                            &rom.game_id,
                        ) {
                            tracing::error!("Failed to load game with P2P session: {}", e);
                            self.error_state = Some(super::super::GameError {
                                summary: "Connection Error".to_string(),
                                details: format!("Failed to start game: {}", e),
                                stack_trace: None,
                                tick: None,
                                phase: GameErrorPhase::Update,
                                suggestions: vec![],
                            });
                        } else {
                            tracing::info!("Host mode: game started with peer");
                            // Set audio volume
                            if let Some(session) = runner.session_mut()
                                && let Some(audio) = session.runtime.audio_mut()
                            {
                                let config = super::super::config::load();
                                audio.set_master_volume(config.audio.master_volume);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to create P2P session: {}", e);
                        self.error_state = Some(super::super::GameError {
                            summary: "Connection Error".to_string(),
                            details: format!("Failed to create session: {}", e),
                            stack_trace: None,
                            tick: None,
                            phase: GameErrorPhase::Update,
                            suggestions: vec![],
                        });
                    }
                }
            }

            self.needs_redraw = true;
            return true;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use nethercore_shared::console::TickRate;
    use tempfile::NamedTempFile;

    use crate::net::nchs::{
        NetworkConfig, PlayerConnectionInfo, PlayerInfo, SaveConfig, SaveMode, SessionStart,
    };

    #[test]
    fn session_file_exposes_save_config() {
        let mut file = NamedTempFile::new().expect("create temp session file");

        let session_start = SessionStart {
            local_player_handle: 0,
            random_seed: 123,
            start_frame: 0,
            tick_rate: TickRate::Fixed60,
            players: vec![PlayerConnectionInfo {
                handle: 0,
                active: true,
                info: PlayerInfo::default(),
                addr: "127.0.0.1".to_string(),
                ggrs_port: 7000,
            }],
            player_count: 1,
            network_config: NetworkConfig::default(),
            save_config: Some(SaveConfig {
                slot_index: 0,
                mode: SaveMode::Synchronized,
                synchronized_save: Some(vec![1, 2, 3]),
            }),
            extra_data: Vec::new(),
        };

        let encoded = bitcode::encode(&session_start);
        file.write_all(&encoded)
            .and_then(|_| file.flush())
            .expect("write session file");

        let decoded = super::decode_session_file(file.path()).expect("decode session file");
        assert_eq!(decoded.save_config, session_start.save_config);
    }
}
