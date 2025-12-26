//! Texture generators for NEON DRIFT
//!
//! Synthwave/cyberpunk aesthetic textures.

use proc_gen::texture::*;
use std::path::Path;

pub fn generate_vehicle_textures(output_dir: &Path) {
    let speedster = metal(128, 128, [180, 190, 200, 255], 1);
    let path = output_dir.join("speedster.png");
    write_png(&speedster, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let speedster_emissive = solid(64, 64, [0, 255, 255, 255]);
    let path = output_dir.join("speedster_emissive.png");
    write_png(&speedster_emissive, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let mut muscle = metal(128, 128, [60, 60, 70, 255], 2);
    muscle.apply(Contrast { factor: 1.3 });
    let path = output_dir.join("muscle.png");
    write_png(&muscle, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let muscle_emissive = gradient_v(64, 64, [255, 100, 0, 255], [255, 50, 0, 255]);
    let path = output_dir.join("muscle_emissive.png");
    write_png(&muscle_emissive, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let racer = gradient_h(128, 128, [240, 240, 250, 255], [220, 220, 240, 255]);
    let path = output_dir.join("racer.png");
    write_png(&racer, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let racer_emissive = solid(64, 64, [255, 0, 180, 255]);
    let path = output_dir.join("racer_emissive.png");
    write_png(&racer_emissive, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let drift = gradient_v(128, 128, [80, 80, 100, 255], [40, 40, 60, 255]);
    let path = output_dir.join("drift.png");
    write_png(&drift, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let drift_emissive = solid(64, 64, [180, 0, 255, 255]);
    let path = output_dir.join("drift_emissive.png");
    write_png(&drift_emissive, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let tire = checker(64, 64, 8, [30, 30, 35, 255], [20, 20, 25, 255]);
    let path = output_dir.join("tire.png");
    write_png(&tire, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());
}

pub fn generate_track_textures(output_dir: &Path) {
    let mut road = stone(256, 256, [40, 40, 45, 255], 42);
    road.apply(Contrast { factor: 0.8 });
    let path = output_dir.join("road.png");
    write_png(&road, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let barrier = stone(64, 64, [80, 75, 70, 255], 123);
    let path = output_dir.join("barrier.png");
    write_png(&barrier, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let neon_strip = gradient_h(32, 4, [255, 0, 128, 255], [0, 255, 255, 255]);
    let path = output_dir.join("neon_strip.png");
    write_png(&neon_strip, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let boost = gradient_radial(64, 64, [0, 255, 255, 255], [0, 150, 200, 255]);
    let path = output_dir.join("boost_pad.png");
    write_png(&boost, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let tunnel = metal(128, 128, [50, 50, 60, 255], 77);
    let path = output_dir.join("tunnel.png");
    write_png(&tunnel, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let ramp = checker(64, 64, 8, [255, 200, 0, 255], [30, 30, 30, 255]);
    let path = output_dir.join("ramp.png");
    write_png(&ramp, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());
}

pub fn generate_prop_textures(output_dir: &Path) {
    let billboard = solid(128, 64, [20, 20, 30, 255]);
    let path = output_dir.join("billboard.png");
    write_png(&billboard, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let billboard_neon = solid(32, 32, [255, 20, 147, 255]);
    let path = output_dir.join("billboard_neon.png");
    write_png(&billboard_neon, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let building = gradient_v(128, 256, [30, 35, 50, 255], [20, 25, 40, 255]);
    let path = output_dir.join("building.png");
    write_png(&building, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let windows = checker(64, 128, 8, [100, 150, 200, 255], [20, 30, 50, 255]);
    let path = output_dir.join("building_windows.png");
    write_png(&windows, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());

    let antenna = solid(8, 8, [255, 0, 0, 255]);
    let path = output_dir.join("antenna_light.png");
    write_png(&antenna, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());
}
