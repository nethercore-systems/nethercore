//! Generate multi-mesh GLB files for glb-rigid example
//!
//! Creates SEPARATE GLB files for each mesh piece:
//! - glb-rigid-base.glb - Base platform
//! - glb-rigid-arm.glb - Extending arm segment
//! - glb-rigid-claw.glb - Claw/gripper end effector
//! - glb-rigid-anim.glb - Animation data (skeleton + keyframes, no mesh)
//!
//! The example demonstrates rigid animation by using keyframe_read() to
//! sample transforms from imported GLTF animation and applying them manually.
//!
//! Usage:
//!   cargo run -p gen-glb-rigid-assets

use anyhow::{Context, Result};
use glb_builder::{
    assemble_glb, AnimationBuilder, BufferBuilder, GltfBuilder, MeshBuilder, SkeletonBuilder,
};
use gltf_json as json;
use std::f32::consts::TAU;
use std::fs;
use std::path::PathBuf;

/// Number of nodes for the rigid hierarchy (Base, Arm, Claw)
const NODE_COUNT: usize = 3;
/// Frame count for animation
const FRAME_COUNT: usize = 60;
/// Animation frame rate
const FRAME_RATE: f32 = 30.0;

fn main() -> Result<()> {
    // Output to shared examples/assets folder
    let output_dir = PathBuf::from("examples/assets");

    // Ensure output directory exists
    fs::create_dir_all(&output_dir).context("Failed to create output directory")?;

    println!("Generating rigid mesh and animation GLBs for glb-rigid example...\n");

    // Generate each mesh piece as a separate GLB
    // Base - flat platform
    let base_path = output_dir.join("glb-rigid-base.glb");
    let base_data = generate_mesh_glb(MeshType::Base);
    fs::write(&base_path, &base_data).context("Failed to write base GLB")?;
    println!(
        "Generated: {} ({} bytes)",
        base_path.display(),
        base_data.len()
    );

    // Arm - elongated segment
    let arm_path = output_dir.join("glb-rigid-arm.glb");
    let arm_data = generate_mesh_glb(MeshType::Arm);
    fs::write(&arm_path, &arm_data).context("Failed to write arm GLB")?;
    println!(
        "Generated: {} ({} bytes)",
        arm_path.display(),
        arm_data.len()
    );

    // Claw - gripper end
    let claw_path = output_dir.join("glb-rigid-claw.glb");
    let claw_data = generate_mesh_glb(MeshType::Claw);
    fs::write(&claw_path, &claw_data).context("Failed to write claw GLB")?;
    println!(
        "Generated: {} ({} bytes)",
        claw_path.display(),
        claw_data.len()
    );

    // Animation GLB - contains skeleton + animation (no mesh)
    let anim_path = output_dir.join("glb-rigid-anim.glb");
    let anim_data = generate_animation_glb();
    fs::write(&anim_path, &anim_data).context("Failed to write animation GLB")?;
    println!(
        "Generated: {} ({} bytes)",
        anim_path.display(),
        anim_data.len()
    );

    println!("\nGenerated 4 GLBs for rigid animation demo:");
    println!("  - glb-rigid-base.glb: rotating platform mesh");
    println!("  - glb-rigid-arm.glb: extending segment mesh");
    println!("  - glb-rigid-claw.glb: opening/closing gripper mesh");
    println!("  - glb-rigid-anim.glb: skeleton + animation keyframes");
    println!("\nAnimation uses keyframe_read() (not skeleton_bind)");

    Ok(())
}

enum MeshType {
    Base,
    Arm,
    Claw,
}

/// Generate a GLB file containing a single mesh piece
fn generate_mesh_glb(mesh_type: MeshType) -> Vec<u8> {
    let (positions, normals, colors, indices, name) = match mesh_type {
        MeshType::Base => create_base_mesh(),
        MeshType::Arm => create_arm_mesh(),
        MeshType::Claw => create_claw_mesh(),
    };

    let mut buffer = BufferBuilder::new();
    let mesh = MeshBuilder::new()
        .positions(&positions)
        .normals(&normals)
        .colors(&colors)
        .indices(&indices)
        .build(&mut buffer);

    // Create node for the mesh
    let nodes = vec![json::Node {
        camera: None,
        children: None,
        extensions: Default::default(),
        extras: Default::default(),
        matrix: None,
        mesh: Some(json::Index::new(0)),
        name: Some(name.clone()),
        rotation: None,
        scale: None,
        skin: None,
        translation: None,
        weights: None,
    }];

    let gltf = GltfBuilder::new()
        .buffer_byte_length(buffer.data().len() as u64)
        .add_nodes(nodes)
        .add_mesh_from_accessors(&name, &mesh)
        .add_scene("Scene", &[0]);

    let root = gltf.build(buffer.views(), buffer.accessors(), "gen-glb-rigid-assets");
    assemble_glb(&root, buffer.data())
}

/// Create base mesh: flat hexagonal platform (origin at center)
fn create_base_mesh() -> (
    Vec<[f32; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 4]>,
    Vec<u16>,
    String,
) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let color = [0.4, 0.4, 0.5, 1.0]; // Gray metallic
    let radius = 0.6;
    let height = 0.15;
    let sides = 6;

    // Top face
    let top_center = positions.len() as u16;
    positions.push([0.0, height, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    colors.push(color);

    for i in 0..sides {
        let angle = (i as f32 / sides as f32) * TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        positions.push([x, height, z]);
        normals.push([0.0, 1.0, 0.0]);
        colors.push(color);
    }

    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.push(top_center);
        indices.push(top_center + 1 + i as u16);
        indices.push(top_center + 1 + next as u16);
    }

    // Bottom face
    let bottom_center = positions.len() as u16;
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    colors.push(color);

    for i in 0..sides {
        let angle = (i as f32 / sides as f32) * TAU;
        let x = angle.cos() * radius;
        let z = angle.sin() * radius;
        positions.push([x, 0.0, z]);
        normals.push([0.0, -1.0, 0.0]);
        colors.push(color);
    }

    for i in 0..sides {
        let next = (i + 1) % sides;
        indices.push(bottom_center);
        indices.push(bottom_center + 1 + next as u16);
        indices.push(bottom_center + 1 + i as u16);
    }

    // Side faces
    for i in 0..sides {
        let next = (i + 1) % sides;
        let angle1 = (i as f32 / sides as f32) * TAU;
        let angle2 = (next as f32 / sides as f32) * TAU;

        let x1 = angle1.cos() * radius;
        let z1 = angle1.sin() * radius;
        let x2 = angle2.cos() * radius;
        let z2 = angle2.sin() * radius;

        // Normal pointing outward (average of the two edge directions)
        let mid_angle = (angle1 + angle2) / 2.0;
        let normal = [mid_angle.cos(), 0.0, mid_angle.sin()];

        let base_idx = positions.len() as u16;

        positions.push([x1, 0.0, z1]);
        positions.push([x2, 0.0, z2]);
        positions.push([x2, height, z2]);
        positions.push([x1, height, z1]);

        for _ in 0..4 {
            normals.push(normal);
            colors.push(color);
        }

        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    (positions, normals, colors, indices, "Base".to_string())
}

/// Create arm mesh: elongated box (origin at pivot point)
fn create_arm_mesh() -> (
    Vec<[f32; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 4]>,
    Vec<u16>,
    String,
) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let color = [0.6, 0.3, 0.2, 1.0]; // Orange-brown
    let width = 0.15;
    let length = 1.0;

    // Box centered at origin in X/Z, extending in +Y
    let faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
        // Front (+Z)
        (
            [0.0, 0.0, 1.0],
            [
                [-width, 0.0, width],
                [width, 0.0, width],
                [width, length, width],
                [-width, length, width],
            ],
        ),
        // Back (-Z)
        (
            [0.0, 0.0, -1.0],
            [
                [width, 0.0, -width],
                [-width, 0.0, -width],
                [-width, length, -width],
                [width, length, -width],
            ],
        ),
        // Right (+X)
        (
            [1.0, 0.0, 0.0],
            [
                [width, 0.0, width],
                [width, 0.0, -width],
                [width, length, -width],
                [width, length, width],
            ],
        ),
        // Left (-X)
        (
            [-1.0, 0.0, 0.0],
            [
                [-width, 0.0, -width],
                [-width, 0.0, width],
                [-width, length, width],
                [-width, length, -width],
            ],
        ),
        // Top (+Y)
        (
            [0.0, 1.0, 0.0],
            [
                [-width, length, width],
                [width, length, width],
                [width, length, -width],
                [-width, length, -width],
            ],
        ),
        // Bottom (-Y)
        (
            [0.0, -1.0, 0.0],
            [
                [-width, 0.0, -width],
                [width, 0.0, -width],
                [width, 0.0, width],
                [-width, 0.0, width],
            ],
        ),
    ];

    for (normal, corners) in &faces {
        let base_idx = positions.len() as u16;
        for corner in corners {
            positions.push(*corner);
            normals.push(*normal);
            colors.push(color);
        }
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    (positions, normals, colors, indices, "Arm".to_string())
}

/// Create claw mesh: two prongs that can open/close (origin at pivot)
fn create_claw_mesh() -> (
    Vec<[f32; 3]>,
    Vec<[f32; 3]>,
    Vec<[f32; 4]>,
    Vec<u16>,
    String,
) {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut colors = Vec::new();
    let mut indices = Vec::new();

    let color = [0.2, 0.5, 0.3, 1.0]; // Green
    let prong_width = 0.08;
    let prong_height = 0.08;
    let prong_length = 0.4;
    let spacing = 0.15;

    // Create two prongs (left and right)
    for offset in [-spacing, spacing] {
        let faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
            // Front (+Z)
            (
                [0.0, 0.0, 1.0],
                [
                    [offset - prong_width, 0.0, prong_height],
                    [offset + prong_width, 0.0, prong_height],
                    [offset + prong_width, prong_length, prong_height],
                    [offset - prong_width, prong_length, prong_height],
                ],
            ),
            // Back (-Z)
            (
                [0.0, 0.0, -1.0],
                [
                    [offset + prong_width, 0.0, -prong_height],
                    [offset - prong_width, 0.0, -prong_height],
                    [offset - prong_width, prong_length, -prong_height],
                    [offset + prong_width, prong_length, -prong_height],
                ],
            ),
            // Right (+X)
            (
                [1.0, 0.0, 0.0],
                [
                    [offset + prong_width, 0.0, prong_height],
                    [offset + prong_width, 0.0, -prong_height],
                    [offset + prong_width, prong_length, -prong_height],
                    [offset + prong_width, prong_length, prong_height],
                ],
            ),
            // Left (-X)
            (
                [-1.0, 0.0, 0.0],
                [
                    [offset - prong_width, 0.0, -prong_height],
                    [offset - prong_width, 0.0, prong_height],
                    [offset - prong_width, prong_length, prong_height],
                    [offset - prong_width, prong_length, -prong_height],
                ],
            ),
            // Top (+Y at end of prong)
            (
                [0.0, 1.0, 0.0],
                [
                    [offset - prong_width, prong_length, prong_height],
                    [offset + prong_width, prong_length, prong_height],
                    [offset + prong_width, prong_length, -prong_height],
                    [offset - prong_width, prong_length, -prong_height],
                ],
            ),
            // Bottom (-Y at base)
            (
                [0.0, -1.0, 0.0],
                [
                    [offset - prong_width, 0.0, -prong_height],
                    [offset + prong_width, 0.0, -prong_height],
                    [offset + prong_width, 0.0, prong_height],
                    [offset - prong_width, 0.0, prong_height],
                ],
            ),
        ];

        for (normal, corners) in &faces {
            let base_idx = positions.len() as u16;
            for corner in corners {
                positions.push(*corner);
                normals.push(*normal);
                colors.push(color);
            }
            indices.push(base_idx);
            indices.push(base_idx + 1);
            indices.push(base_idx + 2);
            indices.push(base_idx);
            indices.push(base_idx + 2);
            indices.push(base_idx + 3);
        }
    }

    // Add a connecting bridge at the base
    let bridge_width = spacing + prong_width;
    let bridge_height = 0.06;
    let bridge_depth = prong_height;

    let bridge_faces: Vec<([f32; 3], [[f32; 3]; 4])> = vec![
        // Front (+Z)
        (
            [0.0, 0.0, 1.0],
            [
                [-bridge_width, -bridge_height, bridge_depth],
                [bridge_width, -bridge_height, bridge_depth],
                [bridge_width, 0.0, bridge_depth],
                [-bridge_width, 0.0, bridge_depth],
            ],
        ),
        // Back (-Z)
        (
            [0.0, 0.0, -1.0],
            [
                [bridge_width, -bridge_height, -bridge_depth],
                [-bridge_width, -bridge_height, -bridge_depth],
                [-bridge_width, 0.0, -bridge_depth],
                [bridge_width, 0.0, -bridge_depth],
            ],
        ),
        // Bottom
        (
            [0.0, -1.0, 0.0],
            [
                [-bridge_width, -bridge_height, -bridge_depth],
                [bridge_width, -bridge_height, -bridge_depth],
                [bridge_width, -bridge_height, bridge_depth],
                [-bridge_width, -bridge_height, bridge_depth],
            ],
        ),
    ];

    for (normal, corners) in &bridge_faces {
        let base_idx = positions.len() as u16;
        for corner in corners {
            positions.push(*corner);
            normals.push(*normal);
            colors.push(color);
        }
        indices.push(base_idx);
        indices.push(base_idx + 1);
        indices.push(base_idx + 2);
        indices.push(base_idx);
        indices.push(base_idx + 2);
        indices.push(base_idx + 3);
    }

    (positions, normals, colors, indices, "Claw".to_string())
}

// ============================================================================
// Animation GLB Generation (skeleton + keyframes, no mesh)
// ============================================================================

/// Generate animation GLB with skeleton + keyframes for rigid animation
fn generate_animation_glb() -> Vec<u8> {
    let mut buffer = BufferBuilder::new();

    // Build skeleton with identity inverse bind matrices
    #[rustfmt::skip]
    let identity: [f32; 16] = [
        1.0, 0.0, 0.0, 0.0,
        0.0, 1.0, 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0,
    ];
    let skeleton = SkeletonBuilder::new()
        .inverse_bind_matrices(&[identity; NODE_COUNT])
        .build(&mut buffer);

    // Build animation
    let animation = create_operate_animation(&mut buffer);

    // Create nodes: 3 nodes representing Base, Arm, Claw
    let nodes = vec![
        json::Node {
            name: Some("Base".to_string()),
            mesh: None,
            skin: None,
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
        },
        json::Node {
            name: Some("Arm".to_string()),
            mesh: None,
            skin: None,
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
        },
        json::Node {
            name: Some("Claw".to_string()),
            mesh: None,
            skin: None,
            camera: None,
            children: None,
            extensions: Default::default(),
            extras: Default::default(),
            matrix: None,
            rotation: None,
            scale: None,
            translation: None,
            weights: None,
        },
    ];

    let bone_node_indices: Vec<u32> = (0..NODE_COUNT as u32).collect();

    let gltf = GltfBuilder::new()
        .buffer_byte_length(buffer.data().len() as u64)
        .add_nodes(nodes)
        .add_skin("RigidSkeleton", 0, &bone_node_indices, &skeleton)
        .add_animation("Operate", &bone_node_indices, &animation)
        .add_scene("Scene", &[0, 1, 2]);

    let root = gltf.build(buffer.views(), buffer.accessors(), "gen-glb-rigid-assets");
    assemble_glb(&root, buffer.data())
}

/// Create "Operate" animation for the mechanical arm
/// Node 0 (Base): Rotates around Y
/// Node 1 (Arm): Tilts (X rotation) and extends (Z translation)
/// Node 2 (Claw): Opens/closes (Y translation offset)
fn create_operate_animation(buffer: &mut BufferBuilder) -> glb_builder::AnimationAccessors {
    let duration = FRAME_COUNT as f32 / FRAME_RATE;

    let mut times = Vec::new();
    let mut translations: Vec<Vec<[f32; 3]>> = vec![Vec::new(); NODE_COUNT];
    let mut rotations: Vec<Vec<[f32; 4]>> = vec![Vec::new(); NODE_COUNT];
    let mut scales: Vec<Vec<[f32; 3]>> = vec![Vec::new(); NODE_COUNT];

    for frame in 0..FRAME_COUNT {
        let t = frame as f32 / FRAME_COUNT as f32;
        times.push(t * duration);

        let anim_time = t * TAU;

        // Node 0: Base - rotates around Y axis
        {
            let rotation_y = anim_time * 0.5; // Slow rotation
            let half = rotation_y * 0.5;
            translations[0].push([0.0, 0.0, 0.0]);
            rotations[0].push([0.0, half.sin(), 0.0, half.cos()]);
            scales[0].push([1.0, 1.0, 1.0]);
        }

        // Node 1: Arm - tilts and extends
        {
            // Tilt angle (X rotation)
            let tilt = (anim_time * 1.2).sin() * 0.4;
            let half_tilt = tilt * 0.5;

            // Extension (Z translation)
            let extension = ((anim_time * 1.5).sin() + 1.0) * 0.3;

            // Position relative to base attachment point
            translations[1].push([0.0, 0.5, extension]);
            rotations[1].push([half_tilt.sin(), 0.0, 0.0, half_tilt.cos()]);
            scales[1].push([1.0, 1.0, 1.0]);
        }

        // Node 2: Claw - positioned at end of arm, opens/closes
        {
            // Claw opening (spread in X)
            let open_amount = ((anim_time * 2.0).sin() + 1.0) * 0.15;

            // Position at end of arm segment
            translations[2].push([open_amount, 1.0, 0.0]);
            rotations[2].push([0.0, 0.0, 0.0, 1.0]); // Identity rotation
            scales[2].push([1.0, 1.0, 1.0]);
        }
    }

    AnimationBuilder::new(NODE_COUNT)
        .times(&times)
        .all_translations(translations)
        .all_rotations(rotations)
        .all_scales(scales)
        .build(buffer)
}
