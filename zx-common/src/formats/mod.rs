//! Emberware Z binary asset and ROM formats
//!
//! These are POD (Plain Old Data) formats for GPU-ready assets and ROM packaging.
//! No magic bytes - the format is determined by context (which FFI function is called).

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

/// File extension for EmberZ mesh files
pub const EWZ_MESH_EXT: &str = "ewzmesh";

/// File extension for EmberZ texture files
pub const EWZ_TEXTURE_EXT: &str = "ewztex";

/// File extension for EmberZ sound files
pub const EWZ_SOUND_EXT: &str = "ewzsnd";

/// File extension for EmberZ skeleton files
pub const EWZ_SKELETON_EXT: &str = "ewzskel";

/// File extension for EmberZ animation files
pub const EWZ_ANIMATION_EXT: &str = "ewzanim";
