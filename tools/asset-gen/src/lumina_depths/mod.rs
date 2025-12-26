//! LUMINA DEPTHS - Procedural Asset Generator
//!
//! Generates meshes and textures for the underwater exploration game.
//! Zones: Sunlit Waters, Twilight Realm, Midnight Abyss, Hydrothermal Vents
//! Player vessel: Submersible

mod creatures;
mod flora;
mod submersible;
mod terrain;
mod textures;

use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    let meshes_dir = output_dir.join("meshes");
    let textures_dir = output_dir.join("textures");

    std::fs::create_dir_all(&meshes_dir).unwrap();
    std::fs::create_dir_all(&textures_dir).unwrap();

    println!("  Meshes -> {}", meshes_dir.display());
    println!("  Textures -> {}", textures_dir.display());

    // Player vessel
    println!("\n  --- Submersible ---");
    submersible::generate_submersible(&meshes_dir);

    // Zone 1: Sunlit Waters
    println!("\n  --- Zone 1: Sunlit Waters ---");
    creatures::generate_reef_fish(&meshes_dir);
    creatures::generate_sea_turtle(&meshes_dir);
    creatures::generate_manta_ray(&meshes_dir);

    // Zone 2: Twilight Realm
    println!("\n  --- Zone 2: Twilight Realm ---");
    creatures::generate_moon_jelly(&meshes_dir);
    creatures::generate_lanternfish(&meshes_dir);
    creatures::generate_siphonophore(&meshes_dir);

    // Zone 3: Midnight Abyss
    println!("\n  --- Zone 3: Midnight Abyss ---");
    creatures::generate_anglerfish(&meshes_dir);
    creatures::generate_gulper_eel(&meshes_dir);
    creatures::generate_dumbo_octopus(&meshes_dir);

    // Zone 4: Hydrothermal Vents
    println!("\n  --- Zone 4: Hydrothermal Vents ---");
    creatures::generate_tube_worms(&meshes_dir);
    creatures::generate_vent_shrimp(&meshes_dir);

    // Epic Encounters
    println!("\n  --- Epic Encounters ---");
    creatures::generate_blue_whale(&meshes_dir);

    // Flora
    println!("\n  --- Flora ---");
    flora::generate_coral_brain(&meshes_dir);
    flora::generate_coral_fan(&meshes_dir);
    flora::generate_coral_branch(&meshes_dir);
    flora::generate_kelp(&meshes_dir);
    flora::generate_anemone(&meshes_dir);
    flora::generate_sea_grass(&meshes_dir);

    // Terrain
    println!("\n  --- Terrain ---");
    terrain::generate_rock_boulder(&meshes_dir);
    terrain::generate_rock_pillar(&meshes_dir);
    terrain::generate_vent_chimney(&meshes_dir);
    terrain::generate_seafloor_patch(&meshes_dir);

    // Effects
    println!("\n  --- Effects ---");
    terrain::generate_bubble_cluster(&meshes_dir);

    // Textures
    println!("\n  --- Textures ---");
    textures::generate_creature_textures(&textures_dir);
    textures::generate_flora_textures(&textures_dir);
    textures::generate_terrain_textures(&textures_dir);
    textures::generate_submersible_textures(&textures_dir);
}
