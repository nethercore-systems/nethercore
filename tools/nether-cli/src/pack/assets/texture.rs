//! Texture loading and compression.

use anyhow::{Context, Result};
use zx_common::{PackedTexture, TextureFormat};

/// Load a texture from an image file (PNG, JPG, etc.)
///
/// Compresses to BC7 if the format requires it.
pub fn load_texture(
    id: &str,
    path: &std::path::Path,
    format: TextureFormat,
) -> Result<PackedTexture> {
    let img =
        image::open(path).with_context(|| format!("Failed to load texture: {}", path.display()))?;

    let rgba = img.to_rgba8();
    let (width, height) = rgba.dimensions();
    let pixels = rgba.as_raw();

    // Compress or pass through based on format
    let data = match format {
        TextureFormat::Rgba8 => pixels.to_vec(),
        TextureFormat::Bc7 => compress_bc7(pixels, width, height)?,
        TextureFormat::Bc5 => compress_bc5(pixels, width, height)?,
    };

    Ok(PackedTexture::with_format(
        id,
        width as u16,
        height as u16,
        format,
        data,
    ))
}

/// Compress RGBA8 pixels to BC7 format
///
/// Uses intel_tex_2 (ISPC-based) for high-quality BC7 compression.
fn compress_bc7(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    use intel_tex_2::bc7;

    let w = width as usize;
    let h = height as usize;

    // Calculate block dimensions (round up to 4x4 blocks)
    let blocks_x = w.div_ceil(4);
    let blocks_y = h.div_ceil(4);
    let output_size = blocks_x * blocks_y * 16;

    let mut output = vec![0u8; output_size];

    // Create padded buffer if dimensions aren't multiples of 4
    let padded_width = blocks_x * 4;
    let padded_height = blocks_y * 4;

    let input_data: Vec<u8> = if w == padded_width && h == padded_height {
        pixels.to_vec()
    } else {
        // Create padded buffer with edge extension
        let mut padded = vec![0u8; padded_width * padded_height * 4];

        for y in 0..padded_height {
            for x in 0..padded_width {
                let src_x = x.min(w - 1);
                let src_y = y.min(h - 1);

                let src_idx = (src_y * w + src_x) * 4;
                let dst_idx = (y * padded_width + x) * 4;

                padded[dst_idx..dst_idx + 4].copy_from_slice(&pixels[src_idx..src_idx + 4]);
            }
        }

        padded
    };

    // Create surface for intel_tex_2
    let surface = intel_tex_2::RgbaSurface {
        width: padded_width as u32,
        height: padded_height as u32,
        stride: (padded_width * 4) as u32,
        data: &input_data,
    };

    // Compress using intel_tex_2 BC7 (fast settings for good speed/quality balance)
    bc7::compress_blocks_into(&bc7::opaque_fast_settings(), &surface, &mut output);

    Ok(output)
}

/// Compress RGBA8 pixels to BC5 format (2-channel RG)
///
/// Used for normal maps. Extracts R and G channels from RGBA input.
/// The Z component is reconstructed in the shader: z = sqrt(1 - x^2 - y^2)
fn compress_bc5(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    use intel_tex_2::bc5;

    let w = width as usize;
    let h = height as usize;

    // Calculate block dimensions (round up to 4x4 blocks)
    let blocks_x = w.div_ceil(4);
    let blocks_y = h.div_ceil(4);
    let output_size = blocks_x * blocks_y * 16; // BC5 is 16 bytes per 4x4 block

    let mut output = vec![0u8; output_size];

    // Create padded buffer if dimensions aren't multiples of 4
    let padded_width = blocks_x * 4;
    let padded_height = blocks_y * 4;

    // Extract R and G channels into a 2-byte-per-pixel buffer
    let mut rg_data: Vec<u8> = Vec::with_capacity(padded_width * padded_height * 2);

    for y in 0..padded_height {
        for x in 0..padded_width {
            let src_x = x.min(w - 1);
            let src_y = y.min(h - 1);
            let src_idx = (src_y * w + src_x) * 4;

            // Extract R and G channels
            rg_data.push(pixels[src_idx]); // R
            rg_data.push(pixels[src_idx + 1]); // G
        }
    }

    // Create surface for intel_tex_2
    let surface = intel_tex_2::RgSurface {
        width: padded_width as u32,
        height: padded_height as u32,
        stride: (padded_width * 2) as u32,
        data: &rg_data,
    };

    // Compress using intel_tex_2 BC5
    bc5::compress_blocks_into(&surface, &mut output);

    Ok(output)
}
