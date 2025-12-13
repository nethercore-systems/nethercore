//! Generate assets for multi-skinned-rom example
//!
//! Creates skeleton, mesh, and animation files for testing ROM-backed skinned meshes.

use std::f32::consts::TAU;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use z_common::formats::animation::{encode_bone_transform, EmberZAnimationHeader};
use z_common::formats::mesh::EmberZMeshHeader;
use z_common::formats::skeleton::EmberZSkeletonHeader;
use z_common::packing::{pack_vertex_data, FORMAT_NORMAL, FORMAT_SKINNED};

const FORMAT_POS_NORMAL_SKINNED: u8 = FORMAT_NORMAL | FORMAT_SKINNED;

fn main() {
    let output_dir = PathBuf::from("examples/multi-skinned-rom/assets");

    // Ensure output directory exists
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Generate assets for character 1 (3-bone vertical arm)
    generate_skeleton(&output_dir.join("arm1.ewzskel"), 3, &[
        [0.0, 0.0, 0.0],    // Bone 0: origin
        [0.0, -1.5, 0.0],   // Bone 1: inverse of T(0, 1.5, 0)
        [0.0, -3.0, 0.0],   // Bone 2: inverse of T(0, 3.0, 0)
    ]);

    generate_arm_mesh(&output_dir.join("arm1.ewzmesh"), 3, 1.5, true);

    generate_animation(&output_dir.join("wave1.ewzanim"), 3, 30, &[
        (0.0, 0.5),   // Bone 0: phase 0, amplitude 0.5
        (0.5, 0.7),   // Bone 1: phase 0.5, amplitude 0.7
        (1.0, 0.4),   // Bone 2: phase 1.0, amplitude 0.4
    ]);

    // Generate assets for character 2 (4-bone horizontal arm)
    generate_skeleton(&output_dir.join("arm2.ewzskel"), 4, &[
        [0.0, 0.0, 0.0],    // Bone 0: origin
        [-1.0, 0.0, 0.0],   // Bone 1: inverse of T(1, 0, 0)
        [-2.0, 0.0, 0.0],   // Bone 2: inverse of T(2, 0, 0)
        [-3.0, 0.0, 0.0],   // Bone 3: inverse of T(3, 0, 0)
    ]);

    generate_horizontal_arm_mesh(&output_dir.join("arm2.ewzmesh"), 4, 1.0);

    generate_horizontal_animation(&output_dir.join("wave2.ewzanim"), 4, 30);

    println!("Assets generated successfully in {}", output_dir.display());
}

/// Generate a skeleton file with inverse bind matrices
fn generate_skeleton(path: &PathBuf, bone_count: u32, translations: &[[f32; 3]]) {
    let header = EmberZSkeletonHeader::new(bone_count);

    let mut file = File::create(path).expect("Failed to create skeleton file");
    file.write_all(&header.to_bytes()).expect("Failed to write header");

    // Write inverse bind matrices (3x4 column-major)
    for translation in translations.iter().take(bone_count as usize) {
        // Identity rotation, translation for inverse bind
        let matrix: [f32; 12] = [
            1.0, 0.0, 0.0,  // col 0
            0.0, 1.0, 0.0,  // col 1
            0.0, 0.0, 1.0,  // col 2
            translation[0], translation[1], translation[2],  // col 3
        ];
        for f in matrix {
            file.write_all(&f.to_le_bytes()).expect("Failed to write matrix");
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
            ([0.0, 0.0, 1.0], [
                [-half_w, y_base, half_w],
                [half_w, y_base, half_w],
                [half_w, y_base + segment_height, half_w],
                [-half_w, y_base + segment_height, half_w],
            ]),
            // Back (-Z)
            ([0.0, 0.0, -1.0], [
                [half_w, y_base, -half_w],
                [-half_w, y_base, -half_w],
                [-half_w, y_base + segment_height, -half_w],
                [half_w, y_base + segment_height, -half_w],
            ]),
            // Right (+X)
            ([1.0, 0.0, 0.0], [
                [half_w, y_base, half_w],
                [half_w, y_base, -half_w],
                [half_w, y_base + segment_height, -half_w],
                [half_w, y_base + segment_height, half_w],
            ]),
            // Left (-X)
            ([-1.0, 0.0, 0.0], [
                [-half_w, y_base, -half_w],
                [-half_w, y_base, half_w],
                [-half_w, y_base + segment_height, half_w],
                [-half_w, y_base + segment_height, -half_w],
            ]),
            // Top (+Y)
            ([0.0, 1.0, 0.0], [
                [-half_w, y_base + segment_height, half_w],
                [half_w, y_base + segment_height, half_w],
                [half_w, y_base + segment_height, -half_w],
                [-half_w, y_base + segment_height, -half_w],
            ]),
            // Bottom (-Y)
            ([0.0, -1.0, 0.0], [
                [-half_w, y_base, -half_w],
                [half_w, y_base, -half_w],
                [half_w, y_base, half_w],
                [-half_w, y_base, half_w],
            ]),
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
    let header = EmberZMeshHeader::new(vertex_count, index_count, FORMAT_POS_NORMAL_SKINNED);

    let mut file = File::create(path).expect("Failed to create mesh file");
    file.write_all(&header.to_bytes()).expect("Failed to write header");
    file.write_all(&packed_vertices).expect("Failed to write vertices");
    for idx in &indices {
        file.write_all(&idx.to_le_bytes()).expect("Failed to write index");
    }

    println!("Generated {} ({} vertices, {} indices)", path.display(), vertex_count, index_count);
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
            ([0.0, 0.0, 1.0], [
                [x_base, -half_h, half_h],
                [x_base + segment_length, -half_h, half_h],
                [x_base + segment_length, half_h, half_h],
                [x_base, half_h, half_h],
            ]),
            // Back (-Z)
            ([0.0, 0.0, -1.0], [
                [x_base + segment_length, -half_h, -half_h],
                [x_base, -half_h, -half_h],
                [x_base, half_h, -half_h],
                [x_base + segment_length, half_h, -half_h],
            ]),
            // Top (+Y)
            ([0.0, 1.0, 0.0], [
                [x_base, half_h, half_h],
                [x_base + segment_length, half_h, half_h],
                [x_base + segment_length, half_h, -half_h],
                [x_base, half_h, -half_h],
            ]),
            // Bottom (-Y)
            ([0.0, -1.0, 0.0], [
                [x_base, -half_h, -half_h],
                [x_base + segment_length, -half_h, -half_h],
                [x_base + segment_length, -half_h, half_h],
                [x_base, -half_h, half_h],
            ]),
            // Right (+X)
            ([1.0, 0.0, 0.0], [
                [x_base + segment_length, -half_h, half_h],
                [x_base + segment_length, -half_h, -half_h],
                [x_base + segment_length, half_h, -half_h],
                [x_base + segment_length, half_h, half_h],
            ]),
            // Left (-X)
            ([-1.0, 0.0, 0.0], [
                [x_base, -half_h, -half_h],
                [x_base, -half_h, half_h],
                [x_base, half_h, half_h],
                [x_base, half_h, -half_h],
            ]),
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

    let header = EmberZMeshHeader::new(vertex_count, index_count, FORMAT_POS_NORMAL_SKINNED);

    let mut file = File::create(path).expect("Failed to create mesh file");
    file.write_all(&header.to_bytes()).expect("Failed to write header");
    file.write_all(&packed_vertices).expect("Failed to write vertices");
    for idx in &indices {
        file.write_all(&idx.to_le_bytes()).expect("Failed to write index");
    }

    println!("Generated {} ({} vertices, {} indices)", path.display(), vertex_count, index_count);
}

/// Generate vertical arm animation (Z-axis rotations)
fn generate_animation(path: &PathBuf, bone_count: u8, frame_count: u16, params: &[(f32, f32)]) {
    let header = EmberZAnimationHeader::new(bone_count, frame_count);

    let mut file = File::create(path).expect("Failed to create animation file");
    file.write_all(&header.to_bytes()).expect("Failed to write header");

    for frame in 0..frame_count {
        let t = (frame as f32 / frame_count as f32) * TAU;

        for bone in 0..bone_count {
            let (phase, amplitude) = if (bone as usize) < params.len() {
                params[bone as usize]
            } else {
                (0.0, 0.3)
            };

            let angle = (t + phase).sin() * amplitude;

            // Quaternion for Z rotation
            let half_angle = angle * 0.5;
            let qx = 0.0f32;
            let qy = 0.0f32;
            let qz = half_angle.sin();
            let qw = half_angle.cos();

            // Position: bone offset along Y axis
            let py = bone as f32 * 1.5;

            let keyframe = encode_bone_transform([qx, qy, qz, qw], [0.0, py, 0.0], [1.0, 1.0, 1.0]);
            file.write_all(&keyframe.to_bytes()).expect("Failed to write keyframe");
        }
    }

    println!("Generated {} ({} bones, {} frames)", path.display(), bone_count, frame_count);
}

/// Generate horizontal arm animation (Y-axis rotations)
fn generate_horizontal_animation(path: &PathBuf, bone_count: u8, frame_count: u16) {
    let header = EmberZAnimationHeader::new(bone_count, frame_count);

    let mut file = File::create(path).expect("Failed to create animation file");
    file.write_all(&header.to_bytes()).expect("Failed to write header");

    for frame in 0..frame_count {
        let t = (frame as f32 / frame_count as f32) * TAU;

        for bone in 0..bone_count {
            let phase = bone as f32 * 0.3;
            let amplitude = 0.3 + (bone as f32 * 0.1);
            let angle = (t + phase).sin() * amplitude;

            // Quaternion for Y rotation
            let half_angle = angle * 0.5;
            let qx = 0.0f32;
            let qy = half_angle.sin();
            let qz = 0.0f32;
            let qw = half_angle.cos();

            // Position: bone offset along X axis
            let px = bone as f32 * 1.0;

            let keyframe = encode_bone_transform([qx, qy, qz, qw], [px, 0.0, 0.0], [1.0, 1.0, 1.0]);
            file.write_all(&keyframe.to_bytes()).expect("Failed to write keyframe");
        }
    }

    println!("Generated {} ({} bones, {} frames)", path.display(), bone_count, frame_count);
}
