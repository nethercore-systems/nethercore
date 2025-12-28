//! Prism Survivors Procedural Asset Generator
//!
//! Generates OBJ meshes and PNG textures for all game assets.
//! Run with: cargo run --release

use image::{Rgba, RgbaImage};
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use std::f32::consts::PI;
use std::fs::{self, File};
use std::io::Write;
use std::path::Path;

fn main() {
    let output_dir = Path::new("../../assets/models");
    fs::create_dir_all(output_dir.join("meshes")).unwrap();
    fs::create_dir_all(output_dir.join("textures")).unwrap();

    println!("=== Prism Survivors Asset Generator ===\n");

    // Generate heroes - IMPROVED: More saturated, distinctive colors
    println!("Generating hero assets...");
    // Knight: Steel blue armor with gold trim - noble warrior
    generate_hero("knight", [0.35, 0.45, 0.6], [0.85, 0.7, 0.2], true, output_dir);
    // Mage: Deep purple robes with cyan arcane energy
    generate_hero("mage", [0.35, 0.15, 0.55], [0.3, 0.8, 0.95], false, output_dir);
    // Ranger: Forest green with brown leather accents
    generate_hero("ranger", [0.2, 0.45, 0.25], [0.55, 0.4, 0.25], false, output_dir);
    // Cleric: White/cream robes with warm gold holy light
    generate_hero("cleric", [0.95, 0.92, 0.85], [1.0, 0.8, 0.3], false, output_dir);
    // Necromancer: Dark purple/black with sickly green soul energy
    generate_hero("necromancer", [0.15, 0.1, 0.2], [0.4, 0.9, 0.3], false, output_dir);
    // Paladin: Polished gold armor with white holy radiance
    generate_hero("paladin", [0.9, 0.75, 0.25], [1.0, 0.95, 0.8], true, output_dir);

    // Generate basic enemies
    println!("\nGenerating basic enemy assets...");
    generate_crawler(output_dir);
    generate_skeleton(output_dir);
    generate_wisp(output_dir);
    generate_golem(output_dir);
    generate_shade(output_dir);
    generate_berserker(output_dir);
    generate_arcane_sentinel(output_dir);

    // Generate elite enemies
    println!("\nGenerating elite enemy assets...");
    generate_crystal_knight(output_dir);
    generate_void_mage(output_dir);
    generate_golem_titan(output_dir);
    generate_specter_lord(output_dir);

    // Generate bosses
    println!("\nGenerating boss assets...");
    generate_prism_colossus(output_dir);
    generate_void_dragon(output_dir);

    // Generate environment
    println!("\nGenerating environment assets...");
    generate_xp_gem(output_dir);
    generate_arena_floor(output_dir);

    println!("\n=== Generation Complete ===");
}

// =============================================================================
// Hero Generation
// =============================================================================

fn generate_hero(name: &str, primary: [f32; 3], secondary: [f32; 3], armored: bool, output_dir: &Path) {
    println!("  {} hero...", name);

    // Generate mesh
    let mesh = generate_humanoid_mesh(0.8, armored);
    write_obj(&mesh, &output_dir.join(format!("meshes/{}.obj", name)));

    // Generate texture
    let tex = generate_hero_texture(name, primary, secondary, armored);
    tex.save(output_dir.join(format!("textures/{}.png", name))).unwrap();
}

fn generate_humanoid_mesh(height: f32, armored: bool) -> Mesh {
    let mut mesh = Mesh::new();
    let torso_width = if armored { 0.35 } else { 0.28 };
    let torso_depth = if armored { 0.22 } else { 0.18 };

    // Torso
    mesh.add_box(0.0, height * 0.45, 0.0, torso_width, height * 0.35, torso_depth);

    // Head
    let head_size = 0.18;
    mesh.add_sphere(0.0, height * 0.78, 0.0, head_size, 8, 6);

    // Arms
    let arm_width = 0.08;
    let arm_length = height * 0.35;
    mesh.add_box(-torso_width - arm_width, height * 0.5, 0.0, arm_width, arm_length, arm_width);
    mesh.add_box(torso_width + arm_width, height * 0.5, 0.0, arm_width, arm_length, arm_width);

    // Legs
    let leg_width = 0.1;
    let leg_length = height * 0.4;
    mesh.add_box(-0.1, height * 0.2, 0.0, leg_width, leg_length, leg_width);
    mesh.add_box(0.1, height * 0.2, 0.0, leg_width, leg_length, leg_width);

    // Shoulders (if armored)
    if armored {
        mesh.add_sphere(-torso_width - 0.05, height * 0.68, 0.0, 0.12, 6, 4);
        mesh.add_sphere(torso_width + 0.05, height * 0.68, 0.0, 0.12, 6, 4);
    }

    mesh
}

fn generate_hero_texture(name: &str, primary: [f32; 3], secondary: [f32; 3], armored: bool) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(name));

    // IMPROVED: Better base with gradient lighting simulation
    for y in 0..256 {
        for x in 0..256 {
            let noise = rng.gen::<f32>() * 0.08 - 0.04;
            let region = get_texture_region(x, y);

            // Add subtle lighting gradient (top-lit)
            let light_y = 1.0 - (y as f32 / 256.0) * 0.25;
            // Add slight rim lighting on edges
            let edge_x = ((x as f32 - 128.0).abs() / 128.0).powi(2) * 0.15;

            let base_color = match region {
                TextureRegion::Body => primary,
                TextureRegion::Accent => secondary,
                TextureRegion::Dark => [primary[0] * 0.4, primary[1] * 0.4, primary[2] * 0.4],
            };

            // Apply lighting
            let r = ((base_color[0] * light_y + edge_x + noise).clamp(0.0, 1.0) * 255.0) as u8;
            let g = ((base_color[1] * light_y + edge_x * 0.8 + noise).clamp(0.0, 1.0) * 255.0) as u8;
            let b = ((base_color[2] * light_y + edge_x * 0.6 + noise).clamp(0.0, 1.0) * 255.0) as u8;

            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }

    // Add detail patterns
    if armored {
        add_armor_detail(&mut img, secondary, &mut rng);
    } else {
        add_cloth_detail(&mut img, secondary, &mut rng);
    }

    // IMPROVED: Add highlight band at accent region boundary
    for x in 0..256 {
        let highlight = to_rgba(secondary, 0.5);
        blend_pixel(&mut img, x, 82, highlight);
        blend_pixel(&mut img, x, 83, to_rgba([1.0, 1.0, 1.0], 0.3));
        blend_pixel(&mut img, x, 167, highlight);
        blend_pixel(&mut img, x, 168, to_rgba([1.0, 1.0, 1.0], 0.3));
    }

    // Add class-specific emblem (larger and more visible)
    add_class_emblem(&mut img, name, secondary);

    img
}

fn get_texture_region(x: u32, y: u32) -> TextureRegion {
    // UV layout: top = body, middle = accent, bottom = dark areas
    if y < 85 {
        TextureRegion::Body
    } else if y < 170 {
        TextureRegion::Accent
    } else {
        TextureRegion::Dark
    }
}

#[derive(Clone, Copy)]
enum TextureRegion {
    Body,
    Accent,
    Dark,
}

fn add_armor_detail(img: &mut RgbaImage, accent: [f32; 3], rng: &mut StdRng) {
    // IMPROVED: More prominent plate segments with beveled edges
    let dark_accent = [accent[0] * 0.4, accent[1] * 0.4, accent[2] * 0.4];
    let light_accent = [
        (accent[0] * 1.3).min(1.0),
        (accent[1] * 1.3).min(1.0),
        (accent[2] * 1.3).min(1.0),
    ];

    // Horizontal plate divisions with bevel effect
    for row in (25..240).step_by(35) {
        // Dark line (shadow)
        for x in 5..251 {
            blend_pixel(img, x, row, to_rgba(dark_accent, 0.7));
            blend_pixel(img, x, row + 1, to_rgba(dark_accent, 0.5));
        }
        // Light line below (highlight from next plate)
        for x in 5..251 {
            blend_pixel(img, x, row + 3, to_rgba(light_accent, 0.6));
            blend_pixel(img, x, row + 4, to_rgba(light_accent, 0.3));
        }
    }

    // Vertical plate divisions (fewer, for chest/shoulder plates)
    for col in [80, 176] {
        for y in 10..80 {
            blend_pixel(img, col, y, to_rgba(dark_accent, 0.5));
            blend_pixel(img, col + 1, y, to_rgba(light_accent, 0.4));
        }
    }

    // Rivets - larger and more prominent
    for _ in 0..40 {
        let x = rng.gen_range(15..241);
        let y = rng.gen_range(15..241);
        // Rivet with highlight
        draw_circle(img, x, y, 3, to_rgba(dark_accent, 0.8));
        draw_circle(img, x - 1, y - 1, 1, to_rgba(light_accent, 0.6));
    }

    // Add specular highlights in center of plates
    for _ in 0..12 {
        let x = rng.gen_range(40..216);
        let y = rng.gen_range(40..216);
        draw_glow_spot(img, x, y, 8, light_accent);
    }
}

fn add_cloth_detail(img: &mut RgbaImage, accent: [f32; 3], rng: &mut StdRng) {
    let dark_accent = [accent[0] * 0.5, accent[1] * 0.5, accent[2] * 0.5];
    let light_accent = [
        (accent[0] * 1.4).min(1.0),
        (accent[1] * 1.4).min(1.0),
        (accent[2] * 1.4).min(1.0),
    ];

    // IMPROVED: More visible fabric folds (diagonal lines)
    for i in 0..8 {
        let start_x = i * 32;
        for offset in 0..256 {
            let x = (start_x + offset) % 256;
            let y = offset;
            if y < 256 {
                let intensity = 0.2 + (((offset as f32) / 40.0).sin().abs() * 0.3);
                blend_pixel(img, x as u32, y as u32, to_rgba(dark_accent, intensity));
            }
        }
    }

    // Decorative trim at region boundaries - more ornate
    for x in 0..256 {
        // Top trim (gold/accent braid pattern)
        let braid = ((x as f32 * 0.3).sin().abs() * 0.3) + 0.5;
        blend_pixel(img, x, 82, to_rgba(dark_accent, 0.8));
        blend_pixel(img, x, 83, to_rgba(accent, braid));
        blend_pixel(img, x, 84, to_rgba(light_accent, braid * 0.8));
        blend_pixel(img, x, 85, to_rgba(accent, braid));
        blend_pixel(img, x, 86, to_rgba(dark_accent, 0.8));

        // Bottom trim
        blend_pixel(img, x, 167, to_rgba(dark_accent, 0.8));
        blend_pixel(img, x, 168, to_rgba(accent, braid));
        blend_pixel(img, x, 169, to_rgba(light_accent, braid * 0.8));
        blend_pixel(img, x, 170, to_rgba(accent, braid));
        blend_pixel(img, x, 171, to_rgba(dark_accent, 0.8));
    }

    // Add subtle magical sparkles for caster classes
    for _ in 0..20 {
        let x = rng.gen_range(20..236);
        let y = rng.gen_range(20..236);
        draw_glow_spot(img, x, y, 4, light_accent);
    }
}

fn add_class_emblem(img: &mut RgbaImage, class: &str, accent: [f32; 3]) {
    let cx = 128;
    let cy = 42; // Upper body region

    match class {
        "knight" => draw_shield(img, cx, cy, accent),
        "mage" => draw_star(img, cx, cy, 5, accent),
        "ranger" => draw_arrow(img, cx, cy, accent),
        "cleric" => draw_cross(img, cx, cy, accent),
        "necromancer" => draw_skull(img, cx, cy, accent),
        "paladin" => draw_sun(img, cx, cy, accent),
        _ => {}
    }
}

// =============================================================================
// Enemy Generation - Basic
// =============================================================================

fn generate_crawler(output_dir: &Path) {
    println!("  crawler...");
    let mut mesh = Mesh::new();

    // Spider-like body
    mesh.add_sphere(0.0, 0.15, 0.0, 0.2, 8, 6);  // Main body
    mesh.add_sphere(0.0, 0.18, 0.15, 0.12, 6, 4); // Head

    // 6 legs (simplified)
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * PI * 2.0;
        let x = angle.cos() * 0.2;
        let z = angle.sin() * 0.2;
        mesh.add_box(x, 0.1, z, 0.03, 0.15, 0.03);
    }

    write_obj(&mesh, &output_dir.join("meshes/crawler.obj"));

    // Texture: dark purple/black chitinous
    let tex = generate_creature_texture([0.15, 0.1, 0.2], [0.3, 0.1, 0.35], 0.4, "crawler");
    tex.save(output_dir.join("textures/crawler.png")).unwrap();
}

fn generate_skeleton(output_dir: &Path) {
    println!("  skeleton...");
    let mut mesh = Mesh::new();

    // Skull
    mesh.add_sphere(0.0, 0.7, 0.0, 0.12, 8, 6);
    // Ribcage (simplified)
    mesh.add_box(0.0, 0.45, 0.0, 0.15, 0.2, 0.1);
    // Spine
    mesh.add_box(0.0, 0.3, 0.0, 0.04, 0.3, 0.04);
    // Pelvis
    mesh.add_box(0.0, 0.12, 0.0, 0.12, 0.08, 0.06);
    // Arms (bones)
    mesh.add_box(-0.2, 0.4, 0.0, 0.03, 0.25, 0.03);
    mesh.add_box(0.2, 0.4, 0.0, 0.03, 0.25, 0.03);
    // Legs (bones)
    mesh.add_box(-0.08, 0.0, 0.0, 0.03, 0.25, 0.03);
    mesh.add_box(0.08, 0.0, 0.0, 0.03, 0.25, 0.03);

    write_obj(&mesh, &output_dir.join("meshes/skeleton.obj"));

    let tex = generate_creature_texture([0.9, 0.85, 0.75], [0.6, 0.55, 0.45], 0.2, "skeleton");
    tex.save(output_dir.join("textures/skeleton.png")).unwrap();
}

fn generate_wisp(output_dir: &Path) {
    println!("  wisp...");
    let mut mesh = Mesh::new();

    // Glowing orb core
    mesh.add_sphere(0.0, 0.25, 0.0, 0.15, 10, 8);
    // Wispy tail
    for i in 0..4 {
        let y = 0.25 - (i as f32 * 0.08);
        let r = 0.12 - (i as f32 * 0.025);
        mesh.add_sphere(0.0, y, 0.0, r, 6, 4);
    }

    write_obj(&mesh, &output_dir.join("meshes/wisp.obj"));

    // Ethereal blue glow
    let tex = generate_glow_texture([0.3, 0.6, 1.0], [0.8, 0.9, 1.0], "wisp");
    tex.save(output_dir.join("textures/wisp.png")).unwrap();
}

fn generate_golem(output_dir: &Path) {
    println!("  golem...");
    let mut mesh = Mesh::new();

    // Chunky rock body
    mesh.add_box(0.0, 0.5, 0.0, 0.4, 0.5, 0.35);   // Torso
    mesh.add_box(0.0, 0.95, 0.0, 0.25, 0.2, 0.22); // Head
    mesh.add_box(-0.55, 0.55, 0.0, 0.18, 0.4, 0.18); // Left arm
    mesh.add_box(0.55, 0.55, 0.0, 0.18, 0.4, 0.18);  // Right arm
    mesh.add_box(-0.2, 0.0, 0.0, 0.15, 0.35, 0.15);  // Left leg
    mesh.add_box(0.2, 0.0, 0.0, 0.15, 0.35, 0.15);   // Right leg

    write_obj(&mesh, &output_dir.join("meshes/golem.obj"));

    let tex = generate_stone_texture([0.4, 0.38, 0.35], [0.55, 0.5, 0.45], "golem");
    tex.save(output_dir.join("textures/golem.png")).unwrap();
}

fn generate_shade(output_dir: &Path) {
    println!("  shade...");
    let mut mesh = Mesh::new();

    // Ghostly hooded figure
    mesh.add_sphere(0.0, 0.55, 0.0, 0.25, 8, 6);  // Body mass
    mesh.add_sphere(0.0, 0.75, 0.0, 0.15, 8, 6);  // Hood
    // Wispy bottom
    for i in 0..3 {
        let y = 0.35 - (i as f32 * 0.12);
        let r = 0.22 - (i as f32 * 0.05);
        mesh.add_sphere(0.0, y, 0.0, r, 6, 4);
    }

    write_obj(&mesh, &output_dir.join("meshes/shade.obj"));

    // Dark shadowy texture
    let tex = generate_shadow_texture([0.1, 0.08, 0.15], [0.25, 0.2, 0.35], "shade");
    tex.save(output_dir.join("textures/shade.png")).unwrap();
}

fn generate_berserker(output_dir: &Path) {
    println!("  berserker...");
    let mesh = generate_humanoid_mesh(0.9, false);
    write_obj(&mesh, &output_dir.join("meshes/berserker.obj"));

    // Red-tinted savage warrior
    let tex = generate_creature_texture([0.6, 0.2, 0.15], [0.4, 0.35, 0.3], 0.5, "berserker");
    tex.save(output_dir.join("textures/berserker.png")).unwrap();
}

fn generate_arcane_sentinel(output_dir: &Path) {
    println!("  arcane_sentinel...");
    let mut mesh = Mesh::new();

    // Floating magical construct
    mesh.add_box(0.0, 0.5, 0.0, 0.3, 0.4, 0.2);      // Core
    mesh.add_sphere(0.0, 0.85, 0.0, 0.18, 10, 8);    // Eye/gem
    mesh.add_box(-0.4, 0.5, 0.0, 0.08, 0.3, 0.08);   // Left arm
    mesh.add_box(0.4, 0.5, 0.0, 0.08, 0.3, 0.08);    // Right arm

    write_obj(&mesh, &output_dir.join("meshes/arcane_sentinel.obj"));

    // Magical blue/purple construct
    let tex = generate_magic_texture([0.2, 0.3, 0.5], [0.5, 0.4, 0.8], "arcane_sentinel");
    tex.save(output_dir.join("textures/arcane_sentinel.png")).unwrap();
}

// =============================================================================
// Enemy Generation - Elites
// =============================================================================

fn generate_crystal_knight(output_dir: &Path) {
    println!("  crystal_knight (elite)...");
    let mesh = generate_humanoid_mesh(1.1, true);
    write_obj(&mesh, &output_dir.join("meshes/crystal_knight.obj"));

    // Purple crystalline armor with glow
    let tex = generate_crystal_texture([0.4, 0.2, 0.5], [0.8, 0.5, 1.0], "crystal_knight");
    tex.save(output_dir.join("textures/crystal_knight.png")).unwrap();
}

fn generate_void_mage(output_dir: &Path) {
    println!("  void_mage (elite)...");
    let mesh = generate_humanoid_mesh(0.95, false);
    write_obj(&mesh, &output_dir.join("meshes/void_mage.obj"));

    // Dark void robes with magical runes
    let tex = generate_void_texture([0.1, 0.05, 0.15], [0.4, 0.2, 0.6], "void_mage");
    tex.save(output_dir.join("textures/void_mage.png")).unwrap();
}

fn generate_golem_titan(output_dir: &Path) {
    println!("  golem_titan (elite)...");
    let mut mesh = Mesh::new();

    // Massive stone titan
    mesh.add_box(0.0, 0.7, 0.0, 0.55, 0.7, 0.45);    // Torso
    mesh.add_box(0.0, 1.3, 0.0, 0.35, 0.3, 0.3);     // Head
    mesh.add_box(-0.75, 0.75, 0.0, 0.25, 0.55, 0.25); // Left arm
    mesh.add_box(0.75, 0.75, 0.0, 0.25, 0.55, 0.25);  // Right arm
    mesh.add_box(-0.28, 0.0, 0.0, 0.2, 0.45, 0.2);    // Left leg
    mesh.add_box(0.28, 0.0, 0.0, 0.2, 0.45, 0.2);     // Right leg
    // Multiple glowing eyes
    for i in 0..3 {
        let x = -0.15 + (i as f32 * 0.15);
        mesh.add_sphere(x, 1.35, 0.28, 0.06, 6, 4);
    }

    write_obj(&mesh, &output_dir.join("meshes/golem_titan.obj"));

    let tex = generate_stone_texture([0.35, 0.32, 0.28], [0.6, 0.4, 0.8], "golem_titan");
    tex.save(output_dir.join("textures/golem_titan.png")).unwrap();
}

fn generate_specter_lord(output_dir: &Path) {
    println!("  specter_lord (elite)...");
    let mut mesh = Mesh::new();

    // Crowned ghost with chains
    mesh.add_sphere(0.0, 0.7, 0.0, 0.3, 10, 8);  // Main body
    mesh.add_sphere(0.0, 1.0, 0.0, 0.18, 8, 6);  // Head
    // Crown
    for i in 0..5 {
        let angle = (i as f32 / 5.0) * PI * 2.0;
        let x = angle.cos() * 0.12;
        let z = angle.sin() * 0.12;
        mesh.add_box(x, 1.15, z, 0.02, 0.08, 0.02);
    }
    // Wispy trail
    for i in 0..4 {
        let y = 0.5 - (i as f32 * 0.12);
        let r = 0.25 - (i as f32 * 0.05);
        mesh.add_sphere(0.0, y, 0.0, r, 6, 4);
    }

    write_obj(&mesh, &output_dir.join("meshes/specter_lord.obj"));

    let tex = generate_ghost_texture([0.15, 0.2, 0.25], [0.6, 0.8, 1.0], "specter_lord");
    tex.save(output_dir.join("textures/specter_lord.png")).unwrap();
}

// =============================================================================
// Enemy Generation - Bosses
// =============================================================================

fn generate_prism_colossus(output_dir: &Path) {
    println!("  prism_colossus (boss)...");
    let mut mesh = Mesh::new();

    // Giant crystalline humanoid
    mesh.add_box(0.0, 1.2, 0.0, 0.7, 0.9, 0.5);     // Torso
    mesh.add_box(0.0, 2.0, 0.0, 0.4, 0.35, 0.35);   // Head
    mesh.add_box(-1.0, 1.3, 0.0, 0.3, 0.7, 0.3);    // Left arm
    mesh.add_box(1.0, 1.3, 0.0, 0.3, 0.7, 0.3);     // Right arm
    mesh.add_box(-0.35, 0.0, 0.0, 0.25, 0.6, 0.25); // Left leg
    mesh.add_box(0.35, 0.0, 0.0, 0.25, 0.6, 0.25);  // Right leg
    // Crystal spikes
    for i in 0..6 {
        let angle = (i as f32 / 6.0) * PI * 2.0;
        let x = angle.cos() * 0.5;
        let z = angle.sin() * 0.5;
        mesh.add_box(x, 1.5 + (i as f32 * 0.1), z, 0.08, 0.25, 0.08);
    }

    write_obj(&mesh, &output_dir.join("meshes/prism_colossus.obj"));

    // Multi-colored crystalline with red glow
    let tex = generate_boss_crystal_texture([0.3, 0.4, 0.5], [1.0, 0.3, 0.3], "prism_colossus");
    tex.save(output_dir.join("textures/prism_colossus.png")).unwrap();
}

fn generate_void_dragon(output_dir: &Path) {
    println!("  void_dragon (boss)...");
    let mut mesh = Mesh::new();

    // Ethereal dragon form
    // Body segments
    mesh.add_sphere(0.0, 0.6, 0.0, 0.5, 12, 8);     // Main body
    mesh.add_sphere(0.0, 0.7, 0.6, 0.3, 10, 8);     // Neck
    mesh.add_sphere(0.0, 0.8, 1.0, 0.25, 10, 8);    // Head
    // Tail
    for i in 0..5 {
        let z = -0.3 - (i as f32 * 0.25);
        let r = 0.35 - (i as f32 * 0.06);
        mesh.add_sphere(0.0, 0.5, z, r, 8, 6);
    }
    // Wings (simplified)
    mesh.add_box(-0.8, 0.7, 0.0, 0.5, 0.05, 0.4);
    mesh.add_box(0.8, 0.7, 0.0, 0.5, 0.05, 0.4);
    // Horns
    mesh.add_box(-0.15, 1.0, 1.1, 0.04, 0.15, 0.04);
    mesh.add_box(0.15, 1.0, 1.1, 0.04, 0.15, 0.04);

    write_obj(&mesh, &output_dir.join("meshes/void_dragon.obj"));

    // Void flames: deep purple/black with red accents
    let tex = generate_void_dragon_texture([0.15, 0.05, 0.2], [0.8, 0.2, 0.3], "void_dragon");
    tex.save(output_dir.join("textures/void_dragon.png")).unwrap();
}

// =============================================================================
// Environment Generation
// =============================================================================

fn generate_xp_gem(output_dir: &Path) {
    println!("  xp_gem...");
    let mut mesh = Mesh::new();

    // Diamond-shaped gem
    mesh.add_diamond(0.0, 0.15, 0.0, 0.1, 0.2);

    write_obj(&mesh, &output_dir.join("meshes/xp_gem.obj"));

    // Bright green glowing gem
    let tex = generate_gem_texture([0.2, 0.8, 0.3], [0.5, 1.0, 0.6], "xp_gem");
    tex.save(output_dir.join("textures/xp_gem.png")).unwrap();
}

fn generate_arena_floor(output_dir: &Path) {
    println!("  arena_floor...");

    // Simple flat quad for floor
    let mut mesh = Mesh::new();
    mesh.add_quad_floor(0.0, 0.0, 0.0, 50.0, 50.0);
    write_obj(&mesh, &output_dir.join("meshes/arena_floor.obj"));

    // Generate floor texture (Crystal Cavern as default)
    let tex = generate_floor_texture([0.1, 0.15, 0.25], [0.3, 0.4, 0.6], "arena_floor");
    tex.save(output_dir.join("textures/arena_floor.png")).unwrap();
}

// =============================================================================
// Texture Generation Helpers
// =============================================================================

fn generate_creature_texture(base: [f32; 3], accent: [f32; 3], roughness: f32, seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    // IMPROVED: More interesting organic texture with visible detail
    let dark = [base[0] * 0.5, base[1] * 0.5, base[2] * 0.5];

    for y in 0..256 {
        for x in 0..256 {
            let noise = (rng.gen::<f32>() - 0.5) * roughness * 0.4;

            // Organic cell pattern
            let cell_x = (x as f32 / 32.0).floor();
            let cell_y = (y as f32 / 32.0).floor();
            let cell_center_x = cell_x * 32.0 + 16.0;
            let cell_center_y = cell_y * 32.0 + 16.0;
            let dist_to_center = ((x as f32 - cell_center_x).powi(2) + (y as f32 - cell_center_y).powi(2)).sqrt() / 22.0;
            let cell_shade = (1.0 - dist_to_center.min(1.0)) * 0.2;

            // Vertical gradient (darker at bottom for grounding)
            let t = (y as f32 / 256.0) * 0.4;

            let r = (base[0] * (1.0 - t) + accent[0] * t + noise + cell_shade).clamp(0.0, 1.0);
            let g = (base[1] * (1.0 - t) + accent[1] * t + noise + cell_shade).clamp(0.0, 1.0);
            let b = (base[2] * (1.0 - t) + accent[2] * t + noise + cell_shade).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add sinister glowing eyes/spots for enemies
    let glow_color = [
        (accent[0] * 1.5).min(1.0),
        (accent[1] * 1.5).min(1.0),
        (accent[2] * 1.5).min(1.0),
    ];
    for _ in 0..6 {
        let x = rng.gen_range(40..216);
        let y = rng.gen_range(20..100); // Upper region (head/eyes)
        draw_glow_spot(&mut img, x, y, 6, glow_color);
    }

    // Add dark veins/cracks
    for _ in 0..8 {
        let x1 = rng.gen_range(20..236);
        let y1 = rng.gen_range(20..236);
        let x2 = (x1 as i32 + rng.gen_range(-40..40)).clamp(0, 255) as u32;
        let y2 = (y1 as i32 + rng.gen_range(-40..40)).clamp(0, 255) as u32;
        draw_line(&mut img, x1, y1, x2, y2, to_rgba(dark, 0.6));
    }

    img
}

fn generate_glow_texture(core: [f32; 3], glow: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    let cx = 128.0;
    let cy = 128.0;

    // IMPROVED: More ethereal with pulsing rings
    let bright_core = [
        (core[0] * 1.5).min(1.0),
        (core[1] * 1.5).min(1.0),
        (core[2] * 1.5).min(1.0),
    ];

    for y in 0..256 {
        for x in 0..256 {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt() / 128.0;
            let t = (1.0 - dist).clamp(0.0, 1.0);
            let noise = rng.gen::<f32>() * 0.08;

            // Add concentric ring pattern for ethereal effect
            let ring = ((dist * 8.0).sin().abs() * 0.15) * t;

            // Swirling energy pattern
            let angle = dy.atan2(dx);
            let swirl = ((angle * 3.0 + dist * 5.0).sin().abs() * 0.1) * t;

            let r = (core[0] * t * t + glow[0] * (1.0 - t) + ring + swirl + noise).clamp(0.0, 1.0);
            let g = (core[1] * t * t + glow[1] * (1.0 - t) + ring + swirl + noise).clamp(0.0, 1.0);
            let b = (core[2] * t * t + glow[2] * (1.0 - t) + ring + swirl + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add bright core highlight
    draw_glow_spot(&mut img, 128, 128, 25, bright_core);

    // Add sparkle points around the wisp
    for _ in 0..15 {
        let angle = rng.gen::<f32>() * 6.28;
        let dist = rng.gen_range(40..100) as f32;
        let x = (128.0 + angle.cos() * dist) as u32;
        let y = (128.0 + angle.sin() * dist) as u32;
        if x < 256 && y < 256 {
            draw_glow_spot(&mut img, x, y, 4, bright_core);
        }
    }

    img
}

fn generate_stone_texture(base: [f32; 3], accent: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    let dark = [base[0] * 0.4, base[1] * 0.4, base[2] * 0.4];
    let light = [
        (base[0] * 1.3).min(1.0),
        (base[1] * 1.3).min(1.0),
        (base[2] * 1.3).min(1.0),
    ];

    // IMPROVED: Rocky texture with visible stone blocks and moss
    for y in 0..256 {
        for x in 0..256 {
            let noise = (rng.gen::<f32>() - 0.5) * 0.15;

            // Stone block pattern
            let block_y = (y / 30) % 2;
            let block_offset = if block_y == 0 { 20 } else { 0 };
            let in_mortar_h = (y % 30) < 3;
            let in_mortar_v = ((x + block_offset) % 40) < 3;

            let base_shade = if in_mortar_h || in_mortar_v {
                0.5  // Dark mortar lines
            } else {
                0.85 + rng.gen::<f32>() * 0.15 // Stone surface variation
            };

            // Add rough surface detail
            let roughness = ((x as f32 * 0.5).sin() * (y as f32 * 0.3).cos()).abs() * 0.1;

            let r = (base[0] * base_shade + roughness + noise).clamp(0.0, 1.0);
            let g = (base[1] * base_shade + roughness + noise).clamp(0.0, 1.0);
            let b = (base[2] * base_shade + roughness + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add deep cracks
    for _ in 0..15 {
        let x1 = rng.gen_range(20..236);
        let y1 = rng.gen_range(20..236);
        let x2 = (x1 as i32 + rng.gen_range(-60..60)).clamp(0, 255) as u32;
        let y2 = (y1 as i32 + rng.gen_range(-60..60)).clamp(0, 255) as u32;
        draw_line(&mut img, x1, y1, x2, y2, to_rgba(dark, 0.7));
    }

    // Add glowing accents (magical runes/energy for titans)
    if seed.contains("titan") {
        // Titans have glowing purple energy coursing through them
        for _ in 0..12 {
            let cx = rng.gen_range(30..226);
            let cy = rng.gen_range(30..226);
            draw_glow_spot(&mut img, cx, cy, 20, accent);
        }
        // Add energy vein lines
        for _ in 0..8 {
            let x1 = rng.gen_range(30..226);
            let y1 = rng.gen_range(30..226);
            let x2 = (x1 as i32 + rng.gen_range(-80..80)).clamp(0, 255) as u32;
            let y2 = (y1 as i32 + rng.gen_range(-80..80)).clamp(0, 255) as u32;
            draw_line(&mut img, x1, y1, x2, y2, to_rgba(accent, 0.6));
        }
    } else {
        // Regular golems get subtle moss/lichen patches
        for _ in 0..8 {
            let cx = rng.gen_range(40..216);
            let cy = rng.gen_range(40..216);
            draw_glow_spot(&mut img, cx, cy, 10, [0.2, 0.35, 0.15]); // Greenish moss
        }
    }

    // Add stone edge highlights
    for _ in 0..10 {
        let x = rng.gen_range(30..226);
        let y = rng.gen_range(30..226);
        draw_glow_spot(&mut img, x, y, 5, light);
    }

    img
}

fn generate_shadow_texture(dark: [f32; 3], edge: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    for y in 0..256 {
        for x in 0..256 {
            let noise = rng.gen::<f32>() * 0.15;
            let edge_dist = ((x as f32 - 128.0).abs() + (y as f32 - 128.0).abs()) / 256.0;

            let r = (dark[0] * (1.0 - edge_dist) + edge[0] * edge_dist + noise).clamp(0.0, 1.0);
            let g = (dark[1] * (1.0 - edge_dist) + edge[1] * edge_dist + noise).clamp(0.0, 1.0);
            let b = (dark[2] * (1.0 - edge_dist) + edge[2] * edge_dist + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                (200 + rng.gen_range(0..55)) // Slight alpha variation
            ]));
        }
    }

    img
}

fn generate_magic_texture(base: [f32; 3], magic: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    for y in 0..256 {
        for x in 0..256 {
            let noise = rng.gen::<f32>() * 0.1;
            let magic_pattern = ((x as f32 * 0.1).sin() * (y as f32 * 0.1).cos()).abs();

            let r = (base[0] * (1.0 - magic_pattern * 0.5) + magic[0] * magic_pattern * 0.5 + noise).clamp(0.0, 1.0);
            let g = (base[1] * (1.0 - magic_pattern * 0.5) + magic[1] * magic_pattern * 0.5 + noise).clamp(0.0, 1.0);
            let b = (base[2] * (1.0 - magic_pattern * 0.5) + magic[2] * magic_pattern * 0.5 + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add rune-like markings
    for _ in 0..8 {
        let cx = rng.gen_range(40..216);
        let cy = rng.gen_range(40..216);
        draw_rune(&mut img, cx, cy, magic, &mut rng);
    }

    img
}

fn generate_crystal_texture(base: [f32; 3], glow: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    for y in 0..256 {
        for x in 0..256 {
            let noise = rng.gen::<f32>() * 0.15;
            // Faceted crystal pattern
            let facet = ((x / 16 + y / 16) % 2) as f32 * 0.15;

            let r = (base[0] + facet + noise).clamp(0.0, 1.0);
            let g = (base[1] + facet + noise).clamp(0.0, 1.0);
            let b = (base[2] + facet + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add glow edges
    for _ in 0..15 {
        let x = rng.gen_range(0..256);
        let y = rng.gen_range(0..256);
        draw_glow_spot(&mut img, x, y, 8, glow);
    }

    img
}

fn generate_void_texture(dark: [f32; 3], rune: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    for y in 0..256 {
        for x in 0..256 {
            let noise = rng.gen::<f32>() * 0.08;
            let swirl = ((x as f32 * 0.02 + y as f32 * 0.02).sin() * 0.1).abs();

            let r = (dark[0] + swirl + noise).clamp(0.0, 1.0);
            let g = (dark[1] + swirl + noise).clamp(0.0, 1.0);
            let b = (dark[2] + swirl + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add floating runes
    for _ in 0..12 {
        let cx = rng.gen_range(30..226);
        let cy = rng.gen_range(30..226);
        draw_rune(&mut img, cx, cy, rune, &mut rng);
    }

    img
}

fn generate_ghost_texture(base: [f32; 3], glow: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    for y in 0..256 {
        for x in 0..256 {
            let cx = 128.0;
            let cy = 128.0;
            let dist = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() / 180.0;
            let t = dist.clamp(0.0, 1.0);
            let noise = rng.gen::<f32>() * 0.1;

            let r = (base[0] * (1.0 - t) + glow[0] * t + noise).clamp(0.0, 1.0);
            let g = (base[1] * (1.0 - t) + glow[1] * t + noise).clamp(0.0, 1.0);
            let b = (base[2] * (1.0 - t) + glow[2] * t + noise).clamp(0.0, 1.0);
            let a = (255.0 * (1.0 - t * 0.3)) as u8;

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                a
            ]));
        }
    }

    img
}

fn generate_boss_crystal_texture(base: [f32; 3], glow: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    // IMPROVED: More dramatic crystalline effect with visible facets
    let dark = [base[0] * 0.3, base[1] * 0.3, base[2] * 0.3];
    let bright = [
        (base[0] * 1.5).min(1.0),
        (base[1] * 1.5).min(1.0),
        (base[2] * 1.5).min(1.0),
    ];

    for y in 0..256 {
        for x in 0..256 {
            let noise = rng.gen::<f32>() * 0.1;

            // Multi-faceted crystal with sharp edges
            let facet1 = ((x / 16 + y / 24) % 4) as f32 * 0.15;
            let facet2 = ((x / 24 + y / 16) % 3) as f32 * 0.12;

            // Add diagonal crystal shards
            let shard = if (x + y * 2) % 48 < 4 || (x * 2 + y) % 64 < 4 {
                0.25  // Bright edge
            } else if (x + y * 2) % 48 < 8 || (x * 2 + y) % 64 < 8 {
                -0.15 // Dark edge
            } else {
                0.0
            };

            let r = (base[0] + facet1 + shard + noise).clamp(0.0, 1.0);
            let g = (base[1] + facet2 + shard * 0.8 + noise).clamp(0.0, 1.0);
            let b = (base[2] + facet1 + facet2 + shard + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add menacing red/orange glow spots (boss power cores)
    for _ in 0..35 {
        let x = rng.gen_range(10..246);
        let y = rng.gen_range(10..246);
        draw_glow_spot(&mut img, x, y, 15, glow);
    }

    // Add bright crystal highlights
    for _ in 0..20 {
        let x = rng.gen_range(20..236);
        let y = rng.gen_range(20..236);
        draw_glow_spot(&mut img, x, y, 6, bright);
    }

    // Add dark cracks for visual interest
    for _ in 0..12 {
        let x1 = rng.gen_range(30..226);
        let y1 = rng.gen_range(30..226);
        let x2 = (x1 as i32 + rng.gen_range(-50..50)).clamp(0, 255) as u32;
        let y2 = (y1 as i32 + rng.gen_range(-50..50)).clamp(0, 255) as u32;
        draw_line(&mut img, x1, y1, x2, y2, to_rgba(dark, 0.5));
    }

    img
}

fn generate_void_dragon_texture(base: [f32; 3], flame: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(256, 256);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    for y in 0..256 {
        for x in 0..256 {
            let noise = rng.gen::<f32>() * 0.1;
            // Swirling void pattern
            let swirl = ((x as f32 * 0.03).sin() + (y as f32 * 0.03).cos()).abs() * 0.3;

            let r = (base[0] + swirl * flame[0] + noise).clamp(0.0, 1.0);
            let g = (base[1] + swirl * flame[1] * 0.5 + noise).clamp(0.0, 1.0);
            let b = (base[2] + swirl * 0.5 + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add flame streaks
    for _ in 0..20 {
        let x1 = rng.gen_range(0..256);
        let y1 = rng.gen_range(0..256);
        let x2 = (x1 as i32 + rng.gen_range(-30..30)).clamp(0, 255) as u32;
        let y2 = (y1 as i32 + rng.gen_range(-30..30)).clamp(0, 255) as u32;
        draw_line(&mut img, x1, y1, x2, y2, to_rgba(flame, 0.7));
    }

    img
}

fn generate_gem_texture(core: [f32; 3], edge: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(64, 64);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    for y in 0..64 {
        for x in 0..64 {
            let cx = 32.0;
            let cy = 32.0;
            let dist = ((x as f32 - cx).powi(2) + (y as f32 - cy).powi(2)).sqrt() / 45.0;
            let t = dist.clamp(0.0, 1.0);
            let noise = rng.gen::<f32>() * 0.1;

            let r = (core[0] * (1.0 - t) + edge[0] * t + noise).clamp(0.0, 1.0);
            let g = (core[1] * (1.0 - t) + edge[1] * t + noise).clamp(0.0, 1.0);
            let b = (core[2] * (1.0 - t) + edge[2] * t + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    img
}

fn generate_floor_texture(base: [f32; 3], accent: [f32; 3], seed: &str) -> RgbaImage {
    let mut img = RgbaImage::new(512, 512);
    let mut rng = StdRng::seed_from_u64(hash_str(seed));

    // Crystal cavern floor pattern
    for y in 0..512 {
        for x in 0..512 {
            let noise = (rng.gen::<f32>() - 0.5) * 0.15;
            // Tile pattern
            let tile = if (x / 32 + y / 32) % 2 == 0 { 0.0 } else { 0.08 };
            // Crystal vein pattern
            let vein = if (x + y * 3) % 64 < 2 || (x * 2 + y) % 48 < 2 {
                0.3
            } else {
                0.0
            };

            let r = (base[0] + tile + vein * accent[0] + noise).clamp(0.0, 1.0);
            let g = (base[1] + tile + vein * accent[1] + noise).clamp(0.0, 1.0);
            let b = (base[2] + tile + vein * accent[2] + noise).clamp(0.0, 1.0);

            img.put_pixel(x, y, Rgba([
                (r * 255.0) as u8,
                (g * 255.0) as u8,
                (b * 255.0) as u8,
                255
            ]));
        }
    }

    // Add glowing crystal spots
    for _ in 0..30 {
        let cx = rng.gen_range(20..492);
        let cy = rng.gen_range(20..492);
        draw_glow_spot(&mut img, cx, cy, 15, accent);
    }

    img
}

// =============================================================================
// Drawing Helpers
// =============================================================================

fn draw_circle(img: &mut RgbaImage, cx: u32, cy: u32, r: u32, color: Rgba<u8>) {
    let r2 = (r * r) as i32;
    for dy in -(r as i32)..=(r as i32) {
        for dx in -(r as i32)..=(r as i32) {
            if dx * dx + dy * dy <= r2 {
                let x = (cx as i32 + dx).clamp(0, img.width() as i32 - 1) as u32;
                let y = (cy as i32 + dy).clamp(0, img.height() as i32 - 1) as u32;
                blend_pixel(img, x, y, color);
            }
        }
    }
}

fn draw_glow_spot(img: &mut RgbaImage, cx: u32, cy: u32, r: u32, color: [f32; 3]) {
    for dy in -(r as i32)..=(r as i32) {
        for dx in -(r as i32)..=(r as i32) {
            let dist = ((dx * dx + dy * dy) as f32).sqrt() / r as f32;
            if dist <= 1.0 {
                let x = (cx as i32 + dx).clamp(0, img.width() as i32 - 1) as u32;
                let y = (cy as i32 + dy).clamp(0, img.height() as i32 - 1) as u32;
                let alpha = (1.0 - dist) * 0.5;
                blend_pixel(img, x, y, to_rgba(color, alpha));
            }
        }
    }
}

fn draw_line(img: &mut RgbaImage, x0: u32, y0: u32, x1: u32, y1: u32, color: Rgba<u8>) {
    let dx = (x1 as i32 - x0 as i32).abs();
    let dy = (y1 as i32 - y0 as i32).abs();
    let sx = if x0 < x1 { 1i32 } else { -1i32 };
    let sy = if y0 < y1 { 1i32 } else { -1i32 };
    let mut err = dx - dy;
    let mut x = x0 as i32;
    let mut y = y0 as i32;

    let w = img.width() as i32;
    let h = img.height() as i32;

    loop {
        if x >= 0 && x < w && y >= 0 && y < h {
            blend_pixel(img, x as u32, y as u32, color);
        }
        if x == x1 as i32 && y == y1 as i32 { break; }
        let e2 = 2 * err;
        if e2 > -dy { err -= dy; x += sx; }
        if e2 < dx { err += dx; y += sy; }
    }
}

fn draw_rune(img: &mut RgbaImage, cx: u32, cy: u32, color: [f32; 3], rng: &mut StdRng) {
    let size = rng.gen_range(8..16);
    let rgba = to_rgba(color, 0.7);

    // Random rune-like shape
    match rng.gen_range(0..4) {
        0 => { // Cross
            draw_line(img, cx - size, cy, cx + size, cy, rgba);
            draw_line(img, cx, cy - size, cx, cy + size, rgba);
        }
        1 => { // Diamond
            draw_line(img, cx, cy - size, cx + size, cy, rgba);
            draw_line(img, cx + size, cy, cx, cy + size, rgba);
            draw_line(img, cx, cy + size, cx - size, cy, rgba);
            draw_line(img, cx - size, cy, cx, cy - size, rgba);
        }
        2 => { // Triangle
            draw_line(img, cx, cy - size, cx + size, cy + size, rgba);
            draw_line(img, cx + size, cy + size, cx - size, cy + size, rgba);
            draw_line(img, cx - size, cy + size, cx, cy - size, rgba);
        }
        _ => { // Circle outline
            for i in 0..8 {
                let a1 = (i as f32 / 8.0) * PI * 2.0;
                let a2 = ((i + 1) as f32 / 8.0) * PI * 2.0;
                let x1 = cx as i32 + (a1.cos() * size as f32) as i32;
                let y1 = cy as i32 + (a1.sin() * size as f32) as i32;
                let x2 = cx as i32 + (a2.cos() * size as f32) as i32;
                let y2 = cy as i32 + (a2.sin() * size as f32) as i32;
                draw_line(img, x1.max(0) as u32, y1.max(0) as u32, x2.max(0) as u32, y2.max(0) as u32, rgba);
            }
        }
    }
}

fn blend_pixel(img: &mut RgbaImage, x: u32, y: u32, color: Rgba<u8>) {
    if x >= img.width() || y >= img.height() { return; }
    let old = img.get_pixel(x, y);
    let alpha = color[3] as f32 / 255.0;
    let new = Rgba([
        ((old[0] as f32 * (1.0 - alpha) + color[0] as f32 * alpha) as u8),
        ((old[1] as f32 * (1.0 - alpha) + color[1] as f32 * alpha) as u8),
        ((old[2] as f32 * (1.0 - alpha) + color[2] as f32 * alpha) as u8),
        255
    ]);
    img.put_pixel(x, y, new);
}

fn to_rgba(color: [f32; 3], alpha: f32) -> Rgba<u8> {
    Rgba([
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (alpha * 255.0) as u8
    ])
}

// Class emblem drawing
fn draw_shield(img: &mut RgbaImage, cx: u32, cy: u32, color: [f32; 3]) {
    let rgba = to_rgba(color, 0.9);
    for dy in -10i32..=10 {
        let width = if dy < 0 { 8 } else { 8 - dy / 2 };
        for dx in -width..=width {
            blend_pixel(img, (cx as i32 + dx) as u32, (cy as i32 + dy) as u32, rgba);
        }
    }
}

fn draw_star(img: &mut RgbaImage, cx: u32, cy: u32, points: u32, color: [f32; 3]) {
    let rgba = to_rgba(color, 0.9);
    for i in 0..points {
        let angle = (i as f32 / points as f32) * PI * 2.0 - PI / 2.0;
        let x2 = cx as f32 + angle.cos() * 12.0;
        let y2 = cy as f32 + angle.sin() * 12.0;
        draw_line(img, cx, cy, x2 as u32, y2 as u32, rgba);
    }
}

fn draw_arrow(img: &mut RgbaImage, cx: u32, cy: u32, color: [f32; 3]) {
    let rgba = to_rgba(color, 0.9);
    draw_line(img, cx, cy - 12, cx, cy + 12, rgba);
    draw_line(img, cx - 5, cy - 7, cx, cy - 12, rgba);
    draw_line(img, cx + 5, cy - 7, cx, cy - 12, rgba);
}

fn draw_cross(img: &mut RgbaImage, cx: u32, cy: u32, color: [f32; 3]) {
    let rgba = to_rgba(color, 0.9);
    draw_line(img, cx, cy - 12, cx, cy + 8, rgba);
    draw_line(img, cx - 8, cy - 4, cx + 8, cy - 4, rgba);
}

fn draw_skull(img: &mut RgbaImage, cx: u32, cy: u32, color: [f32; 3]) {
    let rgba = to_rgba(color, 0.9);
    draw_circle(img, cx, cy - 2, 8, rgba);
    draw_circle(img, cx - 3, cy - 3, 2, Rgba([0, 0, 0, 200])); // Eye
    draw_circle(img, cx + 3, cy - 3, 2, Rgba([0, 0, 0, 200])); // Eye
    draw_line(img, cx, cy + 4, cx, cy + 10, rgba); // Jaw
}

fn draw_sun(img: &mut RgbaImage, cx: u32, cy: u32, color: [f32; 3]) {
    let rgba = to_rgba(color, 0.9);
    draw_circle(img, cx, cy, 6, rgba);
    for i in 0..8 {
        let angle = (i as f32 / 8.0) * PI * 2.0;
        let x1 = cx as f32 + angle.cos() * 8.0;
        let y1 = cy as f32 + angle.sin() * 8.0;
        let x2 = cx as f32 + angle.cos() * 12.0;
        let y2 = cy as f32 + angle.sin() * 12.0;
        draw_line(img, x1 as u32, y1 as u32, x2 as u32, y2 as u32, rgba);
    }
}

// =============================================================================
// Mesh Generation
// =============================================================================

struct Mesh {
    vertices: Vec<[f32; 3]>,
    uvs: Vec<[f32; 2]>,
    normals: Vec<[f32; 3]>,
    faces: Vec<[[usize; 3]; 3]>, // [vertex, uv, normal] x 3
}

impl Mesh {
    fn new() -> Self {
        Self {
            vertices: Vec::new(),
            uvs: Vec::new(),
            normals: Vec::new(),
            faces: Vec::new(),
        }
    }

    fn add_box(&mut self, cx: f32, cy: f32, cz: f32, hw: f32, hh: f32, hd: f32) {
        let base_v = self.vertices.len();
        let base_n = self.normals.len();
        let base_uv = self.uvs.len();

        // 8 vertices
        self.vertices.push([cx - hw, cy - hh, cz - hd]);
        self.vertices.push([cx + hw, cy - hh, cz - hd]);
        self.vertices.push([cx + hw, cy + hh, cz - hd]);
        self.vertices.push([cx - hw, cy + hh, cz - hd]);
        self.vertices.push([cx - hw, cy - hh, cz + hd]);
        self.vertices.push([cx + hw, cy - hh, cz + hd]);
        self.vertices.push([cx + hw, cy + hh, cz + hd]);
        self.vertices.push([cx - hw, cy + hh, cz + hd]);

        // 6 normals
        self.normals.push([0.0, 0.0, -1.0]); // front
        self.normals.push([0.0, 0.0, 1.0]);  // back
        self.normals.push([0.0, 1.0, 0.0]);  // top
        self.normals.push([0.0, -1.0, 0.0]); // bottom
        self.normals.push([-1.0, 0.0, 0.0]); // left
        self.normals.push([1.0, 0.0, 0.0]);  // right

        // 4 UVs
        self.uvs.push([0.0, 0.0]);
        self.uvs.push([1.0, 0.0]);
        self.uvs.push([1.0, 1.0]);
        self.uvs.push([0.0, 1.0]);

        // Front face
        self.faces.push([[base_v + 0, base_uv + 0, base_n + 0], [base_v + 1, base_uv + 1, base_n + 0], [base_v + 2, base_uv + 2, base_n + 0]]);
        self.faces.push([[base_v + 0, base_uv + 0, base_n + 0], [base_v + 2, base_uv + 2, base_n + 0], [base_v + 3, base_uv + 3, base_n + 0]]);
        // Back face
        self.faces.push([[base_v + 5, base_uv + 0, base_n + 1], [base_v + 4, base_uv + 1, base_n + 1], [base_v + 7, base_uv + 2, base_n + 1]]);
        self.faces.push([[base_v + 5, base_uv + 0, base_n + 1], [base_v + 7, base_uv + 2, base_n + 1], [base_v + 6, base_uv + 3, base_n + 1]]);
        // Top face
        self.faces.push([[base_v + 3, base_uv + 0, base_n + 2], [base_v + 2, base_uv + 1, base_n + 2], [base_v + 6, base_uv + 2, base_n + 2]]);
        self.faces.push([[base_v + 3, base_uv + 0, base_n + 2], [base_v + 6, base_uv + 2, base_n + 2], [base_v + 7, base_uv + 3, base_n + 2]]);
        // Bottom face
        self.faces.push([[base_v + 4, base_uv + 0, base_n + 3], [base_v + 5, base_uv + 1, base_n + 3], [base_v + 1, base_uv + 2, base_n + 3]]);
        self.faces.push([[base_v + 4, base_uv + 0, base_n + 3], [base_v + 1, base_uv + 2, base_n + 3], [base_v + 0, base_uv + 3, base_n + 3]]);
        // Left face
        self.faces.push([[base_v + 4, base_uv + 0, base_n + 4], [base_v + 0, base_uv + 1, base_n + 4], [base_v + 3, base_uv + 2, base_n + 4]]);
        self.faces.push([[base_v + 4, base_uv + 0, base_n + 4], [base_v + 3, base_uv + 2, base_n + 4], [base_v + 7, base_uv + 3, base_n + 4]]);
        // Right face
        self.faces.push([[base_v + 1, base_uv + 0, base_n + 5], [base_v + 5, base_uv + 1, base_n + 5], [base_v + 6, base_uv + 2, base_n + 5]]);
        self.faces.push([[base_v + 1, base_uv + 0, base_n + 5], [base_v + 6, base_uv + 2, base_n + 5], [base_v + 2, base_uv + 3, base_n + 5]]);
    }

    fn add_sphere(&mut self, cx: f32, cy: f32, cz: f32, radius: f32, segments: u32, rings: u32) {
        let base_v = self.vertices.len();
        let base_n = self.normals.len();
        let base_uv = self.uvs.len();

        // Generate vertices
        for ring in 0..=rings {
            let phi = PI * (ring as f32 / rings as f32);
            for seg in 0..=segments {
                let theta = 2.0 * PI * (seg as f32 / segments as f32);

                let x = phi.sin() * theta.cos();
                let y = phi.cos();
                let z = phi.sin() * theta.sin();

                self.vertices.push([cx + x * radius, cy + y * radius, cz + z * radius]);
                self.normals.push([x, y, z]);
                self.uvs.push([seg as f32 / segments as f32, ring as f32 / rings as f32]);
            }
        }

        // Generate faces
        let row_size = (segments + 1) as usize;
        for ring in 0..rings as usize {
            for seg in 0..segments as usize {
                let v0 = base_v + ring * row_size + seg;
                let v1 = base_v + ring * row_size + seg + 1;
                let v2 = base_v + (ring + 1) * row_size + seg + 1;
                let v3 = base_v + (ring + 1) * row_size + seg;

                let uv0 = base_uv + ring * row_size + seg;
                let uv1 = base_uv + ring * row_size + seg + 1;
                let uv2 = base_uv + (ring + 1) * row_size + seg + 1;
                let uv3 = base_uv + (ring + 1) * row_size + seg;

                self.faces.push([[v0, uv0, base_n + ring * row_size + seg], [v1, uv1, base_n + ring * row_size + seg + 1], [v2, uv2, base_n + (ring + 1) * row_size + seg + 1]]);
                self.faces.push([[v0, uv0, base_n + ring * row_size + seg], [v2, uv2, base_n + (ring + 1) * row_size + seg + 1], [v3, uv3, base_n + (ring + 1) * row_size + seg]]);
            }
        }
    }

    fn add_diamond(&mut self, cx: f32, cy: f32, cz: f32, radius: f32, height: f32) {
        let base_v = self.vertices.len();
        let base_n = self.normals.len();
        let base_uv = self.uvs.len();

        // Diamond: top point, middle ring, bottom point
        let top = cy + height / 2.0;
        let bottom = cy - height / 2.0;
        let mid = cy;

        // Top vertex
        self.vertices.push([cx, top, cz]);
        // Middle ring (6 vertices)
        for i in 0..6 {
            let angle = (i as f32 / 6.0) * PI * 2.0;
            self.vertices.push([cx + angle.cos() * radius, mid, cz + angle.sin() * radius]);
        }
        // Bottom vertex
        self.vertices.push([cx, bottom, cz]);

        // Normals (simplified)
        self.normals.push([0.0, 1.0, 0.0]);  // top
        for i in 0..6 {
            let angle = (i as f32 / 6.0) * PI * 2.0;
            self.normals.push([angle.cos(), 0.5, angle.sin()]);
        }
        self.normals.push([0.0, -1.0, 0.0]); // bottom

        // UVs
        self.uvs.push([0.5, 0.0]); // top
        for i in 0..6 {
            self.uvs.push([(i as f32 / 6.0), 0.5]);
        }
        self.uvs.push([0.5, 1.0]); // bottom

        // Top cone faces
        for i in 0..6 {
            let next = (i + 1) % 6;
            self.faces.push([[base_v, base_uv, base_n], [base_v + 1 + i, base_uv + 1 + i, base_n + 1 + i], [base_v + 1 + next, base_uv + 1 + next, base_n + 1 + next]]);
        }
        // Bottom cone faces
        for i in 0..6 {
            let next = (i + 1) % 6;
            self.faces.push([[base_v + 7, base_uv + 7, base_n + 7], [base_v + 1 + next, base_uv + 1 + next, base_n + 1 + next], [base_v + 1 + i, base_uv + 1 + i, base_n + 1 + i]]);
        }
    }

    fn add_quad_floor(&mut self, cx: f32, cy: f32, cz: f32, width: f32, depth: f32) {
        let base_v = self.vertices.len();
        let base_n = self.normals.len();
        let base_uv = self.uvs.len();

        let hw = width / 2.0;
        let hd = depth / 2.0;

        self.vertices.push([cx - hw, cy, cz - hd]);
        self.vertices.push([cx + hw, cy, cz - hd]);
        self.vertices.push([cx + hw, cy, cz + hd]);
        self.vertices.push([cx - hw, cy, cz + hd]);

        self.normals.push([0.0, 1.0, 0.0]);

        self.uvs.push([0.0, 0.0]);
        self.uvs.push([1.0, 0.0]);
        self.uvs.push([1.0, 1.0]);
        self.uvs.push([0.0, 1.0]);

        self.faces.push([[base_v + 0, base_uv + 0, base_n], [base_v + 1, base_uv + 1, base_n], [base_v + 2, base_uv + 2, base_n]]);
        self.faces.push([[base_v + 0, base_uv + 0, base_n], [base_v + 2, base_uv + 2, base_n], [base_v + 3, base_uv + 3, base_n]]);
    }
}

fn write_obj(mesh: &Mesh, path: &Path) {
    let mut file = File::create(path).unwrap();

    // Write header
    writeln!(file, "# Generated by Nethercore Asset Generator v2").unwrap();
    writeln!(file, "# Vertices: {} UVs: {} Triangles: {}", mesh.vertices.len(), mesh.uvs.len(), mesh.faces.len()).unwrap();
    writeln!(file).unwrap();

    // Write vertices
    for v in &mesh.vertices {
        writeln!(file, "v {} {} {}", v[0], v[1], v[2]).unwrap();
    }
    writeln!(file).unwrap();

    // Write UVs (critical for texturing)
    for uv in &mesh.uvs {
        writeln!(file, "vt {} {}", uv[0], uv[1]).unwrap();
    }
    writeln!(file).unwrap();

    // Write normals
    for n in &mesh.normals {
        writeln!(file, "vn {} {} {}", n[0], n[1], n[2]).unwrap();
    }
    writeln!(file).unwrap();

    // Write faces (OBJ is 1-indexed) with format f v/vt/vn
    for f in &mesh.faces {
        writeln!(file, "f {}/{}/{} {}/{}/{} {}/{}/{}",
            f[0][0] + 1, f[0][1] + 1, f[0][2] + 1,
            f[1][0] + 1, f[1][1] + 1, f[1][2] + 1,
            f[2][0] + 1, f[2][1] + 1, f[2][2] + 1
        ).unwrap();
    }
}

fn hash_str(s: &str) -> u64 {
    let mut hash: u64 = 5381;
    for c in s.bytes() {
        hash = hash.wrapping_mul(33).wrapping_add(c as u64);
    }
    hash
}
