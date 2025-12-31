//! Skeleton construction utilities

use crate::buffer::{AccessorIndex, BufferBuilder};

/// Accessor indices for skeleton data
#[derive(Debug, Clone)]
pub struct SkeletonAccessors {
    pub inverse_bind_matrices: AccessorIndex,
}

/// Builder for skeleton data
pub struct SkeletonBuilder {
    inverse_bind_matrices: Vec<[f32; 16]>,
}

impl SkeletonBuilder {
    pub fn new() -> Self {
        Self {
            inverse_bind_matrices: Vec::new(),
        }
    }

    /// Add a bone with its inverse bind matrix
    pub fn add_bone(mut self, inverse_bind_matrix: [f32; 16]) -> Self {
        self.inverse_bind_matrices.push(inverse_bind_matrix);
        self
    }

    /// Set all inverse bind matrices at once
    pub fn inverse_bind_matrices(mut self, matrices: &[[f32; 16]]) -> Self {
        self.inverse_bind_matrices = matrices.to_vec();
        self
    }

    /// Get bone count
    pub fn bone_count(&self) -> usize {
        self.inverse_bind_matrices.len()
    }

    /// Build and pack into buffer
    pub fn build(self, buffer: &mut BufferBuilder) -> SkeletonAccessors {
        let inverse_bind_matrices = buffer.pack_mat4(&self.inverse_bind_matrices);
        SkeletonAccessors {
            inverse_bind_matrices,
        }
    }
}

impl Default for SkeletonBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDENTITY_MAT4: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 0.0, 1.0,
    ];

    #[test]
    fn test_skeleton_builder() {
        let mut buffer = BufferBuilder::new();
        let skeleton = SkeletonBuilder::new()
            .add_bone(IDENTITY_MAT4)
            .add_bone(IDENTITY_MAT4)
            .build(&mut buffer);

        assert_eq!(skeleton.inverse_bind_matrices, AccessorIndex(0));
        // 2 matrices * 64 bytes = 128 bytes
        assert_eq!(buffer.data().len(), 128);
    }

    #[test]
    fn test_skeleton_builder_bulk() {
        let mut buffer = BufferBuilder::new();
        let matrices = [IDENTITY_MAT4; 5];
        let builder = SkeletonBuilder::new().inverse_bind_matrices(&matrices);

        assert_eq!(builder.bone_count(), 5);
        let skeleton = builder.build(&mut buffer);
        assert_eq!(skeleton.inverse_bind_matrices, AccessorIndex(0));
    }
}
