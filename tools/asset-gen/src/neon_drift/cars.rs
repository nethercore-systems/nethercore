//! Vehicle generators for NEON DRIFT
//!
//! Four distinct car classes with unique silhouettes and neon accents.

use proc_gen::mesh::*;
use std::path::Path;

/// Generate a wheel mesh
fn generate_wheel(radius: f32, width: f32) -> UnpackedMesh {
    let mut wheel: UnpackedMesh = generate_cylinder(radius, radius, width, 12);
    wheel.apply(Transform::rotate_z(90.0));
    wheel
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

    let mut body: UnpackedMesh = generate_capsule(0.15, 0.6, 8, 4);
    body.apply(Transform::scale(1.0, 0.6, 1.8));
    body.apply(Transform::translate(0.0, 0.12, 0.0));

    let mut hood: UnpackedMesh = generate_cube(0.45, 0.08, 0.35);
    hood.apply(Transform::translate(0.35, 0.12, 0.0));

    let mut windshield: UnpackedMesh = generate_cube(0.15, 0.12, 0.32);
    windshield.apply(Transform::translate(0.1, 0.22, 0.0));
    windshield.apply(Transform::rotate_z(-17.0));

    let mut roof: UnpackedMesh = generate_cube(0.25, 0.06, 0.30);
    roof.apply(Transform::translate(-0.1, 0.25, 0.0));

    let mut rear: UnpackedMesh = generate_cube(0.3, 0.08, 0.34);
    rear.apply(Transform::translate(-0.35, 0.14, 0.0));

    let mut skirt_l: UnpackedMesh = generate_cube(0.5, 0.03, 0.02);
    skirt_l.apply(Transform::translate(0.0, 0.04, -0.38));

    let mut skirt_r: UnpackedMesh = generate_cube(0.5, 0.03, 0.02);
    skirt_r.apply(Transform::translate(0.0, 0.04, 0.38));

    let wheels = generate_wheels(0.32, -0.32, 0.08, 0.40, 0.08, 0.06);

    let mut headlight_l: UnpackedMesh = generate_cube(0.04, 0.03, 0.06);
    headlight_l.apply(Transform::translate(0.58, 0.12, -0.15));

    let mut headlight_r: UnpackedMesh = generate_cube(0.04, 0.03, 0.06);
    headlight_r.apply(Transform::translate(0.58, 0.12, 0.15));

    let mut taillight_l: UnpackedMesh = generate_cube(0.02, 0.04, 0.08);
    taillight_l.apply(Transform::translate(-0.52, 0.14, -0.16));

    let mut taillight_r: UnpackedMesh = generate_cube(0.02, 0.04, 0.08);
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

    let mut body: UnpackedMesh = generate_cube(0.7, 0.18, 0.45);
    body.apply(Transform::translate(0.0, 0.14, 0.0));

    let mut hood: UnpackedMesh = generate_cube(0.35, 0.06, 0.40);
    hood.apply(Transform::translate(0.25, 0.22, 0.0));

    let mut scoop: UnpackedMesh = generate_cube(0.12, 0.08, 0.15);
    scoop.apply(Transform::translate(0.25, 0.28, 0.0));

    let mut intake: UnpackedMesh = generate_cylinder(0.04, 0.04, 0.08, 6);
    intake.apply(Transform::translate(0.25, 0.34, 0.0));

    let mut windshield: UnpackedMesh = generate_cube(0.12, 0.14, 0.38);
    windshield.apply(Transform::translate(0.0, 0.28, 0.0));
    windshield.apply(Transform::rotate_z(-14.0));

    let mut roof: UnpackedMesh = generate_cube(0.22, 0.08, 0.36);
    roof.apply(Transform::translate(-0.12, 0.32, 0.0));

    let mut trunk: UnpackedMesh = generate_cube(0.2, 0.12, 0.42);
    trunk.apply(Transform::translate(-0.35, 0.18, 0.0));

    let mut fender_fl: UnpackedMesh = generate_cube(0.15, 0.1, 0.08);
    fender_fl.apply(Transform::translate(0.35, 0.15, -0.28));

    let mut fender_fr: UnpackedMesh = generate_cube(0.15, 0.1, 0.08);
    fender_fr.apply(Transform::translate(0.35, 0.15, 0.28));

    let mut fender_rl: UnpackedMesh = generate_cube(0.18, 0.12, 0.1);
    fender_rl.apply(Transform::translate(-0.3, 0.16, -0.30));

    let mut fender_rr: UnpackedMesh = generate_cube(0.18, 0.12, 0.1);
    fender_rr.apply(Transform::translate(-0.3, 0.16, 0.30));

    let mut exhaust_l: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.15, 6);
    exhaust_l.apply(Transform::rotate_x(90.0));
    exhaust_l.apply(Transform::translate(-0.2, 0.06, -0.32));

    let mut exhaust_r: UnpackedMesh = generate_cylinder(0.025, 0.025, 0.15, 6);
    exhaust_r.apply(Transform::rotate_x(90.0));
    exhaust_r.apply(Transform::translate(-0.2, 0.06, 0.32));

    let mut wheels = generate_wheels(0.32, -0.32, 0.1, 0.42, 0.1, 0.08);
    wheels[2].apply(Transform::scale(1.1, 1.1, 1.0));
    wheels[3].apply(Transform::scale(1.1, 1.1, 1.0));

    let mut headlight_l: UnpackedMesh = generate_cube(0.03, 0.05, 0.1);
    headlight_l.apply(Transform::translate(0.53, 0.16, -0.18));

    let mut headlight_r: UnpackedMesh = generate_cube(0.03, 0.05, 0.1);
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

    let mut body: UnpackedMesh = generate_capsule(0.08, 0.55, 8, 4);
    body.apply(Transform::scale(1.0, 0.6, 0.8));
    body.apply(Transform::translate(0.0, 0.1, 0.0));

    let mut nose: UnpackedMesh = generate_capsule(0.06, 0.25, 6, 3);
    nose.apply(Transform::scale(1.0, 0.5, 0.8));
    nose.apply(Transform::translate(0.45, 0.08, 0.0));

    let mut front_wing: UnpackedMesh = generate_cube(0.08, 0.015, 0.5);
    front_wing.apply(Transform::translate(0.55, 0.04, 0.0));

    let mut front_endplate_l: UnpackedMesh = generate_cube(0.1, 0.04, 0.01);
    front_endplate_l.apply(Transform::translate(0.55, 0.04, -0.26));

    let mut front_endplate_r: UnpackedMesh = generate_cube(0.1, 0.04, 0.01);
    front_endplate_r.apply(Transform::translate(0.55, 0.04, 0.26));

    let mut cockpit: UnpackedMesh = generate_cube(0.15, 0.06, 0.12);
    cockpit.apply(Transform::translate(-0.05, 0.15, 0.0));

    let mut helmet: UnpackedMesh = generate_sphere(0.045, 6, 4);
    helmet.apply(Transform::translate(-0.05, 0.2, 0.0));

    let mut sidepod_l: UnpackedMesh = generate_cube(0.2, 0.06, 0.1);
    sidepod_l.apply(Transform::translate(-0.1, 0.1, -0.2));

    let mut sidepod_r: UnpackedMesh = generate_cube(0.2, 0.06, 0.1);
    sidepod_r.apply(Transform::translate(-0.1, 0.1, 0.2));

    let mut engine: UnpackedMesh = generate_cube(0.25, 0.08, 0.15);
    engine.apply(Transform::translate(-0.3, 0.12, 0.0));

    let mut rear_wing: UnpackedMesh = generate_cube(0.12, 0.02, 0.45);
    rear_wing.apply(Transform::translate(-0.48, 0.28, 0.0));

    let mut wing_support_l: UnpackedMesh = generate_cube(0.02, 0.14, 0.02);
    wing_support_l.apply(Transform::translate(-0.48, 0.2, -0.2));

    let mut wing_support_r: UnpackedMesh = generate_cube(0.02, 0.14, 0.02);
    wing_support_r.apply(Transform::translate(-0.48, 0.2, 0.2));

    let mut rear_endplate_l: UnpackedMesh = generate_cube(0.15, 0.12, 0.01);
    rear_endplate_l.apply(Transform::translate(-0.48, 0.24, -0.24));

    let mut rear_endplate_r: UnpackedMesh = generate_cube(0.15, 0.12, 0.01);
    rear_endplate_r.apply(Transform::translate(-0.48, 0.24, 0.24));

    let mut front_arm_l: UnpackedMesh = generate_cylinder(0.01, 0.01, 0.2, 4);
    front_arm_l.apply(Transform::rotate_x(90.0));
    front_arm_l.apply(Transform::translate(0.35, 0.08, -0.15));

    let mut front_arm_r: UnpackedMesh = generate_cylinder(0.01, 0.01, 0.2, 4);
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

    let mut body: UnpackedMesh = generate_cube(0.55, 0.16, 0.38);
    body.apply(Transform::translate(0.0, 0.14, 0.0));

    let mut hood: UnpackedMesh = generate_cube(0.25, 0.04, 0.34);
    hood.apply(Transform::translate(0.28, 0.2, 0.0));
    hood.apply(Transform::rotate_z(-8.5));

    let mut windshield: UnpackedMesh = generate_cube(0.1, 0.13, 0.32);
    windshield.apply(Transform::translate(0.08, 0.26, 0.0));
    windshield.apply(Transform::rotate_z(-26.0));

    let mut roof: UnpackedMesh = generate_cube(0.2, 0.05, 0.32);
    roof.apply(Transform::translate(-0.08, 0.3, 0.0));

    let mut hatch: UnpackedMesh = generate_cube(0.18, 0.1, 0.32);
    hatch.apply(Transform::translate(-0.28, 0.22, 0.0));
    hatch.apply(Transform::rotate_z(11.5));

    let mut fender_fl: UnpackedMesh = generate_cube(0.14, 0.08, 0.06);
    fender_fl.apply(Transform::translate(0.24, 0.16, -0.24));

    let mut fender_fr: UnpackedMesh = generate_cube(0.14, 0.08, 0.06);
    fender_fr.apply(Transform::translate(0.24, 0.16, 0.24));

    let mut fender_rl: UnpackedMesh = generate_cube(0.16, 0.1, 0.08);
    fender_rl.apply(Transform::translate(-0.22, 0.16, -0.26));

    let mut fender_rr: UnpackedMesh = generate_cube(0.16, 0.1, 0.08);
    fender_rr.apply(Transform::translate(-0.22, 0.16, 0.26));

    let mut splitter: UnpackedMesh = generate_cube(0.06, 0.02, 0.42);
    splitter.apply(Transform::translate(0.45, 0.06, 0.0));

    let mut skirt_l: UnpackedMesh = generate_cube(0.4, 0.03, 0.02);
    skirt_l.apply(Transform::translate(0.0, 0.06, -0.20));

    let mut skirt_r: UnpackedMesh = generate_cube(0.4, 0.03, 0.02);
    skirt_r.apply(Transform::translate(0.0, 0.06, 0.20));

    let mut spoiler: UnpackedMesh = generate_cube(0.08, 0.04, 0.36);
    spoiler.apply(Transform::translate(-0.38, 0.28, 0.0));
    spoiler.apply(Transform::rotate_z(-17.0));

    let mut diffuser: UnpackedMesh = generate_cube(0.12, 0.04, 0.3);
    diffuser.apply(Transform::translate(-0.42, 0.08, 0.0));

    let wheels = generate_wheels(0.28, -0.28, 0.09, 0.36, 0.09, 0.08);

    let mut well_fl: UnpackedMesh = generate_torus(0.11, 0.01, 12, 4);
    well_fl.apply(Transform::rotate_x(90.0));
    well_fl.apply(Transform::translate(0.28, 0.09, -0.22));

    let mut well_fr: UnpackedMesh = generate_torus(0.11, 0.01, 12, 4);
    well_fr.apply(Transform::rotate_x(90.0));
    well_fr.apply(Transform::translate(0.28, 0.09, 0.22));

    let mut well_rl: UnpackedMesh = generate_torus(0.11, 0.01, 12, 4);
    well_rl.apply(Transform::rotate_x(90.0));
    well_rl.apply(Transform::translate(-0.28, 0.09, -0.22));

    let mut well_rr: UnpackedMesh = generate_torus(0.11, 0.01, 12, 4);
    well_rr.apply(Transform::rotate_x(90.0));
    well_rr.apply(Transform::translate(-0.28, 0.09, 0.22));

    let mut headlight_l: UnpackedMesh = generate_cube(0.06, 0.03, 0.08);
    headlight_l.apply(Transform::translate(0.42, 0.2, -0.14));

    let mut headlight_r: UnpackedMesh = generate_cube(0.06, 0.03, 0.08);
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
