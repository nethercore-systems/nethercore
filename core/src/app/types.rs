//! Application types and data structures
//!
//! Console-agnostic types used across the application framework.

use std::collections::VecDeque;
use std::path::PathBuf;

/// Application mode state machine
///
/// Represents the current mode of the application, determining which
/// UI is shown and what logic is active.
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    /// Library/launcher UI mode
    Library,
    /// Playing a game by ID (from installed games)
    Playing { game_id: String },
    /// Playing a game directly from a file path (for development)
    PlayingFromPath { path: PathBuf },
    /// Settings/configuration UI mode
    Settings,
}

/// Runtime error for state machine transitions
///
/// Stores an error message that is displayed to the user when returning
/// to the library screen after a runtime error occurs.
#[derive(Debug, Clone)]
pub struct RuntimeError(pub String);

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Frame time sample for graph
pub const FRAME_TIME_HISTORY_SIZE: usize = 120;
/// Target frame time for reference line (60 FPS = 16.67ms)
pub const TARGET_FRAME_TIME_MS: f32 = 16.67;
/// Maximum frame time shown in graph (30 FPS = 33.33ms, 2x target)
pub const GRAPH_MAX_FRAME_TIME_MS: f32 = 33.33;

/// Debug statistics for overlay
///
/// Tracks performance metrics and network statistics for the debug overlay.
/// This data is displayed when the user presses F3.
#[derive(Debug, Default)]
pub struct DebugStats {
    /// Frame times ring buffer (milliseconds) - application render times
    pub frame_times: VecDeque<f32>,
    /// Game tick times ring buffer (milliseconds) - game update() times
    pub game_tick_times: VecDeque<f32>,
    /// Game render times ring buffer (milliseconds) - game render() times
    pub game_render_times: VecDeque<f32>,
    /// VRAM usage in bytes
    pub vram_used: usize,
    /// VRAM limit in bytes
    pub vram_limit: usize,
    /// Network stats (when in P2P session)
    pub ping_ms: Option<u32>,
    /// Rollback frames this session
    pub rollback_frames: u64,
    /// Frame advantage (how far ahead of opponent)
    pub frame_advantage: i32,
    /// Network interrupted warning (disconnect timeout in ms, None if connected)
    pub network_interrupted: Option<u64>,
}
