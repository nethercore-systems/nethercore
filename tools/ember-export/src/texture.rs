//! Texture converter (PNG/JPG -> .embertex)
//!
//! Supports two output formats:
//! - RGBA8 (Mode 0): Uncompressed, pixel-perfect, 32 bpp
//! - BC7 (Modes 1-3): Compressed, 8 bpp, 4× size reduction

use anyhow::{Context, Result};
use image::GenericImageView;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use z_common::TextureFormat;

use crate::formats::write_ember_texture;

/// Convert an image file to EmberTexture format (RGBA8)
pub fn convert_image(input: &Path, output: &Path) -> Result<()> {
    convert_image_with_format(input, output, TextureFormat::Rgba8)
}

/// Convert an image file to EmberTexture format with explicit format
pub fn convert_image_with_format(input: &Path, output: &Path, format: TextureFormat) -> Result<()> {
    // Load image
    let img = image::open(input).with_context(|| format!("Failed to load image: {:?}", input))?;

    let (width, height) = img.dimensions();

    // Convert to RGBA8
    let rgba = img.to_rgba8();
    let pixels = rgba.as_raw();

    // Compress or pass through based on format
    let (output_data, output_format) = match format {
        TextureFormat::Rgba8 => (pixels.to_vec(), TextureFormat::Rgba8),
        TextureFormat::Bc7 => {
            let compressed = compress_bc7(pixels, width, height)?;
            (compressed, TextureFormat::Bc7)
        }
    };

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_ember_texture(
        &mut writer,
        width as u16,
        height as u16,
        output_format,
        &output_data,
    )?;

    let compression_info = if output_format.is_bc7() {
        let original_size = (width * height * 4) as usize;
        let compressed_size = output_data.len();
        let ratio = original_size as f32 / compressed_size as f32;
        format!(
            " (BC7: {} -> {} bytes, {:.1}× compression)",
            original_size, compressed_size, ratio
        )
    } else {
        String::new()
    };

    tracing::info!(
        "Converted texture: {}x{}, {} format{}",
        width,
        height,
        output_format.wgpu_format_name(),
        compression_info
    );

    Ok(())
}

/// Compress RGBA8 pixels to BC7 format
///
/// Uses intel_tex_2 (ISPC-based) for high-quality BC7 compression.
/// BC7 compresses 4×4 pixel blocks into 16 bytes each (8 bpp vs 32 bpp for RGBA8).
///
/// # Arguments
/// * `pixels` - RGBA8 pixel data (width × height × 4 bytes)
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
///
/// # Returns
/// BC7 compressed block data
pub fn compress_bc7(pixels: &[u8], width: u32, height: u32) -> Result<Vec<u8>> {
    use intel_tex_2::bc7;

    let w = width as usize;
    let h = height as usize;

    // Calculate block dimensions (round up to 4×4 blocks)
    let blocks_x = (w + 3) / 4;
    let blocks_y = (h + 3) / 4;
    let output_size = blocks_x * blocks_y * 16;

    let mut output = vec![0u8; output_size];

    // Create padded buffer if dimensions aren't multiples of 4
    let padded_width = blocks_x * 4;
    let padded_height = blocks_y * 4;

    let input_data: Vec<u8> = if w == padded_width && h == padded_height {
        // No padding needed
        pixels.to_vec()
    } else {
        // Create padded buffer, copying original and extending edges
        let mut padded = vec![0u8; padded_width * padded_height * 4];

        for y in 0..padded_height {
            for x in 0..padded_width {
                // Clamp to original dimensions (edge extension)
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

/// Process texture for packing with specified format
///
/// This is the main entry point for the pack command to compress textures.
/// Returns (width, height, format, data).
#[allow(dead_code)]
pub fn process_texture_for_pack(
    pixels: &[u8],
    width: u32,
    height: u32,
    format: TextureFormat,
) -> Result<(u16, u16, TextureFormat, Vec<u8>)> {
    let data = match format {
        TextureFormat::Rgba8 => pixels.to_vec(),
        TextureFormat::Bc7 => compress_bc7(pixels, width, height)?,
    };

    Ok((width as u16, height as u16, format, data))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bc7_compression_basic() {
        // Create a simple 4×4 solid color image
        let width = 4u32;
        let height = 4u32;
        let pixels: Vec<u8> = vec![255, 0, 0, 255].repeat(16); // Red

        let compressed = compress_bc7(&pixels, width, height).unwrap();

        // BC7 output should be exactly 16 bytes for a 4×4 block
        assert_eq!(compressed.len(), 16);
    }

    #[test]
    fn test_bc7_compression_larger() {
        // 64×64 image = 16×16 blocks = 256 blocks × 16 bytes = 4096 bytes
        let width = 64u32;
        let height = 64u32;
        let pixels: Vec<u8> = vec![0, 128, 255, 255].repeat((width * height) as usize);

        let compressed = compress_bc7(&pixels, width, height).unwrap();

        assert_eq!(compressed.len(), 4096);
    }

    #[test]
    fn test_bc7_compression_non_aligned() {
        // 30×30 image should be padded to 32×32 (8×8 blocks)
        let width = 30u32;
        let height = 30u32;
        let pixels: Vec<u8> = vec![128, 128, 128, 255].repeat((width * height) as usize);

        let compressed = compress_bc7(&pixels, width, height).unwrap();

        // 8×8 blocks × 16 bytes = 1024 bytes
        assert_eq!(compressed.len(), 8 * 8 * 16);
    }

    #[test]
    fn test_bc7_compression_ratio() {
        let width = 64u32;
        let height = 64u32;
        let pixels: Vec<u8> = vec![0; (width * height * 4) as usize];

        let compressed = compress_bc7(&pixels, width, height).unwrap();

        let original_size = (width * height * 4) as usize;
        let ratio = original_size / compressed.len();
        assert_eq!(ratio, 4); // 4× compression
    }

    #[test]
    fn test_process_texture_rgba8() {
        let pixels = vec![255u8; 16 * 16 * 4];
        let (w, h, fmt, data) =
            process_texture_for_pack(&pixels, 16, 16, TextureFormat::Rgba8).unwrap();

        assert_eq!(w, 16);
        assert_eq!(h, 16);
        assert_eq!(fmt, TextureFormat::Rgba8);
        assert_eq!(data.len(), 16 * 16 * 4);
    }

    #[test]
    fn test_process_texture_bc7() {
        let pixels = vec![255u8; 16 * 16 * 4];
        let (w, h, fmt, data) =
            process_texture_for_pack(&pixels, 16, 16, TextureFormat::Bc7).unwrap();

        assert_eq!(w, 16);
        assert_eq!(h, 16);
        assert_eq!(fmt, TextureFormat::Bc7);
        assert_eq!(data.len(), 4 * 4 * 16); // 4×4 blocks × 16 bytes
    }
}
