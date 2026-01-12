//! Generate multi-animation GLB for glb-inline example
//!
//! Creates a GLB with:
//! - 1 mesh: "CharacterMesh" (3-segment skinned cube)
//! - 1 skin: "CharacterSkin" (3 bones: Root, Spine, Head)
//! - 3 animations: "Wave", "Bounce", "Twist"
//!
//! Usage:
//!   cargo run -p gen-glb-inline-assets

use anyhow::{Context, Result};
use glb_builder::{
    assemble_glb, AnimationBuilder, BufferBuilder, GltfBuilder, MeshBuilder, SkeletonBuilder,
};
use gltf_json as json;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

/// Face definition: (normal, corners with positions and UVs)
type FaceDefinition = ([f32; 3], [([f32; 3], [f32; 2]); 4]);

/// Skinned mesh data: (positions, normals, uvs, colors, joints, weights, indices)
type SkinnedMeshData = (Vec<[f32; 3]>, Vec<[f32; 3]>, Vec<[f32; 2]>, Vec<[f32; 4]>, Vec<[u8; 4]>, Vec<[f32; 4]>, Vec<u16>);

/// Bone count for the test skeleton
const BONE_COUNT: usize = 3;
/// Frame count for each animation
const FRAME_COUNT: usize = 30;
/// Segment height between bones
const SEGMENT_HEIGHT: f32 = 1.0;
/// Animation frame rate (FPS)
const FRAME_RATE: f32 = 30.0;

fn main() -> Result<()> {
    // Output to shared examples/assets folder
    let output_dir = PathBuf::from("examples/assets");

    // Ensure output directory exists
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    println!("Generating multi-animation GLB for glb-inline example...\n");

    // Generate GLB file with 3 animations
    let glb_path = output_dir.join("glb-inline-multi.glb");
    let glb_data = generate_multi_animation_glb();
    fs::write(&glb_path, &glb_data).context("Failed to write GLB file")?;
    println!(
        "Generated GLB: {} ({} bytes)",
        glb_path.display(),
        glb_data.len()
    );

    println!("\nGenerated assets:");
    println!("  - glb-inline-multi.glb (mesh + skeleton + 3 animations)");
    println!("\nAnimations included:");
    println!("  - Wave: side-to-side motion");
    println!("  - Bounce: vertical bounce");
    println!("  - Twist: Y-axis rotation");

    Ok(())
}

// ============================================================================
// GLB Generation
// ============================================================================

/// Generate a complete GLB with skinned mesh, skeleton, and THREE animations
fn generate_multi_animation_glb() -> Vec<u8> {
    let mut buffer = BufferBuilder::new();

    // Build mesh
    let (positions, normals, uvs, colors, joints, weights, indices) = create_mesh_data();
    let mesh = MeshBuilder::new()
        .positions(&positions)
        .normals(&normals)
        .uvs(&uvs)
        .colors(&colors)
        .joints(&joints)
        .weights(&weights)
        .indices(&indices)
        .build(&mut buffer);

    // Build skeleton
    let inverse_bind_matrices = create_inverse_bind_matrices();
    let skeleton = SkeletonBuilder::new()
        .inverse_bind_matrices(&inverse_bind_matrices)
        .build(&mut buffer);

    // Build animations
    let wave_anim = create_wave_animation(&mut buffer);
    let bounce_anim = create_bounce_animation(&mut buffer);
    let twist_anim = create_twist_animation(&mut buffer);

    // Node indices
    const ROOT_NODE: u32 = 0;
    const SPINE_NODE: u32 = 1;
    const HEAD_NODE: u32 = 2;
    const MESH_NODE: u32 = 3;

    // Create nodes
    let nodes = vec![
        // Root bone (index 0)
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
            skin: None,
            translation: Some([0.0, 0.0, 0.0]),
            weights: None,
        },
        // Spine bone (index 1)
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
            skin: None,
            translation: Some([0.0, SEGMENT_HEIGHT, 0.0]),
            weights: None,
        },
        // Head bone (index 2)
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
            skin: None,
            translation: Some([0.0, SEGMENT_HEIGHT, 0.0]),
            weights: None,
        },
        // Mesh node (index 3)
        json::Node {
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            mesh: Some(json::Index::new(0)),
            name: Some("CharacterMesh".to_string()),
            rotation: None,
            scale: None,
            skin: Some(json::Index::new(0)),
            translation: None,
            weights: None,
        },
    ];

    // Build GLTF document
    let bone_node_indices: Vec<u32> = (0..BONE_COUNT as u32).collect();
    let joint_indices: Vec<u32> = vec![ROOT_NODE, SPINE_NODE, HEAD_NODE];

    let gltf = GltfBuilder::new()
        .buffer_byte_length(buffer.data().len() as u64)
        .add_nodes(nodes)
        .add_mesh_from_accessors("CharacterMesh", &mesh)
        .add_skin("CharacterSkin", ROOT_NODE, &joint_indices, &skeleton)
        .add_animation("Wave", &bone_node_indices, &wave_anim)
        .add_animation("Bounce", &bone_node_indices, &bounce_anim)
        .add_animation("Twist", &bone_node_indices, &twist_anim)
        .add_scene("Scene", &[ROOT_NODE, MESH_NODE]);

    let root = gltf.build(buffer.views(), buffer.accessors(), "gen-glb-inline-assets");

    assemble_glb(&root, buffer.data())
}

/// Create mesh data: 3 stacked cubes with proper UV mapping and vertex colors
fn create_mesh_data() -> SkinnedMeshData {
    let half_w = 0.3;
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut colors = Vec::new();
    let mut joints = Vec::new();
    let mut weights = Vec::new();
    let mut indices = Vec::new();

    // Distinct colors for each bone segment (RGB + alpha)
    let segment_colors: [[f32; 4]; 3] = [
        [1.0, 0.2, 0.2, 1.0], // Red for root
        [0.2, 1.0, 0.2, 1.0], // Green for spine
        [0.2, 0.2, 1.0, 1.0], // Blue for head
    ];

    for (seg, &color) in segment_colors.iter().enumerate() {
        let y_base = seg as f32 * SEGMENT_HEIGHT;
        let bone = seg as u8;
        let base_vert = (seg * 24) as u16;

        // Face definitions: (normal, corners with UVs)
        let faces: Vec<FaceDefinition> = vec![
            // Front (+Z)
            (
                [0.0, 0.0, 1.0],
                [
                    ([-half_w, y_base, half_w], [0.0, 0.0]),
                    ([half_w, y_base, half_w], [1.0, 0.0]),
                    ([half_w, y_base + SEGMENT_HEIGHT, half_w], [1.0, 1.0]),
                    ([-half_w, y_base + SEGMENT_HEIGHT, half_w], [0.0, 1.0]),
                ],
            ),
            // Back (-Z)
            (
                [0.0, 0.0, -1.0],
                [
                    ([half_w, y_base, -half_w], [0.0, 0.0]),
                    ([-half_w, y_base, -half_w], [1.0, 0.0]),
                    ([-half_w, y_base + SEGMENT_HEIGHT, -half_w], [1.0, 1.0]),
                    ([half_w, y_base + SEGMENT_HEIGHT, -half_w], [0.0, 1.0]),
                ],
            ),
            // Right (+X)
            (
                [1.0, 0.0, 0.0],
                [
                    ([half_w, y_base, half_w], [0.0, 0.0]),
                    ([half_w, y_base, -half_w], [1.0, 0.0]),
                    ([half_w, y_base + SEGMENT_HEIGHT, -half_w], [1.0, 1.0]),
                    ([half_w, y_base + SEGMENT_HEIGHT, half_w], [0.0, 1.0]),
                ],
            ),
            // Left (-X)
            (
                [-1.0, 0.0, 0.0],
                [
                    ([-half_w, y_base, -half_w], [0.0, 0.0]),
                    ([-half_w, y_base, half_w], [1.0, 0.0]),
                    ([-half_w, y_base + SEGMENT_HEIGHT, half_w], [1.0, 1.0]),
                    ([-half_w, y_base + SEGMENT_HEIGHT, -half_w], [0.0, 1.0]),
                ],
            ),
            // Top (+Y)
            (
                [0.0, 1.0, 0.0],
                [
                    ([-half_w, y_base + SEGMENT_HEIGHT, half_w], [0.0, 0.0]),
                    ([half_w, y_base + SEGMENT_HEIGHT, half_w], [1.0, 0.0]),
                    ([half_w, y_base + SEGMENT_HEIGHT, -half_w], [1.0, 1.0]),
                    ([-half_w, y_base + SEGMENT_HEIGHT, -half_w], [0.0, 1.0]),
                ],
            ),
            // Bottom (-Y)
            (
                [0.0, -1.0, 0.0],
                [
                    ([-half_w, y_base, -half_w], [0.0, 0.0]),
                    ([half_w, y_base, -half_w], [1.0, 0.0]),
                    ([half_w, y_base, half_w], [1.0, 1.0]),
                    ([-half_w, y_base, half_w], [0.0, 1.0]),
                ],
            ),
        ];

        for (face_idx, (normal, corners)) in faces.iter().enumerate() {
            let face_base = base_vert + (face_idx * 4) as u16;

            for (pos, uv) in corners {
                positions.push(*pos);
                normals.push(*normal);
                uvs.push(*uv);
                colors.push(color);
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

    (positions, normals, uvs, colors, joints, weights, indices)
}

/// Create inverse bind matrices for skeleton
fn create_inverse_bind_matrices() -> Vec<[f32; 16]> {
    let mut matrices = Vec::new();

    for i in 0..BONE_COUNT {
        let y = i as f32 * SEGMENT_HEIGHT;

        // Inverse bind matrix: translate by -y
        #[rustfmt::skip]
        let ibm: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, -y, 0.0, 1.0,
        ];
        matrices.push(ibm);
    }

    matrices
}

/// Create Wave animation: side-to-side Z-rotation
fn create_wave_animation(buffer: &mut BufferBuilder) -> glb_builder::AnimationAccessors {
    let duration = FRAME_COUNT as f32 / FRAME_RATE;

    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    for frame in 0..FRAME_COUNT {
        let t = frame as f32 / FRAME_COUNT as f32;
        times.push(t * duration);

        let wave_time = t * TAU;
        let mut accumulated_angle = 0.0f32;

        for bone in 0..BONE_COUNT {
            let phase = bone as f32 * 0.5;
            let amplitude = 0.4 + (bone as f32 * 0.1);
            let local_angle = (wave_time + phase).sin() * amplitude;
            accumulated_angle += local_angle;

            // World position
            let world_pos = if bone == 0 {
                [0.0, 0.0, 0.0]
            } else {
                let parent_pos = translations[bone - 1].last().unwrap();
                let parent_angle = accumulated_angle - local_angle;
                let c = parent_angle.cos();
                let s = parent_angle.sin();
                let dx = -SEGMENT_HEIGHT * s;
                let dy = SEGMENT_HEIGHT * c;
                [parent_pos[0] + dx, parent_pos[1] + dy, parent_pos[2]]
            };

            // World rotation quaternion (Z-axis)
            let half_angle = accumulated_angle * 0.5;
            let world_quat = [0.0, 0.0, half_angle.sin(), half_angle.cos()];

            translations[bone].push(world_pos);
            rotations[bone].push(world_quat);
            scales[bone].push([1.0, 1.0, 1.0]);
        }
    }

    AnimationBuilder::new(BONE_COUNT)
        .times(&times)
        .all_translations(translations)
        .all_rotations(rotations)
        .all_scales(scales)
        .build(buffer)
}

/// Create Bounce animation: vertical Y-translation bounce
fn create_bounce_animation(buffer: &mut BufferBuilder) -> glb_builder::AnimationAccessors {
    let duration = FRAME_COUNT as f32 / FRAME_RATE;

    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    for frame in 0..FRAME_COUNT {
        let t = frame as f32 / FRAME_COUNT as f32;
        times.push(t * duration);

        let bounce_time = t * TAU;

        for bone in 0..BONE_COUNT {
            // Each bone bounces with a slight delay
            let phase = bone as f32 * 0.3;
            let bounce_offset = ((bounce_time + phase).sin().abs()) * 0.3;

            let base_y = bone as f32 * SEGMENT_HEIGHT;
            let world_pos = [0.0, base_y + bounce_offset, 0.0];

            // No rotation, just identity quaternion
            let world_quat = [0.0, 0.0, 0.0, 1.0];

            translations[bone].push(world_pos);
            rotations[bone].push(world_quat);
            scales[bone].push([1.0, 1.0, 1.0]);
        }
    }

    AnimationBuilder::new(BONE_COUNT)
        .times(&times)
        .all_translations(translations)
        .all_rotations(rotations)
        .all_scales(scales)
        .build(buffer)
}

/// Create Twist animation: Y-axis rotation twist
fn create_twist_animation(buffer: &mut BufferBuilder) -> glb_builder::AnimationAccessors {
    let duration = FRAME_COUNT as f32 / FRAME_RATE;

    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    for frame in 0..FRAME_COUNT {
        let t = frame as f32 / FRAME_COUNT as f32;
        times.push(t * duration);

        let twist_time = t * TAU;

        for bone in 0..BONE_COUNT {
            // Each bone has increasing twist amplitude
            let amplitude = (bone as f32 + 1.0) * 0.3;
            let angle = twist_time.sin() * amplitude;

            let base_y = bone as f32 * SEGMENT_HEIGHT;
            let world_pos = [0.0, base_y, 0.0];

            // Y-axis rotation quaternion
            let half_angle = angle * 0.5;
            let world_quat = [0.0, half_angle.sin(), 0.0, half_angle.cos()];

            translations[bone].push(world_pos);
            rotations[bone].push(world_quat);
            scales[bone].push([1.0, 1.0, 1.0]);
        }
    }

    AnimationBuilder::new(BONE_COUNT)
        .times(&times)
        .all_translations(translations)
        .all_rotations(rotations)
        .all_scales(scales)
        .build(buffer)
}
