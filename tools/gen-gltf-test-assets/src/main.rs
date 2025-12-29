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
use gltf_json as json;
use image::{ImageBuffer, Rgba};
use json::validation::Checked::Valid;
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

    // Step 1: Generate checkerboard texture (use existing one in shared assets)
    // The shared checkerboard.png already exists, but we generate a larger one for better UV visibility
    generate_checkerboard_texture(&output_dir.join("gltf-test-checker.png"))?;

    // Step 2: Generate GLB file
    let glb_path = output_dir.join("gltf-test.glb");
    let glb_data = generate_skinned_glb();
    fs::write(&glb_path, &glb_data).context("Failed to write GLB file")?;
    println!("Generated GLB: {} ({} bytes)", glb_path.display(), glb_data.len());

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
            let is_white = (checker_x + checker_y) % 2 == 0;

            let color = if is_white {
                Rgba([255u8, 255, 255, 255])
            } else {
                Rgba([80u8, 80, 80, 255])
            };
            img.put_pixel(x, y, color);
        }
    }

    img.save(path).context("Failed to save checkerboard texture")?;
    println!("Generated texture: {} ({}x{})", path.display(), SIZE, SIZE);

    Ok(())
}

/// Convert GLB to native .nczxmesh, .nczxskel, .nczxanim formats
fn convert_glb_to_native(glb_path: &PathBuf, output_dir: &PathBuf, prefix: &str) -> Result<()> {
    // Convert mesh
    let mesh_path = output_dir.join(format!("{}.nczxmesh", prefix));
    let mesh = nether_export::convert_gltf_to_memory(glb_path)
        .context("Failed to convert mesh")?;

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
// GLB Generation (based on gltf_generator.rs from tests)
// ============================================================================

/// Mesh data container
struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,  // RGBA vertex colors
    joints: Vec<[u8; 4]>,
    weights: Vec<[f32; 4]>,
    indices: Vec<u16>,
}

/// Skeleton data container
struct SkeletonData {
    bone_translations: Vec<[f32; 3]>,
    inverse_bind_matrices: Vec<[f32; 16]>,
}

/// Animation data container
struct AnimationData {
    times: Vec<f32>,
    translations: Vec<Vec<[f32; 3]>>,
    rotations: Vec<Vec<[f32; 4]>>,
    scales: Vec<Vec<[f32; 3]>>,
}

/// Generate a complete GLB with skinned mesh, skeleton, and animation
fn generate_skinned_glb() -> Vec<u8> {
    let mesh = create_mesh_data();
    let skeleton = create_skeleton();
    let animation = create_animation();

    let (buffer_data, buffer_views, accessors) =
        pack_binary_data(&mesh, &skeleton, &animation);

    let mut root = build_gltf_json(&mesh, &buffer_views, &accessors);

    // Update buffer size
    root.buffers[0].byte_length = (buffer_data.len() as u64).into();

    assemble_glb(&root, &buffer_data)
}

/// Create mesh data: 3 stacked cubes with proper UV mapping and vertex colors
fn create_mesh_data() -> MeshData {
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
        [1.0, 0.2, 0.2, 1.0],  // Red for root
        [0.2, 1.0, 0.2, 1.0],  // Green for spine
        [0.2, 0.2, 1.0, 1.0],  // Blue for head
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

    MeshData {
        positions,
        normals,
        uvs,
        colors,
        joints,
        weights,
        indices,
    }
}

/// Create skeleton data: 3-bone vertical chain
fn create_skeleton() -> SkeletonData {
    let mut bone_translations = Vec::new();
    let mut inverse_bind_matrices = Vec::new();

    for i in 0..BONE_COUNT {
        let y = i as f32 * SEGMENT_HEIGHT;
        bone_translations.push([0.0, y, 0.0]);

        // Inverse bind matrix: translate by -y
        #[rustfmt::skip]
        let ibm: [f32; 16] = [
            1.0, 0.0, 0.0, 0.0,
            0.0, 1.0, 0.0, 0.0,
            0.0, 0.0, 1.0, 0.0,
            0.0, -y, 0.0, 1.0,
        ];
        inverse_bind_matrices.push(ibm);
    }

    SkeletonData {
        bone_translations,
        inverse_bind_matrices,
    }
}

/// Create animation data: wave animation (Z-axis rotation)
fn create_animation() -> AnimationData {
    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    let duration = FRAME_COUNT as f32 / FRAME_RATE;

    for frame in 0..FRAME_COUNT {
        let t = frame as f32 / FRAME_COUNT as f32;
        times.push(t * duration);

        let wave_time = t * TAU;

        // Compute hierarchical transforms
        let mut accumulated_angle = 0.0f32;

        for bone in 0..BONE_COUNT {
            let phase = bone as f32 * 0.5;
            let amplitude = 0.4 + (bone as f32 * 0.1);

            // Local rotation
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

    AnimationData {
        times,
        translations,
        rotations,
        scales,
    }
}

/// Compute bounding box for positions
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

/// Align buffer to 4-byte boundary
fn align_buffer(buffer: &mut Vec<u8>) {
    while buffer.len() % 4 != 0 {
        buffer.push(0);
    }
}

/// Pack all binary data into buffer
fn pack_binary_data(
    mesh: &MeshData,
    skeleton: &SkeletonData,
    animation: &AnimationData,
) -> (Vec<u8>, Vec<json::buffer::View>, Vec<json::Accessor>) {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();
    let mut accessor_idx = 0u32;

    // --- Mesh data ---

    // Positions
    let pos_offset = buffer.len();
    for pos in &mesh.positions {
        buffer.extend_from_slice(bytemuck::cast_slice(pos));
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.positions.len() * 12).into(),
        byte_offset: Some((pos_offset as u64).into()),
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
        min: Some(json::Value::Array(min.into_iter().map(json::Value::from).collect())),
        max: Some(json::Value::Array(max.into_iter().map(json::Value::from).collect())),
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
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.normals.len() * 12).into(),
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
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.uvs.len() * 8).into(),
        byte_offset: Some((uv_offset as u64).into()),
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

    // Colors (COLOR_0)
    let color_offset = buffer.len();
    for color in &mesh.colors {
        buffer.extend_from_slice(bytemuck::cast_slice(color));
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.colors.len() * 16).into(), // vec4 f32 = 16 bytes
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
        count: mesh.colors.len().into(),
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
    let _color_accessor = accessor_idx;
    accessor_idx += 1;
    align_buffer(&mut buffer);

    // Joints (JOINTS_0)
    let joints_offset = buffer.len();
    for joint in &mesh.joints {
        buffer.extend_from_slice(joint);
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.joints.len() * 4).into(),
        byte_offset: Some((joints_offset as u64).into()),
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
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.weights.len() * 16).into(),
        byte_offset: Some((weights_offset as u64).into()),
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
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.indices.len() * 2).into(),
        byte_offset: Some((indices_offset as u64).into()),
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
        for f in mat {
            buffer.extend_from_slice(&f.to_le_bytes());
        }
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (skeleton.inverse_bind_matrices.len() * 64).into(),
        byte_offset: Some((ibm_offset as u64).into()),
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
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (animation.times.len() * 4).into(),
        byte_offset: Some((times_offset as u64).into()),
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
    let mut _trans_accessors = Vec::new();
    for bone_trans in &animation.translations {
        let offset = buffer.len();
        for t in bone_trans {
            buffer.extend_from_slice(bytemuck::cast_slice(t));
        }
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (bone_trans.len() * 12).into(),
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
        _trans_accessors.push(accessor_idx);
        accessor_idx += 1;
        align_buffer(&mut buffer);
    }

    // Animation rotations (one accessor per bone)
    let mut _rot_accessors = Vec::new();
    for bone_rot in &animation.rotations {
        let offset = buffer.len();
        for r in bone_rot {
            buffer.extend_from_slice(bytemuck::cast_slice(r));
        }
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (bone_rot.len() * 16).into(),
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
        _rot_accessors.push(accessor_idx);
        accessor_idx += 1;
        align_buffer(&mut buffer);
    }

    // Animation scales (one accessor per bone)
    let mut _scale_accessors = Vec::new();
    for bone_scale in &animation.scales {
        let offset = buffer.len();
        for s in bone_scale {
            buffer.extend_from_slice(bytemuck::cast_slice(s));
        }
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (bone_scale.len() * 12).into(),
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
        _scale_accessors.push(accessor_idx);
        accessor_idx += 1;
        align_buffer(&mut buffer);
    }

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

    // Create mesh with primitive
    // Accessor indices after adding colors:
    // 0=positions, 1=normals, 2=uvs, 3=colors, 4=joints, 5=weights, 6=indices
    // 7=inverse_bind_matrices, 8=times, 9-11=translations, 12-14=rotations, 15-17=scales
    let meshes = vec![json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("TestMesh".to_string()),
        primitives: vec![json::mesh::Primitive {
            attributes: {
                let mut attrs = std::collections::BTreeMap::new();
                attrs.insert(
                    Valid(json::mesh::Semantic::Positions),
                    json::Index::new(0),
                );
                attrs.insert(
                    Valid(json::mesh::Semantic::Normals),
                    json::Index::new(1),
                );
                attrs.insert(
                    Valid(json::mesh::Semantic::TexCoords(0)),
                    json::Index::new(2),
                );
                attrs.insert(
                    Valid(json::mesh::Semantic::Colors(0)),
                    json::Index::new(3),
                );
                attrs.insert(
                    Valid(json::mesh::Semantic::Joints(0)),
                    json::Index::new(4),
                );
                attrs.insert(
                    Valid(json::mesh::Semantic::Weights(0)),
                    json::Index::new(5),
                );
                attrs
            },
            extensions: Default::default(),
            extras: Default::default(),
            indices: Some(json::Index::new(6)),
            material: None,
            mode: Valid(json::mesh::Mode::Triangles),
            targets: None,
        }],
        weights: None,
    }];

    // Create skin (inverse_bind_matrices is accessor 7 now)
    let skins = vec![json::Skin {
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(json::Index::new(7)),
        joints: vec![
            json::Index::new(ROOT_NODE),
            json::Index::new(SPINE_NODE),
            json::Index::new(HEAD_NODE),
        ],
        name: Some("TestSkin".to_string()),
        skeleton: Some(json::Index::new(ROOT_NODE)),
    }];

    // Create animation
    // Accessor indices (after adding colors): 8=times, 9-11=trans, 12-14=rot, 15-17=scale
    let mut samplers = Vec::new();
    let mut channels = Vec::new();

    for bone in 0..BONE_COUNT {
        let bone_u32 = bone as u32;

        // Translation sampler
        samplers.push(json::animation::Sampler {
            input: json::Index::new(8), // times
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(9 + bone_u32), // translations[bone]
            extensions: Default::default(),
            extras: Default::default(),
        });
        channels.push(json::animation::Channel {
            sampler: json::Index::new(samplers.len() as u32 - 1),
            target: json::animation::Target {
                node: json::Index::new(bone_u32),
                path: Valid(json::animation::Property::Translation),
                extensions: Default::default(),
                extras: Default::default(),
            },
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Rotation sampler
        samplers.push(json::animation::Sampler {
            input: json::Index::new(8), // times
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(12 + bone_u32), // rotations[bone]
            extensions: Default::default(),
            extras: Default::default(),
        });
        channels.push(json::animation::Channel {
            sampler: json::Index::new(samplers.len() as u32 - 1),
            target: json::animation::Target {
                node: json::Index::new(bone_u32),
                path: Valid(json::animation::Property::Rotation),
                extensions: Default::default(),
                extras: Default::default(),
            },
            extensions: Default::default(),
            extras: Default::default(),
        });

        // Scale sampler
        samplers.push(json::animation::Sampler {
            input: json::Index::new(8), // times
            interpolation: Valid(json::animation::Interpolation::Linear),
            output: json::Index::new(15 + bone_u32), // scales[bone]
            extensions: Default::default(),
            extras: Default::default(),
        });
        channels.push(json::animation::Channel {
            sampler: json::Index::new(samplers.len() as u32 - 1),
            target: json::animation::Target {
                node: json::Index::new(bone_u32),
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
        name: Some("Scene".to_string()),
        nodes: vec![
            json::Index::new(ROOT_NODE),
            json::Index::new(MESH_NODE),
        ],
    }];

    // Create buffer (byte length will be updated later)
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
            generator: Some("gen-gltf-test-assets".to_string()),
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
        skins,
        textures: Vec::new(),
    }
}

/// Assemble GLB binary from JSON and buffer data
fn assemble_glb(root: &json::Root, buffer_data: &[u8]) -> Vec<u8> {
    let json_string = json::serialize::to_string(root).expect("Failed to serialize GLTF JSON");
    let json_bytes = json_string.as_bytes();

    // Pad JSON to 4-byte alignment
    let json_padding = (4 - (json_bytes.len() % 4)) % 4;
    let json_chunk_length = json_bytes.len() + json_padding;

    // Pad buffer to 4-byte alignment
    let buffer_padding = (4 - (buffer_data.len() % 4)) % 4;
    let buffer_chunk_length = buffer_data.len() + buffer_padding;

    // Total file length
    let total_length = 12 + 8 + json_chunk_length + 8 + buffer_chunk_length;

    let mut glb = Vec::with_capacity(total_length);

    // GLB header
    glb.extend_from_slice(b"glTF");
    glb.extend_from_slice(&2u32.to_le_bytes()); // version
    glb.extend_from_slice(&(total_length as u32).to_le_bytes());

    // JSON chunk
    glb.extend_from_slice(&(json_chunk_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x4E4F534Au32.to_le_bytes()); // "JSON"
    glb.extend_from_slice(json_bytes);
    for _ in 0..json_padding {
        glb.push(0x20); // Space for JSON padding
    }

    // Binary chunk
    glb.extend_from_slice(&(buffer_chunk_length as u32).to_le_bytes());
    glb.extend_from_slice(&0x004E4942u32.to_le_bytes()); // "BIN\0"
    glb.extend_from_slice(buffer_data);
    for _ in 0..buffer_padding {
        glb.push(0);
    }

    glb
}
