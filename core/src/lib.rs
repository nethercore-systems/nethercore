//! Nethercore Core - Shared console framework
//!
//! This crate provides the foundational traits and types for building
//! Nethercore fantasy consoles with shared rollback netcode infrastructure.
//!
//! # Architecture
//!
//! - [`Console`] - Trait implemented by each fantasy console (e.g., Nethercore ZX)
//! - [`Runtime`] - Game loop orchestration with fixed timestep updates
//! - [`GameInstance`] - WASM game loaded and instantiated
//! - [`RollbackSession`] - GGRS integration for rollback netcode

pub mod analysis;
pub mod app;
pub mod capture;
pub mod console;
pub mod debug;
pub mod ffi;
#[cfg(test)]
mod integration;
pub mod library;
pub mod rollback;
pub mod runner;
pub mod runtime;
#[cfg(test)]
pub mod test_utils;
pub mod wasm;

// Re-export core traits and types
pub use console::{
    Audio, AudioGenerator, Console, ConsoleInput, ConsoleRollbackState, ConsoleSpecs, Graphics,
};
pub use runner::ConsoleRunner;
pub use runtime::{Runtime, RuntimeConfig};
#[allow(deprecated)]
pub use wasm::{
    GameInstance, GameState, GameStateWithConsole, MAX_PLAYERS, MAX_SAVE_SIZE, MAX_SAVE_SLOTS,
    WasmEngine, WasmGameContext,
};

// Re-export rollback types
pub use rollback::{
    ConnectionQuality, DEFAULT_INPUT_DELAY, DEFAULT_LOCAL_PORT, DEFAULT_ONLINE_INPUT_DELAY,
    NethercoreConfig, GameStateSnapshot, LoadStateError, LocalSocket, LocalSocketError,
    MAX_INPUT_DELAY, MAX_ROLLBACK_FRAMES, MAX_STATE_SIZE, NetworkInput, PlayerNetworkStats,
    PlayerSessionConfig, RollbackSession, RollbackStateManager, STATE_POOL_SIZE, SaveStateError,
    SessionConfig, SessionError, SessionEvent, SessionType, StatePool,
};

// Re-export GGRS types for convenience
pub use ggrs::{GgrsError, GgrsEvent, GgrsRequest, InputStatus, PlayerType, SessionState};

// Re-export analysis types for build-time WASM analysis
pub use analysis::{AnalysisError, AnalysisResult, TextureFormatHint, analyze_wasm};

// Re-export debug types
pub use debug::DebugStat;

// Re-export capture types
pub use capture::{CaptureSupport, SaveResult, ScreenCapture, read_capture_pixels, read_render_target_pixels};
