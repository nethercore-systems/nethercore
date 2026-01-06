//! Pack command - create .nczx ROM from WASM + assets
//!
//! Automatically compresses textures based on render mode:
//! - Mode 0 (Lambert): RGBA8 (uncompressed)
//! - Mode 1-3 (Matcap/PBR/Hybrid): BC7 (4:1 compression)

mod assets;
mod command;
mod manifest;
mod output;
mod validation;

pub use command::{execute, PackArgs};
