//! Replay runtime
//!
//! This module contains the execution infrastructure for replays:
//! - **Executor**: Runs compiled scripts with snapshot/assertion support
//! - **Recorder**: Captures gameplay for later playback
//! - **Player**: Plays back recorded replays

mod executor;
mod player;
mod recorder;

pub use executor::{ExecutionReport, ReportSummary, ScriptExecutor, StepResult, StopReason};
pub use player::{Player, PlayerConfig, SeekResult};
pub use recorder::{Recorder, RecorderConfig};
