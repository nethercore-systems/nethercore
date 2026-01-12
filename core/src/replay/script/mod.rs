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

mod ast;
mod compiler;
mod decompiler;
mod parser;
mod validation;

pub use ast::{
    ActionParamValue, AssertCondition, AssertValue, CompareOp, FrameEntry, InputValue,
    ReplayScript, StructuredInput,
};
pub use compiler::{
    CompileError, CompiledAction, CompiledAssertValue, CompiledAssertion, CompiledScript, Compiler,
    InputLayout,
};
pub use decompiler::decompile;
pub use parser::ParseError;
pub use validation::{ValidationError, validate_script};
