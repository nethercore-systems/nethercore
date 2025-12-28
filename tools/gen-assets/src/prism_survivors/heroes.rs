//! Hero mesh generation for PRISM SURVIVORS
//!
//! Knight, Mage, Ranger, Cleric, Necromancer, Paladin

use crate::mesh_helpers::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    generate_knight(output_dir);
    generate_mage(output_dir);
    generate_ranger(output_dir);
    generate_cleric(output_dir);
    generate_necromancer(output_dir);
    generate_paladin(output_dir);
}

fn generate_knight(output_dir: &Path) {
    let mut torso: UnpackedMesh = generate_capsule(0.25, 0.4, 8, 4);
    torso.apply(Transform::scale(1.0, 1.0, 0.8));

    let mut head: UnpackedMesh = generate_sphere(0.15, 8, 6);
    head.apply(Transform::translate(0.0, 0.5, 0.0));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.08, 0.08, 0.3, 6);
    arm_r.apply(Transform::translate(0.35, 0.15, 0.0));
    arm_r.apply(Transform::rotate_z(-17.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.08, 0.08, 0.3, 6);
    arm_l.apply(Transform::translate(-0.35, 0.15, 0.0));
    arm_l.apply(Transform::rotate_z(17.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.1, 0.1, 0.35, 6);
    leg_r.apply(Transform::translate(0.12, -0.45, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.1, 0.1, 0.35, 6);
    leg_l.apply(Transform::translate(-0.12, -0.45, 0.0));

    let mut shield: UnpackedMesh = generate_cube(0.25, 0.35, 0.05);
    shield.apply(Transform::translate(-0.45, 0.1, 0.0));

    let mesh = combine(&[&torso, &head, &arm_r, &arm_l, &leg_r, &leg_l, &shield]);
    write_mesh(&mesh, "knight", output_dir);
}

fn generate_mage(output_dir: &Path) {
    let mut robe: UnpackedMesh = generate_capsule(0.2, 0.5, 8, 4);
    robe.apply(Transform::scale(1.2, 1.0, 1.2));

    let mut head: UnpackedMesh = generate_sphere(0.14, 8, 6);
    head.apply(Transform::translate(0.0, 0.5, 0.0));

    let mut hood: UnpackedMesh = generate_sphere(0.18, 8, 4);
    hood.apply(Transform::translate(0.0, 0.52, -0.02));
    hood.apply(Transform::scale(1.0, 0.8, 1.2));

    let mut staff: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.9, 6);
    staff.apply(Transform::translate(0.35, 0.1, 0.0));

    let mut orb: UnpackedMesh = generate_sphere(0.08, 8, 6);
    orb.apply(Transform::translate(0.35, 0.6, 0.0));

    let mesh = combine(&[&robe, &head, &hood, &staff, &orb]);
    write_mesh(&mesh, "mage", output_dir);
}

fn generate_ranger(output_dir: &Path) {
    let mut torso: UnpackedMesh = generate_capsule(0.18, 0.35, 8, 4);
    torso.apply(Transform::scale(0.9, 1.0, 0.7));

    let mut head: UnpackedMesh = generate_sphere(0.12, 8, 6);
    head.apply(Transform::translate(0.0, 0.42, 0.0));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.28, 6);
    arm_r.apply(Transform::translate(0.28, 0.1, 0.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.28, 6);
    arm_l.apply(Transform::translate(-0.28, 0.1, 0.0));
    arm_l.apply(Transform::rotate_z(29.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.07, 0.07, 0.32, 6);
    leg_r.apply(Transform::translate(0.1, -0.4, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.07, 0.07, 0.32, 6);
    leg_l.apply(Transform::translate(-0.1, -0.4, 0.0));

    let mut bow: UnpackedMesh = generate_torus(0.25, 0.02, 12, 4);
    bow.apply(Transform::translate(-0.4, 0.15, 0.0));
    bow.apply(Transform::scale(0.5, 1.0, 0.3));

    let mut quiver: UnpackedMesh = generate_cylinder(0.06, 0.06, 0.3, 6);
    quiver.apply(Transform::translate(0.15, 0.1, -0.15));

    let mesh = combine(&[&torso, &head, &arm_r, &arm_l, &leg_r, &leg_l, &bow, &quiver]);
    write_mesh(&mesh, "ranger", output_dir);
}

fn generate_cleric(output_dir: &Path) {
    let torso: UnpackedMesh = generate_capsule(0.22, 0.4, 8, 4);

    let mut head: UnpackedMesh = generate_sphere(0.13, 8, 6);
    head.apply(Transform::translate(0.0, 0.48, 0.0));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.07, 0.07, 0.26, 6);
    arm_r.apply(Transform::translate(0.3, 0.12, 0.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.07, 0.07, 0.26, 6);
    arm_l.apply(Transform::translate(-0.3, 0.12, 0.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.08, 0.08, 0.3, 6);
    leg_r.apply(Transform::translate(0.1, -0.42, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.08, 0.08, 0.3, 6);
    leg_l.apply(Transform::translate(-0.1, -0.42, 0.0));

    let mut staff: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.8, 6);
    staff.apply(Transform::translate(0.35, 0.05, 0.0));

    let mut cross_v: UnpackedMesh = generate_cube(0.04, 0.12, 0.02);
    cross_v.apply(Transform::translate(0.35, 0.5, 0.0));

    let mut cross_h: UnpackedMesh = generate_cube(0.08, 0.03, 0.02);
    cross_h.apply(Transform::translate(0.35, 0.52, 0.0));

    let mesh = combine(&[
        &torso, &head, &arm_r, &arm_l, &leg_r, &leg_l, &staff, &cross_v, &cross_h,
    ]);
    write_mesh(&mesh, "cleric", output_dir);
}

fn generate_necromancer(output_dir: &Path) {
    // Dark robed figure with floating skulls
    let mut robe: UnpackedMesh = generate_capsule(0.22, 0.55, 8, 4);
    robe.apply(Transform::scale(1.3, 1.0, 1.3));

    let mut head: UnpackedMesh = generate_sphere(0.14, 8, 6);
    head.apply(Transform::translate(0.0, 0.55, 0.0));

    let mut hood: UnpackedMesh = generate_sphere(0.2, 8, 4);
    hood.apply(Transform::translate(0.0, 0.58, -0.03));
    hood.apply(Transform::scale(1.0, 0.9, 1.3));

    // Bone staff
    let mut staff: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.85, 6);
    staff.apply(Transform::translate(0.35, 0.05, 0.0));

    // Skull on staff
    let mut skull: UnpackedMesh = generate_sphere(0.08, 6, 5);
    skull.apply(Transform::translate(0.35, 0.55, 0.0));
    skull.apply(Transform::scale(1.0, 1.2, 0.9));

    // Floating orbiting skulls
    let mut skull1: UnpackedMesh = generate_sphere(0.06, 6, 4);
    skull1.apply(Transform::translate(-0.35, 0.4, 0.2));

    let mut skull2: UnpackedMesh = generate_sphere(0.06, 6, 4);
    skull2.apply(Transform::translate(-0.3, 0.5, -0.25));

    let mesh = combine(&[&robe, &head, &hood, &staff, &skull, &skull1, &skull2]);
    write_mesh(&mesh, "necromancer", output_dir);
}

fn generate_paladin(output_dir: &Path) {
    // Heavy armored holy warrior
    let mut torso: UnpackedMesh = generate_capsule(0.3, 0.5, 10, 5);
    torso.apply(Transform::scale(1.1, 1.0, 0.9));

    let mut head: UnpackedMesh = generate_sphere(0.16, 8, 6);
    head.apply(Transform::translate(0.0, 0.55, 0.0));

    // Helmet crest
    let mut crest: UnpackedMesh = generate_cube(0.02, 0.12, 0.15);
    crest.apply(Transform::translate(0.0, 0.7, 0.0));

    // Large shoulder pauldrons
    let mut shoulder_r: UnpackedMesh = generate_sphere(0.15, 6, 5);
    shoulder_r.apply(Transform::translate(0.35, 0.35, 0.0));
    shoulder_r.apply(Transform::scale(1.2, 0.8, 1.0));

    let mut shoulder_l: UnpackedMesh = generate_sphere(0.15, 6, 5);
    shoulder_l.apply(Transform::translate(-0.35, 0.35, 0.0));
    shoulder_l.apply(Transform::scale(1.2, 0.8, 1.0));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.09, 0.09, 0.32, 6);
    arm_r.apply(Transform::translate(0.38, 0.1, 0.0));
    arm_r.apply(Transform::rotate_z(-15.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.09, 0.09, 0.32, 6);
    arm_l.apply(Transform::translate(-0.38, 0.1, 0.0));
    arm_l.apply(Transform::rotate_z(15.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.11, 0.11, 0.38, 6);
    leg_r.apply(Transform::translate(0.14, -0.5, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.11, 0.11, 0.38, 6);
    leg_l.apply(Transform::translate(-0.14, -0.5, 0.0));

    // Large hammer
    let mut hammer_handle: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.7, 6);
    hammer_handle.apply(Transform::translate(0.5, 0.2, 0.0));

    let mut hammer_head: UnpackedMesh = generate_cube(0.2, 0.15, 0.12);
    hammer_head.apply(Transform::translate(0.5, 0.6, 0.0));

    // Tower shield
    let mut shield: UnpackedMesh = generate_cube(0.3, 0.45, 0.05);
    shield.apply(Transform::translate(-0.5, 0.1, 0.0));

    let mesh = combine(&[
        &torso, &head, &crest, &shoulder_r, &shoulder_l, &arm_r, &arm_l,
        &leg_r, &leg_l, &hammer_handle, &hammer_head, &shield,
    ]);
    write_mesh(&mesh, "paladin", output_dir);
}
