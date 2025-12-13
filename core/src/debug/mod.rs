//! Debug inspection system
//!
//! Provides runtime value inspection and modification for fast iteration
//! during Emberware game development.
//!
//! # Overview
//!
//! Games register pointers to values they want to expose via FFI functions
//! during `init()`. The console provides:
//!
//! - Debug panel UI for viewing/editing registered values
//! - Frame control (pause, step, slow-mo) for precise observation
//! - Export to copy tuned values back to source code
//!
//! # Usage
//!
//! ## Game Side (WASM)
//!
//! ```rust,ignore
//! #[cfg(debug_assertions)]
//! extern "C" {
//!     fn debug_register_f32(name: *const i8, ptr: *const f32);
//!     fn debug_group_begin(name: *const i8);
//!     fn debug_group_end();
//! }
//!
//! static mut PLAYER_SPEED: f32 = 5.0;
//!
//! fn init() {
//!     #[cfg(debug_assertions)]
//!     unsafe {
//!         debug_group_begin(c"player");
//!         debug_register_f32(c"speed", &PLAYER_SPEED);
//!         debug_group_end();
//!     }
//! }
//! ```
//!
//! ## Console Side (Host)
//!
//! The debug panel is integrated into the game session and rendered via egui.
//! Values are read from and written to WASM linear memory.
//!
//! # Design Principles
//!
//! - **Zero ship overhead**: Debug code compiles out via `#[cfg(debug_assertions)]`
//! - **No code path divergence**: Same workflow, just add inspection
//! - **Game controls visualization**: Console edits data; games draw overlays
//! - **Cross-console**: Same FFI for all Emberware consoles

pub mod export;
pub mod ffi;
pub mod frame_control;
pub mod panel;
pub mod registry;
pub mod types;

// Re-export commonly used types
pub use ffi::{HasDebugRegistry, register_debug_ffi};
pub use frame_control::{FrameController, TIME_SCALE_OPTIONS};
pub use panel::DebugPanel;
pub use registry::{DebugRegistry, RegisteredValue, TreeNode};
pub use types::{Constraints, DebugValue, ValueType};
