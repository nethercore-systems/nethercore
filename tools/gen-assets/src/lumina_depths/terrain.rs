//! Terrain and effect generators for LUMINA DEPTHS
//!
//! Rocks, vent chimneys, seafloor patches, and bubble effects.

use proc_gen::mesh::*;
use std::path::Path;

/// Boulder - large rounded rock (~200 tris)
pub fn generate_rock_boulder(output_dir: &Path) {
    println!("  Generating: rock_boulder.obj");

    // Main boulder shape (deformed sphere)
    let mut rock: UnpackedMesh = generate_sphere(0.3, 10, 8);
    rock.apply(Transform::scale(1.2, 0.8, 1.0));

    // Add some irregularity with smaller overlapping spheres
    let mut bump1: UnpackedMesh = generate_sphere(0.12, 6, 4);
    bump1.apply(Transform::translate(0.2, 0.1, 0.1));

    let mut bump2: UnpackedMesh = generate_sphere(0.1, 6, 4);
    bump2.apply(Transform::translate(-0.15, 0.05, 0.18));

    let mut bump3: UnpackedMesh = generate_sphere(0.08, 5, 4);
    bump3.apply(Transform::translate(0.1, -0.1, -0.2));

    let mesh = combine(&[&rock, &bump1, &bump2, &bump3]);

    let path = output_dir.join("rock_boulder.obj");
    write_obj(&mesh, &path, "rock_boulder").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Rock pillar - tall vertical formation (~180 tris)
pub fn generate_rock_pillar(output_dir: &Path) {
    println!("  Generating: rock_pillar.obj");

    // Main column
    let mut pillar: UnpackedMesh = generate_cylinder(0.15, 0.15, 0.8, 8);
    pillar.apply(Transform::translate(0.0, 0.4, 0.0));

    // Tapered top
    let mut top: UnpackedMesh = generate_sphere(0.18, 8, 6);
    top.apply(Transform::scale(1.0, 0.6, 1.0));
    top.apply(Transform::translate(0.0, 0.8, 0.0));

    // Wider base
    let mut base: UnpackedMesh = generate_cylinder(0.22, 0.22, 0.15, 8);
    base.apply(Transform::translate(0.0, 0.075, 0.0));

    // Ledges/ridges
    let mut ledge1: UnpackedMesh = generate_torus(0.18, 0.03, 10, 4);
    ledge1.apply(Transform::translate(0.0, 0.3, 0.0));

    let mut ledge2: UnpackedMesh = generate_torus(0.16, 0.025, 10, 4);
    ledge2.apply(Transform::translate(0.0, 0.55, 0.0));

    let mesh = combine(&[&pillar, &top, &base, &ledge1, &ledge2]);

    let path = output_dir.join("rock_pillar.obj");
    write_obj(&mesh, &path, "rock_pillar").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Hydrothermal vent chimney (~150 tris)
pub fn generate_vent_chimney(output_dir: &Path) {
    println!("  Generating: vent_chimney.obj");

    // Main chimney stack
    let mut stack: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.5, 8);
    stack.apply(Transform::translate(0.0, 0.25, 0.0));

    // Flared top (vent opening)
    let mut top: UnpackedMesh = generate_torus(0.15, 0.05, 10, 4);
    top.apply(Transform::translate(0.0, 0.5, 0.0));

    // Opening cavity
    let mut opening: UnpackedMesh = generate_cylinder(0.1, 0.1, 0.1, 8);
    opening.apply(Transform::translate(0.0, 0.52, 0.0));

    // Mineral deposits on sides
    let mut deposit1: UnpackedMesh = generate_sphere(0.06, 5, 4);
    deposit1.apply(Transform::translate(0.12, 0.2, 0.0));

    let mut deposit2: UnpackedMesh = generate_sphere(0.05, 5, 4);
    deposit2.apply(Transform::translate(-0.1, 0.35, 0.08));

    let mut deposit3: UnpackedMesh = generate_sphere(0.04, 4, 3);
    deposit3.apply(Transform::translate(0.08, 0.4, -0.1));

    // Base mound
    let mut base: UnpackedMesh = generate_sphere(0.2, 8, 4);
    base.apply(Transform::scale(1.0, 0.3, 1.0));
    base.apply(Transform::translate(0.0, 0.02, 0.0));

    let mesh = combine(&[&stack, &top, &opening, &deposit1, &deposit2, &deposit3, &base]);

    let path = output_dir.join("vent_chimney.obj");
    write_obj(&mesh, &path, "vent_chimney").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Seafloor patch - undulating ground surface (~100 tris)
pub fn generate_seafloor_patch(output_dir: &Path) {
    println!("  Generating: seafloor_patch.obj");

    // Base plane with subdivisions for undulation
    let mut floor: UnpackedMesh = generate_plane(2.0, 2.0, 8, 8);
    floor.apply(Transform::rotate_x(-90.0)); // degrees

    // Gentle mounds
    let mut mound1: UnpackedMesh = generate_sphere(0.3, 6, 4);
    mound1.apply(Transform::scale(1.5, 0.15, 1.2));
    mound1.apply(Transform::translate(0.4, 0.0, 0.3));

    let mut mound2: UnpackedMesh = generate_sphere(0.2, 5, 4);
    mound2.apply(Transform::scale(1.3, 0.1, 1.0));
    mound2.apply(Transform::translate(-0.5, 0.0, -0.4));

    let mesh = combine(&[&floor, &mound1, &mound2]);

    let path = output_dir.join("seafloor_patch.obj");
    write_obj(&mesh, &path, "seafloor_patch").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Bubble cluster - group of rising spheres (~30 tris)
pub fn generate_bubble_cluster(output_dir: &Path) {
    println!("  Generating: bubble_cluster.obj");

    // Various sized bubbles
    let bubble_data = [
        (0.0, 0.0, 0.0, 0.03),
        (0.04, 0.05, 0.02, 0.02),
        (-0.03, 0.08, -0.01, 0.025),
        (0.02, 0.12, 0.03, 0.015),
        (-0.02, 0.15, -0.02, 0.02),
        (0.01, 0.18, 0.0, 0.012),
    ];

    let mut bubbles = Vec::new();
    for (x, y, z, radius) in bubble_data {
        let mut bubble: UnpackedMesh = generate_sphere(radius, 6, 4);
        bubble.apply(Transform::translate(x, y, z));
        bubbles.push(bubble);
    }

    let bubble_refs: Vec<&UnpackedMesh> = bubbles.iter().collect();
    let mesh = combine(&bubble_refs);

    let path = output_dir.join("bubble_cluster.obj");
    write_obj(&mesh, &path, "bubble_cluster").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
