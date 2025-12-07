//! Application framework types and utilities
//!
//! This module contains console-agnostic application types that are shared
//! across all fantasy consoles (Emberware Z, Classic, etc.).

pub mod config;
pub mod debug;
pub mod input;
pub mod session;
pub mod types;

pub use config::Config;
pub use debug::{calculate_fps, render_debug_overlay, update_frame_times};
pub use input::InputManager;
pub use session::GameSession;
pub use types::{
    AppMode, DebugStats, RuntimeError, FRAME_TIME_HISTORY_SIZE, GRAPH_MAX_FRAME_TIME_MS,
    TARGET_FRAME_TIME_MS,
};
