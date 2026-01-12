//! Mesh converter (glTF/OBJ -> .nczmesh)

use anyhow::{bail, Context, Result};

/// Maximum index value for u16 indices (65535)
/// Meshes with more vertices must be split before export.
const MAX_INDEX_VALUE: u32 = u16::MAX as u32;

/// Maximum joint index for u8 storage (255)
/// Skeletons with more bones are not supported.
const MAX_JOINT_INDEX: u16 = u8::MAX as u16;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

use crate::formats::write_nether_mesh;
use crate::{
    pack_bone_weights_unorm8, pack_tangent_f32x4, vertex_stride_packed, FORMAT_COLOR,
    FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV,
};

/// Skinning data: tuple of (bone indices, bone weights)
type SkinningData<'a> = (&'a [[u8; 4]], &'a [[f32; 4]]);

/// Result of in-memory mesh conversion
pub struct ConvertedMesh {
    /// Format flags (UV, normal, etc.)
    pub format: u8,
    /// Number of vertices
    pub vertex_count: u32,
    /// Number of indices
    pub index_count: u32,
    /// Packed vertex data
    pub vertex_data: Vec<u8>,
    /// Index data (u16)
    pub indices: Vec<u16>,
}

/// Convert an OBJ file to in-memory mesh data (for direct ROM packing)
pub fn convert_obj_to_memory(input: &Path) -> Result<ConvertedMesh> {
    let file = File::open(input).with_context(|| format!("Failed to open OBJ: {:?}", input))?;
    let reader = BufReader::new(file);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut tex_coords: Vec<[f32; 2]> = Vec::new();
    let mut normals_raw: Vec<[f32; 3]> = Vec::new();

    // Final vertex data (expanded from faces)
    let mut final_positions: Vec<[f32; 3]> = Vec::new();
    let mut final_uvs: Vec<[f32; 2]> = Vec::new();
    let mut final_normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                positions.push([x, y, z]);
            }
            "vt" if parts.len() >= 3 => {
                let u: f32 = parts[1].parse().unwrap_or(0.0);
                let v: f32 = parts[2].parse().unwrap_or(0.0);
                tex_coords.push([u, v]);
            }
            "vn" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                normals_raw.push([x, y, z]);
            }
            "f" if parts.len() >= 4 => {
                // Parse face vertices (triangulate if needed)
                let face_verts: Vec<(usize, Option<usize>, Option<usize>)> = parts[1..]
                    .iter()
                    .filter_map(|v| parse_obj_vertex(v))
                    .collect();

                if face_verts.len() < 3 {
                    continue;
                }

                // Triangulate (fan triangulation for convex polygons)
                for i in 1..face_verts.len() - 1 {
                    for &idx in &[0, i, i + 1] {
                        let (vi, vti, vni) = face_verts[idx];

                        let base_idx = final_positions.len() as u16;
                        indices.push(base_idx);

                        final_positions.push(positions.get(vi).copied().unwrap_or([0.0; 3]));

                        if let Some(ti) = vti {
                            final_uvs.push(tex_coords.get(ti).copied().unwrap_or([0.0; 2]));
                        }

                        if let Some(ni) = vni {
                            final_normals
                                .push(normals_raw.get(ni).copied().unwrap_or([0.0, 1.0, 0.0]));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if final_positions.is_empty() {
        bail!("No vertices found in OBJ file");
    }

    // Validate vertex count fits in u16 index range
    if final_positions.len() > MAX_INDEX_VALUE as usize + 1 {
        bail!(
            "OBJ mesh has {} vertices, exceeds maximum {} for u16 indices. \
            Split the mesh into smaller parts.",
            final_positions.len(),
            MAX_INDEX_VALUE + 1
        );
    }

    // Determine format
    let has_uvs = !final_uvs.is_empty() && final_uvs.len() == final_positions.len();
    let has_normals = !final_normals.is_empty() && final_normals.len() == final_positions.len();

    let mut format = 0u8;
    if has_uvs {
        format |= FORMAT_UV;
    }
    if has_normals {
        format |= FORMAT_NORMAL;
    }

    // Pack vertex data
    let uvs = if has_uvs {
        Some(final_uvs.as_slice())
    } else {
        None
    };
    let normals = if has_normals {
        Some(final_normals.as_slice())
    } else {
        None
    };
    let vertex_data = pack_vertices(&final_positions, uvs, normals, format);

    Ok(ConvertedMesh {
        format,
        vertex_count: final_positions.len() as u32,
        index_count: indices.len() as u32,
        vertex_data,
        indices,
    })
}

/// Convert a glTF/GLB file to in-memory mesh data (for direct ROM packing)
///
/// Automatically detects and includes skinning data (bone indices + weights)
/// when present in the glTF file.
pub fn convert_gltf_to_memory(input: &Path) -> Result<ConvertedMesh> {
    let (document, buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    // Get the first mesh
    let mesh = document
        .meshes()
        .next()
        .context("No meshes found in glTF")?;
    let primitive = mesh
        .primitives()
        .next()
        .context("No primitives found in mesh")?;

    // Extract vertex data
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    // Positions (required)
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .context("No positions in mesh")?
        .collect();

    // UVs (optional)
    let uvs: Option<Vec<[f32; 2]>> = reader
        .read_tex_coords(0)
        .map(|iter| iter.into_f32().collect());

    // Normals (optional)
    let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(|iter| iter.collect());

    // Tangents (optional) - vec4: xyz=tangent direction, w=handedness sign (+1 or -1)
    // Requires normals to be present (tangent without normal is invalid)
    let tangents: Option<Vec<[f32; 4]>> = if normals.is_some() {
        reader.read_tangents().map(|iter| iter.collect())
    } else {
        None
    };

    // Colors (optional) - COLOR_0 as RGBA
    let colors: Option<Vec<[f32; 4]>> = reader
        .read_colors(0)
        .map(|iter| iter.into_rgba_f32().collect());

    // Skinning data (optional) - JOINTS_0 and WEIGHTS_0
    let joints: Option<Vec<[u8; 4]>> = if let Some(iter) = reader.read_joints(0) {
        let u16_joints: Vec<[u16; 4]> = iter.into_u16().collect();

        // Validate joint indices fit in u8 (max 256 bones)
        for (vertex_idx, joint_set) in u16_joints.iter().enumerate() {
            for (component, &joint_idx) in joint_set.iter().enumerate() {
                if joint_idx > MAX_JOINT_INDEX {
                    bail!(
                        "Joint index {} at vertex {} component {} exceeds maximum {} for u8 storage. \
                        Reduce skeleton bone count to ≤256.",
                        joint_idx, vertex_idx, component, MAX_JOINT_INDEX
                    );
                }
            }
        }

        Some(
            u16_joints
                .into_iter()
                .map(|j| [j[0] as u8, j[1] as u8, j[2] as u8, j[3] as u8])
                .collect(),
        )
    } else {
        None
    };
    let weights: Option<Vec<[f32; 4]>> =
        reader.read_weights(0).map(|iter| iter.into_f32().collect());

    // Validate skinning data consistency
    let skinning = match (&joints, &weights) {
        (Some(j), Some(w)) if j.len() == positions.len() && w.len() == positions.len() => {
            Some((j.as_slice(), w.as_slice()))
        }
        (Some(_), None) | (None, Some(_)) => {
            tracing::warn!(
                "Mesh has partial skinning data (joints or weights missing), ignoring skinning"
            );
            None
        }
        _ => None,
    };

    // Indices (optional) - validate before truncating to u16
    let indices: Vec<u16> = if let Some(iter) = reader.read_indices() {
        let u32_indices: Vec<u32> = iter.into_u32().collect();

        // Validate all indices fit in u16
        if let Some((idx, &value)) = u32_indices
            .iter()
            .enumerate()
            .find(|(_, &v)| v > MAX_INDEX_VALUE)
        {
            bail!(
                "Index {} at position {} exceeds maximum {} for u16 indices. \
                The mesh has too many vertices (>65536). Split the mesh into smaller parts.",
                value,
                idx,
                MAX_INDEX_VALUE
            );
        }

        u32_indices.into_iter().map(|i| i as u16).collect()
    } else {
        Vec::new()
    };

    // Validate tangent data consistency
    let tangents = match tangents {
        Some(ref t) if t.len() == positions.len() => tangents,
        Some(_) => {
            tracing::warn!(
                "Mesh has mismatched tangent count ({} vs {} vertices), ignoring tangents",
                tangents.as_ref().map(|t| t.len()).unwrap_or(0),
                positions.len()
            );
            None
        }
        None => None,
    };

    // Determine format
    let mut format = 0u8;
    if uvs.is_some() {
        format |= FORMAT_UV;
    }
    if colors.is_some() {
        format |= FORMAT_COLOR;
    }
    if normals.is_some() {
        format |= FORMAT_NORMAL;
    }
    if skinning.is_some() {
        format |= FORMAT_SKINNED;
    }
    if tangents.is_some() {
        format |= FORMAT_TANGENT;
    }

    // Pack vertex data
    let vertex_data = pack_vertices_skinned(
        &positions,
        uvs.as_deref(),
        colors.as_deref(),
        normals.as_deref(),
        tangents.as_deref(),
        skinning,
        format,
    );

    Ok(ConvertedMesh {
        format,
        vertex_count: positions.len() as u32,
        index_count: indices.len() as u32,
        vertex_data,
        indices,
    })
}

/// Convert a glTF/GLB file to NetherMesh format
pub fn convert_gltf(input: &Path, output: &Path, format_override: Option<&str>) -> Result<()> {
    let (document, buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    // Get the first mesh
    let mesh = document
        .meshes()
        .next()
        .context("No meshes found in glTF")?;
    let primitive = mesh
        .primitives()
        .next()
        .context("No primitives found in mesh")?;

    // Extract vertex data
    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

    // Positions (required)
    let positions: Vec<[f32; 3]> = reader
        .read_positions()
        .context("No positions in mesh")?
        .collect();

    // UVs (optional)
    let uvs: Option<Vec<[f32; 2]>> = reader
        .read_tex_coords(0)
        .map(|iter| iter.into_f32().collect());

    // Normals (optional)
    let normals: Option<Vec<[f32; 3]>> = reader.read_normals().map(|iter| iter.collect());

    // Tangents (optional) - requires normals
    let tangents: Option<Vec<[f32; 4]>> = if normals.is_some() {
        reader.read_tangents().map(|iter| iter.collect())
    } else {
        None
    };

    // Validate tangent count matches vertex count
    let tangents = match tangents {
        Some(ref t) if t.len() == positions.len() => tangents,
        Some(_) => None,
        None => None,
    };

    // Determine format
    let format = if let Some(fmt_str) = format_override {
        parse_format_string(fmt_str)
    } else {
        // Auto-detect from available attributes
        let mut fmt = 0u8;
        if uvs.is_some() {
            fmt |= FORMAT_UV;
        }
        if normals.is_some() {
            fmt |= FORMAT_NORMAL;
        }
        if tangents.is_some() {
            fmt |= FORMAT_TANGENT;
        }
        fmt
    };

    // Indices (optional) - validate before truncating to u16
    let indices: Option<Vec<u16>> = if let Some(iter) = reader.read_indices() {
        let u32_indices: Vec<u32> = iter.into_u32().collect();

        // Validate all indices fit in u16
        if let Some((idx, &value)) = u32_indices
            .iter()
            .enumerate()
            .find(|(_, &v)| v > MAX_INDEX_VALUE)
        {
            bail!(
                "Index {} at position {} exceeds maximum {} for u16 indices. \
                The mesh has too many vertices (>65536). Split the mesh into smaller parts.",
                value,
                idx,
                MAX_INDEX_VALUE
            );
        }

        Some(u32_indices.into_iter().map(|i| i as u16).collect())
    } else {
        None
    };

    // Pack vertex data (use pack_vertices_skinned for tangent support)
    let vertex_data = pack_vertices_skinned(
        &positions,
        uvs.as_deref(),
        None, // no colors
        normals.as_deref(),
        tangents.as_deref(),
        None, // no skinning
        format,
    );

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_nether_mesh(&mut writer, format, &vertex_data, indices.as_deref())?;

    let stride = vertex_stride_packed(format);
    tracing::info!(
        "Converted mesh: {} vertices, {} indices, format={}, stride={}",
        positions.len(),
        indices.as_ref().map(|i| i.len()).unwrap_or(0),
        format,
        stride
    );

    Ok(())
}

fn parse_format_string(s: &str) -> u8 {
    let s = s.to_uppercase();
    let mut format = 0u8;
    if s.contains("UV") {
        format |= FORMAT_UV;
    }
    if s.contains("COLOR") {
        format |= crate::FORMAT_COLOR;
    }
    if s.contains("NORMAL") {
        format |= FORMAT_NORMAL;
    }
    if s.contains("SKINNED") {
        format |= crate::FORMAT_SKINNED;
    }
    if s.contains("TANGENT") {
        format |= FORMAT_TANGENT;
    }
    format
}

fn pack_vertices(
    positions: &[[f32; 3]],
    uvs: Option<&[[f32; 2]]>,
    normals: Option<&[[f32; 3]]>,
    format: u8,
) -> Vec<u8> {
    // Delegate to the skinned version with no skinning data, colors, or tangents
    pack_vertices_skinned(positions, uvs, None, normals, None, None, format)
}

/// Pack vertices with optional color, tangent, and skinning support
///
/// Vertex layout (in order): Position → UV → Color → Normal → Tangent → Skinning
/// - Color adds 4 bytes per vertex (unorm8 × 4)
/// - Tangent adds 4 bytes per vertex (octahedral u32 with sign bit)
/// - Skinning adds 8 bytes per vertex (bone indices u8×4 + weights unorm8×4)
fn pack_vertices_skinned(
    positions: &[[f32; 3]],
    uvs: Option<&[[f32; 2]]>,
    colors: Option<&[[f32; 4]]>,
    normals: Option<&[[f32; 3]]>,
    tangents: Option<&[[f32; 4]]>,
    skinning: Option<SkinningData>,
    format: u8,
) -> Vec<u8> {
    use crate::{
        pack_color_rgba_unorm8, pack_normal_octahedral, pack_position_f16, pack_uv_unorm16,
    };
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

/// Convert an OBJ file to NetherMesh format
pub fn convert_obj(input: &Path, output: &Path, format_override: Option<&str>) -> Result<()> {
    let file = File::open(input).with_context(|| format!("Failed to open OBJ: {:?}", input))?;
    let reader = BufReader::new(file);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut tex_coords: Vec<[f32; 2]> = Vec::new();
    let mut normals_raw: Vec<[f32; 3]> = Vec::new();

    // Final vertex data (expanded from faces)
    let mut final_positions: Vec<[f32; 3]> = Vec::new();
    let mut final_uvs: Vec<[f32; 2]> = Vec::new();
    let mut final_normals: Vec<[f32; 3]> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "v" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                positions.push([x, y, z]);
            }
            "vt" if parts.len() >= 3 => {
                let u: f32 = parts[1].parse().unwrap_or(0.0);
                let v: f32 = parts[2].parse().unwrap_or(0.0);
                tex_coords.push([u, v]);
            }
            "vn" if parts.len() >= 4 => {
                let x: f32 = parts[1].parse().unwrap_or(0.0);
                let y: f32 = parts[2].parse().unwrap_or(0.0);
                let z: f32 = parts[3].parse().unwrap_or(0.0);
                normals_raw.push([x, y, z]);
            }
            "f" if parts.len() >= 4 => {
                // Parse face vertices (triangulate if needed)
                let face_verts: Vec<(usize, Option<usize>, Option<usize>)> = parts[1..]
                    .iter()
                    .filter_map(|v| parse_obj_vertex(v))
                    .collect();

                if face_verts.len() < 3 {
                    continue;
                }

                // Triangulate (fan triangulation for convex polygons)
                for i in 1..face_verts.len() - 1 {
                    for &idx in &[0, i, i + 1] {
                        let (vi, vti, vni) = face_verts[idx];

                        let base_idx = final_positions.len() as u16;
                        indices.push(base_idx);

                        final_positions.push(positions.get(vi).copied().unwrap_or([0.0; 3]));

                        if let Some(ti) = vti {
                            final_uvs.push(tex_coords.get(ti).copied().unwrap_or([0.0; 2]));
                        }

                        if let Some(ni) = vni {
                            final_normals
                                .push(normals_raw.get(ni).copied().unwrap_or([0.0, 1.0, 0.0]));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if final_positions.is_empty() {
        bail!("No vertices found in OBJ file");
    }

    // Validate vertex count fits in u16 index range
    if final_positions.len() > MAX_INDEX_VALUE as usize + 1 {
        bail!(
            "OBJ mesh has {} vertices, exceeds maximum {} for u16 indices. \
            Split the mesh into smaller parts.",
            final_positions.len(),
            MAX_INDEX_VALUE + 1
        );
    }

    // Determine format
    let has_uvs = !final_uvs.is_empty() && final_uvs.len() == final_positions.len();
    let has_normals = !final_normals.is_empty() && final_normals.len() == final_positions.len();

    let format = if let Some(fmt_str) = format_override {
        parse_format_string(fmt_str)
    } else {
        let mut fmt = 0u8;
        if has_uvs {
            fmt |= FORMAT_UV;
        }
        if has_normals {
            fmt |= FORMAT_NORMAL;
        }
        fmt
    };

    // Pack vertex data
    let uvs = if has_uvs {
        Some(final_uvs.as_slice())
    } else {
        None
    };
    let normals = if has_normals {
        Some(final_normals.as_slice())
    } else {
        None
    };
    let vertex_data = pack_vertices(&final_positions, uvs, normals, format);

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_nether_mesh(&mut writer, format, &vertex_data, Some(&indices))?;

    let stride = vertex_stride_packed(format);
    tracing::info!(
        "Converted OBJ mesh: {} vertices, {} indices, format={}, stride={}",
        final_positions.len(),
        indices.len(),
        format,
        stride
    );

    Ok(())
}

/// Parse OBJ vertex reference: "v", "v/vt", "v/vt/vn", or "v//vn"
fn parse_obj_vertex(s: &str) -> Option<(usize, Option<usize>, Option<usize>)> {
    let parts: Vec<&str> = s.split('/').collect();

    let vi = parts.first()?.parse::<usize>().ok()?.checked_sub(1)?; // OBJ indices are 1-based

    let vti = parts
        .get(1)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(|i| i.checked_sub(1));

    let vni = parts
        .get(2)
        .filter(|s| !s.is_empty())
        .and_then(|s| s.parse::<usize>().ok())
        .and_then(|i| i.checked_sub(1));

    Some((vi, vti, vni))
}
