//! Frame rendering and presentation
//!
//! This module handles the main rendering loop, including:
//! - Blitting render target to window
//! - Processing and executing draw commands
//! - Managing render passes and GPU state
//! - Buffer capacity management

mod bind_group_cache;
mod blit;
mod buffer_capacity;
mod render_frame;

// Re-export public items from submodules
// All items are already implemented as impl blocks on ZXGraphics,
// so no additional re-exports are needed.
