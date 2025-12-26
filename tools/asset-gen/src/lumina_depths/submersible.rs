//! Submersible (player vessel) generator
//!
//! The player controls a small research submersible - their window into the deep.
//! From SHOWCASE_3.md:
//! - Rounded capsule shape hull
//! - Glass bubble observation dome (alpha 12-14)
//! - Forward-facing headlight (emissive)
//! - Small rear thrusters
//! - Thin sensor antenna on top
//! - ~400-500 tris

use proc_gen::mesh::*;
use std::path::Path;

pub fn generate_submersible(output_dir: &Path) {
    println!("Generating: submersible.obj");

    // Main hull - rounded capsule shape
    let mut hull: UnpackedMesh = generate_capsule(0.25, 0.5, 12, 6);
    hull.apply(Transform::rotate_z(90.0));
    hull.apply(Transform::scale(1.0, 0.85, 0.85));

    // Glass observation dome (front hemisphere)
    let mut dome: UnpackedMesh = generate_sphere(0.22, 12, 8);
    dome.apply(Transform::translate(0.35, 0.04, 0.0));
    dome.apply(Transform::scale(0.8, 0.8, 0.9));

    // Headlight housing
    let mut headlight: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.08, 8);
    headlight.apply(Transform::rotate_z(90.0));
    headlight.apply(Transform::translate(0.42, -0.05, 0.0));

    // Headlight lens (emissive when on)
    let mut lens: UnpackedMesh = generate_sphere(0.05, 8, 6);
    lens.apply(Transform::translate(0.47, -0.05, 0.0));

    // Rear thruster pods (2)
    let mut thruster_l: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.15, 6);
    thruster_l.apply(Transform::rotate_z(90.0));
    thruster_l.apply(Transform::translate(-0.4, 0.0, -0.15));

    let mut thruster_r: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.15, 6);
    thruster_r.apply(Transform::rotate_z(90.0));
    thruster_r.apply(Transform::translate(-0.4, 0.0, 0.15));

    // Thruster nozzles (emissive when moving)
    let mut nozzle_l: UnpackedMesh = generate_torus(0.05, 0.015, 8, 4);
    nozzle_l.apply(Transform::rotate_z(90.0));
    nozzle_l.apply(Transform::translate(-0.48, 0.0, -0.15));

    let mut nozzle_r: UnpackedMesh = generate_torus(0.05, 0.015, 8, 4);
    nozzle_r.apply(Transform::rotate_z(90.0));
    nozzle_r.apply(Transform::translate(-0.48, 0.0, 0.15));

    // Top sensor antenna
    let mut antenna_mast: UnpackedMesh = generate_cylinder(0.015, 0.015, 0.15, 4);
    antenna_mast.apply(Transform::translate(0.0, 0.28, 0.0));

    let mut antenna_dish: UnpackedMesh = generate_sphere(0.03, 6, 4);
    antenna_dish.apply(Transform::translate(0.0, 0.35, 0.0));
    antenna_dish.apply(Transform::scale(1.0, 0.5, 1.0));

    // Stabilizer fins (small)
    let mut fin_top: UnpackedMesh = generate_cube(0.08, 0.06, 0.01);
    fin_top.apply(Transform::translate(-0.25, 0.18, 0.0));
    fin_top.apply(Transform::rotate_x(-11.5)); // degrees

    let mut fin_l: UnpackedMesh = generate_cube(0.08, 0.01, 0.05);
    fin_l.apply(Transform::translate(-0.25, 0.0, -0.18));

    let mut fin_r: UnpackedMesh = generate_cube(0.08, 0.01, 0.05);
    fin_r.apply(Transform::translate(-0.25, 0.0, 0.18));

    // Ballast tanks (side pods)
    let mut ballast_l: UnpackedMesh = generate_capsule(0.04, 0.2, 6, 3);
    ballast_l.apply(Transform::rotate_z(90.0));
    ballast_l.apply(Transform::translate(0.0, -0.12, -0.2));

    let mut ballast_r: UnpackedMesh = generate_capsule(0.04, 0.2, 6, 3);
    ballast_r.apply(Transform::rotate_z(90.0));
    ballast_r.apply(Transform::translate(0.0, -0.12, 0.2));

    // Combine all parts
    let mesh = combine(&[
        &hull,
        &dome,
        &headlight,
        &lens,
        &thruster_l,
        &thruster_r,
        &nozzle_l,
        &nozzle_r,
        &antenna_mast,
        &antenna_dish,
        &fin_top,
        &fin_l,
        &fin_r,
        &ballast_l,
        &ballast_r,
    ]);

    let path = output_dir.join("submersible.obj");
    write_obj(&mesh, &path, "submersible").expect("Failed to write OBJ file");
    println!(
        "  -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
