//! Emberware Z - Library interface
//!
//! This module exports the public API for emberware-z, allowing it to be
//! used as a library by the unified launcher or as a standalone player.

pub mod audio;
pub mod capture;
pub mod console;
pub mod ffi;
mod font;
pub mod graphics;
pub mod input;
pub mod library;
pub mod player;
mod procedural;
pub mod resource_manager;
mod shader_gen;
pub mod state;
