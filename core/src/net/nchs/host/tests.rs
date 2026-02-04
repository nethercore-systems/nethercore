//! Unit tests for host state machine

use nethercore_shared::console::{ConsoleType, TickRate};
use nethercore_shared::netplay::NetplayMetadata;

use crate::net::nchs::messages::{NetworkConfig, PlayerInfo, SaveConfig, SaveMode};

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
        None, // No save config
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
        None, // No save config
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
        None, // No save config
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
        None, // No save config
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
        None, // No save config
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
        None, // No save config
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

#[test]
fn test_host_save_config_synchronized() {
    // Test synchronized save mode where host's save data is shared with all players
    let save_data = vec![0x01, 0x02, 0x03, 0x04]; // Mock save data
    let save_config = SaveConfig {
        slot_index: 0,
        mode: SaveMode::Synchronized,
        synchronized_save: Some(save_data.clone()),
    };

    let host = HostStateMachine::new(
        0,
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
        Some(save_config),
    )
    .unwrap();

    // Verify save config is stored
    assert!(host.save_config.is_some());
    let config = host.save_config.as_ref().unwrap();
    assert_eq!(config.mode, SaveMode::Synchronized);
    assert_eq!(config.slot_index, 0);
    assert_eq!(config.synchronized_save, Some(save_data));
}

#[test]
fn test_host_save_config_per_player() {
    // Test per-player save mode where each player uses their own save
    let save_config = SaveConfig {
        slot_index: 1,
        mode: SaveMode::PerPlayer,
        synchronized_save: None,
    };

    let host = HostStateMachine::new(
        0,
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
        Some(save_config),
    )
    .unwrap();

    // Verify save config is stored
    assert!(host.save_config.is_some());
    let config = host.save_config.as_ref().unwrap();
    assert_eq!(config.mode, SaveMode::PerPlayer);
    assert_eq!(config.slot_index, 1);
    assert!(config.synchronized_save.is_none());
}

#[test]
fn test_host_save_config_new_game() {
    // Test new game mode where no save data is used
    let save_config = SaveConfig {
        slot_index: 0,
        mode: SaveMode::NewGame,
        synchronized_save: None,
    };

    let host = HostStateMachine::new(
        0,
        test_netplay(),
        test_player_info("Host"),
        NetworkConfig::default(),
        Some(save_config),
    )
    .unwrap();

    // Verify save config is stored
    assert!(host.save_config.is_some());
    let config = host.save_config.as_ref().unwrap();
    assert_eq!(config.mode, SaveMode::NewGame);
}
