//! NEON DRIFT - Procedural Asset Generator
//!
//! Generates meshes, textures, and sounds for the arcade racing game.
//! Vehicles: Speedster, Muscle, Racer, Drift
//! Track: Straight, Curves, Tunnel, Jump
//! Props: Barrier, Boost Pad, Billboard, Building

mod cars;
mod sounds;
mod track;
mod textures;

use proc_gen::audio::*;
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

    // Sounds
    println!("\n  --- Sounds ---");
    generate_sounds(&audio_dir);
}

fn generate_sounds(output_dir: &Path) {
    println!("  Generating {} sounds", sounds::SOUNDS.len());

    let synth = Synth::new(SAMPLE_RATE);

    for (id, description) in sounds::SOUNDS {
        let samples = generate_neon_sound(&synth, id);
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

fn generate_neon_sound(synth: &Synth, id: &str) -> Vec<f32> {
    match id {
        // Engine
        "engine_idle" => {
            // Low rumble loop
            let sustain_env = Envelope::new(0.01, 0.1, 0.9, 0.2);
            let base = synth.tone(Waveform::Saw, 80.0, 1.0, sustain_env);
            let harmonics = synth.tone(Waveform::Triangle, 120.0, 1.0, sustain_env);
            mix(&[(&base, 0.6), (&harmonics, 0.3)])
        }
        "engine_rev" => {
            synth.sweep(Waveform::Saw, 200.0, 600.0, 0.4, Envelope::pad())
        }
        "boost" => {
            let sweep1 = synth.sweep(Waveform::Square, 400.0, 1200.0, 0.3, Envelope::pluck());
            let sweep2 = synth.sweep(Waveform::Saw, 600.0, 1400.0, 0.35, Envelope::pad());
            mix(&[(&sweep1, 0.5), (&sweep2, 0.4)])
        }

        // Driving
        "drift" => {
            // Screech/whine
            let noise = synth.filtered_noise(0.6, Some(800.0), None, Envelope::pad(), 42);
            let tone = synth.sweep(Waveform::Saw, 600.0, 400.0, 0.6, Envelope::pad());
            mix(&[(&noise, 0.4), (&tone, 0.3)])
        }
        "brake" => {
            synth.filtered_noise(0.25, Some(600.0), None, Envelope::hit(), 42)
        }
        "shift" => {
            synth.sweep(Waveform::Triangle, 300.0, 250.0, 0.08, Envelope::pluck())
        }

        // Collisions
        "wall" => {
            synth.filtered_noise(0.3, None, Some(800.0), Envelope::hit(), 42)
        }
        "barrier" => {
            let crash = synth.filtered_noise(0.25, None, Some(600.0), Envelope::hit(), 42);
            let clang = synth.tone(Waveform::Triangle, 300.0, 0.15, Envelope::pad());
            mix(&[(&crash, 0.5), (&clang, 0.3)])
        }

        // Race
        "countdown" => {
            synth.tone(Waveform::Square, 800.0, 0.1, Envelope::pluck())
        }
        "checkpoint" => {
            let beep1 = synth.tone(Waveform::Sine, 880.0, 0.08, Envelope::pluck());
            let beep2 = synth.tone(Waveform::Sine, 1320.0, 0.12, Envelope::pad());
            concat(&[&beep1, &beep2])
        }
        "finish" => {
            let decay_env = Envelope::new(0.02, 0.2, 0.0, 0.15);
            let tone1 = synth.tone(Waveform::Square, 523.0, 0.15, Envelope::pluck());
            let tone2 = synth.tone(Waveform::Square, 659.0, 0.15, Envelope::pluck());
            let tone3 = synth.tone(Waveform::Square, 784.0, 0.2, decay_env);
            let tone4 = synth.tone(Waveform::Square, 1046.0, 0.3, decay_env);
            concat(&[&tone1, &tone2, &tone3, &tone4])
        }

        _ => panic!("Unknown sound ID: {}", id),
    }
}
