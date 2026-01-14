//! Unit tests for host state machine

use nethercore_shared::console::{ConsoleType, TickRate};
use nethercore_shared::netplay::NetplayMetadata;

use crate::net::nchs::messages::{NetworkConfig, PlayerInfo};

use super::state::{HostState, HostStateMachine};

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

#[test]
fn test_host_create() {
    let host = HostStateMachine::new(
        0, // Let OS assign port
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
    )
    .unwrap();

    assert_eq!(host.state(), HostState::Listening);
    assert!(host.port() > 0);
    assert_eq!(host.player_count(), 1); // Just the host
}

#[test]
fn test_host_lobby_state() {
    let host = HostStateMachine::new(
        0,
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
    )
    .unwrap();

    let lobby = host.lobby_state();
    assert_eq!(lobby.players.len(), 4); // max_players slots
    assert!(lobby.players[0].active); // Host
    assert!(!lobby.players[1].active); // Empty slot
    assert_eq!(lobby.host_handle, 0);
}

#[test]
fn test_host_all_ready_empty() {
    let host = HostStateMachine::new(
        0,
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
    )
    .unwrap();

    // With no other players, "all ready" is true (vacuously)
    assert!(host.all_ready());
}

#[test]
fn test_host_is_full() {
    let mut netplay = test_netplay();
    netplay.max_players = 1; // Only host

    let host = HostStateMachine::new(
        0,
        netplay,
        test_player_info("Host"),
        NetworkConfig::default(),
    )
    .unwrap();

    assert!(host.is_full());
}

#[test]
fn test_host_public_addr_not_zero() {
    let host = HostStateMachine::new(
        0,
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
    )
    .unwrap();

    // Public address should not start with 0.0.0.0
    assert!(
        !host.public_addr.starts_with("0.0.0.0"),
        "public_addr should not be 0.0.0.0, got: {}",
        host.public_addr
    );
}

#[test]
fn test_host_lobby_state_has_real_ip() {
    let host = HostStateMachine::new(
        0,
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
    )
    .unwrap();

    let lobby = host.lobby_state();
    let host_slot = &lobby.players[0];

    // Host slot should have a real IP address, not 0.0.0.0
    assert!(host_slot.addr.is_some());
    let addr = host_slot.addr.as_ref().unwrap();
    assert!(
        !addr.starts_with("0.0.0.0"),
        "Host address in lobby should not be 0.0.0.0, got: {}",
        addr
    );
}
