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
    creatures::generate_coral_crab(&meshes_dir);

    // Zone 2: Twilight Realm
    println!("\n  --- Zone 2: Twilight Realm ---");
    creatures::generate_moon_jelly(&meshes_dir);
    creatures::generate_lanternfish(&meshes_dir);
    creatures::generate_siphonophore(&meshes_dir);
    creatures::generate_giant_squid(&meshes_dir);

    // Zone 3: Midnight Abyss
    println!("\n  --- Zone 3: Midnight Abyss ---");
    creatures::generate_anglerfish(&meshes_dir);
    creatures::generate_gulper_eel(&meshes_dir);
    creatures::generate_dumbo_octopus(&meshes_dir);
    creatures::generate_vampire_squid(&meshes_dir);

    // Zone 4: Hydrothermal Vents
    println!("\n  --- Zone 4: Hydrothermal Vents ---");
    creatures::generate_tube_worms(&meshes_dir);
    creatures::generate_vent_shrimp(&meshes_dir);
    creatures::generate_ghost_fish(&meshes_dir);
    creatures::generate_vent_octopus(&meshes_dir);

    // Epic Encounters
    println!("\n  --- Epic Encounters ---");
    creatures::generate_blue_whale(&meshes_dir);
    creatures::generate_sperm_whale(&meshes_dir);
    creatures::generate_giant_isopod(&meshes_dir);

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

    // Custom Font
    println!("\n  --- Font ---");
    textures::generate_font(&textures_dir);

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
        // === SUBMERSIBLE ===
        "sonar" => {
            let ping = synth.tone(Waveform::Sine, 1200.0, 0.05, Envelope::pluck());
            let echo1 = synth.tone(Waveform::Sine, 1180.0, 0.03, Envelope::pluck());
            let echo2 = synth.tone(Waveform::Sine, 1160.0, 0.02, Envelope::pluck());
            let silence1 = silence(0.15, SAMPLE_RATE);
            let silence2 = silence(0.12, SAMPLE_RATE);
            concat(&[&ping, &silence1, &echo1, &silence2, &echo2])
        }
        "propeller" => {
            let sustain_env = Envelope::new(0.01, 0.1, 0.9, 0.2);
            let base = synth.tone(Waveform::Triangle, 60.0, 0.8, sustain_env);
            let harmonics = synth.tone(Waveform::Sine, 90.0, 0.8, sustain_env);
            let modulation = synth.tone(Waveform::Sine, 120.0, 0.8, sustain_env);
            mix(&[(&base, 0.35), (&harmonics, 0.2), (&modulation, 0.15)])
        }
        "surface" => {
            let rush = synth.sweep(Waveform::Triangle, 200.0, 600.0, 0.4, Envelope::pad());
            let bubbles = synth.filtered_noise(0.4, Some(2000.0), None, Envelope::pad(), 77);
            mix(&[(&rush, 0.5), (&bubbles, 0.4)])
        }
        "hull_creak" => {
            // Metal stress sounds
            let creak1 = synth.sweep(Waveform::Saw, 80.0, 60.0, 0.15, Envelope::pluck());
            let creak2 = synth.sweep(Waveform::Saw, 120.0, 90.0, 0.2, Envelope::pluck());
            let silence = silence(0.1, SAMPLE_RATE);
            concat(&[&creak1, &silence, &creak2])
        }
        "pressure_warning" => {
            // Alarming beep pattern
            let beep = synth.tone(Waveform::Square, 880.0, 0.1, Envelope::pluck());
            let pause = silence(0.1, SAMPLE_RATE);
            concat(&[&beep, &pause, &beep, &pause, &beep])
        }
        "headlight_on" => {
            synth.sweep(Waveform::Sine, 400.0, 800.0, 0.15, Envelope::pluck())
        }
        "headlight_off" => {
            synth.sweep(Waveform::Sine, 600.0, 300.0, 0.12, Envelope::pluck())
        }

        // === ZONE AMBIENTS (longer, loopable) ===
        "ambient_sunlit" => {
            // Bright, active ocean with distant life
            let sustain = Envelope::new(0.5, 1.0, 0.8, 1.0);
            let waves = synth.filtered_noise(3.0, Some(800.0), Some(100.0), sustain, 11);
            let shimmer = synth.tone(Waveform::Sine, 440.0, 3.0, Envelope::new(0.5, 0.5, 0.3, 0.5));
            let high = synth.filtered_noise(3.0, Some(2000.0), Some(1000.0), Envelope::new(1.0, 0.5, 0.2, 0.5), 22);
            mix(&[(&waves, 0.3), (&shimmer, 0.1), (&high, 0.15)])
        }
        "ambient_twilight" => {
            // Mysterious, deeper tone
            let sustain = Envelope::new(0.8, 1.5, 0.7, 1.0);
            let drone = synth.tone(Waveform::Sine, 110.0, 4.0, sustain);
            let mystery = synth.sweep(Waveform::Triangle, 220.0, 180.0, 4.0, Envelope::new(1.0, 1.0, 0.5, 1.0));
            let particles = synth.filtered_noise(4.0, Some(600.0), Some(200.0), sustain, 33);
            mix(&[(&drone, 0.25), (&mystery, 0.15), (&particles, 0.2)])
        }
        "ambient_midnight" => {
            // Deep, ominous pressure
            let sustain = Envelope::new(1.0, 1.5, 0.8, 1.0);
            let deep = synth.tone(Waveform::Sine, 55.0, 5.0, sustain);
            let pressure = synth.tone(Waveform::Triangle, 40.0, 5.0, sustain);
            let distant = synth.filtered_noise(5.0, Some(300.0), Some(50.0), sustain, 44);
            mix(&[(&deep, 0.3), (&pressure, 0.2), (&distant, 0.15)])
        }
        "ambient_vents" => {
            // Rumbling, hissing volcanic activity
            let sustain = Envelope::new(0.5, 1.0, 0.9, 0.5);
            let rumble = synth.filtered_noise(4.0, None, Some(100.0), sustain, 55);
            let hiss = synth.filtered_noise(4.0, Some(3000.0), Some(1500.0), sustain, 66);
            let bass = synth.tone(Waveform::Saw, 35.0, 4.0, sustain);
            mix(&[(&rumble, 0.35), (&hiss, 0.2), (&bass, 0.25)])
        }

        // === CREATURE SOUNDS ===
        "whale" => {
            let decay_env = Envelope::new(0.1, 0.8, 0.3, 0.6);
            let call1 = synth.sweep(Waveform::Sine, 80.0, 140.0, 1.5, decay_env);
            let call2 = synth.sweep(Waveform::Sine, 120.0, 60.0, 1.2, decay_env);
            let harmonic = synth.sweep(Waveform::Sine, 160.0, 200.0, 1.0, Envelope::new(0.2, 0.4, 0.2, 0.3));
            let mixed = mix(&[(&call1, 0.4), (&harmonic, 0.2)]);
            concat(&[&mixed, &call2])
        }
        "whale_echo" => {
            let decay_env = Envelope::new(0.2, 0.5, 0.0, 0.8);
            let echo = synth.sweep(Waveform::Sine, 90.0, 70.0, 1.5, decay_env);
            let filtered = synth.filtered_noise(1.5, Some(200.0), Some(50.0), decay_env, 88);
            mix(&[(&echo, 0.3), (&filtered, 0.2)])
        }
        "fish" => {
            synth.filtered_noise(0.35, Some(1200.0), Some(400.0), Envelope::pluck(), 42)
        }
        "jellyfish" => {
            let pulse1 = synth.tone(Waveform::Sine, 220.0, 0.5, Envelope::pluck());
            let pulse2 = synth.tone(Waveform::Sine, 330.0, 0.4, Envelope::pluck());
            let silence1 = silence(0.25, SAMPLE_RATE);
            let pulse3 = synth.tone(Waveform::Sine, 275.0, 0.45, Envelope::pluck());
            let part1 = mix(&[(&pulse1, 0.4), (&pulse2, 0.3)]);
            concat(&[&part1, &silence1, &pulse3])
        }
        "squid" => {
            // Jet propulsion whoosh
            synth.sweep(Waveform::Triangle, 300.0, 800.0, 0.3, Envelope::pluck())
        }
        "anglerfish_lure" => {
            // Eerie pulsing glow sound
            let glow1 = synth.tone(Waveform::Sine, 660.0, 0.2, Envelope::pluck());
            let glow2 = synth.tone(Waveform::Sine, 880.0, 0.15, Envelope::pluck());
            let pause = silence(0.3, SAMPLE_RATE);
            let glow3 = synth.tone(Waveform::Sine, 770.0, 0.25, Envelope::pluck());
            let part1 = mix(&[(&glow1, 0.3), (&glow2, 0.2)]);
            concat(&[&part1, &pause, &glow3])
        }
        "crab_click" => {
            // Sharp clicking
            let click1 = synth.tone(Waveform::Square, 2000.0, 0.02, Envelope::pluck());
            let click2 = synth.tone(Waveform::Square, 2200.0, 0.015, Envelope::pluck());
            let pause = silence(0.08, SAMPLE_RATE);
            concat(&[&click1, &pause, &click2, &pause, &click1])
        }
        "shrimp_snap" => {
            // Loud snapping pop
            let pop = synth.filtered_noise(0.03, Some(4000.0), None, Envelope::pluck(), 99);
            let crack = synth.tone(Waveform::Square, 3000.0, 0.02, Envelope::pluck());
            mix(&[(&pop, 0.6), (&crack, 0.4)])
        }
        "octopus_move" => {
            // Flowing movement
            synth.sweep(Waveform::Sine, 200.0, 400.0, 0.4, Envelope::pad())
        }
        "eel_hiss" => {
            // Threatening hiss
            synth.filtered_noise(0.5, Some(2500.0), Some(800.0), Envelope::new(0.05, 0.1, 0.6, 0.2), 77)
        }
        "isopod_scuttle" => {
            // Rapid leg movements
            let tick1 = synth.tone(Waveform::Square, 1500.0, 0.01, Envelope::pluck());
            let tick2 = synth.tone(Waveform::Square, 1800.0, 0.01, Envelope::pluck());
            let pause = silence(0.04, SAMPLE_RATE);
            concat(&[&tick1, &pause, &tick2, &pause, &tick1, &pause, &tick2, &pause, &tick1])
        }

        // === ENVIRONMENT ===
        "bubbles" => {
            synth.filtered_noise(0.7, Some(1400.0), Some(400.0), Envelope::pad(), 42)
        }
        "bubbles_small" => {
            synth.filtered_noise(0.3, Some(2000.0), Some(800.0), Envelope::pluck(), 43)
        }
        "vent" => {
            let sustain_env = Envelope::new(0.15, 0.3, 0.85, 0.4);
            let rumble = synth.filtered_noise(2.0, None, Some(150.0), sustain_env, 42);
            let base = synth.tone(Waveform::Saw, 35.0, 2.0, sustain_env);
            let mid = synth.tone(Waveform::Triangle, 70.0, 2.0, sustain_env);
            mix(&[(&rumble, 0.4), (&base, 0.25), (&mid, 0.15)])
        }
        "vent_hiss" => {
            synth.filtered_noise(0.8, Some(4000.0), Some(1500.0), Envelope::pad(), 56)
        }
        "cave" => {
            let drip = synth.tone(Waveform::Sine, 900.0, 0.025, Envelope::pluck());
            let echo1 = synth.tone(Waveform::Sine, 880.0, 0.018, Envelope::pluck());
            let echo2 = synth.tone(Waveform::Sine, 860.0, 0.012, Envelope::pluck());
            let s1 = silence(0.18, SAMPLE_RATE);
            let s2 = silence(0.14, SAMPLE_RATE);
            let s3 = silence(0.1, SAMPLE_RATE);
            let echo3 = synth.tone(Waveform::Sine, 840.0, 0.008, Envelope::pluck());
            concat(&[&drip, &s1, &echo1, &s2, &echo2, &s3, &echo3])
        }
        "current" => {
            // Flowing water sound
            synth.filtered_noise(1.5, Some(600.0), Some(100.0), Envelope::pad(), 67)
        }
        "sediment" => {
            // Disturbed seafloor
            synth.filtered_noise(0.4, Some(800.0), Some(200.0), Envelope::pluck(), 78)
        }

        // === DISCOVERY & UI ===
        "artifact" => {
            let decay_env = Envelope::new(0.02, 0.4, 0.2, 0.3);
            let chime1 = synth.tone(Waveform::Sine, 659.0, 0.4, decay_env);
            let chime2 = synth.tone(Waveform::Sine, 880.0, 0.45, decay_env);
            let chime3 = synth.tone(Waveform::Sine, 1046.0, 0.5, decay_env);
            let chime4 = synth.tone(Waveform::Sine, 1318.0, 0.4, Envelope::new(0.1, 0.3, 0.1, 0.4));
            mix(&[(&chime1, 0.25), (&chime2, 0.25), (&chime3, 0.25), (&chime4, 0.2)])
        }
        "scan" => {
            let sweep1 = synth.sweep(Waveform::Sine, 500.0, 1500.0, 0.2, Envelope::pluck());
            let sweep2 = synth.sweep(Waveform::Sine, 600.0, 1200.0, 0.15, Envelope::pluck());
            mix(&[(&sweep1, 0.5), (&sweep2, 0.4)])
        }
        "log" => {
            let beep1 = synth.tone(Waveform::Square, 700.0, 0.07, Envelope::pluck());
            let beep2 = synth.tone(Waveform::Square, 880.0, 0.09, Envelope::pluck());
            concat(&[&beep1, &beep2])
        }
        "discovery" => {
            // Triumphant fanfare for new species
            let note1 = synth.tone(Waveform::Sine, 523.0, 0.2, Envelope::pluck()); // C
            let note2 = synth.tone(Waveform::Sine, 659.0, 0.2, Envelope::pluck()); // E
            let note3 = synth.tone(Waveform::Sine, 784.0, 0.3, Envelope::pluck()); // G
            let note4 = synth.tone(Waveform::Sine, 1046.0, 0.4, Envelope::new(0.02, 0.2, 0.3, 0.3)); // C
            let pause = silence(0.05, SAMPLE_RATE);
            let chord = mix(&[(&note1, 0.3), (&note2, 0.3), (&note3, 0.3)]);
            concat(&[&chord, &pause, &note4])
        }
        "zone_enter" => {
            // Transition chime
            let low = synth.tone(Waveform::Sine, 220.0, 0.3, Envelope::pad());
            let high = synth.tone(Waveform::Sine, 440.0, 0.3, Envelope::pad());
            mix(&[(&low, 0.4), (&high, 0.35)])
        }
        "depth_milestone" => {
            // Achievement-like sound
            let ding = synth.tone(Waveform::Sine, 1000.0, 0.15, Envelope::pluck());
            let pause = silence(0.08, SAMPLE_RATE);
            let dong = synth.tone(Waveform::Sine, 800.0, 0.2, Envelope::pluck());
            concat(&[&ding, &pause, &dong])
        }

        // === ENCOUNTERS ===
        "encounter_start" => {
            // Dramatic beginning
            let drone = synth.tone(Waveform::Sine, 80.0, 1.0, Envelope::new(0.3, 0.4, 0.5, 0.3));
            let rise = synth.sweep(Waveform::Triangle, 100.0, 300.0, 1.0, Envelope::pad());
            mix(&[(&drone, 0.4), (&rise, 0.35)])
        }
        "encounter_end" => {
            // Fading resolution
            synth.sweep(Waveform::Sine, 300.0, 100.0, 1.2, Envelope::new(0.1, 0.2, 0.5, 0.6))
        }
        "danger_near" => {
            // Warning pulse
            let pulse1 = synth.tone(Waveform::Saw, 150.0, 0.15, Envelope::pluck());
            let pause = silence(0.1, SAMPLE_RATE);
            let pulse2 = synth.tone(Waveform::Saw, 180.0, 0.15, Envelope::pluck());
            concat(&[&pulse1, &pause, &pulse2, &pause, &pulse1])
        }

        _ => panic!("Unknown sound ID: {}", id),
    }
}
