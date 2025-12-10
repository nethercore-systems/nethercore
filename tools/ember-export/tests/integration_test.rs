//! Integration tests for ember-export
//!
//! Tests the full pipeline: generate test assets -> convert -> verify output

mod generate_test_assets;

use std::path::Path;
use tempfile::tempdir;

/// Test OBJ -> EmberZMesh conversion
#[test]
fn test_obj_to_embermesh() {
    let dir = tempdir().expect("Failed to create temp dir");
    let obj_path = dir.path().join("cube.obj");
    let mesh_path = dir.path().join("cube.embermesh");

    // Generate test OBJ
    generate_test_assets::generate_cube_obj(&obj_path).expect("Failed to generate OBJ");
    assert!(obj_path.exists(), "OBJ file should exist");

    // Convert to EmberZMesh
    ember_export_convert_obj(&obj_path, &mesh_path);
    assert!(mesh_path.exists(), "EmberMesh file should exist");

    // Verify the output file structure
    let data = std::fs::read(&mesh_path).expect("Failed to read mesh file");
    verify_ember_z_mesh(&data);
}

/// Test PNG -> EmberZTexture conversion
#[test]
fn test_png_to_embertex() {
    let dir = tempdir().expect("Failed to create temp dir");
    let png_path = dir.path().join("test.png");
    let tex_path = dir.path().join("test.embertex");

    // Generate test PNG
    generate_test_assets::generate_checkerboard_png(&png_path).expect("Failed to generate PNG");
    assert!(png_path.exists(), "PNG file should exist");

    // Convert to EmberZTexture
    ember_export_convert_texture(&png_path, &tex_path);
    assert!(tex_path.exists(), "EmberTexture file should exist");

    // Verify the output file structure
    let data = std::fs::read(&tex_path).expect("Failed to read texture file");
    verify_ember_z_texture(&data, 4, 4); // 4x4 checkerboard
}

/// Test minimal triangle OBJ
#[test]
fn test_triangle_obj() {
    let dir = tempdir().expect("Failed to create temp dir");
    let obj_path = dir.path().join("triangle.obj");
    let mesh_path = dir.path().join("triangle.embermesh");

    generate_test_assets::generate_triangle_obj(&obj_path).expect("Failed to generate triangle OBJ");
    ember_export_convert_obj(&obj_path, &mesh_path);

    let data = std::fs::read(&mesh_path).expect("Failed to read mesh file");
    verify_ember_z_mesh(&data);
}

// Helper to run ember-export mesh command
fn ember_export_convert_obj(input: &Path, output: &Path) {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_ember-export"))
        .args(["mesh", input.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .status()
        .expect("Failed to run ember-export");
    assert!(status.success(), "ember-export mesh command failed");
}

// Helper to run ember-export texture command
fn ember_export_convert_texture(input: &Path, output: &Path) {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_ember-export"))
        .args(["texture", input.to_str().unwrap(), "-o", output.to_str().unwrap()])
        .status()
        .expect("Failed to run ember-export");
    assert!(status.success(), "ember-export texture command failed");
}

// Verify EmberZMesh header structure
fn verify_ember_z_mesh(data: &[u8]) {
    use emberware_shared::formats::EmberZMeshHeader;

    assert!(data.len() >= EmberZMeshHeader::SIZE, "Mesh data too small for header");

    let header = EmberZMeshHeader::from_bytes(data).expect("Failed to parse mesh header");

    assert!(header.vertex_count > 0, "Should have vertices");
    assert!(header.format <= 15, "Format should be valid (0-15)");

    // Calculate expected data size
    let stride = calculate_stride(header.format);
    let vertex_size = header.vertex_count as usize * stride;
    let index_size = header.index_count as usize * 2;
    let expected_size = EmberZMeshHeader::SIZE + vertex_size + index_size;

    assert!(
        data.len() >= expected_size,
        "Mesh data too small: {} < {} (vertices: {}, indices: {}, format: {})",
        data.len(),
        expected_size,
        header.vertex_count,
        header.index_count,
        header.format
    );

    println!(
        "Verified mesh: {} vertices, {} indices, format={}, stride={}",
        header.vertex_count, header.index_count, header.format, stride
    );
}

// Verify EmberZTexture header structure
fn verify_ember_z_texture(data: &[u8], expected_width: u32, expected_height: u32) {
    use emberware_shared::formats::EmberZTextureHeader;

    assert!(data.len() >= EmberZTextureHeader::SIZE, "Texture data too small for header");

    let header = EmberZTextureHeader::from_bytes(data).expect("Failed to parse texture header");

    assert_eq!(header.width, expected_width, "Width mismatch");
    assert_eq!(header.height, expected_height, "Height mismatch");

    let pixel_size = header.pixel_size();
    let expected_total = EmberZTextureHeader::SIZE + pixel_size;

    assert_eq!(
        data.len(),
        expected_total,
        "Texture data size mismatch: {} != {}",
        data.len(),
        expected_total
    );

    println!(
        "Verified texture: {}x{}, {} bytes pixel data",
        header.width, header.height, pixel_size
    );
}

// Calculate vertex stride based on format flags
fn calculate_stride(format: u8) -> usize {
    const FORMAT_UV: u8 = 1;
    const FORMAT_COLOR: u8 = 2;
    const FORMAT_NORMAL: u8 = 4;
    const FORMAT_SKINNED: u8 = 8;

    let mut stride = 8; // Position (f16x4)

    if format & FORMAT_UV != 0 {
        stride += 4; // UV (unorm16x2)
    }
    if format & FORMAT_COLOR != 0 {
        stride += 4; // Color (unorm8x4)
    }
    if format & FORMAT_NORMAL != 0 {
        stride += 4; // Normal (octahedral u32)
    }
    if format & FORMAT_SKINNED != 0 {
        stride += 8; // Bone indices (u8x4) + weights (unorm8x4)
    }

    stride
}
