//! Generate multi-mesh GLB files for glb-rigid example
//!
//! Creates SEPARATE GLB files for each mesh piece:
//! - glb-rigid-base.glb - Base platform
//! - glb-rigid-arm.glb - Extending arm segment
//! - glb-rigid-claw.glb - Claw/gripper end effector
//! - glb-rigid-anim.glb - Animation data (skeleton + keyframes, no mesh)
//!
//! The example demonstrates rigid animation by using keyframe_read() to
//! sample transforms from imported GLTF animation and applying them manually.
//!
//! Usage:
//!   cargo run -p gen-glb-rigid-assets

use anyhow::{Context, Result};
use gltf_json as json;
use json::validation::Checked::Valid;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

/// Number of nodes for the rigid hierarchy (Base, Arm, Claw)
const NODE_COUNT: usize = 3;
/// Frame count for animation
const FRAME_COUNT: usize = 60;
/// Animation frame rate
const FRAME_RATE: f32 = 30.0;

fn main() -> Result<()> {
    // Output to shared examples/assets folder
    let output_dir = PathBuf::from("examples/assets");

    // Ensure output directory exists
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    println!("Generating rigid mesh and animation GLBs for glb-rigid example...\n");

    // Generate each mesh piece as a separate GLB
    // (Current nether-export only extracts first mesh, so we need separate files)

    // Base - flat platform
    let base_path = output_dir.join("glb-rigid-base.glb");
    let base_data = generate_mesh_glb(MeshType::Base);
    fs::write(&base_path, &base_data).context("Failed to write base GLB")?;
    println!("Generated: {} ({} bytes)", base_path.display(), base_data.len());

    // Arm - elongated segment
    let arm_path = output_dir.join("glb-rigid-arm.glb");
    let arm_data = generate_mesh_glb(MeshType::Arm);
    fs::write(&arm_path, &arm_data).context("Failed to write arm GLB")?;
    println!("Generated: {} ({} bytes)", arm_path.display(), arm_data.len());

    // Claw - gripper end
    let claw_path = output_dir.join("glb-rigid-claw.glb");
    let claw_data = generate_mesh_glb(MeshType::Claw);
    fs::write(&claw_path, &claw_data).context("Failed to write claw GLB")?;
    println!("Generated: {} ({} bytes)", claw_path.display(), claw_data.len());

    // Animation GLB - contains skeleton + animation (no mesh)
    let anim_path = output_dir.join("glb-rigid-anim.glb");
    let anim_data = generate_animation_glb();
    fs::write(&anim_path, &anim_data).context("Failed to write animation GLB")?;
    println!("Generated: {} ({} bytes)", anim_path.display(), anim_data.len());

    println!("\nGenerated 4 GLBs for rigid animation demo:");
    println!("  - glb-rigid-base.glb: rotating platform mesh");
    println!("  - glb-rigid-arm.glb: extending segment mesh");
    println!("  - glb-rigid-claw.glb: opening/closing gripper mesh");
    println!("  - glb-rigid-anim.glb: skeleton + animation keyframes");
    println!("\nAnimation uses keyframe_read() (not skeleton_bind)");

    Ok(())
}

enum MeshType {
    Base,
    Arm,
    Claw,
}

/// Generate a GLB file containing a single mesh piece
fn generate_mesh_glb(mesh_type: MeshType) -> Vec<u8> {
    let (positions, normals, colors, indices, name) = match mesh_type {
        MeshType::Base => create_base_mesh(),
        MeshType::Arm => create_arm_mesh(),
        MeshType::Claw => create_claw_mesh(),
    };

    let (buffer_data, buffer_views, accessors) =
        pack_mesh_data(&positions, &normals, &colors, &indices);

    let mut root = build_mesh_gltf_json(&buffer_views, &accessors, &name);
    root.buffers[0].byte_length = (buffer_data.len() as u64).into();

    assemble_glb(&root, &buffer_data)
}

/// Create base mesh: flat hexagonal platform (origin at center)
fn create_base_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 4]>, Vec<u16>, String) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let color = [0.4, 0.4, 0.5, 1.0]; // Gray metallic
    let radius = 0.6;
    let height = 0.15;
    let sides = 6;

    // Top face
    let top_center = positions.len() as u16;
    positions.push([0.0, height, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    colors.push(color);

    for i in 0..sides {
        let angle = (i as f32 / sides as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        positions.push([x, height, z]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(color);
    }

    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.push(top_center);
        indices.push(top_center + 1 + i as u16);
        indices.push(top_center + 1 + next as u16);
    }

    // Bottom face
    let bottom_center = positions.len() as u16;
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    colors.push(color);

    for i in 0..sides {
        let angle = (i as f32 / sides as f32) * std::f32::consts::TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        positions.push([x, 0.0, z]);
        normals.push([0.0, -1.0, 0.0]);
        colors.push(color);
    }

    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.push(bottom_center);
        indices.push(bottom_center + 1 + next as u16);
        indices.push(bottom_center + 1 + i as u16);
    }

    // Side faces
    for i in 0..sides {
        let next = (i + 1) % sides;
        let angle1 = (i as f32 / sides as f32) * std::f32::consts::TAU;
        let angle2 = (next as f32 / sides as f32) * std::f32::consts::TAU;

        let x1 = angle1.cos() * radius;
        let z1 = angle1.sin() * radius;
        let x2 = angle2.cos() * radius;
        let z2 = angle2.sin() * radius;

        // Normal pointing outward (average of the two edge directions)
        let mid_angle = (angle1 + angle2) / 2.0;
        let normal = [mid_angle.cos(), 0.0, mid_angle.sin()];

        let base_idx = positions.len() as u16;

        positions.push([x1, 0.0, z1]);
        positions.push([x2, 0.0, z2]);
        positions.push([x2, height, z2]);
        positions.push([x1, height, z1]);

        for _ in 0..4 {
            normals.push(normal);
            colors.push(color);
        }

        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    (positions, normals, colors, indices, "Base".to_string())
}

/// Create arm mesh: elongated box (origin at pivot point)
fn create_arm_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 4]>, Vec<u16>, String) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let color = [0.6, 0.3, 0.2, 1.0]; // Orange-brown
    let width = 0.15;
    let height = 0.15;
    let length = 1.0;

    // Box centered at origin in X/Z, extending in +Y (arm goes "up" when unrotated)
    let faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
        // Front (+Z)
        ([0.0, 0.0, 1.0], [
            [-width, 0.0, width],
            [width, 0.0, width],
            [width, length, width],
            [-width, length, width],
        ]),
        // Back (-Z)
        ([0.0, 0.0, -1.0], [
            [width, 0.0, -width],
            [-width, 0.0, -width],
            [-width, length, -width],
            [width, length, -width],
        ]),
        // Right (+X)
        ([1.0, 0.0, 0.0], [
            [width, 0.0, width],
            [width, 0.0, -width],
            [width, length, -width],
            [width, length, width],
        ]),
        // Left (-X)
        ([-1.0, 0.0, 0.0], [
            [-width, 0.0, -width],
            [-width, 0.0, width],
            [-width, length, width],
            [-width, length, -width],
        ]),
        // Top (+Y)
        ([0.0, 1.0, 0.0], [
            [-width, length, width],
            [width, length, width],
            [width, length, -width],
            [-width, length, -width],
        ]),
        // Bottom (-Y)
        ([0.0, -1.0, 0.0], [
            [-width, 0.0, -width],
            [width, 0.0, -width],
            [width, 0.0, width],
            [-width, 0.0, width],
        ]),
    ];

    for (normal, corners) in &faces {
        let base_idx = positions.len() as u16;
        for corner in corners {
            positions.push(*corner);
            normals.push(*normal);
            colors.push(color);
        }
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    (positions, normals, colors, indices, "Arm".to_string())
}

/// Create claw mesh: two prongs that can open/close (origin at pivot)
fn create_claw_mesh() -> (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 4]>, Vec<u16>, String) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let color = [0.2, 0.5, 0.3, 1.0]; // Green
    let prong_width = 0.08;
    let prong_height = 0.08;
    let prong_length = 0.4;
    let spacing = 0.15; // Distance from center to each prong

    // Create two prongs (left and right)
    for offset in [-spacing, spacing] {
        let faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
            // Front (+Z)
            ([0.0, 0.0, 1.0], [
                [offset - prong_width, 0.0, prong_height],
                [offset + prong_width, 0.0, prong_height],
                [offset + prong_width, prong_length, prong_height],
                [offset - prong_width, prong_length, prong_height],
            ]),
            // Back (-Z)
            ([0.0, 0.0, -1.0], [
                [offset + prong_width, 0.0, -prong_height],
                [offset - prong_width, 0.0, -prong_height],
                [offset - prong_width, prong_length, -prong_height],
                [offset + prong_width, prong_length, -prong_height],
            ]),
            // Right (+X)
            ([1.0, 0.0, 0.0], [
                [offset + prong_width, 0.0, prong_height],
                [offset + prong_width, 0.0, -prong_height],
                [offset + prong_width, prong_length, -prong_height],
                [offset + prong_width, prong_length, prong_height],
            ]),
            // Left (-X)
            ([-1.0, 0.0, 0.0], [
                [offset - prong_width, 0.0, -prong_height],
                [offset - prong_width, 0.0, prong_height],
                [offset - prong_width, prong_length, prong_height],
                [offset - prong_width, prong_length, -prong_height],
            ]),
            // Top (+Y at end of prong)
            ([0.0, 1.0, 0.0], [
                [offset - prong_width, prong_length, prong_height],
                [offset + prong_width, prong_length, prong_height],
                [offset + prong_width, prong_length, -prong_height],
                [offset - prong_width, prong_length, -prong_height],
            ]),
            // Bottom (-Y at base)
            ([0.0, -1.0, 0.0], [
                [offset - prong_width, 0.0, -prong_height],
                [offset + prong_width, 0.0, -prong_height],
                [offset + prong_width, 0.0, prong_height],
                [offset - prong_width, 0.0, prong_height],
            ]),
        ];

        for (normal, corners) in &faces {
            let base_idx = positions.len() as u16;
            for corner in corners {
                positions.push(*corner);
                normals.push(*normal);
                colors.push(color);
            }
            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);
            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }
    }

    // Add a connecting bridge at the base
    let bridge_width = spacing + prong_width;
    let bridge_height = 0.06;
    let bridge_depth = prong_height;

    let bridge_faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
        // Front (+Z)
        ([0.0, 0.0, 1.0], [
            [-bridge_width, -bridge_height, bridge_depth],
            [bridge_width, -bridge_height, bridge_depth],
            [bridge_width, 0.0, bridge_depth],
            [-bridge_width, 0.0, bridge_depth],
        ]),
        // Back (-Z)
        ([0.0, 0.0, -1.0], [
            [bridge_width, -bridge_height, -bridge_depth],
            [-bridge_width, -bridge_height, -bridge_depth],
            [-bridge_width, 0.0, -bridge_depth],
            [bridge_width, 0.0, -bridge_depth],
        ]),
        // Bottom
        ([0.0, -1.0, 0.0], [
            [-bridge_width, -bridge_height, -bridge_depth],
            [bridge_width, -bridge_height, -bridge_depth],
            [bridge_width, -bridge_height, bridge_depth],
            [-bridge_width, -bridge_height, bridge_depth],
        ]),
    ];

    for (normal, corners) in &bridge_faces {
        let base_idx = positions.len() as u16;
        for corner in corners {
            positions.push(*corner);
            normals.push(*normal);
            colors.push(color);
        }
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    (positions, normals, colors, indices, "Claw".to_string())
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

fn align_buffer(buffer: &mut Vec<u8>) {
    while buffer.len() % 4 != 0 {
        buffer.push(0);
    }
}

fn pack_mesh_data(
    positions: &[[f32; 3]],
    normals: &[[f32; 3]],
    colors: &[[f32; 4]],
    indices: &[u16],
) -> (Vec<u8>, Vec<json::buffer::View>, Vec<json::Accessor>) {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();

    // Positions (accessor 0)
    let pos_offset = buffer.len();
    for pos in positions {
        buffer.extend_from_slice(bytemuck::cast_slice(pos));
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (positions.len() * 12).into(),
        byte_offset: Some((pos_offset as u64).into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    let (min, max) = compute_bounds(positions);
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: positions.len().into(),
        component_type: Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::F32)),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Vec3),
        min: Some(json::Value::Array(min.into_iter().map(json::Value::from).collect())),
        max: Some(json::Value::Array(max.into_iter().map(json::Value::from).collect())),
        name: None,
        normalized: false,
        sparse: None,
    });
    align_buffer(&mut buffer);

    // Normals (accessor 1)
    let norm_offset = buffer.len();
    for norm in normals {
        buffer.extend_from_slice(bytemuck::cast_slice(norm));
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (normals.len() * 12).into(),
        byte_offset: Some((norm_offset as u64).into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: normals.len().into(),
        component_type: Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::F32)),
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

    // Colors (accessor 2)
    let color_offset = buffer.len();
    for color in colors {
        buffer.extend_from_slice(bytemuck::cast_slice(color));
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (colors.len() * 16).into(),
        byte_offset: Some((color_offset as u64).into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: colors.len().into(),
        component_type: Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::F32)),
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

    // Indices (accessor 3)
    let idx_offset = buffer.len();
    for idx in indices {
        buffer.extend_from_slice(&idx.to_le_bytes());
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (indices.len() * 2).into(),
        byte_offset: Some((idx_offset as u64).into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: Some(Valid(json::buffer::Target::ElementArrayBuffer)),
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
        byte_offset: Some(0u64.into()),
        count: indices.len().into(),
        component_type: Valid(json::accessor::GenericComponentType(json::accessor::ComponentType::U16)),
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

    (buffer, views, accessors)
}

fn build_mesh_gltf_json(
    buffer_views: &[json::buffer::View],
    accessors: &[json::Accessor],
    mesh_name: &str,
) -> json::Root {
    let nodes = vec![json::Node {
        camera: None,
        children: None,
        extensions: Default::default(),
        extras: Default::default(),
        matrix: None,
        mesh: Some(json::Index::new(0)),
        name: Some(mesh_name.to_string()),
        rotation: None,
        scale: None,
        skin: None,
        translation: None,
        weights: None,
    }];

    let meshes = vec![json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some(mesh_name.to_string()),
        primitives: vec![json::mesh::Primitive {
            attributes: {
                let mut attrs = std::collections::BTreeMap::new();
                attrs.insert(Valid(json::mesh::Semantic::Positions), json::Index::new(0));
                attrs.insert(Valid(json::mesh::Semantic::Normals), json::Index::new(1));
                attrs.insert(Valid(json::mesh::Semantic::Colors(0)), json::Index::new(2));
                attrs
            },
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(json::Index::new(3)),
            material: None,
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        }],
        weights: None,
    }];

    let scenes = vec![json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("Scene".to_string()),
        nodes: vec![json::Index::new(0)],
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
        animations: Vec::new(),
        asset: json::Asset {
            copyright: None,
            extensions: Default::default(),
            extras: Default::default(),
            generator: Some("gen-glb-rigid-assets".to_string()),
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
        meshes,
        nodes,
        samplers: Vec::new(),
        scene: Some(json::Index::new(0)),
        scenes,
        skins: Vec::new(),
        textures: Vec::new(),
    }
}

fn assemble_glb(root: &json::Root, buffer_data: &[u8]) -> Vec<u8> {
    let json_string = json::serialize::to_string(root).expect("Failed to serialize GLTF JSON");
    let json_bytes = json_string.as_bytes();

    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_length = json_bytes.len() + json_padding;

    let buffer_padding = (4 - (buffer_data.len() % 4)) % 4;
    let buffer_chunk_length = buffer_data.len() + buffer_padding;

    let total_length = 12 + 8 + json_chunk_length + 8 + buffer_chunk_length;

    let mut glb = Vec::with_capacity(total_length);

    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes());
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    glb.extend_from_slice(&(json_chunk_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes());
    glb.extend_from_slice(json_bytes);
    for _ in 0..json_padding {
        glb.push(0x20);
    }

    glb.extend_from_slice(&(buffer_chunk_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes());
    glb.extend_from_slice(buffer_data);
    for _ in 0..buffer_padding {
        glb.push(0);
    }

    glb
}

// ============================================================================
// Animation GLB Generation (skeleton + keyframes, no mesh)
// ============================================================================

/// Animation data container
struct AnimationData {
    name: String,
    times: Vec<f32>,
    translations: Vec<Vec<[f32; 3]>>,
    rotations: Vec<Vec<[f32; 4]>>,
    scales: Vec<Vec<[f32; 3]>>,
}

/// Generate animation GLB with skeleton + keyframes for rigid animation
fn generate_animation_glb() -> Vec<u8> {
    let animation = create_operate_animation();
    let (buffer_data, buffer_views, accessors) = pack_animation_data(&animation);
    let mut root = build_animation_gltf_json(&buffer_views, &accessors, &animation);
    root.buffers[0].byte_length = (buffer_data.len() as u64).into();
    assemble_glb(&root, &buffer_data)
}

/// Create "Operate" animation for the mechanical arm
/// Node 0 (Base): Rotates around Y
/// Node 1 (Arm): Tilts (X rotation) and extends (Z translation)
/// Node 2 (Claw): Opens/closes (Y translation offset)
fn create_operate_animation() -> AnimationData {
    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); NODE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); NODE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); NODE_COUNT];

    let duration = FRAME_COUNT as f32 / FRAME_RATE;

    for frame in 0..FRAME_COUNT {
        let t = frame as f32 / FRAME_COUNT as f32;
        times.push(t * duration);

        let anim_time = t * TAU;

        // Node 0: Base - rotates around Y axis
        {
            let rotation_y = anim_time * 0.5; // Slow rotation
            let half = rotation_y * 0.5;
            translations[0].push([0.0, 0.0, 0.0]);
            rotations[0].push([0.0, half.sin(), 0.0, half.cos()]);
            scales[0].push([1.0, 1.0, 1.0]);
        }

        // Node 1: Arm - tilts and extends
        {
            // Tilt angle (X rotation)
            let tilt = (anim_time * 1.2).sin() * 0.4; // ±0.4 rad (~23 deg)
            let half_tilt = tilt * 0.5;

            // Extension (Z translation)
            let extension = ((anim_time * 1.5).sin() + 1.0) * 0.3; // 0 to 0.6

            // Position relative to base attachment point
            translations[1].push([0.0, 0.5, extension]);
            rotations[1].push([half_tilt.sin(), 0.0, 0.0, half_tilt.cos()]);
            scales[1].push([1.0, 1.0, 1.0]);
        }

        // Node 2: Claw - positioned at end of arm, opens/closes
        {
            // Claw opening (spread in X)
            let open_amount = ((anim_time * 2.0).sin() + 1.0) * 0.15; // 0 to 0.3

            // Position at end of arm segment (arm is ~1.0 long)
            translations[2].push([open_amount, 1.0, 0.0]);
            rotations[2].push([0.0, 0.0, 0.0, 1.0]); // Identity rotation
            scales[2].push([1.0, 1.0, 1.0]);
        }
    }

    AnimationData {
        name: "Operate".to_string(),
        times,
        translations,
        rotations,
        scales,
    }
}

/// Pack animation data into binary buffer
fn pack_animation_data(
    animation: &AnimationData,
) -> (Vec<u8>, Vec<json::buffer::View>, Vec<json::Accessor>) {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();

    // Accessor 0: Inverse bind matrices (3x identity 4x4 matrices)
    let ibm_offset = buffer.len();
    for _node in 0..NODE_COUNT {
        #[rustfmt::skip]
        let identity: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, 0.0, 0.0, 1.0,
        ];
        for val in identity {
            buffer.extend_from_slice(&val.to_le_bytes());
        }
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (NODE_COUNT * 64).into(),
        byte_offset: Some((ibm_offset as u64).into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: None,
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(0)),
        byte_offset: Some(0u64.into()),
        count: NODE_COUNT.into(),
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

    // Accessor 1: Animation times
    let time_offset = buffer.len();
    for t in &animation.times {
        buffer.extend_from_slice(&t.to_le_bytes());
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (animation.times.len() * 4).into(),
        byte_offset: Some((time_offset as u64).into()),
        byte_stride: None,
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        target: None,
    });
    accessors.push(json::Accessor {
        buffer_view: Some(json::Index::new(1)),
        byte_offset: Some(0u64.into()),
        count: animation.times.len().into(),
        component_type: Valid(json::accessor::GenericComponentType(
            json::accessor::ComponentType::F32,
        )),
        extensions: Default::default(),
        extras: Default::default(),
        type_: Valid(json::accessor::Type::Scalar),
        min: Some(json::Value::Array(vec![json::Value::from(0.0)])),
        max: Some(json::Value::Array(vec![json::Value::from(
            *animation.times.last().unwrap(),
        )])),
        name: None,
        normalized: false,
        sparse: None,
    });
    align_buffer(&mut buffer);

    // Accessors 2-4: Translation data for each node
    for node in 0..NODE_COUNT {
        let offset = buffer.len();
        for trans in &animation.translations[node] {
            for v in trans {
                buffer.extend_from_slice(&v.to_le_bytes());
            }
        }
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (animation.translations[node].len() * 12).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: animation.translations[node].len().into(),
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
    }

    // Accessors 5-7: Rotation data for each node
    for node in 0..NODE_COUNT {
        let offset = buffer.len();
        for rot in &animation.rotations[node] {
            for v in rot {
                buffer.extend_from_slice(&v.to_le_bytes());
            }
        }
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (animation.rotations[node].len() * 16).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: animation.rotations[node].len().into(),
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
    }

    // Accessors 8-10: Scale data for each node
    for node in 0..NODE_COUNT {
        let offset = buffer.len();
        for scl in &animation.scales[node] {
            for v in scl {
                buffer.extend_from_slice(&v.to_le_bytes());
            }
        }
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (animation.scales[node].len() * 12).into(),
            byte_offset: Some((offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: animation.scales[node].len().into(),
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
    }

    (buffer, views, accessors)
}

/// Build GLTF JSON for animation GLB
fn build_animation_gltf_json(
    buffer_views: &[json::buffer::View],
    accessors: &[json::Accessor],
    animation: &AnimationData,
) -> json::Root {
    // Nodes: 3 nodes representing Base, Arm, Claw
    let nodes = vec![
        json::Node {
            name: Some("Base".to_string()),
            mesh: None,
            skin: None,
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
        },
        json::Node {
            name: Some("Arm".to_string()),
            mesh: None,
            skin: None,
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
        },
        json::Node {
            name: Some("Claw".to_string()),
            mesh: None,
            skin: None,
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
        },
    ];

    // Skin referencing all 3 nodes
    let skins = vec![json::Skin {
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(json::Index::new(0)), // Accessor 0
        joints: vec![
            json::Index::new(0),
            json::Index::new(1),
            json::Index::new(2),
        ],
        name: Some("RigidSkeleton".to_string()),
        skeleton: Some(json::Index::new(0)),
    }];

    // Animation channels and samplers
    let mut channels = Vec::new();
    let mut samplers = Vec::new();

    // Time accessor is at index 1
    let time_accessor = 1u32;

    for node in 0..NODE_COUNT {
        let node_idx = node as u32;

        // Translation sampler
        let trans_accessor = 2 + node as u32; // Accessors 2, 3, 4
        samplers.push(json::animation::Sampler {
            extensions: Default::default(),
            extras: Default::default(),
            input: json::Index::new(time_accessor),
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(trans_accessor),
        });
        channels.push(json::animation::Channel {
            extensions: Default::default(),
            extras: Default::default(),
            sampler: json::Index::new(samplers.len() as u32 - 1),
            target: json::animation::Target {
                extensions: Default::default(),
                extras: Default::default(),
                node: json::Index::new(node_idx),
                path: Valid(json::animation::Property::Translation),
            },
        });

        // Rotation sampler
        let rot_accessor = 5 + node as u32; // Accessors 5, 6, 7
        samplers.push(json::animation::Sampler {
            extensions: Default::default(),
            extras: Default::default(),
            input: json::Index::new(time_accessor),
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(rot_accessor),
        });
        channels.push(json::animation::Channel {
            extensions: Default::default(),
            extras: Default::default(),
            sampler: json::Index::new(samplers.len() as u32 - 1),
            target: json::animation::Target {
                extensions: Default::default(),
                extras: Default::default(),
                node: json::Index::new(node_idx),
                path: Valid(json::animation::Property::Rotation),
            },
        });

        // Scale sampler
        let scale_accessor = 8 + node as u32; // Accessors 8, 9, 10
        samplers.push(json::animation::Sampler {
            extensions: Default::default(),
            extras: Default::default(),
            input: json::Index::new(time_accessor),
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(scale_accessor),
        });
        channels.push(json::animation::Channel {
            extensions: Default::default(),
            extras: Default::default(),
            sampler: json::Index::new(samplers.len() as u32 - 1),
            target: json::animation::Target {
                extensions: Default::default(),
                extras: Default::default(),
                node: json::Index::new(node_idx),
                path: Valid(json::animation::Property::Scale),
            },
        });
    }

    let animations = vec![json::Animation {
        extensions: Default::default(),
        extras: Default::default(),
        channels,
        name: Some(animation.name.clone()),
        samplers,
    }];

    let scenes = vec![json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("Scene".to_string()),
        nodes: vec![
            json::Index::new(0),
            json::Index::new(1),
            json::Index::new(2),
        ],
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
            generator: Some("gen-glb-rigid-assets".to_string()),
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
        meshes: Vec::new(),
        nodes,
        samplers: Vec::new(),
        scene: Some(json::Index::new(0)),
        scenes,
        skins,
        textures: Vec::new(),
    }
}
