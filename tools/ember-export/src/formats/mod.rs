//! Binary format definitions for Emberware Z asset files
//!
//! Re-exports from emberware-shared for writing asset files.

pub use emberware_shared::formats::*;

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

/// Write a complete EmberTexture file
pub fn write_ember_texture<W: Write>(
    w: &mut W,
    width: u32,
    height: u32,
    pixels: &[u8],
) -> Result<()> {
    let header = EmberZTextureHeader::new(width, height);
    w.write_all(&header.to_bytes())?;
    w.write_all(pixels)?;
    Ok(())
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
