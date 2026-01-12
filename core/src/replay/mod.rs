//! Nethercore Replay System
//!
//! A dual-format replay system for recording, playback, and AI-assisted debugging:
//!
//! - **Binary format (`.ncrp`)**  ECompact storage for recorded gameplay
//! - **Script format (`.ncrs`)**  EHuman-readable TOML for testing and debugging
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────━E
//! ━E                   Recording Mode                           ━E
//! ━E gameplay ↁERecorder ↁE.ncrp (binary)                      ━E
//! └─────────────────────────────────────────────────────────────━E
//!
//! ┌─────────────────────────────────────────────────────────────━E
//! ━E                   Playback Mode                            ━E
//! ━E .ncrp ↁEPlayer ↁEre-executes gameplay                     ━E
//! └─────────────────────────────────────────────────────────────━E
//!
//! ┌─────────────────────────────────────────────────────────────━E
//! ━E                   Script Execution                         ━E
//! ━E .ncrs (TOML) ↁECompiler ↁEExecutor ↁEreport.json          ━E
//! ━E                             ↁE                             ━E
//! ━E                   snap: capture debug vars                ━E
//! ━E                   assert: evaluate conditions             ━E
//! └─────────────────────────────────────────────────────────────━E
//! ```
//!
//! # Usage
//!
//! ## Recording Gameplay
//!
//! ```ignore
//! use nethercore_core::replay::{Recorder, RecorderConfig};
//!
//! let config = RecorderConfig {
//!     console_id: 1,
//!     player_count: 1,
//!     input_size: 8,
//!     seed: 12345,
//!     checkpoint_interval: 300,
//!     compress: true,
//! };
//!
//! let mut recorder = Recorder::new(config);
//! recorder.start();
//!
//! // During game loop:
//! recorder.record_frame(vec![raw_input_bytes]);
//! if recorder.should_checkpoint() {
//!     recorder.record_checkpoint(game_state);
//! }
//!
//! let replay = recorder.stop();
//! ```
//!
//! ## Playback
//!
//! ```ignore
//! use nethercore_core::replay::{Player, PlayerConfig};
//!
//! let config = PlayerConfig {
//!     speed: 1.0,
//!     loop_playback: false,
//!     show_debug: true,
//! };
//!
//! let mut player = Player::new(replay, config);
//! player.play();
//!
//! while !player.is_complete() {
//!     if let Some(inputs) = player.current_inputs() {
//!         // Apply inputs to game
//!     }
//!     player.advance_frame();
//! }
//! ```
//!
//! ## Script Execution
//!
//! ```ignore
//! use nethercore_core::replay::{ReplayScript, Compiler, ScriptExecutor};
//!
//! let script = ReplayScript::from_file("test.ncrs")?;
//! let compiled = Compiler::new(&layout).compile(&script)?;
//! let mut executor = ScriptExecutor::new(compiled);
//!
//! while !executor.is_complete() {
//!     if executor.needs_snapshot() {
//!         let pre = capture_debug_values();
//!         // Run game frame
//!         let post = capture_debug_values();
//!         executor.capture_post_snapshot(pre, post, input_string);
//!     }
//!
//!     for assertion in executor.current_assertions() {
//!         executor.evaluate_assertion(assertion, &values, fail_fast);
//!     }
//!
//!     executor.advance_frame();
//! }
//!
//! let report = executor.generate_report();
//! ```

pub mod binary;
pub mod runtime;
pub mod script;
pub mod types;

// Re-export core types
pub use types::{
    AssertExpr, Assertion, AssertionResult, Breakpoint, Checkpoint, DebugValue, DebugValueData,
    InputSequence, InspectDirective, InspectTiming, LogDirective, Replay, ReplayFlags,
    ReplayHeader, Snapshot,
};

// Re-export binary format
pub use binary::{BinaryReader, BinaryWriter};

// Re-export script format
pub use script::{
    ActionParamValue, AssertCondition, AssertValue, CompareOp, CompileError, CompiledAction,
    CompiledAssertValue, CompiledAssertion, CompiledScript, Compiler, FrameEntry, InputLayout,
    InputValue, ParseError, ReplayScript, StructuredInput, decompile,
};

// Re-export runtime
pub use runtime::{
    DebugVariableInfo, ExecutionReport, HeadlessBackend, HeadlessConfig, HeadlessRunner, Player,
    PlayerConfig, Recorder, RecorderConfig, ReportSummary, ScriptExecutor, SeekResult, StepResult,
    StopReason,
};
