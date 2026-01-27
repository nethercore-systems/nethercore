//! Replay script AST types.

use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

/// Complete replay script file (TOML structure)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayScript {
    /// Console identifier (e.g., "zx")
    pub console: String,

    /// Random seed for deterministic execution
    #[serde(default)]
    pub seed: u64,

    /// Number of players
    #[serde(default = "default_players")]
    pub players: u8,

    /// Frame entries
    pub frames: Vec<FrameEntry>,
}

fn default_players() -> u8 {
    1
}

/// Single frame entry in the replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameEntry {
    /// Frame number
    pub f: u64,

    /// Player 1 input
    #[serde(default)]
    pub p1: Option<InputValue>,

    /// Player 2 input
    #[serde(default)]
    pub p2: Option<InputValue>,

    /// Player 3 input
    #[serde(default)]
    pub p3: Option<InputValue>,

    /// Player 4 input
    #[serde(default)]
    pub p4: Option<InputValue>,

    /// Capture debug variables before and after update()
    #[serde(default)]
    pub snap: bool,

    /// Request a screenshot capture after rendering this frame
    #[serde(default)]
    pub screenshot: bool,

    /// Assertion condition (e.g., "$velocity_y < 0")
    #[serde(default)]
    pub assert: Option<String>,

    /// Debug action to invoke (e.g., "Load Level")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action: Option<String>,

    /// Parameters for the debug action
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub action_params: Option<HashMap<String, ActionParamValue>>,
}

/// Value for a debug action parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ActionParamValue {
    /// Integer parameter
    Int(i32),
    /// Float parameter
    Float(f32),
    /// String parameter
    String(String),
    /// Boolean parameter
    Bool(bool),
}

/// Input value - can be symbolic, hex, or structured
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InputValue {
    /// Simple symbolic input: "idle", "a", "right+a"
    Symbolic(String),

    /// Hex bytes: [0x80, 0x80, 0x00, 0x00]
    HexBytes(Vec<u8>),

    /// Structured input for analog controllers
    Structured(StructuredInput),
}

/// Structured input for analog controllers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StructuredInput {
    /// Digital buttons: ["a", "b"]
    #[serde(default)]
    pub buttons: Vec<String>,

    /// Left stick: [x, y] where -1.0 to 1.0
    #[serde(default)]
    pub lstick: Option<[f32; 2]>,

    /// Right stick: [x, y] where -1.0 to 1.0
    #[serde(default)]
    pub rstick: Option<[f32; 2]>,

    /// Left trigger: 0.0 to 1.0
    #[serde(default)]
    pub lt: Option<f32>,

    /// Right trigger: 0.0 to 1.0
    #[serde(default)]
    pub rt: Option<f32>,
}

/// Parsed assertion condition
#[derive(Debug, Clone)]
pub struct AssertCondition {
    /// Variable name (e.g., "$player_x")
    pub variable: String,
    /// Comparison operator
    pub operator: CompareOp,
    /// Value to compare against
    pub value: AssertValue,
}

/// Comparison operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    /// ==
    Eq,
    /// !=
    Ne,
    /// <
    Lt,
    /// >
    Gt,
    /// <=
    Le,
    /// >=
    Ge,
}

impl std::fmt::Display for CompareOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompareOp::Eq => write!(f, "=="),
            CompareOp::Ne => write!(f, "!="),
            CompareOp::Lt => write!(f, "<"),
            CompareOp::Gt => write!(f, ">"),
            CompareOp::Le => write!(f, "<="),
            CompareOp::Ge => write!(f, ">="),
        }
    }
}

/// Value in an assertion
#[derive(Debug, Clone)]
pub enum AssertValue {
    /// Numeric literal
    Number(f64),
    /// Another variable
    Variable(String),
    /// Previous frame value
    PrevValue(String),
}

impl ReplayScript {
    /// Get the maximum frame number in the script
    pub fn max_frame(&self) -> u64 {
        self.frames.iter().map(|f| f.f).max().unwrap_or(0)
    }

    /// Get frames that have snap enabled
    pub fn snap_frames(&self) -> impl Iterator<Item = &FrameEntry> {
        self.frames.iter().filter(|f| f.snap)
    }

    /// Get frames that have assertions
    pub fn assert_frames(&self) -> impl Iterator<Item = &FrameEntry> {
        self.frames.iter().filter(|f| f.assert.is_some())
    }
}
