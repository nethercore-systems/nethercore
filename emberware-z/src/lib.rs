//! Emberware Z library
//!
//! Exposes internal modules for testing and integration.

pub mod audio;
pub mod config;
pub mod console;
pub mod ffi;
pub mod font;
pub mod graphics;
pub mod input;
pub mod library;
pub mod shader_gen;
pub mod state;

// Re-export commonly used types for tests
pub use graphics::{MvpIndex, VirtualRenderPass};
pub use state::ZFFIState;
