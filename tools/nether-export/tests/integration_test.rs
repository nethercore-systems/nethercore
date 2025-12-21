//! Integration tests for nether-export
//!
//! Tests the full pipeline: generate test assets -> convert -> verify output

mod generate_test_assets;

use std::path::Path;
use tempfile::tempdir;

/// Test OBJ -> NetherZMesh conversion
#[test]
fn test_obj_to_embermesh() {
    let dir = tempdir().expect("Failed to create temp dir");
    let obj_path = dir.path().join("cube.obj");
    let mesh_path = dir.path().join("cube.embermesh");

    // Generate test OBJ
    generate_test_assets::generate_cube_obj(&obj_path).expect("Failed to generate OBJ");
    assert!(obj_path.exists(), "OBJ file should exist");

    // Convert to NetherZMesh
    ember_export_convert_obj(&obj_path, &mesh_path);
    assert!(mesh_path.exists(), "EmberMesh file should exist");

    // Verify the output file structure
    let data = std::fs::read(&mesh_path).expect("Failed to read mesh file");
    verify_ember_z_mesh(&data);
}

/// Test PNG -> NetherZTexture conversion
#[test]
fn test_png_to_embertex() {
    let dir = tempdir().expect("Failed to create temp dir");
    let png_path = dir.path().join("test.png");
    let tex_path = dir.path().join("test.embertex");

    // Generate test PNG
    generate_test_assets::generate_checkerboard_png(&png_path).expect("Failed to generate PNG");
    assert!(png_path.exists(), "PNG file should exist");

    // Convert to NetherZTexture
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

    generate_test_assets::generate_triangle_obj(&obj_path)
        .expect("Failed to generate triangle OBJ");
    ember_export_convert_obj(&obj_path, &mesh_path);

    let data = std::fs::read(&mesh_path).expect("Failed to read mesh file");
    verify_ember_z_mesh(&data);
}

// Helper to run nether-export mesh command
fn ember_export_convert_obj(input: &Path, output: &Path) {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_nether-export"))
        .args([
            "mesh",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run nether-export");
    assert!(status.success(), "nether-export mesh command failed");
}

// Helper to run nether-export texture command
fn ember_export_convert_texture(input: &Path, output: &Path) {
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_nether-export"))
        .args([
            "texture",
            input.to_str().unwrap(),
            "-o",
            output.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run nether-export");
    assert!(status.success(), "nether-export texture command failed");
}

// Verify NetherZMesh header structure
fn verify_ember_z_mesh(data: &[u8]) {
    use zx_common::NetherZMeshHeader;

    assert!(
        data.len() >= NetherZMeshHeader::SIZE,
        "Mesh data too small for header"
    );

    let header = NetherZMeshHeader::from_bytes(data).expect("Failed to parse mesh header");

    assert!(header.vertex_count > 0, "Should have vertices");
    assert!(header.format <= 15, "Format should be valid (0-15)");

    // Calculate expected data size
    let stride = calculate_stride(header.format);
    let vertex_size = header.vertex_count as usize * stride;
    let index_size = header.index_count as usize * 2;
    let expected_size = NetherZMeshHeader::SIZE + vertex_size + index_size;

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

// Verify NetherZTexture header structure
fn verify_ember_z_texture(data: &[u8], expected_width: u16, expected_height: u16) {
    use zx_common::NetherZTextureHeader;

    assert!(
        data.len() >= NetherZTextureHeader::SIZE,
        "Texture data too small for header"
    );

    let header = NetherZTextureHeader::from_bytes(data).expect("Failed to parse texture header");

    assert_eq!(header.width, expected_width, "Width mismatch");
    assert_eq!(header.height, expected_height, "Height mismatch");

    let pixel_size = header.rgba8_size();
    let expected_total = NetherZTextureHeader::SIZE + pixel_size;

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

// Use zx-common's stride calculation (no duplication)
fn calculate_stride(format: u8) -> usize {
    zx_common::vertex_stride_packed(format) as usize
}
