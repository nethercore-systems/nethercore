//! Rollback state management
//!
//! Provides state snapshot and buffer pool functionality for GGRS rollback.

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::wasm::GameInstance;
use smallvec::SmallVec;

use super::config::MAX_STATE_SIZE;

// ============================================================================
// Host Rollback State
// ============================================================================

/// Size of HostRollbackState in bytes (for inline storage)
pub const HOST_STATE_SIZE: usize = std::mem::size_of::<HostRollbackState>();

/// Host-side state that must be rolled back for determinism
///
/// This state lives on the host (not in WASM memory) but affects game
/// simulation and must be restored during rollback.
#[derive(Clone, Copy, Default, Debug, PartialEq)]
#[repr(C)]
pub struct HostRollbackState {
    /// RNG state for deterministic random numbers
    pub rng_state: u64,
    /// Current tick count
    pub tick_count: u64,
    /// Elapsed time in seconds (f32 stored as bits for Pod compatibility)
    pub elapsed_time_bits: u32,
    /// Padding for alignment
    _padding: u32,
}

// SAFETY: HostRollbackState is #[repr(C)] with only primitive types
unsafe impl bytemuck::Zeroable for HostRollbackState {}
unsafe impl bytemuck::Pod for HostRollbackState {}

impl HostRollbackState {
    /// Create from game state values
    pub fn new(rng_state: u64, tick_count: u64, elapsed_time: f32) -> Self {
        Self {
            rng_state,
            tick_count,
            elapsed_time_bits: elapsed_time.to_bits(),
            _padding: 0,
        }
    }

    /// Get elapsed time as f32
    pub fn elapsed_time(&self) -> f32 {
        f32::from_bits(self.elapsed_time_bits)
    }
}

/// Inline storage size for console rollback state (avoids heap allocation)
/// 512 bytes covers Nethercore ZX's 340-byte AudioPlaybackState with room to spare
pub type ConsoleDataVec = SmallVec<[u8; 512]>;

/// Inline storage size for input state (avoids heap allocation)
/// 128 bytes covers ZInput (8 bytes) × 4 players × 2 (prev+curr) = 64 bytes with room to spare
pub type InputDataVec = SmallVec<[u8; 128]>;

/// Number of pre-allocated state buffers in the pool
pub const STATE_POOL_SIZE: usize = super::config::MAX_ROLLBACK_FRAMES + 2;

// ============================================================================
// Game State Snapshot
// ============================================================================

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
    /// xxHash3 checksum for desync detection (covers all state)
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
            checksum,
            frame,
        }
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
        self.data.len() + self.console_data.len() + self.input_data.len() + HOST_STATE_SIZE
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

// ============================================================================
// State Buffer Pool
// ============================================================================

/// Pre-allocated buffer pool for rollback state saves
///
/// Avoids allocations in the hot path during rollback. GGRS may need to
/// save/load state multiple times per frame during rollback, so this is
/// critical for performance.
pub struct StatePool {
    /// Pool of reusable buffers
    buffers: Vec<Vec<u8>>,
    /// Size each buffer was pre-allocated to
    buffer_size: usize,
}

impl StatePool {
    /// Create a new state pool with pre-allocated buffers
    pub fn new(buffer_size: usize, pool_size: usize) -> Self {
        let buffers = (0..pool_size)
            .map(|_| Vec::with_capacity(buffer_size))
            .collect();
        Self {
            buffers,
            buffer_size,
        }
    }

    /// Create a pool with default settings
    pub fn with_defaults() -> Self {
        Self::new(MAX_STATE_SIZE, STATE_POOL_SIZE)
    }

    /// Acquire a buffer from the pool
    ///
    /// Returns a buffer with capacity >= buffer_size.
    /// If pool is empty, allocates a new buffer (should be rare in steady state).
    pub fn acquire(&mut self) -> Vec<u8> {
        self.buffers.pop().unwrap_or_else(|| {
            log::warn!("StatePool exhausted, allocating new buffer");
            Vec::with_capacity(self.buffer_size)
        })
    }

    /// Return a buffer to the pool
    ///
    /// The buffer is cleared but retains its capacity for reuse.
    pub fn release(&mut self, mut buffer: Vec<u8>) {
        buffer.clear();
        // Only keep buffers that haven't grown too large
        if buffer.capacity() <= self.buffer_size * 2 {
            self.buffers.push(buffer);
        }
    }

    /// Number of available buffers in the pool
    pub fn available(&self) -> usize {
        self.buffers.len()
    }
}

impl Default for StatePool {
    fn default() -> Self {
        Self::with_defaults()
    }
}

// ============================================================================
// Rollback State Manager
// ============================================================================

/// Manages game state saves and loads for GGRS rollback
///
/// This struct handles the integration between GGRS requests and the
/// `GameInstance` save/load functionality. It uses a `StatePool` to
/// avoid allocations during the rollback hot path.
pub struct RollbackStateManager {
    /// Pre-allocated buffer pool
    pool: StatePool,
    /// Maximum state size in bytes (should match console's RAM limit)
    max_state_size: usize,
}

impl RollbackStateManager {
    /// Create a new rollback state manager with specified max state size
    ///
    /// The `max_state_size` should match the console's RAM limit from `ConsoleSpecs::ram_limit`.
    /// For example:
    /// - Nethercore ZX: 4MB
    /// - Nethercore Chroma: 1MB
    pub fn new(max_state_size: usize) -> Self {
        Self {
            pool: StatePool::new(max_state_size, STATE_POOL_SIZE),
            max_state_size,
        }
    }

    /// Create a rollback state manager with default settings
    ///
    /// Uses [`MAX_STATE_SIZE`](super::config::MAX_STATE_SIZE) (16MB) as the default.
    /// **Prefer using `new(console.specs().ram_limit)` to respect console limits.**
    pub fn with_defaults() -> Self {
        Self::new(MAX_STATE_SIZE)
    }

    /// Save the current game state
    ///
    /// Calls `game.save_state()` to snapshot the entire WASM linear memory,
    /// serializes the console rollback state via bytemuck, captures input state
    /// (for button_pressed to work correctly), and host-side state (RNG, tick
    /// count, elapsed time) for determinism.
    /// Returns a `GameStateSnapshot` with checksum.
    pub fn save_state<I: ConsoleInput, S: Send + Default + 'static, R: ConsoleRollbackState>(
        &mut self,
        game: &mut GameInstance<I, S, R>,
        frame: i32,
    ) -> Result<GameStateSnapshot, SaveStateError> {
        // Snapshot entire WASM linear memory
        let snapshot_data = game
            .save_state()
            .map_err(|e| SaveStateError::WasmError(e.to_string()))?;

        // Serialize console rollback state via bytemuck (zero-copy for POD types)
        // SmallVec stores inline (no heap allocation) for typical console states (<512 bytes)
        let console_data = SmallVec::from_slice(bytemuck::bytes_of(game.rollback_state()));

        // Serialize input state (input_prev and input_curr)
        // Required for button_pressed() to work correctly after rollback
        let game_state = game.state();
        let mut input_data: InputDataVec = SmallVec::new();
        input_data.extend_from_slice(bytemuck::cast_slice(&game_state.input_prev));
        input_data.extend_from_slice(bytemuck::cast_slice(&game_state.input_curr));

        // Capture host-side state that affects game simulation
        let host_state = HostRollbackState::new(
            game_state.rng_state,
            game_state.tick_count,
            game_state.elapsed_time,
        );

        let total_size = snapshot_data.len() + console_data.len() + input_data.len() + HOST_STATE_SIZE;
        if total_size > self.max_state_size {
            return Err(SaveStateError::StateTooLarge {
                size: total_size,
                max: self.max_state_size,
            });
        }

        // Create snapshot with checksum covering all state
        Ok(GameStateSnapshot::from_full_state(
            snapshot_data,
            console_data,
            input_data,
            host_state,
            frame,
        ))
    }

    /// Load a game state from a snapshot
    ///
    /// Calls `game.load_state()` to restore the WASM linear memory,
    /// deserializes the console rollback state via bytemuck, restores input
    /// state (for button_pressed to work correctly), and host-side state
    /// (RNG, tick count, elapsed time) for determinism.
    pub fn load_state<I: ConsoleInput, S: Send + Default + 'static, R: ConsoleRollbackState>(
        &mut self,
        game: &mut GameInstance<I, S, R>,
        snapshot: &GameStateSnapshot,
    ) -> Result<(), LoadStateError> {
        use crate::wasm::state::MAX_PLAYERS;

        if snapshot.is_empty() {
            // Nothing to load
            return Ok(());
        }

        // Restore WASM linear memory
        game.load_state(&snapshot.data)
            .map_err(|e| LoadStateError::WasmError(e.to_string()))?;

        // Restore console rollback state if present
        if !snapshot.console_data.is_empty() {
            if let Ok(console_state) = bytemuck::try_from_bytes::<R>(&snapshot.console_data) {
                *game.rollback_state_mut() = *console_state;
            } else {
                return Err(LoadStateError::WasmError(
                    "Console rollback state size mismatch".to_string(),
                ));
            }
        }

        // Restore input state if present
        // Input data layout: [input_prev × MAX_PLAYERS][input_curr × MAX_PLAYERS]
        let input_size = std::mem::size_of::<I>();
        let expected_input_len = input_size * MAX_PLAYERS * 2;
        if snapshot.input_data.len() == expected_input_len {
            let game_state = game.state_mut();
            let input_bytes = &snapshot.input_data[..];

            // Restore input_prev (first half)
            let prev_bytes = &input_bytes[..input_size * MAX_PLAYERS];
            if let Ok(prev_inputs) = bytemuck::try_cast_slice::<u8, I>(prev_bytes) {
                game_state.input_prev.copy_from_slice(prev_inputs);
            }

            // Restore input_curr (second half)
            let curr_bytes = &input_bytes[input_size * MAX_PLAYERS..];
            if let Ok(curr_inputs) = bytemuck::try_cast_slice::<u8, I>(curr_bytes) {
                game_state.input_curr.copy_from_slice(curr_inputs);
            }
        }

        // Restore host-side state for determinism
        let game_state = game.state_mut();
        game_state.rng_state = snapshot.host_state.rng_state;
        game_state.tick_count = snapshot.host_state.tick_count;
        game_state.elapsed_time = snapshot.host_state.elapsed_time();

        Ok(())
    }

    /// Return a snapshot's buffer to the pool
    ///
    /// Call this when GGRS is done with a snapshot (e.g., after confirming a frame).
    pub fn recycle_snapshot(&mut self, snapshot: GameStateSnapshot) {
        if !snapshot.data.is_empty() {
            self.pool.release(snapshot.data);
        }
    }
}

impl Default for RollbackStateManager {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Error saving game state
#[derive(Debug, Clone)]
pub enum SaveStateError {
    /// WASM error during save
    WasmError(String),
    /// State exceeds maximum size
    StateTooLarge { size: usize, max: usize },
}

impl std::fmt::Display for SaveStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WasmError(e) => write!(f, "WASM error during save_state: {}", e),
            Self::StateTooLarge { size, max } => {
                write!(f, "State too large: {} bytes (max {})", size, max)
            }
        }
    }
}

impl std::error::Error for SaveStateError {}

/// Error loading game state
#[derive(Debug, Clone)]
pub enum LoadStateError {
    /// WASM error during load
    WasmError(String),
}

impl std::fmt::Display for LoadStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WasmError(e) => write!(f, "WASM error during load_state: {}", e),
        }
    }
}

impl std::error::Error for LoadStateError {}

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
        let host_state = HostRollbackState::new(0xDEADBEEF, 42, 3.14159);
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

        let snapshot1 =
            GameStateSnapshot::from_full_state(data.clone(), console_data.clone(), input_data.clone(), host1, 0);
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

        let snapshot1 =
            GameStateSnapshot::from_full_state(data.clone(), console_data.clone(), input1, host_state, 0);
        let snapshot2 =
            GameStateSnapshot::from_full_state(data, console_data, input2, host_state, 0);

        assert_ne!(snapshot1.checksum, snapshot2.checksum);
    }
}
