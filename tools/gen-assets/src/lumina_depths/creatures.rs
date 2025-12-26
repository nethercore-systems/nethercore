//! Sea creature generators for LUMINA DEPTHS
//!
//! All creatures follow specs from SHOWCASE_3.md:
//! - Zone 1 (Sunlit): Reef fish, sea turtle, manta ray
//! - Zone 2 (Twilight): Moon jelly, lanternfish, siphonophore
//! - Zone 3 (Midnight): Anglerfish, gulper eel, dumbo octopus
//! - Zone 4 (Vents): Tube worms, vent shrimp
//! - Epic: Blue whale

use proc_gen::mesh::*;
use std::path::Path;

// === ZONE 1: SUNLIT WATERS ===

/// Reef fish - flat-bodied with fins (~40 tris)
pub fn generate_reef_fish(output_dir: &Path) {
    println!("  Generating: reef_fish.obj");

    // Flat oval body
    let mut body: UnpackedMesh = generate_sphere(0.08, 8, 6);
    body.apply(Transform::scale(1.5, 1.0, 0.4));

    // Tail fin
    let mut tail: UnpackedMesh = generate_cube(0.04, 0.06, 0.01);
    tail.apply(Transform::translate(-0.12, 0.0, 0.0));
    tail.apply(Transform::rotate_z(17.0)); // degrees

    // Dorsal fin
    let mut dorsal: UnpackedMesh = generate_cube(0.05, 0.03, 0.005);
    dorsal.apply(Transform::translate(0.0, 0.06, 0.0));

    // Eye (simple bump)
    let mut eye: UnpackedMesh = generate_sphere(0.015, 4, 3);
    eye.apply(Transform::translate(0.06, 0.02, 0.025));

    let mesh = combine(&[&body, &tail, &dorsal, &eye]);

    let path = output_dir.join("reef_fish.obj");
    write_obj(&mesh, &path, "reef_fish").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Sea turtle - shell with flippers (~200 tris)
pub fn generate_sea_turtle(output_dir: &Path) {
    println!("  Generating: sea_turtle.obj");

    // Shell (dome)
    let mut shell: UnpackedMesh = generate_sphere(0.2, 10, 6);
    shell.apply(Transform::scale(1.2, 0.5, 1.0));
    shell.apply(Transform::translate(0.0, 0.05, 0.0));

    // Underside (flat)
    let mut plastron: UnpackedMesh = generate_sphere(0.18, 8, 4);
    plastron.apply(Transform::scale(1.2, 0.2, 1.0));
    plastron.apply(Transform::translate(0.0, -0.02, 0.0));

    // Head
    let mut head: UnpackedMesh = generate_sphere(0.06, 6, 5);
    head.apply(Transform::translate(0.22, 0.0, 0.0));

    // Front flippers
    let mut flipper_fl: UnpackedMesh = generate_capsule(0.03, 0.12, 4, 2);
    flipper_fl.apply(Transform::rotate_z(-29.0)); // degrees
    flipper_fl.apply(Transform::rotate_x(-17.0));
    flipper_fl.apply(Transform::translate(0.1, -0.02, -0.2));

    let mut flipper_fr: UnpackedMesh = generate_capsule(0.03, 0.12, 4, 2);
    flipper_fr.apply(Transform::rotate_z(-29.0));
    flipper_fr.apply(Transform::rotate_x(17.0));
    flipper_fr.apply(Transform::translate(0.1, -0.02, 0.2));

    // Rear flippers (smaller)
    let mut flipper_rl: UnpackedMesh = generate_capsule(0.02, 0.06, 4, 2);
    flipper_rl.apply(Transform::rotate_z(29.0));
    flipper_rl.apply(Transform::translate(-0.18, -0.02, -0.12));

    let mut flipper_rr: UnpackedMesh = generate_capsule(0.02, 0.06, 4, 2);
    flipper_rr.apply(Transform::rotate_z(29.0));
    flipper_rr.apply(Transform::translate(-0.18, -0.02, 0.12));

    // Tail
    let mut tail: UnpackedMesh = generate_capsule(0.015, 0.05, 4, 2);
    tail.apply(Transform::rotate_z(90.0));
    tail.apply(Transform::translate(-0.25, 0.0, 0.0));

    let mesh = combine(&[
        &shell, &plastron, &head, &flipper_fl, &flipper_fr, &flipper_rl, &flipper_rr, &tail,
    ]);

    let path = output_dir.join("sea_turtle.obj");
    write_obj(&mesh, &path, "sea_turtle").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Manta ray - wide diamond shape with tail (~180 tris)
pub fn generate_manta_ray(output_dir: &Path) {
    println!("  Generating: manta_ray.obj");

    // Wide diamond body (flattened sphere)
    let mut body: UnpackedMesh = generate_sphere(0.3, 10, 6);
    body.apply(Transform::scale(1.2, 0.15, 1.8));

    // Wing tips (slightly elevated)
    let mut wing_l: UnpackedMesh = generate_sphere(0.1, 6, 4);
    wing_l.apply(Transform::scale(0.8, 0.3, 1.2));
    wing_l.apply(Transform::translate(0.0, 0.02, -0.4));

    let mut wing_r: UnpackedMesh = generate_sphere(0.1, 6, 4);
    wing_r.apply(Transform::scale(0.8, 0.3, 1.2));
    wing_r.apply(Transform::translate(0.0, 0.02, 0.4));

    // Cephalic fins (horn-like)
    let mut horn_l: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.1, 4);
    horn_l.apply(Transform::rotate_z(-29.0)); // degrees
    horn_l.apply(Transform::rotate_y(-17.0));
    horn_l.apply(Transform::translate(0.28, 0.0, -0.08));

    let mut horn_r: UnpackedMesh = generate_cylinder(0.02, 0.02, 0.1, 4);
    horn_r.apply(Transform::rotate_z(-29.0));
    horn_r.apply(Transform::rotate_y(17.0));
    horn_r.apply(Transform::translate(0.28, 0.0, 0.08));

    // Long tail
    let mut tail: UnpackedMesh = generate_cylinder(0.015, 0.015, 0.4, 4);
    tail.apply(Transform::rotate_z(90.0));
    tail.apply(Transform::rotate_y(5.7)); // Slight curve
    tail.apply(Transform::translate(-0.45, 0.0, 0.0));

    let mesh = combine(&[&body, &wing_l, &wing_r, &horn_l, &horn_r, &tail]);

    let path = output_dir.join("manta_ray.obj");
    write_obj(&mesh, &path, "manta_ray").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === ZONE 2: TWILIGHT REALM ===

/// Moon jellyfish - translucent dome with tendrils (~120 tris)
pub fn generate_moon_jelly(output_dir: &Path) {
    println!("  Generating: moon_jelly.obj");

    // Bell (dome shape)
    let mut bell: UnpackedMesh = generate_sphere(0.15, 12, 8);
    bell.apply(Transform::scale(1.0, 0.6, 1.0));
    bell.apply(Transform::translate(0.0, 0.05, 0.0));

    // Oral arms (4 short tendrils)
    let angles = [0.0_f32, 90.0, 180.0, 270.0];
    let mut arms = Vec::new();
    for angle_deg in angles {
        let angle = angle_deg.to_radians();
        let mut arm: UnpackedMesh = generate_cylinder(0.015, 0.015, 0.12, 4);
        arm.apply(Transform::translate(0.05 * angle.cos(), -0.08, 0.05 * angle.sin()));
        arms.push(arm);
    }

    // Marginal tentacles (thin ring around edge)
    let mut margin: UnpackedMesh = generate_torus(0.14, 0.008, 16, 4);
    margin.apply(Transform::translate(0.0, -0.02, 0.0));

    let arm_refs: Vec<&UnpackedMesh> = arms.iter().collect();
    let mut parts = vec![&bell, &margin];
    parts.extend(arm_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("moon_jelly.obj");
    write_obj(&mesh, &path, "moon_jelly").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Lanternfish - small with bioluminescent spots (~50 tris)
pub fn generate_lanternfish(output_dir: &Path) {
    println!("  Generating: lanternfish.obj");

    // Torpedo body
    let mut body: UnpackedMesh = generate_capsule(0.03, 0.08, 6, 4);
    body.apply(Transform::rotate_z(90.0));

    // Large eyes (for deep water)
    let mut eye_l: UnpackedMesh = generate_sphere(0.012, 4, 3);
    eye_l.apply(Transform::translate(0.04, 0.015, -0.02));

    let mut eye_r: UnpackedMesh = generate_sphere(0.012, 4, 3);
    eye_r.apply(Transform::translate(0.04, 0.015, 0.02));

    // Tail fin
    let mut tail: UnpackedMesh = generate_cube(0.02, 0.03, 0.005);
    tail.apply(Transform::translate(-0.07, 0.0, 0.0));

    // Photophores (bioluminescent dots)
    let mut photo1: UnpackedMesh = generate_sphere(0.005, 3, 2);
    photo1.apply(Transform::translate(0.02, -0.015, 0.0));

    let mut photo2: UnpackedMesh = generate_sphere(0.005, 3, 2);
    photo2.apply(Transform::translate(-0.02, -0.015, 0.0));

    let mesh = combine(&[&body, &eye_l, &eye_r, &tail, &photo1, &photo2]);

    let path = output_dir.join("lanternfish.obj");
    write_obj(&mesh, &path, "lanternfish").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Siphonophore - chain of segments (~150 tris)
pub fn generate_siphonophore(output_dir: &Path) {
    println!("  Generating: siphonophore.obj");

    // Chain of bioluminescent segments
    let segment_count = 8;
    let mut segments = Vec::new();

    for i in 0..segment_count {
        let y_offset = -(i as f32) * 0.08;
        let size = 0.04 - (i as f32 * 0.003); // Tapering

        let mut segment: UnpackedMesh = generate_sphere(size, 6, 4);
        segment.apply(Transform::translate(0.0, y_offset, 0.0));
        segments.push(segment);

        // Connecting filament
        if i < segment_count - 1 {
            let mut filament: UnpackedMesh = generate_cylinder(0.005, 0.005, 0.04, 4);
            filament.apply(Transform::translate(0.0, y_offset - 0.04, 0.0));
            segments.push(filament);
        }
    }

    // Float (pneumatophore) at top
    let mut float: UnpackedMesh = generate_sphere(0.05, 6, 5);
    float.apply(Transform::translate(0.0, 0.06, 0.0));
    float.apply(Transform::scale(1.0, 1.3, 1.0));

    let segment_refs: Vec<&UnpackedMesh> = segments.iter().collect();
    let mut parts = vec![&float];
    parts.extend(segment_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("siphonophore.obj");
    write_obj(&mesh, &path, "siphonophore").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === ZONE 3: MIDNIGHT ABYSS ===

/// Anglerfish - bulbous body with bioluminescent lure (~180 tris)
pub fn generate_anglerfish(output_dir: &Path) {
    println!("  Generating: anglerfish.obj");

    // Bulbous body
    let mut body: UnpackedMesh = generate_sphere(0.15, 10, 8);
    body.apply(Transform::scale(1.0, 0.9, 0.8));

    // Large mouth (open)
    let mut jaw: UnpackedMesh = generate_sphere(0.1, 8, 4);
    jaw.apply(Transform::scale(1.0, 0.4, 1.0));
    jaw.apply(Transform::translate(0.12, -0.05, 0.0));

    // Teeth (spiky projections)
    let mut teeth: Vec<UnpackedMesh> = Vec::new();
    for i in 0..6 {
        let angle = (i as f32) * 60.0 - 90.0;
        let angle_rad = angle.to_radians();
        let mut tooth: UnpackedMesh = generate_cylinder(0.008, 0.008, 0.03, 3);
        tooth.apply(Transform::rotate_z(29.0));
        tooth.apply(Transform::translate(0.18, -0.04 + angle_rad.sin() * 0.03, angle_rad.cos() * 0.06));
        teeth.push(tooth);
    }

    // Illicium (fishing rod)
    let mut rod: UnpackedMesh = generate_cylinder(0.008, 0.008, 0.15, 4);
    rod.apply(Transform::rotate_z(-34.0));
    rod.apply(Transform::translate(0.08, 0.12, 0.0));

    // Esca (bioluminescent lure)
    let mut lure: UnpackedMesh = generate_sphere(0.025, 6, 5);
    lure.apply(Transform::translate(0.18, 0.2, 0.0));

    // Small pectoral fins
    let mut fin_l: UnpackedMesh = generate_sphere(0.03, 4, 3);
    fin_l.apply(Transform::scale(0.3, 1.0, 1.0));
    fin_l.apply(Transform::translate(-0.05, 0.0, -0.12));

    let mut fin_r: UnpackedMesh = generate_sphere(0.03, 4, 3);
    fin_r.apply(Transform::scale(0.3, 1.0, 1.0));
    fin_r.apply(Transform::translate(-0.05, 0.0, 0.12));

    // Eye (small, adapted to dark)
    let mut eye: UnpackedMesh = generate_sphere(0.015, 4, 3);
    eye.apply(Transform::translate(0.08, 0.06, 0.08));

    let teeth_refs: Vec<&UnpackedMesh> = teeth.iter().collect();
    let mut parts = vec![&body, &jaw, &rod, &lure, &fin_l, &fin_r, &eye];
    parts.extend(teeth_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("anglerfish.obj");
    write_obj(&mesh, &path, "anglerfish").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Gulper eel - long body with huge jaw (~120 tris)
pub fn generate_gulper_eel(output_dir: &Path) {
    println!("  Generating: gulper_eel.obj");

    // Huge expandable mouth
    let mut mouth: UnpackedMesh = generate_sphere(0.12, 8, 6);
    mouth.apply(Transform::scale(1.2, 0.8, 1.0));

    // Long snake-like body
    let mut body: UnpackedMesh = generate_cylinder(0.04, 0.04, 0.5, 6);
    body.apply(Transform::rotate_z(90.0));
    body.apply(Transform::translate(-0.3, 0.0, 0.0));

    // Whip-like tail
    let mut tail: UnpackedMesh = generate_cylinder(0.01, 0.01, 0.3, 4);
    tail.apply(Transform::rotate_z(90.0));
    tail.apply(Transform::translate(-0.7, 0.0, 0.0));

    // Bioluminescent tail tip
    let mut tip: UnpackedMesh = generate_sphere(0.015, 4, 3);
    tip.apply(Transform::translate(-0.85, 0.0, 0.0));

    // Tiny eyes
    let mut eye_l: UnpackedMesh = generate_sphere(0.008, 3, 2);
    eye_l.apply(Transform::translate(0.1, 0.04, -0.06));

    let mut eye_r: UnpackedMesh = generate_sphere(0.008, 3, 2);
    eye_r.apply(Transform::translate(0.1, 0.04, 0.06));

    let mesh = combine(&[&mouth, &body, &tail, &tip, &eye_l, &eye_r]);

    let path = output_dir.join("gulper_eel.obj");
    write_obj(&mesh, &path, "gulper_eel").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Dumbo octopus - round body with ear-like fins (~150 tris)
pub fn generate_dumbo_octopus(output_dir: &Path) {
    println!("  Generating: dumbo_octopus.obj");

    // Round mantle
    let mut mantle: UnpackedMesh = generate_sphere(0.12, 10, 8);
    mantle.apply(Transform::scale(1.0, 1.1, 0.9));

    // Ear-like fins
    let mut ear_l: UnpackedMesh = generate_sphere(0.06, 6, 4);
    ear_l.apply(Transform::scale(0.3, 0.8, 1.0));
    ear_l.apply(Transform::translate(0.0, 0.08, -0.12));

    let mut ear_r: UnpackedMesh = generate_sphere(0.06, 6, 4);
    ear_r.apply(Transform::scale(0.3, 0.8, 1.0));
    ear_r.apply(Transform::translate(0.0, 0.08, 0.12));

    // 8 short arms
    let mut arms = Vec::new();
    for i in 0..8 {
        let angle = (i as f32) * 45.0;
        let angle_rad = angle.to_radians();
        let mut arm: UnpackedMesh = generate_cylinder(0.015, 0.015, 0.1, 4);
        arm.apply(Transform::translate(angle_rad.cos() * 0.08, -0.12, angle_rad.sin() * 0.08));
        arms.push(arm);
    }

    // Eyes (large)
    let mut eye_l: UnpackedMesh = generate_sphere(0.025, 5, 4);
    eye_l.apply(Transform::translate(0.06, 0.04, -0.06));

    let mut eye_r: UnpackedMesh = generate_sphere(0.025, 5, 4);
    eye_r.apply(Transform::translate(0.06, 0.04, 0.06));

    let arm_refs: Vec<&UnpackedMesh> = arms.iter().collect();
    let mut parts = vec![&mantle, &ear_l, &ear_r, &eye_l, &eye_r];
    parts.extend(arm_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("dumbo_octopus.obj");
    write_obj(&mesh, &path, "dumbo_octopus").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === ZONE 4: HYDROTHERMAL VENTS ===

/// Tube worms - clustered stalks with red plumes (~100 tris)
pub fn generate_tube_worms(output_dir: &Path) {
    println!("  Generating: tube_worms.obj");

    let mut worms = Vec::new();

    // Cluster of 5 tube worms
    let positions: [(f32, f32); 5] = [
        (0.0, 0.0),
        (0.06, 0.03),
        (-0.05, 0.04),
        (0.03, -0.05),
        (-0.04, -0.03),
    ];

    for (x, z) in positions {
        let height = 0.15 + (x.abs() + z.abs()) * 0.3;

        // White tube
        let mut tube: UnpackedMesh = generate_cylinder(0.015, 0.015, height, 6);
        tube.apply(Transform::translate(x, height / 2.0, z));
        worms.push(tube);

        // Red plume (feathery top)
        let mut plume: UnpackedMesh = generate_sphere(0.025, 6, 4);
        plume.apply(Transform::scale(1.0, 0.6, 1.0));
        plume.apply(Transform::translate(x, height + 0.01, z));
        worms.push(plume);
    }

    let worm_refs: Vec<&UnpackedMesh> = worms.iter().collect();
    let mesh = combine(&worm_refs);

    let path = output_dir.join("tube_worms.obj");
    write_obj(&mesh, &path, "tube_worms").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// Vent shrimp - small crustacean (~40 tris)
pub fn generate_vent_shrimp(output_dir: &Path) {
    println!("  Generating: vent_shrimp.obj");

    // Segmented body
    let mut body: UnpackedMesh = generate_capsule(0.015, 0.04, 4, 3);
    body.apply(Transform::rotate_z(90.0));

    // Head
    let mut head: UnpackedMesh = generate_sphere(0.012, 4, 3);
    head.apply(Transform::translate(0.03, 0.005, 0.0));

    // Antennae
    let mut antenna_l: UnpackedMesh = generate_cylinder(0.002, 0.002, 0.03, 3);
    antenna_l.apply(Transform::rotate_z(-29.0));
    antenna_l.apply(Transform::translate(0.04, 0.01, -0.008));

    let mut antenna_r: UnpackedMesh = generate_cylinder(0.002, 0.002, 0.03, 3);
    antenna_r.apply(Transform::rotate_z(-29.0));
    antenna_r.apply(Transform::translate(0.04, 0.01, 0.008));

    // Tail fan
    let mut tail: UnpackedMesh = generate_cube(0.015, 0.005, 0.02);
    tail.apply(Transform::translate(-0.04, 0.0, 0.0));

    let mesh = combine(&[&body, &head, &antenna_l, &antenna_r, &tail]);

    let path = output_dir.join("vent_shrimp.obj");
    write_obj(&mesh, &path, "vent_shrimp").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === EPIC ENCOUNTERS ===

/// Blue whale - massive silhouette (~600 tris)
pub fn generate_blue_whale(output_dir: &Path) {
    println!("  Generating: blue_whale.obj");

    // Massive streamlined body
    let mut body: UnpackedMesh = generate_capsule(0.4, 2.0, 16, 10);
    body.apply(Transform::rotate_z(90.0));
    body.apply(Transform::scale(1.0, 0.7, 0.6));

    // Head (broader)
    let mut head: UnpackedMesh = generate_sphere(0.35, 12, 8);
    head.apply(Transform::scale(1.2, 0.8, 0.9));
    head.apply(Transform::translate(1.1, 0.05, 0.0));

    // Pectoral flippers
    let mut flipper_l: UnpackedMesh = generate_capsule(0.08, 0.4, 6, 4);
    flipper_l.apply(Transform::rotate_z(90.0));
    flipper_l.apply(Transform::rotate_y(17.0));
    flipper_l.apply(Transform::translate(0.3, -0.1, -0.35));
    flipper_l.apply(Transform::scale(1.0, 0.3, 1.0));

    let mut flipper_r: UnpackedMesh = generate_capsule(0.08, 0.4, 6, 4);
    flipper_r.apply(Transform::rotate_z(90.0));
    flipper_r.apply(Transform::rotate_y(-17.0));
    flipper_r.apply(Transform::translate(0.3, -0.1, 0.35));
    flipper_r.apply(Transform::scale(1.0, 0.3, 1.0));

    // Dorsal ridge (small hump)
    let mut dorsal: UnpackedMesh = generate_sphere(0.08, 6, 4);
    dorsal.apply(Transform::scale(2.0, 0.5, 0.6));
    dorsal.apply(Transform::translate(-0.5, 0.25, 0.0));

    // Tail flukes
    let mut fluke_l: UnpackedMesh = generate_sphere(0.2, 6, 4);
    fluke_l.apply(Transform::scale(1.5, 0.15, 1.0));
    fluke_l.apply(Transform::rotate_y(23.0));
    fluke_l.apply(Transform::translate(-1.3, 0.0, -0.2));

    let mut fluke_r: UnpackedMesh = generate_sphere(0.2, 6, 4);
    fluke_r.apply(Transform::scale(1.5, 0.15, 1.0));
    fluke_r.apply(Transform::rotate_y(-23.0));
    fluke_r.apply(Transform::translate(-1.3, 0.0, 0.2));

    // Eye
    let mut eye: UnpackedMesh = generate_sphere(0.04, 5, 4);
    eye.apply(Transform::translate(0.95, 0.1, 0.28));

    // Throat grooves (represented as ridges)
    let mut groove1: UnpackedMesh = generate_cylinder(0.02, 0.02, 1.0, 4);
    groove1.apply(Transform::rotate_z(90.0));
    groove1.apply(Transform::translate(0.3, -0.15, 0.0));

    let mesh = combine(&[
        &body, &head, &flipper_l, &flipper_r, &dorsal, &fluke_l, &fluke_r, &eye, &groove1,
    ]);

    let path = output_dir.join("blue_whale.obj");
    write_obj(&mesh, &path, "blue_whale").expect("Failed to write OBJ file");
    println!(
        "    -> Written: {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
