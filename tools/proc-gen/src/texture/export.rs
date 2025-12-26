//! PNG export for texture buffers

use super::TextureBuffer;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

/// Write a TextureBuffer to a PNG file
///
/// # Arguments
/// * `texture` - The texture buffer to write
/// * `path` - Output file path
///
/// # Example
/// ```no_run
/// use proc_gen::texture::{solid, write_png};
/// use std::path::Path;
///
/// let tex = solid(64, 64, [255, 0, 0, 255]);
/// write_png(&tex, Path::new("red.png")).unwrap();
/// ```
pub fn write_png(texture: &TextureBuffer, path: &Path) -> std::io::Result<()> {
    let file = File::create(path)?;
    let w = BufWriter::new(file);

    let mut encoder = png::Encoder::new(w, texture.width, texture.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_compression(png::Compression::Default);

    let mut writer = encoder
        .write_header()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    writer
        .write_image_data(&texture.pixels)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::checker;
    use std::fs;

    #[test]
    fn test_write_png() {
        let tex = checker(32, 32, 8, [255, 255, 255, 255], [0, 0, 0, 255]);
        let path = Path::new("test_output.png");

        // Write the file
        write_png(&tex, path).unwrap();

        // Verify file exists and has content
        let metadata = fs::metadata(path).unwrap();
        assert!(metadata.len() > 0);

        // Clean up
        fs::remove_file(path).unwrap();
    }
}
