//! LUMINA DEPTHS - Procedural Asset Generator
//!
//! Generates meshes, textures, and sounds for the underwater exploration game.
//! Zones: Sunlit Waters, Twilight Realm, Midnight Abyss, Hydrothermal Vents
//! Player vessel: Submersible

mod creatures;
mod flora;
mod sounds;
mod submersible;
mod terrain;
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

    // Sounds
    println!("\n  --- Sounds ---");
    generate_sounds(&audio_dir);
}

fn generate_sounds(output_dir: &Path) {
    println!("  Generating {} sounds", sounds::SOUNDS.len());

    let synth = Synth::new(SAMPLE_RATE);

    for (id, description) in sounds::SOUNDS {
        let samples = generate_lumina_sound(&synth, id);
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

fn generate_lumina_sound(synth: &Synth, id: &str) -> Vec<f32> {
    match id {
        // Submersible
        "sonar" => {
            let ping = synth.tone(Waveform::Sine, 1200.0, 0.05, Envelope::pluck());
            let echo1 = synth.tone(Waveform::Sine, 1200.0, 0.03, Envelope::pluck());
            let echo2 = synth.tone(Waveform::Sine, 1200.0, 0.02, Envelope::pluck());
            let silence1 = silence(0.15, SAMPLE_RATE);
            let silence2 = silence(0.1, SAMPLE_RATE);
            concat(&[&ping, &silence1, &echo1, &silence2, &echo2])
        }
        "propeller" => {
            // Low frequency hum with modulation
            let sustain_env = Envelope::new(0.01, 0.1, 0.9, 0.2);
            let base = synth.tone(Waveform::Triangle, 60.0, 0.8, sustain_env);
            let harmonics = synth.tone(Waveform::Sine, 90.0, 0.8, sustain_env);
            mix(&[(&base, 0.4), (&harmonics, 0.2)])
        }
        "surface" => {
            synth.sweep(Waveform::Triangle, 200.0, 400.0, 0.5, Envelope::pad())
        }

        // Creatures
        "whale" => {
            // Deep, resonant call
            let decay_env = Envelope::new(0.05, 0.6, 0.0, 0.4);
            let call1 = synth.sweep(Waveform::Sine, 80.0, 120.0, 1.2, decay_env);
            let call2 = synth.sweep(Waveform::Sine, 100.0, 70.0, 0.8, decay_env);
            concat(&[&call1, &call2])
        }
        "fish" => {
            // Quick bubbling movement
            synth.filtered_noise(0.3, Some(1000.0), None, Envelope::pluck(), 42)
        }
        "jellyfish" => {
            // Soft pulsing tone
            let pulse1 = synth.tone(Waveform::Sine, 220.0, 0.4, Envelope::pluck());
            let silence1 = silence(0.2, SAMPLE_RATE);
            let pulse2 = synth.tone(Waveform::Sine, 220.0, 0.4, Envelope::pluck());
            concat(&[&pulse1, &silence1, &pulse2])
        }

        // Environment
        "bubbles" => {
            synth.filtered_noise(0.6, Some(1200.0), None, Envelope::pad(), 42)
        }
        "vent" => {
            // Deep rumble
            let sustain_env = Envelope::new(0.1, 0.2, 0.9, 0.3);
            let rumble = synth.filtered_noise(1.5, None, Some(200.0), sustain_env, 42);
            let base = synth.tone(Waveform::Saw, 40.0, 1.5, sustain_env);
            mix(&[(&rumble, 0.4), (&base, 0.3)])
        }
        "cave" => {
            // Water drip with reverb-like echo
            let drip = synth.tone(Waveform::Sine, 800.0, 0.02, Envelope::pluck());
            let silence1 = silence(0.15, SAMPLE_RATE);
            let echo1 = synth.tone(Waveform::Sine, 800.0, 0.015, Envelope::pluck());
            let silence2 = silence(0.1, SAMPLE_RATE);
            let echo2 = synth.tone(Waveform::Sine, 800.0, 0.01, Envelope::pluck());
            concat(&[&drip, &silence1, &echo1, &silence2, &echo2])
        }

        // Discovery
        "artifact" => {
            // Mysterious discovery chime
            let decay_env = Envelope::new(0.02, 0.3, 0.0, 0.2);
            let chime1 = synth.tone(Waveform::Sine, 659.0, 0.3, decay_env);
            let chime2 = synth.tone(Waveform::Sine, 880.0, 0.35, decay_env);
            let chime3 = synth.tone(Waveform::Sine, 1046.0, 0.4, decay_env);
            mix(&[(&chime1, 0.3), (&chime2, 0.3), (&chime3, 0.3)])
        }
        "scan" => {
            synth.sweep(Waveform::Sine, 600.0, 1200.0, 0.25, Envelope::pluck())
        }
        "log" => {
            let beep1 = synth.tone(Waveform::Square, 700.0, 0.06, Envelope::pluck());
            let beep2 = synth.tone(Waveform::Square, 850.0, 0.08, Envelope::pluck());
            concat(&[&beep1, &beep2])
        }

        _ => panic!("Unknown sound ID: {}", id),
    }
}
