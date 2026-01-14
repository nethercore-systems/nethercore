//! GLTF JSON structure building.

use super::mesh_data::{BONE_COUNT, SEGMENT_HEIGHT};
use gltf_json as json;
use json::validation::Checked::Valid;

/// Build the GLTF JSON structure
pub(crate) fn build_gltf_json(
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
