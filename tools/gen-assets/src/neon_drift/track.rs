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

// === CRYSTAL CAVERN SEGMENTS ===

/// Crystal formation prop - tall glowing crystal cluster
pub fn generate_crystal_formation(output_dir: &Path) {
    println!("  Generating: crystal_formation.obj");

    let mut crystals: Vec<UnpackedMesh> = Vec::new();

    // Main tall crystal - hexagonal prism with pointed top
    let mut main_crystal = generate_crystal(0.4, 2.0, 6);
    main_crystal.apply(Transform::translate(0.0, 1.0, 0.0));
    main_crystal.apply(Transform::rotate_y(15.0));
    crystals.push(main_crystal);

    // Secondary crystals at various angles
    let mut crystal2 = generate_crystal(0.25, 1.4, 6);
    crystal2.apply(Transform::rotate_z(20.0));
    crystal2.apply(Transform::translate(0.3, 0.6, 0.2));
    crystals.push(crystal2);

    let mut crystal3 = generate_crystal(0.2, 1.1, 6);
    crystal3.apply(Transform::rotate_z(-25.0));
    crystal3.apply(Transform::rotate_y(60.0));
    crystal3.apply(Transform::translate(-0.25, 0.5, -0.15));
    crystals.push(crystal3);

    let mut crystal4 = generate_crystal(0.15, 0.8, 6);
    crystal4.apply(Transform::rotate_z(35.0));
    crystal4.apply(Transform::rotate_y(120.0));
    crystal4.apply(Transform::translate(0.1, 0.4, -0.3));
    crystals.push(crystal4);

    // Small accent crystals
    let mut crystal5 = generate_crystal(0.1, 0.5, 6);
    crystal5.apply(Transform::rotate_z(-15.0));
    crystal5.apply(Transform::translate(-0.4, 0.25, 0.1));
    crystals.push(crystal5);

    let crystal_refs: Vec<&UnpackedMesh> = crystals.iter().collect();
    let mesh = combine(&crystal_refs);

    let path = output_dir.join("crystal_formation.obj");
    write_obj(&mesh, &path, "crystal_formation").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Crystal cavern S-curve segment - tight curves through crystal formations
pub fn generate_cavern_scurve(output_dir: &Path) {
    println!("  Generating: track_cavern_scurve.obj");

    let mut parts: Vec<UnpackedMesh> = Vec::new();

    // S-curve road surface
    let segments = 16;
    let total_length = SEGMENT_LENGTH * 2.0;
    let curve_amplitude = 1.5;

    for i in 0..segments {
        let t = i as f32 / segments as f32;
        let next_t = (i + 1) as f32 / segments as f32;

        // S-curve: sin wave
        let x = t * total_length - total_length / 2.0;
        let z = (t * std::f32::consts::PI * 2.0).sin() * curve_amplitude;
        let next_z = (next_t * std::f32::consts::PI * 2.0).sin() * curve_amplitude;

        let mut segment: UnpackedMesh = generate_plane(total_length / segments as f32 + 0.05, TRACK_WIDTH, 1, 1);
        segment.apply(Transform::rotate_x(-90.0));

        // Calculate rotation based on curve direction
        let dz = next_z - z;
        let angle = (dz / (total_length / segments as f32)).atan().to_degrees();
        segment.apply(Transform::rotate_y(-angle));
        segment.apply(Transform::translate(x, 0.0, z));

        parts.push(segment);
    }

    // Cavern walls (rough rock with openings)
    let mut wall_l: UnpackedMesh = generate_cube(total_length, 1.5, 0.3);
    wall_l.apply(Transform::translate(0.0, 0.75, -TRACK_WIDTH - curve_amplitude));

    let mut wall_r: UnpackedMesh = generate_cube(total_length, 1.5, 0.3);
    wall_r.apply(Transform::translate(0.0, 0.75, TRACK_WIDTH + curve_amplitude));

    // Ceiling (low for claustrophobic feel)
    let mut ceiling: UnpackedMesh = generate_plane(total_length, TRACK_WIDTH * 3.0 + curve_amplitude * 2.0, 4, 4);
    ceiling.apply(Transform::rotate_x(90.0));
    ceiling.apply(Transform::translate(0.0, 2.0, 0.0));

    parts.push(wall_l);
    parts.push(wall_r);
    parts.push(ceiling);

    // Crystal obstacles along curves
    let mut crystal1 = generate_crystal(0.2, 0.8, 6);
    crystal1.apply(Transform::translate(-2.0, 0.4, curve_amplitude + 0.5));
    parts.push(crystal1);

    let mut crystal2 = generate_crystal(0.25, 1.0, 6);
    crystal2.apply(Transform::translate(2.0, 0.5, -curve_amplitude - 0.5));
    parts.push(crystal2);

    let part_refs: Vec<&UnpackedMesh> = parts.iter().collect();
    let mesh = combine(&part_refs);

    let path = output_dir.join("track_cavern_scurve.obj");
    write_obj(&mesh, &path, "track_cavern_scurve").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Crystal cavern low ceiling section - claustrophobic tunnel
pub fn generate_cavern_low_ceiling(output_dir: &Path) {
    println!("  Generating: track_cavern_low.obj");

    let length = SEGMENT_LENGTH * 1.5;

    // Road surface
    let mut road: UnpackedMesh = generate_plane(length, TRACK_WIDTH, 6, 2);
    road.apply(Transform::rotate_x(-90.0));

    // Low rocky ceiling with stalactites
    let mut ceiling: UnpackedMesh = generate_plane(length, TRACK_WIDTH + 0.4, 4, 4);
    ceiling.apply(Transform::rotate_x(90.0));
    ceiling.apply(Transform::translate(0.0, 1.2, 0.0));

    // Stalactites (inverted crystals)
    let mut stalactites: Vec<UnpackedMesh> = Vec::new();
    for i in 0..5 {
        let x = -length / 2.0 + 0.5 + i as f32 * (length / 5.0);
        let z = (i as f32 * 1.7).sin() * 0.3;
        let height = 0.3 + (i as f32 * 2.3).cos().abs() * 0.3;

        let mut stalactite = generate_crystal(0.08, height, 5);
        stalactite.apply(Transform::rotate_z(180.0));
        stalactite.apply(Transform::translate(x, 1.2 - height / 2.0, z));
        stalactites.push(stalactite);
    }

    // Cave walls
    let mut wall_l: UnpackedMesh = generate_cube(length, 1.2, 0.25);
    wall_l.apply(Transform::translate(0.0, 0.6, -TRACK_WIDTH / 2.0 - 0.3));

    let mut wall_r: UnpackedMesh = generate_cube(length, 1.2, 0.25);
    wall_r.apply(Transform::translate(0.0, 0.6, TRACK_WIDTH / 2.0 + 0.3));

    // Glowing crystal veins on walls
    let mut vein_l: UnpackedMesh = generate_cube(length - 0.2, 0.05, 0.02);
    vein_l.apply(Transform::translate(0.0, 0.8, -TRACK_WIDTH / 2.0 - 0.16));

    let mut vein_r: UnpackedMesh = generate_cube(length - 0.2, 0.05, 0.02);
    vein_r.apply(Transform::translate(0.0, 0.5, TRACK_WIDTH / 2.0 + 0.16));

    let stalactite_refs: Vec<&UnpackedMesh> = stalactites.iter().collect();
    let mut parts = vec![&road, &ceiling, &wall_l, &wall_r, &vein_l, &vein_r];
    parts.extend(stalactite_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("track_cavern_low.obj");
    write_obj(&mesh, &path, "track_cavern_low").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === SOLAR HIGHWAY SEGMENTS ===

/// Solar highway long straight - high-speed section with solar panels
pub fn generate_solar_straight(output_dir: &Path) {
    println!("  Generating: track_solar_straight.obj");

    let length = SEGMENT_LENGTH * 3.0; // Extra long for high-speed

    // Wide road surface
    let mut road: UnpackedMesh = generate_plane(length, TRACK_WIDTH * 1.2, 12, 2);
    road.apply(Transform::rotate_x(-90.0));

    // Low-profile barriers for aerodynamics
    let mut barrier_l: UnpackedMesh = generate_cube(length, 0.08, 0.15);
    barrier_l.apply(Transform::translate(0.0, 0.04, -TRACK_WIDTH * 0.6 - 0.1));

    let mut barrier_r: UnpackedMesh = generate_cube(length, 0.08, 0.15);
    barrier_r.apply(Transform::translate(0.0, 0.04, TRACK_WIDTH * 0.6 + 0.1));

    // Solar panels along the sides
    let mut panels: Vec<UnpackedMesh> = Vec::new();
    for i in 0..4 {
        let x = -length / 2.0 + 1.5 + i as f32 * 3.0;

        // Left panel array
        let mut panel_l = generate_solar_panel();
        panel_l.apply(Transform::translate(x, 0.0, -TRACK_WIDTH * 0.6 - 1.5));
        panels.push(panel_l);

        // Right panel array
        let mut panel_r = generate_solar_panel();
        panel_r.apply(Transform::translate(x, 0.0, TRACK_WIDTH * 0.6 + 1.5));
        panels.push(panel_r);
    }

    // Speed boost strips on road
    let mut boost1: UnpackedMesh = generate_cube(0.3, 0.01, TRACK_WIDTH);
    boost1.apply(Transform::translate(-length / 3.0, 0.005, 0.0));

    let mut boost2: UnpackedMesh = generate_cube(0.3, 0.01, TRACK_WIDTH);
    boost2.apply(Transform::translate(0.0, 0.005, 0.0));

    let mut boost3: UnpackedMesh = generate_cube(0.3, 0.01, TRACK_WIDTH);
    boost3.apply(Transform::translate(length / 3.0, 0.005, 0.0));

    let panel_refs: Vec<&UnpackedMesh> = panels.iter().collect();
    let mut parts = vec![&road, &barrier_l, &barrier_r, &boost1, &boost2, &boost3];
    parts.extend(panel_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("track_solar_straight.obj");
    write_obj(&mesh, &path, "track_solar_straight").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Solar highway wide sweeping curve
pub fn generate_solar_curve(output_dir: &Path) {
    println!("  Generating: track_solar_curve.obj");

    let segments = 12;
    let angle_per_segment = 60.0_f32.to_radians() / segments as f32;
    let outer_radius = 10.0; // Wide curve for high speed
    let inner_radius = outer_radius - TRACK_WIDTH * 1.2;

    let mut parts: Vec<UnpackedMesh> = Vec::new();

    for i in 0..segments {
        let angle = i as f32 * angle_per_segment;
        let next_angle = (i + 1) as f32 * angle_per_segment;

        let mut segment: UnpackedMesh = generate_plane(1.0, TRACK_WIDTH * 1.2, 1, 1);
        segment.apply(Transform::rotate_x(-90.0));

        let mid_radius = (outer_radius + inner_radius) / 2.0;
        let mid_angle = (angle + next_angle) / 2.0;

        segment.apply(Transform::translate(
            -mid_radius * mid_angle.sin(),
            0.0,
            mid_radius * mid_angle.cos() - outer_radius + TRACK_WIDTH * 0.6,
        ));
        segment.apply(Transform::rotate_y(-mid_angle.to_degrees()));

        parts.push(segment);
    }

    // Banked outer barrier
    let mut outer_barrier: UnpackedMesh = generate_torus(outer_radius + 0.1, 0.1, 24, 4);
    outer_barrier.apply(Transform::scale(1.0, 1.0, 0.15));
    outer_barrier.apply(Transform::translate(0.0, 0.1, -outer_radius + TRACK_WIDTH * 0.6));

    // Inner barrier
    let mut inner_barrier: UnpackedMesh = generate_torus(inner_radius - 0.1, 0.08, 24, 4);
    inner_barrier.apply(Transform::scale(1.0, 1.0, 0.15));
    inner_barrier.apply(Transform::translate(0.0, 0.08, -outer_radius + TRACK_WIDTH * 0.6));

    parts.push(outer_barrier);
    parts.push(inner_barrier);

    let part_refs: Vec<&UnpackedMesh> = parts.iter().collect();
    let mesh = combine(&part_refs);

    let path = output_dir.join("track_solar_curve.obj");
    write_obj(&mesh, &path, "track_solar_curve").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Solar flare jump - dramatic ramp with solar arch
pub fn generate_solar_flare_jump(output_dir: &Path) {
    println!("  Generating: track_solar_jump.obj");

    // Approach ramp
    let mut ramp_base: UnpackedMesh = generate_cube(3.0, 0.4, TRACK_WIDTH * 1.2);
    ramp_base.apply(Transform::translate(-1.5, 0.2, 0.0));

    let mut ramp_surface: UnpackedMesh = generate_cube(3.0, 0.1, TRACK_WIDTH * 1.2 - 0.2);
    ramp_surface.apply(Transform::rotate_z(-8.0));
    ramp_surface.apply(Transform::translate(-1.5, 0.5, 0.0));

    // Launch platform
    let mut launch: UnpackedMesh = generate_cube(1.5, 0.5, TRACK_WIDTH * 1.2);
    launch.apply(Transform::translate(0.5, 0.6, 0.0));

    // Landing platform
    let mut landing: UnpackedMesh = generate_cube(2.0, 0.3, TRACK_WIDTH * 1.2);
    landing.apply(Transform::translate(4.0, 0.15, 0.0));

    // Solar arch over the jump (ring segment)
    let mut arch: UnpackedMesh = generate_torus(3.0, 0.2, 24, 6);
    arch.apply(Transform::scale(1.0, 1.0, 0.2));
    arch.apply(Transform::rotate_x(90.0));
    arch.apply(Transform::translate(2.0, 3.0, 0.0));

    // Glowing corona around arch
    let mut corona: UnpackedMesh = generate_torus(3.3, 0.08, 24, 4);
    corona.apply(Transform::scale(1.0, 1.0, 0.15));
    corona.apply(Transform::rotate_x(90.0));
    corona.apply(Transform::translate(2.0, 3.0, 0.0));

    // Side rails with solar accents
    let mut rail_l: UnpackedMesh = generate_cube(2.5, 0.3, 0.1);
    rail_l.apply(Transform::rotate_z(-6.0));
    rail_l.apply(Transform::translate(-1.2, 0.6, -TRACK_WIDTH * 0.6));

    let mut rail_r: UnpackedMesh = generate_cube(2.5, 0.3, 0.1);
    rail_r.apply(Transform::rotate_z(-6.0));
    rail_r.apply(Transform::translate(-1.2, 0.6, TRACK_WIDTH * 0.6));

    // Chevron markings on ramp
    let mut chevrons: Vec<UnpackedMesh> = Vec::new();
    for i in 0..3 {
        let x = -2.5 + i as f32 * 0.8;
        let mut chevron: UnpackedMesh = generate_cube(0.08, 0.05, 0.6);
        chevron.apply(Transform::translate(x, 0.55 + i as f32 * 0.08, 0.0));
        chevrons.push(chevron);
    }

    let chevron_refs: Vec<&UnpackedMesh> = chevrons.iter().collect();
    let mut parts = vec![
        &ramp_base, &ramp_surface, &launch, &landing, &arch, &corona, &rail_l, &rail_r,
    ];
    parts.extend(chevron_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("track_solar_jump.obj");
    write_obj(&mesh, &path, "track_solar_jump").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === HELPER FUNCTIONS ===

/// Generate a crystal shape (hexagonal prism with pointed top)
fn generate_crystal(radius: f32, height: f32, sides: u32) -> UnpackedMesh {
    // Base is a cylinder (tapers slightly)
    let base: UnpackedMesh = generate_cylinder(radius, radius * 0.7, height * 0.7, sides);

    // Tip is a tapered cylinder (cone-like: top radius = 0)
    let mut tip: UnpackedMesh = generate_cylinder(radius * 0.7, 0.01, height * 0.3, sides);
    tip.apply(Transform::translate(0.0, height * 0.7, 0.0));

    combine(&[&base, &tip])
}

/// Generate a solar panel array
fn generate_solar_panel() -> UnpackedMesh {
    // Post
    let mut post: UnpackedMesh = generate_cylinder(0.06, 0.06, 1.2, 6);
    post.apply(Transform::translate(0.0, 0.6, 0.0));

    // Panel frame
    let mut frame: UnpackedMesh = generate_cube(1.5, 0.05, 0.8);
    frame.apply(Transform::rotate_z(-20.0));
    frame.apply(Transform::translate(0.0, 1.3, 0.0));

    // Panel surface (reflective)
    let mut panel: UnpackedMesh = generate_cube(1.4, 0.02, 0.7);
    panel.apply(Transform::rotate_z(-20.0));
    panel.apply(Transform::translate(0.0, 1.35, 0.0));

    combine(&[&post, &frame, &panel])
}

// === SUNSET STRIP PROPS ===

/// Palm tree for tropical roadside atmosphere
pub fn generate_palm_tree(output_dir: &Path) {
    println!("  Generating: prop_palm_tree.obj");

    // Trunk - tapered cylinder with slight curve
    let mut trunk1: UnpackedMesh = generate_cylinder(0.12, 0.10, 1.0, 8);
    trunk1.apply(Transform::translate(0.0, 0.5, 0.0));

    let mut trunk2: UnpackedMesh = generate_cylinder(0.10, 0.08, 0.8, 8);
    trunk2.apply(Transform::translate(0.05, 1.4, 0.0));

    let mut trunk3: UnpackedMesh = generate_cylinder(0.08, 0.06, 0.6, 8);
    trunk3.apply(Transform::translate(0.08, 2.1, 0.0));

    // Palm fronds (flat planes arranged radially)
    let mut fronds: Vec<UnpackedMesh> = Vec::new();
    for i in 0..7 {
        let angle = i as f32 * 51.4; // ~360/7 degrees
        let droop = 15.0 + (i as f32 * 5.0) % 20.0; // Varying droop

        let mut frond: UnpackedMesh = generate_plane(1.2, 0.25, 4, 1);
        frond.apply(Transform::rotate_x(-droop));
        frond.apply(Transform::rotate_y(angle));
        frond.apply(Transform::translate(0.08, 2.5, 0.0));
        fronds.push(frond);
    }

    // Coconuts cluster
    let mut coconut1: UnpackedMesh = generate_sphere(0.08, 6, 6);
    coconut1.apply(Transform::translate(0.1, 2.35, 0.05));

    let mut coconut2: UnpackedMesh = generate_sphere(0.07, 6, 6);
    coconut2.apply(Transform::translate(0.05, 2.38, -0.06));

    let frond_refs: Vec<&UnpackedMesh> = fronds.iter().collect();
    let mut parts = vec![&trunk1, &trunk2, &trunk3, &coconut1, &coconut2];
    parts.extend(frond_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("prop_palm_tree.obj");
    write_obj(&mesh, &path, "prop_palm_tree").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Retro-style highway sign for sunset strip
pub fn generate_highway_sign(output_dir: &Path) {
    println!("  Generating: prop_highway_sign.obj");

    // Two support posts
    let mut post_l: UnpackedMesh = generate_cylinder(0.08, 0.08, 3.0, 6);
    post_l.apply(Transform::translate(-1.2, 1.5, 0.0));

    let mut post_r: UnpackedMesh = generate_cylinder(0.08, 0.08, 3.0, 6);
    post_r.apply(Transform::translate(1.2, 1.5, 0.0));

    // Sign board
    let mut board: UnpackedMesh = generate_cube(3.0, 1.0, 0.08);
    board.apply(Transform::translate(0.0, 3.2, 0.0));

    // Neon border (thin strips)
    let mut border_top: UnpackedMesh = generate_cube(3.1, 0.05, 0.12);
    border_top.apply(Transform::translate(0.0, 3.75, 0.0));

    let mut border_bottom: UnpackedMesh = generate_cube(3.1, 0.05, 0.12);
    border_bottom.apply(Transform::translate(0.0, 2.65, 0.0));

    let mut border_l: UnpackedMesh = generate_cube(0.05, 1.15, 0.12);
    border_l.apply(Transform::translate(-1.55, 3.2, 0.0));

    let mut border_r: UnpackedMesh = generate_cube(0.05, 1.15, 0.12);
    border_r.apply(Transform::translate(1.55, 3.2, 0.0));

    let mesh = combine(&[&post_l, &post_r, &board, &border_top, &border_bottom, &border_l, &border_r]);

    let path = output_dir.join("prop_highway_sign.obj");
    write_obj(&mesh, &path, "prop_highway_sign").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === NEON CITY PROPS ===

/// Holographic advertisement display
pub fn generate_hologram_ad(output_dir: &Path) {
    println!("  Generating: prop_hologram_ad.obj");

    // Base platform
    let mut base: UnpackedMesh = generate_cylinder(0.4, 0.35, 0.15, 8);
    base.apply(Transform::translate(0.0, 0.075, 0.0));

    // Projector cone
    let mut projector: UnpackedMesh = generate_cylinder(0.15, 0.08, 0.2, 8);
    projector.apply(Transform::translate(0.0, 0.25, 0.0));

    // Hologram frame (wireframe-like thin cubes)
    let mut holo_outer: UnpackedMesh = generate_cube(1.2, 1.6, 0.02);
    holo_outer.apply(Transform::translate(0.0, 1.3, 0.0));

    // Inner hologram planes (for layered effect)
    let mut holo_inner1: UnpackedMesh = generate_cube(0.9, 1.2, 0.01);
    holo_inner1.apply(Transform::translate(0.0, 1.35, 0.02));

    let mut holo_inner2: UnpackedMesh = generate_cube(0.6, 0.8, 0.01);
    holo_inner2.apply(Transform::translate(0.0, 1.4, 0.04));

    // Floating accent rings
    let mut ring1: UnpackedMesh = generate_torus(0.3, 0.02, 16, 4);
    ring1.apply(Transform::translate(0.0, 0.6, 0.0));

    let mut ring2: UnpackedMesh = generate_torus(0.25, 0.015, 16, 4);
    ring2.apply(Transform::translate(0.0, 1.8, 0.0));

    let mesh = combine(&[&base, &projector, &holo_outer, &holo_inner1, &holo_inner2, &ring1, &ring2]);

    let path = output_dir.join("prop_hologram_ad.obj");
    write_obj(&mesh, &path, "prop_hologram_ad").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Neon street lamp for city atmosphere
pub fn generate_street_lamp(output_dir: &Path) {
    println!("  Generating: prop_street_lamp.obj");

    // Main pole
    let mut pole: UnpackedMesh = generate_cylinder(0.05, 0.05, 2.5, 6);
    pole.apply(Transform::translate(0.0, 1.25, 0.0));

    // Curved arm
    let mut arm: UnpackedMesh = generate_cylinder(0.03, 0.03, 0.6, 6);
    arm.apply(Transform::rotate_z(60.0));
    arm.apply(Transform::translate(0.15, 2.4, 0.0));

    // Lamp housing
    let mut housing: UnpackedMesh = generate_cylinder(0.15, 0.12, 0.25, 8);
    housing.apply(Transform::translate(0.4, 2.55, 0.0));

    // Glowing bulb
    let mut bulb: UnpackedMesh = generate_sphere(0.08, 8, 8);
    bulb.apply(Transform::translate(0.4, 2.45, 0.0));

    // Decorative rings on pole
    let mut ring1: UnpackedMesh = generate_torus(0.08, 0.02, 12, 4);
    ring1.apply(Transform::rotate_x(90.0));
    ring1.apply(Transform::translate(0.0, 0.5, 0.0));

    let mut ring2: UnpackedMesh = generate_torus(0.07, 0.015, 12, 4);
    ring2.apply(Transform::rotate_x(90.0));
    ring2.apply(Transform::translate(0.0, 1.5, 0.0));

    // Base
    let mut base: UnpackedMesh = generate_cylinder(0.12, 0.10, 0.15, 8);
    base.apply(Transform::translate(0.0, 0.075, 0.0));

    let mesh = combine(&[&pole, &arm, &housing, &bulb, &ring1, &ring2, &base]);

    let path = output_dir.join("prop_street_lamp.obj");
    write_obj(&mesh, &path, "prop_street_lamp").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === VOID TUNNEL PROPS ===

/// Energy pillar - floating arcane structure
pub fn generate_energy_pillar(output_dir: &Path) {
    println!("  Generating: prop_energy_pillar.obj");

    // Core pillar (octagonal)
    let mut core: UnpackedMesh = generate_cylinder(0.2, 0.15, 2.0, 8);
    core.apply(Transform::translate(0.0, 1.0, 0.0));

    // Floating rings at different heights
    let mut ring1: UnpackedMesh = generate_torus(0.35, 0.04, 16, 4);
    ring1.apply(Transform::rotate_x(90.0));
    ring1.apply(Transform::translate(0.0, 0.4, 0.0));

    let mut ring2: UnpackedMesh = generate_torus(0.4, 0.035, 16, 4);
    ring2.apply(Transform::rotate_x(90.0));
    ring2.apply(Transform::rotate_y(45.0));
    ring2.apply(Transform::translate(0.0, 1.0, 0.0));

    let mut ring3: UnpackedMesh = generate_torus(0.35, 0.04, 16, 4);
    ring3.apply(Transform::rotate_x(90.0));
    ring3.apply(Transform::translate(0.0, 1.6, 0.0));

    // Floating crystal shards
    let mut shard1: UnpackedMesh = generate_crystal(0.08, 0.4, 4);
    shard1.apply(Transform::rotate_z(15.0));
    shard1.apply(Transform::translate(0.35, 1.2, 0.0));

    let mut shard2: UnpackedMesh = generate_crystal(0.06, 0.3, 4);
    shard2.apply(Transform::rotate_z(-20.0));
    shard2.apply(Transform::translate(-0.3, 0.8, 0.2));

    let mut shard3: UnpackedMesh = generate_crystal(0.07, 0.35, 4);
    shard3.apply(Transform::rotate_x(25.0));
    shard3.apply(Transform::translate(0.1, 1.5, -0.35));

    // Base glow disc
    let mut base_glow: UnpackedMesh = generate_cylinder(0.5, 0.5, 0.05, 16);
    base_glow.apply(Transform::translate(0.0, 0.025, 0.0));

    let mesh = combine(&[&core, &ring1, &ring2, &ring3, &shard1, &shard2, &shard3, &base_glow]);

    let path = output_dir.join("prop_energy_pillar.obj");
    write_obj(&mesh, &path, "prop_energy_pillar").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Portal ring - dimensional gateway decoration
pub fn generate_portal_ring(output_dir: &Path) {
    println!("  Generating: prop_portal_ring.obj");

    // Main ring
    let mut main_ring: UnpackedMesh = generate_torus(1.5, 0.12, 32, 8);
    main_ring.apply(Transform::rotate_x(90.0));
    main_ring.apply(Transform::translate(0.0, 1.5, 0.0));

    // Inner ring
    let mut inner_ring: UnpackedMesh = generate_torus(1.2, 0.06, 32, 6);
    inner_ring.apply(Transform::rotate_x(90.0));
    inner_ring.apply(Transform::translate(0.0, 1.5, 0.0));

    // Outer decorative ring
    let mut outer_ring: UnpackedMesh = generate_torus(1.8, 0.04, 32, 4);
    outer_ring.apply(Transform::rotate_x(90.0));
    outer_ring.apply(Transform::translate(0.0, 1.5, 0.0));

    // Support pillars
    let mut pillar_l: UnpackedMesh = generate_cylinder(0.1, 0.08, 1.5, 6);
    pillar_l.apply(Transform::translate(-1.6, 0.75, 0.0));

    let mut pillar_r: UnpackedMesh = generate_cylinder(0.1, 0.08, 1.5, 6);
    pillar_r.apply(Transform::translate(1.6, 0.75, 0.0));

    // Rune stones at base
    let mut rune1: UnpackedMesh = generate_cube(0.2, 0.3, 0.08);
    rune1.apply(Transform::rotate_y(30.0));
    rune1.apply(Transform::translate(-0.8, 0.15, 0.0));

    let mut rune2: UnpackedMesh = generate_cube(0.2, 0.35, 0.08);
    rune2.apply(Transform::rotate_y(-25.0));
    rune2.apply(Transform::translate(0.8, 0.175, 0.0));

    let mut rune3: UnpackedMesh = generate_cube(0.18, 0.25, 0.08);
    rune3.apply(Transform::translate(0.0, 0.125, 0.0));

    let mesh = combine(&[
        &main_ring, &inner_ring, &outer_ring, &pillar_l, &pillar_r,
        &rune1, &rune2, &rune3
    ]);

    let path = output_dir.join("prop_portal_ring.obj");
    write_obj(&mesh, &path, "prop_portal_ring").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === CRYSTAL CAVERN PROPS ===

/// Glowing mushroom cluster for cavern atmosphere
pub fn generate_glowing_mushrooms(output_dir: &Path) {
    println!("  Generating: prop_mushrooms.obj");

    let mut parts: Vec<UnpackedMesh> = Vec::new();

    // Large mushroom
    let mut stem1: UnpackedMesh = generate_cylinder(0.08, 0.06, 0.4, 8);
    stem1.apply(Transform::translate(0.0, 0.2, 0.0));
    parts.push(stem1);

    let mut cap1: UnpackedMesh = generate_sphere(0.25, 12, 8);
    cap1.apply(Transform::scale(1.0, 0.5, 1.0));
    cap1.apply(Transform::translate(0.0, 0.45, 0.0));
    parts.push(cap1);

    // Medium mushroom
    let mut stem2: UnpackedMesh = generate_cylinder(0.05, 0.04, 0.25, 6);
    stem2.apply(Transform::translate(0.2, 0.125, 0.15));
    parts.push(stem2);

    let mut cap2: UnpackedMesh = generate_sphere(0.15, 10, 6);
    cap2.apply(Transform::scale(1.0, 0.5, 1.0));
    cap2.apply(Transform::translate(0.2, 0.3, 0.15));
    parts.push(cap2);

    // Small mushrooms
    for i in 0..3 {
        let angle = i as f32 * 120.0;
        let x = 0.15 * angle.to_radians().cos();
        let z = 0.15 * angle.to_radians().sin() - 0.1;

        let mut stem: UnpackedMesh = generate_cylinder(0.025, 0.02, 0.12, 5);
        stem.apply(Transform::translate(x, 0.06, z));
        parts.push(stem);

        let mut cap: UnpackedMesh = generate_sphere(0.06, 8, 5);
        cap.apply(Transform::scale(1.0, 0.5, 1.0));
        cap.apply(Transform::translate(x, 0.14, z));
        parts.push(cap);
    }

    let part_refs: Vec<&UnpackedMesh> = parts.iter().collect();
    let mesh = combine(&part_refs);

    let path = output_dir.join("prop_mushrooms.obj");
    write_obj(&mesh, &path, "prop_mushrooms").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === SOLAR HIGHWAY PROPS ===

/// Heat vent - glowing geyser prop
pub fn generate_heat_vent(output_dir: &Path) {
    println!("  Generating: prop_heat_vent.obj");

    // Base plate
    let mut base: UnpackedMesh = generate_cylinder(0.5, 0.5, 0.08, 8);
    base.apply(Transform::translate(0.0, 0.04, 0.0));

    // Vent grate (ring)
    let mut grate: UnpackedMesh = generate_torus(0.35, 0.05, 16, 4);
    grate.apply(Transform::translate(0.0, 0.1, 0.0));

    // Inner vent hole
    let mut inner: UnpackedMesh = generate_cylinder(0.25, 0.2, 0.15, 8);
    inner.apply(Transform::translate(0.0, 0.12, 0.0));

    // Glowing core
    let mut core_glow: UnpackedMesh = generate_sphere(0.15, 8, 8);
    core_glow.apply(Transform::translate(0.0, 0.15, 0.0));

    // Steam/heat jets (small cones)
    let mut jet1: UnpackedMesh = generate_cylinder(0.08, 0.02, 0.4, 6);
    jet1.apply(Transform::translate(0.0, 0.4, 0.0));

    let mut jet2: UnpackedMesh = generate_cylinder(0.06, 0.01, 0.3, 6);
    jet2.apply(Transform::rotate_z(15.0));
    jet2.apply(Transform::translate(0.1, 0.35, 0.0));

    let mesh = combine(&[&base, &grate, &inner, &core_glow, &jet1, &jet2]);

    let path = output_dir.join("prop_heat_vent.obj");
    write_obj(&mesh, &path, "prop_heat_vent").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Solar beacon - tall light tower
pub fn generate_solar_beacon(output_dir: &Path) {
    println!("  Generating: prop_solar_beacon.obj");

    // Base platform
    let mut base: UnpackedMesh = generate_cylinder(0.4, 0.35, 0.2, 8);
    base.apply(Transform::translate(0.0, 0.1, 0.0));

    // Main tower (tapered)
    let mut tower: UnpackedMesh = generate_cylinder(0.15, 0.08, 2.5, 6);
    tower.apply(Transform::translate(0.0, 1.45, 0.0));

    // Solar collectors (angled panels)
    let mut panel1: UnpackedMesh = generate_cube(0.4, 0.02, 0.25);
    panel1.apply(Transform::rotate_x(-30.0));
    panel1.apply(Transform::translate(0.0, 2.0, 0.2));

    let mut panel2: UnpackedMesh = generate_cube(0.4, 0.02, 0.25);
    panel2.apply(Transform::rotate_x(30.0));
    panel2.apply(Transform::translate(0.0, 2.0, -0.2));

    // Beacon light housing
    let mut housing: UnpackedMesh = generate_sphere(0.2, 12, 12);
    housing.apply(Transform::translate(0.0, 2.8, 0.0));

    // Light rays (thin cylinders)
    let mut ray1: UnpackedMesh = generate_cylinder(0.02, 0.01, 0.6, 4);
    ray1.apply(Transform::rotate_z(25.0));
    ray1.apply(Transform::translate(0.3, 3.0, 0.0));

    let mut ray2: UnpackedMesh = generate_cylinder(0.02, 0.01, 0.5, 4);
    ray2.apply(Transform::rotate_z(-30.0));
    ray2.apply(Transform::translate(-0.25, 3.05, 0.0));

    let mesh = combine(&[&base, &tower, &panel1, &panel2, &housing, &ray1, &ray2]);

    let path = output_dir.join("prop_solar_beacon.obj");
    write_obj(&mesh, &path, "prop_solar_beacon").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
