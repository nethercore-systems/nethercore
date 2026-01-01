//! Script replay format (.ncrs)
//!
//! The script format is a human-readable TOML format designed for:
//! - AI-assisted debugging (LLMs can write and analyze)
//! - Manual test case creation
//! - Debugging with snap/assert flags
//!
//! # Example Script
//!
//! ```toml
//! console = "zx"
//! seed = 12345
//! players = 1
//!
//! frames = [
//!   { f = 0, p1 = "idle", snap = true },
//!   { f = 1, p1 = "a", snap = true, assert = "$velocity_y < 0" },
//!   { f = 60, p1 = "idle", snap = true },
//! ]
//! ```
//!
//! # Input Formats
//!
//! - **Symbolic**: `"idle"`, `"a"`, `"right+a"`, `"up+right+b"`
//! - **Hex bytes**: `[0x80, 0x80, 0x00, 0x00]`
//! - **Structured**: `{ buttons = ["a"], lstick = [1.0, 0.0], rt = 0.8 }`

mod compiler;
mod decompiler;
mod parser;

pub use compiler::{
    CompiledAction, CompiledAssertion, CompiledAssertValue, CompiledScript, CompileError, Compiler,
    InputLayout,
};
pub use decompiler::decompile;
pub use parser::{
    ActionParamValue, AssertCondition, AssertValue, CompareOp, FrameEntry, InputValue, ParseError,
    ReplayScript, StructuredInput,
};
