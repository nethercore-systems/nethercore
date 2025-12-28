//! Boss mesh generation for PRISM SURVIVORS
//!
//! Prism Colossus, Void Dragon

use crate::mesh_helpers::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    generate_prism_colossus(output_dir);
    generate_void_dragon(output_dir);
}

fn generate_prism_colossus(output_dir: &Path) {
    // Massive crystalline humanoid
    let core: UnpackedMesh = generate_cube(1.0, 1.4, 0.8);

    // Central prism chest
    let mut prism: UnpackedMesh = generate_cylinder(0.0, 0.5, 0.6, 8);
    prism.apply(Transform::translate(0.0, 0.3, 0.3));
    prism.apply(Transform::rotate_x(-30.0));

    // Head - angular
    let mut head: UnpackedMesh = generate_cube(0.45, 0.35, 0.35);
    head.apply(Transform::translate(0.0, 0.95, 0.0));

    // Massive arms made of crystal
    let mut arm_r: UnpackedMesh = generate_cube(0.3, 0.8, 0.25);
    arm_r.apply(Transform::translate(0.7, 0.2, 0.0));

    let mut arm_l: UnpackedMesh = generate_cube(0.3, 0.8, 0.25);
    arm_l.apply(Transform::translate(-0.7, 0.2, 0.0));

    // Crystal spikes on back
    let mut spike1: UnpackedMesh = generate_cylinder(0.0, 0.2, 0.5, 6);
    spike1.apply(Transform::translate(0.3, 0.5, -0.4));
    spike1.apply(Transform::rotate_x(45.0));

    let mut spike2: UnpackedMesh = generate_cylinder(0.0, 0.2, 0.6, 6);
    spike2.apply(Transform::translate(-0.3, 0.6, -0.35));
    spike2.apply(Transform::rotate_x(40.0));

    let mut spike3: UnpackedMesh = generate_cylinder(0.0, 0.25, 0.7, 6);
    spike3.apply(Transform::translate(0.0, 0.7, -0.45));
    spike3.apply(Transform::rotate_x(50.0));

    // Thick legs
    let mut leg_r: UnpackedMesh = generate_cube(0.35, 0.7, 0.3);
    leg_r.apply(Transform::translate(0.35, -1.0, 0.0));

    let mut leg_l: UnpackedMesh = generate_cube(0.35, 0.7, 0.3);
    leg_l.apply(Transform::translate(-0.35, -1.0, 0.0));

    let mesh = combine(&[
        &core, &prism, &head, &arm_r, &arm_l,
        &spike1, &spike2, &spike3, &leg_r, &leg_l,
    ]);
    write_mesh(&mesh, "prism_colossus", output_dir);
}

fn generate_void_dragon(output_dir: &Path) {
    // Body
    let mut body: UnpackedMesh = generate_sphere(0.8, 12, 10);
    body.apply(Transform::scale(1.5, 0.8, 1.0));

    // Neck and head
    let mut neck: UnpackedMesh = generate_cylinder(0.25, 0.15, 0.6, 8);
    neck.apply(Transform::translate(0.9, 0.2, 0.0));
    neck.apply(Transform::rotate_z(-45.0));

    let mut head: UnpackedMesh = generate_cube(0.5, 0.3, 0.25);
    head.apply(Transform::translate(1.3, 0.55, 0.0));

    // Horns
    let mut horn_r: UnpackedMesh = generate_cylinder(0.0, 0.08, 0.25, 5);
    horn_r.apply(Transform::translate(1.35, 0.75, 0.12));
    horn_r.apply(Transform::rotate_z(-30.0));

    let mut horn_l: UnpackedMesh = generate_cylinder(0.0, 0.08, 0.25, 5);
    horn_l.apply(Transform::translate(1.35, 0.75, -0.12));
    horn_l.apply(Transform::rotate_z(-30.0));

    // Wings (simplified as large flat shapes)
    let mut wing_r: UnpackedMesh = generate_cube(0.8, 0.6, 0.05);
    wing_r.apply(Transform::translate(0.0, 0.3, 0.7));
    wing_r.apply(Transform::rotate_x(30.0));

    let mut wing_l: UnpackedMesh = generate_cube(0.8, 0.6, 0.05);
    wing_l.apply(Transform::translate(0.0, 0.3, -0.7));
    wing_l.apply(Transform::rotate_x(-30.0));

    // Tail
    let mut tail: UnpackedMesh = generate_cylinder(0.2, 0.05, 1.0, 8);
    tail.apply(Transform::translate(-1.2, 0.0, 0.0));
    tail.apply(Transform::rotate_z(15.0));

    // Legs
    let mut leg_fr: UnpackedMesh = generate_cylinder(0.15, 0.1, 0.4, 6);
    leg_fr.apply(Transform::translate(0.5, -0.5, 0.4));

    let mut leg_fl: UnpackedMesh = generate_cylinder(0.15, 0.1, 0.4, 6);
    leg_fl.apply(Transform::translate(0.5, -0.5, -0.4));

    let mut leg_br: UnpackedMesh = generate_cylinder(0.18, 0.12, 0.45, 6);
    leg_br.apply(Transform::translate(-0.4, -0.55, 0.35));

    let mut leg_bl: UnpackedMesh = generate_cylinder(0.18, 0.12, 0.45, 6);
    leg_bl.apply(Transform::translate(-0.4, -0.55, -0.35));

    let mesh = combine(&[
        &body, &neck, &head, &horn_r, &horn_l, &wing_r, &wing_l,
        &tail, &leg_fr, &leg_fl, &leg_br, &leg_bl,
    ]);
    write_mesh(&mesh, "void_dragon", output_dir);
}
