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

// Re-export BoneMatrix3x4 (defined below)

/// Maximum number of bones for GPU skinning
pub const MAX_BONES: usize = 256;

use glam::Vec4;

/// 3x4 affine bone matrix (row-major storage)
///
/// Stores 3 rows of a 4x4 affine matrix. The implicit 4th row is [0, 0, 0, 1].
/// Each row is a Vec4: [Xx, Xy, Xz, Tx] etc.
///
/// Memory layout (48 bytes):
/// - row0: rotation row 0 + translation X
/// - row1: rotation row 1 + translation Y
/// - row2: rotation row 2 + translation Z
#[derive(Clone, Copy, Debug, Default)]
#[repr(C)]
pub struct BoneMatrix3x4 {
    pub row0: Vec4,
    pub row1: Vec4,
    pub row2: Vec4,
}

impl BoneMatrix3x4 {
    /// Identity bone matrix (no transformation)
    pub const IDENTITY: Self = Self {
        row0: Vec4::new(1.0, 0.0, 0.0, 0.0),
        row1: Vec4::new(0.0, 1.0, 0.0, 0.0),
        row2: Vec4::new(0.0, 0.0, 1.0, 0.0),
    };

    /// Convert from Mat4 (column-major) to BoneMatrix3x4 (row-major)
    pub fn from_mat4(m: glam::Mat4) -> Self {
        // Mat4 is column-major: m.col(0) = [m00, m10, m20, m30]
        // We want row-major: row0 = [m00, m01, m02, m03]
        let cols = m.to_cols_array_2d();
        Self {
            row0: Vec4::new(cols[0][0], cols[1][0], cols[2][0], cols[3][0]),
            row1: Vec4::new(cols[0][1], cols[1][1], cols[2][1], cols[3][1]),
            row2: Vec4::new(cols[0][2], cols[1][2], cols[2][2], cols[3][2]),
        }
    }

    /// Convert to flat f32 array for GPU upload (row-major)
    pub fn to_array(&self) -> [f32; 12] {
        [
            self.row0.x,
            self.row0.y,
            self.row0.z,
            self.row0.w,
            self.row1.x,
            self.row1.y,
            self.row1.z,
            self.row1.w,
            self.row2.x,
            self.row2.y,
            self.row2.z,
            self.row2.w,
        ]
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
