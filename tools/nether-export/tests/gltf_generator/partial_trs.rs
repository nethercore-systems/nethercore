//! Partial TRS animation test generator.
//!
//! Generates a GLB where bone 1 has only rotation channels (no T, no S).
//! This tests that missing channels use the node's rest pose instead of identity.

use super::glb_assembly;
use super::mesh_data::{
    compute_bounds, create_mesh_data, create_skeleton, MeshData, SkeletonData, SEGMENT_HEIGHT,
};
use gltf_json as json;
use json::validation::Checked::Valid;

/// Generate a GLB where bone 1 has only rotation channels (no T, no S).
pub(crate) fn generate_partial_trs_glb() -> Vec<u8> {
    let mesh = create_mesh_data();
    let skeleton = create_skeleton();
    let animation = create_partial_trs_animation();

    let (buffer_data, buffer_views, accessors) =
        pack_partial_trs_binary_data(&mesh, &skeleton, &animation);

    let root = build_partial_trs_gltf_json(&buffer_views, &accessors);

    glb_assembly::assemble_glb(&root, &buffer_data)
}

/// Animation data for partial TRS test
struct PartialTrsAnimationData {
    times: Vec<f32>,
    // Bone 0: has T, R, S
    bone0_translations: Vec<[f32; 3]>,
    bone0_rotations: Vec<[f32; 4]>,
    bone0_scales: Vec<[f32; 3]>,
    // Bone 1: has R only
    bone1_rotations: Vec<[f32; 4]>,
    // Bone 2: has T, R, S
    bone2_translations: Vec<[f32; 3]>,
    bone2_rotations: Vec<[f32; 4]>,
    bone2_scales: Vec<[f32; 3]>,
}

fn create_partial_trs_animation() -> PartialTrsAnimationData {
    use std::f32::consts::TAU;

    let frame_count = 10;
    let duration = 1.0f32;

    let mut times = Vec::with_capacity(frame_count);
    let mut bone0_translations = Vec::with_capacity(frame_count);
    let mut bone0_rotations = Vec::with_capacity(frame_count);
    let mut bone0_scales = Vec::with_capacity(frame_count);
    let mut bone1_rotations = Vec::with_capacity(frame_count);
    let mut bone2_translations = Vec::with_capacity(frame_count);
    let mut bone2_rotations = Vec::with_capacity(frame_count);
    let mut bone2_scales = Vec::with_capacity(frame_count);

    for frame in 0..frame_count {
        let t = frame as f32 / (frame_count - 1) as f32;
        times.push(t * duration);

        let phase = t * TAU;
        let angle = phase.sin() * 0.3;
        let half = angle * 0.5;
        let quat = [0.0, 0.0, half.sin(), half.cos()];

        // Bone 0: full TRS at origin
        bone0_translations.push([0.0, 0.0, 0.0]);
        bone0_rotations.push(quat);
        bone0_scales.push([1.0, 1.0, 1.0]);

        // Bone 1: R only (no T, no S channels)
        bone1_rotations.push(quat);

        // Bone 2: full TRS at Y=2
        bone2_translations.push([0.0, 2.0 * SEGMENT_HEIGHT, 0.0]);
        bone2_rotations.push(quat);
        bone2_scales.push([1.0, 1.0, 1.0]);
    }

    PartialTrsAnimationData {
        times,
        bone0_translations,
        bone0_rotations,
        bone0_scales,
        bone1_rotations,
        bone2_translations,
        bone2_rotations,
        bone2_scales,
    }
}

fn pack_partial_trs_binary_data(
    mesh: &MeshData,
    skeleton: &SkeletonData,
    animation: &PartialTrsAnimationData,
) -> (Vec<u8>, Vec<json::buffer::View>, Vec<json::Accessor>) {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();

    fn align_buffer(buffer: &mut Vec<u8>) {
        while !buffer.len().is_multiple_of(4) {
            buffer.push(0);
        }
    }

    // --- Mesh data (same as before) ---

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
    align_buffer(&mut buffer);

    // Joints
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
    align_buffer(&mut buffer);

    // Weights
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
    align_buffer(&mut buffer);

    // --- Skeleton: IBM ---
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
    align_buffer(&mut buffer);

    // --- Animation data (partial TRS) ---

    // Helper to pack Vec3 data
    let pack_vec3 = |buffer: &mut Vec<u8>,
                     views: &mut Vec<json::buffer::View>,
                     accessors: &mut Vec<json::Accessor>,
                     data: &[[f32; 3]]| {
        let offset = buffer.len();
        for v in data {
            buffer.extend_from_slice(bytemuck::cast_slice(v));
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
        accessors.len() as u32 - 1
    };

    // Helper to pack Vec4 data
    let pack_vec4 = |buffer: &mut Vec<u8>,
                     views: &mut Vec<json::buffer::View>,
                     accessors: &mut Vec<json::Accessor>,
                     data: &[[f32; 4]]| {
        let offset = buffer.len();
        for v in data {
            buffer.extend_from_slice(bytemuck::cast_slice(v));
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
        accessors.len() as u32 - 1
    };

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
    let times_accessor = accessors.len() as u32 - 1;
    align_buffer(&mut buffer);

    // Bone 0: T, R, S
    let bone0_t = pack_vec3(
        &mut buffer,
        &mut views,
        &mut accessors,
        &animation.bone0_translations,
    );
    align_buffer(&mut buffer);
    let bone0_r = pack_vec4(
        &mut buffer,
        &mut views,
        &mut accessors,
        &animation.bone0_rotations,
    );
    align_buffer(&mut buffer);
    let bone0_s = pack_vec3(
        &mut buffer,
        &mut views,
        &mut accessors,
        &animation.bone0_scales,
    );
    align_buffer(&mut buffer);

    // Bone 1: R only
    let bone1_r = pack_vec4(
        &mut buffer,
        &mut views,
        &mut accessors,
        &animation.bone1_rotations,
    );
    align_buffer(&mut buffer);

    // Bone 2: T, R, S
    let bone2_t = pack_vec3(
        &mut buffer,
        &mut views,
        &mut accessors,
        &animation.bone2_translations,
    );
    align_buffer(&mut buffer);
    let bone2_r = pack_vec4(
        &mut buffer,
        &mut views,
        &mut accessors,
        &animation.bone2_rotations,
    );
    align_buffer(&mut buffer);
    let bone2_s = pack_vec3(
        &mut buffer,
        &mut views,
        &mut accessors,
        &animation.bone2_scales,
    );
    align_buffer(&mut buffer);

    // Store accessor indices for building JSON
    // We'll encode them in a way the JSON builder can use
    // For simplicity, store as last elements or pass through closure

    // The accessor indices are:
    // 0: positions, 1: normals, 2: uvs, 3: joints, 4: weights, 5: indices
    // 6: IBM, 7: times
    // 8: bone0_t, 9: bone0_r, 10: bone0_s
    // 11: bone1_r
    // 12: bone2_t, 13: bone2_r, 14: bone2_s
    let _ = (
        times_accessor,
        bone0_t,
        bone0_r,
        bone0_s,
        bone1_r,
        bone2_t,
        bone2_r,
        bone2_s,
    );

    (buffer, views, accessors)
}

fn build_partial_trs_gltf_json(
    buffer_views: &[json::buffer::View],
    accessors: &[json::Accessor],
) -> json::Root {
    const ROOT_NODE: u32 = 0;
    const SPINE_NODE: u32 = 1;
    const HEAD_NODE: u32 = 2;
    const MESH_NODE: u32 = 3;

    const POS_ACCESSOR: u32 = 0;
    const NORM_ACCESSOR: u32 = 1;
    const UV_ACCESSOR: u32 = 2;
    const JOINTS_ACCESSOR: u32 = 3;
    const WEIGHTS_ACCESSOR: u32 = 4;
    const INDICES_ACCESSOR: u32 = 5;
    const IBM_ACCESSOR: u32 = 6;
    const TIMES_ACCESSOR: u32 = 7;

    // Animation accessors
    const BONE0_T: u32 = 8;
    const BONE0_R: u32 = 9;
    const BONE0_S: u32 = 10;
    const BONE1_R: u32 = 11;
    const BONE2_T: u32 = 12;
    const BONE2_R: u32 = 13;
    const BONE2_S: u32 = 14;

    let nodes = vec![
        // Node 0: Root at origin
        json::Node {
            camera: None,
            children: Some(vec![json::Index::new(SPINE_NODE)]),
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: None,
            name: Some("Root".to_string()),
            rotation: None,
            scale: None,
            translation: Some([0.0, 0.0, 0.0]),
            skin: None,
            weights: None,
        },
        // Node 1: Spine at Y=1 (THIS IS THE KEY - rest pose translation)
        json::Node {
            camera: None,
            children: Some(vec![json::Index::new(HEAD_NODE)]),
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: None,
            name: Some("Spine".to_string()),
            rotation: None,
            scale: None,
            translation: Some([0.0, SEGMENT_HEIGHT, 0.0]), // Y = 1.0
            skin: None,
            weights: None,
        },
        // Node 2: Head at Y=1 relative to Spine (Y=2 absolute)
        json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: None,
            name: Some("Head".to_string()),
            rotation: None,
            scale: None,
            translation: Some([0.0, SEGMENT_HEIGHT, 0.0]),
            skin: None,
            weights: None,
        },
        // Node 3: Mesh with skin
        json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: Some(json::Index::new(0)),
            name: Some("SkinnedMesh".to_string()),
            rotation: None,
            scale: None,
            translation: None,
            skin: Some(json::Index::new(0)),
            weights: None,
        },
    ];

    // Mesh
    let mut attributes = std::collections::BTreeMap::new();
    attributes.insert(
        Valid(json::mesh::Semantic::Positions),
        json::Index::new(POS_ACCESSOR),
    );
    attributes.insert(
        Valid(json::mesh::Semantic::Normals),
        json::Index::new(NORM_ACCESSOR),
    );
    attributes.insert(
        Valid(json::mesh::Semantic::TexCoords(0)),
        json::Index::new(UV_ACCESSOR),
    );
    attributes.insert(
        Valid(json::mesh::Semantic::Joints(0)),
        json::Index::new(JOINTS_ACCESSOR),
    );
    attributes.insert(
        Valid(json::mesh::Semantic::Weights(0)),
        json::Index::new(WEIGHTS_ACCESSOR),
    );

    let meshes = vec![json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("PartialTrsMesh".to_string()),
        primitives: vec![json::mesh::Primitive {
            attributes,
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(json::Index::new(INDICES_ACCESSOR)),
            material: None,
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        }],
        weights: None,
    }];

    // Skin
    let skins = vec![json::Skin {
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(json::Index::new(IBM_ACCESSOR)),
        joints: vec![
            json::Index::new(ROOT_NODE),
            json::Index::new(SPINE_NODE),
            json::Index::new(HEAD_NODE),
        ],
        name: Some("PartialTrsSkeleton".to_string()),
        skeleton: Some(json::Index::new(ROOT_NODE)),
    }];

    // Animation with PARTIAL TRS
    // Bone 0: has T, R, S
    // Bone 1: has R ONLY (no T, no S channels!)
    // Bone 2: has T, R, S
    let mut samplers = Vec::new();
    let mut channels = Vec::new();

    // Bone 0: T, R, S
    samplers.push(json::animation::Sampler {
        input: json::Index::new(TIMES_ACCESSOR),
        interpolation: Valid(json::animation::Interpolation::Linear),
        output: json::Index::new(BONE0_T),
        extensions: Default::default(),
        extras: Default::default(),
    });
    channels.push(json::animation::Channel {
        sampler: json::Index::new(samplers.len() as u32 - 1),
        target: json::animation::Target {
            node: json::Index::new(ROOT_NODE),
            path: Valid(json::animation::Property::Translation),
            extensions: Default::default(),
            extras: Default::default(),
        },
        extensions: Default::default(),
        extras: Default::default(),
    });

    samplers.push(json::animation::Sampler {
        input: json::Index::new(TIMES_ACCESSOR),
        interpolation: Valid(json::animation::Interpolation::Linear),
        output: json::Index::new(BONE0_R),
        extensions: Default::default(),
        extras: Default::default(),
    });
    channels.push(json::animation::Channel {
        sampler: json::Index::new(samplers.len() as u32 - 1),
        target: json::animation::Target {
            node: json::Index::new(ROOT_NODE),
            path: Valid(json::animation::Property::Rotation),
            extensions: Default::default(),
            extras: Default::default(),
        },
        extensions: Default::default(),
        extras: Default::default(),
    });

    samplers.push(json::animation::Sampler {
        input: json::Index::new(TIMES_ACCESSOR),
        interpolation: Valid(json::animation::Interpolation::Linear),
        output: json::Index::new(BONE0_S),
        extensions: Default::default(),
        extras: Default::default(),
    });
    channels.push(json::animation::Channel {
        sampler: json::Index::new(samplers.len() as u32 - 1),
        target: json::animation::Target {
            node: json::Index::new(ROOT_NODE),
            path: Valid(json::animation::Property::Scale),
            extensions: Default::default(),
            extras: Default::default(),
        },
        extensions: Default::default(),
        extras: Default::default(),
    });

    // Bone 1 (Spine): R ONLY - NO T, NO S CHANNELS!
    samplers.push(json::animation::Sampler {
        input: json::Index::new(TIMES_ACCESSOR),
        interpolation: Valid(json::animation::Interpolation::Linear),
        output: json::Index::new(BONE1_R),
        extensions: Default::default(),
        extras: Default::default(),
    });
    channels.push(json::animation::Channel {
        sampler: json::Index::new(samplers.len() as u32 - 1),
        target: json::animation::Target {
            node: json::Index::new(SPINE_NODE),
            path: Valid(json::animation::Property::Rotation),
            extensions: Default::default(),
            extras: Default::default(),
        },
        extensions: Default::default(),
        extras: Default::default(),
    });

    // Bone 2: T, R, S
    samplers.push(json::animation::Sampler {
        input: json::Index::new(TIMES_ACCESSOR),
        interpolation: Valid(json::animation::Interpolation::Linear),
        output: json::Index::new(BONE2_T),
        extensions: Default::default(),
        extras: Default::default(),
    });
    channels.push(json::animation::Channel {
        sampler: json::Index::new(samplers.len() as u32 - 1),
        target: json::animation::Target {
            node: json::Index::new(HEAD_NODE),
            path: Valid(json::animation::Property::Translation),
            extensions: Default::default(),
            extras: Default::default(),
        },
        extensions: Default::default(),
        extras: Default::default(),
    });

    samplers.push(json::animation::Sampler {
        input: json::Index::new(TIMES_ACCESSOR),
        interpolation: Valid(json::animation::Interpolation::Linear),
        output: json::Index::new(BONE2_R),
        extensions: Default::default(),
        extras: Default::default(),
    });
    channels.push(json::animation::Channel {
        sampler: json::Index::new(samplers.len() as u32 - 1),
        target: json::animation::Target {
            node: json::Index::new(HEAD_NODE),
            path: Valid(json::animation::Property::Rotation),
            extensions: Default::default(),
            extras: Default::default(),
        },
        extensions: Default::default(),
        extras: Default::default(),
    });

    samplers.push(json::animation::Sampler {
        input: json::Index::new(TIMES_ACCESSOR),
        interpolation: Valid(json::animation::Interpolation::Linear),
        output: json::Index::new(BONE2_S),
        extensions: Default::default(),
        extras: Default::default(),
    });
    channels.push(json::animation::Channel {
        sampler: json::Index::new(samplers.len() as u32 - 1),
        target: json::animation::Target {
            node: json::Index::new(HEAD_NODE),
            path: Valid(json::animation::Property::Scale),
            extensions: Default::default(),
            extras: Default::default(),
        },
        extensions: Default::default(),
        extras: Default::default(),
    });

    let animations = vec![json::Animation {
        channels,
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("PartialTrsAnim".to_string()),
        samplers,
    }];

    let scenes = vec![json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("PartialTrsScene".to_string()),
        nodes: vec![json::Index::new(ROOT_NODE), json::Index::new(MESH_NODE)],
    }];

    let buffers = vec![json::Buffer {
        byte_length: 0u64.into(),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: None,
    }];

    json::Root {
        accessors: accessors.to_vec(),
        animations,
        asset: json::Asset {
            copyright: None,
            extensions: Default::default(),
            extras: Default::default(),
            generator: Some("nether-export-partial-trs-test".to_string()),
            min_version: None,
            version: "2.0".to_string(),
        },
        buffers,
        buffer_views: buffer_views.to_vec(),
        cameras: Vec::new(),
        extensions: Default::default(),
        extras: Default::default(),
        extensions_required: Vec::new(),
        extensions_used: Vec::new(),
        images: Vec::new(),
        materials: Vec::new(),
        meshes,
        nodes,
        samplers: Vec::new(),
        scene: Some(json::Index::new(0)),
        scenes,
        skins,
        textures: Vec::new(),
    }
}
