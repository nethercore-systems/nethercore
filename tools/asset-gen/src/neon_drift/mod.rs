//! NEON DRIFT - Procedural Asset Generator
//!
//! Generates meshes and textures for the arcade racing game.
//! Vehicles: Speedster, Muscle, Racer, Drift
//! Track: Straight, Curves, Tunnel, Jump
//! Props: Barrier, Boost Pad, Billboard, Building

mod cars;
mod track;
mod textures;

use std::path::Path;

pub fn generate_all(output_dir: &Path) {
    let meshes_dir = output_dir.join("meshes");
    let textures_dir = output_dir.join("textures");

    std::fs::create_dir_all(&meshes_dir).unwrap();
    std::fs::create_dir_all(&textures_dir).unwrap();

    println!("  Meshes -> {}", meshes_dir.display());
    println!("  Textures -> {}", textures_dir.display());

    // Vehicles
    println!("\n  --- Vehicles ---");
    cars::generate_speedster(&meshes_dir);
    cars::generate_muscle(&meshes_dir);
    cars::generate_racer(&meshes_dir);
    cars::generate_drift(&meshes_dir);

    // Track segments
    println!("\n  --- Track Segments ---");
    track::generate_straight(&meshes_dir);
    track::generate_curve_left(&meshes_dir);
    track::generate_curve_right(&meshes_dir);
    track::generate_tunnel(&meshes_dir);
    track::generate_jump_ramp(&meshes_dir);

    // Props
    println!("\n  --- Props ---");
    track::generate_barrier(&meshes_dir);
    track::generate_boost_pad(&meshes_dir);
    track::generate_billboard(&meshes_dir);
    track::generate_building(&meshes_dir);

    // Textures
    println!("\n  --- Textures ---");
    textures::generate_vehicle_textures(&textures_dir);
    textures::generate_track_textures(&textures_dir);
    textures::generate_prop_textures(&textures_dir);
}
