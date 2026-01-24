//! FFI declarations for Nethercore ZX
//!
//! Re-exports the canonical zx.rs FFI bindings.

// Include the canonical FFI bindings from nethercore/include/zx.rs
#[path = "../../../include/zx.rs"]
mod zx;
pub use zx::*;

// Re-export button constants for convenience in examples
pub use button::*;
