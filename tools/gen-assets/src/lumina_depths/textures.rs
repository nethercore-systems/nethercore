//! Texture generators for LUMINA DEPTHS
//!
//! Underwater materials with bioluminescence, translucency, and wet surfaces.
//! Uses Mode 3 (Specular-Shininess) for underwater light effects.

use proc_gen::texture::*;
use std::path::Path;

/// Generate creature textures (fish, jellyfish, etc.)
pub fn generate_creature_textures(output_dir: &Path) {
    // Reef fish - colorful tropical colors
    let fish_colors = [
        ("reef_fish_orange", [255, 140, 50, 255]),
        ("reef_fish_blue", [50, 150, 255, 255]),
        ("reef_fish_yellow", [255, 220, 50, 255]),
        ("reef_fish_purple", [180, 80, 200, 255]),
    ];

    for (name, color) in fish_colors {
        let tex = gradient_v(32, 32, color, [
            (color[0] as i32 - 40).max(0) as u8,
            (color[1] as i32 - 40).max(0) as u8,
            (color[2] as i32 - 40).max(0) as u8,
            255,
        ]);
        let path = output_dir.join(format!("{}.png", name));
        write_png(&tex, &path).expect("Failed to write PNG");
        println!("  -> Written: {}", path.display());
    }

    // Sea turtle - mottled green/brown shell
    let turtle_shell = stone(64, 64, [60, 80, 50, 255], 42);
    let path = output_dir.join("turtle_shell.png");
    write_png(&turtle_shell, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Manta ray - dark dorsal, light ventral
    let manta_dorsal = gradient_v(64, 64, [30, 35, 40, 255], [20, 25, 30, 255]);
    let path = output_dir.join("manta_dorsal.png");
    write_png(&manta_dorsal, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    let manta_ventral = solid(64, 64, [200, 200, 210, 255]);
    let path = output_dir.join("manta_ventral.png");
    write_png(&manta_ventral, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Moon jelly - translucent with soft glow
    let jelly = gradient_radial(64, 64, [180, 200, 255, 180], [120, 140, 200, 100]);
    let path = output_dir.join("moon_jelly.png");
    write_png(&jelly, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Bioluminescent emissive (for deep creatures)
    let bio_cyan = solid(32, 32, [0, 255, 230, 255]);
    let path = output_dir.join("bio_cyan.png");
    write_png(&bio_cyan, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    let bio_magenta = solid(32, 32, [255, 50, 200, 255]);
    let path = output_dir.join("bio_magenta.png");
    write_png(&bio_magenta, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    let bio_green = solid(32, 32, [100, 255, 100, 255]);
    let path = output_dir.join("bio_green.png");
    write_png(&bio_green, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Anglerfish lure - bright yellow/orange
    let lure = gradient_radial(16, 16, [255, 220, 100, 255], [255, 180, 50, 255]);
    let path = output_dir.join("anglerfish_lure.png");
    write_png(&lure, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Deep creature body (near black)
    let deep_body = solid(32, 32, [15, 15, 20, 255]);
    let path = output_dir.join("deep_body.png");
    write_png(&deep_body, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Whale skin - blue-gray with mottling
    let whale = stone(128, 128, [60, 70, 85, 255], 77);
    let path = output_dir.join("whale_skin.png");
    write_png(&whale, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Tube worm - white tube, red plume
    let tube_worm_tube = solid(32, 32, [230, 225, 220, 255]);
    let path = output_dir.join("tube_worm_tube.png");
    write_png(&tube_worm_tube, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    let tube_worm_plume = solid(32, 32, [200, 40, 40, 255]);
    let path = output_dir.join("tube_worm_plume.png");
    write_png(&tube_worm_plume, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());
}

/// Generate flora textures (coral, kelp, etc.)
pub fn generate_flora_textures(output_dir: &Path) {
    // Brain coral - pinkish tan
    let brain_coral = stone(64, 64, [180, 140, 120, 255], 33);
    let path = output_dir.join("coral_brain.png");
    write_png(&brain_coral, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Fan coral - purple/red
    let fan_coral = gradient_v(64, 64, [150, 60, 100, 255], [120, 40, 80, 255]);
    let path = output_dir.join("coral_fan.png");
    write_png(&fan_coral, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Branching coral - orange/yellow
    let branch_coral = solid(64, 64, [255, 160, 80, 255]);
    let path = output_dir.join("coral_branch.png");
    write_png(&branch_coral, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Kelp - green-brown
    let kelp = gradient_v(64, 64, [60, 80, 40, 255], [40, 60, 30, 255]);
    let path = output_dir.join("kelp.png");
    write_png(&kelp, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Anemone - various colors
    let anemone_pink = gradient_radial(32, 32, [255, 150, 180, 255], [200, 100, 130, 255]);
    let path = output_dir.join("anemone.png");
    write_png(&anemone_pink, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Sea grass - pale green
    let sea_grass = solid(32, 32, [80, 120, 60, 255]);
    let path = output_dir.join("sea_grass.png");
    write_png(&sea_grass, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());
}

/// Generate terrain textures (rocks, sand, vents)
pub fn generate_terrain_textures(output_dir: &Path) {
    // Rock - gray with variation
    let rock = stone(128, 128, [80, 80, 85, 255], 99);
    let path = output_dir.join("rock.png");
    write_png(&rock, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Seafloor sand - tan
    let sand = stone(128, 128, [160, 140, 110, 255], 55);
    let path = output_dir.join("sand.png");
    write_png(&sand, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Vent chimney - dark with mineral deposits
    let vent = gradient_v(64, 64, [50, 45, 40, 255], [30, 28, 25, 255]);
    let path = output_dir.join("vent_chimney.png");
    write_png(&vent, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Vent mineral deposits (yellow/orange sulfur)
    let sulfur = solid(32, 32, [200, 180, 60, 255]);
    let path = output_dir.join("sulfur_deposit.png");
    write_png(&sulfur, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Bubble (translucent)
    let bubble = gradient_radial(16, 16, [200, 220, 255, 150], [150, 180, 220, 80]);
    let path = output_dir.join("bubble.png");
    write_png(&bubble, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());
}

/// Generate submersible textures
pub fn generate_submersible_textures(output_dir: &Path) {
    // Hull - matte metal
    let hull = metal(64, 64, [140, 150, 160, 255], 11);
    let path = output_dir.join("sub_hull.png");
    write_png(&hull, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Glass dome (tinted)
    let glass = solid(32, 32, [180, 200, 220, 200]);
    let path = output_dir.join("sub_glass.png");
    write_png(&glass, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Headlight (emissive white)
    let headlight = gradient_radial(32, 32, [255, 255, 240, 255], [255, 240, 200, 255]);
    let path = output_dir.join("headlight.png");
    write_png(&headlight, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());

    // Thruster glow (blue emissive)
    let thruster = gradient_radial(16, 16, [100, 180, 255, 255], [50, 100, 200, 255]);
    let path = output_dir.join("thruster_glow.png");
    write_png(&thruster, &path).expect("Failed to write PNG");
    println!("  -> Written: {}", path.display());
}
