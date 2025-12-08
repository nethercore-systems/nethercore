//! Emberware Z FFI state and types
//!
//! FFI staging state for Emberware Z console.
//! This state is rebuilt each frame from FFI calls and consumed by ZGraphics.
//! It is NOT part of rollback state - only GameState is rolled back.

mod config;
mod ffi_state;
mod resources;

pub use config::ZInitConfig;
pub use ffi_state::ZFFIState;
pub use resources::{Font, PendingMesh, PendingMeshPacked, PendingTexture};

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

/// A batch of quad instances that share the same texture bindings
#[derive(Debug, Clone)]
pub struct QuadBatch {
    /// Texture handles for this batch (snapshot of bound_textures when batch was created)
    pub textures: [u32; 4],
    /// Quad instances in this batch
    pub instances: Vec<crate::graphics::QuadInstance>,
}
