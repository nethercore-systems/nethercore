//! Nethercore Unified Library
//!
//! This crate provides the unified launcher for all Nethercore fantasy consoles.
//! It contains the console-agnostic UI and application logic.
//!
//! The library is 100% console-agnostic. Games are launched as separate player
//! processes (e.g., `nethercore-zx` for ZX games). This provides:
//! - Crash isolation (game crash doesn't crash the library)
//! - Clean separation of concerns
//! - Easy addition of new console types

pub mod app;
pub mod protocol;
pub mod registry;
pub mod ui;
