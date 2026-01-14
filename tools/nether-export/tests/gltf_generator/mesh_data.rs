//! Mesh, skeleton, and animation data structures and creation.

use std::f32::consts::TAU;

/// Bone count for the test skeleton
pub const BONE_COUNT: usize = 3;
/// Frame count for the test animation
pub const FRAME_COUNT: usize = 30;
/// Segment height between bones
pub(crate) const SEGMENT_HEIGHT: f32 = 1.0;

/// Mesh data for the test asset
pub(crate) struct MeshData {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub joints: Vec<[u8; 4]>,
    pub weights: Vec<[f32; 4]>,
    pub indices: Vec<u16>,
}

/// Skeleton data
pub(crate) struct SkeletonData {
    pub inverse_bind_matrices: Vec<[[f32; 4]; 4]>,
}

/// Animation data
pub(crate) struct AnimationData {
    pub times: Vec<f32>,
    pub translations: Vec<Vec<[f32; 3]>>,
    pub rotations: Vec<Vec<[f32; 4]>>,
    pub scales: Vec<Vec<[f32; 3]>>,
}

/// Create mesh data: 3 stacked boxes with skinning
pub(crate) fn create_mesh_data() -> MeshData {
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
pub(crate) fn create_skeleton() -> SkeletonData {
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
pub(crate) fn create_animation() -> AnimationData {
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

// Helper functions

pub(crate) fn mat4_identity() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}

pub(crate) fn mat4_translate(x: f32, y: f32, z: f32) -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [x, y, z, 1.0],
    ]
}

pub(crate) fn compute_bounds(positions: &[[f32; 3]]) -> (Vec<f32>, Vec<f32>) {
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
