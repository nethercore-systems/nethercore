//! Emberware Core - Shared console framework
//!
//! This crate provides the foundational traits and types for building
//! Emberware fantasy consoles with shared rollback netcode infrastructure.

pub mod console;
pub mod ffi;
pub mod rollback;
pub mod runtime;
pub mod wasm;

pub use console::{Audio, Console, ConsoleInput, ConsoleSpecs, Graphics};
pub use runtime::Runtime;
pub use wasm::{GameInstance, WasmEngine};
