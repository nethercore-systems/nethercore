//! Script execution engine
//!
//! Executes compiled scripts frame-by-frame, capturing snapshots and
//! evaluating assertions.

mod helpers;
mod report;

#[cfg(test)]
mod tests;

// Re-export public types
pub use report::{DebugVariableInfo, ExecutionReport, ReportSummary};

use crate::replay::script::{
    CompareOp, CompiledAction, CompiledAssertValue, CompiledAssertion, CompiledScript,
};
use crate::replay::types::{AssertionResult, DebugValueData, Snapshot};
use hashbrown::HashMap;
use helpers::{compute_delta, format_expected, map_to_btree};

/// Script executor state
pub struct ScriptExecutor {
    /// Compiled script to execute
    script: CompiledScript,
    /// Current frame number
    current_frame: u64,
    /// Previous frame's debug values (for $prev_ comparisons)
    prev_values: HashMap<String, DebugValueData>,
    /// Collected snapshots
    snapshots: Vec<Snapshot>,
    /// Collected assertion results
    assertion_results: Vec<AssertionResult>,
    /// Whether execution should stop (breakpoint or completion)
    stopped: bool,
    /// Stop reason if stopped
    stop_reason: Option<StopReason>,
}

/// Reason for stopping execution
#[derive(Debug, Clone)]
pub enum StopReason {
    /// Reached end of script
    Complete,
    /// Hit a breakpoint
    Breakpoint { frame: u64, message: Option<String> },
    /// Assertion failed (only with fail_fast mode)
    AssertionFailed { frame: u64, condition: String },
    /// Timeout exceeded
    Timeout,
    /// Error during execution
    Error(String),
}

/// Result of executing one frame
#[derive(Debug)]
pub enum StepResult {
    /// Continue to next frame
    Continue,
    /// Execution complete
    Complete,
    /// Hit breakpoint
    Breakpoint { message: Option<String> },
    /// Assertion failed (only if fail_fast)
    AssertionFailed { condition: String },
}

impl ScriptExecutor {
    /// Create a new executor for the given script
    pub fn new(script: CompiledScript) -> Self {
        Self {
            script,
            current_frame: 0,
            prev_values: HashMap::new(),
            snapshots: Vec::new(),
            assertion_results: Vec::new(),
            stopped: false,
            stop_reason: None,
        }
    }

    /// Get the current frame number
    pub fn current_frame(&self) -> u64 {
        self.current_frame
    }

    /// Get the total frame count
    pub fn frame_count(&self) -> u64 {
        self.script.frame_count
    }

    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        self.current_frame >= self.script.frame_count || self.stopped
    }

    /// Get the inputs for the current frame
    pub fn current_inputs(&self) -> Option<&Vec<Vec<u8>>> {
        self.script.inputs.get_frame(self.current_frame)
    }

    /// Check if current frame needs a snapshot
    pub fn needs_snapshot(&self) -> bool {
        self.script.snap_frames.contains(&self.current_frame)
    }

    /// Check if current frame needs a screenshot
    pub fn needs_screenshot(&self) -> bool {
        self.script.screenshot_frames.contains(&self.current_frame)
    }

    /// Get assertions for the current frame
    pub fn current_assertions(&self) -> Vec<&CompiledAssertion> {
        self.script
            .assertions
            .iter()
            .filter(|a| a.frame == self.current_frame)
            .collect()
    }

    /// Get debug actions for the current frame
    ///
    /// Actions are invoked before inputs are applied. Multiple actions
    /// can be specified for the same frame.
    pub fn current_actions(&self) -> Vec<&CompiledAction> {
        self.script
            .actions
            .iter()
            .filter(|a| a.frame == self.current_frame)
            .collect()
    }

    /// Check if the current frame has any debug actions
    pub fn has_actions(&self) -> bool {
        self.script
            .actions
            .iter()
            .any(|a| a.frame == self.current_frame)
    }

    /// Record pre-update snapshot values
    pub fn capture_pre_snapshot(
        &mut self,
        values: HashMap<String, DebugValueData>,
    ) -> HashMap<String, DebugValueData> {
        values
    }

    /// Record post-update snapshot values and create snapshot entry
    pub fn capture_post_snapshot(
        &mut self,
        pre_values: HashMap<String, DebugValueData>,
        post_values: HashMap<String, DebugValueData>,
        input_string: String,
    ) {
        // Compute delta
        let delta = compute_delta(&pre_values, &post_values);

        let pre_ordered = map_to_btree(pre_values);
        let post_ordered = map_to_btree(post_values.clone());

        self.snapshots.push(Snapshot {
            frame: self.current_frame,
            input: input_string,
            pre: pre_ordered,
            post: post_ordered,
            delta,
        });

        // Store for next frame's $prev_ comparisons
        self.prev_values = post_values;
    }

    /// Evaluate an assertion
    pub fn evaluate_assertion(
        &mut self,
        assertion: &CompiledAssertion,
        values: &HashMap<String, DebugValueData>,
        fail_fast: bool,
    ) -> bool {
        let actual = values.get(&assertion.variable).and_then(|v| v.as_f64());

        let expected = match &assertion.value {
            CompiledAssertValue::Number(n) => Some(*n),
            CompiledAssertValue::Variable(name) => values.get(name).and_then(|v| v.as_f64()),
            CompiledAssertValue::PrevValue(name) => {
                let full_name = format!("${}", name);
                self.prev_values.get(&full_name).and_then(|v| v.as_f64())
            }
        };

        let passed = match (actual, expected) {
            (Some(a), Some(e)) => match assertion.operator {
                CompareOp::Eq => (a - e).abs() < f64::EPSILON,
                CompareOp::Ne => (a - e).abs() >= f64::EPSILON,
                CompareOp::Lt => a < e,
                CompareOp::Gt => a > e,
                CompareOp::Le => a <= e,
                CompareOp::Ge => a >= e,
            },
            _ => false,
        };

        self.assertion_results.push(AssertionResult {
            frame: self.current_frame,
            condition: assertion.condition.clone(),
            passed,
            actual,
            expected: if passed {
                None
            } else {
                Some(format!(
                    "{} {}",
                    assertion.operator,
                    format_expected(&assertion.value, expected)
                ))
            },
        });

        if !passed && fail_fast {
            self.stopped = true;
            self.stop_reason = Some(StopReason::AssertionFailed {
                frame: self.current_frame,
                condition: assertion.condition.clone(),
            });
        }

        passed
    }

    /// Record an assertion result
    ///
    /// Used by headless runners to manually record assertion results.
    pub fn record_assertion_result(
        &mut self,
        frame: u64,
        condition: String,
        passed: bool,
        actual: Option<f64>,
        expected: Option<String>,
    ) {
        self.assertion_results.push(AssertionResult {
            frame,
            condition,
            passed,
            actual,
            expected,
        });
    }

    /// Advance to the next frame
    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
        if self.current_frame >= self.script.frame_count {
            self.stopped = true;
            self.stop_reason = Some(StopReason::Complete);
        }
    }

    /// Generate the execution report
    pub fn generate_report(&self) -> ExecutionReport {
        let passed = self.assertion_results.iter().filter(|r| r.passed).count();
        let failed = self.assertion_results.iter().filter(|r| !r.passed).count();

        ExecutionReport {
            version: "1.0".to_string(),
            script: None,      // Set by caller if available
            executed_at: None, // Set by caller if available
            duration_ms: None, // Set by caller if available
            console: self.script.console.clone(),
            seed: self.script.seed,
            frames_executed: self.current_frame,
            total_frames: self.script.frame_count,
            snapshots: self.snapshots.clone(),
            assertions: self.assertion_results.clone(),
            registered_variables: None, // Set by caller if available
            summary: ReportSummary {
                frames_with_snap: self.snapshots.len(),
                assertions_passed: passed,
                assertions_failed: failed,
                status: if failed > 0 {
                    "FAILED".to_string()
                } else {
                    "PASSED".to_string()
                },
            },
        }
    }

    /// Stop execution with an error
    pub fn stop_with_error(&mut self, error: String) {
        self.stopped = true;
        self.stop_reason = Some(StopReason::Error(error));
    }

    /// Get the stop reason
    pub fn stop_reason(&self) -> Option<&StopReason> {
        self.stop_reason.as_ref()
    }
}
