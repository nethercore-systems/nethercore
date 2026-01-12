//! Core types for the replay system
//!
//! This module defines the data structures used by both binary (.ncrp) and
//! script (.ncrs) replay formats.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Complete replay data (in-memory representation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Replay {
    pub header: ReplayHeader,
    pub inputs: InputSequence,
    pub checkpoints: Vec<Checkpoint>,
    pub assertions: Vec<Assertion>,
}

impl Default for Replay {
    fn default() -> Self {
        Self {
            header: ReplayHeader::default(),
            inputs: InputSequence::new(),
            checkpoints: Vec::new(),
            assertions: Vec::new(),
        }
    }
}

/// Lean header - no ROM identification (replays survive code changes and renames)
/// Extensibility via flags and reserved bytes (no version field needed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayHeader {
    /// Console identifier (e.g., 1 for ZX)
    pub console_id: u8,
    /// Number of players
    pub player_count: u8,
    /// Bytes per player per frame
    pub input_size: u8,
    /// Feature flags
    pub flags: ReplayFlags,
    /// Reserved for future use
    pub reserved: [u8; 4],
    /// Random seed for deterministic execution
    pub seed: u64,
    /// Total number of frames
    pub frame_count: u64,
}

impl Default for ReplayHeader {
    fn default() -> Self {
        Self {
            console_id: 1, // ZX
            player_count: 1,
            input_size: 8, // ZX input size
            flags: ReplayFlags::empty(),
            reserved: [0u8; 4],
            seed: 0,
            frame_count: 0,
        }
    }
}

bitflags::bitflags! {
    /// Replay feature flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct ReplayFlags: u8 {
        /// Replay contains state checkpoints for seeking
        const HAS_CHECKPOINTS = 0b0000_0001;
        /// Input stream is delta + LZ4 compressed
        const COMPRESSED_INPUTS = 0b0000_0010;
        /// Replay contains assertions (for script format)
        const HAS_ASSERTIONS = 0b0000_0100;
    }
}

// Manual serde implementation for ReplayFlags
impl Serialize for ReplayFlags {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.bits().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for ReplayFlags {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bits = u8::deserialize(deserializer)?;
        Ok(ReplayFlags::from_bits_truncate(bits))
    }
}

/// Sequence of inputs, indexed by frame
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InputSequence {
    /// Raw input bytes per frame per player
    /// frames[frame_idx][player_idx] = input_bytes
    frames: Vec<Vec<Vec<u8>>>,
}

impl InputSequence {
    /// Create a new empty input sequence
    pub fn new() -> Self {
        Self { frames: Vec::new() }
    }

    /// Add a frame of inputs for all players
    pub fn push_frame(&mut self, player_inputs: Vec<Vec<u8>>) {
        self.frames.push(player_inputs);
    }

    /// Get inputs for a specific frame
    pub fn get_frame(&self, frame: u64) -> Option<&Vec<Vec<u8>>> {
        self.frames.get(frame as usize)
    }

    /// Get the total number of frames
    pub fn frame_count(&self) -> u64 {
        self.frames.len() as u64
    }

    /// Iterate over all frames
    pub fn iter(&self) -> impl Iterator<Item = &Vec<Vec<u8>>> {
        self.frames.iter()
    }

    /// Check if the sequence is empty
    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

/// State checkpoint for seeking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    /// Frame number this checkpoint was taken at
    pub frame: u64,
    /// Serialized game state (WASM memory snapshot)
    pub state: Vec<u8>,
}

/// Script assertion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assertion {
    /// Frame number to evaluate assertion
    pub frame: u64,
    /// Optional name for the assertion
    pub name: Option<String>,
    /// The assertion expression
    pub expression: AssertExpr,
    /// Source line number (for error reporting)
    pub source_line: usize,
}

/// Assertion expression types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssertExpr {
    /// memory[addr] == value
    MemEq {
        addr: u32,
        value: i64,
    },
    /// memory[addr] != value
    MemNe {
        addr: u32,
        value: i64,
    },
    /// memory[addr] > value
    MemGt {
        addr: u32,
        value: i64,
    },
    /// memory[addr] < value
    MemLt {
        addr: u32,
        value: i64,
    },
    /// memory[addr] >= value
    MemGe {
        addr: u32,
        value: i64,
    },
    /// memory[addr] <= value
    MemLe {
        addr: u32,
        value: i64,
    },
    /// memory[addr] ~= value ± tolerance
    MemApprox {
        addr: u32,
        value: i64,
        tolerance: i64,
    },
    /// memory[addr] > prev(memory[addr])
    MemIncreased {
        addr: u32,
    },
    /// memory[addr] < prev(memory[addr])
    MemDecreased {
        addr: u32,
    },
    /// tick > value
    TickGt {
        value: u64,
    },
    /// Debug variable comparison (e.g., "$player_x > 100")
    VarEq {
        name: String,
        value: f64,
    },
    VarNe {
        name: String,
        value: f64,
    },
    VarGt {
        name: String,
        value: f64,
    },
    VarLt {
        name: String,
        value: f64,
    },
    VarGe {
        name: String,
        value: f64,
    },
    VarLe {
        name: String,
        value: f64,
    },
}

/// Log directive (uses debug-registered variable names)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogDirective {
    /// Frame number to log
    pub frame: u64,
    /// Format string with {} placeholders
    pub format: String,
    /// Variable names like "$player_x"
    pub variables: Vec<String>,
    /// Source line number
    pub source_line: usize,
}

/// Debug inspector capture directive
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectDirective {
    /// Frame number to capture
    pub frame: u64,
    /// When to capture (before/after update)
    pub timing: InspectTiming,
    /// Variables to capture (empty = all)
    pub variables: Vec<String>,
    /// Source line number
    pub source_line: usize,
}

/// When to capture debug variables
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InspectTiming {
    /// Before update()
    Pre,
    /// After update()
    Post,
    /// Capture both and compute delta
    Diff,
}

/// Debug variable value (from debug inspector)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugValue {
    pub name: String,
    pub type_name: String,
    pub value: DebugValueData,
}

/// Debug value data variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugValueData {
    I32(i32),
    U32(u32),
    I64(i64),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
    Bytes(Vec<u8>),
}

impl DebugValueData {
    /// Convert to f64 for numeric comparisons
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            DebugValueData::I32(v) => Some(*v as f64),
            DebugValueData::U32(v) => Some(*v as f64),
            DebugValueData::I64(v) => Some(*v as f64),
            DebugValueData::U64(v) => Some(*v as f64),
            DebugValueData::F32(v) => Some(*v as f64),
            DebugValueData::F64(v) => Some(*v),
            DebugValueData::Bool(v) => Some(if *v { 1.0 } else { 0.0 }),
            _ => None,
        }
    }
}

/// Breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Frame number to break at
    pub frame: u64,
    /// Optional message to display
    pub message: Option<String>,
    /// Optional condition (break only if true)
    pub condition: Option<AssertExpr>,
}

/// Snapshot captured during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Frame number
    pub frame: u64,
    /// Input applied this frame
    pub input: String,
    /// Variables before update()
    pub pre: BTreeMap<String, DebugValueData>,
    /// Variables after update()
    pub post: BTreeMap<String, DebugValueData>,
    /// Computed delta (only changed values)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta: Option<BTreeMap<String, String>>,
}

/// Assertion result from execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    /// Frame number
    pub frame: u64,
    /// Condition string
    pub condition: String,
    /// Whether the assertion passed
    pub passed: bool,
    /// Actual value observed
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<f64>,
    /// Expected value (for failed assertions)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_sequence_roundtrip() {
        let mut inputs = InputSequence::new();
        inputs.push_frame(vec![vec![0x0F], vec![0xF0]]);
        inputs.push_frame(vec![vec![0x1F], vec![0xE0]]);

        assert_eq!(inputs.frame_count(), 2);
        assert_eq!(inputs.get_frame(0), Some(&vec![vec![0x0F], vec![0xF0]]));
        assert_eq!(inputs.get_frame(1), Some(&vec![vec![0x1F], vec![0xE0]]));
        assert_eq!(inputs.get_frame(2), None);
    }

    #[test]
    fn test_replay_flags() {
        let flags = ReplayFlags::HAS_CHECKPOINTS | ReplayFlags::COMPRESSED_INPUTS;
        assert!(flags.contains(ReplayFlags::HAS_CHECKPOINTS));
        assert!(flags.contains(ReplayFlags::COMPRESSED_INPUTS));
        assert!(!flags.contains(ReplayFlags::HAS_ASSERTIONS));
        assert_eq!(flags.bits(), 0b011);
    }

    #[test]
    fn test_debug_value_as_f64() {
        assert_eq!(DebugValueData::I32(42).as_f64(), Some(42.0));
        // Use 0.5 which converts exactly between f32 and f64
        assert_eq!(DebugValueData::F32(0.5).as_f64(), Some(0.5));
        assert_eq!(DebugValueData::Bool(true).as_f64(), Some(1.0));
        assert_eq!(DebugValueData::Bool(false).as_f64(), Some(0.0));
        assert_eq!(DebugValueData::Vec2 { x: 1.0, y: 2.0 }.as_f64(), None);
    }
}
