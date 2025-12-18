//! Emberware ZX binary asset and ROM formats
//!
//! These are POD (Plain Old Data) formats for GPU-ready assets and ROM packaging.
//! No magic bytes - the format is determined by context (which FFI function is called).
//!
//! ROM format constants (extensions, magic bytes) are defined in `emberware_shared::RomFormat`.
//! Use `ZX_ROM_FORMAT` for all ZX-specific format constants.

pub mod animation;
pub mod mesh;
pub mod skeleton;
pub mod sound;
pub mod texture;
pub mod z_data_pack;
pub mod z_rom;

pub use animation::*;
pub use mesh::*;
pub use skeleton::*;
pub use sound::*;
pub use texture::*;
pub use z_data_pack::*;
pub use z_rom::*;

// Re-export ROM format from shared for convenience
pub use emberware_shared::{RomFormat, ZX_ROM_FORMAT};
