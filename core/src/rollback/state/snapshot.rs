//! Game state snapshot for rollback

use smallvec::SmallVec;

use super::host_state::{HOST_STATE_SIZE, HostRollbackState};
use super::{ConsoleDataVec, InputDataVec};

/// Snapshot of game state for rollback
///
/// Contains the serialized WASM game state, console-specific rollback data,
/// host-side rollback state, input state, and a checksum for desync detection.
/// The data comes from calling `GameInstance::save_state()` which snapshots
/// the entire WASM linear memory.
#[derive(Clone)]
pub struct GameStateSnapshot {
    /// Serialized WASM game state (entire linear memory)
    pub data: Vec<u8>,
    /// Console-specific rollback state (POD, serialized via bytemuck)
    /// Uses SmallVec to store inline (no heap allocation for typical console states)
    pub console_data: ConsoleDataVec,
    /// Input state (input_prev + input_curr) serialized via bytemuck
    /// Required for button_pressed() to work correctly after rollback
    pub input_data: InputDataVec,
    /// Host-side state (RNG, tick count, elapsed time) that must be rolled back
    pub host_state: HostRollbackState,
    /// Logical save slots are simulation state; disk commit timing is not.
    pub save_data: [Option<Vec<u8>>; crate::wasm::MAX_SAVE_SLOTS],
    /// xxHash3 checksum for desync detection (shared simulation state)
    pub checksum: u64,
    /// Frame number this snapshot was taken at
    pub frame: i32,
}

impl GameStateSnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            console_data: SmallVec::new(),
            input_data: SmallVec::new(),
            host_state: HostRollbackState::default(),
            save_data: Default::default(),
            checksum: 0,
            frame: -1,
        }
    }

    /// Create a snapshot from serialized data (no console, input, or host data)
    pub fn from_data(data: Vec<u8>, frame: i32) -> Self {
        let host_state = HostRollbackState::default();
        let checksum = Self::compute_checksum(&data, &[], &[], &host_state);
        Self {
            data,
            console_data: SmallVec::new(),
            input_data: SmallVec::new(),
            host_state,
            save_data: Default::default(),
            checksum,
            frame,
        }
    }

    /// Create a complete snapshot with all rollback state
    pub fn from_full_state(
        data: Vec<u8>,
        console_data: ConsoleDataVec,
        input_data: InputDataVec,
        host_state: HostRollbackState,
        frame: i32,
    ) -> Self {
        let checksum = Self::compute_checksum(&data, &console_data, &input_data, &host_state);
        Self {
            data,
            console_data,
            input_data,
            host_state,
            save_data: Default::default(),
            checksum,
            frame,
        }
    }

    /// Create a snapshot from a pre-allocated buffer (avoids allocation)
    pub fn from_buffer(buffer: &mut Vec<u8>, len: usize, frame: i32) -> Self {
        buffer.truncate(len);
        let host_state = HostRollbackState::default();
        let checksum = Self::compute_checksum(buffer, &[], &[], &host_state);
        Self {
            data: std::mem::take(buffer),
            console_data: SmallVec::new(),
            input_data: SmallVec::new(),
            host_state,
            save_data: Default::default(),
            checksum,
            frame,
        }
    }

    pub fn with_save_data(mut self, saves: [Option<Vec<u8>>; crate::wasm::MAX_SAVE_SLOTS]) -> Self {
        self.save_data = saves;
        // Controller saves can differ between peers even when the game never reads
        // them. Restore them locally, but hash shared simulation state only.
        self
    }

    /// Check if this snapshot is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the size of the serialized WASM state in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get total snapshot size including all state
    pub fn total_len(&self) -> usize {
        self.data.len()
            + self.console_data.len()
            + self.input_data.len()
            + HOST_STATE_SIZE
            + self.save_data.iter().flatten().map(Vec::len).sum::<usize>()
    }

    /// Compute xxHash3 checksum for desync detection
    ///
    /// xxHash3 is SIMD-optimized (~50 GB/s throughput) for fast checksumming
    /// of large state buffers. We use this to detect desyncs between clients.
    /// Checksum covers WASM memory, console rollback state, input state, and host state.
    fn compute_checksum(
        data: &[u8],
        console_data: &[u8],
        input_data: &[u8],
        host_state: &HostRollbackState,
    ) -> u64 {
        use xxhash_rust::xxh3::Xxh3;
        let mut hasher = Xxh3::new();
        hasher.update(data);
        hasher.update(console_data);
        hasher.update(input_data);
        hasher.update(bytemuck::bytes_of(host_state));
        hasher.digest()
    }
}

impl Default for GameStateSnapshot {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod local_save_tests {
    use super::GameStateSnapshot;

    #[test]
    fn shipping_local_saves_restore_without_false_peer_desync() {
        let mut a: [Option<Vec<u8>>; crate::wasm::MAX_SAVE_SLOTS] = Default::default();
        let mut b: [Option<Vec<u8>>; crate::wasm::MAX_SAVE_SLOTS] = Default::default();
        a[0] = Some(vec![1]);
        b[1] = Some(vec![2]);
        let left = GameStateSnapshot::from_data(vec![42], 1).with_save_data(a);
        let right = GameStateSnapshot::from_data(vec![42], 1).with_save_data(b);
        assert_eq!(
            left.checksum, right.checksum,
            "per-controller disk data is local, not peer identity"
        );
        assert_eq!(left.save_data[0].as_deref(), Some(&[1][..]));
        assert_eq!(right.save_data[1].as_deref(), Some(&[2][..]));
    }
}
