//! Low-level buffer packing with automatic alignment and accessor creation

use crate::utils::{align_buffer, compute_bounds};
use gltf_json as json;
use gltf_json::validation::Checked::Valid;

/// Accessor index returned by buffer operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AccessorIndex(pub u32);

impl AccessorIndex {
    pub fn as_json_index(&self) -> json::Index<json::Accessor> {
        json::Index::new(self.0)
    }
}

/// Builder for binary buffer with automatic alignment
pub struct BufferBuilder {
    buffer: Vec<u8>,
    views: Vec<json::buffer::View>,
    accessors: Vec<json::Accessor>,
}

impl BufferBuilder {
    /// Create a new empty buffer builder
    pub fn new() -> Self {
        Self {
            buffer: Vec::new(),
            views: Vec::new(),
            accessors: Vec::new(),
        }
    }

    /// Get the current accessor count
    pub fn accessor_count(&self) -> u32 {
        self.accessors.len() as u32
    }

    /// Get next accessor index (without creating it)
    pub fn next_accessor_index(&self) -> AccessorIndex {
        AccessorIndex(self.accessor_count())
    }

    /// Get the binary buffer data
    pub fn data(&self) -> &[u8] {
        &self.buffer
    }

    /// Get the buffer views
    pub fn views(&self) -> &[json::buffer::View] {
        &self.views
    }

    /// Get the accessors
    pub fn accessors(&self) -> &[json::Accessor] {
        &self.accessors
    }

    /// Pack Vec3 positions with bounds calculation
    pub fn pack_positions(&mut self, positions: &[[f32; 3]]) -> AccessorIndex {
        let offset = self.buffer.len();
        for pos in positions {
            self.buffer.extend_from_slice(bytemuck::cast_slice(pos));
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (positions.len() * 12).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        });

        let (min, max) = compute_bounds(positions);
        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: positions.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: Some(json::Value::Array(
                min.into_iter().map(json::Value::from).collect(),
            )),
            max: Some(json::Value::Array(
                max.into_iter().map(json::Value::from).collect(),
            )),
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }

    /// Pack Vec3 data (normals, translations, scales, etc.)
    pub fn pack_vec3(&mut self, data: &[[f32; 3]]) -> AccessorIndex {
        let offset = self.buffer.len();
        for item in data {
            self.buffer.extend_from_slice(bytemuck::cast_slice(item));
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (data.len() * 12).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: data.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec3),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }

    /// Pack Vec2 data (UVs, etc.)
    pub fn pack_vec2(&mut self, data: &[[f32; 2]]) -> AccessorIndex {
        let offset = self.buffer.len();
        for item in data {
            self.buffer.extend_from_slice(bytemuck::cast_slice(item));
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (data.len() * 8).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: data.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec2),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }

    /// Pack Vec4 data (colors, rotations, weights, etc.)
    pub fn pack_vec4(&mut self, data: &[[f32; 4]]) -> AccessorIndex {
        let offset = self.buffer.len();
        for item in data {
            self.buffer.extend_from_slice(bytemuck::cast_slice(item));
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (data.len() * 16).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: data.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec4),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }

    /// Pack joint indices (Vec4<u8>)
    pub fn pack_joints(&mut self, joints: &[[u8; 4]]) -> AccessorIndex {
        let offset = self.buffer.len();
        for joint in joints {
            self.buffer.extend_from_slice(joint);
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (joints.len() * 4).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ArrayBuffer)),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: joints.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U8,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Vec4),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }

    /// Pack u16 indices
    pub fn pack_indices_u16(&mut self, indices: &[u16]) -> AccessorIndex {
        let offset = self.buffer.len();
        for idx in indices {
            self.buffer.extend_from_slice(&idx.to_le_bytes());
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (indices.len() * 2).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: indices.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::U16,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Scalar),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }

    /// Pack Mat4 data (inverse bind matrices, etc.)
    pub fn pack_mat4(&mut self, matrices: &[[f32; 16]]) -> AccessorIndex {
        let offset = self.buffer.len();
        for mat in matrices {
            for f in mat {
                self.buffer.extend_from_slice(&f.to_le_bytes());
            }
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (matrices.len() * 64).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: matrices.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Mat4),
            min: None,
            max: None,
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }

    /// Pack scalar f32 data with min/max (animation times, etc.)
    pub fn pack_scalars_with_bounds(&mut self, scalars: &[f32]) -> AccessorIndex {
        let offset = self.buffer.len();
        for scalar in scalars {
            self.buffer.extend_from_slice(&scalar.to_le_bytes());
        }

        self.views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (scalars.len() * 4).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });

        let min_val = scalars.iter().copied().fold(f32::INFINITY, f32::min) as f64;
        let max_val = scalars.iter().copied().fold(f32::NEG_INFINITY, f32::max) as f64;

        let accessor_idx = self.accessors.len() as u32;
        self.accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(self.views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: scalars.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Scalar),
            min: Some(json::Value::Array(vec![json::Value::from(min_val)])),
            max: Some(json::Value::Array(vec![json::Value::from(max_val)])),
            name: None,
            normalized: false,
            sparse: None,
        });

        align_buffer(&mut self.buffer);
        AccessorIndex(accessor_idx)
    }
}

impl Default for BufferBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_builder_positions() {
        let mut builder = BufferBuilder::new();
        let positions = [[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]];
        let idx = builder.pack_positions(&positions);

        assert_eq!(idx, AccessorIndex(0));
        assert_eq!(builder.accessor_count(), 1);
        assert_eq!(builder.views().len(), 1);
        // 3 positions * 12 bytes = 36 bytes, aligned to 4 = 36
        assert_eq!(builder.data().len(), 36);
    }

    #[test]
    fn test_buffer_builder_indices() {
        let mut builder = BufferBuilder::new();
        let indices: [u16; 3] = [0, 1, 2];
        let idx = builder.pack_indices_u16(&indices);

        assert_eq!(idx, AccessorIndex(0));
        // 3 indices * 2 bytes = 6 bytes, aligned to 8
        assert_eq!(builder.data().len(), 8);
    }
}
