//! Pickup and projectile mesh generation for PRISM SURVIVORS
//!
//! XP Gem, Coin, Powerup Orb, Frost Shard, Void Orb, Lightning Bolt, Arena Floor

use crate::mesh_helpers::write_mesh;
use proc_gen::mesh::*;
use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    // Pickups
    generate_xp_gem(output_dir);
    generate_coin(output_dir);
    generate_powerup_orb(output_dir);

    // Projectiles
    generate_frost_shard(output_dir);
    generate_void_orb(output_dir);
    generate_lightning_bolt(output_dir);

    // Arena
    generate_arena_floor(output_dir);
}

fn generate_xp_gem(output_dir: &Path) {
    // Diamond/gem shape - two pyramids joined
    let top: UnpackedMesh = generate_cylinder(0.0, 0.15, 0.15, 6);
    let mut bottom: UnpackedMesh = generate_cylinder(0.15, 0.0, 0.1, 6);
    bottom.apply(Transform::translate(0.0, -0.1, 0.0));

    let mesh = combine(&[&top, &bottom]);
    write_mesh(&mesh, "xp_gem", output_dir);
}

fn generate_coin(output_dir: &Path) {
    let mesh: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.03, 16);
    write_mesh(&mesh, "coin", output_dir);
}

fn generate_powerup_orb(output_dir: &Path) {
    // Glowing orb with outer shell
    let inner: UnpackedMesh = generate_sphere(0.1, 10, 8);
    let mut outer: UnpackedMesh = generate_sphere(0.15, 8, 6);
    outer.apply(SmoothNormals::default());

    let mesh = combine(&[&inner, &outer]);
    write_mesh(&mesh, "powerup_orb", output_dir);
}

fn generate_frost_shard(output_dir: &Path) {
    // Icy crystal projectile
    let shard: UnpackedMesh = generate_cylinder(0.0, 0.08, 0.25, 6);

    // Trailing ice crystals
    let mut trail1: UnpackedMesh = generate_cylinder(0.0, 0.04, 0.1, 4);
    trail1.apply(Transform::translate(0.0, -0.12, 0.0));
    trail1.apply(Transform::rotate_z(25.0));

    let mut trail2: UnpackedMesh = generate_cylinder(0.0, 0.04, 0.1, 4);
    trail2.apply(Transform::translate(0.0, -0.12, 0.0));
    trail2.apply(Transform::rotate_z(-25.0));

    let mesh = combine(&[&shard, &trail1, &trail2]);
    write_mesh(&mesh, "frost_shard", output_dir);
}

fn generate_void_orb(output_dir: &Path) {
    // Dark energy orb with swirling effect
    let core: UnpackedMesh = generate_sphere(0.1, 8, 6);

    let mut outer: UnpackedMesh = generate_sphere(0.15, 6, 4);
    outer.apply(SmoothNormals::default());

    // Orbiting particles
    let mut particle1: UnpackedMesh = generate_sphere(0.03, 4, 3);
    particle1.apply(Transform::translate(0.18, 0.0, 0.0));

    let mut particle2: UnpackedMesh = generate_sphere(0.025, 4, 3);
    particle2.apply(Transform::translate(0.0, 0.18, 0.0));

    let mut particle3: UnpackedMesh = generate_sphere(0.03, 4, 3);
    particle3.apply(Transform::translate(0.0, 0.0, 0.18));

    let mesh = combine(&[&core, &outer, &particle1, &particle2, &particle3]);
    write_mesh(&mesh, "void_orb", output_dir);
}

fn generate_lightning_bolt(output_dir: &Path) {
    // Jagged lightning projectile
    let segment1: UnpackedMesh = generate_cube(0.04, 0.15, 0.02);

    let mut segment2: UnpackedMesh = generate_cube(0.04, 0.12, 0.02);
    segment2.apply(Transform::translate(0.05, 0.12, 0.0));
    segment2.apply(Transform::rotate_z(-20.0));

    let mut segment3: UnpackedMesh = generate_cube(0.04, 0.14, 0.02);
    segment3.apply(Transform::translate(0.02, 0.25, 0.0));
    segment3.apply(Transform::rotate_z(15.0));

    // Glow effect
    let mut glow: UnpackedMesh = generate_sphere(0.08, 6, 4);
    glow.apply(Transform::translate(0.03, 0.15, 0.0));

    let mesh = combine(&[&segment1, &segment2, &segment3, &glow]);
    write_mesh(&mesh, "lightning_bolt", output_dir);
}

fn generate_arena_floor(output_dir: &Path) {
    // Large flat arena floor
    let mesh: UnpackedMesh = generate_plane(40.0, 40.0, 8, 8);
    write_mesh(&mesh, "arena_floor", output_dir);
}
