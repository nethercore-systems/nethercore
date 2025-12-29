//! Generate assets for multi-skinned-rom example
//!
//! Creates skeleton, mesh, and animation files for testing ROM-backed skinned meshes.

use std::f32::consts::TAU;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use zx_common::formats::animation::{encode_bone_transform, NetherZXAnimationHeader};
use zx_common::formats::mesh::NetherZXMeshHeader;
use zx_common::formats::skeleton::NetherZXSkeletonHeader;
use zx_common::packing::{pack_vertex_data, FORMAT_NORMAL, FORMAT_SKINNED};

const FORMAT_POS_NORMAL_SKINNED: u8 = FORMAT_NORMAL | FORMAT_SKINNED;

fn main() {
    // Output to shared examples/assets folder with multi-skinned- prefix
    let output_dir = PathBuf::from("examples/assets");

    // Ensure output directory exists
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Generate assets for character 1 (3-bone vertical arm)
    generate_skeleton(
        &output_dir.join("multi-skinned-arm1.nczxskel"),
        3,
        &[
            [0.0, 0.0, 0.0],  // Bone 0: origin
            [0.0, -1.5, 0.0], // Bone 1: inverse of T(0, 1.5, 0)
            [0.0, -3.0, 0.0], // Bone 2: inverse of T(0, 3.0, 0)
        ],
    );

    generate_arm_mesh(&output_dir.join("multi-skinned-arm1.nczxmesh"), 3, 1.5, true);

    generate_animation(
        &output_dir.join("multi-skinned-wave1.nczxanim"),
        3,
        30,
        &[
            (0.0, 0.5), // Bone 0: phase 0, amplitude 0.5
            (0.5, 0.7), // Bone 1: phase 0.5, amplitude 0.7
            (1.0, 0.4), // Bone 2: phase 1.0, amplitude 0.4
        ],
    );

    // Generate assets for character 2 (4-bone horizontal arm)
    generate_skeleton(
        &output_dir.join("multi-skinned-arm2.nczxskel"),
        4,
        &[
            [0.0, 0.0, 0.0],  // Bone 0: origin
            [-1.0, 0.0, 0.0], // Bone 1: inverse of T(1, 0, 0)
            [-2.0, 0.0, 0.0], // Bone 2: inverse of T(2, 0, 0)
            [-3.0, 0.0, 0.0], // Bone 3: inverse of T(3, 0, 0)
        ],
    );

    generate_horizontal_arm_mesh(&output_dir.join("multi-skinned-arm2.nczxmesh"), 4, 1.0);

    generate_horizontal_animation(&output_dir.join("multi-skinned-wave2.nczxanim"), 4, 30);

    println!("Assets generated successfully in {}", output_dir.display());
}

/// Generate a skeleton file with inverse bind matrices
fn generate_skeleton(path: &PathBuf, bone_count: u32, translations: &[[f32; 3]]) {
    let header = NetherZXSkeletonHeader::new(bone_count);

    let mut file = File::create(path).expect("Failed to create skeleton file");
    file.write_all(&header.to_bytes())
        .expect("Failed to write header");

    // Write inverse bind matrices (3x4 column-major)
    for translation in translations.iter().take(bone_count as usize) {
        // Identity rotation, translation for inverse bind
        let matrix: [f32; 12] = [
            1.0,
            0.0,
            0.0, // col 0
            0.0,
            1.0,
            0.0, // col 1
            0.0,
            0.0,
            1.0, // col 2
            translation[0],
            translation[1],
            translation[2], // col 3
        ];
        for f in matrix {
            file.write_all(&f.to_le_bytes())
                .expect("Failed to write matrix");
        }
    }

    println!("Generated {} ({} bones)", path.display(), bone_count);
}

/// Generate a vertical arm mesh (box segments along Y axis)
fn generate_arm_mesh(path: &PathBuf, bone_count: u32, segment_height: f32, _vertical: bool) {
    let half_w = 0.15;

    // Build unpacked vertex data
    let mut vertices: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for seg in 0..bone_count {
        let y_base = seg as f32 * segment_height;
        let bone = seg;
        let base_vert = (seg * 24) as u16;

        // Pack bone indices (same bone for all 4 indices)
        let bone_packed = f32::from_bits(bone | (bone << 8) | (bone << 16) | (bone << 24));

        // 6 faces, 4 vertices each
        let faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
            // Front (+Z)
            (
                [0.0, 0.0, 1.0],
                [
                    [-half_w, y_base, half_w],
                    [half_w, y_base, half_w],
                    [half_w, y_base + segment_height, half_w],
                    [-half_w, y_base + segment_height, half_w],
                ],
            ),
            // Back (-Z)
            (
                [0.0, 0.0, -1.0],
                [
                    [half_w, y_base, -half_w],
                    [-half_w, y_base, -half_w],
                    [-half_w, y_base + segment_height, -half_w],
                    [half_w, y_base + segment_height, -half_w],
                ],
            ),
            // Right (+X)
            (
                [1.0, 0.0, 0.0],
                [
                    [half_w, y_base, half_w],
                    [half_w, y_base, -half_w],
                    [half_w, y_base + segment_height, -half_w],
                    [half_w, y_base + segment_height, half_w],
                ],
            ),
            // Left (-X)
            (
                [-1.0, 0.0, 0.0],
                [
                    [-half_w, y_base, -half_w],
                    [-half_w, y_base, half_w],
                    [-half_w, y_base + segment_height, half_w],
                    [-half_w, y_base + segment_height, -half_w],
                ],
            ),
            // Top (+Y)
            (
                [0.0, 1.0, 0.0],
                [
                    [-half_w, y_base + segment_height, half_w],
                    [half_w, y_base + segment_height, half_w],
                    [half_w, y_base + segment_height, -half_w],
                    [-half_w, y_base + segment_height, -half_w],
                ],
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
            ),
        ];

        for (face_idx, (normal, corners)) in faces.iter().enumerate() {
            let face_base = base_vert + (face_idx * 4) as u16;

            for corner in corners {
                // Position
                vertices.push(corner[0]);
                vertices.push(corner[1]);
                vertices.push(corner[2]);
                // Normal
                vertices.push(normal[0]);
                vertices.push(normal[1]);
                vertices.push(normal[2]);
                // Bone indices (packed)
                vertices.push(bone_packed);
                // Weights
                vertices.push(1.0);
                vertices.push(0.0);
                vertices.push(0.0);
                vertices.push(0.0);
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

    let vertex_count = (vertices.len() / 11) as u32;
    let index_count = indices.len() as u32;

    // Pack vertices to GPU format
    let packed_vertices = pack_vertex_data(&vertices, FORMAT_POS_NORMAL_SKINNED);

    // Write mesh file
    let header = NetherZXMeshHeader::new(vertex_count, index_count, FORMAT_POS_NORMAL_SKINNED);

    let mut file = File::create(path).expect("Failed to create mesh file");
    file.write_all(&header.to_bytes())
        .expect("Failed to write header");
    file.write_all(&packed_vertices)
        .expect("Failed to write vertices");
    for idx in &indices {
        file.write_all(&idx.to_le_bytes())
            .expect("Failed to write index");
    }

    println!(
        "Generated {} ({} vertices, {} indices)",
        path.display(),
        vertex_count,
        index_count
    );
}

/// Generate a horizontal arm mesh (box segments along X axis)
fn generate_horizontal_arm_mesh(path: &PathBuf, bone_count: u32, segment_length: f32) {
    let half_h = 0.12;

    let mut vertices: Vec<f32> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for seg in 0..bone_count {
        let x_base = seg as f32 * segment_length;
        let bone = seg;
        let base_vert = (seg * 24) as u16;

        let bone_packed = f32::from_bits(bone | (bone << 8) | (bone << 16) | (bone << 24));

        let faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
            // Front (+Z)
            (
                [0.0, 0.0, 1.0],
                [
                    [x_base, -half_h, half_h],
                    [x_base + segment_length, -half_h, half_h],
                    [x_base + segment_length, half_h, half_h],
                    [x_base, half_h, half_h],
                ],
            ),
            // Back (-Z)
            (
                [0.0, 0.0, -1.0],
                [
                    [x_base + segment_length, -half_h, -half_h],
                    [x_base, -half_h, -half_h],
                    [x_base, half_h, -half_h],
                    [x_base + segment_length, half_h, -half_h],
                ],
            ),
            // Top (+Y)
            (
                [0.0, 1.0, 0.0],
                [
                    [x_base, half_h, half_h],
                    [x_base + segment_length, half_h, half_h],
                    [x_base + segment_length, half_h, -half_h],
                    [x_base, half_h, -half_h],
                ],
            ),
            // Bottom (-Y)
            (
                [0.0, -1.0, 0.0],
                [
                    [x_base, -half_h, -half_h],
                    [x_base + segment_length, -half_h, -half_h],
                    [x_base + segment_length, -half_h, half_h],
                    [x_base, -half_h, half_h],
                ],
            ),
            // Right (+X)
            (
                [1.0, 0.0, 0.0],
                [
                    [x_base + segment_length, -half_h, half_h],
                    [x_base + segment_length, -half_h, -half_h],
                    [x_base + segment_length, half_h, -half_h],
                    [x_base + segment_length, half_h, half_h],
                ],
            ),
            // Left (-X)
            (
                [-1.0, 0.0, 0.0],
                [
                    [x_base, -half_h, -half_h],
                    [x_base, -half_h, half_h],
                    [x_base, half_h, half_h],
                    [x_base, half_h, -half_h],
                ],
            ),
        ];

        for (face_idx, (normal, corners)) in faces.iter().enumerate() {
            let face_base = base_vert + (face_idx * 4) as u16;

            for corner in corners {
                vertices.push(corner[0]);
                vertices.push(corner[1]);
                vertices.push(corner[2]);
                vertices.push(normal[0]);
                vertices.push(normal[1]);
                vertices.push(normal[2]);
                vertices.push(bone_packed);
                vertices.push(1.0);
                vertices.push(0.0);
                vertices.push(0.0);
                vertices.push(0.0);
            }

            indices.push(face_base);
            indices.push(face_base + 1);
            indices.push(face_base + 2);
            indices.push(face_base);
            indices.push(face_base + 2);
            indices.push(face_base + 3);
        }
    }

    let vertex_count = (vertices.len() / 11) as u32;
    let index_count = indices.len() as u32;

    let packed_vertices = pack_vertex_data(&vertices, FORMAT_POS_NORMAL_SKINNED);

    let header = NetherZXMeshHeader::new(vertex_count, index_count, FORMAT_POS_NORMAL_SKINNED);

    let mut file = File::create(path).expect("Failed to create mesh file");
    file.write_all(&header.to_bytes())
        .expect("Failed to write header");
    file.write_all(&packed_vertices)
        .expect("Failed to write vertices");
    for idx in &indices {
        file.write_all(&idx.to_le_bytes())
            .expect("Failed to write index");
    }

    println!(
        "Generated {} ({} vertices, {} indices)",
        path.display(),
        vertex_count,
        index_count
    );
}

/// Generate vertical arm animation (Z-axis rotations) with proper hierarchical chaining
fn generate_animation(path: &PathBuf, bone_count: u8, frame_count: u16, params: &[(f32, f32)]) {
    let header = NetherZXAnimationHeader::new(bone_count, frame_count);

    let mut file = File::create(path).expect("Failed to create animation file");
    file.write_all(&header.to_bytes())
        .expect("Failed to write header");

    let segment_length = 1.5f32;

    for frame in 0..frame_count {
        let t = (frame as f32 / frame_count as f32) * TAU;

        // Compute hierarchical world transforms for each bone
        // We need to chain: bone[i] world = bone[i-1] world * bone[i] local
        let mut world_positions: Vec<[f32; 3]> = Vec::new();
        let mut world_rotations: Vec<[f32; 4]> = Vec::new();
        let mut accumulated_angle = 0.0f32;

        for bone in 0..bone_count {
            let (phase, amplitude) = if (bone as usize) < params.len() {
                params[bone as usize]
            } else {
                (0.0, 0.3)
            };

            // This bone's LOCAL rotation (relative to parent)
            let local_angle = (t + phase).sin() * amplitude;
            accumulated_angle += local_angle;

            // Compute world position by rotating through the chain
            let world_pos = if bone == 0 {
                [0.0, 0.0, 0.0] // Root bone at origin
            } else {
                // Position is parent's position + segment rotated by parent's accumulated rotation
                let parent_pos = world_positions[bone as usize - 1];
                let parent_angle = accumulated_angle - local_angle; // Parent's total rotation

                // Rotate the segment vector by parent's rotation around Z
                let c = parent_angle.cos();
                let s = parent_angle.sin();
                // Segment goes from parent along Y axis (0, segment_length, 0)
                // Rotated around Z: (x', y') = (x*c - y*s, x*s + y*c)
                let dx = -segment_length * s;
                let dy = segment_length * c;

                [parent_pos[0] + dx, parent_pos[1] + dy, parent_pos[2]]
            };

            // World rotation quaternion (Z-axis rotation)
            let half_angle = accumulated_angle * 0.5;
            let world_quat = [0.0f32, 0.0, half_angle.sin(), half_angle.cos()];

            world_positions.push(world_pos);
            world_rotations.push(world_quat);

            let keyframe = encode_bone_transform(world_quat, world_pos, [1.0, 1.0, 1.0]);
            file.write_all(&keyframe.to_bytes())
                .expect("Failed to write keyframe");
        }
    }

    println!(
        "Generated {} ({} bones, {} frames)",
        path.display(),
        bone_count,
        frame_count
    );
}

/// Generate horizontal arm animation (Y-axis rotations) with proper hierarchical chaining
fn generate_horizontal_animation(path: &PathBuf, bone_count: u8, frame_count: u16) {
    let header = NetherZXAnimationHeader::new(bone_count, frame_count);

    let mut file = File::create(path).expect("Failed to create animation file");
    file.write_all(&header.to_bytes())
        .expect("Failed to write header");

    let segment_length = 1.0f32;

    for frame in 0..frame_count {
        let t = (frame as f32 / frame_count as f32) * TAU;

        // Compute hierarchical world transforms for each bone
        let mut world_positions: Vec<[f32; 3]> = Vec::new();
        let mut accumulated_angle = 0.0f32;

        for bone in 0..bone_count {
            let phase = bone as f32 * 0.3;
            let amplitude = 0.3 + (bone as f32 * 0.1);

            // This bone's LOCAL rotation (relative to parent)
            let local_angle = (t + phase).sin() * amplitude;
            accumulated_angle += local_angle;

            // Compute world position by rotating through the chain
            let world_pos = if bone == 0 {
                [0.0, 0.0, 0.0] // Root bone at origin
            } else {
                // Position is parent's position + segment rotated by parent's accumulated rotation
                let parent_pos = world_positions[bone as usize - 1];
                let parent_angle = accumulated_angle - local_angle; // Parent's total rotation

                // Rotate the segment vector by parent's rotation around Y
                let c = parent_angle.cos();
                let s = parent_angle.sin();
                // Segment goes from parent along X axis (segment_length, 0, 0)
                // Rotated around Y: (x', z') = (x*c + z*s, -x*s + z*c)
                let dx = segment_length * c;
                let dz = -segment_length * s;

                [parent_pos[0] + dx, parent_pos[1], parent_pos[2] + dz]
            };

            // World rotation quaternion (Y-axis rotation)
            let half_angle = accumulated_angle * 0.5;
            let world_quat = [0.0f32, half_angle.sin(), 0.0, half_angle.cos()];

            world_positions.push(world_pos);

            let keyframe = encode_bone_transform(world_quat, world_pos, [1.0, 1.0, 1.0]);
            file.write_all(&keyframe.to_bytes())
                .expect("Failed to write keyframe");
        }
    }

    println!(
        "Generated {} ({} bones, {} frames)",
        path.display(),
        bone_count,
        frame_count
    );
}
