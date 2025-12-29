//! Skeleton converter (glTF -> .nczxskel)
//!
//! Extracts inverse bind matrices from glTF skins for skeletal animation.

use anyhow::{bail, Context, Result};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::formats::write_nether_skeleton;

/// Result of in-memory skeleton conversion
#[derive(Debug, Clone)]
pub struct ConvertedSkeleton {
    /// Number of bones in the skeleton
    pub bone_count: u32,
    /// Inverse bind matrices in column-major 3x4 format (12 floats per bone)
    pub inverse_bind_matrices: Vec<[f32; 12]>,
}

/// Convert glTF skeleton to in-memory format (for direct ROM packing)
///
/// # Arguments
/// * `input` - Path to the glTF/GLB file
/// * `skin_name` - Optional skin name to select (uses first skin if None)
pub fn convert_gltf_skeleton_to_memory(
    input: &Path,
    skin_name: Option<&str>,
) -> Result<ConvertedSkeleton> {
    let (document, buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    // Find skin by name or use first
    let skin = if let Some(name) = skin_name {
        document
            .skins()
            .find(|s| s.name() == Some(name))
            .with_context(|| format!("Skin '{}' not found in glTF", name))?
    } else {
        document
            .skins()
            .next()
            .context("No skins found in glTF file")?
    };

    // Get inverse bind matrices
    let inverse_bind_matrices = extract_inverse_bind_matrices(&skin, &buffers)?;

    if inverse_bind_matrices.is_empty() {
        bail!("No bones found in skin");
    }

    if inverse_bind_matrices.len() > 256 {
        bail!(
            "Skeleton has {} bones, but maximum is 256",
            inverse_bind_matrices.len()
        );
    }

    Ok(ConvertedSkeleton {
        bone_count: inverse_bind_matrices.len() as u32,
        inverse_bind_matrices,
    })
}

/// Convert glTF skin data to NetherSkeleton format
pub fn convert_gltf_skeleton(input: &Path, output: &Path, skin_index: Option<usize>) -> Result<()> {
    let (document, buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    // Get the skin to export
    let skin = if let Some(idx) = skin_index {
        document
            .skins()
            .nth(idx)
            .with_context(|| format!("Skin index {} not found in glTF", idx))?
    } else {
        document
            .skins()
            .next()
            .context("No skins found in glTF file")?
    };

    // Get inverse bind matrices
    let inverse_bind_matrices = extract_inverse_bind_matrices(&skin, &buffers)?;

    if inverse_bind_matrices.is_empty() {
        bail!("No bones found in skin");
    }

    if inverse_bind_matrices.len() > 256 {
        bail!(
            "Skeleton has {} bones, but maximum is 256",
            inverse_bind_matrices.len()
        );
    }

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_nether_skeleton(&mut writer, &inverse_bind_matrices)?;

    tracing::info!(
        "Exported skeleton: {} bones from skin '{}'",
        inverse_bind_matrices.len(),
        skin.name().unwrap_or("unnamed")
    );

    Ok(())
}

/// Extract inverse bind matrices from a glTF skin
fn extract_inverse_bind_matrices(
    skin: &gltf::Skin,
    buffers: &[gltf::buffer::Data],
) -> Result<Vec<[f32; 12]>> {
    let joints = skin.joints().collect::<Vec<_>>();
    let joint_count = joints.len();

    // Get inverse bind matrices accessor
    let ibm_accessor = skin
        .inverse_bind_matrices()
        .context("Skin has no inverse bind matrices")?;

    // Read inverse bind matrices from buffer
    let ibm_view = ibm_accessor
        .view()
        .context("Invalid inverse bind matrices accessor")?;
    let buffer = &buffers[ibm_view.buffer().index()];
    let offset = ibm_view.offset() + ibm_accessor.offset();
    let data = &buffer[offset..];

    // Parse mat4 matrices and convert to 3x4 column-major
    let mut inverse_bind_matrices = Vec::with_capacity(joint_count);

    for i in 0..joint_count {
        // Each mat4 is 16 floats (64 bytes) in column-major order
        let mat_offset = i * 64;
        if mat_offset + 64 > data.len() {
            bail!(
                "Invalid inverse bind matrix data: expected {} bytes, got {}",
                (i + 1) * 64,
                data.len()
            );
        }

        // Read 16 floats (column-major mat4)
        let mut mat4 = [0.0f32; 16];
        for (j, float) in mat4.iter_mut().enumerate() {
            let byte_offset = mat_offset + j * 4;
            let bytes = [
                data[byte_offset],
                data[byte_offset + 1],
                data[byte_offset + 2],
                data[byte_offset + 3],
            ];
            *float = f32::from_le_bytes(bytes);
        }

        // Convert to 3x4 column-major: [col0.xyz, col1.xyz, col2.xyz, col3.xyz]
        // glTF mat4 is already column-major: [col0 (0-3), col1 (4-7), col2 (8-11), col3 (12-15)]
        let mat3x4: [f32; 12] = [
            mat4[0], mat4[1], mat4[2], // col0.xyz
            mat4[4], mat4[5], mat4[6], // col1.xyz
            mat4[8], mat4[9], mat4[10], // col2.xyz
            mat4[12], mat4[13], mat4[14], // col3.xyz (translation)
        ];

        inverse_bind_matrices.push(mat3x4);
    }

    Ok(inverse_bind_matrices)
}

/// List available skins in a glTF file
pub fn list_skins(input: &Path) -> Result<()> {
    let (document, _buffers, _images) =
        gltf::import(input).with_context(|| format!("Failed to load glTF: {:?}", input))?;

    let skins: Vec<_> = document.skins().collect();
    if skins.is_empty() {
        tracing::info!("No skins found in {:?}", input);
        return Ok(());
    }

    tracing::info!("Skins in {:?}:", input);
    for (i, skin) in skins.iter().enumerate() {
        let name = skin.name().unwrap_or("unnamed");
        let joint_count = skin.joints().count();
        tracing::info!("  [{}] '{}': {} joints", i, name, joint_count);
    }

    Ok(())
}
