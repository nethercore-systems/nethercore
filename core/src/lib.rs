//! Emberware Core - Shared console framework
//!
//! This crate provides the foundational traits and types for building
//! Emberware fantasy consoles with shared rollback netcode infrastructure.
//!
//! # Architecture
//!
//! - [`Console`] - Trait implemented by each fantasy console (e.g., Emberware Z)
//! - [`Runtime`] - Game loop orchestration with fixed timestep updates
//! - [`GameInstance`] - WASM game loaded and instantiated
//! - [`RollbackSession`] - GGRS integration for rollback netcode

pub mod console;
pub mod ffi;
#[cfg(test)]
mod integration;
pub mod rollback;
pub mod runtime;
#[cfg(test)]
pub mod test_utils;
pub mod wasm;

// Re-export core traits and types
pub use console::{Audio, Console, ConsoleInput, ConsoleSpecs, Graphics};
pub use runtime::{Runtime, RuntimeConfig};
pub use wasm::{
    GameInstance, GameState, GameStateWithConsole, WasmEngine, MAX_PLAYERS, MAX_SAVE_SIZE,
    MAX_SAVE_SLOTS,
};

// Re-export rollback types
pub use rollback::{
    ConnectionQuality, EmberwareConfig, GameStateSnapshot, LoadStateError, LocalSocket,
    LocalSocketError, NetworkInput, PlayerNetworkStats, PlayerSessionConfig, RollbackSession,
    RollbackStateManager, SaveStateError, SessionConfig, SessionError, SessionEvent, SessionType,
    StatePool, DEFAULT_INPUT_DELAY, DEFAULT_LOCAL_PORT, DEFAULT_ONLINE_INPUT_DELAY,
    MAX_INPUT_DELAY, MAX_ROLLBACK_FRAMES, MAX_STATE_SIZE, STATE_POOL_SIZE,
};

// Re-export GGRS types for convenience
pub use ggrs::{GgrsError, GgrsEvent, GgrsRequest, InputStatus, PlayerType, SessionState};
