//! PRISM SURVIVORS - Procedural Asset Generator
//!
//! Generates meshes, textures, and sounds for the top-down survivors game.
//! Heroes: Knight, Mage, Ranger, Cleric, Necromancer, Paladin
//! Basic Enemies: Golem, Crawler, Wisp, Skeleton, Shade, Berserker, Arcane Sentinel
//! Elite Enemies: Crystal Knight, Void Mage, Golem Titan, Specter Lord
//! Bosses: Prism Colossus, Void Dragon
//! Effects: XP Gem, Weapons, Arena Floor, Projectiles

mod bosses;
mod elites;
mod enemies;
mod font;
mod heroes;
mod pickups;
mod sounds;
mod textures;

use crate::mesh_helpers::write_mesh;
use proc_gen::audio::*;
use proc_gen::mesh::*;
use proc_gen::texture::{checker, write_png};
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
    println!("\n  --- Heroes ---");
    heroes::generate_all(&meshes_dir);

    // Basic Enemies
    println!("\n  --- Basic Enemies ---");
    enemies::generate_all(&meshes_dir);

    // Elite Enemies
    println!("\n  --- Elite Enemies ---");
    elites::generate_all(&meshes_dir);

    // Bosses
    println!("\n  --- Bosses ---");
    bosses::generate_all(&meshes_dir);

    // Pickups, Projectiles & Arena
    println!("\n  --- Pickups & Effects ---");
    pickups::generate_all(&meshes_dir);

    // Textures (all categories via consolidated generator)
    println!("\n  --- Textures ---");
    textures::generate_all(&textures_dir);

    // Font
    println!("\n  --- Font ---");
    font::generate(&textures_dir);

    // Sounds
    println!("\n  --- Sounds ---");
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
    let mesh: UnpackedMesh = generate_cube(1.0, 1.0, 1.0);
    write_mesh(&mesh, "test_cube", output_dir);
}

fn generate_test_sphere(output_dir: &Path) {
    let mesh: UnpackedMesh = generate_sphere(0.5, 16, 12);
    write_mesh(&mesh, "test_sphere", output_dir);
}

fn generate_test_texture(output_dir: &Path) {
    println!("  Generating: test_checker.png");
    let tex = checker(64, 64, 8, [200, 200, 200, 255], [50, 50, 50, 255]);
    let path = output_dir.join("test_checker.png");
    write_png(&tex, &path).expect("Failed to write PNG");
    println!("    -> {}", path.display());
}
