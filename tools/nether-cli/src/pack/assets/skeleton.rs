//! Skeleton loading for skeletal animation.

use anyhow::{Context, Result};
use nethercore_shared::math::BoneMatrix3x4;
use zx_common::{NetherZXSkeletonHeader, PackedSkeleton, INVERSE_BIND_MATRIX_SIZE};

/// Load a skeleton from file
///
/// Supports:
/// - .nczxskel (Nethercore skeleton format) - direct load
/// - .gltf / .glb (glTF 2.0) - auto-converted via nether-export
pub fn load_skeleton(
    id: &str,
    path: &std::path::Path,
    skin_name: Option<&str>,
) -> Result<PackedSkeleton> {
    // Detect format by extension
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        // Native format - direct load
        "nczxskel" => load_skeleton_native(id, path),

        // glTF/GLB - convert via nether-export
        "gltf" | "glb" => {
            let converted = nether_export::convert_gltf_skeleton_to_memory(path, skin_name)
                .with_context(|| format!("Failed to convert glTF skeleton: {}", path.display()))?;

            // Convert [f32; 12] column-major to BoneMatrix3x4 row-major
            let inverse_bind_matrices: Vec<BoneMatrix3x4> = converted
                .inverse_bind_matrices
                .iter()
                .map(|mat| {
                    // Input is column-major: [col0.xyz, col1.xyz, col2.xyz, col3.xyz]
                    BoneMatrix3x4::from_rows(
                        [mat[0], mat[3], mat[6], mat[9]],  // row 0
                        [mat[1], mat[4], mat[7], mat[10]], // row 1
                        [mat[2], mat[5], mat[8], mat[11]], // row 2
                    )
                })
                .collect();

            Ok(PackedSkeleton {
                id: id.to_string(),
                bone_count: converted.bone_count,
                inverse_bind_matrices,
            })
        }

        _ => anyhow::bail!(
            "Unsupported skeleton format '{}': {} (use .nczxskel, .gltf, or .glb)",
            ext,
            path.display()
        ),
    }
}

/// Load a skeleton from native .nczxskel file
fn load_skeleton_native(id: &str, path: &std::path::Path) -> Result<PackedSkeleton> {
    let data = std::fs::read(path)
        .with_context(|| format!("Failed to load skeleton: {}", path.display()))?;

    // Parse header
    let header = NetherZXSkeletonHeader::from_bytes(&data)
        .context("Failed to parse skeleton header - file may be corrupted or wrong format")?;

    let bone_count = header.bone_count;

    // Validate header
    if bone_count == 0 {
        anyhow::bail!("Skeleton has no bones: {}", path.display());
    }
    if bone_count > 256 {
        anyhow::bail!(
            "Skeleton has too many bones ({}, max 256): {}",
            bone_count,
            path.display()
        );
    }

    // Validate data size
    let expected_size =
        NetherZXSkeletonHeader::SIZE + (bone_count as usize * INVERSE_BIND_MATRIX_SIZE);
    if data.len() < expected_size {
        anyhow::bail!(
            "Skeleton data too small: {} bytes, expected {} (bones: {})",
            data.len(),
            expected_size,
            bone_count
        );
    }

    // Extract inverse bind matrices
    let matrix_data = &data[NetherZXSkeletonHeader::SIZE..expected_size];
    let mut inverse_bind_matrices = Vec::with_capacity(bone_count as usize);

    for i in 0..bone_count as usize {
        let offset = i * INVERSE_BIND_MATRIX_SIZE;
        let matrix_bytes = &matrix_data[offset..offset + INVERSE_BIND_MATRIX_SIZE];

        // Parse 12 floats from file (column-major: col0, col1, col2, col3)
        let mut cols = [[0.0f32; 3]; 4];
        for (col, col_data) in cols.iter_mut().enumerate() {
            for (row, cell) in col_data.iter_mut().enumerate() {
                let float_offset = (col * 3 + row) * 4;
                *cell = f32::from_le_bytes([
                    matrix_bytes[float_offset],
                    matrix_bytes[float_offset + 1],
                    matrix_bytes[float_offset + 2],
                    matrix_bytes[float_offset + 3],
                ]);
            }
        }

        // Convert to row-major for BoneMatrix3x4
        let matrix = BoneMatrix3x4::from_rows(
            [cols[0][0], cols[1][0], cols[2][0], cols[3][0]], // row 0
            [cols[0][1], cols[1][1], cols[2][1], cols[3][1]], // row 1
            [cols[0][2], cols[1][2], cols[2][2], cols[3][2]], // row 2
        );
        inverse_bind_matrices.push(matrix);
    }

    Ok(PackedSkeleton {
        id: id.to_string(),
        bone_count,
        inverse_bind_matrices,
    })
}
