//! Emberware Z - Library interface
//!
//! This module exports the public API for emberware-z, allowing it to be
//! used as a library by the unified launcher.
//!
//! # Features
//!
//! - **`runtime`** (default): Full console runtime with graphics, audio, input, and networking.
//!   Required for running games.
//! - **No features**: Minimal build with just packing utilities and vertex format constants.
//!   Used by `ember-export` and other asset tools.

// ============================================================================
// Always available: graphics packing utilities
// ============================================================================

pub mod graphics;  // Contains packing functions (always) and ZGraphics (runtime only)

// ============================================================================
// Runtime-only modules
// ============================================================================

#[cfg(feature = "runtime")]
pub mod app;
#[cfg(feature = "runtime")]
mod audio;
#[cfg(feature = "runtime")]
pub mod console;
#[cfg(feature = "runtime")]
mod ffi;
#[cfg(feature = "runtime")]
mod font;
#[cfg(feature = "runtime")]
mod input;
#[cfg(feature = "runtime")]
mod library;
#[cfg(feature = "runtime")]
mod procedural;
#[cfg(feature = "runtime")]
mod resource_manager;
#[cfg(feature = "runtime")]
mod settings_ui;
#[cfg(feature = "runtime")]
mod shader_gen;
#[cfg(feature = "runtime")]
mod state;
#[cfg(feature = "runtime")]
mod ui;
