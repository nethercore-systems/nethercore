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

/// Extension trait for BoneMatrix3x4 with glam conversion methods
///
/// This adds glam-specific methods to the POD BoneMatrix3x4 type
/// without requiring glam as a dependency in the shared crate.
pub trait BoneMatrix3x4Ext {
    /// Convert from Mat4 (column-major) to BoneMatrix3x4 (row-major)
    fn from_mat4(m: glam::Mat4) -> BoneMatrix3x4;

    /// Get row 0 as a glam Vec4
    fn row0_vec4(&self) -> glam::Vec4;

    /// Get row 1 as a glam Vec4
    fn row1_vec4(&self) -> glam::Vec4;

    /// Get row 2 as a glam Vec4
    fn row2_vec4(&self) -> glam::Vec4;
}

impl BoneMatrix3x4Ext for BoneMatrix3x4 {
    fn from_mat4(m: glam::Mat4) -> BoneMatrix3x4 {
        // Mat4 is column-major: m.col(0) = [m00, m10, m20, m30]
        // We want row-major: row0 = [m00, m01, m02, m03]
        let cols = m.to_cols_array_2d();
        BoneMatrix3x4 {
            row0: [cols[0][0], cols[1][0], cols[2][0], cols[3][0]],
            row1: [cols[0][1], cols[1][1], cols[2][1], cols[3][1]],
            row2: [cols[0][2], cols[1][2], cols[2][2], cols[3][2]],
        }
    }

    fn row0_vec4(&self) -> glam::Vec4 {
        glam::Vec4::from_array(self.row0)
    }

    fn row1_vec4(&self) -> glam::Vec4 {
        glam::Vec4::from_array(self.row1)
    }

    fn row2_vec4(&self) -> glam::Vec4 {
        glam::Vec4::from_array(self.row2)
    }
}

/// A batch of quad instances that share the same texture bindings
#[derive(Debug, Clone)]
pub struct QuadBatch {
    /// Texture handles for this batch (snapshot of bound_textures when batch was created)
    pub textures: [u32; 4],
    /// Quad instances in this batch
    pub instances: Vec<crate::graphics::QuadInstance>,
}
