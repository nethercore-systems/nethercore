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
pub use resources::{Font, PendingMesh, PendingMeshPacked, PendingSkeleton, PendingTexture};

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

/// Maximum number of skeletons that can be loaded
pub const MAX_SKELETONS: usize = 64;

// Re-export BoneMatrix3x4 from shared (the canonical POD type)
pub use emberware_shared::math::BoneMatrix3x4;

/// Skeleton data containing inverse bind matrices
///
/// Stored on the CPU side for upload to GPU when bound.
/// The inverse bind matrices transform vertices from model space
/// to bone-local space at bind time.
#[derive(Clone, Debug)]
pub struct SkeletonData {
    /// Inverse bind matrices (one per bone, 3x4 row-major format)
    pub inverse_bind: Vec<BoneMatrix3x4>,
    /// Number of bones in this skeleton
    pub bone_count: u32,
}

/// A batch of quad instances that share the same texture bindings
#[derive(Debug, Clone)]
pub struct QuadBatch {
    /// Texture handles for this batch (snapshot of bound_textures when batch was created)
    pub textures: [u32; 4],
    /// Quad instances in this batch
    pub instances: Vec<crate::graphics::QuadInstance>,
}
