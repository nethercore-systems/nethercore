//! PRISM SURVIVORS - Procedural Asset Generator
//!
//! Generates meshes, textures, and sounds for the top-down survivors game.
//! Heroes: Knight, Mage, Ranger, Cleric, Necromancer, Paladin
//! Basic Enemies: Golem, Crawler, Wisp, Skeleton, Shade, Berserker, Arcane Sentinel
//! Elite Enemies: Crystal Knight, Void Mage, Golem Titan, Specter Lord
//! Bosses: Prism Colossus, Void Dragon
//! Effects: XP Gem, Weapons, Arena Floor, Projectiles

mod sounds;
mod textures;

use proc_gen::audio::*;
use proc_gen::mesh::*;
use proc_gen::texture::{checker, write_png, TextureBuffer};
use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    let meshes_dir = output_dir.join("meshes");
    let textures_dir = output_dir.join("textures");
    let audio_dir = output_dir.parent().unwrap().join("audio");

    std::fs::create_dir_all(&meshes_dir).unwrap();
    std::fs::create_dir_all(&textures_dir).unwrap();
    std::fs::create_dir_all(&audio_dir).unwrap();

    println!("  Meshes -> {}", meshes_dir.display());
    println!("  Textures -> {}", textures_dir.display());
    println!("  Audio -> {}", audio_dir.display());

    // Test assets
    generate_test_cube(&meshes_dir);
    generate_test_sphere(&meshes_dir);
    generate_test_texture(&textures_dir);

    // Heroes
    generate_knight(&meshes_dir);
    generate_mage(&meshes_dir);
    generate_ranger(&meshes_dir);
    generate_cleric(&meshes_dir);
    generate_necromancer(&meshes_dir);
    generate_paladin(&meshes_dir);

    // Basic Enemies
    generate_golem(&meshes_dir);
    generate_crawler(&meshes_dir);
    generate_wisp(&meshes_dir);
    generate_skeleton(&meshes_dir);
    generate_shade(&meshes_dir);
    generate_berserker(&meshes_dir);
    generate_arcane_sentinel(&meshes_dir);

    // Projectiles
    generate_frost_shard(&meshes_dir);
    generate_void_orb(&meshes_dir);
    generate_lightning_bolt(&meshes_dir);

    // Elite Enemies
    generate_crystal_knight(&meshes_dir);
    generate_void_mage(&meshes_dir);
    generate_golem_titan(&meshes_dir);
    generate_specter_lord(&meshes_dir);

    // Bosses
    generate_prism_colossus(&meshes_dir);
    generate_void_dragon(&meshes_dir);

    // Pickups & Effects
    generate_xp_gem(&meshes_dir);
    generate_coin(&meshes_dir);
    generate_powerup_orb(&meshes_dir);

    // Arena
    generate_arena_floor(&meshes_dir);

    // Textures
    textures::generate_hero_textures(&textures_dir);
    textures::generate_enemy_textures(&textures_dir);
    textures::generate_elite_textures(&textures_dir);
    textures::generate_boss_textures(&textures_dir);
    textures::generate_pickup_textures(&textures_dir);
    textures::generate_arena_textures(&textures_dir);
    textures::generate_projectile_textures(&textures_dir);

    // Font
    generate_font(&textures_dir);

    // Sounds
    generate_sounds(&audio_dir);
}

fn generate_sounds(output_dir: &Path) {
    println!("  Generating {} sounds", sounds::SOUNDS.len());

    let synth = Synth::new(SAMPLE_RATE);

    for (id, description) in sounds::SOUNDS {
        let samples = generate_prism_sound(&synth, id);
        let pcm = to_pcm_i16(&samples);
        let path = output_dir.join(format!("{}.wav", id));

        write_wav(&pcm, SAMPLE_RATE, &path).expect("Failed to write WAV file");

        println!(
            "    -> {}.wav ({} samples, {:.2}s) - {}",
            id,
            pcm.len(),
            pcm.len() as f32 / SAMPLE_RATE as f32,
            description
        );
    }
}

fn generate_prism_sound(synth: &Synth, id: &str) -> Vec<f32> {
    match id {
        // Combat
        "shoot" => synth.sweep(Waveform::Square, 800.0, 200.0, 0.08, Envelope::pluck()),
        "hit" => synth.noise_burst(0.05, Envelope::hit()),
        "death" => {
            let decay_env = Envelope::new(0.02, 0.2, 0.0, 0.15);
            let tone1 = synth.tone(Waveform::Saw, 440.0, 0.15, decay_env);
            let tone2 = synth.tone(Waveform::Saw, 349.0, 0.2, decay_env);
            let tone3 = synth.tone(Waveform::Saw, 294.0, 0.25, decay_env);
            mix(&[
                (&tone1, 0.4),
                (&tone2, 0.4),
                (&tone3, 0.4),
            ])
        }

        // Player
        "dash" => synth.sweep(Waveform::Triangle, 300.0, 600.0, 0.12, Envelope::pluck()),
        "level_up" => {
            let decay_env = Envelope::new(0.02, 0.15, 0.0, 0.1);
            let tone1 = synth.tone(Waveform::Square, 523.0, 0.1, Envelope::pluck());
            let tone2 = synth.tone(Waveform::Square, 659.0, 0.1, Envelope::pluck());
            let tone3 = synth.tone(Waveform::Square, 784.0, 0.15, decay_env);
            concat(&[&tone1, &tone2, &tone3])
        }
        "hurt" => synth.sweep(Waveform::Saw, 600.0, 300.0, 0.1, Envelope::hit()),

        // Pickups
        "xp" => synth.sweep(Waveform::Sine, 400.0, 800.0, 0.06, Envelope::pluck()),
        "coin" => synth.sweep(Waveform::Square, 600.0, 1200.0, 0.1, Envelope::pluck()),
        "powerup" => {
            let decay_env = Envelope::new(0.02, 0.15, 0.0, 0.1);
            let tone1 = synth.tone(Waveform::Square, 440.0, 0.12, Envelope::pluck());
            let tone2 = synth.tone(Waveform::Square, 554.0, 0.12, Envelope::pluck());
            let tone3 = synth.tone(Waveform::Square, 659.0, 0.15, decay_env);
            concat(&[&tone1, &tone2, &tone3])
        }

        // UI
        "menu" => synth.tone(Waveform::Sine, 800.0, 0.05, Envelope::pluck()),
        "select" => synth.tone(Waveform::Sine, 1000.0, 0.04, Envelope::pluck()),
        "back" => synth.tone(Waveform::Sine, 600.0, 0.05, Envelope::pluck()),

        _ => panic!("Unknown sound ID: {}", id),
    }
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

fn generate_necromancer(output_dir: &Path) {
    println!("  Generating: necromancer.obj");

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

    let path = output_dir.join("necromancer.obj");
    write_obj(&mesh, &path, "necromancer").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_paladin(output_dir: &Path) {
    println!("  Generating: paladin.obj");

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

    let mesh = combine(&[&torso, &head, &crest, &shoulder_r, &shoulder_l, &arm_r, &arm_l, &leg_r, &leg_l, &hammer_handle, &hammer_head, &shield]);

    let path = output_dir.join("paladin.obj");
    write_obj(&mesh, &path, "paladin").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
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

fn generate_shade(output_dir: &Path) {
    println!("  Generating: shade.obj");

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

    let path = output_dir.join("shade.obj");
    write_obj(&mesh, &path, "shade").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_berserker(output_dir: &Path) {
    println!("  Generating: berserker.obj");

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

    let mesh = combine(&[&torso, &head, &arm_r, &arm_l, &leg_r, &leg_l, &axe_handle, &axe_head1, &axe_head2]);

    let path = output_dir.join("berserker.obj");
    write_obj(&mesh, &path, "berserker").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_arcane_sentinel(output_dir: &Path) {
    println!("  Generating: arcane_sentinel.obj");

    // Floating magical construct
    let mut core: UnpackedMesh = generate_sphere(0.2, 10, 8);

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

    let path = output_dir.join("arcane_sentinel.obj");
    write_obj(&mesh, &path, "arcane_sentinel").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

// === PROJECTILES ===

fn generate_frost_shard(output_dir: &Path) {
    println!("  Generating: frost_shard.obj");

    // Icy crystal projectile
    let mut shard: UnpackedMesh = generate_cylinder(0.0, 0.08, 0.25, 6);

    // Trailing ice crystals
    let mut trail1: UnpackedMesh = generate_cylinder(0.0, 0.04, 0.1, 4);
    trail1.apply(Transform::translate(0.0, -0.12, 0.0));
    trail1.apply(Transform::rotate_z(25.0));

    let mut trail2: UnpackedMesh = generate_cylinder(0.0, 0.04, 0.1, 4);
    trail2.apply(Transform::translate(0.0, -0.12, 0.0));
    trail2.apply(Transform::rotate_z(-25.0));

    let mesh = combine(&[&shard, &trail1, &trail2]);

    let path = output_dir.join("frost_shard.obj");
    write_obj(&mesh, &path, "frost_shard").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_void_orb(output_dir: &Path) {
    println!("  Generating: void_orb.obj");

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

    let path = output_dir.join("void_orb.obj");
    write_obj(&mesh, &path, "void_orb").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_lightning_bolt(output_dir: &Path) {
    println!("  Generating: lightning_bolt.obj");

    // Jagged lightning projectile
    let mut segment1: UnpackedMesh = generate_cube(0.04, 0.15, 0.02);

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

    let path = output_dir.join("lightning_bolt.obj");
    write_obj(&mesh, &path, "lightning_bolt").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

// === ELITE ENEMIES ===

fn generate_crystal_knight(output_dir: &Path) {
    println!("  Generating: crystal_knight.obj");

    // Larger, more imposing knight with crystalline growths
    let mut torso: UnpackedMesh = generate_capsule(0.35, 0.55, 10, 5);
    torso.apply(Transform::scale(1.1, 1.0, 0.9));

    let mut head: UnpackedMesh = generate_sphere(0.2, 10, 8);
    head.apply(Transform::translate(0.0, 0.6, 0.0));

    // Crystal spikes on shoulders
    let mut crystal_r: UnpackedMesh = generate_cylinder(0.0, 0.1, 0.3, 6);
    crystal_r.apply(Transform::translate(0.4, 0.5, 0.0));
    crystal_r.apply(Transform::rotate_z(-30.0));

    let mut crystal_l: UnpackedMesh = generate_cylinder(0.0, 0.1, 0.3, 6);
    crystal_l.apply(Transform::translate(-0.4, 0.5, 0.0));
    crystal_l.apply(Transform::rotate_z(30.0));

    let mut arm_r: UnpackedMesh = generate_cylinder(0.1, 0.1, 0.4, 8);
    arm_r.apply(Transform::translate(0.45, 0.15, 0.0));
    arm_r.apply(Transform::rotate_z(-20.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.1, 0.1, 0.4, 8);
    arm_l.apply(Transform::translate(-0.45, 0.15, 0.0));
    arm_l.apply(Transform::rotate_z(20.0));

    let mut leg_r: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.45, 8);
    leg_r.apply(Transform::translate(0.15, -0.55, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.45, 8);
    leg_l.apply(Transform::translate(-0.15, -0.55, 0.0));

    // Large crystal sword
    let mut sword: UnpackedMesh = generate_cube(0.08, 0.6, 0.04);
    sword.apply(Transform::translate(0.55, 0.3, 0.0));

    let mesh = combine(&[&torso, &head, &crystal_r, &crystal_l, &arm_r, &arm_l, &leg_r, &leg_l, &sword]);

    let path = output_dir.join("crystal_knight.obj");
    write_obj(&mesh, &path, "crystal_knight").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_void_mage(output_dir: &Path) {
    println!("  Generating: void_mage.obj");

    // Taller, ethereal mage
    let mut robe: UnpackedMesh = generate_capsule(0.28, 0.7, 10, 5);
    robe.apply(Transform::scale(1.3, 1.0, 1.3));

    let mut head: UnpackedMesh = generate_sphere(0.18, 10, 8);
    head.apply(Transform::translate(0.0, 0.65, 0.0));

    let mut hood: UnpackedMesh = generate_sphere(0.24, 10, 5);
    hood.apply(Transform::translate(0.0, 0.68, -0.03));
    hood.apply(Transform::scale(1.0, 0.85, 1.3));

    // Floating void orbs
    let mut orb1: UnpackedMesh = generate_sphere(0.1, 8, 6);
    orb1.apply(Transform::translate(0.4, 0.4, 0.0));

    let mut orb2: UnpackedMesh = generate_sphere(0.08, 8, 6);
    orb2.apply(Transform::translate(-0.35, 0.5, 0.15));

    let mut orb3: UnpackedMesh = generate_sphere(0.12, 8, 6);
    orb3.apply(Transform::translate(0.0, 0.9, 0.0));

    let mesh = combine(&[&robe, &head, &hood, &orb1, &orb2, &orb3]);

    let path = output_dir.join("void_mage.obj");
    write_obj(&mesh, &path, "void_mage").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_golem_titan(output_dir: &Path) {
    println!("  Generating: golem_titan.obj");

    // Much larger golem with additional mass
    let mut body: UnpackedMesh = generate_cube(0.7, 0.85, 0.55);

    let mut head: UnpackedMesh = generate_cube(0.35, 0.28, 0.28);
    head.apply(Transform::translate(0.0, 0.6, 0.0));

    // Massive arms
    let mut arm_r: UnpackedMesh = generate_cylinder(0.2, 0.15, 0.6, 8);
    arm_r.apply(Transform::translate(0.55, 0.1, 0.0));

    let mut arm_l: UnpackedMesh = generate_cylinder(0.2, 0.15, 0.6, 8);
    arm_l.apply(Transform::translate(-0.55, 0.1, 0.0));

    // Fists
    let mut fist_r: UnpackedMesh = generate_sphere(0.2, 8, 6);
    fist_r.apply(Transform::translate(0.55, -0.3, 0.0));

    let mut fist_l: UnpackedMesh = generate_sphere(0.2, 8, 6);
    fist_l.apply(Transform::translate(-0.55, -0.3, 0.0));

    // Thick legs
    let mut leg_r: UnpackedMesh = generate_cylinder(0.2, 0.2, 0.5, 8);
    leg_r.apply(Transform::translate(0.25, -0.7, 0.0));

    let mut leg_l: UnpackedMesh = generate_cylinder(0.2, 0.2, 0.5, 8);
    leg_l.apply(Transform::translate(-0.25, -0.7, 0.0));

    // Boulder shoulder pads
    let mut shoulder_r: UnpackedMesh = generate_sphere(0.25, 8, 6);
    shoulder_r.apply(Transform::translate(0.45, 0.45, 0.0));

    let mut shoulder_l: UnpackedMesh = generate_sphere(0.25, 8, 6);
    shoulder_l.apply(Transform::translate(-0.45, 0.45, 0.0));

    let mesh = combine(&[&body, &head, &arm_r, &arm_l, &fist_r, &fist_l, &leg_r, &leg_l, &shoulder_r, &shoulder_l]);

    let path = output_dir.join("golem_titan.obj");
    write_obj(&mesh, &path, "golem_titan").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_specter_lord(output_dir: &Path) {
    println!("  Generating: specter_lord.obj");

    // Ethereal hovering specter
    let mut core: UnpackedMesh = generate_sphere(0.25, 12, 10);

    // Ghostly tail/train
    let mut tail: UnpackedMesh = generate_sphere(0.2, 10, 8);
    tail.apply(Transform::translate(0.0, -0.3, 0.0));
    tail.apply(Transform::scale(1.0, 2.0, 0.8));

    // Crown/horns
    let mut horn_r: UnpackedMesh = generate_cylinder(0.0, 0.06, 0.2, 5);
    horn_r.apply(Transform::translate(0.15, 0.3, 0.0));
    horn_r.apply(Transform::rotate_z(-20.0));

    let mut horn_l: UnpackedMesh = generate_cylinder(0.0, 0.06, 0.2, 5);
    horn_l.apply(Transform::translate(-0.15, 0.3, 0.0));
    horn_l.apply(Transform::rotate_z(20.0));

    // Floating hands
    let mut hand_r: UnpackedMesh = generate_sphere(0.1, 6, 5);
    hand_r.apply(Transform::translate(0.4, 0.1, 0.0));

    let mut hand_l: UnpackedMesh = generate_sphere(0.1, 6, 5);
    hand_l.apply(Transform::translate(-0.4, 0.1, 0.0));

    let mesh = combine(&[&core, &tail, &horn_r, &horn_l, &hand_r, &hand_l]);

    let path = output_dir.join("specter_lord.obj");
    write_obj(&mesh, &path, "specter_lord").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

// === BOSSES ===

fn generate_prism_colossus(output_dir: &Path) {
    println!("  Generating: prism_colossus.obj");

    // Massive crystalline humanoid
    let mut core: UnpackedMesh = generate_cube(1.0, 1.4, 0.8);

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

    let mesh = combine(&[&core, &prism, &head, &arm_r, &arm_l, &spike1, &spike2, &spike3, &leg_r, &leg_l]);

    let path = output_dir.join("prism_colossus.obj");
    write_obj(&mesh, &path, "prism_colossus").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_void_dragon(output_dir: &Path) {
    println!("  Generating: void_dragon.obj");

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

    let mesh = combine(&[&body, &neck, &head, &horn_r, &horn_l, &wing_r, &wing_l, &tail, &leg_fr, &leg_fl, &leg_br, &leg_bl]);

    let path = output_dir.join("void_dragon.obj");
    write_obj(&mesh, &path, "void_dragon").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

// === PICKUPS & EFFECTS ===

fn generate_xp_gem(output_dir: &Path) {
    println!("  Generating: xp_gem.obj");

    // Diamond/gem shape - two pyramids joined
    let mut top: UnpackedMesh = generate_cylinder(0.0, 0.15, 0.15, 6);
    let mut bottom: UnpackedMesh = generate_cylinder(0.15, 0.0, 0.1, 6);
    bottom.apply(Transform::translate(0.0, -0.1, 0.0));

    let mesh = combine(&[&top, &bottom]);

    let path = output_dir.join("xp_gem.obj");
    write_obj(&mesh, &path, "xp_gem").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_coin(output_dir: &Path) {
    println!("  Generating: coin.obj");

    let mesh: UnpackedMesh = generate_cylinder(0.12, 0.12, 0.03, 16);

    let path = output_dir.join("coin.obj");
    write_obj(&mesh, &path, "coin").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_powerup_orb(output_dir: &Path) {
    println!("  Generating: powerup_orb.obj");

    // Glowing orb with outer shell
    let inner: UnpackedMesh = generate_sphere(0.1, 10, 8);
    let mut outer: UnpackedMesh = generate_sphere(0.15, 8, 6);
    outer.apply(SmoothNormals::default());

    let mesh = combine(&[&inner, &outer]);

    let path = output_dir.join("powerup_orb.obj");
    write_obj(&mesh, &path, "powerup_orb").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

fn generate_arena_floor(output_dir: &Path) {
    println!("  Generating: arena_floor.obj");

    // Large flat arena floor
    let mesh: UnpackedMesh = generate_plane(40.0, 40.0, 8, 8);

    let path = output_dir.join("arena_floor.obj");
    write_obj(&mesh, &path, "arena_floor").expect("Failed to write OBJ file");
    println!("    -> {} ({} verts, {} tris)", path.display(), mesh.positions.len(), mesh.indices.len() / 3);
}

// === FONT ===

/// Generates a custom 8x12 bitmap font atlas for PRISM SURVIVORS
/// 96 ASCII characters (32-127) in a 128x72 texture
fn generate_font(output_dir: &Path) {
    println!("  Generating: prism_font.png");

    const GLYPH_W: usize = 8;
    const GLYPH_H: usize = 12;
    const CHARS_PER_ROW: usize = 16;
    const NUM_CHARS: usize = 96;
    const NUM_ROWS: usize = (NUM_CHARS + CHARS_PER_ROW - 1) / CHARS_PER_ROW; // 6 rows
    const TEX_W: usize = CHARS_PER_ROW * GLYPH_W; // 128
    const TEX_H: usize = NUM_ROWS * GLYPH_H;      // 72

    // PRISM SURVIVORS color scheme - cyan primary, purple glow
    let fg: [u8; 4] = [0, 255, 255, 255];      // Cyan
    let glow: [u8; 4] = [180, 100, 255, 255];  // Purple glow
    let bg: [u8; 4] = [0, 0, 0, 0];            // Transparent

    // 8x12 pixel font glyphs (ASCII 32-127)
    // Each glyph is 12 rows of 8 bits
    #[rustfmt::skip]
    const GLYPHS: [[u8; 12]; 96] = [
        // 32 SPACE
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 33 !
        [0x00, 0x18, 0x18, 0x18, 0x18, 0x18, 0x18, 0x00, 0x18, 0x18, 0x00, 0x00],
        // 34 "
        [0x00, 0x6C, 0x6C, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 35 #
        [0x00, 0x00, 0x24, 0x24, 0x7E, 0x24, 0x7E, 0x24, 0x24, 0x00, 0x00, 0x00],
        // 36 $
        [0x00, 0x10, 0x3C, 0x50, 0x38, 0x14, 0x78, 0x10, 0x00, 0x00, 0x00, 0x00],
        // 37 %
        [0x00, 0x00, 0x62, 0x64, 0x08, 0x10, 0x26, 0x46, 0x00, 0x00, 0x00, 0x00],
        // 38 &
        [0x00, 0x30, 0x48, 0x48, 0x30, 0x4A, 0x44, 0x3A, 0x00, 0x00, 0x00, 0x00],
        // 39 '
        [0x00, 0x18, 0x18, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 40 (
        [0x00, 0x08, 0x10, 0x20, 0x20, 0x20, 0x20, 0x10, 0x08, 0x00, 0x00, 0x00],
        // 41 )
        [0x00, 0x20, 0x10, 0x08, 0x08, 0x08, 0x08, 0x10, 0x20, 0x00, 0x00, 0x00],
        // 42 *
        [0x00, 0x00, 0x24, 0x18, 0x7E, 0x18, 0x24, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 43 +
        [0x00, 0x00, 0x00, 0x18, 0x18, 0x7E, 0x18, 0x18, 0x00, 0x00, 0x00, 0x00],
        // 44 ,
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x08, 0x10, 0x00],
        // 45 -
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 46 .
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00],
        // 47 /
        [0x00, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x80, 0x00, 0x00, 0x00, 0x00],
        // 48 0
        [0x00, 0x3C, 0x46, 0x4A, 0x4A, 0x52, 0x52, 0x62, 0x3C, 0x00, 0x00, 0x00],
        // 49 1
        [0x00, 0x08, 0x18, 0x28, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00, 0x00, 0x00],
        // 50 2
        [0x00, 0x3C, 0x42, 0x02, 0x0C, 0x30, 0x40, 0x40, 0x7E, 0x00, 0x00, 0x00],
        // 51 3
        [0x00, 0x3C, 0x42, 0x02, 0x1C, 0x02, 0x02, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 52 4
        [0x00, 0x04, 0x0C, 0x14, 0x24, 0x44, 0x7E, 0x04, 0x04, 0x00, 0x00, 0x00],
        // 53 5
        [0x00, 0x7E, 0x40, 0x40, 0x7C, 0x02, 0x02, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 54 6
        [0x00, 0x1C, 0x20, 0x40, 0x7C, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 55 7
        [0x00, 0x7E, 0x02, 0x04, 0x08, 0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00],
        // 56 8
        [0x00, 0x3C, 0x42, 0x42, 0x3C, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 57 9
        [0x00, 0x3C, 0x42, 0x42, 0x42, 0x3E, 0x02, 0x04, 0x38, 0x00, 0x00, 0x00],
        // 58 :
        [0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x00],
        // 59 ;
        [0x00, 0x00, 0x00, 0x18, 0x18, 0x00, 0x00, 0x18, 0x18, 0x08, 0x10, 0x00],
        // 60 <
        [0x00, 0x00, 0x04, 0x08, 0x10, 0x20, 0x10, 0x08, 0x04, 0x00, 0x00, 0x00],
        // 61 =
        [0x00, 0x00, 0x00, 0x00, 0x7E, 0x00, 0x7E, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 62 >
        [0x00, 0x00, 0x20, 0x10, 0x08, 0x04, 0x08, 0x10, 0x20, 0x00, 0x00, 0x00],
        // 63 ?
        [0x00, 0x3C, 0x42, 0x02, 0x04, 0x08, 0x08, 0x00, 0x08, 0x08, 0x00, 0x00],
        // 64 @
        [0x00, 0x3C, 0x42, 0x4E, 0x52, 0x4E, 0x40, 0x40, 0x3C, 0x00, 0x00, 0x00],
        // 65 A
        [0x00, 0x18, 0x24, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 66 B
        [0x00, 0x7C, 0x42, 0x42, 0x7C, 0x42, 0x42, 0x42, 0x7C, 0x00, 0x00, 0x00],
        // 67 C
        [0x00, 0x3C, 0x42, 0x40, 0x40, 0x40, 0x40, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 68 D
        [0x00, 0x78, 0x44, 0x42, 0x42, 0x42, 0x42, 0x44, 0x78, 0x00, 0x00, 0x00],
        // 69 E
        [0x00, 0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x40, 0x7E, 0x00, 0x00, 0x00],
        // 70 F
        [0x00, 0x7E, 0x40, 0x40, 0x7C, 0x40, 0x40, 0x40, 0x40, 0x00, 0x00, 0x00],
        // 71 G
        [0x00, 0x3C, 0x42, 0x40, 0x40, 0x4E, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 72 H
        [0x00, 0x42, 0x42, 0x42, 0x7E, 0x42, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 73 I
        [0x00, 0x3E, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x3E, 0x00, 0x00, 0x00],
        // 74 J
        [0x00, 0x1E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x44, 0x38, 0x00, 0x00, 0x00],
        // 75 K
        [0x00, 0x42, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 76 L
        [0x00, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x40, 0x7E, 0x00, 0x00, 0x00],
        // 77 M
        [0x00, 0x42, 0x66, 0x5A, 0x5A, 0x42, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 78 N
        [0x00, 0x42, 0x62, 0x52, 0x4A, 0x46, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 79 O
        [0x00, 0x3C, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 80 P
        [0x00, 0x7C, 0x42, 0x42, 0x7C, 0x40, 0x40, 0x40, 0x40, 0x00, 0x00, 0x00],
        // 81 Q
        [0x00, 0x3C, 0x42, 0x42, 0x42, 0x42, 0x4A, 0x44, 0x3A, 0x00, 0x00, 0x00],
        // 82 R
        [0x00, 0x7C, 0x42, 0x42, 0x7C, 0x48, 0x44, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 83 S
        [0x00, 0x3C, 0x42, 0x40, 0x3C, 0x02, 0x02, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 84 T
        [0x00, 0x7F, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00],
        // 85 U
        [0x00, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 86 V
        [0x00, 0x42, 0x42, 0x42, 0x42, 0x42, 0x24, 0x24, 0x18, 0x00, 0x00, 0x00],
        // 87 W
        [0x00, 0x42, 0x42, 0x42, 0x42, 0x5A, 0x5A, 0x66, 0x42, 0x00, 0x00, 0x00],
        // 88 X
        [0x00, 0x42, 0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 89 Y
        [0x00, 0x41, 0x22, 0x14, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00, 0x00],
        // 90 Z
        [0x00, 0x7E, 0x02, 0x04, 0x08, 0x10, 0x20, 0x40, 0x7E, 0x00, 0x00, 0x00],
        // 91 [
        [0x00, 0x1C, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1C, 0x00, 0x00, 0x00],
        // 92 backslash
        [0x00, 0x80, 0x40, 0x20, 0x10, 0x08, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00],
        // 93 ]
        [0x00, 0x38, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x38, 0x00, 0x00, 0x00],
        // 94 ^
        [0x00, 0x10, 0x28, 0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 95 _
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x7E, 0x00, 0x00],
        // 96 `
        [0x00, 0x10, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 97 a
        [0x00, 0x00, 0x00, 0x3C, 0x02, 0x3E, 0x42, 0x42, 0x3E, 0x00, 0x00, 0x00],
        // 98 b
        [0x00, 0x40, 0x40, 0x5C, 0x62, 0x42, 0x42, 0x62, 0x5C, 0x00, 0x00, 0x00],
        // 99 c
        [0x00, 0x00, 0x00, 0x3C, 0x42, 0x40, 0x40, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 100 d
        [0x00, 0x02, 0x02, 0x3A, 0x46, 0x42, 0x42, 0x46, 0x3A, 0x00, 0x00, 0x00],
        // 101 e
        [0x00, 0x00, 0x00, 0x3C, 0x42, 0x7E, 0x40, 0x40, 0x3C, 0x00, 0x00, 0x00],
        // 102 f
        [0x00, 0x0C, 0x10, 0x10, 0x3C, 0x10, 0x10, 0x10, 0x10, 0x00, 0x00, 0x00],
        // 103 g
        [0x00, 0x00, 0x00, 0x3A, 0x46, 0x42, 0x46, 0x3A, 0x02, 0x42, 0x3C, 0x00],
        // 104 h
        [0x00, 0x40, 0x40, 0x5C, 0x62, 0x42, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 105 i
        [0x00, 0x08, 0x00, 0x18, 0x08, 0x08, 0x08, 0x08, 0x1C, 0x00, 0x00, 0x00],
        // 106 j
        [0x00, 0x04, 0x00, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x04, 0x44, 0x38, 0x00],
        // 107 k
        [0x00, 0x40, 0x40, 0x44, 0x48, 0x70, 0x48, 0x44, 0x42, 0x00, 0x00, 0x00],
        // 108 l
        [0x00, 0x18, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x1C, 0x00, 0x00, 0x00],
        // 109 m
        [0x00, 0x00, 0x00, 0x76, 0x49, 0x49, 0x49, 0x49, 0x49, 0x00, 0x00, 0x00],
        // 110 n
        [0x00, 0x00, 0x00, 0x5C, 0x62, 0x42, 0x42, 0x42, 0x42, 0x00, 0x00, 0x00],
        // 111 o
        [0x00, 0x00, 0x00, 0x3C, 0x42, 0x42, 0x42, 0x42, 0x3C, 0x00, 0x00, 0x00],
        // 112 p
        [0x00, 0x00, 0x00, 0x5C, 0x62, 0x42, 0x62, 0x5C, 0x40, 0x40, 0x40, 0x00],
        // 113 q
        [0x00, 0x00, 0x00, 0x3A, 0x46, 0x42, 0x46, 0x3A, 0x02, 0x02, 0x02, 0x00],
        // 114 r
        [0x00, 0x00, 0x00, 0x5C, 0x62, 0x40, 0x40, 0x40, 0x40, 0x00, 0x00, 0x00],
        // 115 s
        [0x00, 0x00, 0x00, 0x3E, 0x40, 0x3C, 0x02, 0x02, 0x7C, 0x00, 0x00, 0x00],
        // 116 t
        [0x00, 0x10, 0x10, 0x3C, 0x10, 0x10, 0x10, 0x10, 0x0C, 0x00, 0x00, 0x00],
        // 117 u
        [0x00, 0x00, 0x00, 0x42, 0x42, 0x42, 0x42, 0x46, 0x3A, 0x00, 0x00, 0x00],
        // 118 v
        [0x00, 0x00, 0x00, 0x42, 0x42, 0x42, 0x24, 0x24, 0x18, 0x00, 0x00, 0x00],
        // 119 w
        [0x00, 0x00, 0x00, 0x41, 0x49, 0x49, 0x49, 0x49, 0x36, 0x00, 0x00, 0x00],
        // 120 x
        [0x00, 0x00, 0x00, 0x42, 0x24, 0x18, 0x18, 0x24, 0x42, 0x00, 0x00, 0x00],
        // 121 y
        [0x00, 0x00, 0x00, 0x42, 0x42, 0x42, 0x46, 0x3A, 0x02, 0x42, 0x3C, 0x00],
        // 122 z
        [0x00, 0x00, 0x00, 0x7E, 0x04, 0x08, 0x10, 0x20, 0x7E, 0x00, 0x00, 0x00],
        // 123 {
        [0x00, 0x0C, 0x10, 0x10, 0x10, 0x20, 0x10, 0x10, 0x10, 0x0C, 0x00, 0x00],
        // 124 |
        [0x00, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x08, 0x00, 0x00],
        // 125 }
        [0x00, 0x30, 0x08, 0x08, 0x08, 0x04, 0x08, 0x08, 0x08, 0x30, 0x00, 0x00],
        // 126 ~
        [0x00, 0x00, 0x00, 0x00, 0x32, 0x4C, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        // 127 DEL (empty)
        [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
    ];

    // Create texture buffer
    let mut pixels = vec![[0u8; 4]; TEX_W * TEX_H];

    // Render each glyph
    for (i, glyph) in GLYPHS.iter().enumerate() {
        let col = i % CHARS_PER_ROW;
        let row = i / CHARS_PER_ROW;
        let base_x = col * GLYPH_W;
        let base_y = row * GLYPH_H;

        for (gy, &bits) in glyph.iter().enumerate() {
            for gx in 0..8 {
                let px = base_x + gx;
                let py = base_y + gy;
                let idx = py * TEX_W + px;

                if (bits >> (7 - gx)) & 1 == 1 {
                    pixels[idx] = fg;
                } else {
                    // Check if adjacent to a lit pixel for glow effect
                    let mut has_neighbor = false;
                    for dy in -1i32..=1 {
                        for dx in -1i32..=1 {
                            if dx == 0 && dy == 0 { continue; }
                            let nx = gx as i32 + dx;
                            let ny = gy as i32 + dy;
                            if nx >= 0 && nx < 8 && ny >= 0 && ny < 12 {
                                let neighbor_bits = glyph[ny as usize];
                                if (neighbor_bits >> (7 - nx)) & 1 == 1 {
                                    has_neighbor = true;
                                    break;
                                }
                            }
                        }
                        if has_neighbor { break; }
                    }
                    pixels[idx] = if has_neighbor { glow } else { bg };
                }
            }
        }
    }

    // Write to file
    let path = output_dir.join("prism_font.png");
    let flat: Vec<u8> = pixels.iter().flat_map(|p| p.iter().copied()).collect();
    let tex = TextureBuffer {
        width: TEX_W as u32,
        height: TEX_H as u32,
        pixels: flat,
    };
    write_png(&tex, &path).expect("Failed to write font PNG");
    println!("    -> {} ({}x{}, {} glyphs)", path.display(), TEX_W, TEX_H, NUM_CHARS);
}

