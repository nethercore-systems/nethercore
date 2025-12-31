//! High-level mesh construction

use crate::buffer::{AccessorIndex, BufferBuilder};

/// Accessor indices for a mesh
#[derive(Debug, Clone)]
pub struct MeshAccessors {
    pub positions: AccessorIndex,
    pub normals: Option<AccessorIndex>,
    pub uvs: Option<AccessorIndex>,
    pub colors: Option<AccessorIndex>,
    pub joints: Option<AccessorIndex>,
    pub weights: Option<AccessorIndex>,
    pub indices: Option<AccessorIndex>,
}

/// Builder for mesh data
pub struct MeshBuilder {
    positions: Vec<[f32; 3]>,
    normals: Option<Vec<[f32; 3]>>,
    uvs: Option<Vec<[f32; 2]>>,
    colors: Option<Vec<[f32; 4]>>,
    joints: Option<Vec<[u8; 4]>>,
    weights: Option<Vec<[f32; 4]>>,
    indices: Option<Vec<u16>>,
}

impl MeshBuilder {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: None,
            uvs: None,
            colors: None,
            joints: None,
            weights: None,
            indices: None,
        }
    }

    /// Set positions (required)
    pub fn positions(mut self, positions: &[[f32; 3]]) -> Self {
        self.positions = positions.to_vec();
        self
    }

    /// Set normals (optional)
    pub fn normals(mut self, normals: &[[f32; 3]]) -> Self {
        self.normals = Some(normals.to_vec());
        self
    }

    /// Set UVs (optional)
    pub fn uvs(mut self, uvs: &[[f32; 2]]) -> Self {
        self.uvs = Some(uvs.to_vec());
        self
    }

    /// Set vertex colors (optional)
    pub fn colors(mut self, colors: &[[f32; 4]]) -> Self {
        self.colors = Some(colors.to_vec());
        self
    }

    /// Set joint indices (optional, for skinned meshes)
    pub fn joints(mut self, joints: &[[u8; 4]]) -> Self {
        self.joints = Some(joints.to_vec());
        self
    }

    /// Set joint weights (optional, for skinned meshes)
    pub fn weights(mut self, weights: &[[f32; 4]]) -> Self {
        self.weights = Some(weights.to_vec());
        self
    }

    /// Set indices (optional)
    pub fn indices(mut self, indices: &[u16]) -> Self {
        self.indices = Some(indices.to_vec());
        self
    }

    /// Build and pack into buffer
    pub fn build(self, buffer: &mut BufferBuilder) -> MeshAccessors {
        let positions = buffer.pack_positions(&self.positions);
        let normals = self.normals.as_ref().map(|n| buffer.pack_vec3(n));
        let uvs = self.uvs.as_ref().map(|uv| buffer.pack_vec2(uv));
        let colors = self.colors.as_ref().map(|c| buffer.pack_vec4(c));
        let joints = self.joints.as_ref().map(|j| buffer.pack_joints(j));
        let weights = self.weights.as_ref().map(|w| buffer.pack_vec4(w));
        let indices = self.indices.as_ref().map(|i| buffer.pack_indices_u16(i));

        MeshAccessors {
            positions,
            normals,
            uvs,
            colors,
            joints,
            weights,
            indices,
        }
    }
}

impl Default for MeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mesh_builder_basic() {
        let mut buffer = BufferBuilder::new();
        let mesh = MeshBuilder::new()
            .positions(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]])
            .normals(&[[0.0, 0.0, 1.0]; 3])
            .indices(&[0, 1, 2])
            .build(&mut buffer);

        assert_eq!(mesh.positions, AccessorIndex(0));
        assert_eq!(mesh.normals, Some(AccessorIndex(1)));
        assert_eq!(mesh.indices, Some(AccessorIndex(2)));
        assert!(mesh.uvs.is_none());
        assert!(mesh.colors.is_none());
    }

    #[test]
    fn test_mesh_builder_skinned() {
        let mut buffer = BufferBuilder::new();
        let mesh = MeshBuilder::new()
            .positions(&[[0.0, 0.0, 0.0]])
            .joints(&[[0, 0, 0, 0]])
            .weights(&[[1.0, 0.0, 0.0, 0.0]])
            .build(&mut buffer);

        assert!(mesh.joints.is_some());
        assert!(mesh.weights.is_some());
    }
}
