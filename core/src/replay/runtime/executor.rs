//! Script execution engine
//!
//! Executes compiled scripts frame-by-frame, capturing snapshots and
//! evaluating assertions.

use crate::replay::script::{
    CompiledAction, CompiledAssertion, CompiledAssertValue, CompiledScript, CompareOp,
};
use crate::replay::types::{AssertionResult, DebugValueData, Snapshot};
use hashbrown::HashMap;

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
    pub fn capture_pre_snapshot(&mut self, values: HashMap<String, DebugValueData>) -> HashMap<String, DebugValueData> {
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

        self.snapshots.push(Snapshot {
            frame: self.current_frame,
            input: input_string,
            pre: pre_values,
            post: post_values.clone(),
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
                Some(format!("{} {}", assertion.operator, format_expected(&assertion.value, expected)))
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
            console: self.script.console.clone(),
            seed: self.script.seed,
            frames_executed: self.current_frame,
            total_frames: self.script.frame_count,
            snapshots: self.snapshots.clone(),
            assertions: self.assertion_results.clone(),
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

/// Compute the delta between pre and post values
fn compute_delta(
    pre: &HashMap<String, DebugValueData>,
    post: &HashMap<String, DebugValueData>,
) -> Option<HashMap<String, String>> {
    let mut delta = HashMap::new();

    for (name, post_value) in post {
        if let Some(pre_value) = pre.get(name) {
            let changed = match (pre_value, post_value) {
                (DebugValueData::I32(a), DebugValueData::I32(b)) if a != b => {
                    Some(format!("{}", b - a))
                }
                (DebugValueData::U32(a), DebugValueData::U32(b)) if a != b => {
                    Some(format!("{}", (*b as i64) - (*a as i64)))
                }
                (DebugValueData::F32(a), DebugValueData::F32(b)) if (a - b).abs() > f32::EPSILON => {
                    Some(format!("{:.2}", b - a))
                }
                (DebugValueData::F64(a), DebugValueData::F64(b)) if (a - b).abs() > f64::EPSILON => {
                    Some(format!("{:.2}", b - a))
                }
                (DebugValueData::Bool(a), DebugValueData::Bool(b)) if a != b => {
                    Some(format!("{} → {}", a, b))
                }
                _ => None,
            };

            if let Some(change) = changed {
                delta.insert(name.clone(), change);
            }
        }
    }

    if delta.is_empty() {
        None
    } else {
        Some(delta)
    }
}

/// Format the expected value for error messages
fn format_expected(value: &CompiledAssertValue, resolved: Option<f64>) -> String {
    match value {
        CompiledAssertValue::Number(n) => format!("{}", n),
        CompiledAssertValue::Variable(name) => {
            if let Some(v) = resolved {
                format!("{} ({})", name, v)
            } else {
                format!("{} (undefined)", name)
            }
        }
        CompiledAssertValue::PrevValue(name) => {
            if let Some(v) = resolved {
                format!("$prev_{} ({})", name, v)
            } else {
                format!("$prev_{} (undefined)", name)
            }
        }
    }
}

/// Execution report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExecutionReport {
    /// Console name
    pub console: String,
    /// Random seed used
    pub seed: u64,
    /// Frames executed
    pub frames_executed: u64,
    /// Total frames in script
    pub total_frames: u64,
    /// Captured snapshots
    pub snapshots: Vec<Snapshot>,
    /// Assertion results
    pub assertions: Vec<AssertionResult>,
    /// Summary statistics
    pub summary: ReportSummary,
}

/// Report summary
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReportSummary {
    /// Number of frames with snapshots
    pub frames_with_snap: usize,
    /// Number of passed assertions
    pub assertions_passed: usize,
    /// Number of failed assertions
    pub assertions_failed: usize,
    /// Overall status
    pub status: String,
}

impl ExecutionReport {
    /// Serialize to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::replay::script::CompiledAssertion;
    use crate::replay::types::InputSequence;

    #[test]
    fn test_executor_basic() {
        let mut inputs = InputSequence::new();
        inputs.push_frame(vec![vec![0x00]]);
        inputs.push_frame(vec![vec![0x10]]);

        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![0],
            assertions: vec![],
            actions: vec![],
            frame_count: 2,
        };

        let mut executor = ScriptExecutor::new(script);

        assert_eq!(executor.current_frame(), 0);
        assert!(!executor.is_complete());
        assert!(executor.needs_snapshot());

        executor.advance_frame();
        assert_eq!(executor.current_frame(), 1);
        assert!(!executor.needs_snapshot());

        executor.advance_frame();
        assert!(executor.is_complete());
    }

    #[test]
    fn test_assertion_evaluation_all_operators() {
        let inputs = InputSequence::new();

        // Test Eq operator
        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs: inputs.clone(),
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x == 100".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Eq,
                value: CompiledAssertValue::Number(100.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);
        let mut values = HashMap::new();
        values.insert("$x".to_string(), DebugValueData::F32(100.0));
        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(99.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));

        // Test Ne operator
        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs: inputs.clone(),
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x != 0".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Ne,
                value: CompiledAssertValue::Number(0.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);
        values.insert("$x".to_string(), DebugValueData::F32(50.0));
        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(0.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));

        // Test Lt operator
        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs: inputs.clone(),
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x < 100".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Lt,
                value: CompiledAssertValue::Number(100.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);
        values.insert("$x".to_string(), DebugValueData::F32(50.0));
        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(100.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));

        // Test Gt operator
        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs: inputs.clone(),
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x > 100".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Gt,
                value: CompiledAssertValue::Number(100.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);
        values.insert("$x".to_string(), DebugValueData::F32(150.0));
        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(100.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));

        // Test Le operator
        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs: inputs.clone(),
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x <= 100".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Le,
                value: CompiledAssertValue::Number(100.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);
        values.insert("$x".to_string(), DebugValueData::F32(100.0));
        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(50.0));
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(101.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));

        // Test Ge operator
        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x >= 100".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Ge,
                value: CompiledAssertValue::Number(100.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);
        values.insert("$x".to_string(), DebugValueData::F32(100.0));
        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(150.0));
        assert!(executor.evaluate_assertion(assertion, &values, false));
        values.insert("$x".to_string(), DebugValueData::F32(99.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));
    }

    #[test]
    fn test_assertion_prev_value() {
        let inputs = InputSequence::new();

        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![0, 1],
            assertions: vec![CompiledAssertion {
                frame: 1,
                condition: "$x > $prev_x".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Gt,
                value: CompiledAssertValue::PrevValue("x".to_string()),
            }],
            actions: vec![],
            frame_count: 2,
        };
        let mut executor = ScriptExecutor::new(script);

        // Frame 0: capture initial values
        let mut values = HashMap::new();
        values.insert("$x".to_string(), DebugValueData::F32(100.0));
        executor.capture_post_snapshot(values.clone(), values.clone(), "idle".to_string());
        executor.advance_frame();

        // Frame 1: x increased, assertion should pass
        values.insert("$x".to_string(), DebugValueData::F32(150.0));
        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));

        // Frame 1: x decreased, assertion should fail
        values.insert("$x".to_string(), DebugValueData::F32(50.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));
    }

    #[test]
    fn test_assertion_variable_to_variable() {
        let inputs = InputSequence::new();

        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$player_x > $enemy_x".to_string(),
                variable: "$player_x".to_string(),
                operator: CompareOp::Gt,
                value: CompiledAssertValue::Variable("$enemy_x".to_string()),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);

        let mut values = HashMap::new();
        values.insert("$player_x".to_string(), DebugValueData::F32(200.0));
        values.insert("$enemy_x".to_string(), DebugValueData::F32(100.0));

        let assertion = &executor.current_assertions()[0].clone();
        assert!(executor.evaluate_assertion(assertion, &values, false));

        // Flip positions - should fail
        values.insert("$player_x".to_string(), DebugValueData::F32(50.0));
        assert!(!executor.evaluate_assertion(assertion, &values, false));

        // Missing variable - should fail
        values.remove("$enemy_x");
        assert!(!executor.evaluate_assertion(assertion, &values, false));
    }

    #[test]
    fn test_assertion_different_value_types() {
        let inputs = InputSequence::new();

        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x == 100".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Eq,
                value: CompiledAssertValue::Number(100.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor = ScriptExecutor::new(script);
        let assertion = &executor.current_assertions()[0].clone();

        // Test I32
        let mut values = HashMap::new();
        values.insert("$x".to_string(), DebugValueData::I32(100));
        assert!(executor.evaluate_assertion(assertion, &values, false));

        // Test U32
        values.insert("$x".to_string(), DebugValueData::U32(100));
        assert!(executor.evaluate_assertion(assertion, &values, false));

        // Test F64
        values.insert("$x".to_string(), DebugValueData::F64(100.0));
        assert!(executor.evaluate_assertion(assertion, &values, false));

        // Test Bool (true = 1.0)
        values.insert("$x".to_string(), DebugValueData::Bool(true));
        let script2 = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs: InputSequence::new(),
            snap_frames: vec![],
            assertions: vec![CompiledAssertion {
                frame: 0,
                condition: "$x == 1".to_string(),
                variable: "$x".to_string(),
                operator: CompareOp::Eq,
                value: CompiledAssertValue::Number(1.0),
            }],
            actions: vec![],
            frame_count: 1,
        };
        let mut executor2 = ScriptExecutor::new(script2);
        let assertion2 = &executor2.current_assertions()[0].clone();
        assert!(executor2.evaluate_assertion(assertion2, &values, false));
    }

    #[test]
    fn test_delta_computation() {
        let mut pre = HashMap::new();
        pre.insert("$x".to_string(), DebugValueData::F32(100.0));
        pre.insert("$y".to_string(), DebugValueData::F32(200.0));
        pre.insert("$grounded".to_string(), DebugValueData::Bool(true));

        let mut post = HashMap::new();
        post.insert("$x".to_string(), DebugValueData::F32(105.0));
        post.insert("$y".to_string(), DebugValueData::F32(200.0)); // unchanged
        post.insert("$grounded".to_string(), DebugValueData::Bool(false));

        let delta = compute_delta(&pre, &post).unwrap();

        assert!(delta.contains_key("$x"));
        assert!(!delta.contains_key("$y")); // No change
        assert!(delta.contains_key("$grounded"));
        assert!(delta.get("$grounded").unwrap().contains("→"));
    }

    #[test]
    fn test_executor_with_actions() {
        use crate::replay::script::{ActionParamValue, CompiledAction};

        let inputs = InputSequence::new();
        let mut params = HashMap::new();
        params.insert("level".to_string(), ActionParamValue::Int(2));

        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![],
            assertions: vec![],
            actions: vec![
                CompiledAction {
                    frame: 0,
                    name: "Load Level".to_string(),
                    params,
                },
            ],
            frame_count: 2,
        };

        let executor = ScriptExecutor::new(script);

        // Frame 0 has actions
        assert!(executor.has_actions());
        let actions = executor.current_actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "Load Level");
        assert!(actions[0].params.contains_key("level"));
    }

    #[test]
    fn test_executor_multiple_actions_same_frame() {
        use crate::replay::script::{ActionParamValue, CompiledAction};

        let inputs = InputSequence::new();

        let mut params1 = HashMap::new();
        params1.insert("difficulty".to_string(), ActionParamValue::Int(3));

        let mut params2 = HashMap::new();
        params2.insert("players".to_string(), ActionParamValue::Int(2));

        let mut params3 = HashMap::new();
        params3.insert("speed".to_string(), ActionParamValue::Float(1.5));

        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![],
            assertions: vec![],
            actions: vec![
                CompiledAction {
                    frame: 0,
                    name: "Set Difficulty".to_string(),
                    params: params1,
                },
                CompiledAction {
                    frame: 0,
                    name: "Set Players".to_string(),
                    params: params2,
                },
                CompiledAction {
                    frame: 1,
                    name: "Set Speed".to_string(),
                    params: params3,
                },
            ],
            frame_count: 3,
        };

        let mut executor = ScriptExecutor::new(script);

        // Frame 0 should have 2 actions
        assert!(executor.has_actions());
        let actions = executor.current_actions();
        assert_eq!(actions.len(), 2);
        assert_eq!(actions[0].name, "Set Difficulty");
        assert_eq!(actions[1].name, "Set Players");

        // Advance to frame 1
        executor.advance_frame();
        assert!(executor.has_actions());
        let actions = executor.current_actions();
        assert_eq!(actions.len(), 1);
        assert_eq!(actions[0].name, "Set Speed");

        // Advance to frame 2 - no actions
        executor.advance_frame();
        assert!(!executor.has_actions());
        assert!(executor.current_actions().is_empty());
    }

    #[test]
    fn test_executor_action_all_param_types() {
        use crate::replay::script::{ActionParamValue, CompiledAction};

        let inputs = InputSequence::new();

        let mut params = HashMap::new();
        params.insert("int_val".to_string(), ActionParamValue::Int(42));
        params.insert("float_val".to_string(), ActionParamValue::Float(3.14));
        params.insert("string_val".to_string(), ActionParamValue::String("hello".to_string()));
        params.insert("bool_val".to_string(), ActionParamValue::Bool(true));

        let script = CompiledScript {
            console: "zx".to_string(),
            console_id: 1,
            seed: 0,
            player_count: 1,
            input_size: 1,
            inputs,
            snap_frames: vec![],
            assertions: vec![],
            actions: vec![CompiledAction {
                frame: 0,
                name: "Test All Types".to_string(),
                params,
            }],
            frame_count: 1,
        };

        let executor = ScriptExecutor::new(script);
        let actions = executor.current_actions();
        assert_eq!(actions.len(), 1);

        let params = &actions[0].params;
        assert!(matches!(params.get("int_val"), Some(ActionParamValue::Int(42))));
        match params.get("float_val") {
            Some(ActionParamValue::Float(f)) => assert!((*f - 3.14).abs() < 0.001),
            _ => panic!("Expected Float"),
        }
        assert!(matches!(
            params.get("string_val"),
            Some(ActionParamValue::String(s)) if s == "hello"
        ));
        assert!(matches!(params.get("bool_val"), Some(ActionParamValue::Bool(true))));
    }
}
