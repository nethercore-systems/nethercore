//! NEON DRIFT - Procedural Asset Generator
//!
//! Generates meshes, textures, and sounds for the arcade racing game.
//! Vehicles: Speedster, Muscle, Racer, Drift, Phantom, Titan, Viper
//! Track: Straight, Curves, Tunnel, Jump
//! Props: Barrier, Boost Pad, Billboard, Building

mod cars;
mod sounds;
mod track;
mod textures;

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

    // Vehicles
    println!("\n  --- Vehicles ---");
    cars::generate_speedster(&meshes_dir);
    cars::generate_muscle(&meshes_dir);
    cars::generate_racer(&meshes_dir);
    cars::generate_drift(&meshes_dir);
    cars::generate_phantom(&meshes_dir);
    cars::generate_titan(&meshes_dir);
    cars::generate_viper(&meshes_dir);

    // Track segments
    println!("\n  --- Track Segments ---");
    track::generate_straight(&meshes_dir);
    track::generate_curve_left(&meshes_dir);
    track::generate_curve_right(&meshes_dir);
    track::generate_tunnel(&meshes_dir);
    track::generate_jump_ramp(&meshes_dir);

    // Props (generic)
    println!("\n  --- Props ---");
    track::generate_barrier(&meshes_dir);
    track::generate_boost_pad(&meshes_dir);
    track::generate_billboard(&meshes_dir);
    track::generate_building(&meshes_dir);

    // Sunset Strip props
    println!("\n  --- Sunset Strip Props ---");
    track::generate_palm_tree(&meshes_dir);
    track::generate_highway_sign(&meshes_dir);

    // Neon City props
    println!("\n  --- Neon City Props ---");
    track::generate_hologram_ad(&meshes_dir);
    track::generate_street_lamp(&meshes_dir);

    // Void Tunnel props
    println!("\n  --- Void Tunnel Props ---");
    track::generate_energy_pillar(&meshes_dir);
    track::generate_portal_ring(&meshes_dir);

    // Crystal Cavern props
    println!("\n  --- Crystal Cavern Props ---");
    track::generate_glowing_mushrooms(&meshes_dir);

    // Solar Highway props
    println!("\n  --- Solar Highway Props ---");
    track::generate_heat_vent(&meshes_dir);
    track::generate_solar_beacon(&meshes_dir);

    // Crystal Cavern segments
    println!("\n  --- Crystal Cavern ---");
    track::generate_crystal_formation(&meshes_dir);
    track::generate_cavern_scurve(&meshes_dir);
    track::generate_cavern_low_ceiling(&meshes_dir);

    // Solar Highway segments
    println!("\n  --- Solar Highway ---");
    track::generate_solar_straight(&meshes_dir);
    track::generate_solar_curve(&meshes_dir);
    track::generate_solar_flare_jump(&meshes_dir);

    // Textures (all categories via consolidated generator)
    println!("\n  --- Textures ---");
    textures::generate_all(&textures_dir);

    // Font
    println!("\n  --- Font ---");
    textures::generate_font_texture(&textures_dir);

    // Sounds
    println!("\n  --- Sounds ---");
    sounds::generate_all(&audio_dir);
}
