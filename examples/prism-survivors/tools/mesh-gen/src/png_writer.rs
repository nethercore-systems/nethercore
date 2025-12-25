//! PNG file writer for procedural texture generation
//!
//! Simple PNG writer using the `png` crate.

use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Write RGBA pixel data to a PNG file
///
/// # Arguments
/// * `path` - Output file path
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `pixels` - RGBA pixel data (4 bytes per pixel, row-major order)
pub fn write_png(path: &Path, width: u32, height: u32, pixels: &[u8]) -> std::io::Result<()> {
    let file = File::create(path)?;
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, width, height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Default);

    let mut writer = encoder
        .write_header()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    writer
        .write_image_data(pixels)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}

/// Generate a solid color texture
pub fn generate_solid(width: u32, height: u32, color: [u8; 4]) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];
    for i in 0..(width * height) as usize {
        pixels[i * 4] = color[0];
        pixels[i * 4 + 1] = color[1];
        pixels[i * 4 + 2] = color[2];
        pixels[i * 4 + 3] = color[3];
    }
    pixels
}

/// Generate a checkerboard pattern texture
pub fn generate_checker(
    width: u32,
    height: u32,
    tile_size: u32,
    color_a: [u8; 4],
    color_b: [u8; 4],
) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let checker = ((x / tile_size) + (y / tile_size)) % 2 == 0;
            let color = if checker { color_a } else { color_b };
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
        }
    }
    pixels
}

/// Generate a gradient texture (vertical)
pub fn generate_gradient_v(
    width: u32,
    height: u32,
    color_top: [u8; 4],
    color_bottom: [u8; 4],
) -> Vec<u8> {
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        let t = y as f32 / (height - 1) as f32;
        let color = [
            lerp_u8(color_top[0], color_bottom[0], t),
            lerp_u8(color_top[1], color_bottom[1], t),
            lerp_u8(color_top[2], color_bottom[2], t),
            lerp_u8(color_top[3], color_bottom[3], t),
        ];

        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = color[0];
            pixels[idx + 1] = color[1];
            pixels[idx + 2] = color[2];
            pixels[idx + 3] = color[3];
        }
    }
    pixels
}

/// Linear interpolation for u8 values
fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    let a = a as f32;
    let b = b as f32;
    (a + (b - a) * t).clamp(0.0, 255.0) as u8
}

/// Generate a noise-based texture (useful for materials)
pub fn generate_noise(
    width: u32,
    height: u32,
    base_color: [u8; 4],
    noise_intensity: u8,
    seed: u32,
) -> Vec<u8> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            // Simple deterministic noise using hash
            let mut hasher = DefaultHasher::new();
            (x, y, seed).hash(&mut hasher);
            let hash = hasher.finish();
            let noise_val = ((hash % 256) as i16 - 128) * noise_intensity as i16 / 128;

            let idx = ((y * width + x) * 4) as usize;
            pixels[idx] = (base_color[0] as i16 + noise_val).clamp(0, 255) as u8;
            pixels[idx + 1] = (base_color[1] as i16 + noise_val).clamp(0, 255) as u8;
            pixels[idx + 2] = (base_color[2] as i16 + noise_val).clamp(0, 255) as u8;
            pixels[idx + 3] = base_color[3];
        }
    }
    pixels
}
