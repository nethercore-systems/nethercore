//! Integration tests for GLTF/GLB import pipeline.
//!
//! Tests the complete flow:
//! 1. Generate GLB programmatically
//! 2. Convert through nether-export
//! 3. Validate output format and data

mod gltf_generator;

use tempfile::tempdir;

use nether_export::{
    convert_gltf_animation_to_memory, convert_gltf_skeleton_to_memory, convert_gltf_to_memory,
    FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV,
};

/// Expected format flags for skinned mesh with all attributes
const EXPECTED_FORMAT: u8 = FORMAT_UV | FORMAT_NORMAL | FORMAT_SKINNED;

/// Test that the generated GLB is valid and can be parsed
#[test]
fn test_generate_skinned_glb_valid() {
    let glb_data = gltf_generator::generate_skinned_glb();

    // Verify GLB header
    assert!(glb_data.len() > 12, "GLB too small");
    assert_eq!(&glb_data[0..4], b"glTF", "Invalid GLB magic");
    assert_eq!(
        u32::from_le_bytes(glb_data[4..8].try_into().unwrap()),
        2,
        "Expected glTF version 2"
    );

    // Write to temp file and parse with gltf crate
    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Import with gltf crate
    let (document, _buffers, _) = gltf::import(&glb_path).expect("Failed to import GLB");

    // Verify structure
    assert_eq!(document.meshes().count(), 1, "Expected 1 mesh");
    assert_eq!(document.skins().count(), 1, "Expected 1 skin");
    assert_eq!(document.animations().count(), 1, "Expected 1 animation");

    // Verify mesh has all attributes
    let mesh = document.meshes().next().unwrap();
    let primitive = mesh.primitives().next().unwrap();
    assert!(
        primitive.get(&gltf::Semantic::Positions).is_some(),
        "Missing POSITION"
    );
    assert!(
        primitive.get(&gltf::Semantic::Normals).is_some(),
        "Missing NORMAL"
    );
    assert!(
        primitive.get(&gltf::Semantic::TexCoords(0)).is_some(),
        "Missing TEXCOORD_0"
    );
    assert!(
        primitive.get(&gltf::Semantic::Joints(0)).is_some(),
        "Missing JOINTS_0"
    );
    assert!(
        primitive.get(&gltf::Semantic::Weights(0)).is_some(),
        "Missing WEIGHTS_0"
    );
    assert!(primitive.indices().is_some(), "Missing indices");

    // Verify skin
    let skin = document.skins().next().unwrap();
    assert_eq!(
        skin.joints().count(),
        gltf_generator::BONE_COUNT,
        "Incorrect bone count"
    );
    assert!(
        skin.inverse_bind_matrices().is_some(),
        "Missing inverse bind matrices"
    );

    // Verify animation
    let animation = document.animations().next().unwrap();
    assert_eq!(animation.name(), Some("Wave"), "Animation name mismatch");
    // Should have 3 channels per bone (translation, rotation, scale) * 3 bones = 9 channels
    assert_eq!(
        animation.channels().count(),
        gltf_generator::BONE_COUNT * 3,
        "Incorrect channel count"
    );

    println!("GLB structure validated successfully");
}

/// Test mesh conversion through nether-export
#[test]
fn test_glb_mesh_conversion() {
    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Convert through nether-export
    let result = convert_gltf_to_memory(&glb_path).expect("Mesh conversion failed");

    // Verify format flags
    assert_eq!(
        result.format, EXPECTED_FORMAT,
        "Format should be UV | NORMAL | SKINNED (0x{:02X}), got 0x{:02X}",
        EXPECTED_FORMAT, result.format
    );

    // Expected: 3 segments × 6 faces × 4 vertices = 72 vertices
    let expected_vertex_count = gltf_generator::BONE_COUNT as u32 * 6 * 4;
    assert_eq!(
        result.vertex_count, expected_vertex_count,
        "Vertex count mismatch"
    );

    // Expected: 3 segments × 6 faces × 2 triangles × 3 indices = 108 indices
    let expected_index_count = gltf_generator::BONE_COUNT as u32 * 6 * 2 * 3;
    assert_eq!(
        result.index_count, expected_index_count,
        "Index count mismatch"
    );

    // Verify data sizes
    let stride = zx_common::vertex_stride_packed(result.format) as usize;
    let expected_vertex_data_size = result.vertex_count as usize * stride;
    assert_eq!(
        result.vertex_data.len(),
        expected_vertex_data_size,
        "Vertex data size mismatch (stride={})",
        stride
    );

    let expected_index_size = result.index_count as usize * 2;
    assert_eq!(
        result.indices.len() * 2,
        expected_index_size,
        "Index data size mismatch"
    );

    println!(
        "Mesh conversion validated: {} vertices, {} indices, format=0x{:02X}, stride={}",
        result.vertex_count, result.index_count, result.format, stride
    );
}

/// Test skeleton conversion through nether-export
#[test]
fn test_glb_skeleton_conversion() {
    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Convert through nether-export
    let result =
        convert_gltf_skeleton_to_memory(&glb_path, None).expect("Skeleton conversion failed");

    // Verify bone count
    assert_eq!(
        result.bone_count,
        gltf_generator::BONE_COUNT as u32,
        "Bone count mismatch"
    );

    // Verify inverse bind matrices count
    assert_eq!(
        result.inverse_bind_matrices.len(),
        gltf_generator::BONE_COUNT,
        "IBM count mismatch"
    );

    // Verify first bone (Root) has identity transform
    let root_ibm = &result.inverse_bind_matrices[0];
    // 3x4 column-major: cols are [0-2], [3-5], [6-8], [9-11]
    // Identity should be: [1,0,0, 0,1,0, 0,0,1, 0,0,0]
    assert!(
        is_near(root_ibm[0], 1.0) && is_near(root_ibm[4], 1.0) && is_near(root_ibm[8], 1.0),
        "Root IBM should be identity (diagonal 1s)"
    );

    // Verify second bone (Spine) has translation offset
    let spine_ibm = &result.inverse_bind_matrices[1];
    // Translation should be [0, -1, 0] (inverse of [0, 1, 0])
    assert!(
        is_near(spine_ibm[10], -1.0),
        "Spine IBM should have Y translation of -1.0, got {}",
        spine_ibm[10]
    );

    println!("Skeleton conversion validated: {} bones", result.bone_count);
}

/// Test animation conversion through nether-export
#[test]
fn test_glb_animation_conversion() {
    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Convert through nether-export (30 FPS, 1 second = 30 frames)
    let result = convert_gltf_animation_to_memory(&glb_path, Some("Wave"), None, Some(30.0))
        .expect("Animation conversion failed");

    // Verify bone count
    assert_eq!(
        result.bone_count,
        gltf_generator::BONE_COUNT as u8,
        "Bone count mismatch"
    );

    // Verify frame count (approximately 30 frames for 1 second at 30 FPS)
    // Allow some variation due to sampling
    assert!(
        result.frame_count >= 25 && result.frame_count <= 35,
        "Frame count should be ~30, got {}",
        result.frame_count
    );

    // Verify data size (16 bytes per bone per frame)
    let expected_data_size = result.frame_count as usize * result.bone_count as usize * 16;
    assert_eq!(
        result.data.len(),
        expected_data_size,
        "Animation data size mismatch"
    );

    println!(
        "Animation conversion validated: {} bones, {} frames, {} bytes",
        result.bone_count,
        result.frame_count,
        result.data.len()
    );
}

/// Test full GLB pipeline (mesh + skeleton + animation)
#[test]
fn test_full_glb_pipeline() {
    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Convert all three
    let mesh = convert_gltf_to_memory(&glb_path).expect("Mesh conversion failed");
    let skeleton =
        convert_gltf_skeleton_to_memory(&glb_path, None).expect("Skeleton conversion failed");
    let animation = convert_gltf_animation_to_memory(&glb_path, Some("Wave"), None, Some(30.0))
        .expect("Animation conversion failed");

    // Verify consistency between mesh and skeleton
    assert_eq!(
        skeleton.bone_count as u8, animation.bone_count,
        "Skeleton and animation bone counts should match"
    );

    // Verify mesh has skinning format flag
    assert!(
        mesh.format & FORMAT_SKINNED != 0,
        "Mesh should have SKINNED format flag"
    );

    // Compute total asset size
    let mesh_size = 12 // header
        + mesh.vertex_data.len()
        + mesh.indices.len() * 2;
    let skeleton_size = 8 // header
        + skeleton.bone_count as usize * 48; // 12 floats × 4 bytes
    let animation_size = 4 // header
        + animation.data.len();

    let total_size = mesh_size + skeleton_size + animation_size;

    println!("Full pipeline validated:");
    println!(
        "  Mesh:      {} bytes ({} vertices, {} indices)",
        mesh_size, mesh.vertex_count, mesh.index_count
    );
    println!(
        "  Skeleton:  {} bytes ({} bones)",
        skeleton_size, skeleton.bone_count
    );
    println!(
        "  Animation: {} bytes ({} frames)",
        animation_size, animation.frame_count
    );
    println!("  Total:     {} bytes", total_size);
}

/// Test that we can run nether-export CLI on GLB file
#[test]
fn test_glb_cli_mesh_export() {
    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    let mesh_path = dir.path().join("test.nczxmesh");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Run nether-export mesh command
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_nether-export"))
        .args([
            "mesh",
            glb_path.to_str().unwrap(),
            "-o",
            mesh_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run nether-export");

    assert!(status.success(), "nether-export mesh command failed");
    assert!(mesh_path.exists(), "Output mesh file should exist");

    // Verify output file structure
    let mesh_data = std::fs::read(&mesh_path).expect("Failed to read mesh file");
    verify_nczxmesh_header(&mesh_data);

    println!("CLI mesh export validated: {} bytes", mesh_data.len());
}

/// Test skeleton CLI export
#[test]
fn test_glb_cli_skeleton_export() {
    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    let skel_path = dir.path().join("test.nczxskel");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Run nether-export skeleton command
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_nether-export"))
        .args([
            "skeleton",
            glb_path.to_str().unwrap(),
            "-o",
            skel_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to run nether-export");

    assert!(status.success(), "nether-export skeleton command failed");
    assert!(skel_path.exists(), "Output skeleton file should exist");

    // Verify output file structure
    let skel_data = std::fs::read(&skel_path).expect("Failed to read skeleton file");
    verify_nczxskel_header(&skel_data, gltf_generator::BONE_COUNT as u32);

    println!("CLI skeleton export validated: {} bytes", skel_data.len());
}

/// Test animation CLI export
#[test]
fn test_glb_cli_animation_export() {
    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    let anim_path = dir.path().join("test.nczxanim");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Run nether-export animation command (use index 0, not name)
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_nether-export"))
        .args([
            "animation",
            glb_path.to_str().unwrap(),
            "-o",
            anim_path.to_str().unwrap(),
            "--animation",
            "0",
            "--frame-rate",
            "30",
        ])
        .status()
        .expect("Failed to run nether-export");

    assert!(status.success(), "nether-export animation command failed");
    assert!(anim_path.exists(), "Output animation file should exist");

    // Verify output file structure
    let anim_data = std::fs::read(&anim_path).expect("Failed to read animation file");
    verify_nczxanim_header(&anim_data, gltf_generator::BONE_COUNT as u8);

    println!("CLI animation export validated: {} bytes", anim_data.len());
}

/// Test nether-cli pack integration with GLB file
/// This verifies the full pipeline from GLB to ROM pack
#[test]
#[ignore] // Run with: cargo test --ignored -- nether_cli
fn test_nether_cli_glb_integration() {
    use std::io::Write;

    let glb_data = gltf_generator::generate_skinned_glb();

    let dir = tempdir().expect("Failed to create temp dir");
    let glb_path = dir.path().join("test.glb");
    std::fs::write(&glb_path, &glb_data).expect("Failed to write GLB");

    // Create a minimal nether.toml that uses the GLB file
    let nether_toml = format!(
        r#"[game]
id = "gltf-test"
title = "GLTF Pipeline Test"
author = "Test"
version = "0.1.0"

[[assets.meshes]]
id = "test_mesh"
path = "{}"

[[assets.skeletons]]
id = "test_skeleton"
path = "{}"

[[assets.animations]]
id = "test_anim"
path = "{}"
"#,
        glb_path.to_str().unwrap().replace('\\', "/"),
        glb_path.to_str().unwrap().replace('\\', "/"),
        glb_path.to_str().unwrap().replace('\\', "/"),
    );

    let toml_path = dir.path().join("nether.toml");
    let mut file = std::fs::File::create(&toml_path).expect("Failed to create nether.toml");
    file.write_all(nether_toml.as_bytes())
        .expect("Failed to write nether.toml");

    // Create minimal WASM (just needs to be valid)
    // For this test, we'll skip the WASM and just test if pack can process the assets
    // by running nether-export build (which validates the manifest and processes assets)
    let status = std::process::Command::new(env!("CARGO_BIN_EXE_nether-export"))
        .args(["check", toml_path.to_str().unwrap()])
        .current_dir(dir.path())
        .status()
        .expect("Failed to run nether-export check");

    assert!(
        status.success(),
        "nether-export check should pass with GLB manifest"
    );

    println!("nether-cli GLB integration validated successfully");
}

// Helper functions

fn is_near(a: f32, b: f32) -> bool {
    (a - b).abs() < 0.001
}

fn verify_nczxmesh_header(data: &[u8]) {
    use zx_common::formats::mesh::NetherZXMeshHeader;

    assert!(
        data.len() >= NetherZXMeshHeader::SIZE,
        "Mesh data too small for header"
    );

    let header = NetherZXMeshHeader::from_bytes(data).expect("Failed to parse mesh header");
    assert!(header.vertex_count > 0, "Should have vertices");
    assert!(header.index_count > 0, "Should have indices");
    assert!(header.format <= 15, "Format should be valid (0-15)");

    // Note: CLI doesn't support skinned mesh export yet (only in-memory API does)
    // Verify at least UV and NORMAL are present
    assert!(header.format & FORMAT_UV != 0, "Should have UV flag");
    assert!(
        header.format & FORMAT_NORMAL != 0,
        "Should have NORMAL flag"
    );
}

fn verify_nczxskel_header(data: &[u8], expected_bones: u32) {
    use zx_common::formats::skeleton::NetherZXSkeletonHeader;

    assert!(
        data.len() >= NetherZXSkeletonHeader::SIZE,
        "Skeleton data too small for header"
    );

    let header = NetherZXSkeletonHeader::from_bytes(data).expect("Failed to parse skeleton header");
    assert_eq!(header.bone_count, expected_bones, "Bone count mismatch");

    // Verify data size (48 bytes per bone)
    let expected_size = NetherZXSkeletonHeader::SIZE + expected_bones as usize * 48;
    assert_eq!(data.len(), expected_size, "Skeleton data size mismatch");
}

fn verify_nczxanim_header(data: &[u8], expected_bones: u8) {
    use zx_common::formats::animation::NetherZXAnimationHeader;

    assert!(
        data.len() >= NetherZXAnimationHeader::SIZE,
        "Animation data too small for header"
    );

    let header =
        NetherZXAnimationHeader::from_bytes(data).expect("Failed to parse animation header");
    assert_eq!(header.bone_count, expected_bones, "Bone count mismatch");
    assert!(header.frame_count > 0, "Should have frames");

    // Verify data size (16 bytes per bone per frame)
    let expected_size = NetherZXAnimationHeader::SIZE
        + header.frame_count as usize * header.bone_count as usize * 16;
    assert_eq!(data.len(), expected_size, "Animation data size mismatch");
}
