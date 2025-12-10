//! Emberware Z binary asset formats
//!
//! These are POD (Plain Old Data) formats for GPU-ready assets.
//! No magic bytes - the format is determined by context (which FFI function is called).

pub mod ember_z_animation;
pub mod ember_z_mesh;
pub mod ember_z_skeleton;
pub mod ember_z_sound;
pub mod ember_z_texture;

pub use ember_z_animation::*;
pub use ember_z_mesh::*;
pub use ember_z_skeleton::*;
pub use ember_z_sound::*;
pub use ember_z_texture::*;

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
