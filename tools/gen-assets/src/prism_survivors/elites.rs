//! Elite enemy mesh generation for PRISM SURVIVORS
//!
//! Crystal Knight, Void Mage, Golem Titan, Specter Lord

use crate::mesh_helpers::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    generate_crystal_knight(output_dir);
    generate_void_mage(output_dir);
    generate_golem_titan(output_dir);
    generate_specter_lord(output_dir);
}

fn generate_crystal_knight(output_dir: &Path) {
    // Larger, more imposing knight with crystalline growths
    let mut torso: UnpackedMesh = generate_capsule(0.35, 0.55, 10, 5);
    torso.apply(Transform::scale(1.1, 1.0, 0.9));

    let mut head: UnpackedMesh = generate_sphere(0.2, 10, 8);
    head.apply(Transform::translate(0.0, 0.6, 0.0));

    // Crystal spikes on shoulders
    let mut crystal_r: UnpackedMesh = generate_cylinder(0.0, 0.1, 0.3, 6);
    crystal_r.apply(Transform::translate(0.4, 0.5, 0.0));
    crystal_r.apply(Transform::rotate_z(-30.0));

    let mut crystal_l: UnpackedMesh = generate_cylinder(0.0, 0.1, 0.3, 6);
    crystal_l.apply(Transform::translate(-0.4, 0.5, 0.0));
    crystal_l.apply(Transform::rotate_z(30.0));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.1, 0.1, 0.4, 8);
    arm_r.apply(Transform::translate(0.45, 0.15, 0.0));
    arm_r.apply(Transform::rotate_z(-20.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.1, 0.1, 0.4, 8);
    arm_l.apply(Transform::translate(-0.45, 0.15, 0.0));
    arm_l.apply(Transform::rotate_z(20.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.45, 8);
    leg_r.apply(Transform::translate(0.15, -0.55, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.45, 8);
    leg_l.apply(Transform::translate(-0.15, -0.55, 0.0));

    // Large crystal sword
    let mut sword: UnpackedMesh = generate_cube(0.08, 0.6, 0.04);
    sword.apply(Transform::translate(0.55, 0.3, 0.0));

    let mesh = combine(&[
        &torso, &head, &crystal_r, &crystal_l, &arm_r, &arm_l, &leg_r, &leg_l, &sword,
    ]);
    write_mesh(&mesh, "crystal_knight", output_dir);
}

fn generate_void_mage(output_dir: &Path) {
    // Taller, ethereal mage
    let mut robe: UnpackedMesh = generate_capsule(0.28, 0.7, 10, 5);
    robe.apply(Transform::scale(1.3, 1.0, 1.3));

    let mut head: UnpackedMesh = generate_sphere(0.18, 10, 8);
    head.apply(Transform::translate(0.0, 0.65, 0.0));

    let mut hood: UnpackedMesh = generate_sphere(0.24, 10, 5);
    hood.apply(Transform::translate(0.0, 0.68, -0.03));
    hood.apply(Transform::scale(1.0, 0.85, 1.3));

    // Floating void orbs
    let mut orb1: UnpackedMesh = generate_sphere(0.1, 8, 6);
    orb1.apply(Transform::translate(0.4, 0.4, 0.0));

    let mut orb2: UnpackedMesh = generate_sphere(0.08, 8, 6);
    orb2.apply(Transform::translate(-0.35, 0.5, 0.15));

    let mut orb3: UnpackedMesh = generate_sphere(0.12, 8, 6);
    orb3.apply(Transform::translate(0.0, 0.9, 0.0));

    let mesh = combine(&[&robe, &head, &hood, &orb1, &orb2, &orb3]);
    write_mesh(&mesh, "void_mage", output_dir);
}

fn generate_golem_titan(output_dir: &Path) {
    // Much larger golem with additional mass
    let body: UnpackedMesh = generate_cube(0.7, 0.85, 0.55);

    let mut head: UnpackedMesh = generate_cube(0.35, 0.28, 0.28);
    head.apply(Transform::translate(0.0, 0.6, 0.0));

    // Massive arms
    let mut arm_r: UnpackedMesh = generate_cylinder(0.2, 0.15, 0.6, 8);
    arm_r.apply(Transform::translate(0.55, 0.1, 0.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.2, 0.15, 0.6, 8);
    arm_l.apply(Transform::translate(-0.55, 0.1, 0.0));

    // Fists
    let mut fist_r: UnpackedMesh = generate_sphere(0.2, 8, 6);
    fist_r.apply(Transform::translate(0.55, -0.3, 0.0));

    let mut fist_l: UnpackedMesh = generate_sphere(0.2, 8, 6);
    fist_l.apply(Transform::translate(-0.55, -0.3, 0.0));

    // Thick legs
    let mut leg_r: UnpackedMesh = generate_cylinder(0.2, 0.2, 0.5, 8);
    leg_r.apply(Transform::translate(0.25, -0.7, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.2, 0.2, 0.5, 8);
    leg_l.apply(Transform::translate(-0.25, -0.7, 0.0));

    // Boulder shoulder pads
    let mut shoulder_r: UnpackedMesh = generate_sphere(0.25, 8, 6);
    shoulder_r.apply(Transform::translate(0.45, 0.45, 0.0));

    let mut shoulder_l: UnpackedMesh = generate_sphere(0.25, 8, 6);
    shoulder_l.apply(Transform::translate(-0.45, 0.45, 0.0));

    let mesh = combine(&[
        &body, &head, &arm_r, &arm_l, &fist_r, &fist_l,
        &leg_r, &leg_l, &shoulder_r, &shoulder_l,
    ]);
    write_mesh(&mesh, "golem_titan", output_dir);
}

fn generate_specter_lord(output_dir: &Path) {
    // Ethereal hovering specter
    let core: UnpackedMesh = generate_sphere(0.25, 12, 10);

    // Ghostly tail/train
    let mut tail: UnpackedMesh = generate_sphere(0.2, 10, 8);
    tail.apply(Transform::translate(0.0, -0.3, 0.0));
    tail.apply(Transform::scale(1.0, 2.0, 0.8));

    // Crown/horns
    let mut horn_r: UnpackedMesh = generate_cylinder(0.0, 0.06, 0.2, 5);
    horn_r.apply(Transform::translate(0.15, 0.3, 0.0));
    horn_r.apply(Transform::rotate_z(-20.0));

    let mut horn_l: UnpackedMesh = generate_cylinder(0.0, 0.06, 0.2, 5);
    horn_l.apply(Transform::translate(-0.15, 0.3, 0.0));
    horn_l.apply(Transform::rotate_z(20.0));

    // Floating hands
    let mut hand_r: UnpackedMesh = generate_sphere(0.1, 6, 5);
    hand_r.apply(Transform::translate(0.4, 0.1, 0.0));

    let mut hand_l: UnpackedMesh = generate_sphere(0.1, 6, 5);
    hand_l.apply(Transform::translate(-0.4, 0.1, 0.0));

    let mesh = combine(&[&core, &tail, &horn_r, &horn_l, &hand_r, &hand_l]);
    write_mesh(&mesh, "specter_lord", output_dir);
}
