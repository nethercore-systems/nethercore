//! Common utilities for Emberware examples

#![no_std]



/// Maximum vertices and indices for level 3 icosphere subdivision
pub const MAX_ICOSPHERE_VERTS_L3: usize = 642 * 6; // 642 vertices × 6 floats (pos + normal)
pub const MAX_ICOSPHERE_INDICES_L3: usize = 1280 * 3; // 1280 triangles × 3 indices

// Temporary buffer for subdivision (needs to be 2x for temp storage)
const TEMP_INDICES_SIZE: usize = MAX_ICOSPHERE_INDICES_L3 * 2;

/// Base icosphere vertices (12 vertices, each with position + normal)
/// Position and normal are the same for a unit sphere
const BASE_ICOSPHERE_VERTS: [f32; 12 * 6] = {
    const N: f32 = 0.5257311; // 1 / sqrt(1 + PHI^2)
    const P: f32 = 0.8506508; // PHI / sqrt(1 + PHI^2)

    [
        // Vertex 0-3: vertices along XY plane
        -N,  P,  0.0,  -N,  P,  0.0,
         N,  P,  0.0,   N,  P,  0.0,
        -N, -P,  0.0,  -N, -P,  0.0,
         N, -P,  0.0,   N, -P,  0.0,

        // Vertex 4-7: vertices along YZ plane
         0.0, -N,  P,   0.0, -N,  P,
         0.0,  N,  P,   0.0,  N,  P,
         0.0, -N, -P,   0.0, -N, -P,
         0.0,  N, -P,   0.0,  N, -P,

        // Vertex 8-11: vertices along ZX plane
         P,  0.0, -N,   P,  0.0, -N,
         P,  0.0,  N,   P,  0.0,  N,
        -P,  0.0, -N,  -P,  0.0, -N,
        -P,  0.0,  N,  -P,  0.0,  N,
    ]
};

/// Base icosphere faces (20 triangles)
const BASE_ICOSPHERE_INDICES: [u16; 60] = [
    // 5 faces around vertex 0
    0, 11, 5,
    0, 5, 1,
    0, 1, 7,
    0, 7, 10,
    0, 10, 11,
    // 5 adjacent faces
    1, 5, 9,
    5, 11, 4,
    11, 10, 2,
    10, 7, 6,
    7, 1, 8,
    // 5 faces around vertex 3
    3, 9, 4,
    3, 4, 2,
    3, 2, 6,
    3, 6, 8,
    3, 8, 9,
    // 5 bottom faces
    4, 9, 5,
    2, 4, 11,
    6, 2, 10,
    8, 6, 7,
    9, 8, 1,
];

/// Fast inverse square root for vector normalization
#[inline]
fn fast_inv_sqrt(x: f32) -> f32 {
    let half_x = 0.5 * x;
    let i = x.to_bits();
    let i = 0x5f3759df - (i >> 1);
    let y = f32::from_bits(i);
    y * (1.5 - half_x * y * y)
}

/// Normalize a 3D vector in place
#[inline]
fn normalize(v: &mut [f32; 3]) {
    let len_sq = v[0] * v[0] + v[1] * v[1] + v[2] * v[2];
    if len_sq > 0.0 {
        let inv_len = fast_inv_sqrt(len_sq);
        v[0] *= inv_len;
        v[1] *= inv_len;
        v[2] *= inv_len;
    }
}

/// Get midpoint between two vertices, normalized to unit sphere
/// Checks for existing vertices to avoid duplicates
///
/// # Safety
/// Caller must ensure verts pointer is valid and vert_count is accurate
unsafe fn get_midpoint(
    verts: *mut f32,
    vert_count: *mut usize,
    v1_idx: u16,
    v2_idx: u16,
) -> u16 {
    let v1_base = v1_idx as usize * 6;
    let v2_base = v2_idx as usize * 6;

    // Calculate midpoint position
    let mut mid = [
        (*verts.add(v1_base) + *verts.add(v2_base)) * 0.5,
        (*verts.add(v1_base + 1) + *verts.add(v2_base + 1)) * 0.5,
        (*verts.add(v1_base + 2) + *verts.add(v2_base + 2)) * 0.5,
    ];

    // Normalize to unit sphere
    normalize(&mut mid);

    // Check if this vertex already exists (deduplication)
    const EPSILON: f32 = 0.0001;
    let current_vert_count = *vert_count;
    for i in 0..current_vert_count {
        let vx = *verts.add(i * 6);
        let vy = *verts.add(i * 6 + 1);
        let vz = *verts.add(i * 6 + 2);
        let dx = vx - mid[0];
        let dy = vy - mid[1];
        let dz = vz - mid[2];
        if dx * dx + dy * dy + dz * dz < EPSILON {
            return i as u16;
        }
    }

    // Add new vertex (only if not found)
    let new_idx = current_vert_count;
    let new_base = new_idx * 6;
    *verts.add(new_base) = mid[0];
    *verts.add(new_base + 1) = mid[1];
    *verts.add(new_base + 2) = mid[2];
    *verts.add(new_base + 3) = mid[0]; // Normal = position for unit sphere
    *verts.add(new_base + 4) = mid[1];
    *verts.add(new_base + 5) = mid[2];

    *vert_count = new_idx + 1;
    new_idx as u16
}

/// Generate a subdivided icosphere
///
/// # Arguments
/// * `level` - Subdivision level (0 = base icosahedron with 12 verts, each level ~4x more triangles)
/// * `verts` - Output vertex buffer (format: [x, y, z, nx, ny, nz, ...])
/// * `indices` - Output index buffer
/// * `vert_count` - Output: number of vertices written
/// * `index_count` - Output: number of indices written
///
/// # Buffer sizes needed
/// - Level 0: 12 verts, 60 indices
/// - Level 1: 42 verts, 240 indices
/// - Level 2: 162 verts, 960 indices
/// - Level 3: 642 verts, 3840 indices
/// # Safety
/// Caller must ensure the pointers are valid and properly aligned
pub unsafe fn generate_icosphere(
    level: usize,
    verts: *mut f32,
    _verts_len: usize,
    indices: *mut u16,
    _indices_len: usize,
    vert_count: *mut usize,
    index_count: *mut usize,
) {
    // Copy base vertices
    for i in 0..12 {
        for j in 0..6 {
            *verts.add(i * 6 + j) = BASE_ICOSPHERE_VERTS[i * 6 + j];
        }
    }
    *vert_count = 12;

    // Copy base indices
    for i in 0..60 {
        *indices.add(i) = BASE_ICOSPHERE_INDICES[i];
    }
    *index_count = 60;

    // Static temp buffer to avoid stack overflow (reused across calls)
    static mut TEMP_INDICES: [u16; TEMP_INDICES_SIZE] = [0; TEMP_INDICES_SIZE];

    // Subdivide
    for _ in 0..level {
        let old_index_count = *index_count;
        let mut new_idx_count = 0;

        for tri_idx in 0..(old_index_count / 3) {
            let base = tri_idx * 3;
            let v0 = *indices.add(base);
            let v1 = *indices.add(base + 1);
            let v2 = *indices.add(base + 2);

            // Get midpoints (creates new vertices if needed)
            let m01 = get_midpoint(verts, vert_count, v0, v1);
            let m12 = get_midpoint(verts, vert_count, v1, v2);
            let m20 = get_midpoint(verts, vert_count, v2, v0);

            // Write 4 new triangles to temp buffer
            TEMP_INDICES[new_idx_count] = v0;
            TEMP_INDICES[new_idx_count + 1] = m01;
            TEMP_INDICES[new_idx_count + 2] = m20;

            TEMP_INDICES[new_idx_count + 3] = v1;
            TEMP_INDICES[new_idx_count + 4] = m12;
            TEMP_INDICES[new_idx_count + 5] = m01;

            TEMP_INDICES[new_idx_count + 6] = v2;
            TEMP_INDICES[new_idx_count + 7] = m20;
            TEMP_INDICES[new_idx_count + 8] = m12;

            TEMP_INDICES[new_idx_count + 9] = m01;
            TEMP_INDICES[new_idx_count + 10] = m12;
            TEMP_INDICES[new_idx_count + 11] = m20;

            new_idx_count += 12;
        }

        // Copy from temp buffer back to indices
        for i in 0..new_idx_count {
            *indices.add(i) = TEMP_INDICES[i];
        }
        *index_count = new_idx_count;
    }
}
