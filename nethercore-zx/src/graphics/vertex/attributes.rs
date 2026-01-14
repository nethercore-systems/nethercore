//! wgpu-specific vertex attribute definitions
//!
//! This module contains the pre-computed vertex attribute arrays for all 32 vertex formats.

/// Attribute sizes in bytes for offset calculation (packed formats - GPU only)
const SIZE_POS: u64 = 8; // Float16x4 (padded for alignment)
const SIZE_UV: u64 = 4; // Unorm16x2
const SIZE_COLOR: u64 = 4; // Unorm8x4
const SIZE_NORMAL: u64 = 4; // Octahedral u32
const SIZE_TANGENT: u64 = 4; // Octahedral u32 with sign bit
const SIZE_BONE_INDICES: u64 = 4; // Uint8x4
// Note: SIZE_BONE_WEIGHTS not needed - bone weights is always the last attribute
// so its size never appears in offset calculations

/// Shader locations for each attribute type
const LOC_POS: u32 = 0;
const LOC_UV: u32 = 1;
const LOC_COLOR: u32 = 2;
const LOC_NORMAL: u32 = 3;
const LOC_BONE_INDICES: u32 = 4;
const LOC_BONE_WEIGHTS: u32 = 5;
const LOC_TANGENT: u32 = 6;

/// Creates a position attribute at offset 0 (Float16x4, padded)
const fn attr_pos() -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Float16x4,
        offset: 0,
        shader_location: LOC_POS,
    }
}

/// Creates a UV attribute at the given offset (Unorm16x2)
const fn attr_uv(offset: u64) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Unorm16x2,
        offset,
        shader_location: LOC_UV,
    }
}

/// Creates a color attribute at the given offset (Unorm8x4)
const fn attr_color(offset: u64) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Unorm8x4,
        offset,
        shader_location: LOC_COLOR,
    }
}

/// Creates a normal attribute at the given offset (Uint32 - octahedral encoded)
const fn attr_normal(offset: u64) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset,
        shader_location: LOC_NORMAL,
    }
}

/// Creates a tangent attribute at the given offset (Uint32 - octahedral with sign bit)
const fn attr_tangent(offset: u64) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint32,
        offset,
        shader_location: LOC_TANGENT,
    }
}

/// Creates bone indices attribute at the given offset (Uint8x4)
const fn attr_bone_indices(offset: u64) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Uint8x4,
        offset,
        shader_location: LOC_BONE_INDICES,
    }
}

/// Creates bone weights attribute at the given offset (Unorm8x4)
const fn attr_bone_weights(offset: u64) -> wgpu::VertexAttribute {
    wgpu::VertexAttribute {
        format: wgpu::VertexFormat::Unorm8x4,
        offset,
        shader_location: LOC_BONE_WEIGHTS,
    }
}

/// Pre-computed vertex attribute arrays for all 32 formats.
///
/// Vertex layout order: Position → UV → Color → Normal → Tangent → Bone Indices → Bone Weights
/// Each attribute is only present if its corresponding flag is set.
/// Offsets are computed based on which attributes precede each one.
///
/// Note: Formats 16-19 and 24-27 have tangent but no normal - these are invalid
/// but still defined to avoid runtime panics. They should never be used.
pub static VERTEX_ATTRIBUTES: [&[wgpu::VertexAttribute]; 32] = [
    // ============================================================================
    // Formats 0-15: Without tangent (same as before)
    // ============================================================================
    // Format 0: POS
    &[attr_pos()],
    // Format 1: POS_UV
    &[attr_pos(), attr_uv(SIZE_POS)],
    // Format 2: POS_COLOR
    &[attr_pos(), attr_color(SIZE_POS)],
    // Format 3: POS_UV_COLOR
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
    ],
    // Format 4: POS_NORMAL
    &[attr_pos(), attr_normal(SIZE_POS)],
    // Format 5: POS_UV_NORMAL
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_UV),
    ],
    // Format 6: POS_COLOR_NORMAL
    &[
        attr_pos(),
        attr_color(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_COLOR),
    ],
    // Format 7: POS_UV_COLOR_NORMAL
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
        attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
    ],
    // Format 8: POS_SKINNED
    &[
        attr_pos(),
        attr_bone_indices(SIZE_POS),
        attr_bone_weights(SIZE_POS + SIZE_BONE_INDICES),
    ],
    // Format 9: POS_UV_SKINNED
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_bone_indices(SIZE_POS + SIZE_UV),
        attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_BONE_INDICES),
    ],
    // Format 10: POS_COLOR_SKINNED
    &[
        attr_pos(),
        attr_color(SIZE_POS),
        attr_bone_indices(SIZE_POS + SIZE_COLOR),
        attr_bone_weights(SIZE_POS + SIZE_COLOR + SIZE_BONE_INDICES),
    ],
    // Format 11: POS_UV_COLOR_SKINNED
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
        attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR),
        attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_BONE_INDICES),
    ],
    // Format 12: POS_NORMAL_SKINNED
    &[
        attr_pos(),
        attr_normal(SIZE_POS),
        attr_bone_indices(SIZE_POS + SIZE_NORMAL),
        attr_bone_weights(SIZE_POS + SIZE_NORMAL + SIZE_BONE_INDICES),
    ],
    // Format 13: POS_UV_NORMAL_SKINNED
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_UV),
        attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_NORMAL),
        attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_NORMAL + SIZE_BONE_INDICES),
    ],
    // Format 14: POS_COLOR_NORMAL_SKINNED
    &[
        attr_pos(),
        attr_color(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_COLOR),
        attr_bone_indices(SIZE_POS + SIZE_COLOR + SIZE_NORMAL),
        attr_bone_weights(SIZE_POS + SIZE_COLOR + SIZE_NORMAL + SIZE_BONE_INDICES),
    ],
    // Format 15: POS_UV_COLOR_NORMAL_SKINNED
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
        attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
        attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL),
        attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL + SIZE_BONE_INDICES),
    ],
    // ============================================================================
    // Formats 16-31: With tangent flag (bit 4)
    // ============================================================================
    // Format 16: POS_TANGENT (INVALID - tangent requires normal)
    &[attr_pos(), attr_tangent(SIZE_POS)],
    // Format 17: POS_UV_TANGENT (INVALID)
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_tangent(SIZE_POS + SIZE_UV),
    ],
    // Format 18: POS_COLOR_TANGENT (INVALID)
    &[
        attr_pos(),
        attr_color(SIZE_POS),
        attr_tangent(SIZE_POS + SIZE_COLOR),
    ],
    // Format 19: POS_UV_COLOR_TANGENT (INVALID)
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
        attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR),
    ],
    // Format 20: POS_NORMAL_TANGENT
    &[
        attr_pos(),
        attr_normal(SIZE_POS),
        attr_tangent(SIZE_POS + SIZE_NORMAL),
    ],
    // Format 21: POS_UV_NORMAL_TANGENT
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_UV),
        attr_tangent(SIZE_POS + SIZE_UV + SIZE_NORMAL),
    ],
    // Format 22: POS_COLOR_NORMAL_TANGENT
    &[
        attr_pos(),
        attr_color(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_COLOR),
        attr_tangent(SIZE_POS + SIZE_COLOR + SIZE_NORMAL),
    ],
    // Format 23: POS_UV_COLOR_NORMAL_TANGENT
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
        attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
        attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL),
    ],
    // Format 24: POS_TANGENT_SKINNED (INVALID)
    &[
        attr_pos(),
        attr_tangent(SIZE_POS),
        attr_bone_indices(SIZE_POS + SIZE_TANGENT),
        attr_bone_weights(SIZE_POS + SIZE_TANGENT + SIZE_BONE_INDICES),
    ],
    // Format 25: POS_UV_TANGENT_SKINNED (INVALID)
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_tangent(SIZE_POS + SIZE_UV),
        attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_TANGENT),
        attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_TANGENT + SIZE_BONE_INDICES),
    ],
    // Format 26: POS_COLOR_TANGENT_SKINNED (INVALID)
    &[
        attr_pos(),
        attr_color(SIZE_POS),
        attr_tangent(SIZE_POS + SIZE_COLOR),
        attr_bone_indices(SIZE_POS + SIZE_COLOR + SIZE_TANGENT),
        attr_bone_weights(SIZE_POS + SIZE_COLOR + SIZE_TANGENT + SIZE_BONE_INDICES),
    ],
    // Format 27: POS_UV_COLOR_TANGENT_SKINNED (INVALID)
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
        attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR),
        attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_TANGENT),
        attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_TANGENT + SIZE_BONE_INDICES),
    ],
    // Format 28: POS_NORMAL_TANGENT_SKINNED
    &[
        attr_pos(),
        attr_normal(SIZE_POS),
        attr_tangent(SIZE_POS + SIZE_NORMAL),
        attr_bone_indices(SIZE_POS + SIZE_NORMAL + SIZE_TANGENT),
        attr_bone_weights(SIZE_POS + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES),
    ],
    // Format 29: POS_UV_NORMAL_TANGENT_SKINNED
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_UV),
        attr_tangent(SIZE_POS + SIZE_UV + SIZE_NORMAL),
        attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_NORMAL + SIZE_TANGENT),
        attr_bone_weights(SIZE_POS + SIZE_UV + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES),
    ],
    // Format 30: POS_COLOR_NORMAL_TANGENT_SKINNED
    &[
        attr_pos(),
        attr_color(SIZE_POS),
        attr_normal(SIZE_POS + SIZE_COLOR),
        attr_tangent(SIZE_POS + SIZE_COLOR + SIZE_NORMAL),
        attr_bone_indices(SIZE_POS + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT),
        attr_bone_weights(
            SIZE_POS + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES,
        ),
    ],
    // Format 31: POS_UV_COLOR_NORMAL_TANGENT_SKINNED
    &[
        attr_pos(),
        attr_uv(SIZE_POS),
        attr_color(SIZE_POS + SIZE_UV),
        attr_normal(SIZE_POS + SIZE_UV + SIZE_COLOR),
        attr_tangent(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL),
        attr_bone_indices(SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT),
        attr_bone_weights(
            SIZE_POS + SIZE_UV + SIZE_COLOR + SIZE_NORMAL + SIZE_TANGENT + SIZE_BONE_INDICES,
        ),
    ],
];
