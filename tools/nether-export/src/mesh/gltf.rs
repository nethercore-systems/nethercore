//! glTF/GLB mesh conversion

use super::packing::{pack_vertices_skinned, parse_format_string};
use super::types::{ConvertedMesh, SkinningData, MAX_INDEX_VALUE, MAX_JOINT_INDEX};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::formats::write_nether_mesh;
use crate::{
    vertex_stride_packed, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_TANGENT, FORMAT_UV,
};

/// Convert a glTF/GLB file to in-memory mesh data (for direct ROM packing)
///
/// Automatically detects and includes skinning data (bone indices + weights)
/// when present in the glTF file.
pub fn convert_gltf_to_memory(input: &Path) -> Result<ConvertedMesh> {
    let (positions, uvs, colors, normals, tangents, skinning, indices, format) =
        parse_gltf_file(input)?;

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
    let (positions, uvs, colors, normals, tangents, skinning, indices, auto_format) =
        parse_gltf_file(input)?;

    // Use override format if provided, otherwise use auto-detected format
    let format = if let Some(fmt_str) = format_override {
        parse_format_string(fmt_str)
    } else {
        auto_format
    };

    // Pack vertex data (note: format override may exclude some attributes from export)
    let vertex_data = pack_vertices_skinned(
        &positions,
        uvs.as_deref(),
        colors.as_deref(),
        normals.as_deref(),
        tangents.as_deref(),
        skinning,
        format,
    );

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_nether_mesh(&mut writer, format, &vertex_data, Some(&indices))?;

    let stride = vertex_stride_packed(format);
    tracing::info!(
        "Converted mesh: {} vertices, {} indices, format={}, stride={}",
        positions.len(),
        indices.len(),
        format,
        stride
    );

    Ok(())
}

/// Parse glTF file and extract vertex data + auto-detected format
///
/// Returns: (positions, uvs, colors, normals, tangents, skinning, indices, format)
#[allow(clippy::type_complexity)]
fn parse_gltf_file(
    input: &Path,
) -> Result<(
    Vec<[f32; 3]>,
    Option<Vec<[f32; 2]>>,
    Option<Vec<[f32; 4]>>,
    Option<Vec<[f32; 3]>>,
    Option<Vec<[f32; 4]>>,
    Option<SkinningData<'static>>,
    Vec<u16>,
    u8,
)> {
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
    let skinning: Option<SkinningData<'static>> = match (&joints, &weights) {
        (Some(j), Some(w)) if j.len() == positions.len() && w.len() == positions.len() => {
            // Box and leak to get 'static lifetime (safe for conversion tool)
            let joints_static: &'static [[u8; 4]] = Box::leak(j.clone().into_boxed_slice());
            let weights_static: &'static [[f32; 4]] = Box::leak(w.clone().into_boxed_slice());
            Some((joints_static, weights_static))
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

    Ok((
        positions, uvs, colors, normals, tangents, skinning, indices, format,
    ))
}
