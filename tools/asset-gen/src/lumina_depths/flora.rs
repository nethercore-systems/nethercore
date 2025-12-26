//! Flora generators for LUMINA DEPTHS
//!
//! Coral, kelp, anemones, and sea grass for underwater environments.

use proc_gen::mesh::*;
use std::path::Path;

/// Brain coral - rounded bumpy surface (~150 tris)
pub fn generate_coral_brain(output_dir: &Path) {
    println!("  Generating: coral_brain.obj");

    // Main dome
    let mut dome: UnpackedMesh = generate_sphere(0.2, 12, 8);
    dome.apply(Transform::scale(1.0, 0.7, 1.0));

    // Bumpy ridges (simplified as small spheres)
    let mut ridges = Vec::new();
    for i in 0..8 {
        let angle = (i as f32) * 45.0;
        let angle_rad = angle.to_radians();
        let mut ridge: UnpackedMesh = generate_sphere(0.04, 4, 3);
        ridge.apply(Transform::translate(
            angle_rad.cos() * 0.12,
            0.08,
            angle_rad.sin() * 0.12,
        ));
        ridges.push(ridge);
    }

    // Base attachment
    let mut base: UnpackedMesh = generate_cylinder(0.08, 0.08, 0.05, 6);
    base.apply(Transform::translate(0.0, -0.08, 0.0));

    let ridge_refs: Vec<&UnpackedMesh> = ridges.iter().collect();
    let mut parts = vec![&dome, &base];
    parts.extend(ridge_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("coral_brain.obj");
    write_obj(&mesh, &path, "coral_brain").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Fan coral - flat fan shape (~100 tris)
pub fn generate_coral_fan(output_dir: &Path) {
    println!("  Generating: coral_fan.obj");

    // Fan surface (flattened sphere)
    let mut fan: UnpackedMesh = generate_sphere(0.2, 10, 8);
    fan.apply(Transform::scale(1.0, 1.2, 0.1));
    fan.apply(Transform::translate(0.0, 0.15, 0.0));

    // Stem
    let mut stem: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.1, 6);
    stem.apply(Transform::translate(0.0, 0.0, 0.0));

    // Base holdfast
    let mut base: UnpackedMesh = generate_sphere(0.04, 5, 4);
    base.apply(Transform::scale(1.5, 0.5, 1.5));
    base.apply(Transform::translate(0.0, -0.05, 0.0));

    let mesh = combine(&[&fan, &stem, &base]);

    let path = output_dir.join("coral_fan.obj");
    write_obj(&mesh, &path, "coral_fan").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Branching coral - tree-like structure (~180 tris)
pub fn generate_coral_branch(output_dir: &Path) {
    println!("  Generating: coral_branch.obj");

    // Main trunk
    let mut trunk: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.2, 6);
    trunk.apply(Transform::translate(0.0, 0.1, 0.0));

    // Primary branches
    let mut branch1: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.12, 5);
    branch1.apply(Transform::rotate_z(-29.0)); // degrees
    branch1.apply(Transform::translate(0.05, 0.18, 0.0));

    let mut branch2: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.1, 5);
    branch2.apply(Transform::rotate_z(34.0));
    branch2.apply(Transform::translate(-0.04, 0.15, 0.02));

    let mut branch3: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.11, 5);
    branch3.apply(Transform::rotate_x(-29.0));
    branch3.apply(Transform::translate(0.0, 0.17, 0.04));

    // Secondary twigs
    let mut twig1: UnpackedMesh = generate_cylinder(0.01, 0.01, 0.06, 4);
    twig1.apply(Transform::rotate_z(-46.0));
    twig1.apply(Transform::translate(0.1, 0.22, 0.0));

    let mut twig2: UnpackedMesh = generate_cylinder(0.01, 0.01, 0.05, 4);
    twig2.apply(Transform::rotate_z(40.0));
    twig2.apply(Transform::translate(-0.06, 0.2, 0.0));

    // Tips (polyps)
    let mut tip1: UnpackedMesh = generate_sphere(0.015, 4, 3);
    tip1.apply(Transform::translate(0.12, 0.26, 0.0));

    let mut tip2: UnpackedMesh = generate_sphere(0.015, 4, 3);
    tip2.apply(Transform::translate(-0.08, 0.24, 0.0));

    let mut tip3: UnpackedMesh = generate_sphere(0.015, 4, 3);
    tip3.apply(Transform::translate(0.0, 0.25, 0.08));

    // Base
    let mut base: UnpackedMesh = generate_cylinder(0.05, 0.05, 0.02, 6);

    let mesh = combine(&[
        &trunk, &branch1, &branch2, &branch3, &twig1, &twig2, &tip1, &tip2, &tip3, &base,
    ]);

    let path = output_dir.join("coral_branch.obj");
    write_obj(&mesh, &path, "coral_branch").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Kelp stalk - tall swaying plant (~80 tris)
pub fn generate_kelp(output_dir: &Path) {
    println!("  Generating: kelp.obj");

    // Long stipe (stem)
    let mut stipe: UnpackedMesh = generate_cylinder(0.015, 0.015, 0.6, 6);
    stipe.apply(Transform::translate(0.0, 0.3, 0.0));

    // Blades (leaves) along the stipe
    let mut blades = Vec::new();
    for i in 0..5 {
        let y = 0.15 + (i as f32) * 0.12;
        let angle = (i as f32) * 68.75; // degrees

        let mut blade: UnpackedMesh = generate_cube(0.08, 0.02, 0.15);
        blade.apply(Transform::rotate_y(angle));
        blade.apply(Transform::rotate_z(23.0));
        blade.apply(Transform::translate(0.0, y, 0.0));
        blades.push(blade);
    }

    // Holdfast (root structure)
    let mut holdfast: UnpackedMesh = generate_sphere(0.04, 6, 4);
    holdfast.apply(Transform::scale(1.5, 0.5, 1.5));

    // Float bladder at top
    let mut bladder: UnpackedMesh = generate_sphere(0.025, 5, 4);
    bladder.apply(Transform::translate(0.0, 0.62, 0.0));

    let blade_refs: Vec<&UnpackedMesh> = blades.iter().collect();
    let mut parts = vec![&stipe, &holdfast, &bladder];
    parts.extend(blade_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("kelp.obj");
    write_obj(&mesh, &path, "kelp").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Sea anemone - cylinder with tentacles (~120 tris)
pub fn generate_anemone(output_dir: &Path) {
    println!("  Generating: anemone.obj");

    // Column (body)
    let mut column: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.1, 8);
    column.apply(Transform::translate(0.0, 0.05, 0.0));

    // Oral disc
    let mut disc: UnpackedMesh = generate_sphere(0.07, 8, 4);
    disc.apply(Transform::scale(1.0, 0.3, 1.0));
    disc.apply(Transform::translate(0.0, 0.1, 0.0));

    // Tentacles (ring of cylinders)
    let mut tentacles = Vec::new();
    for i in 0..12 {
        let angle = (i as f32) * 30.0;
        let angle_rad = angle.to_radians();
        let mut tent: UnpackedMesh = generate_cylinder(0.008, 0.008, 0.08, 4);
        tent.apply(Transform::rotate_z(-23.0));
        tent.apply(Transform::rotate_y(angle));
        tent.apply(Transform::translate(angle_rad.cos() * 0.05, 0.14, angle_rad.sin() * 0.05));
        tentacles.push(tent);
    }

    // Base (pedal disc)
    let mut base: UnpackedMesh = generate_cylinder(0.08, 0.08, 0.02, 8);

    let tent_refs: Vec<&UnpackedMesh> = tentacles.iter().collect();
    let mut parts = vec![&column, &disc, &base];
    parts.extend(tent_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("anemone.obj");
    write_obj(&mesh, &path, "anemone").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Sea grass - simple grass blades (~50 tris)
pub fn generate_sea_grass(output_dir: &Path) {
    println!("  Generating: sea_grass.obj");

    // Cluster of grass blades
    let mut blades = Vec::new();
    let positions: [(f32, f32, f32); 5] = [
        (0.0, 0.0, 0.0),
        (0.02, 0.0, 0.015),
        (-0.015, 0.0, 0.02),
        (0.01, 0.0, -0.02),
        (-0.02, 0.0, -0.01),
    ];

    for (x, _, z) in positions {
        let height = 0.12 + x.abs() * 0.5;
        let mut blade: UnpackedMesh = generate_cube(0.005, height, 0.015);
        blade.apply(Transform::rotate_z(x * 114.6)); // degrees - approximate conversion
        blade.apply(Transform::translate(x, height / 2.0, z));
        blades.push(blade);
    }

    let blade_refs: Vec<&UnpackedMesh> = blades.iter().collect();
    let mesh = combine(&blade_refs);

    let path = output_dir.join("sea_grass.obj");
    write_obj(&mesh, &path, "sea_grass").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
