//! Nethercore ZX binary asset and ROM formats
//!
//! These are POD (Plain Old Data) formats for GPU-ready assets and ROM packaging.
//! No magic bytes - the format is determined by context (which FFI function is called).
//!
//! ROM format constants (extensions, magic bytes) are defined in `nethercore_shared::RomFormat`.
//! Use `ZX_ROM_FORMAT` for all ZX-specific format constants.
//!
//! All format headers implement the [`BinarySerializable`] trait for consistent
//! serialization/deserialization.

pub mod animation;
pub mod mesh;
mod serialization;
pub mod skeleton;
pub mod sound;
pub mod texture;
pub mod zx_data_pack;
pub mod zx_rom;

pub use animation::*;
pub use mesh::*;
pub use serialization::BinarySerializable;
pub use skeleton::*;
pub use sound::*;
pub use texture::*;
pub use zx_data_pack::*;
pub use zx_rom::*;

// Re-export ROM format from shared for convenience
pub use nethercore_shared::{RomFormat, ZX_ROM_FORMAT};
