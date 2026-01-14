//! Binary data packing for GLTF buffers.

use super::mesh_data::{compute_bounds, AnimationData, MeshData, SkeletonData};
use gltf_json as json;
use json::validation::Checked::Valid;

/// Pack all binary data into a single buffer
pub(crate) fn pack_binary_data(
    mesh: &MeshData,
    skeleton: &SkeletonData,
    animation: &AnimationData,
) -> (Vec<u8>, Vec<json::buffer::View>, Vec<json::Accessor>) {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();

    // Helper to align buffer to 4 bytes
    fn align_buffer(buffer: &mut Vec<u8>) {
        while !buffer.len().is_multiple_of(4) {
            buffer.push(0);
        }
    }

    // Accessor indices
    let mut accessor_idx = 0u32;

    // --- Mesh data ---

    // Positions
    let pos_offset = buffer.len();
    for pos in &mesh.positions {
        buffer.extend_from_slice(bytemuck::cast_slice(pos));
    }
    let pos_len = buffer.len() - pos_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: pos_len.into(),
        byte_offset: Some(pos_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    let (min, max) = compute_bounds(&mesh.positions);
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: mesh.positions.len().into(),
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
    let _pos_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // Normals
    let norm_offset = buffer.len();
    for norm in &mesh.normals {
        buffer.extend_from_slice(bytemuck::cast_slice(norm));
    }
    let norm_len = buffer.len() - norm_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: norm_len.into(),
        byte_offset: Some(norm_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: mesh.normals.len().into(),
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
    let _norm_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // UVs
    let uv_offset = buffer.len();
    for uv in &mesh.uvs {
        buffer.extend_from_slice(bytemuck::cast_slice(uv));
    }
    let uv_len = buffer.len() - uv_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: uv_len.into(),
        byte_offset: Some(uv_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: mesh.uvs.len().into(),
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
    let _uv_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // Joints (JOINTS_0)
    let joints_offset = buffer.len();
    for joint in &mesh.joints {
        buffer.extend_from_slice(joint);
    }
    let joints_len = buffer.len() - joints_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: joints_len.into(),
        byte_offset: Some(joints_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: mesh.joints.len().into(),
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
    let _joints_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // Weights (WEIGHTS_0)
    let weights_offset = buffer.len();
    for weight in &mesh.weights {
        buffer.extend_from_slice(bytemuck::cast_slice(weight));
    }
    let weights_len = buffer.len() - weights_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: weights_len.into(),
        byte_offset: Some(weights_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: mesh.weights.len().into(),
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
    let _weights_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // Indices
    let indices_offset = buffer.len();
    for idx in &mesh.indices {
        buffer.extend_from_slice(&idx.to_le_bytes());
    }
    let indices_len = buffer.len() - indices_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: indices_len.into(),
        byte_offset: Some(indices_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: mesh.indices.len().into(),
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
    let _indices_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // --- Skeleton data ---

    // Inverse bind matrices
    let ibm_offset = buffer.len();
    for mat in &skeleton.inverse_bind_matrices {
        for col in mat {
            buffer.extend_from_slice(bytemuck::cast_slice(col));
        }
    }
    let ibm_len = buffer.len() - ibm_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: ibm_len.into(),
        byte_offset: Some(ibm_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: None,
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: skeleton.inverse_bind_matrices.len().into(),
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
    let _ibm_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // --- Animation data ---

    // Animation times
    let times_offset = buffer.len();
    for t in &animation.times {
        buffer.extend_from_slice(&t.to_le_bytes());
    }
    let times_len = buffer.len() - times_offset;
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: times_len.into(),
        byte_offset: Some(times_offset.into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: None,
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: animation.times.len().into(),
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Scalar),
        min: Some(json::Value::Array(vec![json::Value::from(0.0f64)])),
        max: Some(json::Value::Array(vec![json::Value::from(1.0f64)])),
        name: None,
        normalized: false,
        sparse: None,
    });
    let _times_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // Animation translations (one accessor per bone)
    let mut trans_accessors = Vec::new();
    for bone_trans in &animation.translations {
        let offset = buffer.len();
        for t in bone_trans {
            buffer.extend_from_slice(bytemuck::cast_slice(t));
        }
        let len = buffer.len() - offset;
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: len.into(),
            byte_offset: Some(offset.into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: bone_trans.len().into(),
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
        trans_accessors.push(accessor_idx);
        accessor_idx += 1;
        align_buffer(&mut buffer);
    }

    // Animation rotations (one accessor per bone)
    let mut rot_accessors = Vec::new();
    for bone_rot in &animation.rotations {
        let offset = buffer.len();
        for r in bone_rot {
            buffer.extend_from_slice(bytemuck::cast_slice(r));
        }
        let len = buffer.len() - offset;
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: len.into(),
            byte_offset: Some(offset.into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: bone_rot.len().into(),
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
        rot_accessors.push(accessor_idx);
        accessor_idx += 1;
        align_buffer(&mut buffer);
    }

    // Animation scales (one accessor per bone)
    let mut scale_accessors = Vec::new();
    for bone_scale in &animation.scales {
        let offset = buffer.len();
        for s in bone_scale {
            buffer.extend_from_slice(bytemuck::cast_slice(s));
        }
        let len = buffer.len() - offset;
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: len.into(),
            byte_offset: Some(offset.into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: bone_scale.len().into(),
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
        scale_accessors.push(accessor_idx);
        accessor_idx += 1;
        align_buffer(&mut buffer);
    }

    // Store accessor indices in a way that build_gltf_json can use
    // For simplicity, we'll use the accessor vector order:
    // 0: positions, 1: normals, 2: uvs, 3: joints, 4: weights, 5: indices
    // 6: IBM, 7: times, 8-10: translations, 11-13: rotations, 14-16: scales

    (buffer, views, accessors)
}
