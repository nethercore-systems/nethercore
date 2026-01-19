//! WASM runtime wrapper
//!
//! Provides abstractions over wasmtime for loading and executing game WASM modules.
//!
//! # Module Organization
//!
//! - [`state`] - Core game state structure (console-agnostic)
//! - [`engine`] - WASM engine for loading and compiling modules
//! - [`instance`] - Game instance for executing WASM games
//!
//! # Key Types
//!
//! - [`WasmEngine`] - Shared WASM engine (one per application)
//! - [`GameInstance`] - Loaded and instantiated game
//! - [`GameState`] - Minimal core state (input, timing, RNG, saves)
//! - [`WasmGameContext`] - Context combining core + console FFI + rollback state

mod engine;
mod instance;
pub mod state;

#[cfg(test)]
mod tests;

// Re-export main types
pub use engine::WasmEngine;
pub use instance::GameInstance;

// Re-export public types from state module
#[allow(deprecated)]
pub use state::{
    GameState, GameStateWithConsole, MAX_PLAYERS, MAX_SAVE_SIZE, MAX_SAVE_SLOTS, MemoryAccessError,
    WasmGameContext, read_bytes_from_memory, read_string_from_memory, write_bytes_to_memory,
};
