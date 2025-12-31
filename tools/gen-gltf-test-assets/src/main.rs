//! Generate test assets for gltf-test example
//!
//! Creates:
//! - Skinned mesh with UV coordinates (from programmatically generated GLB)
//! - Skeleton with inverse bind matrices
//! - Wave animation
//! - Checkerboard texture for UV visualization
//!
//! Usage:
//!   cargo run -p gen-gltf-test-assets

use anyhow::{Context, Result};
use glb_builder::{
    assemble_glb, AnimationBuilder, BufferBuilder, GltfBuilder, MeshBuilder, SkeletonBuilder,
};
use gltf_json as json;
use image::{ImageBuffer, Rgba};
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;
use zx_common::formats::animation::NetherZXAnimationHeader;
use zx_common::formats::mesh::NetherZXMeshHeader;
use zx_common::formats::skeleton::NetherZXSkeletonHeader;

/// Bone count for the test skeleton
const BONE_COUNT: usize = 3;
/// Frame count for the test animation
const FRAME_COUNT: usize = 30;
/// Segment height between bones
const SEGMENT_HEIGHT: f32 = 1.0;
/// Animation frame rate (FPS)
const FRAME_RATE: f32 = 30.0;

fn main() -> Result<()> {
    // Output to shared examples/assets folder with gltf-test- prefix
    let output_dir = PathBuf::from("examples/assets");

    // Ensure output directory exists
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    println!("Generating GLTF test assets to shared examples/assets...\n");

    // Step 1: Generate checkerboard texture
    generate_checkerboard_texture(&output_dir.join("gltf-test-checker.png"))?;

    // Step 2: Generate GLB file
    let glb_path = output_dir.join("gltf-test.glb");
    let glb_data = generate_skinned_glb();
    fs::write(&glb_path, &glb_data).context("Failed to write GLB file")?;
    println!(
        "Generated GLB: {} ({} bytes)",
        glb_path.display(),
        glb_data.len()
    );

    // Step 3: Convert GLB to native formats using nether-export
    convert_glb_to_native(&glb_path, &output_dir, "gltf-test")?;

    println!("\nAll assets generated successfully!");
    println!("\nNext steps:");
    println!("  1. cd examples/6-assets/gltf-test");
    println!("  2. nether build");
    println!("  3. nether pack");
    println!("  4. nether run");

    Ok(())
}

/// Generate a checkerboard texture for UV visualization
fn generate_checkerboard_texture(path: &PathBuf) -> Result<()> {
    const SIZE: u32 = 64;
    const CHECKER_SIZE: u32 = 8;

    let mut img = ImageBuffer::new(SIZE, SIZE);

    for y in 0..SIZE {
        for x in 0..SIZE {
            let checker_x = (x / CHECKER_SIZE) % 2;
            let checker_y = (y / CHECKER_SIZE) % 2;
            let is_white = (checker_x + checker_y).is_multiple_of(2);

            let color = if is_white {
                Rgba([255u8, 255, 255, 255])
            } else {
                Rgba([80u8, 80, 80, 255])
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(path)
        .context("Failed to save checkerboard texture")?;
    println!("Generated texture: {} ({}x{})", path.display(), SIZE, SIZE);

    Ok(())
}

/// Convert GLB to native .nczxmesh, .nczxskel, .nczxanim formats
fn convert_glb_to_native(glb_path: &PathBuf, output_dir: &PathBuf, prefix: &str) -> Result<()> {
    // Convert mesh
    let mesh_path = output_dir.join(format!("{}.nczxmesh", prefix));
    let mesh = nether_export::convert_gltf_to_memory(glb_path).context("Failed to convert mesh")?;

    // Build mesh file with header
    let header = NetherZXMeshHeader::new(mesh.vertex_count, mesh.index_count, mesh.format);
    let mut mesh_data = header.to_bytes().to_vec();
    mesh_data.extend_from_slice(&mesh.vertex_data);
    for idx in &mesh.indices {
        mesh_data.extend_from_slice(&idx.to_le_bytes());
    }
    fs::write(&mesh_path, &mesh_data).context("Failed to write mesh file")?;
    println!(
        "Generated mesh: {} ({} vertices, {} indices, format 0x{:02X})",
        mesh_path.display(),
        mesh.vertex_count,
        mesh.index_count,
        mesh.format
    );

    // Convert skeleton
    let skel_path = output_dir.join(format!("{}.nczxskel", prefix));
    let skeleton = nether_export::convert_gltf_skeleton_to_memory(glb_path, None)
        .context("Failed to convert skeleton")?;

    // Build skeleton file with header
    let skel_header = NetherZXSkeletonHeader::new(skeleton.bone_count);
    let mut skel_data = skel_header.to_bytes().to_vec();
    for ibm in &skeleton.inverse_bind_matrices {
        for f in ibm {
            skel_data.extend_from_slice(&f.to_le_bytes());
        }
    }
    fs::write(&skel_path, &skel_data).context("Failed to write skeleton file")?;
    println!(
        "Generated skeleton: {} ({} bones)",
        skel_path.display(),
        skeleton.bone_count
    );

    // Convert animation
    let anim_path = output_dir.join(format!("{}.nczxanim", prefix));
    let animation = nether_export::convert_gltf_animation_to_memory(
        glb_path,
        None, // First animation
        None, // First skin
        Some(FRAME_RATE),
    )
    .context("Failed to convert animation")?;

    // Build animation file with header
    let anim_header = NetherZXAnimationHeader::new(animation.bone_count, animation.frame_count);
    let mut anim_data = anim_header.to_bytes().to_vec();
    anim_data.extend_from_slice(&animation.data);
    fs::write(&anim_path, &anim_data).context("Failed to write animation file")?;
    println!(
        "Generated animation: {} ({} bones, {} frames)",
        anim_path.display(),
        animation.bone_count,
        animation.frame_count
    );

    Ok(())
}

// ============================================================================
// GLB Generation
// ============================================================================

/// Generate a complete GLB with skinned mesh, skeleton, and animation
fn generate_skinned_glb() -> Vec<u8> {
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

    // Build animation
    let animation = create_wave_animation(&mut buffer);

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
            name: Some("SkinnedMesh".to_string()),
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
        .add_mesh_from_accessors("TestMesh", &mesh)
        .add_skin("TestSkin", ROOT_NODE, &joint_indices, &skeleton)
        .add_animation("Wave", &bone_node_indices, &animation)
        .add_scene("Scene", &[ROOT_NODE, MESH_NODE]);

    let root = gltf.build(buffer.views(), buffer.accessors(), "gen-gltf-test-assets");

    assemble_glb(&root, buffer.data())
}

/// Create mesh data: 3 stacked cubes with proper UV mapping and vertex colors
fn create_mesh_data() -> (
    Vec<[f32; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 2]>,
    Vec<[f32; 4]>,
    Vec<[u8; 4]>,
    Vec<[f32; 4]>,
    Vec<u16>,
) {
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

    for seg in 0..BONE_COUNT {
        let y_base = seg as f32 * SEGMENT_HEIGHT;
        let bone = seg as u8;
        let base_vert = (seg * 24) as u16;

        // Face definitions: (normal, corners with UVs)
        let faces: Vec<([f32; 3], [([f32; 3], [f32; 2]); 4])> = vec![
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
                colors.push(segment_colors[seg]);
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
