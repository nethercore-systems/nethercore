//! Math types for Emberware
//!
//! Provides POD (Plain Old Data) math types that are serializable and
//! can be shared across crates without requiring glam as a dependency.

use bitcode::{Decode, Encode};
use serde::{Deserialize, Serialize};

/// 3x4 affine bone matrix (row-major storage, POD type)
///
/// Stores 3 rows of a 4x4 affine matrix. The implicit 4th row is [0, 0, 0, 1].
/// Each row stores [Xx, Xy, Xz, Tx] etc.
///
/// Memory layout (48 bytes):
/// - row0: rotation row 0 + translation X
/// - row1: rotation row 1 + translation Y
/// - row2: rotation row 2 + translation Z
///
/// This is the POD version for serialization. Console implementations
/// can add conversion methods to their native math types (e.g., glam::Vec4).
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize, Encode, Decode)]
#[repr(C)]
pub struct BoneMatrix3x4 {
    /// First row: [m00, m01, m02, tx]
    pub row0: [f32; 4],
    /// Second row: [m10, m11, m12, ty]
    pub row1: [f32; 4],
    /// Third row: [m20, m21, m22, tz]
    pub row2: [f32; 4],
}

impl BoneMatrix3x4 {
    /// Identity bone matrix (no transformation)
    pub const IDENTITY: Self = Self {
        row0: [1.0, 0.0, 0.0, 0.0],
        row1: [0.0, 1.0, 0.0, 0.0],
        row2: [0.0, 0.0, 1.0, 0.0],
    };

    /// Create from row arrays
    pub const fn from_rows(row0: [f32; 4], row1: [f32; 4], row2: [f32; 4]) -> Self {
        Self { row0, row1, row2 }
    }

    /// Convert to flat f32 array for GPU upload (row-major)
    pub fn to_array(&self) -> [f32; 12] {
        [
            self.row0[0],
            self.row0[1],
            self.row0[2],
            self.row0[3],
            self.row1[0],
            self.row1[1],
            self.row1[2],
            self.row1[3],
            self.row2[0],
            self.row2[1],
            self.row2[2],
            self.row2[3],
        ]
    }

    /// Create from flat f32 array (row-major)
    pub fn from_array(arr: [f32; 12]) -> Self {
        Self {
            row0: [arr[0], arr[1], arr[2], arr[3]],
            row1: [arr[4], arr[5], arr[6], arr[7]],
            row2: [arr[8], arr[9], arr[10], arr[11]],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identity() {
        let m = BoneMatrix3x4::IDENTITY;
        assert_eq!(m.row0, [1.0, 0.0, 0.0, 0.0]);
        assert_eq!(m.row1, [0.0, 1.0, 0.0, 0.0]);
        assert_eq!(m.row2, [0.0, 0.0, 1.0, 0.0]);
    }

    #[test]
    fn test_to_array() {
        let m = BoneMatrix3x4::IDENTITY;
        let arr = m.to_array();
        assert_eq!(
            arr,
            [1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0]
        );
    }

    #[test]
    fn test_from_array() {
        let arr = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let m = BoneMatrix3x4::from_array(arr);
        assert_eq!(m.row0, [1.0, 2.0, 3.0, 4.0]);
        assert_eq!(m.row1, [5.0, 6.0, 7.0, 8.0]);
        assert_eq!(m.row2, [9.0, 10.0, 11.0, 12.0]);
    }

    #[test]
    fn test_roundtrip() {
        let arr = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0];
        let m = BoneMatrix3x4::from_array(arr);
        let arr2 = m.to_array();
        assert_eq!(arr, arr2);
    }
}
