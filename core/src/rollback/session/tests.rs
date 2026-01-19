//! Tests for rollback session

use bytemuck::{Pod, Zeroable};
use ggrs::{GgrsRequest, InputStatus};

use crate::console::{Console, ConsoleInput};
use crate::rollback::ConnectionQuality;
use crate::test_utils::TestConsole;

use super::RollbackSession;
use super::types::{NetworkInput, SessionType};
use crate::rollback::config::SessionConfig;
use crate::rollback::events::{PlayerNetworkStats, SessionError, SessionEvent};
use crate::rollback::player::PlayerSessionConfig;

// Test input type for unit tests
#[repr(C)]
#[derive(
    Clone, Copy, Default, PartialEq, Debug, Pod, Zeroable, serde::Serialize, serde::Deserialize,
)]
struct TestInput {
    buttons: u16,
    x: i8,
    y: i8,
}
impl ConsoleInput for TestInput {}

fn test_ram_limit() -> usize {
    TestConsole::specs().ram_limit
}

#[test]
fn test_rollback_session_local() {
    let session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    assert_eq!(session.session_type(), SessionType::Local);
    assert_eq!(session.config().num_players, 2);
    assert_eq!(session.current_frame(), 0);
    assert_eq!(session.local_players(), &[0, 1]);
}

#[test]
fn test_rollback_session_sync_test() {
    let config = SessionConfig::sync_test();
    let session =
        RollbackSession::<TestInput, ()>::new_sync_test(config, test_ram_limit()).unwrap();
    assert_eq!(session.session_type(), SessionType::SyncTest);
}

#[test]
fn test_local_session_advance() {
    let mut session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    assert_eq!(session.current_frame(), 0);

    let requests = session.advance_frame().unwrap();
    assert_eq!(requests.len(), 1);

    match &requests[0] {
        GgrsRequest::AdvanceFrame { inputs } => {
            assert_eq!(inputs.len(), 2);
            for (input, status) in inputs {
                assert_eq!(*input, TestInput::default());
                assert_eq!(status, &InputStatus::Confirmed);
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
    let session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    assert!(session.all_player_stats().is_empty());
    assert!(session.player_stats(0).is_none());
}

#[test]
fn test_local_session_no_desync() {
    let session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    assert!(!session.has_desync());
}

#[test]
fn test_local_session_total_rollback_frames() {
    let session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
    assert_eq!(session.total_rollback_frames(), 0);
}

#[test]
fn test_local_session_handle_events_empty() {
    let mut session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
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
    let session = RollbackSession::<TestInput, ()>::new_local(2, test_ram_limit());
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
    let session =
        RollbackSession::<TestInput, ()>::new_local_with_config(player_config, test_ram_limit());

    assert_eq!(session.player_config().num_players(), 4);
    assert_eq!(session.player_config().local_player_mask(), 0b0011);
    assert_eq!(session.local_players(), &[0, 1]);
}

#[test]
fn test_rollback_session_sync_test_has_player_config() {
    let config = SessionConfig::sync_test();
    let session =
        RollbackSession::<TestInput, ()>::new_sync_test(config, test_ram_limit()).unwrap();
    let player_config = session.player_config();
    assert_eq!(player_config.num_players(), 1);
    assert!(player_config.is_local_player(0));
}
