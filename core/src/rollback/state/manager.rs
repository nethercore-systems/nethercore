//! Rollback state manager integrating GGRS with game state saves/loads

use smallvec::SmallVec;

use crate::console::{ConsoleInput, ConsoleRollbackState};
use crate::rollback::config::MAX_STATE_SIZE;
use crate::wasm::GameInstance;

use super::host_state::{HOST_STATE_SIZE, HostRollbackState};
use super::pool::StatePool;
use super::snapshot::GameStateSnapshot;
use super::{InputDataVec, STATE_POOL_SIZE};

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
    /// Uses [`MAX_STATE_SIZE`](crate::rollback::config::MAX_STATE_SIZE) as the fallback.
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

        let total_size =
            snapshot_data.len() + console_data.len() + input_data.len() + HOST_STATE_SIZE;
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
        // Input data layout: [input_prev ×MAX_PLAYERS][input_curr ×MAX_PLAYERS]
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
