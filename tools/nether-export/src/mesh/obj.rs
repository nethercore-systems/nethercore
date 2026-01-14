//! OBJ mesh conversion

use super::packing::{pack_vertices_skinned, parse_format_string};
use super::types::{ConvertedMesh, MAX_INDEX_VALUE};
use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter};
use std::path::Path;

use crate::formats::write_nether_mesh;
use crate::{vertex_stride_packed, FORMAT_NORMAL, FORMAT_UV};

/// Convert an OBJ file to in-memory mesh data (for direct ROM packing)
pub fn convert_obj_to_memory(input: &Path) -> Result<ConvertedMesh> {
    let (positions, uvs, normals, indices, format) = parse_obj_file(input)?;

    // Pack vertex data
    let vertex_data = pack_vertices_skinned(
        &positions,
        uvs.as_deref(),
        None, // no colors
        normals.as_deref(),
        None, // no tangents
        None, // no skinning
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

/// Convert an OBJ file to NetherMesh format
pub fn convert_obj(input: &Path, output: &Path, format_override: Option<&str>) -> Result<()> {
    let (positions, uvs, normals, indices, auto_format) = parse_obj_file(input)?;

    // Use override format if provided, otherwise use auto-detected format
    let format = if let Some(fmt_str) = format_override {
        parse_format_string(fmt_str)
    } else {
        auto_format
    };

    // Pack vertex data
    let vertex_data = pack_vertices_skinned(
        &positions,
        uvs.as_deref(),
        None, // no colors
        normals.as_deref(),
        None, // no tangents
        None, // no skinning
        format,
    );

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_nether_mesh(&mut writer, format, &vertex_data, Some(&indices))?;

    let stride = vertex_stride_packed(format);
    tracing::info!(
        "Converted OBJ mesh: {} vertices, {} indices, format={}, stride={}",
        positions.len(),
        indices.len(),
        format,
        stride
    );

    Ok(())
}

/// Parse OBJ file and return vertex data + auto-detected format
///
/// Returns: (positions, uvs, normals, indices, format)
fn parse_obj_file(
    input: &Path,
) -> Result<(
    Vec<[f32; 3]>,
    Option<Vec<[f32; 2]>>,
    Option<Vec<[f32; 3]>>,
    Vec<u16>,
    u8,
)> {
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

    let uvs = if has_uvs { Some(final_uvs) } else { None };
    let normals = if has_normals {
        Some(final_normals)
    } else {
        None
    };

    Ok((final_positions, uvs, normals, indices, format))
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
