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
use gltf_json as json;
use json::validation::Checked::Valid;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

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
    println!("Generated GLB: {} ({} bytes)", glb_path.display(), glb_data.len());

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

/// Mesh data container
struct MeshData {
    positions: Vec<[f32; 3]>,
    normals: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    colors: Vec<[f32; 4]>,
    joints: Vec<[u8; 4]>,
    weights: Vec<[f32; 4]>,
    indices: Vec<u16>,
}

/// Skeleton data container
struct SkeletonData {
    #[allow(dead_code)]
    bone_translations: Vec<[f32; 3]>,
    inverse_bind_matrices: Vec<[f32; 16]>,
}

/// Animation data container
struct AnimationData {
    name: String,
    times: Vec<f32>,
    translations: Vec<Vec<[f32; 3]>>,
    rotations: Vec<Vec<[f32; 4]>>,
    scales: Vec<Vec<[f32; 3]>>,
}

/// Generate a complete GLB with skinned mesh, skeleton, and THREE animations
fn generate_multi_animation_glb() -> Vec<u8> {
    let mesh = create_mesh_data();
    let skeleton = create_skeleton();

    // Create all three animations
    let wave_anim = create_wave_animation();
    let bounce_anim = create_bounce_animation();
    let twist_anim = create_twist_animation();

    let animations = vec![wave_anim, bounce_anim, twist_anim];

    let (buffer_data, buffer_views, accessors) =
        pack_binary_data(&mesh, &skeleton, &animations);

    let mut root = build_gltf_json(&buffer_views, &accessors, &animations);

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

/// Create Wave animation: side-to-side Z-rotation
fn create_wave_animation() -> AnimationData {
    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    let duration = FRAME_COUNT as f32 / FRAME_RATE;

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

    AnimationData {
        name: "Wave".to_string(),
        times,
        translations,
        rotations,
        scales,
    }
}

/// Create Bounce animation: vertical Y-translation bounce
fn create_bounce_animation() -> AnimationData {
    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    let duration = FRAME_COUNT as f32 / FRAME_RATE;

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

    AnimationData {
        name: "Bounce".to_string(),
        times,
        translations,
        rotations,
        scales,
    }
}

/// Create Twist animation: Y-axis rotation twist
fn create_twist_animation() -> AnimationData {
    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); BONE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); BONE_COUNT];

    let duration = FRAME_COUNT as f32 / FRAME_RATE;

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

    AnimationData {
        name: "Twist".to_string(),
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
    animations: &[AnimationData],
) -> (Vec<u8>, Vec<json::buffer::View>, Vec<json::Accessor>) {
    let mut buffer = Vec::new();
    let mut views = Vec::new();
    let mut accessors = Vec::new();

    // --- Mesh data ---

    // Positions (accessor 0)
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
    align_buffer(&mut buffer);

    // Normals (accessor 1)
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
    align_buffer(&mut buffer);

    // UVs (accessor 2)
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
    align_buffer(&mut buffer);

    // Colors (accessor 3)
    let color_offset = buffer.len();
    for color in &mesh.colors {
        buffer.extend_from_slice(bytemuck::cast_slice(color));
    }
    views.push(json::buffer::View {
        buffer: json::Index::new(0),
        byte_length: (mesh.colors.len() * 16).into(),
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
    align_buffer(&mut buffer);

    // Joints (accessor 4)
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
    align_buffer(&mut buffer);

    // Weights (accessor 5)
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
    align_buffer(&mut buffer);

    // Indices (accessor 6)
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
    align_buffer(&mut buffer);

    // --- Skeleton data ---

    // Inverse bind matrices (accessor 7)
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
    align_buffer(&mut buffer);

    // --- Animation data (for each animation) ---
    for anim in animations {
        // Animation times
        let times_offset = buffer.len();
        for t in &anim.times {
            buffer.extend_from_slice(&t.to_le_bytes());
        }
        views.push(json::buffer::View {
            buffer: json::Index::new(0),
            byte_length: (anim.times.len() * 4).into(),
            byte_offset: Some((times_offset as u64).into()),
            byte_stride: None,
            extensions: Default::default(),
            extras: Default::default(),
            name: None,
            target: None,
        });
        let time_min = anim.times.first().copied().unwrap_or(0.0) as f64;
        let time_max = anim.times.last().copied().unwrap_or(1.0) as f64;
        accessors.push(json::Accessor {
            buffer_view: Some(json::Index::new(views.len() as u32 - 1)),
            byte_offset: Some(0u64.into()),
            count: anim.times.len().into(),
            component_type: Valid(json::accessor::GenericComponentType(
                json::accessor::ComponentType::F32,
            )),
            extensions: Default::default(),
            extras: Default::default(),
            type_: Valid(json::accessor::Type::Scalar),
            min: Some(json::Value::Array(vec![json::Value::from(time_min)])),
            max: Some(json::Value::Array(vec![json::Value::from(time_max)])),
            name: None,
            normalized: false,
            sparse: None,
        });
        align_buffer(&mut buffer);

        // Translations for each bone
        for bone_trans in &anim.translations {
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
            align_buffer(&mut buffer);
        }

        // Rotations for each bone
        for bone_rot in &anim.rotations {
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
            align_buffer(&mut buffer);
        }

        // Scales for each bone
        for bone_scale in &anim.scales {
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
            align_buffer(&mut buffer);
        }
    }

    (buffer, views, accessors)
}

/// Build the GLTF JSON structure
fn build_gltf_json(
    buffer_views: &[json::buffer::View],
    accessors: &[json::Accessor],
    animations_data: &[AnimationData],
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
            name: Some("CharacterMesh".to_string()),
            rotation: None,
            scale: None,
            skin: Some(json::Index::new(0)),
            translation: None,
            weights: None,
        },
    ];

    // Create mesh with primitive
    let meshes = vec![json::Mesh {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("CharacterMesh".to_string()),
        primitives: vec![json::mesh::Primitive {
            attributes: {
                let mut attrs = std::collections::BTreeMap::new();
                attrs.insert(Valid(json::mesh::Semantic::Positions), json::Index::new(0));
                attrs.insert(Valid(json::mesh::Semantic::Normals), json::Index::new(1));
                attrs.insert(Valid(json::mesh::Semantic::TexCoords(0)), json::Index::new(2));
                attrs.insert(Valid(json::mesh::Semantic::Colors(0)), json::Index::new(3));
                attrs.insert(Valid(json::mesh::Semantic::Joints(0)), json::Index::new(4));
                attrs.insert(Valid(json::mesh::Semantic::Weights(0)), json::Index::new(5));
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

    // Create skin
    let skins = vec![json::Skin {
        extensions: Default::default(),
        extras: Default::default(),
        inverse_bind_matrices: Some(json::Index::new(7)),
        joints: vec![
            json::Index::new(ROOT_NODE),
            json::Index::new(SPINE_NODE),
            json::Index::new(HEAD_NODE),
        ],
        name: Some("CharacterSkin".to_string()),
        skeleton: Some(json::Index::new(ROOT_NODE)),
    }];

    // Create animations
    // Base accessor index after mesh data (0-6) and skeleton (7)
    // Each animation uses: 1 time accessor + 3 trans + 3 rot + 3 scale = 10 accessors
    let mut gltf_animations = Vec::new();
    let mut accessor_base = 8u32; // Start after mesh (0-6) and skeleton (7)

    for anim_data in animations_data {
        let times_accessor = accessor_base;
        accessor_base += 1;

        let trans_start = accessor_base;
        accessor_base += BONE_COUNT as u32;

        let rot_start = accessor_base;
        accessor_base += BONE_COUNT as u32;

        let scale_start = accessor_base;
        accessor_base += BONE_COUNT as u32;

        let mut samplers = Vec::new();
        let mut channels = Vec::new();

        for bone in 0..BONE_COUNT {
            let bone_u32 = bone as u32;

            // Translation sampler
            samplers.push(json::animation::Sampler {
                input: json::Index::new(times_accessor),
                interpolation: Valid(json::animation::Interpolation::Linear),
                output: json::Index::new(trans_start + bone_u32),
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
                input: json::Index::new(times_accessor),
                interpolation: Valid(json::animation::Interpolation::Linear),
                output: json::Index::new(rot_start + bone_u32),
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
                input: json::Index::new(times_accessor),
                interpolation: Valid(json::animation::Interpolation::Linear),
                output: json::Index::new(scale_start + bone_u32),
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

        gltf_animations.push(json::Animation {
            channels,
            extensions: Default::default(),
            extras: Default::default(),
            name: Some(anim_data.name.clone()),
            samplers,
        });
    }

    // Create scene
    let scenes = vec![json::Scene {
        extensions: Default::default(),
        extras: Default::default(),
        name: Some("Scene".to_string()),
        nodes: vec![json::Index::new(ROOT_NODE), json::Index::new(MESH_NODE)],
    }];

    // Create buffer
    let buffers = vec![json::Buffer {
        byte_length: 0u64.into(),
        extensions: Default::default(),
        extras: Default::default(),
        name: None,
        uri: None,
    }];

    json::Root {
        accessors: accessors.to_vec(),
        animations: gltf_animations,
        asset: json::Asset {
            copyright: None,
            extensions: Default::default(),
            extras: Default::default(),
            generator: Some("gen-glb-inline-assets".to_string()),
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
