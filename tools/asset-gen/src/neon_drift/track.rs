//! Track segment and prop generators for NEON DRIFT
//!
//! Modular track pieces that snap together for level design.

use proc_gen::mesh::*;
use std::path::Path;

const TRACK_WIDTH: f32 = 2.0;
const SEGMENT_LENGTH: f32 = 4.0;

pub fn generate_straight(output_dir: &Path) {
    println!("  Generating: track_straight.obj");

    let mut road: UnpackedMesh = generate_plane(SEGMENT_LENGTH, TRACK_WIDTH, 4, 2);
    road.apply(Transform::rotate_x(-90.0));

    let mut barrier_l: UnpackedMesh = generate_cube(SEGMENT_LENGTH, 0.15, 0.1);
    barrier_l.apply(Transform::translate(0.0, 0.075, -TRACK_WIDTH / 2.0 - 0.05));

    let mut barrier_r: UnpackedMesh = generate_cube(SEGMENT_LENGTH, 0.15, 0.1);
    barrier_r.apply(Transform::translate(0.0, 0.075, TRACK_WIDTH / 2.0 + 0.05));

    let mut neon_l: UnpackedMesh = generate_cube(SEGMENT_LENGTH - 0.1, 0.02, 0.02);
    neon_l.apply(Transform::translate(0.0, 0.16, -TRACK_WIDTH / 2.0 - 0.05));

    let mut neon_r: UnpackedMesh = generate_cube(SEGMENT_LENGTH - 0.1, 0.02, 0.02);
    neon_r.apply(Transform::translate(0.0, 0.16, TRACK_WIDTH / 2.0 + 0.05));

    let mesh = combine(&[&road, &barrier_l, &barrier_r, &neon_l, &neon_r]);

    let path = output_dir.join("track_straight.obj");
    write_obj(&mesh, &path, "track_straight").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

pub fn generate_curve_left(output_dir: &Path) {
    println!("  Generating: track_curve_left.obj");

    let segments = 8;
    let angle_per_segment = 45.0_f32.to_radians() / segments as f32;
    let outer_radius = 6.0;
    let inner_radius = outer_radius - TRACK_WIDTH;

    let mut parts: Vec<UnpackedMesh> = Vec::new();

    for i in 0..segments {
        let angle = i as f32 * angle_per_segment;
        let next_angle = (i + 1) as f32 * angle_per_segment;

        let mut segment: UnpackedMesh = generate_plane(0.6, TRACK_WIDTH, 1, 1);
        segment.apply(Transform::rotate_x(-90.0));

        let mid_radius = (outer_radius + inner_radius) / 2.0;
        let mid_angle = (angle + next_angle) / 2.0;

        segment.apply(Transform::translate(
            -mid_radius * mid_angle.sin(),
            0.0,
            mid_radius * mid_angle.cos() - outer_radius + TRACK_WIDTH / 2.0,
        ));
        segment.apply(Transform::rotate_y(-mid_angle.to_degrees()));

        parts.push(segment);
    }

    let mut outer_barrier: UnpackedMesh = generate_torus(outer_radius + 0.05, 0.08, 16, 4);
    outer_barrier.apply(Transform::scale(1.0, 1.0, 0.2));
    outer_barrier.apply(Transform::translate(0.0, 0.08, -outer_radius + TRACK_WIDTH / 2.0));

    let mut inner_barrier: UnpackedMesh = generate_torus(inner_radius - 0.05, 0.08, 16, 4);
    inner_barrier.apply(Transform::scale(1.0, 1.0, 0.2));
    inner_barrier.apply(Transform::translate(0.0, 0.08, -outer_radius + TRACK_WIDTH / 2.0));

    let part_refs: Vec<&UnpackedMesh> = parts.iter().collect();
    let mut all_parts = part_refs;
    all_parts.push(&outer_barrier);
    all_parts.push(&inner_barrier);

    let mesh = combine(&all_parts);

    let path = output_dir.join("track_curve_left.obj");
    write_obj(&mesh, &path, "track_curve_left").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

pub fn generate_curve_right(output_dir: &Path) {
    println!("  Generating: track_curve_right.obj");

    let segments = 8;
    let angle_per_segment = 45.0_f32.to_radians() / segments as f32;
    let outer_radius = 6.0;
    let inner_radius = outer_radius - TRACK_WIDTH;

    let mut parts: Vec<UnpackedMesh> = Vec::new();

    for i in 0..segments {
        let angle = i as f32 * angle_per_segment;
        let next_angle = (i + 1) as f32 * angle_per_segment;

        let mut segment: UnpackedMesh = generate_plane(0.6, TRACK_WIDTH, 1, 1);
        segment.apply(Transform::rotate_x(-90.0));

        let mid_radius = (outer_radius + inner_radius) / 2.0;
        let mid_angle = (angle + next_angle) / 2.0;

        segment.apply(Transform::translate(
            -mid_radius * mid_angle.sin(),
            0.0,
            -(mid_radius * mid_angle.cos() - outer_radius + TRACK_WIDTH / 2.0),
        ));
        segment.apply(Transform::rotate_y(mid_angle.to_degrees()));

        parts.push(segment);
    }

    let part_refs: Vec<&UnpackedMesh> = parts.iter().collect();
    let mesh = combine(&part_refs);

    let path = output_dir.join("track_curve_right.obj");
    write_obj(&mesh, &path, "track_curve_right").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

pub fn generate_tunnel(output_dir: &Path) {
    println!("  Generating: track_tunnel.obj");

    let mut road: UnpackedMesh = generate_plane(SEGMENT_LENGTH * 2.0, TRACK_WIDTH, 8, 2);
    road.apply(Transform::rotate_x(-90.0));

    let mut tunnel: UnpackedMesh = generate_cylinder(
        TRACK_WIDTH / 2.0 + 0.3,
        TRACK_WIDTH / 2.0 + 0.3,
        SEGMENT_LENGTH * 2.0,
        12,
    );
    tunnel.apply(Transform::rotate_z(90.0));

    let ring_count = 5;
    let mut rings = Vec::new();
    for i in 0..ring_count {
        let x = -SEGMENT_LENGTH + (i as f32 + 0.5) * (SEGMENT_LENGTH * 2.0 / ring_count as f32);
        let mut ring: UnpackedMesh = generate_torus(TRACK_WIDTH / 2.0 + 0.35, 0.05, 16, 4);
        ring.apply(Transform::rotate_z(90.0));
        ring.apply(Transform::translate(x, TRACK_WIDTH / 4.0, 0.0));
        rings.push(ring);
    }

    let ring_refs: Vec<&UnpackedMesh> = rings.iter().collect();
    let mut parts = vec![&road, &tunnel];
    parts.extend(ring_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("track_tunnel.obj");
    write_obj(&mesh, &path, "track_tunnel").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

pub fn generate_jump_ramp(output_dir: &Path) {
    println!("  Generating: track_jump.obj");

    let mut base: UnpackedMesh = generate_cube(2.0, 0.3, TRACK_WIDTH);
    base.apply(Transform::translate(0.0, 0.15, 0.0));

    let mut ramp: UnpackedMesh = generate_cube(1.5, 0.1, TRACK_WIDTH - 0.2);
    ramp.apply(Transform::rotate_z(-11.5));
    ramp.apply(Transform::translate(0.0, 0.4, 0.0));

    let mut rail_l: UnpackedMesh = generate_cube(1.8, 0.2, 0.08);
    rail_l.apply(Transform::rotate_z(-8.5));
    rail_l.apply(Transform::translate(0.0, 0.4, -TRACK_WIDTH / 2.0 + 0.04));

    let mut rail_r: UnpackedMesh = generate_cube(1.8, 0.2, 0.08);
    rail_r.apply(Transform::rotate_z(-8.5));
    rail_r.apply(Transform::translate(0.0, 0.4, TRACK_WIDTH / 2.0 - 0.04));

    let mut chevron1: UnpackedMesh = generate_cube(0.1, 0.05, 0.6);
    chevron1.apply(Transform::translate(-0.5, 0.35, 0.0));

    let mut chevron2: UnpackedMesh = generate_cube(0.1, 0.05, 0.6);
    chevron2.apply(Transform::translate(0.0, 0.45, 0.0));

    let mut chevron3: UnpackedMesh = generate_cube(0.1, 0.05, 0.6);
    chevron3.apply(Transform::translate(0.5, 0.55, 0.0));

    let mesh = combine(&[
        &base, &ramp, &rail_l, &rail_r, &chevron1, &chevron2, &chevron3,
    ]);

    let path = output_dir.join("track_jump.obj");
    write_obj(&mesh, &path, "track_jump").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === PROPS ===

pub fn generate_barrier(output_dir: &Path) {
    println!("  Generating: prop_barrier.obj");

    let mut barrier: UnpackedMesh = generate_cube(1.2, 0.4, 0.25);
    barrier.apply(Transform::scale(1.0, 1.0, 0.8));

    let mut strip: UnpackedMesh = generate_cube(1.1, 0.05, 0.02);
    strip.apply(Transform::translate(0.0, 0.15, 0.14));

    let mesh = combine(&[&barrier, &strip]);

    let path = output_dir.join("prop_barrier.obj");
    write_obj(&mesh, &path, "prop_barrier").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

pub fn generate_boost_pad(output_dir: &Path) {
    println!("  Generating: prop_boost_pad.obj");

    let pad: UnpackedMesh = generate_cube(1.5, 0.02, 0.8);

    let mut arrow1: UnpackedMesh = generate_cube(0.08, 0.025, 0.3);
    arrow1.apply(Transform::translate(-0.4, 0.02, 0.0));

    let mut arrow2: UnpackedMesh = generate_cube(0.08, 0.025, 0.3);
    arrow2.apply(Transform::translate(0.0, 0.02, 0.0));

    let mut arrow3: UnpackedMesh = generate_cube(0.08, 0.025, 0.3);
    arrow3.apply(Transform::translate(0.4, 0.02, 0.0));

    let mut glow_l: UnpackedMesh = generate_cube(1.4, 0.03, 0.03);
    glow_l.apply(Transform::translate(0.0, 0.02, -0.38));

    let mut glow_r: UnpackedMesh = generate_cube(1.4, 0.03, 0.03);
    glow_r.apply(Transform::translate(0.0, 0.02, 0.38));

    let mesh = combine(&[&pad, &arrow1, &arrow2, &arrow3, &glow_l, &glow_r]);

    let path = output_dir.join("prop_boost_pad.obj");
    write_obj(&mesh, &path, "prop_boost_pad").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

pub fn generate_billboard(output_dir: &Path) {
    println!("  Generating: prop_billboard.obj");

    let mut post: UnpackedMesh = generate_cylinder(0.08, 0.08, 2.5, 6);
    post.apply(Transform::translate(0.0, 1.25, 0.0));

    let mut panel: UnpackedMesh = generate_cube(1.8, 0.8, 0.05);
    panel.apply(Transform::translate(0.0, 2.6, 0.0));

    let mut frame_top: UnpackedMesh = generate_cube(1.9, 0.04, 0.06);
    frame_top.apply(Transform::translate(0.0, 3.0, 0.0));

    let mut frame_bottom: UnpackedMesh = generate_cube(1.9, 0.04, 0.06);
    frame_bottom.apply(Transform::translate(0.0, 2.2, 0.0));

    let mut frame_left: UnpackedMesh = generate_cube(0.04, 0.84, 0.06);
    frame_left.apply(Transform::translate(-0.92, 2.6, 0.0));

    let mut frame_right: UnpackedMesh = generate_cube(0.04, 0.84, 0.06);
    frame_right.apply(Transform::translate(0.92, 2.6, 0.0));

    let mesh = combine(&[
        &post, &panel, &frame_top, &frame_bottom, &frame_left, &frame_right,
    ]);

    let path = output_dir.join("prop_billboard.obj");
    write_obj(&mesh, &path, "prop_billboard").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

pub fn generate_building(output_dir: &Path) {
    println!("  Generating: prop_building.obj");

    let mut tower: UnpackedMesh = generate_cube(2.0, 8.0, 1.5);
    tower.apply(Transform::translate(0.0, 4.0, 0.0));

    let mut windows: Vec<UnpackedMesh> = Vec::new();
    for i in 0..6 {
        let y = 1.5 + i as f32 * 1.2;
        let mut strip: UnpackedMesh = generate_cube(1.8, 0.3, 0.02);
        strip.apply(Transform::translate(0.0, y, 0.77));
        windows.push(strip);
    }

    let mut roof: UnpackedMesh = generate_cube(1.2, 0.8, 1.0);
    roof.apply(Transform::translate(0.0, 8.4, 0.0));

    let mut antenna: UnpackedMesh = generate_cylinder(0.05, 0.05, 1.5, 4);
    antenna.apply(Transform::translate(0.0, 9.5, 0.0));

    let mut neon: UnpackedMesh = generate_cube(1.9, 0.08, 0.02);
    neon.apply(Transform::translate(0.0, 7.8, 0.77));

    let window_refs: Vec<&UnpackedMesh> = windows.iter().collect();
    let mut parts = vec![&tower, &roof, &antenna, &neon];
    parts.extend(window_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("prop_building.obj");
    write_obj(&mesh, &path, "prop_building").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
