//! Binary format definitions for Emberware Z asset files
//!
//! Re-exports from z-common for writing asset files.

pub use z_common::formats::*;

use anyhow::Result;
use std::io::Write;

/// Write a complete EmberMesh file
pub fn write_ember_mesh<W: Write>(
    w: &mut W,
    format: u8,
    vertex_data: &[u8],
    indices: Option<&[u16]>,
) -> Result<()> {
    let stride = crate::vertex_stride_packed(format) as usize;
    let vertex_count = (vertex_data.len() / stride) as u32;
    let index_count = indices.map(|i| i.len() as u32).unwrap_or(0);

    let header = EmberZMeshHeader::new(vertex_count, index_count, format);
    w.write_all(&header.to_bytes())?;
    w.write_all(vertex_data)?;

    if let Some(idx) = indices {
        for i in idx {
            w.write_all(&i.to_le_bytes())?;
        }
    }

    Ok(())
}

/// Write a complete EmberTexture file (RGBA8 or BC7)
///
/// # Arguments
/// * `w` - Writer to output to
/// * `width` - Texture width (u16, max 65535)
/// * `height` - Texture height (u16, max 65535)
/// * `format` - Texture format (Rgba8, Bc7, or Bc7Linear)
/// * `data` - Pixel data (RGBA8) or compressed blocks (BC7)
pub fn write_ember_texture<W: Write>(
    w: &mut W,
    width: u16,
    height: u16,
    _format: TextureFormat,
    data: &[u8],
) -> Result<()> {
    let header = EmberZTextureHeader::new(width, height);
    w.write_all(&header.to_bytes())?;
    w.write_all(data)?;
    Ok(())
}

/// Write an RGBA8 EmberTexture file (legacy compatibility)
///
/// This is a convenience wrapper that assumes RGBA8 format.
pub fn write_ember_texture_rgba8<W: Write>(
    w: &mut W,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<()> {
    write_ember_texture(w, width as u16, height as u16, TextureFormat::Rgba8, pixels)
}

/// Write a complete EmberSound file
pub fn write_ember_sound<W: Write>(w: &mut W, samples: &[i16]) -> Result<()> {
    let header = EmberZSoundHeader::new(samples.len() as u32);
    w.write_all(&header.to_bytes())?;
    for sample in samples {
        w.write_all(&sample.to_le_bytes())?;
    }
    Ok(())
}

/// Write a complete EmberSkeleton file
///
/// Inverse bind matrices are stored as 12 floats per bone (3×4 column-major).
pub fn write_ember_skeleton<W: Write>(
    w: &mut W,
    inverse_bind_matrices: &[[f32; 12]],
) -> Result<()> {
    let header = EmberZSkeletonHeader::new(inverse_bind_matrices.len() as u32);
    w.write_all(&header.to_bytes())?;

    for matrix in inverse_bind_matrices {
        for f in matrix {
            w.write_all(&f.to_le_bytes())?;
        }
    }

    Ok(())
}

/// Write a complete EmberAnimation file
///
/// Frame data is stored as bone_count × frame_count matrices (12 floats each).
pub fn write_ember_animation<W: Write>(
    w: &mut W,
    bone_count: u32,
    frame_rate: f32,
    frames: &[Vec<[f32; 12]>],
) -> Result<()> {
    let header = EmberZAnimationHeader::new(bone_count, frames.len() as u32, frame_rate);
    w.write_all(&header.to_bytes())?;

    for frame in frames {
        for matrix in frame {
            for f in matrix {
                w.write_all(&f.to_le_bytes())?;
            }
        }
    }

    Ok(())
}
