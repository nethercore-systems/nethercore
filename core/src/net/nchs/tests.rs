//! Integration tests for NCHS sessions

#![cfg(test)]

use super::*;
use crate::net::nchs::{JoinRejectReason, NetworkConfig, PlayerInfo};
use nethercore_shared::console::{ConsoleType, TickRate};
use nethercore_shared::netplay::NetplayMetadata;
use std::thread;
use std::time::Duration;

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

fn test_config(name: &str) -> NchsConfig {
    NchsConfig {
        netplay: test_netplay(),
        player_info: test_player_info(name),
        network_config: NetworkConfig::default(),
        save_config: None,
    }
}

#[test]
fn test_host_session_create() {
    let config = test_config("Host");
    let session = NchsSession::host(0, config).unwrap();

    assert_eq!(session.role(), NchsRole::Host);
    assert_eq!(session.local_handle(), Some(0));
    assert!(session.port() > 0);
}

#[test]
fn test_guest_session_create() {
    // First create a host so we have a valid port
    let host_config = test_config("Host");
    let host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Now create a guest connecting to that host
    let guest_config = test_config("Guest");
    let guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    assert_eq!(guest.role(), NchsRole::Guest);
    assert_eq!(guest.state(), NchsState::Connecting);
}

#[test]
fn test_host_guest_handshake() {
    // Create host
    let host_config = test_config("Host");
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Create guest connecting to host
    let guest_config = test_config("Guest");
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    // Poll both until guest is accepted or timeout
    let mut guest_accepted = false;
    for _ in 0..100 {
        // Poll host
        match host.poll() {
            NchsEvent::PlayerJoined { handle, .. } => {
                tracing::info!("Host: Player {} joined", handle);
            }
            _ => {}
        }

        // Poll guest
        match guest.poll() {
            NchsEvent::LobbyUpdated(lobby) => {
                tracing::info!("Guest: Lobby updated, {} players", lobby.players.len());
                guest_accepted = true;
                break;
            }
            NchsEvent::PlayerJoined { handle, .. } => {
                tracing::info!("Guest: Accepted as player {}", handle);
                guest_accepted = true;
                break;
            }
            NchsEvent::Error(e) => {
                panic!("Guest error: {:?}", e);
            }
            _ => {}
        }

        thread::sleep(Duration::from_millis(10));
    }

    assert!(guest_accepted, "Guest should have been accepted");
    assert!(guest.local_handle().is_some(), "Guest should have a handle");
    assert_eq!(host.player_count(), 2, "Should have 2 players");
}

#[test]
fn test_host_guest_ready_and_start() {
    // Create host
    let host_config = test_config("Host");
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Create guest
    let guest_config = test_config("Guest");
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    // Wait for guest to join
    let mut joined = false;
    for _ in 0..100 {
        host.poll();
        match guest.poll() {
            NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => {
                joined = true;
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(joined, "Guest should have joined");

    // Guest sets ready
    guest.set_ready(true).unwrap();

    // Wait for host to see guest ready
    let mut all_ready = false;
    for _ in 0..100 {
        match host.poll() {
            NchsEvent::LobbyUpdated(lobby) => {
                let guests_ready = lobby
                    .players
                    .iter()
                    .filter(|p| p.active && p.handle != 0)
                    .all(|p| p.ready);
                if guests_ready {
                    all_ready = true;
                    break;
                }
            }
            _ => {}
        }
        guest.poll(); // Keep guest alive
        thread::sleep(Duration::from_millis(10));
    }
    assert!(all_ready, "All players should be ready");

    // Host starts the session
    let session_start = host.start().expect("Host should be able to start");
    assert!(session_start.random_seed != 0, "Should have random seed");
    assert_eq!(session_start.player_count, 2, "Should have 2 players");

    // Guest should receive session start
    let mut guest_ready = false;
    for _ in 0..100 {
        match guest.poll() {
            NchsEvent::Ready(ss) => {
                assert_eq!(
                    ss.random_seed, session_start.random_seed,
                    "Seeds should match"
                );
                guest_ready = true;
                break;
            }
            NchsEvent::LobbyUpdated(_) => {
                // Session starting, keep polling
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(guest_ready, "Guest should receive Ready event");
}

#[test]
fn test_host_cannot_set_ready() {
    let config = test_config("Host");
    let mut host = NchsSession::host(0, config).unwrap();

    let result = host.set_ready(true);
    assert!(result.is_err());
}

#[test]
fn test_guest_cannot_start() {
    let host_config = test_config("Host");
    let host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    let guest_config = test_config("Guest");
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    let result = guest.start();
    assert!(result.is_err());
}

#[test]
fn test_rom_hash_mismatch_rejected() {
    // Host with one hash
    let host_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xAAAAAAAA),
        player_info: test_player_info("Host"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Guest with different hash
    let guest_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0xBBBBBBBB),
        player_info: test_player_info("Guest"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    // Poll until rejection or timeout
    let mut rejected = false;
    let mut reject_reason = None;
    for _ in 0..100 {
        host.poll();
        match guest.poll() {
            NchsEvent::Error(NchsError::Rejected(reject)) => {
                rejected = true;
                reject_reason = Some(reject.reason);
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(rejected, "Guest should be rejected for ROM hash mismatch");
    assert_eq!(reject_reason, Some(JoinRejectReason::RomHashMismatch));
}

#[test]
fn test_console_type_mismatch_rejected() {
    // Host with ZX
    let host_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678),
        player_info: test_player_info("Host"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Guest with Chroma
    let guest_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::Chroma, TickRate::Fixed60, 4, 0x12345678),
        player_info: test_player_info("Guest"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    // Poll until rejection or timeout
    let mut rejected = false;
    let mut reject_reason = None;
    for _ in 0..100 {
        host.poll();
        match guest.poll() {
            NchsEvent::Error(NchsError::Rejected(reject)) => {
                rejected = true;
                reject_reason = Some(reject.reason);
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(
        rejected,
        "Guest should be rejected for console type mismatch"
    );
    assert_eq!(reject_reason, Some(JoinRejectReason::ConsoleTypeMismatch));
}

#[test]
fn test_tick_rate_mismatch_rejected() {
    // Host with 60Hz
    let host_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 4, 0x12345678),
        player_info: test_player_info("Host"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Guest with 120Hz
    let guest_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed120, 4, 0x12345678),
        player_info: test_player_info("Guest"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    // Poll until rejection or timeout
    let mut rejected = false;
    let mut reject_reason = None;
    for _ in 0..100 {
        host.poll();
        match guest.poll() {
            NchsEvent::Error(NchsError::Rejected(reject)) => {
                rejected = true;
                reject_reason = Some(reject.reason);
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(rejected, "Guest should be rejected for tick rate mismatch");
    assert_eq!(reject_reason, Some(JoinRejectReason::TickRateMismatch));
}

#[test]
fn test_lobby_full_rejected() {
    // Host with max 2 players
    let host_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 2, 0x12345678),
        player_info: test_player_info("Host"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // First guest joins successfully
    let guest1_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 2, 0x12345678),
        player_info: test_player_info("Guest1"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut guest1 = NchsSession::join(&format!("127.0.0.1:{}", port), guest1_config).unwrap();

    // Wait for guest1 to join
    let mut guest1_joined = false;
    for _ in 0..100 {
        host.poll();
        match guest1.poll() {
            NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => {
                guest1_joined = true;
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert!(guest1_joined, "Guest1 should join");

    // Second guest should be rejected (lobby full: host + guest1 = 2)
    let guest2_config = NchsConfig {
        netplay: NetplayMetadata::new(ConsoleType::ZX, TickRate::Fixed60, 2, 0x12345678),
        player_info: test_player_info("Guest2"),
        network_config: NetworkConfig::default(),
        save_config: None,
    };
    let mut guest2 = NchsSession::join(&format!("127.0.0.1:{}", port), guest2_config).unwrap();

    let mut rejected = false;
    let mut reject_reason = None;
    for _ in 0..100 {
        host.poll();
        guest1.poll(); // Keep guest1 alive
        match guest2.poll() {
            NchsEvent::Error(NchsError::Rejected(reject)) => {
                rejected = true;
                reject_reason = Some(reject.reason);
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(rejected, "Guest2 should be rejected because lobby is full");
    assert_eq!(reject_reason, Some(JoinRejectReason::LobbyFull));
}

#[test]
fn test_join_while_game_in_progress_rejected() {
    // Create host and first guest
    let host_config = test_config("Host");
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    let guest1_config = test_config("Guest1");
    let mut guest1 = NchsSession::join(&format!("127.0.0.1:{}", port), guest1_config).unwrap();

    // Wait for guest1 to join
    for _ in 0..100 {
        host.poll();
        match guest1.poll() {
            NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => break,
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Guest1 sets ready
    guest1.set_ready(true).unwrap();

    // Wait for host to see ready
    for _ in 0..100 {
        if host.all_ready() && host.player_count() > 1 {
            break;
        }
        host.poll();
        guest1.poll();
        thread::sleep(Duration::from_millis(10));
    }

    // Host starts the game
    host.start().expect("Should be able to start");

    // Now try to join with a new guest
    let guest2_config = test_config("Guest2");
    let mut guest2 = NchsSession::join(&format!("127.0.0.1:{}", port), guest2_config).unwrap();

    let mut rejected = false;
    let mut reject_reason = None;
    for _ in 0..100 {
        host.poll();
        guest1.poll();
        match guest2.poll() {
            NchsEvent::Error(NchsError::Rejected(reject)) => {
                rejected = true;
                reject_reason = Some(reject.reason);
                break;
            }
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(
        rejected,
        "Guest2 should be rejected because game is in progress"
    );
    assert_eq!(reject_reason, Some(JoinRejectReason::GameInProgress));
}

#[test]
fn test_session_start_has_real_ip() {
    // Create host
    let host_config = test_config("Host");
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Create guest
    let guest_config = test_config("Guest");
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    // Wait for guest to join
    for _ in 0..100 {
        host.poll();
        match guest.poll() {
            NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => break,
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Guest sets ready
    guest.set_ready(true).unwrap();

    // Wait for host to see guest ready
    for _ in 0..100 {
        if host.all_ready() && host.player_count() > 1 {
            break;
        }
        host.poll();
        guest.poll();
        thread::sleep(Duration::from_millis(10));
    }

    // Host starts the session
    let session_start = host.start().expect("Host should be able to start");

    // Verify host's address in SessionStart is not 0.0.0.0
    let host_player = &session_start.players[0];
    assert!(host_player.active, "Host should be active");
    assert!(
        !host_player.addr.starts_with("0.0.0.0"),
        "Host address in SessionStart should not be 0.0.0.0, got: {}",
        host_player.addr
    );
    // Should be localhost for local test
    assert!(
        host_player.addr.starts_with("127.0.0.1") || !host_player.addr.is_empty(),
        "Host address should be a valid IP, got: {}",
        host_player.addr
    );
}

#[test]
fn test_lobby_state_has_real_host_ip() {
    // Create host
    let host_config = test_config("Host");
    let host = NchsSession::host(0, host_config).unwrap();

    // Get lobby state
    let lobby = host.lobby().expect("Host should have lobby state");

    // Verify host's address is not 0.0.0.0
    let host_slot = &lobby.players[0];
    assert!(host_slot.active, "Host slot should be active");
    let addr = host_slot
        .addr
        .as_ref()
        .expect("Host should have an address");
    assert!(
        !addr.starts_with("0.0.0.0"),
        "Host address in lobby should not be 0.0.0.0, got: {}",
        addr
    );
}

#[test]
fn test_host_emits_ready_after_start() {
    // This test verifies that the host emits NchsEvent::Ready after start() is called.
    // Bug: Previously, the host never emitted Ready, causing the library to never
    // spawn the player process for the host.

    // Create host
    let host_config = test_config("Host");
    let mut host = NchsSession::host(0, host_config).unwrap();
    let port = host.port();

    // Create guest
    let guest_config = test_config("Guest");
    let mut guest = NchsSession::join(&format!("127.0.0.1:{}", port), guest_config).unwrap();

    // Wait for guest to join
    for _ in 0..100 {
        host.poll();
        match guest.poll() {
            NchsEvent::LobbyUpdated(_) | NchsEvent::PlayerJoined { .. } => break,
            _ => {}
        }
        thread::sleep(Duration::from_millis(10));
    }

    // Guest sets ready
    guest.set_ready(true).unwrap();

    // Wait for host to see guest ready
    for _ in 0..100 {
        if host.all_ready() && host.player_count() > 1 {
            break;
        }
        host.poll();
        guest.poll();
        thread::sleep(Duration::from_millis(10));
    }

    assert!(host.all_ready(), "All players should be ready before start");
    assert!(host.player_count() >= 2, "Should have at least 2 players");

    // Host starts the session
    let _session_start = host.start().expect("Host should be able to start");

    // CRITICAL: Host should emit Ready event on the next poll
    // This is the bug - previously the host never emitted Ready
    let mut host_ready = false;
    for _ in 0..10 {
        match host.poll() {
            NchsEvent::Ready(ss) => {
                // Verify the session start info is correct
                assert!(ss.random_seed != 0, "Should have random seed");
                assert_eq!(ss.player_count, 2, "Should have 2 players");
                host_ready = true;
                break;
            }
            NchsEvent::Pending => {
                // Give it a few more tries
            }
            other => {
                panic!("Unexpected event from host after start: {:?}", other);
            }
        }
        thread::sleep(Duration::from_millis(10));
    }

    assert!(
        host_ready,
        "Host should emit NchsEvent::Ready after start() - this is required for the library to spawn the player process"
    );

    // Also verify state is Ready
    assert_eq!(
        host.state(),
        NchsState::Ready,
        "Host state should be Ready after emitting Ready event"
    );
}
