//! FFI declarations for Nethercore ZX
//!
//! Re-exports the canonical zx.rs FFI bindings.

// Include the canonical FFI bindings from nethercore/include/zx/
#[path = "../../../include/zx/mod.rs"]
mod zx;
pub use zx::*;

// Re-export button constants for convenience in examples
pub use button::*;
