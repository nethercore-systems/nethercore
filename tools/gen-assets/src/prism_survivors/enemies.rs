//! Basic enemy mesh generation for PRISM SURVIVORS
//!
//! Golem, Crawler, Wisp, Skeleton, Shade, Berserker, Arcane Sentinel

use crate::mesh_helpers::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    generate_golem(output_dir);
    generate_crawler(output_dir);
    generate_wisp(output_dir);
    generate_skeleton(output_dir);
    generate_shade(output_dir);
    generate_berserker(output_dir);
    generate_arcane_sentinel(output_dir);
}

fn generate_golem(output_dir: &Path) {
    let body: UnpackedMesh = generate_cube(0.5, 0.6, 0.4);

    let mut head: UnpackedMesh = generate_cube(0.25, 0.2, 0.2);
    head.apply(Transform::translate(0.0, 0.45, 0.0));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.4, 6);
    arm_r.apply(Transform::translate(0.4, 0.1, 0.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.4, 6);
    arm_l.apply(Transform::translate(-0.4, 0.1, 0.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.14, 0.14, 0.35, 6);
    leg_r.apply(Transform::translate(0.18, -0.5, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.14, 0.14, 0.35, 6);
    leg_l.apply(Transform::translate(-0.18, -0.5, 0.0));

    let mesh = combine(&[&body, &head, &arm_r, &arm_l, &leg_r, &leg_l]);
    write_mesh(&mesh, "golem", output_dir);
}

fn generate_crawler(output_dir: &Path) {
    let mut body: UnpackedMesh = generate_sphere(0.2, 8, 6);
    body.apply(Transform::scale(1.2, 0.6, 1.0));

    let mut head: UnpackedMesh = generate_sphere(0.1, 6, 4);
    head.apply(Transform::translate(0.22, 0.05, 0.0));

    let leg_positions: [(f32, f32, f32, f32); 6] = [
        (0.1, -0.08, 0.18, 34.0),
        (0.0, -0.08, 0.2, 34.0),
        (-0.1, -0.08, 0.18, 34.0),
        (0.1, -0.08, -0.18, -34.0),
        (0.0, -0.08, -0.2, -34.0),
        (-0.1, -0.08, -0.18, -34.0),
    ];

    let mut legs = Vec::new();
    for (x, y, z, rot) in leg_positions {
        let mut leg: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.15, 4);
        leg.apply(Transform::translate(x, y, z));
        leg.apply(Transform::rotate_x(rot));
        legs.push(leg);
    }

    let leg_refs: Vec<&UnpackedMesh> = legs.iter().collect();
    let mut parts = vec![&body, &head];
    parts.extend(leg_refs);

    let mesh = combine(&parts);
    write_mesh(&mesh, "crawler", output_dir);
}

fn generate_wisp(output_dir: &Path) {
    let mut core: UnpackedMesh = generate_sphere(0.12, 10, 8);
    core.apply(SmoothNormals::default());

    let glow: UnpackedMesh = generate_sphere(0.18, 8, 6);

    let mut tail: UnpackedMesh = generate_sphere(0.08, 6, 4);
    tail.apply(Transform::translate(-0.15, 0.0, 0.0));
    tail.apply(Transform::scale(2.0, 0.6, 0.6));

    let mesh = combine(&[&core, &glow, &tail]);
    write_mesh(&mesh, "wisp", output_dir);
}

fn generate_skeleton(output_dir: &Path) {
    let mut ribcage: UnpackedMesh = generate_capsule(0.15, 0.25, 6, 3);
    ribcage.apply(Transform::scale(1.0, 1.0, 0.6));

    let mut skull: UnpackedMesh = generate_sphere(0.12, 8, 6);
    skull.apply(Transform::translate(0.0, 0.35, 0.0));
    skull.apply(Transform::scale(1.0, 1.2, 0.9));

    let mut jaw: UnpackedMesh = generate_cube(0.08, 0.04, 0.06);
    jaw.apply(Transform::translate(0.0, 0.28, 0.04));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.25, 4);
    arm_r.apply(Transform::translate(0.2, 0.05, 0.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.25, 4);
    arm_l.apply(Transform::translate(-0.2, 0.05, 0.0));

    let mut spine: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.15, 4);
    spine.apply(Transform::translate(0.0, -0.2, 0.0));

    let mut pelvis: UnpackedMesh = generate_cube(0.18, 0.08, 0.1);
    pelvis.apply(Transform::translate(0.0, -0.3, 0.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.3, 4);
    leg_r.apply(Transform::translate(0.08, -0.5, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.3, 4);
    leg_l.apply(Transform::translate(-0.08, -0.5, 0.0));

    let mesh = combine(&[
        &ribcage, &skull, &jaw, &arm_r, &arm_l, &spine, &pelvis, &leg_r, &leg_l,
    ]);
    write_mesh(&mesh, "skeleton", output_dir);
}

fn generate_shade(output_dir: &Path) {
    // Fast, ethereal shadow creature
    let mut body: UnpackedMesh = generate_sphere(0.18, 8, 6);
    body.apply(Transform::scale(1.0, 1.5, 0.8));

    // Wispy tendrils
    let mut tendril1: UnpackedMesh = generate_cylinder(0.04, 0.01, 0.3, 4);
    tendril1.apply(Transform::translate(0.1, -0.35, 0.0));
    tendril1.apply(Transform::rotate_z(15.0));

    let mut tendril2: UnpackedMesh = generate_cylinder(0.04, 0.01, 0.25, 4);
    tendril2.apply(Transform::translate(-0.1, -0.32, 0.05));
    tendril2.apply(Transform::rotate_z(-12.0));

    let mut tendril3: UnpackedMesh = generate_cylinder(0.03, 0.01, 0.28, 4);
    tendril3.apply(Transform::translate(0.0, -0.33, -0.08));

    // Glowing eyes
    let mut eye_r: UnpackedMesh = generate_sphere(0.03, 4, 3);
    eye_r.apply(Transform::translate(0.06, 0.1, 0.12));

    let mut eye_l: UnpackedMesh = generate_sphere(0.03, 4, 3);
    eye_l.apply(Transform::translate(-0.06, 0.1, 0.12));

    let mesh = combine(&[&body, &tendril1, &tendril2, &tendril3, &eye_r, &eye_l]);
    write_mesh(&mesh, "shade", output_dir);
}

fn generate_berserker(output_dir: &Path) {
    // Muscular, rage-filled warrior
    let mut torso: UnpackedMesh = generate_capsule(0.28, 0.4, 8, 4);
    torso.apply(Transform::scale(1.3, 1.0, 1.0));

    let mut head: UnpackedMesh = generate_sphere(0.14, 6, 5);
    head.apply(Transform::translate(0.0, 0.4, 0.0));

    // Massive arms
    let mut arm_r: UnpackedMesh = generate_cylinder(0.12, 0.1, 0.35, 6);
    arm_r.apply(Transform::translate(0.38, 0.1, 0.0));
    arm_r.apply(Transform::rotate_z(-25.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.12, 0.1, 0.35, 6);
    arm_l.apply(Transform::translate(-0.38, 0.1, 0.0));
    arm_l.apply(Transform::rotate_z(25.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.11, 0.11, 0.35, 6);
    leg_r.apply(Transform::translate(0.15, -0.45, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.11, 0.11, 0.35, 6);
    leg_l.apply(Transform::translate(-0.15, -0.45, 0.0));

    // Double-headed axe
    let mut axe_handle: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.6, 4);
    axe_handle.apply(Transform::translate(0.45, 0.25, 0.0));

    let mut axe_head1: UnpackedMesh = generate_cube(0.15, 0.2, 0.03);
    axe_head1.apply(Transform::translate(0.45, 0.55, 0.08));

    let mut axe_head2: UnpackedMesh = generate_cube(0.15, 0.2, 0.03);
    axe_head2.apply(Transform::translate(0.45, 0.55, -0.08));

    let mesh = combine(&[
        &torso, &head, &arm_r, &arm_l, &leg_r, &leg_l,
        &axe_handle, &axe_head1, &axe_head2,
    ]);
    write_mesh(&mesh, "berserker", output_dir);
}

fn generate_arcane_sentinel(output_dir: &Path) {
    // Floating magical construct
    let core: UnpackedMesh = generate_sphere(0.2, 10, 8);

    // Outer arcane rings
    let mut ring1: UnpackedMesh = generate_torus(0.3, 0.02, 16, 6);
    ring1.apply(Transform::rotate_x(90.0));

    let mut ring2: UnpackedMesh = generate_torus(0.35, 0.02, 16, 6);
    ring2.apply(Transform::rotate_x(90.0));
    ring2.apply(Transform::rotate_y(45.0));

    // Floating crystal shards
    let mut shard1: UnpackedMesh = generate_cylinder(0.0, 0.06, 0.15, 5);
    shard1.apply(Transform::translate(0.0, 0.35, 0.0));

    let mut shard2: UnpackedMesh = generate_cylinder(0.0, 0.05, 0.12, 5);
    shard2.apply(Transform::translate(0.25, 0.15, 0.15));
    shard2.apply(Transform::rotate_z(-30.0));

    let mut shard3: UnpackedMesh = generate_cylinder(0.0, 0.05, 0.12, 5);
    shard3.apply(Transform::translate(-0.25, 0.15, -0.15));
    shard3.apply(Transform::rotate_z(30.0));

    let mut shard4: UnpackedMesh = generate_cylinder(0.0, 0.06, 0.14, 5);
    shard4.apply(Transform::translate(0.0, -0.3, 0.0));
    shard4.apply(Transform::rotate_x(180.0));

    let mesh = combine(&[&core, &ring1, &ring2, &shard1, &shard2, &shard3, &shard4]);
    write_mesh(&mesh, "arcane_sentinel", output_dir);
}
