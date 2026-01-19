//! Rollback state management
//!
//! Provides state snapshot and buffer pool functionality for GGRS rollback.

use smallvec::SmallVec;

mod host_state;
mod manager;
mod pool;
mod snapshot;

// Re-export public types
pub use host_state::{HOST_STATE_SIZE, HostRollbackState};
pub use manager::{LoadStateError, RollbackStateManager, SaveStateError};
pub use pool::StatePool;
pub use snapshot::GameStateSnapshot;

// Type aliases and constants

/// Inline storage size for console rollback state (avoids heap allocation)
/// 512 bytes covers Nethercore ZX's 340-byte AudioPlaybackState with room to spare
pub type ConsoleDataVec = SmallVec<[u8; 512]>;

/// Inline storage size for input state (avoids heap allocation)
/// 128 bytes covers ZInput (8 bytes) ×4 players ×2 (prev+curr) = 64 bytes with room to spare
pub type InputDataVec = SmallVec<[u8; 128]>;

/// Number of pre-allocated state buffers in the pool
pub const STATE_POOL_SIZE: usize = super::config::MAX_ROLLBACK_FRAMES + 2;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_state_snapshot_empty() {
        let snapshot = GameStateSnapshot::new();
        assert!(snapshot.is_empty());
        assert_eq!(snapshot.len(), 0);
        assert_eq!(snapshot.frame, -1);
    }

    #[test]
    fn test_game_state_snapshot_from_data() {
        let data = vec![1, 2, 3, 4, 5];
        let snapshot = GameStateSnapshot::from_data(data.clone(), 42);
        assert!(!snapshot.is_empty());
        assert_eq!(snapshot.len(), 5);
        assert_eq!(snapshot.frame, 42);
        assert_eq!(snapshot.data, data);
        // Checksum should be non-zero for non-empty data
        assert_ne!(snapshot.checksum, 0);
    }

    #[test]
    fn test_game_state_snapshot_checksum_deterministic() {
        let data = vec![1, 2, 3, 4, 5];
        let snapshot1 = GameStateSnapshot::from_data(data.clone(), 0);
        let snapshot2 = GameStateSnapshot::from_data(data, 0);
        assert_eq!(snapshot1.checksum, snapshot2.checksum);
    }

    #[test]
    fn test_game_state_snapshot_checksum_different_data() {
        let snapshot1 = GameStateSnapshot::from_data(vec![1, 2, 3], 0);
        let snapshot2 = GameStateSnapshot::from_data(vec![4, 5, 6], 0);
        assert_ne!(snapshot1.checksum, snapshot2.checksum);
    }

    #[test]
    fn test_state_pool_acquire_release() {
        let mut pool = StatePool::new(1024, 3);
        assert_eq!(pool.available(), 3);

        let buf1 = pool.acquire();
        assert_eq!(pool.available(), 2);
        assert!(buf1.capacity() >= 1024);

        let buf2 = pool.acquire();
        assert_eq!(pool.available(), 1);

        pool.release(buf1);
        assert_eq!(pool.available(), 2);

        pool.release(buf2);
        assert_eq!(pool.available(), 3);
    }

    #[test]
    fn test_state_pool_exhaustion() {
        let mut pool = StatePool::new(1024, 1);
        let _buf1 = pool.acquire();
        // Pool should allocate a new buffer when exhausted
        let buf2 = pool.acquire();
        assert!(buf2.capacity() >= 1024);
    }

    #[test]
    fn test_host_rollback_state() {
        let host_state = HostRollbackState::new(12345, 100, 1.5);
        assert_eq!(host_state.rng_state, 12345);
        assert_eq!(host_state.tick_count, 100);
        assert_eq!(host_state.elapsed_time(), 1.5);
    }

    #[test]
    fn test_host_rollback_state_serialization() {
        let host_state = HostRollbackState::new(0xDEADBEEF, 42, 2.5);
        let bytes = bytemuck::bytes_of(&host_state);
        assert_eq!(bytes.len(), HOST_STATE_SIZE);

        let restored: &HostRollbackState = bytemuck::from_bytes(bytes);
        assert_eq!(restored.rng_state, host_state.rng_state);
        assert_eq!(restored.tick_count, host_state.tick_count);
        assert_eq!(restored.elapsed_time_bits, host_state.elapsed_time_bits);
    }

    #[test]
    fn test_snapshot_with_host_state() {
        let data = vec![1, 2, 3, 4, 5];
        let console_data = SmallVec::new();
        let input_data = SmallVec::new();
        let host_state = HostRollbackState::new(999, 50, 2.5);

        let snapshot = GameStateSnapshot::from_full_state(
            data.clone(),
            console_data,
            input_data,
            host_state,
            10,
        );

        assert_eq!(snapshot.host_state.rng_state, 999);
        assert_eq!(snapshot.host_state.tick_count, 50);
        assert_eq!(snapshot.host_state.elapsed_time(), 2.5);
        assert_eq!(snapshot.frame, 10);
    }

    #[test]
    fn test_snapshot_checksum_includes_host_state() {
        let data = vec![1, 2, 3];
        let console_data = SmallVec::new();
        let input_data = SmallVec::new();

        // Same data but different host state should produce different checksums
        let host1 = HostRollbackState::new(100, 1, 1.0);
        let host2 = HostRollbackState::new(200, 2, 2.0);

        let snapshot1 = GameStateSnapshot::from_full_state(
            data.clone(),
            console_data.clone(),
            input_data.clone(),
            host1,
            0,
        );
        let snapshot2 =
            GameStateSnapshot::from_full_state(data, console_data, input_data, host2, 0);

        assert_ne!(snapshot1.checksum, snapshot2.checksum);
    }

    #[test]
    fn test_snapshot_checksum_includes_input_state() {
        let data = vec![1, 2, 3];
        let console_data = SmallVec::new();
        let host_state = HostRollbackState::default();

        // Same data but different input state should produce different checksums
        let input1: InputDataVec = SmallVec::from_slice(&[1, 2, 3, 4]);
        let input2: InputDataVec = SmallVec::from_slice(&[5, 6, 7, 8]);

        let snapshot1 = GameStateSnapshot::from_full_state(
            data.clone(),
            console_data.clone(),
            input1,
            host_state,
            0,
        );
        let snapshot2 =
            GameStateSnapshot::from_full_state(data, console_data, input2, host_state, 0);

        assert_ne!(snapshot1.checksum, snapshot2.checksum);
    }
}
