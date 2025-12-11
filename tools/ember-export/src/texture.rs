//! Texture converter (PNG/JPG -> .embertex)

use anyhow::{Context, Result};
use image::GenericImageView;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use crate::formats::write_ember_texture;

/// Convert an image file to EmberTexture format
pub fn convert_image(input: &Path, output: &Path) -> Result<()> {
    // Load image
    let img = image::open(input).with_context(|| format!("Failed to load image: {:?}", input))?;

    let (width, height) = img.dimensions();

    // Convert to RGBA8
    let rgba = img.to_rgba8();
    let pixels = rgba.as_raw();

    // Write output
    let file =
        File::create(output).with_context(|| format!("Failed to create output: {:?}", output))?;
    let mut writer = BufWriter::new(file);

    write_ember_texture(&mut writer, width, height, pixels)?;

    tracing::info!(
        "Converted texture: {}x{}, {} bytes",
        width,
        height,
        pixels.len()
    );

    Ok(())
}
