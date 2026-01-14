//! Tests for NCHS protocol message serialization

use nethercore_shared::console::{ConsoleType, TickRate};

use super::super::{
    GuestReady, JoinAccept, JoinReject, JoinRejectReason, JoinRequest, LobbyState, LobbyUpdate,
    NchsDecodeError, NchsMessage, NetworkConfig, PlayerConnectionInfo, PlayerInfo, PlayerSlot,
    PunchAck, PunchHello, SaveConfig, SaveMode, SessionStart,
};

#[test]
fn test_join_request_roundtrip() {
    let msg = NchsMessage::JoinRequest(JoinRequest {
        console_type: ConsoleType::ZX,
        rom_hash: 0xDEADBEEF12345678,
        tick_rate: TickRate::Fixed60,
        max_players: 4,
        player_info: PlayerInfo {
            name: "TestPlayer".to_string(),
            avatar_id: 42,
            color: [255, 128, 0],
        },
        local_addr: "192.168.1.50:7770".to_string(),
        extra_data: vec![1, 2, 3],
    });

    let bytes = msg.to_bytes();
    let decoded = NchsMessage::from_bytes(&bytes).unwrap();
    assert_eq!(msg, decoded);
}

#[test]
fn test_session_start_roundtrip() {
    let msg = NchsMessage::SessionStart(SessionStart {
        local_player_handle: 0,
        random_seed: 0x123456789ABCDEF0,
        start_frame: 0,
        tick_rate: TickRate::Fixed60,
        players: vec![
            PlayerConnectionInfo {
                handle: 0,
                active: true,
                info: PlayerInfo::default(),
                addr: "192.168.1.50:7770".to_string(),
                ggrs_port: 7771,
            },
            PlayerConnectionInfo {
                handle: 1,
                active: true,
                info: PlayerInfo {
                    name: "Player2".to_string(),
                    avatar_id: 1,
                    color: [0, 255, 0],
                },
                addr: "192.168.1.51:7770".to_string(),
                ggrs_port: 7771,
            },
        ],
        player_count: 2,
        network_config: NetworkConfig::default(),
        save_config: Some(SaveConfig {
            slot_index: 0,
            mode: SaveMode::Synchronized,
            synchronized_save: Some(vec![1, 2, 3, 4]),
        }),
        extra_data: vec![],
    });

    let bytes = msg.to_bytes();
    let decoded = NchsMessage::from_bytes(&bytes).unwrap();
    assert_eq!(msg, decoded);
}

#[test]
fn test_ping_pong_roundtrip() {
    let ping = NchsMessage::Ping;
    let pong = NchsMessage::Pong;

    let ping_bytes = ping.to_bytes();
    let pong_bytes = pong.to_bytes();

    assert_eq!(NchsMessage::from_bytes(&ping_bytes).unwrap(), ping);
    assert_eq!(NchsMessage::from_bytes(&pong_bytes).unwrap(), pong);
}

#[test]
fn test_invalid_magic() {
    let bytes = [b'X', b'X', b'X', b'X', 0, 0, 0, 0, 0, 0];
    let result = NchsMessage::from_bytes(&bytes);
    assert!(matches!(result, Err(NchsDecodeError::InvalidMagic)));
}

#[test]
fn test_version_mismatch() {
    let mut bytes = NchsMessage::Ping.to_bytes();
    bytes[4] = 99; // Set version to 99
    bytes[5] = 0;
    let result = NchsMessage::from_bytes(&bytes);
    assert!(matches!(
        result,
        Err(NchsDecodeError::VersionMismatch {
            expected: 1,
            got: 99
        })
    ));
}

#[test]
fn test_too_short() {
    let bytes = [b'N', b'C', b'H', b'S'];
    let result = NchsMessage::from_bytes(&bytes);
    assert!(matches!(result, Err(NchsDecodeError::TooShort)));
}

#[test]
fn test_join_reject_roundtrip() {
    let msg = NchsMessage::JoinReject(JoinReject {
        reason: JoinRejectReason::RomHashMismatch,
        message: Some("You have a different version of the game".to_string()),
    });

    let bytes = msg.to_bytes();
    let decoded = NchsMessage::from_bytes(&bytes).unwrap();
    assert_eq!(msg, decoded);
}

#[test]
fn test_lobby_state_roundtrip() {
    let msg = NchsMessage::LobbyUpdate(LobbyUpdate {
        lobby: LobbyState {
            players: vec![
                PlayerSlot {
                    handle: 0,
                    active: true,
                    info: Some(PlayerInfo::default()),
                    ready: true,
                    addr: Some("192.168.1.50:7770".to_string()),
                },
                PlayerSlot {
                    handle: 1,
                    active: false,
                    info: None,
                    ready: false,
                    addr: None,
                },
            ],
            max_players: 4,
            host_handle: 0,
        },
    });

    let bytes = msg.to_bytes();
    let decoded = NchsMessage::from_bytes(&bytes).unwrap();
    assert_eq!(msg, decoded);
}

#[test]
fn test_punch_messages_roundtrip() {
    let hello = NchsMessage::PunchHello(PunchHello {
        sender_handle: 1,
        nonce: 0xCAFEBABE,
    });
    let ack = NchsMessage::PunchAck(PunchAck {
        sender_handle: 2,
        nonce: 0xCAFEBABE,
    });

    let hello_bytes = hello.to_bytes();
    let ack_bytes = ack.to_bytes();

    assert_eq!(NchsMessage::from_bytes(&hello_bytes).unwrap(), hello);
    assert_eq!(NchsMessage::from_bytes(&ack_bytes).unwrap(), ack);
}

#[test]
fn test_guest_ready_roundtrip() {
    let msg = NchsMessage::GuestReady(GuestReady { ready: true });

    let bytes = msg.to_bytes();
    let decoded = NchsMessage::from_bytes(&bytes).unwrap();
    assert_eq!(msg, decoded);
}

#[test]
fn test_join_accept_roundtrip() {
    let msg = NchsMessage::JoinAccept(JoinAccept {
        player_handle: 1,
        lobby: LobbyState {
            players: vec![PlayerSlot {
                handle: 0,
                active: true,
                info: Some(PlayerInfo::default()),
                ready: true,
                addr: Some("192.168.1.50:7770".to_string()),
            }],
            max_players: 4,
            host_handle: 0,
        },
    });

    let bytes = msg.to_bytes();
    let decoded = NchsMessage::from_bytes(&bytes).unwrap();
    assert_eq!(msg, decoded);
}
