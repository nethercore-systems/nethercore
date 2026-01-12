//! Replay runtime
//!
//! This module contains the execution infrastructure for replays:
//! - **Executor**: Runs compiled scripts with snapshot/assertion support
//! - **Recorder**: Captures gameplay for later playback
//! - **Player**: Plays back recorded replays
//! - **Headless**: Headless execution for CI/testing

mod executor;
mod headless;
mod player;
mod recorder;

pub use executor::{
    DebugVariableInfo, ExecutionReport, ReportSummary, ScriptExecutor, StepResult, StopReason,
};
pub use headless::{HeadlessBackend, HeadlessConfig, HeadlessRunner};
pub use player::{Player, PlayerConfig, SeekResult};
pub use recorder::{Recorder, RecorderConfig};
