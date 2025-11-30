//! Rollback state management
//!
//! Provides state snapshot and buffer pool functionality for GGRS rollback.

use crate::wasm::GameInstance;

use super::config::MAX_STATE_SIZE;

/// Number of pre-allocated state buffers in the pool
pub const STATE_POOL_SIZE: usize = super::config::MAX_ROLLBACK_FRAMES + 2;

// ============================================================================
// Game State Snapshot
// ============================================================================

/// Snapshot of game state for rollback
///
/// Contains the serialized WASM game state and a checksum for desync detection.
/// The data comes from calling `GameInstance::save_state()` which invokes the
/// game's exported `save_state(ptr, max_len) -> len` function.
#[derive(Clone)]
pub struct GameStateSnapshot {
    /// Serialized WASM game state
    pub data: Vec<u8>,
    /// FNV-1a checksum for desync detection
    pub checksum: u64,
    /// Frame number this snapshot was taken at
    pub frame: i32,
}

impl GameStateSnapshot {
    /// Create a new empty snapshot
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            checksum: 0,
            frame: -1,
        }
    }

    /// Create a snapshot from serialized data
    pub fn from_data(data: Vec<u8>, frame: i32) -> Self {
        let checksum = Self::compute_checksum(&data);
        Self {
            data,
            checksum,
            frame,
        }
    }

    /// Create a snapshot from a pre-allocated buffer (avoids allocation)
    pub fn from_buffer(buffer: &mut Vec<u8>, len: usize, frame: i32) -> Self {
        buffer.truncate(len);
        let checksum = Self::compute_checksum(buffer);
        Self {
            data: std::mem::take(buffer),
            checksum,
            frame,
        }
    }

    /// Check if this snapshot is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the size of the serialized state in bytes
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Compute FNV-1a checksum for desync detection
    ///
    /// FNV-1a is fast and has good distribution for checksumming.
    /// We use this to detect desyncs between clients.
    fn compute_checksum(data: &[u8]) -> u64 {
        const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
        const FNV_PRIME: u64 = 0x100000001b3;

        let mut hash = FNV_OFFSET_BASIS;
        for byte in data {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        hash
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
}

impl RollbackStateManager {
    /// Create a new rollback state manager
    pub fn new() -> Self {
        Self {
            pool: StatePool::with_defaults(),
        }
    }

    /// Save the current game state
    ///
    /// Calls `game.save_state()` and returns a `GameStateSnapshot` with checksum.
    /// Uses the internal buffer pool to avoid allocations.
    pub fn save_state(
        &mut self,
        game: &mut GameInstance,
        frame: i32,
    ) -> Result<GameStateSnapshot, SaveStateError> {
        // Acquire buffer from pool
        let mut buffer = self.pool.acquire();
        buffer.resize(MAX_STATE_SIZE, 0);

        // Call into WASM to save state
        let len = game
            .save_state(&mut buffer)
            .map_err(|e| SaveStateError::WasmError(e.to_string()))?;

        if len == 0 {
            // Game doesn't implement save_state, return empty snapshot
            self.pool.release(buffer);
            return Ok(GameStateSnapshot::new());
        }

        if len > MAX_STATE_SIZE {
            self.pool.release(buffer);
            return Err(SaveStateError::StateTooLarge {
                size: len,
                max: MAX_STATE_SIZE,
            });
        }

        // Create snapshot from buffer
        Ok(GameStateSnapshot::from_buffer(&mut buffer, len, frame))
    }

    /// Load a game state from a snapshot
    ///
    /// Calls `game.load_state()` to restore the game to the saved state.
    pub fn load_state(
        &mut self,
        game: &mut GameInstance,
        snapshot: &GameStateSnapshot,
    ) -> Result<(), LoadStateError> {
        if snapshot.is_empty() {
            // Nothing to load
            return Ok(());
        }

        game.load_state(&snapshot.data)
            .map_err(|e| LoadStateError::WasmError(e.to_string()))
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
        Self::new()
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
}
