//! Test asset generation
//!
//! Generates simple test assets for integration testing.
//! Uses proper libraries (image) and text formats (OBJ) - no magic bytes.

use std::fs;
use std::io::Write;
use std::path::Path;

/// Generate a minimal 2x2 RGBA PNG image using the image crate
pub fn generate_test_png(path: &Path) -> std::io::Result<()> {
    let width = 2u32;
    let height = 2u32;

    // 2x2 image: red, green, blue, white
    let pixels: Vec<u8> = vec![
        255, 0, 0, 255,       // Red
        0, 255, 0, 255,       // Green
        0, 0, 255, 255,       // Blue
        255, 255, 255, 255,   // White
    ];

    image::save_buffer(path, &pixels, width, height, image::ColorType::Rgba8)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Generate a simple 4x4 checkerboard PNG
pub fn generate_checkerboard_png(path: &Path) -> std::io::Result<()> {
    let width = 4u32;
    let height = 4u32;
    let mut pixels = vec![0u8; (width * height * 4) as usize];

    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let is_white = (x + y) % 2 == 0;
            if is_white {
                pixels[idx] = 255;     // R
                pixels[idx + 1] = 255; // G
                pixels[idx + 2] = 255; // B
                pixels[idx + 3] = 255; // A
            } else {
                pixels[idx] = 128;     // R
                pixels[idx + 1] = 64;  // G
                pixels[idx + 2] = 192; // B
                pixels[idx + 3] = 255; // A
            }
        }
    }

    image::save_buffer(path, &pixels, width, height, image::ColorType::Rgba8)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}

/// Generate a simple cube OBJ file (text format, no magic bytes)
pub fn generate_cube_obj(path: &Path) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;

    // Simple 1x1x1 cube centered at origin with normals
    writeln!(file, "# Simple cube for testing")?;
    writeln!(file)?;

    // Vertices (8 corners of unit cube)
    writeln!(file, "v -0.5 -0.5  0.5")?;
    writeln!(file, "v  0.5 -0.5  0.5")?;
    writeln!(file, "v  0.5  0.5  0.5")?;
    writeln!(file, "v -0.5  0.5  0.5")?;
    writeln!(file, "v -0.5 -0.5 -0.5")?;
    writeln!(file, "v  0.5 -0.5 -0.5")?;
    writeln!(file, "v  0.5  0.5 -0.5")?;
    writeln!(file, "v -0.5  0.5 -0.5")?;
    writeln!(file)?;

    // Normals (6 face normals)
    writeln!(file, "vn  0  0  1")?;  // front
    writeln!(file, "vn  0  0 -1")?;  // back
    writeln!(file, "vn  1  0  0")?;  // right
    writeln!(file, "vn -1  0  0")?;  // left
    writeln!(file, "vn  0  1  0")?;  // top
    writeln!(file, "vn  0 -1  0")?;  // bottom
    writeln!(file)?;

    // UVs (simple 0-1 mapping per face)
    writeln!(file, "vt 0 0")?;
    writeln!(file, "vt 1 0")?;
    writeln!(file, "vt 1 1")?;
    writeln!(file, "vt 0 1")?;
    writeln!(file)?;

    // Faces (6 quads = 12 triangles)
    // Format: f v/vt/vn

    // Front face (+Z)
    writeln!(file, "f 1/1/1 2/2/1 3/3/1")?;
    writeln!(file, "f 1/1/1 3/3/1 4/4/1")?;

    // Back face (-Z)
    writeln!(file, "f 6/1/2 5/2/2 8/3/2")?;
    writeln!(file, "f 6/1/2 8/3/2 7/4/2")?;

    // Right face (+X)
    writeln!(file, "f 2/1/3 6/2/3 7/3/3")?;
    writeln!(file, "f 2/1/3 7/3/3 3/4/3")?;

    // Left face (-X)
    writeln!(file, "f 5/1/4 1/2/4 4/3/4")?;
    writeln!(file, "f 5/1/4 4/3/4 8/4/4")?;

    // Top face (+Y)
    writeln!(file, "f 4/1/5 3/2/5 7/3/5")?;
    writeln!(file, "f 4/1/5 7/3/5 8/4/5")?;

    // Bottom face (-Y)
    writeln!(file, "f 5/1/6 6/2/6 2/3/6")?;
    writeln!(file, "f 5/1/6 2/3/6 1/4/6")?;

    Ok(())
}

/// Generate a simple triangle OBJ file (minimal test case)
pub fn generate_triangle_obj(path: &Path) -> std::io::Result<()> {
    let mut file = fs::File::create(path)?;

    writeln!(file, "# Simple triangle for testing")?;
    writeln!(file)?;
    writeln!(file, "v 0 0 0")?;
    writeln!(file, "v 1 0 0")?;
    writeln!(file, "v 0.5 1 0")?;
    writeln!(file)?;
    writeln!(file, "vn 0 0 1")?;
    writeln!(file)?;
    writeln!(file, "f 1//1 2//1 3//1")?;

    Ok(())
}
