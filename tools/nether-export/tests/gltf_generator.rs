//! Programmatic GLB generation for integration tests.
//!
//! Generates a complete GLB file with:
//! - Skinned mesh (positions, normals, UVs, joints, weights, indices)
//! - 3-bone skeleton with inverse bind matrices
//! - 30-frame wave animation

use gltf_json as json;
use json::validation::Checked::Valid;
use std::f32::consts::TAU;

/// Bone count for the test skeleton
pub const BONE_COUNT: usize = 3;
/// Frame count for the test animation
pub const FRAME_COUNT: usize = 30;
/// Segment height between bones
const SEGMENT_HEIGHT: f32 = 1.0;

/// Generate a complete skinned GLB for testing.
///
/// Contains:
/// - 3 stacked box segments (one per bone)
/// - 3-bone skeleton (Root → Spine → Head)
/// - 30-frame wave animation
pub fn generate_skinned_glb() -> Vec<u8> {
    // Build mesh data
    let mesh = create_mesh_data();
    let skeleton = create_skeleton();
    let animation = create_animation();

    // Pack all binary data
    let (buffer_data, buffer_views, accessors) = pack_binary_data(&mesh, &skeleton, &animation);

    // Build GLTF JSON
    let root = build_gltf_json(&mesh, &buffer_views, &accessors);

    // Assemble GLB
    assemble_glb(&root, &buffer_data)
}

/// Mesh data for the test asset
pub struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub joints: Vec<[u8; 4]>,
    pub weights: Vec<[f32; 4]>,
    pub indices: Vec<u16>,
}

/// Skeleton data
struct SkeletonData {
    inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
}

/// Animation data
struct AnimationData {
    times: Vec<f32>,
    translations: Vec<Vec<[f32; 3]>>,
    rotations: Vec<Vec<[f32; 4]>>,
    scales: Vec<Vec<[f32; 3]>>,
}

/// Create mesh data: 3 stacked boxes with skinning
fn create_mesh_data() -> MeshData {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let mut indices = Vec::new();

    let half_w = 0.15;

    for seg in 0..BONE_COUNT {
        let y_base = seg as f32 * SEGMENT_HEIGHT;
        let bone = seg as u8;
        let base_vert = (seg * 24) as u16;

        // 6 faces, 4 vertices each = 24 vertices per segment
        let faces: Vec<([f32; 3], [[f32; 3]; 4], [[f32; 2]; 4])> = vec![
            // Front (+Z)
            (
                [0.0, 0.0, 1.0],
                [
                    [-half_w, y_base, half_w],
                    [half_w, y_base, half_w],
                    [half_w, y_base + SEGMENT_HEIGHT, half_w],
                    [-half_w, y_base + SEGMENT_HEIGHT, half_w],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Back (-Z)
            (
                [0.0, 0.0, -1.0],
                [
                    [half_w, y_base, -half_w],
                    [-half_w, y_base, -half_w],
                    [-half_w, y_base + SEGMENT_HEIGHT, -half_w],
                    [half_w, y_base + SEGMENT_HEIGHT, -half_w],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Right (+X)
            (
                [1.0, 0.0, 0.0],
                [
                    [half_w, y_base, half_w],
                    [half_w, y_base, -half_w],
                    [half_w, y_base + SEGMENT_HEIGHT, -half_w],
                    [half_w, y_base + SEGMENT_HEIGHT, half_w],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Left (-X)
            (
                [-1.0, 0.0, 0.0],
                [
                    [-half_w, y_base, -half_w],
                    [-half_w, y_base, half_w],
                    [-half_w, y_base + SEGMENT_HEIGHT, half_w],
                    [-half_w, y_base + SEGMENT_HEIGHT, -half_w],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Top (+Y)
            (
                [0.0, 1.0, 0.0],
                [
                    [-half_w, y_base + SEGMENT_HEIGHT, half_w],
                    [half_w, y_base + SEGMENT_HEIGHT, half_w],
                    [half_w, y_base + SEGMENT_HEIGHT, -half_w],
                    [-half_w, y_base + SEGMENT_HEIGHT, -half_w],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
            // Bottom (-Y)
            (
                [0.0, -1.0, 0.0],
                [
                    [-half_w, y_base, -half_w],
                    [half_w, y_base, -half_w],
                    [half_w, y_base, half_w],
                    [-half_w, y_base, half_w],
                ],
                [[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]],
            ),
        ];

        for (face_idx, (normal, corners, face_uvs)) in faces.iter().enumerate() {
            let face_base = base_vert + (face_idx * 4) as u16;

            for (i, corner) in corners.iter().enumerate() {
                positions.push(*corner);
                normals.push(*normal);
                uvs.push(face_uvs[i]);
                joints.push([bone, 0, 0, 0]);
                weights.push([1.0, 0.0, 0.0, 0.0]);
            }

            // Two triangles per face
            indices.push(face_base);
            indices.push(face_base + 1);
            indices.push(face_base + 2);
            indices.push(face_base);
            indices.push(face_base + 2);
            indices.push(face_base + 3);
        }
    }

    MeshData {
        positions,
        normals,
        uvs,
        joints,
        weights,
        indices,
    }
}

/// Create skeleton with 3 bones
fn create_skeleton() -> SkeletonData {
    // Inverse bind matrices (4x4 column-major)
    // For a simple vertical bone chain, the inverse bind matrix
    // is just an inverse translation
    let inverse_bind_matrices = vec![
        // Bone 0: at origin
        mat4_identity(),
        // Bone 1: at Y = 1.0
        mat4_translate(0.0, -SEGMENT_HEIGHT, 0.0),
        // Bone 2: at Y = 2.0
        mat4_translate(0.0, -2.0 * SEGMENT_HEIGHT, 0.0),
    ];

    SkeletonData {
        inverse_bind_matrices,
    }
}

/// Create animation: 30-frame wave
fn create_animation() -> AnimationData {
    let duration = 1.0f32;
    let mut times = Vec::with_capacity(FRAME_COUNT);
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    for frame in 0..FRAME_COUNT {
        let t = frame as f32 / (FRAME_COUNT - 1) as f32;
        times.push(t * duration);

        let phase = t * TAU;

        for bone in 0..BONE_COUNT {
            let bone_phase = bone as f32 * 0.5;
            let amplitude = 0.3 + (bone as f32 * 0.1);
            let angle = (phase + bone_phase).sin() * amplitude;

            // Rotation around Z axis
            let half = angle * 0.5;
            let quat = [0.0, 0.0, half.sin(), half.cos()]; // [x, y, z, w]

            // Translation: bone position in bind pose
            let translation = [0.0, bone as f32 * SEGMENT_HEIGHT, 0.0];

            translations[bone].push(translation);
            rotations[bone].push(quat);
            scales[bone].push([1.0, 1.0, 1.0]);
        }
    }

    AnimationData {
        times,
        translations,
        rotations,
        scales,
    }
}

/// Pack all binary data into a single buffer
fn pack_binary_data(
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

/// Build the GLTF JSON structure
fn build_gltf_json(
    _mesh: &MeshData,
    buffer_views: &[json::buffer::View],
    accessors: &[json::Accessor],
) -> json::Root {
    // Node indices
    const ROOT_NODE: u32 = 0;
    const SPINE_NODE: u32 = 1;
    const HEAD_NODE: u32 = 2;
    const MESH_NODE: u32 = 3;

    // Accessor indices (must match pack_binary_data order)
    const POS_ACCESSOR: u32 = 0;
    const NORM_ACCESSOR: u32 = 1;
    const UV_ACCESSOR: u32 = 2;
    const JOINTS_ACCESSOR: u32 = 3;
    const WEIGHTS_ACCESSOR: u32 = 4;
    const INDICES_ACCESSOR: u32 = 5;
    const IBM_ACCESSOR: u32 = 6;
    const TIMES_ACCESSOR: u32 = 7;

    // Create bone nodes
    let nodes = vec![
        // Node 0: Root bone
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
        // Node 1: Spine bone
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
            translation: Some([0.0, SEGMENT_HEIGHT, 0.0]),
            skin: None,
            weights: None,
        },
        // Node 2: Head bone
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
        // Node 3: Mesh node with skin
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

    // Create mesh primitive
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
        name: Some("TestMesh".to_string()),
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

    // Create skin
    let skins = vec![json::Skin {
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(json::Index::new(IBM_ACCESSOR)),
        joints: vec![
            json::Index::new(ROOT_NODE),
            json::Index::new(SPINE_NODE),
            json::Index::new(HEAD_NODE),
        ],
        name: Some("TestSkeleton".to_string()),
        skeleton: Some(json::Index::new(ROOT_NODE)),
    }];

    // Create animation
    let mut samplers = Vec::new();
    let mut channels = Vec::new();

    for bone in 0..BONE_COUNT {
        let bone_node = bone as u32;
        let trans_accessor = 8 + bone as u32;
        let rot_accessor = 8 + BONE_COUNT as u32 + bone as u32;
        let scale_accessor = 8 + 2 * BONE_COUNT as u32 + bone as u32;

        // Translation sampler and channel
        let trans_sampler = samplers.len() as u32;
        samplers.push(json::animation::Sampler {
            input: json::Index::new(TIMES_ACCESSOR),
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(trans_accessor),
            extensions: Default::default(),
            extras: Default::default(),
        });
        channels.push(json::animation::Channel {
            sampler: json::Index::new(trans_sampler),
            target: json::animation::Target {
                node: json::Index::new(bone_node),
                path: Valid(json::animation::Property::Translation),
                extensions: Default::default(),
                extras: Default::default(),
            },
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Rotation sampler and channel
        let rot_sampler = samplers.len() as u32;
        samplers.push(json::animation::Sampler {
            input: json::Index::new(TIMES_ACCESSOR),
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(rot_accessor),
            extensions: Default::default(),
            extras: Default::default(),
        });
        channels.push(json::animation::Channel {
            sampler: json::Index::new(rot_sampler),
            target: json::animation::Target {
                node: json::Index::new(bone_node),
                path: Valid(json::animation::Property::Rotation),
                extensions: Default::default(),
                extras: Default::default(),
            },
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Scale sampler and channel
        let scale_sampler = samplers.len() as u32;
        samplers.push(json::animation::Sampler {
            input: json::Index::new(TIMES_ACCESSOR),
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(scale_accessor),
            extensions: Default::default(),
            extras: Default::default(),
        });
        channels.push(json::animation::Channel {
            sampler: json::Index::new(scale_sampler),
            target: json::animation::Target {
                node: json::Index::new(bone_node),
                path: Valid(json::animation::Property::Scale),
                extensions: Default::default(),
                extras: Default::default(),
            },
            extensions: Default::default(),
            extras: Default::default(),
        });
    }

    let animations = vec![json::Animation {
        channels,
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("Wave".to_string()),
        samplers,
    }];

    // Create scene
    let scenes = vec![json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("TestScene".to_string()),
        nodes: vec![json::Index::new(ROOT_NODE), json::Index::new(MESH_NODE)],
    }];

    // Create buffer (byte length will be set by assemble_glb)
    let buffers = vec![json::Buffer {
        byte_length: 0u64.into(), // Will be updated
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
            generator: Some("nether-export-test".to_string()),
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

/// Assemble the final GLB binary
fn assemble_glb(root: &json::Root, buffer_data: &[u8]) -> Vec<u8> {
    // Update buffer byte length in root
    let mut root = root.clone();
    root.buffers[0].byte_length = buffer_data.len().into();

    // Serialize JSON
    let json_string = json::serialize::to_string(&root).expect("Failed to serialize JSON");
    let json_bytes = json_string.as_bytes();

    // Pad JSON to 4-byte alignment
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_length = json_bytes.len() + json_padding;

    // Pad buffer to 4-byte alignment
    let buffer_padding = (4 - (buffer_data.len() % 4)) % 4;
    let buffer_chunk_length = buffer_data.len() + buffer_padding;

    // Calculate total length
    let total_length = 12 + 8 + json_chunk_length + 8 + buffer_chunk_length;

    // Build GLB
    let mut glb = Vec::with_capacity(total_length);

    // Header
    glb.extend_from_slice(b"glTF"); // magic
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes()); // length

    // JSON chunk
    glb.extend_from_slice(&(json_chunk_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // chunk type "JSON"
    glb.extend_from_slice(json_bytes);
    glb.extend(std::iter::repeat_n(0x20u8, json_padding)); // pad with spaces

    // BIN chunk
    glb.extend_from_slice(&(buffer_chunk_length as u32).to_le_bytes()); // chunk length
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // chunk type "BIN\0"
    glb.extend_from_slice(buffer_data);
    glb.extend(std::iter::repeat_n(0u8, buffer_padding)); // pad with zeros

    glb
}

// Helper functions

fn mat4_identity() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

fn mat4_translate(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [x, y, z, 1.0],
    ]
}

fn compute_bounds(positions: &[[f32; 3]]) -> (Vec<f32>, Vec<f32>) {
    let mut min = [f32::MAX; 3];
    let mut max = [f32::MIN; 3];

    for pos in positions {
        for i in 0..3 {
            min[i] = min[i].min(pos[i]);
            max[i] = max[i].max(pos[i]);
        }
    }

    (min.to_vec(), max.to_vec())
}

// ============================================================================
// Partial TRS Test Generator
// ============================================================================

/// Generate a GLB where bone 1 has only rotation channels (no T, no S).
/// This tests that missing channels use the node's rest pose instead of identity.
///
/// Structure:
/// - Bone 0: At [0, 0, 0], has full T/R/S animation
/// - Bone 1: At [0, 1, 0], has ONLY R animation (no T, no S channels)
/// - Bone 2: At [0, 2, 0], has full T/R/S animation
///
/// Expected behavior: Bone 1's output should have translation [0, 1, 0] for all frames,
/// NOT [0, 0, 0] (which would happen if identity defaults were used).
pub fn generate_partial_trs_glb() -> Vec<u8> {
    let mesh = create_mesh_data();
    let skeleton = create_skeleton();
    let animation = create_partial_trs_animation();

    let (buffer_data, buffer_views, accessors) =
        pack_partial_trs_binary_data(&mesh, &skeleton, &animation);

    let root = build_partial_trs_gltf_json(&buffer_views, &accessors);

    assemble_glb(&root, &buffer_data)
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
    let bone0_t = pack_vec3(&mut buffer, &mut views, &mut accessors, &animation.bone0_translations);
    align_buffer(&mut buffer);
    let bone0_r = pack_vec4(&mut buffer, &mut views, &mut accessors, &animation.bone0_rotations);
    align_buffer(&mut buffer);
    let bone0_s = pack_vec3(&mut buffer, &mut views, &mut accessors, &animation.bone0_scales);
    align_buffer(&mut buffer);

    // Bone 1: R only
    let bone1_r = pack_vec4(&mut buffer, &mut views, &mut accessors, &animation.bone1_rotations);
    align_buffer(&mut buffer);

    // Bone 2: T, R, S
    let bone2_t = pack_vec3(&mut buffer, &mut views, &mut accessors, &animation.bone2_translations);
    align_buffer(&mut buffer);
    let bone2_r = pack_vec4(&mut buffer, &mut views, &mut accessors, &animation.bone2_rotations);
    align_buffer(&mut buffer);
    let bone2_s = pack_vec3(&mut buffer, &mut views, &mut accessors, &animation.bone2_scales);
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
    let _ = (times_accessor, bone0_t, bone0_r, bone0_s, bone1_r, bone2_t, bone2_r, bone2_s);

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
