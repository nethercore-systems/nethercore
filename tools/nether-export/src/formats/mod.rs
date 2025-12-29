//! Binary format definitions for Nethercore ZX asset files
//!
//! Re-exports from z-common for writing asset files.

pub use zx_common::formats::*;

use anyhow::Result;
use std::io::Write;

// Note: encode_bone_transform and NetherZXAnimationHeader are already available
// through the glob re-export above

use crate::animation::BoneTRS;

/// Write a complete NetherMesh file
///
/// Note: Index data alignment for GPU (wgpu COPY_BUFFER_ALIGNMENT) is handled at runtime
/// during GPU upload, not here. This keeps mesh files compact.
pub fn write_nether_mesh<W: Write>(
    w: &mut W,
    format: u8,
    vertex_data: &[u8],
    indices: Option<&[u16]>,
) -> Result<()> {
    let stride = crate::vertex_stride_packed(format) as usize;
    let vertex_count = (vertex_data.len() / stride) as u32;
    let index_count = indices.map(|i| i.len() as u32).unwrap_or(0);

    let header = NetherZXMeshHeader::new(vertex_count, index_count, format);
    w.write_all(&header.to_bytes())?;
    w.write_all(vertex_data)?;

    if let Some(idx) = indices {
        for i in idx {
            w.write_all(&i.to_le_bytes())?;
        }
    }

    Ok(())
}

/// Write a complete NetherTexture file (RGBA8 or BC7)
///
/// # Arguments
/// * `w` - Writer to output to
/// * `width` - Texture width (u16, max 65535)
/// * `height` - Texture height (u16, max 65535)
/// * `format` - Texture format (Rgba8, Bc7, or Bc7Linear)
/// * `data` - Pixel data (RGBA8) or compressed blocks (BC7)
pub fn write_nether_texture<W: Write>(
    w: &mut W,
    width: u16,
    height: u16,
    _format: TextureFormat,
    data: &[u8],
) -> Result<()> {
    let header = NetherZXTextureHeader::new(width, height);
    w.write_all(&header.to_bytes())?;
    w.write_all(data)?;
    Ok(())
}

/// Write a complete NetherSound file (QOA compressed format)
///
/// Uses QOA compression (~5:1 ratio) instead of raw PCM.
/// Format: NetherZXSoundHeader (4 bytes) + QOA frame data.
pub fn write_nether_sound<W: Write>(w: &mut W, samples: &[i16]) -> Result<()> {
    let header = NetherZXSoundHeader::new(samples.len() as u32);
    w.write_all(&header.to_bytes())?;

    let qoa_data = nether_qoa::encode_qoa(samples);
    w.write_all(&qoa_data)?;
    Ok(())
}

/// Write a complete NetherSkeleton file
///
/// Inverse bind matrices are stored as 12 floats per bone (3×4 column-major).
pub fn write_nether_skeleton<W: Write>(
    w: &mut W,
    inverse_bind_matrices: &[[f32; 12]],
) -> Result<()> {
    let header = NetherZXSkeletonHeader::new(inverse_bind_matrices.len() as u32);
    w.write_all(&header.to_bytes())?;

    for matrix in inverse_bind_matrices {
        for f in matrix {
            w.write_all(&f.to_le_bytes())?;
        }
    }

    Ok(())
}

/// Write a complete NetherAnimation file (new platform format)
///
/// Uses the compressed platform format (16 bytes per bone per frame):
/// - rotation: u32 (smallest-three packed quaternion)
/// - position: [u16; 3] (f16 × 3)
/// - scale: [u16; 3] (f16 × 3)
///
/// # Arguments
/// * `w` — Writer to output to
/// * `bone_count` — Number of bones per frame (max 255)
/// * `frames` — Vector of frames, each containing `bone_count` BoneTRS transforms
pub fn write_nether_animation<W: Write>(
    w: &mut W,
    bone_count: u8,
    frames: &[Vec<BoneTRS>],
) -> Result<()> {
    // Validate
    if frames.is_empty() {
        anyhow::bail!("Animation has no frames");
    }
    if bone_count == 0 {
        anyhow::bail!("Animation has no bones");
    }

    let frame_count = frames.len() as u16;

    // Write header (4 bytes)
    let header = NetherZXAnimationHeader::new(bone_count, frame_count);
    w.write_all(&header.to_bytes())?;

    // Write frame data (frame_count × bone_count × 16 bytes)
    for frame in frames {
        if frame.len() != bone_count as usize {
            anyhow::bail!("Frame has {} bones, expected {}", frame.len(), bone_count);
        }

        for bone in frame {
            // Encode TRS to platform format (16 bytes)
            let encoded = encode_bone_transform(bone.rotation, bone.position, bone.scale);
            w.write_all(&encoded.to_bytes())?;
        }
    }

    Ok(())
}
