//! GLTF document construction

use crate::{AnimationAccessors, MeshAccessors, SkeletonAccessors};
use gltf_json as json;
use gltf_json::validation::Checked::Valid;
use std::collections::BTreeMap;

/// Builder for complete GLTF documents
pub struct GltfBuilder {
    nodes: Vec<json::Node>,
    meshes: Vec<json::Mesh>,
    skins: Vec<json::Skin>,
    animations: Vec<json::Animation>,
    scenes: Vec<json::Scene>,
    buffer_byte_length: u64,
}

impl GltfBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            meshes: Vec::new(),
            skins: Vec::new(),
            animations: Vec::new(),
            scenes: Vec::new(),
            buffer_byte_length: 0,
        }
    }

    /// Set buffer byte length (required before building)
    pub fn buffer_byte_length(mut self, length: u64) -> Self {
        self.buffer_byte_length = length;
        self
    }

    /// Add a node
    pub fn add_node(mut self, node: json::Node) -> Self {
        self.nodes.push(node);
        self
    }

    /// Add multiple nodes
    pub fn add_nodes(mut self, nodes: Vec<json::Node>) -> Self {
        self.nodes.extend(nodes);
        self
    }

    /// Get the current node count
    pub fn node_count(&self) -> u32 {
        self.nodes.len() as u32
    }

    /// Add a mesh with primitive
    pub fn add_mesh_from_accessors(mut self, name: &str, accessors: &MeshAccessors) -> Self {
        let mut attributes = BTreeMap::new();
        attributes.insert(
            Valid(json::mesh::Semantic::Positions),
            accessors.positions.as_json_index(),
        );

        if let Some(normals) = accessors.normals {
            attributes.insert(
                Valid(json::mesh::Semantic::Normals),
                normals.as_json_index(),
            );
        }

        if let Some(uvs) = accessors.uvs {
            attributes.insert(
                Valid(json::mesh::Semantic::TexCoords(0)),
                uvs.as_json_index(),
            );
        }

        if let Some(colors) = accessors.colors {
            attributes.insert(
                Valid(json::mesh::Semantic::Colors(0)),
                colors.as_json_index(),
            );
        }

        if let Some(joints) = accessors.joints {
            attributes.insert(
                Valid(json::mesh::Semantic::Joints(0)),
                joints.as_json_index(),
            );
        }

        if let Some(weights) = accessors.weights {
            attributes.insert(
                Valid(json::mesh::Semantic::Weights(0)),
                weights.as_json_index(),
            );
        }

        let primitive = json::mesh::Primitive {
            attributes,
            extensions: Default::default(),
            extras: Default::default(),
            indices: accessors.indices.map(|i| i.as_json_index()),
            material: None,
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        };

        self.meshes.push(json::Mesh {
            extensions: Default::default(),
            extras: Default::default(),
            name: Some(name.to_string()),
            primitives: vec![primitive],
            weights: None,
        });

        self
    }

    /// Get the index of the last added mesh
    pub fn last_mesh_index(&self) -> Option<json::Index<json::Mesh>> {
        if self.meshes.is_empty() {
            None
        } else {
            Some(json::Index::new(self.meshes.len() as u32 - 1))
        }
    }

    /// Add a skin
    pub fn add_skin(
        mut self,
        name: &str,
        skeleton_root: u32,
        joints: &[u32],
        accessors: &SkeletonAccessors,
    ) -> Self {
        self.skins.push(json::Skin {
            extensions: Default::default(),
            extras: Default::default(),
            inverse_bind_matrices: Some(accessors.inverse_bind_matrices.as_json_index()),
            joints: joints.iter().map(|j| json::Index::new(*j)).collect(),
            name: Some(name.to_string()),
            skeleton: Some(json::Index::new(skeleton_root)),
        });
        self
    }

    /// Get the index of the last added skin
    pub fn last_skin_index(&self) -> Option<json::Index<json::Skin>> {
        if self.skins.is_empty() {
            None
        } else {
            Some(json::Index::new(self.skins.len() as u32 - 1))
        }
    }

    /// Add an animation
    pub fn add_animation(
        mut self,
        name: &str,
        bone_node_indices: &[u32],
        accessors: &AnimationAccessors,
    ) -> Self {
        let mut samplers = Vec::new();
        let mut channels = Vec::new();

        for (bone_idx, node_idx) in bone_node_indices.iter().enumerate() {
            // Translation sampler
            samplers.push(json::animation::Sampler {
                input: accessors.times.as_json_index(),
                interpolation: Valid(json::animation::Interpolation::Linear),
                output: accessors.translations[bone_idx].as_json_index(),
                extensions: Default::default(),
                extras: Default::default(),
            });
            channels.push(json::animation::Channel {
                sampler: json::Index::new(samplers.len() as u32 - 1),
                target: json::animation::Target {
                    node: json::Index::new(*node_idx),
                    path: Valid(json::animation::Property::Translation),
                    extensions: Default::default(),
                    extras: Default::default(),
                },
                extensions: Default::default(),
                extras: Default::default(),
            });

            // Rotation sampler
            samplers.push(json::animation::Sampler {
                input: accessors.times.as_json_index(),
                interpolation: Valid(json::animation::Interpolation::Linear),
                output: accessors.rotations[bone_idx].as_json_index(),
                extensions: Default::default(),
                extras: Default::default(),
            });
            channels.push(json::animation::Channel {
                sampler: json::Index::new(samplers.len() as u32 - 1),
                target: json::animation::Target {
                    node: json::Index::new(*node_idx),
                    path: Valid(json::animation::Property::Rotation),
                    extensions: Default::default(),
                    extras: Default::default(),
                },
                extensions: Default::default(),
                extras: Default::default(),
            });

            // Scale sampler
            samplers.push(json::animation::Sampler {
                input: accessors.times.as_json_index(),
                interpolation: Valid(json::animation::Interpolation::Linear),
                output: accessors.scales[bone_idx].as_json_index(),
                extensions: Default::default(),
                extras: Default::default(),
            });
            channels.push(json::animation::Channel {
                sampler: json::Index::new(samplers.len() as u32 - 1),
                target: json::animation::Target {
                    node: json::Index::new(*node_idx),
                    path: Valid(json::animation::Property::Scale),
                    extensions: Default::default(),
                    extras: Default::default(),
                },
                extensions: Default::default(),
                extras: Default::default(),
            });
        }

        self.animations.push(json::Animation {
            channels,
            extensions: Default::default(),
            extras: Default::default(),
            name: Some(name.to_string()),
            samplers,
        });
        self
    }

    /// Add a scene
    pub fn add_scene(mut self, name: &str, root_nodes: &[u32]) -> Self {
        self.scenes.push(json::Scene {
            extensions: Default::default(),
            extras: Default::default(),
            name: Some(name.to_string()),
            nodes: root_nodes.iter().map(|n| json::Index::new(*n)).collect(),
        });
        self
    }

    /// Build final GLTF Root (requires buffer views and accessors from BufferBuilder)
    pub fn build(
        self,
        buffer_views: &[json::buffer::View],
        accessors: &[json::Accessor],
        generator: &str,
    ) -> json::Root {
        let buffers = vec![json::Buffer {
            byte_length: self.buffer_byte_length.into(),
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            uri: None,
        }];

        json::Root {
            accessors: accessors.to_vec(),
            animations: self.animations,
            asset: json::Asset {
                copyright: None,
                extensions: Default::default(),
                extras: Default::default(),
                generator: Some(generator.to_string()),
                min_version: None,
                version: "2.0".to_string(),
            },
            buffers,
            buffer_views: buffer_views.to_vec(),
            cameras: Vec::new(),
            extensions: Default::default(),
            extensions_required: Vec::new(),
            extensions_used: Vec::new(),
            extras: Default::default(),
            images: Vec::new(),
            materials: Vec::new(),
            meshes: self.meshes,
            nodes: self.nodes,
            samplers: Vec::new(),
            scene: if self.scenes.is_empty() {
                None
            } else {
                Some(json::Index::new(0))
            },
            scenes: self.scenes,
            skins: self.skins,
            textures: Vec::new(),
        }
    }
}

impl Default for GltfBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{BufferBuilder, MeshBuilder};

    #[test]
    fn test_gltf_builder_basic() {
        let mut buffer = BufferBuilder::new();
        let mesh = MeshBuilder::new()
            .positions(&[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.5, 1.0, 0.0]])
            .indices(&[0, 1, 2])
            .build(&mut buffer);

        let gltf = GltfBuilder::new()
            .buffer_byte_length(buffer.data().len() as u64)
            .add_mesh_from_accessors("Triangle", &mesh)
            .add_scene("Scene", &[0]);

        let root = gltf.build(buffer.views(), buffer.accessors(), "test");

        assert_eq!(root.meshes.len(), 1);
        assert_eq!(root.scenes.len(), 1);
        assert_eq!(root.asset.version, "2.0");
    }
}
