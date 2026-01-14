//! Programmatic GLB generation for integration tests.
//!
//! Generates a complete GLB file with:
//! - Skinned mesh (positions, normals, UVs, joints, weights, indices)
//! - 3-bone skeleton with inverse bind matrices
//! - 30-frame wave animation

mod binary_packing;
mod glb_assembly;
mod gltf_json;
mod mesh_data;
mod partial_trs;

pub use mesh_data::BONE_COUNT;

use mesh_data::{create_animation, create_mesh_data, create_skeleton};

/// Generate a complete skinned GLB for testing.
///
/// Contains:
/// - 3 stacked box segments (one per bone)
/// - 3-bone skeleton (Root → Spine → Head)
/// - 30-frame wave animation
pub fn generate_skinned_glb() -> Vec<u8> {
    // Build mesh data
    let mesh = create_mesh_data();
    let skeleton = create_skeleton();
    let animation = create_animation();

    // Pack all binary data
    let (buffer_data, buffer_views, accessors) =
        binary_packing::pack_binary_data(&mesh, &skeleton, &animation);

    // Build GLTF JSON
    let root = gltf_json::build_gltf_json(&buffer_views, &accessors);

    // Assemble GLB
    glb_assembly::assemble_glb(&root, &buffer_data)
}

/// Generate a GLB where bone 1 has only rotation channels (no T, no S).
/// This tests that missing channels use the node's rest pose instead of identity.
///
/// Structure:
/// - Bone 0: At [0, 0, 0], has full T/R/S animation
/// - Bone 1: At [0, 1, 0], has ONLY R animation (no T, no S channels)
/// - Bone 2: At [0, 2, 0], has full T/R/S animation
///
/// Expected behavior: Bone 1's output should have translation [0, 1, 0] for all frames,
/// NOT [0, 0, 0] (which would happen if identity defaults were used).
pub fn generate_partial_trs_glb() -> Vec<u8> {
    partial_trs::generate_partial_trs_glb()
}
