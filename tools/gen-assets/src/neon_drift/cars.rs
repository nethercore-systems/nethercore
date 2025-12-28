//! Vehicle generators for NEON DRIFT
//!
//! Seven distinct car classes with unique silhouettes and neon accents.
//! All meshes use UV-mapped primitives for proper texture mapping.

use proc_gen::mesh::*;
use std::path::Path;

/// Generate a wheel mesh with spokes (uses UV-mapped primitives)
fn generate_wheel(radius: f32, width: f32) -> UnpackedMesh {
    // Outer tire (torus-like ring)
    let mut tire: UnpackedMesh = generate_cylinder_uv(radius, radius, width, 16);
    tire.apply(Transform::rotate_z(90.0));

    // Hub (smaller cylinder in center)
    let hub_radius = radius * 0.35;
    let mut hub: UnpackedMesh = generate_cylinder_uv(hub_radius, hub_radius, width * 0.8, 8);
    hub.apply(Transform::rotate_z(90.0));

    // Spokes (5 spokes radiating from hub)
    let spoke_length = radius - hub_radius - 0.01;
    let spoke_width = 0.012;
    let spoke_height = width * 0.3;
    let mut spokes: Vec<UnpackedMesh> = Vec::new();

    for i in 0..5 {
        let angle = (i as f32) * 72.0; // 360/5 = 72 degrees
        let _angle_rad = angle * core::f32::consts::PI / 180.0;

        let mut spoke: UnpackedMesh = generate_cube_uv(spoke_length, spoke_height, spoke_width);
        // Position spoke at midpoint between hub and rim
        let spoke_center = hub_radius + spoke_length / 2.0;
        spoke.apply(Transform::translate(spoke_center, 0.0, 0.0));
        spoke.apply(Transform::rotate_z(angle));
        spokes.push(spoke);
    }

    // Combine all parts
    let spoke_refs: Vec<&UnpackedMesh> = spokes.iter().collect();
    let mut parts: Vec<&UnpackedMesh> = vec![&tire, &hub];
    parts.extend(spoke_refs);

    combine(&parts)
}

/// Generate 4 wheels positioned for a car body
fn generate_wheels(
    front_x: f32,
    rear_x: f32,
    y: f32,
    half_width: f32,
    wheel_radius: f32,
    wheel_width: f32,
) -> Vec<UnpackedMesh> {
    let mut wheels = Vec::new();

    let mut fl = generate_wheel(wheel_radius, wheel_width);
    fl.apply(Transform::translate(front_x, y, -half_width));
    wheels.push(fl);

    let mut fr = generate_wheel(wheel_radius, wheel_width);
    fr.apply(Transform::translate(front_x, y, half_width));
    wheels.push(fr);

    let mut rl = generate_wheel(wheel_radius, wheel_width);
    rl.apply(Transform::translate(rear_x, y, -half_width));
    wheels.push(rl);

    let mut rr = generate_wheel(wheel_radius, wheel_width);
    rr.apply(Transform::translate(rear_x, y, half_width));
    wheels.push(rr);

    wheels
}

/// SPEEDSTER - "The Classic"
pub fn generate_speedster(output_dir: &Path) {
    println!("  Generating: speedster.obj");

    let mut body: UnpackedMesh = generate_capsule_uv(0.15, 0.6, 8, 4);
    body.apply(Transform::scale(1.0, 0.6, 1.8));
    body.apply(Transform::translate(0.0, 0.12, 0.0));

    let mut hood: UnpackedMesh = generate_cube_uv(0.45, 0.08, 0.35);
    hood.apply(Transform::translate(0.35, 0.12, 0.0));

    let mut windshield: UnpackedMesh = generate_cube_uv(0.15, 0.12, 0.32);
    windshield.apply(Transform::translate(0.1, 0.22, 0.0));
    windshield.apply(Transform::rotate_z(-17.0));

    let mut roof: UnpackedMesh = generate_cube_uv(0.25, 0.06, 0.30);
    roof.apply(Transform::translate(-0.1, 0.25, 0.0));

    let mut rear: UnpackedMesh = generate_cube_uv(0.3, 0.08, 0.34);
    rear.apply(Transform::translate(-0.35, 0.14, 0.0));

    let mut skirt_l: UnpackedMesh = generate_cube_uv(0.5, 0.03, 0.02);
    skirt_l.apply(Transform::translate(0.0, 0.04, -0.38));

    let mut skirt_r: UnpackedMesh = generate_cube_uv(0.5, 0.03, 0.02);
    skirt_r.apply(Transform::translate(0.0, 0.04, 0.38));

    let wheels = generate_wheels(0.32, -0.32, 0.08, 0.40, 0.08, 0.06);

    let mut headlight_l: UnpackedMesh = generate_cube_uv(0.04, 0.03, 0.06);
    headlight_l.apply(Transform::translate(0.58, 0.12, -0.15));

    let mut headlight_r: UnpackedMesh = generate_cube_uv(0.04, 0.03, 0.06);
    headlight_r.apply(Transform::translate(0.58, 0.12, 0.15));

    let mut taillight_l: UnpackedMesh = generate_cube_uv(0.02, 0.04, 0.08);
    taillight_l.apply(Transform::translate(-0.52, 0.14, -0.16));

    let mut taillight_r: UnpackedMesh = generate_cube_uv(0.02, 0.04, 0.08);
    taillight_r.apply(Transform::translate(-0.52, 0.14, 0.16));

    let wheel_refs: Vec<&UnpackedMesh> = wheels.iter().collect();
    let mut parts: Vec<&UnpackedMesh> = vec![
        &body, &hood, &windshield, &roof, &rear,
        &skirt_l, &skirt_r,
        &headlight_l, &headlight_r,
        &taillight_l, &taillight_r,
    ];
    parts.extend(wheel_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("speedster.obj");
    write_obj(&mesh, &path, "speedster").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// MUSCLE - "The Beast"
pub fn generate_muscle(output_dir: &Path) {
    println!("  Generating: muscle.obj");

    let mut body: UnpackedMesh = generate_cube_uv(0.7, 0.18, 0.45);
    body.apply(Transform::translate(0.0, 0.14, 0.0));

    let mut hood: UnpackedMesh = generate_cube_uv(0.35, 0.06, 0.40);
    hood.apply(Transform::translate(0.25, 0.22, 0.0));

    let mut scoop: UnpackedMesh = generate_cube_uv(0.12, 0.08, 0.15);
    scoop.apply(Transform::translate(0.25, 0.28, 0.0));

    let mut intake: UnpackedMesh = generate_cylinder_uv(0.04, 0.04, 0.08, 6);
    intake.apply(Transform::translate(0.25, 0.34, 0.0));

    let mut windshield: UnpackedMesh = generate_cube_uv(0.12, 0.14, 0.38);
    windshield.apply(Transform::translate(0.0, 0.28, 0.0));
    windshield.apply(Transform::rotate_z(-14.0));

    let mut roof: UnpackedMesh = generate_cube_uv(0.22, 0.08, 0.36);
    roof.apply(Transform::translate(-0.12, 0.32, 0.0));

    let mut trunk: UnpackedMesh = generate_cube_uv(0.2, 0.12, 0.42);
    trunk.apply(Transform::translate(-0.35, 0.18, 0.0));

    let mut fender_fl: UnpackedMesh = generate_cube_uv(0.15, 0.1, 0.08);
    fender_fl.apply(Transform::translate(0.35, 0.15, -0.28));

    let mut fender_fr: UnpackedMesh = generate_cube_uv(0.15, 0.1, 0.08);
    fender_fr.apply(Transform::translate(0.35, 0.15, 0.28));

    let mut fender_rl: UnpackedMesh = generate_cube_uv(0.18, 0.12, 0.1);
    fender_rl.apply(Transform::translate(-0.3, 0.16, -0.30));

    let mut fender_rr: UnpackedMesh = generate_cube_uv(0.18, 0.12, 0.1);
    fender_rr.apply(Transform::translate(-0.3, 0.16, 0.30));

    let mut exhaust_l: UnpackedMesh = generate_cylinder_uv(0.025, 0.025, 0.15, 6);
    exhaust_l.apply(Transform::rotate_x(90.0));
    exhaust_l.apply(Transform::translate(-0.2, 0.06, -0.32));

    let mut exhaust_r: UnpackedMesh = generate_cylinder_uv(0.025, 0.025, 0.15, 6);
    exhaust_r.apply(Transform::rotate_x(90.0));
    exhaust_r.apply(Transform::translate(-0.2, 0.06, 0.32));

    let mut wheels = generate_wheels(0.32, -0.32, 0.1, 0.42, 0.1, 0.08);
    wheels[2].apply(Transform::scale(1.1, 1.1, 1.0));
    wheels[3].apply(Transform::scale(1.1, 1.1, 1.0));

    let mut headlight_l: UnpackedMesh = generate_cube_uv(0.03, 0.05, 0.1);
    headlight_l.apply(Transform::translate(0.53, 0.16, -0.18));

    let mut headlight_r: UnpackedMesh = generate_cube_uv(0.03, 0.05, 0.1);
    headlight_r.apply(Transform::translate(0.53, 0.16, 0.18));

    let wheel_refs: Vec<&UnpackedMesh> = wheels.iter().collect();
    let mut parts: Vec<&UnpackedMesh> = vec![
        &body, &hood, &scoop, &intake, &windshield, &roof, &trunk,
        &fender_fl, &fender_fr, &fender_rl, &fender_rr,
        &exhaust_l, &exhaust_r,
        &headlight_l, &headlight_r,
    ];
    parts.extend(wheel_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("muscle.obj");
    write_obj(&mesh, &path, "muscle").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// RACER - "The Pro"
pub fn generate_racer(output_dir: &Path) {
    println!("  Generating: racer.obj");

    let mut body: UnpackedMesh = generate_capsule_uv(0.08, 0.55, 8, 4);
    body.apply(Transform::scale(1.0, 0.6, 0.8));
    body.apply(Transform::translate(0.0, 0.1, 0.0));

    let mut nose: UnpackedMesh = generate_capsule_uv(0.06, 0.25, 6, 3);
    nose.apply(Transform::scale(1.0, 0.5, 0.8));
    nose.apply(Transform::translate(0.45, 0.08, 0.0));

    let mut front_wing: UnpackedMesh = generate_cube_uv(0.08, 0.015, 0.5);
    front_wing.apply(Transform::translate(0.55, 0.04, 0.0));

    let mut front_endplate_l: UnpackedMesh = generate_cube_uv(0.1, 0.04, 0.01);
    front_endplate_l.apply(Transform::translate(0.55, 0.04, -0.26));

    let mut front_endplate_r: UnpackedMesh = generate_cube_uv(0.1, 0.04, 0.01);
    front_endplate_r.apply(Transform::translate(0.55, 0.04, 0.26));

    let mut cockpit: UnpackedMesh = generate_cube_uv(0.15, 0.06, 0.12);
    cockpit.apply(Transform::translate(-0.05, 0.15, 0.0));

    let mut helmet: UnpackedMesh = generate_sphere_uv(0.045, 6, 4);
    helmet.apply(Transform::translate(-0.05, 0.2, 0.0));

    let mut sidepod_l: UnpackedMesh = generate_cube_uv(0.2, 0.06, 0.1);
    sidepod_l.apply(Transform::translate(-0.1, 0.1, -0.2));

    let mut sidepod_r: UnpackedMesh = generate_cube_uv(0.2, 0.06, 0.1);
    sidepod_r.apply(Transform::translate(-0.1, 0.1, 0.2));

    let mut engine: UnpackedMesh = generate_cube_uv(0.25, 0.08, 0.15);
    engine.apply(Transform::translate(-0.3, 0.12, 0.0));

    let mut rear_wing: UnpackedMesh = generate_cube_uv(0.12, 0.02, 0.45);
    rear_wing.apply(Transform::translate(-0.48, 0.28, 0.0));

    let mut wing_support_l: UnpackedMesh = generate_cube_uv(0.02, 0.14, 0.02);
    wing_support_l.apply(Transform::translate(-0.48, 0.2, -0.2));

    let mut wing_support_r: UnpackedMesh = generate_cube_uv(0.02, 0.14, 0.02);
    wing_support_r.apply(Transform::translate(-0.48, 0.2, 0.2));

    let mut rear_endplate_l: UnpackedMesh = generate_cube_uv(0.15, 0.12, 0.01);
    rear_endplate_l.apply(Transform::translate(-0.48, 0.24, -0.24));

    let mut rear_endplate_r: UnpackedMesh = generate_cube_uv(0.15, 0.12, 0.01);
    rear_endplate_r.apply(Transform::translate(-0.48, 0.24, 0.24));

    let mut front_arm_l: UnpackedMesh = generate_cylinder_uv(0.01, 0.01, 0.2, 4);
    front_arm_l.apply(Transform::rotate_x(90.0));
    front_arm_l.apply(Transform::translate(0.35, 0.08, -0.15));

    let mut front_arm_r: UnpackedMesh = generate_cylinder_uv(0.01, 0.01, 0.2, 4);
    front_arm_r.apply(Transform::rotate_x(90.0));
    front_arm_r.apply(Transform::translate(0.35, 0.08, 0.15));

    let mut wheel_fl = generate_wheel(0.09, 0.07);
    wheel_fl.apply(Transform::translate(0.35, 0.09, -0.38));

    let mut wheel_fr = generate_wheel(0.09, 0.07);
    wheel_fr.apply(Transform::translate(0.35, 0.09, 0.38));

    let mut wheel_rl = generate_wheel(0.11, 0.09);
    wheel_rl.apply(Transform::translate(-0.35, 0.11, -0.38));

    let mut wheel_rr = generate_wheel(0.11, 0.09);
    wheel_rr.apply(Transform::translate(-0.35, 0.11, 0.38));

    let mesh = combine(&[
        &body, &nose, &front_wing, &front_endplate_l, &front_endplate_r,
        &cockpit, &helmet, &sidepod_l, &sidepod_r, &engine,
        &rear_wing, &wing_support_l, &wing_support_r,
        &rear_endplate_l, &rear_endplate_r,
        &front_arm_l, &front_arm_r,
        &wheel_fl, &wheel_fr, &wheel_rl, &wheel_rr,
    ]);

    let path = output_dir.join("racer.obj");
    write_obj(&mesh, &path, "racer").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// DRIFT - "The Slider"
pub fn generate_drift(output_dir: &Path) {
    println!("  Generating: drift.obj");

    let mut body: UnpackedMesh = generate_cube_uv(0.55, 0.16, 0.38);
    body.apply(Transform::translate(0.0, 0.14, 0.0));

    let mut hood: UnpackedMesh = generate_cube_uv(0.25, 0.04, 0.34);
    hood.apply(Transform::translate(0.28, 0.2, 0.0));
    hood.apply(Transform::rotate_z(-8.5));

    let mut windshield: UnpackedMesh = generate_cube_uv(0.1, 0.13, 0.32);
    windshield.apply(Transform::translate(0.08, 0.26, 0.0));
    windshield.apply(Transform::rotate_z(-26.0));

    let mut roof: UnpackedMesh = generate_cube_uv(0.2, 0.05, 0.32);
    roof.apply(Transform::translate(-0.08, 0.3, 0.0));

    let mut hatch: UnpackedMesh = generate_cube_uv(0.18, 0.1, 0.32);
    hatch.apply(Transform::translate(-0.28, 0.22, 0.0));
    hatch.apply(Transform::rotate_z(11.5));

    let mut fender_fl: UnpackedMesh = generate_cube_uv(0.14, 0.08, 0.06);
    fender_fl.apply(Transform::translate(0.24, 0.16, -0.24));

    let mut fender_fr: UnpackedMesh = generate_cube_uv(0.14, 0.08, 0.06);
    fender_fr.apply(Transform::translate(0.24, 0.16, 0.24));

    let mut fender_rl: UnpackedMesh = generate_cube_uv(0.16, 0.1, 0.08);
    fender_rl.apply(Transform::translate(-0.22, 0.16, -0.26));

    let mut fender_rr: UnpackedMesh = generate_cube_uv(0.16, 0.1, 0.08);
    fender_rr.apply(Transform::translate(-0.22, 0.16, 0.26));

    let mut splitter: UnpackedMesh = generate_cube_uv(0.06, 0.02, 0.42);
    splitter.apply(Transform::translate(0.45, 0.06, 0.0));

    let mut skirt_l: UnpackedMesh = generate_cube_uv(0.4, 0.03, 0.02);
    skirt_l.apply(Transform::translate(0.0, 0.06, -0.20));

    let mut skirt_r: UnpackedMesh = generate_cube_uv(0.4, 0.03, 0.02);
    skirt_r.apply(Transform::translate(0.0, 0.06, 0.20));

    let mut spoiler: UnpackedMesh = generate_cube_uv(0.08, 0.04, 0.36);
    spoiler.apply(Transform::translate(-0.38, 0.28, 0.0));
    spoiler.apply(Transform::rotate_z(-17.0));

    let mut diffuser: UnpackedMesh = generate_cube_uv(0.12, 0.04, 0.3);
    diffuser.apply(Transform::translate(-0.42, 0.08, 0.0));

    let wheels = generate_wheels(0.28, -0.28, 0.09, 0.36, 0.09, 0.08);

    let mut well_fl: UnpackedMesh = generate_torus_uv(0.11, 0.01, 12, 4);
    well_fl.apply(Transform::rotate_x(90.0));
    well_fl.apply(Transform::translate(0.28, 0.09, -0.22));

    let mut well_fr: UnpackedMesh = generate_torus_uv(0.11, 0.01, 12, 4);
    well_fr.apply(Transform::rotate_x(90.0));
    well_fr.apply(Transform::translate(0.28, 0.09, 0.22));

    let mut well_rl: UnpackedMesh = generate_torus_uv(0.11, 0.01, 12, 4);
    well_rl.apply(Transform::rotate_x(90.0));
    well_rl.apply(Transform::translate(-0.28, 0.09, -0.22));

    let mut well_rr: UnpackedMesh = generate_torus_uv(0.11, 0.01, 12, 4);
    well_rr.apply(Transform::rotate_x(90.0));
    well_rr.apply(Transform::translate(-0.28, 0.09, 0.22));

    let mut headlight_l: UnpackedMesh = generate_cube_uv(0.06, 0.03, 0.08);
    headlight_l.apply(Transform::translate(0.42, 0.2, -0.14));

    let mut headlight_r: UnpackedMesh = generate_cube_uv(0.06, 0.03, 0.08);
    headlight_r.apply(Transform::translate(0.42, 0.2, 0.14));

    let wheel_refs: Vec<&UnpackedMesh> = wheels.iter().collect();
    let mut parts: Vec<&UnpackedMesh> = vec![
        &body, &hood, &windshield, &roof, &hatch,
        &fender_fl, &fender_fr, &fender_rl, &fender_rr,
        &splitter, &skirt_l, &skirt_r, &spoiler, &diffuser,
        &well_fl, &well_fr, &well_rl, &well_rr,
        &headlight_l, &headlight_r,
    ];
    parts.extend(wheel_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("drift.obj");
    write_obj(&mesh, &path, "drift").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// PHANTOM - "The Ghost"
/// Stealth supercar with angular low-slung silhouette
pub fn generate_phantom(output_dir: &Path) {
    println!("  Generating: phantom.obj");

    // Extremely low wedge body
    let mut body: UnpackedMesh = generate_cube_uv(0.75, 0.14, 0.42);
    body.apply(Transform::translate(0.0, 0.11, 0.0));

    // Long pointed nose
    let mut nose: UnpackedMesh = generate_cube_uv(0.35, 0.06, 0.36);
    nose.apply(Transform::translate(0.42, 0.08, 0.0));
    nose.apply(Transform::rotate_z(-5.0));

    // Angular hood vents (left and right)
    let mut vent_l: UnpackedMesh = generate_cube_uv(0.15, 0.02, 0.06);
    vent_l.apply(Transform::translate(0.25, 0.16, -0.12));
    vent_l.apply(Transform::rotate_z(-12.0));

    let mut vent_r: UnpackedMesh = generate_cube_uv(0.15, 0.02, 0.06);
    vent_r.apply(Transform::translate(0.25, 0.16, 0.12));
    vent_r.apply(Transform::rotate_z(-12.0));

    // Sharp wedge windshield (-35 degrees)
    let mut windshield: UnpackedMesh = generate_cube_uv(0.12, 0.10, 0.34);
    windshield.apply(Transform::translate(0.05, 0.20, 0.0));
    windshield.apply(Transform::rotate_z(-35.0));

    // Very low roof
    let mut roof: UnpackedMesh = generate_cube_uv(0.22, 0.04, 0.32);
    roof.apply(Transform::translate(-0.10, 0.22, 0.0));

    // Dramatic rear haunches over wheels
    let mut haunch_l: UnpackedMesh = generate_cube_uv(0.22, 0.10, 0.12);
    haunch_l.apply(Transform::translate(-0.30, 0.16, -0.22));

    let mut haunch_r: UnpackedMesh = generate_cube_uv(0.22, 0.10, 0.12);
    haunch_r.apply(Transform::translate(-0.30, 0.16, 0.22));

    // Integrated ducktail spoiler
    let mut spoiler: UnpackedMesh = generate_cube_uv(0.10, 0.025, 0.38);
    spoiler.apply(Transform::translate(-0.48, 0.17, 0.0));
    spoiler.apply(Transform::rotate_z(-8.0));

    // Aggressive front splitter
    let mut splitter: UnpackedMesh = generate_cube_uv(0.12, 0.015, 0.44);
    splitter.apply(Transform::translate(0.58, 0.04, 0.0));

    // Side skirts with vent detail
    let mut skirt_l: UnpackedMesh = generate_cube_uv(0.45, 0.04, 0.025);
    skirt_l.apply(Transform::translate(0.0, 0.05, -0.22));

    let mut skirt_r: UnpackedMesh = generate_cube_uv(0.45, 0.04, 0.025);
    skirt_r.apply(Transform::translate(0.0, 0.05, 0.22));

    // Side vent cutouts
    let mut vent_side_l: UnpackedMesh = generate_cube_uv(0.08, 0.02, 0.015);
    vent_side_l.apply(Transform::translate(-0.12, 0.10, -0.215));

    let mut vent_side_r: UnpackedMesh = generate_cube_uv(0.08, 0.02, 0.015);
    vent_side_r.apply(Transform::translate(-0.12, 0.10, 0.215));

    // Twin exhaust tips
    let mut exhaust_l: UnpackedMesh = generate_cylinder_uv(0.022, 0.022, 0.08, 6);
    exhaust_l.apply(Transform::translate(-0.54, 0.07, -0.10));

    let mut exhaust_r: UnpackedMesh = generate_cylinder_uv(0.022, 0.022, 0.08, 6);
    exhaust_r.apply(Transform::translate(-0.54, 0.07, 0.10));

    // Sleek headlights
    let mut headlight_l: UnpackedMesh = generate_cube_uv(0.03, 0.015, 0.10);
    headlight_l.apply(Transform::translate(0.60, 0.10, -0.14));

    let mut headlight_r: UnpackedMesh = generate_cube_uv(0.03, 0.015, 0.10);
    headlight_r.apply(Transform::translate(0.60, 0.10, 0.14));

    // Thin taillights
    let mut taillight_l: UnpackedMesh = generate_cube_uv(0.015, 0.02, 0.14);
    taillight_l.apply(Transform::translate(-0.54, 0.14, -0.10));

    let mut taillight_r: UnpackedMesh = generate_cube_uv(0.015, 0.02, 0.14);
    taillight_r.apply(Transform::translate(-0.54, 0.14, 0.10));

    // Wheels (low profile)
    let wheels = generate_wheels(0.36, -0.36, 0.08, 0.38, 0.08, 0.06);

    let wheel_refs: Vec<&UnpackedMesh> = wheels.iter().collect();
    let mut parts: Vec<&UnpackedMesh> = vec![
        &body, &nose, &vent_l, &vent_r, &windshield, &roof,
        &haunch_l, &haunch_r, &spoiler, &splitter,
        &skirt_l, &skirt_r, &vent_side_l, &vent_side_r,
        &exhaust_l, &exhaust_r,
        &headlight_l, &headlight_r, &taillight_l, &taillight_r,
    ];
    parts.extend(wheel_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("phantom.obj");
    write_obj(&mesh, &path, "phantom").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// TITAN - "The Tank"
/// Heavy luxury GT cruiser, wide and imposing
pub fn generate_titan(output_dir: &Path) {
    println!("  Generating: titan.obj");

    // Wide boxy body
    let mut body: UnpackedMesh = generate_cube_uv(0.72, 0.22, 0.50);
    body.apply(Transform::translate(0.0, 0.16, 0.0));

    // Tall hood with grille lines
    let mut hood: UnpackedMesh = generate_cube_uv(0.32, 0.06, 0.46);
    hood.apply(Transform::translate(0.30, 0.26, 0.0));

    // Grille vertical bars
    let mut grille: UnpackedMesh = generate_cube_uv(0.04, 0.12, 0.40);
    grille.apply(Transform::translate(0.52, 0.20, 0.0));

    // Upright windshield (-12 degrees)
    let mut windshield: UnpackedMesh = generate_cube_uv(0.12, 0.16, 0.44);
    windshield.apply(Transform::translate(0.08, 0.32, 0.0));
    windshield.apply(Transform::rotate_z(-12.0));

    // Long roof with thick C-pillars
    let mut roof: UnpackedMesh = generate_cube_uv(0.30, 0.07, 0.42);
    roof.apply(Transform::translate(-0.12, 0.38, 0.0));

    // C-pillars
    let mut cpillar_l: UnpackedMesh = generate_cube_uv(0.12, 0.12, 0.05);
    cpillar_l.apply(Transform::translate(-0.28, 0.32, -0.22));
    cpillar_l.apply(Transform::rotate_z(15.0));

    let mut cpillar_r: UnpackedMesh = generate_cube_uv(0.12, 0.12, 0.05);
    cpillar_r.apply(Transform::translate(-0.28, 0.32, 0.22));
    cpillar_r.apply(Transform::rotate_z(15.0));

    // Trunk
    let mut trunk: UnpackedMesh = generate_cube_uv(0.22, 0.14, 0.46);
    trunk.apply(Transform::translate(-0.38, 0.20, 0.0));

    // Pronounced front bumper
    let mut front_bumper: UnpackedMesh = generate_cube_uv(0.06, 0.08, 0.48);
    front_bumper.apply(Transform::translate(0.55, 0.10, 0.0));

    // Pronounced rear bumper
    let mut rear_bumper: UnpackedMesh = generate_cube_uv(0.06, 0.08, 0.48);
    rear_bumper.apply(Transform::translate(-0.52, 0.10, 0.0));

    // Chunky wheel arches
    let mut arch_fl: UnpackedMesh = generate_cube_uv(0.16, 0.12, 0.10);
    arch_fl.apply(Transform::translate(0.34, 0.18, -0.28));

    let mut arch_fr: UnpackedMesh = generate_cube_uv(0.16, 0.12, 0.10);
    arch_fr.apply(Transform::translate(0.34, 0.18, 0.28));

    let mut arch_rl: UnpackedMesh = generate_cube_uv(0.18, 0.14, 0.10);
    arch_rl.apply(Transform::translate(-0.32, 0.18, -0.28));

    let mut arch_rr: UnpackedMesh = generate_cube_uv(0.18, 0.14, 0.10);
    arch_rr.apply(Transform::translate(-0.32, 0.18, 0.28));

    // Twin exhaust stacks
    let mut exhaust_l: UnpackedMesh = generate_cylinder_uv(0.028, 0.028, 0.10, 6);
    exhaust_l.apply(Transform::translate(-0.56, 0.08, -0.16));

    let mut exhaust_r: UnpackedMesh = generate_cylinder_uv(0.028, 0.028, 0.10, 6);
    exhaust_r.apply(Transform::translate(-0.56, 0.08, 0.16));

    // Subtle rear lip spoiler
    let mut lip: UnpackedMesh = generate_cube_uv(0.06, 0.02, 0.44);
    lip.apply(Transform::translate(-0.48, 0.28, 0.0));

    // Large headlights
    let mut headlight_l: UnpackedMesh = generate_cube_uv(0.05, 0.06, 0.10);
    headlight_l.apply(Transform::translate(0.55, 0.22, -0.18));

    let mut headlight_r: UnpackedMesh = generate_cube_uv(0.05, 0.06, 0.10);
    headlight_r.apply(Transform::translate(0.55, 0.22, 0.18));

    // Wide taillights
    let mut taillight_l: UnpackedMesh = generate_cube_uv(0.03, 0.05, 0.12);
    taillight_l.apply(Transform::translate(-0.52, 0.22, -0.16));

    let mut taillight_r: UnpackedMesh = generate_cube_uv(0.03, 0.05, 0.12);
    taillight_r.apply(Transform::translate(-0.52, 0.22, 0.16));

    // Large wheels
    let wheels = generate_wheels(0.34, -0.34, 0.11, 0.44, 0.11, 0.09);

    let wheel_refs: Vec<&UnpackedMesh> = wheels.iter().collect();
    let mut parts: Vec<&UnpackedMesh> = vec![
        &body, &hood, &grille, &windshield, &roof,
        &cpillar_l, &cpillar_r, &trunk,
        &front_bumper, &rear_bumper,
        &arch_fl, &arch_fr, &arch_rl, &arch_rr,
        &exhaust_l, &exhaust_r, &lip,
        &headlight_l, &headlight_r, &taillight_l, &taillight_r,
    ];
    parts.extend(wheel_refs);

    let mesh = combine(&parts);

    let path = output_dir.join("titan.obj");
    write_obj(&mesh, &path, "titan").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

/// VIPER - "The Strike"
/// Ultra-aggressive hypercar with extreme aerodynamics
pub fn generate_viper(output_dir: &Path) {
    println!("  Generating: viper.obj");

    // Low capsule body (cockpit forward like LMP)
    let mut body: UnpackedMesh = generate_capsule_uv(0.10, 0.50, 8, 4);
    body.apply(Transform::scale(1.0, 0.5, 0.9));
    body.apply(Transform::translate(-0.05, 0.12, 0.0));

    // Extreme wedge nose (nearly flat)
    let mut nose: UnpackedMesh = generate_cube_uv(0.40, 0.04, 0.32);
    nose.apply(Transform::translate(0.40, 0.06, 0.0));
    nose.apply(Transform::rotate_z(-3.0));

    // Cockpit bubble (forward position)
    let mut cockpit: UnpackedMesh = generate_capsule_uv(0.08, 0.16, 6, 4);
    cockpit.apply(Transform::scale(1.0, 0.7, 1.0));
    cockpit.apply(Transform::translate(0.12, 0.18, 0.0));

    // Helmet
    let mut helmet: UnpackedMesh = generate_sphere_uv(0.05, 6, 4);
    helmet.apply(Transform::translate(0.12, 0.24, 0.0));

    // Dive planes (front canards)
    let mut canard_l: UnpackedMesh = generate_cube_uv(0.10, 0.015, 0.08);
    canard_l.apply(Transform::translate(0.48, 0.08, -0.22));
    canard_l.apply(Transform::rotate_z(-15.0));

    let mut canard_r: UnpackedMesh = generate_cube_uv(0.10, 0.015, 0.08);
    canard_r.apply(Transform::translate(0.48, 0.08, 0.22));
    canard_r.apply(Transform::rotate_z(-15.0));

    // Side-mounted radiator inlets
    let mut inlet_l: UnpackedMesh = generate_cube_uv(0.18, 0.08, 0.08);
    inlet_l.apply(Transform::translate(0.0, 0.12, -0.24));

    let mut inlet_r: UnpackedMesh = generate_cube_uv(0.18, 0.08, 0.08);
    inlet_r.apply(Transform::translate(0.0, 0.12, 0.24));

    // Engine cover
    let mut engine: UnpackedMesh = generate_cube_uv(0.28, 0.08, 0.22);
    engine.apply(Transform::translate(-0.28, 0.14, 0.0));

    // Massive rear wing on pylons
    let mut wing: UnpackedMesh = generate_cube_uv(0.16, 0.025, 0.52);
    wing.apply(Transform::translate(-0.50, 0.35, 0.0));

    let mut pylon_l: UnpackedMesh = generate_cube_uv(0.025, 0.18, 0.025);
    pylon_l.apply(Transform::translate(-0.50, 0.24, -0.22));

    let mut pylon_r: UnpackedMesh = generate_cube_uv(0.025, 0.18, 0.025);
    pylon_r.apply(Transform::translate(-0.50, 0.24, 0.22));

    // Wing endplates
    let mut endplate_l: UnpackedMesh = generate_cube_uv(0.18, 0.10, 0.015);
    endplate_l.apply(Transform::translate(-0.50, 0.32, -0.27));

    let mut endplate_r: UnpackedMesh = generate_cube_uv(0.18, 0.10, 0.015);
    endplate_r.apply(Transform::translate(-0.50, 0.32, 0.27));

    // Dramatic rear diffuser with fins
    let mut diffuser: UnpackedMesh = generate_cube_uv(0.16, 0.05, 0.36);
    diffuser.apply(Transform::translate(-0.52, 0.06, 0.0));
    diffuser.apply(Transform::rotate_z(12.0));

    // Diffuser fins
    let mut fin1: UnpackedMesh = generate_cube_uv(0.12, 0.04, 0.01);
    fin1.apply(Transform::translate(-0.52, 0.06, -0.12));

    let mut fin2: UnpackedMesh = generate_cube_uv(0.12, 0.04, 0.01);
    fin2.apply(Transform::translate(-0.52, 0.06, 0.0));

    let mut fin3: UnpackedMesh = generate_cube_uv(0.12, 0.04, 0.01);
    fin3.apply(Transform::translate(-0.52, 0.06, 0.12));

    // Center-exit exhaust
    let mut exhaust: UnpackedMesh = generate_cylinder_uv(0.035, 0.035, 0.10, 8);
    exhaust.apply(Transform::translate(-0.56, 0.10, 0.0));

    // Exposed suspension elements (front)
    let mut susp_fl: UnpackedMesh = generate_cylinder_uv(0.012, 0.012, 0.18, 4);
    susp_fl.apply(Transform::rotate_x(90.0));
    susp_fl.apply(Transform::translate(0.38, 0.08, -0.20));

    let mut susp_fr: UnpackedMesh = generate_cylinder_uv(0.012, 0.012, 0.18, 4);
    susp_fr.apply(Transform::rotate_x(90.0));
    susp_fr.apply(Transform::translate(0.38, 0.08, 0.20));

    // Aggressive headlights
    let mut headlight_l: UnpackedMesh = generate_cube_uv(0.04, 0.02, 0.08);
    headlight_l.apply(Transform::translate(0.58, 0.08, -0.12));

    let mut headlight_r: UnpackedMesh = generate_cube_uv(0.04, 0.02, 0.08);
    headlight_r.apply(Transform::translate(0.58, 0.08, 0.12));

    // Wide rear lights (spanning width)
    let mut taillight: UnpackedMesh = generate_cube_uv(0.02, 0.025, 0.32);
    taillight.apply(Transform::translate(-0.54, 0.16, 0.0));

    // Wheels (wide track, low profile)
    let mut wheel_fl = generate_wheel(0.09, 0.08);
    wheel_fl.apply(Transform::translate(0.38, 0.09, -0.38));

    let mut wheel_fr = generate_wheel(0.09, 0.08);
    wheel_fr.apply(Transform::translate(0.38, 0.09, 0.38));

    let mut wheel_rl = generate_wheel(0.11, 0.10);
    wheel_rl.apply(Transform::translate(-0.38, 0.11, -0.40));

    let mut wheel_rr = generate_wheel(0.11, 0.10);
    wheel_rr.apply(Transform::translate(-0.38, 0.11, 0.40));

    let mesh = combine(&[
        &body, &nose, &cockpit, &helmet,
        &canard_l, &canard_r, &inlet_l, &inlet_r, &engine,
        &wing, &pylon_l, &pylon_r, &endplate_l, &endplate_r,
        &diffuser, &fin1, &fin2, &fin3, &exhaust,
        &susp_fl, &susp_fr,
        &headlight_l, &headlight_r, &taillight,
        &wheel_fl, &wheel_fr, &wheel_rl, &wheel_rr,
    ]);

    let path = output_dir.join("viper.obj");
    write_obj(&mesh, &path, "viper").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}
