//! PRISM SURVIVORS - Procedural Asset Generator
//!
//! Generates meshes and textures for the top-down survivors game.
//! Heroes: Knight, Mage, Ranger, Cleric
//! Enemies: Golem, Crawler, Wisp, Skeleton

use proc_gen::mesh::*;
use proc_gen::texture::*;
use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    let meshes_dir = output_dir.join("meshes");
    let textures_dir = output_dir.join("textures");

    std::fs::create_dir_all(&meshes_dir).unwrap();
    std::fs::create_dir_all(&textures_dir).unwrap();

    println!("  Meshes -> {}", meshes_dir.display());
    println!("  Textures -> {}", textures_dir.display());

    // Test assets
    generate_test_cube(&meshes_dir);
    generate_test_sphere(&meshes_dir);
    generate_test_texture(&textures_dir);

    // Heroes
    generate_knight(&meshes_dir);
    generate_mage(&meshes_dir);
    generate_ranger(&meshes_dir);
    generate_cleric(&meshes_dir);

    // Enemies
    generate_golem(&meshes_dir);
    generate_crawler(&meshes_dir);
    generate_wisp(&meshes_dir);
    generate_skeleton(&meshes_dir);

    // Textures
    generate_hero_textures(&textures_dir);
    generate_enemy_textures(&textures_dir);
}

// === TEST ASSETS ===

fn generate_test_cube(output_dir: &Path) {
    println!("  Generating: test_cube.obj");
    let mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
    let path = output_dir.join("test_cube.obj");
    write_obj(&mesh, &path, "test_cube").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_test_sphere(output_dir: &Path) {
    println!("  Generating: test_sphere.obj");
    let mesh: UnpackedMesh = generate_sphere(0.5, 16, 12);
    let path = output_dir.join("test_sphere.obj");
    write_obj(&mesh, &path, "test_sphere").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_test_texture(output_dir: &Path) {
    println!("  Generating: test_checker.png");
    let tex = checker(64, 64, 8, [200, 200, 200, 255], [50, 50, 50, 255]);
    let path = output_dir.join("test_checker.png");
    write_png(&tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());
}

// === HEROES ===

fn generate_knight(output_dir: &Path) {
    println!("  Generating: knight.obj");

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

    let path = output_dir.join("knight.obj");
    write_obj(&mesh, &path, "knight").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_mage(output_dir: &Path) {
    println!("  Generating: mage.obj");

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

    let path = output_dir.join("mage.obj");
    write_obj(&mesh, &path, "mage").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_ranger(output_dir: &Path) {
    println!("  Generating: ranger.obj");

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

    let path = output_dir.join("ranger.obj");
    write_obj(&mesh, &path, "ranger").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_cleric(output_dir: &Path) {
    println!("  Generating: cleric.obj");

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

    let path = output_dir.join("cleric.obj");
    write_obj(&mesh, &path, "cleric").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === ENEMIES ===

fn generate_golem(output_dir: &Path) {
    println!("  Generating: golem.obj");

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

    let path = output_dir.join("golem.obj");
    write_obj(&mesh, &path, "golem").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_crawler(output_dir: &Path) {
    println!("  Generating: crawler.obj");

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

    let path = output_dir.join("crawler.obj");
    write_obj(&mesh, &path, "crawler").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_wisp(output_dir: &Path) {
    println!("  Generating: wisp.obj");

    let mut core: UnpackedMesh = generate_sphere(0.12, 10, 8);
    core.apply(SmoothNormals::default());

    let glow: UnpackedMesh = generate_sphere(0.18, 8, 6);

    let mut tail: UnpackedMesh = generate_sphere(0.08, 6, 4);
    tail.apply(Transform::translate(-0.15, 0.0, 0.0));
    tail.apply(Transform::scale(2.0, 0.6, 0.6));

    let mesh = combine(&[&core, &glow, &tail]);

    let path = output_dir.join("wisp.obj");
    write_obj(&mesh, &path, "wisp").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

fn generate_skeleton(output_dir: &Path) {
    println!("  Generating: skeleton.obj");

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

    let path = output_dir.join("skeleton.obj");
    write_obj(&mesh, &path, "skeleton").expect("Failed to write OBJ file");
    println!(
        "    -> {} ({} verts, {} tris)",
        path.display(),
        mesh.positions.len(),
        mesh.indices.len() / 3
    );
}

// === TEXTURES ===

fn generate_hero_textures(output_dir: &Path) {
    println!("\n  Generating hero textures...");

    let knight_tex = metal(64, 64, [140, 140, 150, 255], 42);
    let path = output_dir.join("knight.png");
    write_png(&knight_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let mut mage_tex = gradient_v(64, 64, [80, 40, 120, 255], [40, 20, 80, 255]);
    mage_tex.apply(Contrast { factor: 1.1 });
    let path = output_dir.join("mage.png");
    write_png(&mage_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let ranger_tex = gradient_v(64, 64, [60, 80, 40, 255], [40, 50, 30, 255]);
    let path = output_dir.join("ranger.png");
    write_png(&ranger_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let cleric_tex = gradient_v(64, 64, [240, 230, 200, 255], [200, 180, 140, 255]);
    let path = output_dir.join("cleric.png");
    write_png(&cleric_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());
}

fn generate_enemy_textures(output_dir: &Path) {
    println!("\n  Generating enemy textures...");

    let golem_tex = stone(64, 64, [100, 90, 80, 255], 42);
    let path = output_dir.join("golem.png");
    write_png(&golem_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let mut crawler_tex = solid(64, 64, [40, 35, 45, 255]);
    crawler_tex.apply(Contrast { factor: 1.2 });
    let path = output_dir.join("crawler.png");
    write_png(&crawler_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let wisp_tex = gradient_radial(64, 64, [255, 200, 100, 255], [255, 150, 50, 200]);
    let path = output_dir.join("wisp.png");
    write_png(&wisp_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let skeleton_tex = solid(64, 64, [220, 210, 190, 255]);
    let path = output_dir.join("skeleton.png");
    write_png(&skeleton_tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());
}
