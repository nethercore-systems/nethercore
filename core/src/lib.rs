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
//!
//! # Example
//!
//! ```ignore
//! // Create a runtime for a specific console
//! let console = EmberwareZ::new();
//! let mut runtime = Runtime::new(console);
//!
//! // Load and initialize a game
//! let engine = WasmEngine::new()?;
//! let module = engine.load_module(wasm_bytes)?;
//! let game = GameInstance::new(&engine, &module, &linker)?;
//! runtime.load_game(game);
//! runtime.init_game()?;
//!
//! // Main loop
//! loop {
//!     let (ticks, alpha) = runtime.frame()?;
//!     runtime.render()?;
//! }
//! ```

pub mod console;
pub mod ffi;
pub mod rollback;
pub mod runtime;
pub mod wasm;

// Re-export core traits and types
pub use console::{Audio, Console, ConsoleInput, ConsoleSpecs, Graphics};
pub use runtime::{Runtime, RuntimeConfig};
pub use wasm::{GameInstance, GameState, WasmEngine};

// Re-export rollback types
pub use rollback::{
    ConnectionQuality, EmberwareConfig, GameStateSnapshot, LoadStateError, NetworkInput,
    PlayerNetworkStats, RollbackSession, RollbackStateManager, SaveStateError, SessionConfig,
    SessionError, SessionEvent, SessionType, StatePool, DEFAULT_INPUT_DELAY,
    DEFAULT_ONLINE_INPUT_DELAY, MAX_INPUT_DELAY, MAX_ROLLBACK_FRAMES, MAX_STATE_SIZE,
    STATE_POOL_SIZE,
};

// Re-export GGRS types for convenience
pub use ggrs::{GgrsError, GgrsEvent, GgrsRequest, InputStatus, PlayerType, SessionState};
