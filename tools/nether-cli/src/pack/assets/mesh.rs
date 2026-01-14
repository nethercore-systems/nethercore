//! Mesh loading from various formats.

use anyhow::{Context, Result};
use zx_common::{vertex_stride_packed, NetherZXMeshHeader, PackedMesh};

/// Load a mesh from file
///
/// Supports:
/// - .nczxmesh / .nczmesh (Nethercore ZX mesh format) - direct load
/// - .obj (Wavefront OBJ) - auto-converted via nether-export
/// - .gltf / .glb (glTF 2.0) - auto-converted via nether-export
pub fn load_mesh(id: &str, path: &std::path::Path) -> Result<PackedMesh> {
    // Detect format by extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Native format - direct load
        "nczmesh" | "nczxmesh" => load_mesh_native(id, path),

        // OBJ - convert via nether-export
        "obj" => {
            let converted = nether_export::convert_obj_to_memory(path)
                .with_context(|| format!("Failed to convert OBJ: {}", path.display()))?;

            Ok(PackedMesh {
                id: id.to_string(),
                format: converted.format,
                vertex_count: converted.vertex_count,
                index_count: converted.index_count,
                vertex_data: converted.vertex_data,
                index_data: converted.indices,
            })
        }

        // glTF/GLB - convert via nether-export
        "gltf" | "glb" => {
            let converted = nether_export::convert_gltf_to_memory(path)
                .with_context(|| format!("Failed to convert glTF: {}", path.display()))?;

            Ok(PackedMesh {
                id: id.to_string(),
                format: converted.format,
                vertex_count: converted.vertex_count,
                index_count: converted.index_count,
                vertex_data: converted.vertex_data,
                index_data: converted.indices,
            })
        }

        _ => anyhow::bail!(
            "Unsupported mesh format '{}': {} (use .nczmesh, .obj, .gltf, or .glb)",
            ext,
            path.display()
        ),
    }
}

/// Load a native .nczmesh/.nczxmesh file
fn load_mesh_native(id: &str, path: &std::path::Path) -> Result<PackedMesh> {
    let data =
        std::fs::read(path).with_context(|| format!("Failed to load mesh: {}", path.display()))?;

    // Parse NetherZXMesh header
    let header = NetherZXMeshHeader::from_bytes(&data)
        .context("Failed to parse mesh header - file may be corrupted or wrong format")?;

    // Validate header
    if header.vertex_count == 0 {
        anyhow::bail!("Mesh has no vertices: {}", path.display());
    }
    if header.format > 15 {
        anyhow::bail!("Invalid mesh format {}: {}", header.format, path.display());
    }

    // Calculate stride based on format flags (using z-common)
    let stride = vertex_stride_packed(header.format) as usize;
    let vertex_data_size = header.vertex_count as usize * stride;
    let index_data_size = header.index_count as usize * 2; // u16 indices

    // Validate data size
    let expected_size = NetherZXMeshHeader::SIZE + vertex_data_size + index_data_size;
    if data.len() < expected_size {
        anyhow::bail!(
            "Mesh data too small: {} bytes, expected {} (vertices: {}, indices: {}, format: {})",
            data.len(),
            expected_size,
            header.vertex_count,
            header.index_count,
            header.format
        );
    }

    // Extract vertex and index data
    let vertex_start = NetherZXMeshHeader::SIZE;
    let vertex_end = vertex_start + vertex_data_size;
    let index_end = vertex_end + index_data_size;

    let vertex_data = data[vertex_start..vertex_end].to_vec();

    // Convert index bytes to u16 values
    let index_data: Vec<u16> = data[vertex_end..index_end]
        .chunks_exact(2)
        .map(|chunk| u16::from_le_bytes([chunk[0], chunk[1]]))
        .collect();

    Ok(PackedMesh {
        id: id.to_string(),
        format: header.format,
        vertex_count: header.vertex_count,
        index_count: header.index_count,
        vertex_data,
        index_data,
    })
}
