//! Emberware Z library
//!
//! Exposes internal modules for testing and integration.

pub mod console;
pub mod graphics;
pub mod state;
pub mod ffi;
pub mod audio;
pub mod config;
pub mod shader_gen;
pub mod font;
pub mod input;
pub mod library;

// Re-export commonly used types for tests
pub use graphics::{MvpIndex, VirtualRenderPass};
pub use state::ZFFIState;
