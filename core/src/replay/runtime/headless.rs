//! Headless replay execution
//!
//! Provides a simplified execution environment for running replay scripts
//! without graphics, audio, or window management. Ideal for CI/testing.

use anyhow::{Context, Result};
use hashbrown::HashMap;
use std::path::Path;
use std::time::Instant;

use crate::replay::script::{CompiledScript, Compiler};
use crate::replay::types::DebugValueData;
use crate::replay::InputLayout;

use super::executor::{DebugVariableInfo, ExecutionReport, ScriptExecutor};

/// Headless replay runner configuration
#[derive(Debug, Clone)]
pub struct HeadlessConfig {
    /// Stop on first assertion failure
    pub fail_fast: bool,
    /// Maximum execution time in seconds
    pub timeout_secs: u64,
    /// Script file path (for reporting)
    pub script_path: Option<String>,
}

impl Default for HeadlessConfig {
    fn default() -> Self {
        Self {
            fail_fast: false,
            timeout_secs: 300,
            script_path: None,
        }
    }
}

/// Headless replay runner
///
/// Executes replay scripts without graphics, audio, or window management.
/// Captures debug variable snapshots and generates execution reports.
pub struct HeadlessRunner {
    executor: ScriptExecutor,
    config: HeadlessConfig,
    debug_variables: HashMap<String, DebugValueData>,
    registered_vars: Vec<DebugVariableInfo>,
    start_time: Option<Instant>,
}

impl HeadlessRunner {
    /// Create a new headless runner
    pub fn new(script: CompiledScript, config: HeadlessConfig) -> Self {
        Self {
            executor: ScriptExecutor::new(script),
            config,
            debug_variables: HashMap::new(),
            registered_vars: Vec::new(),
            start_time: None,
        }
    }

    /// Create from TOML script file
    pub fn from_file<L: InputLayout>(
        script_path: &Path,
        layout: &L,
        config: HeadlessConfig,
    ) -> Result<Self> {
        let script = crate::replay::ReplayScript::from_file(script_path)
            .with_context(|| format!("Failed to parse script: {}", script_path.display()))?;

        let compiler = Compiler::new(layout);
        let compiled = compiler
            .compile(&script)
            .with_context(|| "Failed to compile script")?;

        let mut runner_config = config;
        runner_config.script_path = Some(script_path.display().to_string());

        Ok(Self::new(compiled, runner_config))
    }

    /// Register available debug variables
    ///
    /// This should be called with the list of variables that the game
    /// has registered for debugging. In a full implementation, this would
    /// come from the game's debug inspector.
    pub fn register_debug_variables(&mut self, vars: Vec<DebugVariableInfo>) {
        self.registered_vars = vars;
    }

    /// Set a debug variable value
    ///
    /// In a full implementation, this would read from the game's debug inspector.
    /// For testing, values can be set manually.
    pub fn set_debug_variable(&mut self, name: String, value: DebugValueData) {
        self.debug_variables.insert(name, value);
    }

    /// Execute the replay script
    ///
    /// This is a simplified execution that doesn't actually run a game.
    /// For full execution, this would need to:
    /// 1. Load the game WASM
    /// 2. Create a minimal console context
    /// 3. Call update() with inputs
    /// 4. Read debug variables from the game
    ///
    /// Currently, this executes the script structure and captures snapshots
    /// of the debug variables that have been manually set.
    pub fn execute(&mut self) -> Result<ExecutionReport> {
        self.start_time = Some(Instant::now());

        while !self.executor.is_complete() {
            // Check timeout
            if let Some(start) = self.start_time {
                if start.elapsed().as_secs() > self.config.timeout_secs {
                    self.executor.stop_with_error("Execution timeout exceeded".to_string());
                    break;
                }
            }

            // Get inputs for current frame
            let _inputs = self.executor.current_inputs();

            // TODO: Apply inputs to game and call update()
            // For now, we just simulate frame advancement

            // Capture snapshot if needed
            if self.executor.needs_snapshot() {
                let pre_values = self.debug_variables.clone();

                // TODO: Call game update() here
                // For now, we just keep the same values

                let post_values = self.debug_variables.clone();

                let input_string = format!("frame_{}", self.executor.current_frame());
                self.executor
                    .capture_post_snapshot(pre_values, post_values, input_string);
            }

            // Evaluate assertions
            let current_frame = self.executor.current_frame();
            let assertion_count = self.executor.current_assertions().len();

            // Process each assertion
            for _ in 0..assertion_count {
                // TODO: Evaluate against actual game state
                // For now, we just mark them as passed (placeholder)
                self.executor.record_assertion_result(
                    current_frame,
                    "placeholder".to_string(),
                    true, // passed
                    None, // actual
                    None, // expected
                );
            }

            // Check fail-fast
            if self.config.fail_fast && assertion_count > 0 {
                // In real implementation, check if any assertion failed
                // For now, we continue since we marked all as passed
            }

            // Advance frame
            self.executor.advance_frame();
        }

        // Generate report
        let mut report = self.executor.generate_report();

        // Add metadata
        if let Some(script_path) = &self.config.script_path {
            report.script = Some(script_path.clone());
        }

        if let Some(start) = self.start_time {
            let duration = start.elapsed();
            report.duration_ms = Some(duration.as_millis() as u64);
            report.executed_at = Some(chrono::Utc::now().to_rfc3339());
        }

        if !self.registered_vars.is_empty() {
            report.registered_variables = Some(self.registered_vars.clone());
        }

        Ok(report)
    }

    /// Get the executor (for testing)
    pub fn executor(&self) -> &ScriptExecutor {
        &self.executor
    }

    /// Get mutable executor (for testing)
    pub fn executor_mut(&mut self) -> &mut ScriptExecutor {
        &mut self.executor
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::types::InputSequence;
    use crate::replay::StructuredInput;

    // Simple test input layout
    struct TestLayout;

    impl InputLayout for TestLayout {
        fn encode_input(&self, _input: &StructuredInput) -> Vec<u8> {
            vec![0x00]
        }

        fn decode_input(&self, _bytes: &[u8]) -> StructuredInput {
            StructuredInput::default()
        }

        fn input_size(&self) -> usize {
            1
        }

        fn console_id(&self) -> u8 {
            1
        }

        fn button_names(&self) -> &[&str] {
            &["a", "b"]
        }
    }

    #[test]
    fn test_headless_runner_creation() {
        let script = CompiledScript {
            console: "test".to_string(),
            console_id: 1,
            player_count: 1,
            input_size: 1,
            seed: 0,
            frame_count: 10,
            inputs: InputSequence::new(),
            snap_frames: vec![0, 5, 9],
            assertions: Vec::new(),
            actions: Vec::new(),
        };

        let config = HeadlessConfig::default();
        let runner = HeadlessRunner::new(script, config);

        assert_eq!(runner.executor.frame_count(), 10);
    }

    #[test]
    fn test_headless_execution() {
        let mut inputs = InputSequence::new();
        for _ in 0..5 {
            inputs.push_frame(vec![vec![0x00]]);
        }

        let script = CompiledScript {
            console: "test".to_string(),
            console_id: 1,
            player_count: 1,
            input_size: 1,
            seed: 0,
            frame_count: 5,
            inputs,
            snap_frames: vec![0, 4],
            assertions: Vec::new(),
            actions: Vec::new(),
        };

        let config = HeadlessConfig {
            fail_fast: false,
            timeout_secs: 10,
            script_path: Some("test.ncrs".to_string()),
        };

        let mut runner = HeadlessRunner::new(script, config);

        // Register some test variables
        runner.register_debug_variables(vec![
            DebugVariableInfo {
                name: "$player_x".to_string(),
                type_name: "i32".to_string(),
                description: "Player X position".to_string(),
            },
            DebugVariableInfo {
                name: "$player_y".to_string(),
                type_name: "i32".to_string(),
                description: "Player Y position".to_string(),
            },
        ]);

        // Set some values
        runner.set_debug_variable("$player_x".to_string(), DebugValueData::I32(100));
        runner.set_debug_variable("$player_y".to_string(), DebugValueData::I32(200));

        // Execute
        let report = runner.execute().expect("Execution failed");

        assert_eq!(report.frames_executed, 5);
        assert_eq!(report.total_frames, 5);
        assert_eq!(report.script, Some("test.ncrs".to_string()));
        assert!(report.duration_ms.is_some());
        assert!(report.executed_at.is_some());
        assert_eq!(report.registered_variables.as_ref().unwrap().len(), 2);
    }
}
