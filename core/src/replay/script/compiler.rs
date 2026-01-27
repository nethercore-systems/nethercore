//! Script compiler
//!
//! Converts parsed TOML scripts into executable form.

use super::ast::{
    ActionParamValue, AssertCondition, AssertValue, CompareOp, InputValue, ReplayScript,
    StructuredInput,
};
use super::validation::validate_script;
use crate::replay::types::*;
use hashbrown::HashMap;

/// Console-specific input encoding
pub trait InputLayout: Send + Sync {
    /// Encode structured input to raw bytes
    fn encode_input(&self, input: &StructuredInput) -> Vec<u8>;

    /// Decode raw bytes to structured input
    fn decode_input(&self, bytes: &[u8]) -> StructuredInput;

    /// Get the input size in bytes
    fn input_size(&self) -> usize;

    /// Get the console ID
    fn console_id(&self) -> u8;

    /// Get available button names
    fn button_names(&self) -> &[&str];

    /// Encode symbolic buttons to raw bytes
    fn encode_buttons(&self, buttons: &[String]) -> Vec<u8> {
        let structured = StructuredInput {
            buttons: buttons.to_vec(),
            ..Default::default()
        };
        self.encode_input(&structured)
    }

    /// Decode raw bytes to symbolic button names
    fn decode_buttons(&self, bytes: &[u8]) -> Vec<String> {
        self.decode_input(bytes).buttons
    }
}

/// Compiled script ready for execution
#[derive(Debug, Default)]
pub struct CompiledScript {
    /// Console identifier
    pub console: String,
    /// Console ID (numeric)
    pub console_id: u8,
    /// Random seed
    pub seed: u64,
    /// Number of players
    pub player_count: u8,
    /// Input size in bytes per player
    pub input_size: u8,
    /// Compiled input sequence
    pub inputs: InputSequence,
    /// Frames that need snapshots
    pub snap_frames: Vec<u64>,
    /// Frames that need screenshot capture
    pub screenshot_frames: Vec<u64>,
    /// Assertions to evaluate
    pub assertions: Vec<CompiledAssertion>,
    /// Debug actions to invoke
    pub actions: Vec<CompiledAction>,
    /// Total frame count (max frame + 1)
    pub frame_count: u64,
}

/// Compiled debug action ready for invocation
#[derive(Debug, Clone)]
pub struct CompiledAction {
    /// Frame to invoke on
    pub frame: u64,
    /// Action name (matches registered debug action)
    pub name: String,
    /// Parameters to pass to the action
    pub params: HashMap<String, ActionParamValue>,
}

/// Compiled assertion ready for evaluation
#[derive(Debug, Clone)]
pub struct CompiledAssertion {
    /// Frame to evaluate on
    pub frame: u64,
    /// Original condition string
    pub condition: String,
    /// Variable name
    pub variable: String,
    /// Comparison operator
    pub operator: CompareOp,
    /// Value to compare against
    pub value: CompiledAssertValue,
}

/// Compiled assertion value
#[derive(Debug, Clone)]
pub enum CompiledAssertValue {
    /// Numeric constant
    Number(f64),
    /// Another variable
    Variable(String),
    /// Previous frame value of a variable
    PrevValue(String),
}

/// Script compiler
pub struct Compiler<'a> {
    layout: &'a dyn InputLayout,
}

impl<'a> Compiler<'a> {
    /// Create a new compiler with the given input layout
    pub fn new(layout: &'a dyn InputLayout) -> Self {
        Self { layout }
    }

    /// Compile a parsed script into executable form
    pub fn compile(&self, script: &ReplayScript) -> Result<CompiledScript, CompileError> {
        validate_script(script).map_err(CompileError::Validation)?;

        // Build frame map for sparse-to-dense conversion
        let max_frame = script.max_frame();
        let mut frame_inputs: HashMap<u64, Vec<Option<InputValue>>> = HashMap::new();
        let mut snap_frames = Vec::new();
        let mut screenshot_frames = Vec::new();
        let mut assertions = Vec::new();
        let mut actions = Vec::new();

        // Process each frame entry
        for entry in &script.frames {
            // Collect player inputs
            let player_inputs = vec![
                entry.p1.clone(),
                entry.p2.clone(),
                entry.p3.clone(),
                entry.p4.clone(),
            ];
            frame_inputs.insert(entry.f, player_inputs);

            // Track snap frames
            if entry.snap {
                snap_frames.push(entry.f);
            }
            if entry.screenshot {
                screenshot_frames.push(entry.f);
            }

            // Compile assertions
            if let Some(ref assert_str) = entry.assert {
                let cond = AssertCondition::parse(assert_str)
                    .map_err(|e| CompileError::InvalidAssertion(e.to_string()))?;

                assertions.push(CompiledAssertion {
                    frame: entry.f,
                    condition: assert_str.clone(),
                    variable: cond.variable,
                    operator: cond.operator,
                    value: match cond.value {
                        AssertValue::Number(n) => CompiledAssertValue::Number(n),
                        AssertValue::Variable(v) => CompiledAssertValue::Variable(v),
                        AssertValue::PrevValue(v) => CompiledAssertValue::PrevValue(v),
                    },
                });
            }

            // Compile debug actions
            if let Some(ref action_name) = entry.action {
                actions.push(CompiledAction {
                    frame: entry.f,
                    name: action_name.clone(),
                    params: entry.action_params.clone().unwrap_or_default(),
                });
            }
        }

        // Build dense input sequence (filling gaps with idle)
        let mut inputs = InputSequence::new();
        let idle_input = vec![0u8; self.layout.input_size()];
        let player_count = script.players as usize;

        for frame in 0..=max_frame {
            let mut frame_bytes = Vec::with_capacity(player_count);

            if let Some(player_inputs) = frame_inputs.get(&frame) {
                for player_idx in 0..player_count {
                    let input = player_inputs.get(player_idx).and_then(|v| v.as_ref());
                    let bytes = match input {
                        Some(InputValue::Symbolic(s)) => {
                            let buttons = InputValue::parse_symbolic(s);
                            self.layout.encode_buttons(&buttons)
                        }
                        Some(InputValue::HexBytes(bytes)) => bytes.clone(),
                        Some(InputValue::Structured(s)) => self.layout.encode_input(s),
                        None => idle_input.clone(),
                    };
                    frame_bytes.push(bytes);
                }
            } else {
                // Frame not specified, use idle for all players
                for _ in 0..player_count {
                    frame_bytes.push(idle_input.clone());
                }
            }

            inputs.push_frame(frame_bytes);
        }

        Ok(CompiledScript {
            console: script.console.clone(),
            console_id: self.layout.console_id(),
            seed: script.seed,
            player_count: script.players,
            input_size: self.layout.input_size() as u8,
            inputs,
            snap_frames,
            screenshot_frames,
            assertions,
            actions,
            frame_count: max_frame + 1,
        })
    }
}

/// Compilation errors
#[derive(Debug)]
pub enum CompileError {
    /// Script validation failed
    Validation(super::validation::ValidationError),
    /// Invalid assertion syntax
    InvalidAssertion(String),
    /// Unknown button name
    UnknownButton(String),
    /// Console mismatch
    ConsoleMismatch { expected: String, got: String },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Validation(e) => write!(f, "Validation error: {}", e),
            CompileError::InvalidAssertion(e) => write!(f, "Invalid assertion: {}", e),
            CompileError::UnknownButton(b) => write!(f, "Unknown button: {}", b),
            CompileError::ConsoleMismatch { expected, got } => {
                write!(f, "Console mismatch: expected {}, got {}", expected, got)
            }
        }
    }
}

impl std::error::Error for CompileError {}

#[cfg(test)]
mod tests {
    use super::super::ast::FrameEntry;
    use super::*;

    /// Mock input layout for testing
    struct MockLayout;

    impl InputLayout for MockLayout {
        fn encode_input(&self, input: &StructuredInput) -> Vec<u8> {
            let mut byte = 0u8;
            for button in &input.buttons {
                match button.as_str() {
                    "up" => byte |= 0x01,
                    "down" => byte |= 0x02,
                    "left" => byte |= 0x04,
                    "right" => byte |= 0x08,
                    "a" => byte |= 0x10,
                    "b" => byte |= 0x20,
                    _ => {}
                }
            }
            vec![byte]
        }

        fn decode_input(&self, bytes: &[u8]) -> StructuredInput {
            let byte = bytes.first().copied().unwrap_or(0);
            let mut buttons = Vec::new();
            if byte & 0x01 != 0 {
                buttons.push("up".to_string());
            }
            if byte & 0x02 != 0 {
                buttons.push("down".to_string());
            }
            if byte & 0x04 != 0 {
                buttons.push("left".to_string());
            }
            if byte & 0x08 != 0 {
                buttons.push("right".to_string());
            }
            if byte & 0x10 != 0 {
                buttons.push("a".to_string());
            }
            if byte & 0x20 != 0 {
                buttons.push("b".to_string());
            }
            StructuredInput {
                buttons,
                ..Default::default()
            }
        }

        fn input_size(&self) -> usize {
            1
        }

        fn console_id(&self) -> u8 {
            1
        }

        fn button_names(&self) -> &[&str] {
            &["up", "down", "left", "right", "a", "b"]
        }
    }

    #[test]
    fn test_compile_basic_script() {
        let script = ReplayScript {
            console: "zx".to_string(),
            seed: 12345,
            players: 1,
            frames: vec![
                FrameEntry {
                    f: 0,
                    p1: Some(InputValue::Symbolic("idle".to_string())),
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: true,
                    screenshot: false,
                    assert: None,
                    action: None,
                    action_params: None,
                },
                FrameEntry {
                    f: 1,
                    p1: Some(InputValue::Symbolic("right+a".to_string())),
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: false,
                    assert: Some("$player_x > 0".to_string()),
                    action: None,
                    action_params: None,
                },
            ],
        };

        let layout = MockLayout;
        let compiler = Compiler::new(&layout);
        let compiled = compiler.compile(&script).unwrap();

        assert_eq!(compiled.console, "zx");
        assert_eq!(compiled.seed, 12345);
        assert_eq!(compiled.player_count, 1);
        assert_eq!(compiled.inputs.frame_count(), 2);
        assert_eq!(compiled.snap_frames, vec![0]);
        assert_eq!(compiled.assertions.len(), 1);

        // Check frame 0 is idle (0x00)
        assert_eq!(compiled.inputs.get_frame(0), Some(&vec![vec![0x00]]));

        // Check frame 1 is right+a (0x08 | 0x10 = 0x18)
        assert_eq!(compiled.inputs.get_frame(1), Some(&vec![vec![0x18]]));
    }

    #[test]
    fn test_compile_sparse_frames() {
        let script = ReplayScript {
            console: "zx".to_string(),
            seed: 0,
            players: 1,
            frames: vec![
                FrameEntry {
                    f: 0,
                    p1: Some(InputValue::Symbolic("a".to_string())),
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: false,
                    assert: None,
                    action: None,
                    action_params: None,
                },
                FrameEntry {
                    f: 5,
                    p1: Some(InputValue::Symbolic("b".to_string())),
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: false,
                    assert: None,
                    action: None,
                    action_params: None,
                },
            ],
        };

        let layout = MockLayout;
        let compiler = Compiler::new(&layout);
        let compiled = compiler.compile(&script).unwrap();

        // Should have 6 frames (0-5)
        assert_eq!(compiled.inputs.frame_count(), 6);

        // Frame 0 = a (0x10)
        assert_eq!(compiled.inputs.get_frame(0), Some(&vec![vec![0x10]]));

        // Frames 1-4 = idle (0x00)
        for f in 1..5 {
            assert_eq!(compiled.inputs.get_frame(f), Some(&vec![vec![0x00]]));
        }

        // Frame 5 = b (0x20)
        assert_eq!(compiled.inputs.get_frame(5), Some(&vec![vec![0x20]]));
    }

    #[test]
    fn test_compile_with_actions() {
        use super::ActionParamValue;

        let mut params = HashMap::new();
        params.insert("level".to_string(), ActionParamValue::Int(2));

        let script = ReplayScript {
            console: "zx".to_string(),
            seed: 0,
            players: 1,
            frames: vec![
                FrameEntry {
                    f: 0,
                    p1: None,
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: false,
                    assert: None,
                    action: Some("Load Level".to_string()),
                    action_params: Some(params),
                },
                FrameEntry {
                    f: 1,
                    p1: Some(InputValue::Symbolic("a".to_string())),
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: true,
                    screenshot: false,
                    assert: None,
                    action: None,
                    action_params: None,
                },
            ],
        };

        let layout = MockLayout;
        let compiler = Compiler::new(&layout);
        let compiled = compiler.compile(&script).unwrap();

        // Should have 1 action
        assert_eq!(compiled.actions.len(), 1);
        assert_eq!(compiled.actions[0].frame, 0);
        assert_eq!(compiled.actions[0].name, "Load Level");
        assert!(compiled.actions[0].params.contains_key("level"));
    }

    #[test]
    fn test_compile_with_screenshots() {
        let script = ReplayScript {
            console: "zx".to_string(),
            seed: 0,
            players: 1,
            frames: vec![
                FrameEntry {
                    f: 0,
                    p1: None,
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: true,
                    assert: None,
                    action: None,
                    action_params: None,
                },
                FrameEntry {
                    f: 10,
                    p1: Some(InputValue::Symbolic("a".to_string())),
                    p2: None,
                    p3: None,
                    p4: None,
                    snap: false,
                    screenshot: true,
                    assert: None,
                    action: None,
                    action_params: None,
                },
            ],
        };

        let layout = MockLayout;
        let compiler = Compiler::new(&layout);
        let compiled = compiler.compile(&script).unwrap();

        assert_eq!(compiled.screenshot_frames, vec![0, 10]);
    }
}
