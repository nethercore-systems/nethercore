//! GGRS rollback integration
//!
//! Provides the configuration and state management for GGRS rollback netcode.
//!
//! # Architecture
//!
//! GGRS (Good Game Rollback SDK) handles deterministic rollback netcode. This module provides:
//!
//! - `EmberwareConfig<I>`: GGRS configuration parameterized by console input type
//! - `GameStateSnapshot`: Serialized game state with checksum for desync detection
//! - `RollbackSession<I>`: Session manager for local, sync-test, and P2P modes
//! - `StatePool`: Pre-allocated buffer pool to avoid allocations during rollback
//!
//! # Input Flow
//!
//! 1. Physical input is mapped to console-specific `ConsoleInput` (e.g., `ZInput`)
//! 2. Input is added to GGRS via `session.add_local_input()`
//! 3. GGRS handles prediction, confirmation, and rollback
//! 4. Confirmed inputs are passed to `GameInstance::update()` during advance
//!
//! # Module Structure
//!
//! - `config`: GGRS configuration types and constants
//! - `player`: Player session configuration (local vs remote)
//! - `state`: State snapshot and buffer pool management
//! - `session`: GGRS session wrapper and event handling

mod config;
mod player;
mod session;
mod state;

// Re-export public types from config
pub use config::{
    EmberwareConfig, SessionConfig, DEFAULT_INPUT_DELAY, DEFAULT_ONLINE_INPUT_DELAY,
    MAX_INPUT_DELAY, MAX_ROLLBACK_FRAMES, MAX_STATE_SIZE, STATE_POOL_SIZE,
};

// Re-export public types from player
pub use player::{PlayerSessionConfig, MAX_PLAYERS};

// Re-export public types from state
pub use state::{
    GameStateSnapshot, LoadStateError, RollbackStateManager, SaveStateError, StatePool,
};

// Re-export public types from session
pub use session::{
    ConnectionQuality, NetworkInput, PlayerNetworkStats, RollbackSession, SessionError,
    SessionEvent, SessionType,
};
