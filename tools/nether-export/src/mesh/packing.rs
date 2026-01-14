//! Vertex packing utilities

use super::types::SkinningData;
use crate::{
    pack_bone_weights_unorm8, pack_color_rgba_unorm8, pack_normal_octahedral, pack_position_f16,
    pack_tangent_f32x4, pack_uv_unorm16, vertex_stride_packed, FORMAT_COLOR, FORMAT_NORMAL,
    FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV,
};

/// Pack vertices with optional color, tangent, and skinning support
///
/// Vertex layout (in order): Position → UV → Color → Normal → Tangent → Skinning
/// - Color adds 4 bytes per vertex (unorm8 × 4)
/// - Tangent adds 4 bytes per vertex (octahedral u32 with sign bit)
/// - Skinning adds 8 bytes per vertex (bone indices u8×4 + weights unorm8×4)
pub(crate) fn pack_vertices_skinned(
    positions: &[[f32; 3]],
    uvs: Option<&[[f32; 2]]>,
    colors: Option<&[[f32; 4]]>,
    normals: Option<&[[f32; 3]]>,
    tangents: Option<&[[f32; 4]]>,
    skinning: Option<SkinningData>,
    format: u8,
) -> Vec<u8> {
    use bytemuck::cast_slice;

    let has_uv = format & FORMAT_UV != 0;
    let has_color = format & FORMAT_COLOR != 0;
    let has_normal = format & FORMAT_NORMAL != 0;
    let has_tangent = format & FORMAT_TANGENT != 0;
    let has_skinning = format & FORMAT_SKINNED != 0;

    let stride = vertex_stride_packed(format) as usize;
    let mut data = Vec::with_capacity(positions.len() * stride);

    for i in 0..positions.len() {
        // Position (f16x4) - 8 bytes
        let pos = positions[i];
        let packed_pos = pack_position_f16(pos[0], pos[1], pos[2]);
        data.extend_from_slice(cast_slice(&packed_pos));

        // UV (unorm16x2) - 4 bytes
        if has_uv {
            let uv = uvs.map(|u| u[i]).unwrap_or([0.0, 0.0]);
            let packed_uv = pack_uv_unorm16(uv[0], uv[1]);
            data.extend_from_slice(cast_slice(&packed_uv));
        }

        // Color (unorm8x4) - 4 bytes
        if has_color {
            let c = colors.map(|c| c[i]).unwrap_or([1.0, 1.0, 1.0, 1.0]);
            let packed_color = pack_color_rgba_unorm8(c[0], c[1], c[2], c[3]);
            data.extend_from_slice(&packed_color);
        }

        // Normal (octahedral u32) - 4 bytes
        if has_normal {
            let n = normals.map(|n| n[i]).unwrap_or([0.0, 1.0, 0.0]);
            let packed_normal = pack_normal_octahedral(n[0], n[1], n[2]);
            data.extend_from_slice(&packed_normal.to_le_bytes());
        }

        // Tangent (octahedral u32 with sign bit) - 4 bytes
        if has_tangent {
            let t = tangents.map(|t| t[i]).unwrap_or([1.0, 0.0, 0.0, 1.0]);
            let packed_tangent = pack_tangent_f32x4(t);
            data.extend_from_slice(&packed_tangent.to_le_bytes());
        }

        // Skinning (bone indices + weights) - 8 bytes
        if has_skinning {
            if let Some((joints, weights)) = skinning {
                // Bone indices (u8 × 4)
                data.extend_from_slice(&joints[i]);
                // Bone weights (unorm8 × 4)
                let packed_weights = pack_bone_weights_unorm8(weights[i]);
                data.extend_from_slice(&packed_weights);
            } else {
                // No skinning data provided but format says skinned - use defaults
                data.extend_from_slice(&[0u8; 4]); // bone indices
                data.extend_from_slice(&[255, 0, 0, 0]); // full weight on bone 0
            }
        }
    }

    data
}

/// Parse format override string (e.g., "UV_NORMAL_TANGENT")
pub(crate) fn parse_format_string(s: &str) -> u8 {
    let s = s.to_uppercase();
    let mut format = 0u8;
    if s.contains("UV") {
        format |= FORMAT_UV;
    }
    if s.contains("COLOR") {
        format |= FORMAT_COLOR;
    }
    if s.contains("NORMAL") {
        format |= FORMAT_NORMAL;
    }
    if s.contains("SKINNED") {
        format |= FORMAT_SKINNED;
    }
    if s.contains("TANGENT") {
        format |= FORMAT_TANGENT;
    }
    format
}
