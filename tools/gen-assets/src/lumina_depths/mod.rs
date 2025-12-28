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

pub use crate::mesh_helpers::write_mesh;

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
    flora::generate_kelp_forest(&meshes_dir);
    flora::generate_anemone(&meshes_dir);
    flora::generate_sea_grass(&meshes_dir);

    // Terrain
    println!("\n  --- Terrain ---");
    terrain::generate_rock_boulder(&meshes_dir);
    terrain::generate_rock_pillar(&meshes_dir);
    terrain::generate_rock_scatter(&meshes_dir);
    terrain::generate_vent_chimney(&meshes_dir);
    terrain::generate_seafloor_patch(&meshes_dir);
    terrain::generate_cave_entrance(&meshes_dir);

    // Effects
    println!("\n  --- Effects ---");
    terrain::generate_bubble_cluster(&meshes_dir);

    // Textures (all categories via consolidated generator)
    println!("\n  --- Textures ---");
    textures::generate_all(&textures_dir);

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

/// Apply underwater muffling effect - water absorbs high frequencies
fn underwater_filter(samples: &mut [f32], depth_factor: f32) {
    // Low-pass effect simulating underwater sound transmission
    // Deeper = more muffling (lower cutoff)
    let cutoff = 3000.0 - (depth_factor * 2000.0); // 3000Hz at surface, 1000Hz at deepest
    low_pass(samples, cutoff.max(500.0), SAMPLE_RATE);
}

/// Add underwater reverb/echo character
fn add_underwater_reverb(samples: &[f32], delay_ms: f32, decay: f32) -> Vec<f32> {
    let delay_samples = ((delay_ms / 1000.0) * SAMPLE_RATE as f32) as usize;
    let mut result = samples.to_vec();
    result.resize(result.len() + delay_samples * 3, 0.0);

    // Add multiple echo taps for underwater diffusion
    for tap in 1..=3 {
        let tap_delay = delay_samples * tap;
        let tap_decay = decay.powi(tap as i32);
        for (i, &s) in samples.iter().enumerate() {
            if i + tap_delay < result.len() {
                result[i + tap_delay] += s * tap_decay;
            }
        }
    }
    result
}

fn generate_lumina_sound(synth: &Synth, id: &str) -> Vec<f32> {
    let mut result = match id {
        // === SUBMERSIBLE ===
        "sonar" => {
            // Rich sonar ping with harmonics and underwater propagation
            let ping_env = Envelope::new(0.001, 0.03, 0.0, 0.08);

            // Main ping with harmonic overtones
            let mut ping1 = synth.tone(Waveform::Sine, 1200.0, 0.12, ping_env);
            let ping2 = synth.tone(Waveform::Sine, 2400.0, 0.1, Envelope::new(0.001, 0.02, 0.0, 0.05));
            let ping3 = synth.tone(Waveform::Sine, 3600.0, 0.08, Envelope::new(0.001, 0.015, 0.0, 0.03));

            // Mix harmonics for richer tone
            let mixed_ping = mix(&[(&ping1, 0.5), (&ping2, 0.25), (&ping3, 0.15)]);

            // Create echoes with decreasing intensity and pitch shift (Doppler-like)
            let echo1_env = Envelope::new(0.01, 0.04, 0.0, 0.12);
            let echo1 = synth.tone(Waveform::Sine, 1180.0, 0.1, echo1_env);
            let echo1_h = synth.tone(Waveform::Sine, 2360.0, 0.08, echo1_env);
            let echo1_mix = mix(&[(&echo1, 0.4), (&echo1_h, 0.15)]);

            let echo2_env = Envelope::new(0.02, 0.05, 0.0, 0.15);
            let echo2 = synth.tone(Waveform::Sine, 1160.0, 0.08, echo2_env);

            let echo3_env = Envelope::new(0.03, 0.06, 0.0, 0.18);
            let echo3 = synth.tone(Waveform::Sine, 1140.0, 0.06, echo3_env);

            // Arrange with realistic underwater delays
            let s1 = silence(0.18, SAMPLE_RATE);
            let s2 = silence(0.22, SAMPLE_RATE);
            let s3 = silence(0.28, SAMPLE_RATE);

            let mut combined = concat(&[&mixed_ping, &s1, &echo1_mix, &s2, &echo2, &s3, &echo3]);
            underwater_filter(&mut combined, 0.2);
            combined
        }

        "propeller" => {
            // Rich propeller hum with mechanical character and water turbulence
            let sustain_env = Envelope::new(0.02, 0.15, 0.85, 0.25);

            // Fundamental motor hum
            let base = synth.tone(Waveform::Triangle, 55.0, 1.0, sustain_env);
            // First harmonic
            let harm1 = synth.tone(Waveform::Sine, 110.0, 1.0, sustain_env);
            // Second harmonic with slight detune for beating
            let harm2a = synth.tone(Waveform::Sine, 165.0, 1.0, sustain_env);
            let harm2b = synth.tone(Waveform::Sine, 167.0, 1.0, sustain_env);
            // Higher mechanical whine
            let whine = synth.tone(Waveform::Saw, 220.0, 1.0, Envelope::new(0.05, 0.2, 0.6, 0.2));

            // Water turbulence noise (low-passed)
            let mut turbulence = synth.filtered_noise(1.0, Some(800.0), Some(100.0), sustain_env, 42);

            // Blade passage frequency modulation
            let mut blade_mod = synth.tone(Waveform::Sine, 12.0, 1.0, sustain_env);
            for s in blade_mod.iter_mut() {
                *s = (*s * 0.3 + 0.7).clamp(0.4, 1.0);
            }

            let mut mixed = mix(&[
                (&base, 0.35),
                (&harm1, 0.25),
                (&harm2a, 0.1),
                (&harm2b, 0.1),
                (&whine, 0.08),
                (&turbulence, 0.15),
            ]);

            // Apply blade modulation
            for (i, s) in mixed.iter_mut().enumerate() {
                if i < blade_mod.len() {
                    *s *= blade_mod[i];
                }
            }

            underwater_filter(&mut mixed, 0.3);
            normalize_to(&mut mixed, 0.75);
            mixed
        }

        "surface" => {
            // Breaking through to surface - dramatic water rushing
            let rush_env = Envelope::new(0.05, 0.2, 0.3, 0.3);

            // Water rush (broadband noise filtered)
            let mut rush_low = synth.filtered_noise(0.6, Some(600.0), Some(80.0), rush_env, 77);
            let mut rush_mid = synth.filtered_noise(0.6, Some(2000.0), Some(400.0), rush_env, 78);
            let mut rush_high = synth.filtered_noise(0.5, Some(6000.0), Some(1500.0),
                Envelope::new(0.1, 0.15, 0.0, 0.2), 79);

            // Ascending pitch sweep (pressure release)
            let sweep = synth.sweep(Waveform::Sine, 150.0, 400.0, 0.5,
                Envelope::new(0.05, 0.25, 0.2, 0.2));

            // Bubbles and splash
            let bubbles = generate_rich_bubbles(synth, 0.6, 80);

            // Air burst at surface
            let mut air = synth.filtered_noise(0.15, Some(4000.0), Some(800.0),
                Envelope::new(0.0, 0.08, 0.0, 0.08), 81);
            air.resize((0.6 * SAMPLE_RATE as f32) as usize, 0.0);

            let mut mixed = mix(&[
                (&rush_low, 0.3),
                (&rush_mid, 0.25),
                (&rush_high, 0.15),
                (&sweep, 0.2),
                (&bubbles, 0.25),
                (&air, 0.2),
            ]);

            normalize_to(&mut mixed, 0.85);
            mixed
        }

        "hull_creak" => {
            // Deep pressure stress on metal hull - eerie and ominous
            let creak_env = Envelope::new(0.01, 0.12, 0.3, 0.15);

            // Low metallic groan
            let mut groan1 = synth.sweep(Waveform::Saw, 65.0, 45.0, 0.25, creak_env);
            let mut groan2 = synth.sweep(Waveform::Square, 68.0, 48.0, 0.22, creak_env);
            low_pass(&mut groan1, 400.0, SAMPLE_RATE);
            low_pass(&mut groan2, 350.0, SAMPLE_RATE);

            // Higher metallic stress
            let stress1 = synth.sweep(Waveform::Saw, 180.0, 120.0, 0.18,
                Envelope::new(0.02, 0.08, 0.2, 0.1));
            let stress2 = synth.sweep(Waveform::Triangle, 220.0, 160.0, 0.15,
                Envelope::new(0.03, 0.06, 0.15, 0.08));

            // Clicking/popping stress
            let mut clicks = Vec::new();
            for i in 0..4 {
                let mut click = synth.tone(Waveform::Square, 800.0 + (i as f32 * 100.0),
                    0.008, Envelope::new(0.0, 0.005, 0.0, 0.003));
                click.resize((0.5 * SAMPLE_RATE as f32) as usize, 0.0);
                // Offset each click
                let offset = (i as f32 * 0.08 * SAMPLE_RATE as f32) as usize;
                let mut padded = vec![0.0; offset];
                padded.extend(click);
                padded.resize((0.5 * SAMPLE_RATE as f32) as usize, 0.0);
                clicks.push(padded);
            }

            let s = silence(0.12, SAMPLE_RATE);
            let groan_part = mix(&[(&groan1, 0.4), (&groan2, 0.3)]);
            let stress_part = mix(&[(&stress1, 0.25), (&stress2, 0.2)]);

            let click_refs: Vec<(&[f32], f32)> = clicks.iter()
                .map(|c| (c.as_slice(), 0.15))
                .collect();
            let clicks_mixed = mix(&click_refs);

            let mut combined = concat(&[&groan_part, &s, &stress_part]);
            for (i, &c) in clicks_mixed.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += c;
                }
            }

            underwater_filter(&mut combined, 0.5);
            normalize_to(&mut combined, 0.8);
            combined
        }

        "pressure_warning" => {
            // Urgent alarm with underwater acoustic character
            let alarm_env = Envelope::new(0.002, 0.04, 0.5, 0.05);

            // Main alarm tone with harmonics
            let mut beep1 = synth.tone(Waveform::Square, 880.0, 0.12, alarm_env);
            let beep1_harm = synth.tone(Waveform::Sine, 1760.0, 0.1, alarm_env);
            low_pass(&mut beep1, 2500.0, SAMPLE_RATE);
            let beep_mixed = mix(&[(&beep1, 0.6), (&beep1_harm, 0.2)]);

            // Lower warning undertone
            let warning_low = synth.tone(Waveform::Sine, 220.0, 0.12, alarm_env);

            let beep_with_low = mix(&[(&beep_mixed, 0.7), (&warning_low, 0.25)]);

            let pause = silence(0.08, SAMPLE_RATE);
            let short_pause = silence(0.04, SAMPLE_RATE);

            let mut combined = concat(&[
                &beep_with_low, &pause,
                &beep_with_low, &pause,
                &beep_with_low, &short_pause,
                &beep_with_low, &short_pause,
                &beep_with_low,
            ]);

            underwater_filter(&mut combined, 0.3);
            combined
        }

        "headlight_on" => {
            // Electronic power-up with underwater acoustic feel
            let power_env = Envelope::new(0.005, 0.08, 0.3, 0.12);

            // Rising power tone
            let sweep = synth.sweep(Waveform::Sine, 350.0, 900.0, 0.2, power_env);
            let sweep_harm = synth.sweep(Waveform::Sine, 700.0, 1800.0, 0.18,
                Envelope::new(0.01, 0.06, 0.2, 0.1));

            // Capacitor charge whine
            let mut whine = synth.sweep(Waveform::Triangle, 1200.0, 2000.0, 0.15,
                Envelope::new(0.02, 0.1, 0.0, 0.05));
            low_pass(&mut whine, 3000.0, SAMPLE_RATE);

            // Click at activation
            let mut click = synth.tone(Waveform::Square, 1500.0, 0.01, Envelope::new(0.0, 0.005, 0.0, 0.005));
            click.resize((0.2 * SAMPLE_RATE as f32) as usize, 0.0);

            let mut mixed = mix(&[(&sweep, 0.4), (&sweep_harm, 0.2), (&whine, 0.2), (&click, 0.25)]);
            underwater_filter(&mut mixed, 0.2);
            mixed
        }

        "headlight_off" => {
            // Power-down with dissipation
            let power_env = Envelope::new(0.005, 0.05, 0.0, 0.15);

            // Descending power tone
            let sweep = synth.sweep(Waveform::Sine, 700.0, 250.0, 0.2, power_env);
            let sweep_harm = synth.sweep(Waveform::Sine, 1400.0, 500.0, 0.18,
                Envelope::new(0.01, 0.04, 0.0, 0.12));

            // Electrical fade
            let mut fade = synth.sweep(Waveform::Triangle, 1800.0, 400.0, 0.2,
                Envelope::new(0.0, 0.02, 0.0, 0.15));
            low_pass(&mut fade, 2500.0, SAMPLE_RATE);

            let mut mixed = mix(&[(&sweep, 0.4), (&sweep_harm, 0.2), (&fade, 0.2)]);
            underwater_filter(&mut mixed, 0.2);
            mixed
        }

        // === ZONE AMBIENTS (longer, loopable) ===
        "ambient_sunlit" => {
            // Bright, vibrant shallow waters with life
            let duration = 4.0;
            let sustain = Envelope::new(0.8, 1.0, 0.9, 1.0);

            // Gentle water movement (multiple frequency bands)
            let waves_low = synth.filtered_noise(duration, Some(400.0), Some(60.0), sustain, 11);
            let waves_mid = synth.filtered_noise(duration, Some(1200.0), Some(300.0), sustain, 12);
            let waves_high = synth.filtered_noise(duration, Some(3000.0), Some(800.0),
                Envelope::new(0.5, 0.8, 0.6, 0.8), 13);

            // Light shimmer (sunlight through water)
            let shimmer1 = synth.tone(Waveform::Sine, 880.0, duration,
                Envelope::new(1.0, 1.0, 0.2, 0.5));
            let shimmer2 = synth.tone(Waveform::Sine, 1320.0, duration,
                Envelope::new(1.2, 0.8, 0.15, 0.6));

            // Distant fish activity
            let fish_activity = synth.filtered_noise(duration, Some(2500.0), Some(600.0),
                Envelope::new(0.5, 1.5, 0.3, 1.0), 14);

            // Subtle bubbles from marine life
            let bubbles = generate_ambient_bubbles(synth, duration, 15);

            let mut mixed = mix(&[
                (&waves_low, 0.25),
                (&waves_mid, 0.2),
                (&waves_high, 0.1),
                (&shimmer1, 0.08),
                (&shimmer2, 0.05),
                (&fish_activity, 0.1),
                (&bubbles, 0.15),
            ]);

            underwater_filter(&mut mixed, 0.1);
            normalize_to(&mut mixed, 0.65);
            mixed
        }

        "ambient_twilight" => {
            // Mysterious, darker waters - less activity, more pressure
            let duration = 5.0;
            let sustain = Envelope::new(1.0, 1.5, 0.85, 1.0);

            // Deep drone
            let drone1 = synth.tone(Waveform::Sine, 82.0, duration, sustain);
            let drone2 = synth.tone(Waveform::Sine, 84.0, duration, sustain); // Slight detune for beating
            let drone3 = synth.tone(Waveform::Triangle, 41.0, duration,
                Envelope::new(1.5, 1.5, 0.7, 1.0));

            // Mysterious tonal swells
            let swell = synth.sweep(Waveform::Sine, 165.0, 140.0, duration,
                Envelope::new(1.5, 1.5, 0.4, 1.5));

            // Distant pressure sounds
            let pressure = synth.filtered_noise(duration, Some(400.0), Some(50.0), sustain, 33);

            // Sparse particle movement
            let particles = synth.filtered_noise(duration, Some(1000.0), Some(200.0),
                Envelope::new(0.8, 1.0, 0.4, 1.2), 34);

            // Occasional distant creature
            let distant = synth.filtered_noise(duration, Some(600.0), Some(150.0),
                Envelope::new(2.0, 1.0, 0.2, 1.5), 35);

            let mut mixed = mix(&[
                (&drone1, 0.2),
                (&drone2, 0.2),
                (&drone3, 0.15),
                (&swell, 0.1),
                (&pressure, 0.2),
                (&particles, 0.1),
                (&distant, 0.08),
            ]);

            underwater_filter(&mut mixed, 0.4);
            normalize_to(&mut mixed, 0.6);
            mixed
        }

        "ambient_midnight" => {
            // Deep, oppressive darkness - crushing pressure, sparse life
            let duration = 6.0;
            let sustain = Envelope::new(1.5, 2.0, 0.9, 1.5);

            // Ultra-low pressure drone
            let deep1 = synth.tone(Waveform::Sine, 35.0, duration, sustain);
            let deep2 = synth.tone(Waveform::Sine, 37.0, duration, sustain);
            let deep3 = synth.tone(Waveform::Triangle, 25.0, duration,
                Envelope::new(2.0, 2.0, 0.8, 1.5));

            // Crushing pressure ambience
            let pressure = synth.filtered_noise(duration, Some(200.0), Some(20.0), sustain, 44);

            // Distant metallic groans (geological)
            let groan = synth.sweep(Waveform::Saw, 60.0, 40.0, duration,
                Envelope::new(2.0, 2.0, 0.3, 2.0));

            // Sparse bioluminescent pulses (subtle high tones)
            let biolum = synth.tone(Waveform::Sine, 1200.0, duration,
                Envelope::new(3.0, 1.0, 0.1, 1.5));

            let mut mixed = mix(&[
                (&deep1, 0.25),
                (&deep2, 0.25),
                (&deep3, 0.2),
                (&pressure, 0.25),
                (&groan, 0.1),
                (&biolum, 0.03),
            ]);

            underwater_filter(&mut mixed, 0.7);
            normalize_to(&mut mixed, 0.55);
            mixed
        }

        "ambient_vents" => {
            // Hydrothermal chaos - rumbling, hissing, mineral-rich water
            let duration = 5.0;
            let sustain = Envelope::new(0.5, 1.0, 0.95, 0.8);

            // Deep volcanic rumble
            let mut rumble = synth.filtered_noise(duration, Some(80.0), Some(15.0), sustain, 55);

            // Sub-bass throbbing
            let throb1 = synth.tone(Waveform::Sine, 28.0, duration, sustain);
            let throb2 = synth.tone(Waveform::Saw, 30.0, duration,
                Envelope::new(0.8, 1.0, 0.85, 0.8));

            // Steam hiss (high-frequency noise)
            let hiss = synth.filtered_noise(duration, Some(6000.0), Some(2000.0),
                Envelope::new(0.3, 0.5, 0.7, 0.5), 56);

            // Mineral crackling (mid-range noise bursts)
            let crackle = synth.filtered_noise(duration, Some(2500.0), Some(800.0),
                Envelope::new(0.2, 0.3, 0.6, 0.4), 57);

            // Bubbling from vents
            let bubbles = generate_vent_bubbles(synth, duration, 58);

            // Occasional pressure release
            let release = synth.sweep(Waveform::Triangle, 100.0, 300.0, duration,
                Envelope::new(1.5, 1.0, 0.2, 1.5));

            let mut mixed = mix(&[
                (&rumble, 0.35),
                (&throb1, 0.2),
                (&throb2, 0.15),
                (&hiss, 0.15),
                (&crackle, 0.12),
                (&bubbles, 0.18),
                (&release, 0.08),
            ]);

            underwater_filter(&mut mixed, 0.5);
            normalize_to(&mut mixed, 0.7);
            mixed
        }

        // === CREATURE SOUNDS ===
        "whale" => {
            // Majestic whale song with harmonics and underwater propagation
            let call_env = Envelope::new(0.15, 1.0, 0.4, 0.8);

            // Main call with rich harmonics (whale songs have many overtones)
            let fundamental = synth.sweep(Waveform::Sine, 70.0, 120.0, 2.0, call_env);
            let harm1 = synth.sweep(Waveform::Sine, 140.0, 240.0, 1.8,
                Envelope::new(0.2, 0.8, 0.3, 0.7));
            let harm2 = synth.sweep(Waveform::Sine, 210.0, 360.0, 1.6,
                Envelope::new(0.25, 0.6, 0.25, 0.6));
            let harm3 = synth.sweep(Waveform::Sine, 280.0, 480.0, 1.4,
                Envelope::new(0.3, 0.5, 0.2, 0.5));

            // Second phrase (descending)
            let phrase2_env = Envelope::new(0.2, 0.8, 0.35, 0.7);
            let phrase2_fund = synth.sweep(Waveform::Sine, 110.0, 55.0, 1.8, phrase2_env);
            let phrase2_harm = synth.sweep(Waveform::Sine, 220.0, 110.0, 1.6,
                Envelope::new(0.25, 0.7, 0.3, 0.6));

            // Modulation for natural wavering
            let phrase1 = mix(&[
                (&fundamental, 0.4),
                (&harm1, 0.25),
                (&harm2, 0.15),
                (&harm3, 0.1),
            ]);

            let phrase2 = mix(&[(&phrase2_fund, 0.4), (&phrase2_harm, 0.25)]);

            let pause = silence(0.3, SAMPLE_RATE);
            let mut combined = concat(&[&phrase1, &pause, &phrase2]);

            // Add underwater distance/reverb effect
            combined = add_underwater_reverb(&combined, 150.0, 0.4);
            underwater_filter(&mut combined, 0.3);
            normalize_to(&mut combined, 0.8);
            combined
        }

        "whale_echo" => {
            // Distant whale call with heavy underwater attenuation
            let echo_env = Envelope::new(0.3, 0.6, 0.0, 1.2);

            let echo = synth.sweep(Waveform::Sine, 75.0, 60.0, 2.0, echo_env);
            let echo_harm = synth.sweep(Waveform::Sine, 150.0, 120.0, 1.8,
                Envelope::new(0.35, 0.5, 0.0, 1.0));

            // Heavy filtering for distance
            let mut mixed = mix(&[(&echo, 0.35), (&echo_harm, 0.15)]);
            mixed = add_underwater_reverb(&mixed, 250.0, 0.3);
            underwater_filter(&mut mixed, 0.7);

            // Add depth rumble
            let rumble = synth.filtered_noise(2.5, Some(120.0), Some(30.0),
                Envelope::new(0.4, 0.6, 0.0, 1.0), 88);

            let mut final_mix = mix(&[(&mixed, 0.6), (&rumble, 0.3)]);
            normalize_to(&mut final_mix, 0.5);
            final_mix
        }

        "fish" => {
            // School of fish movement - rustling, darting sounds
            let duration = 0.45;

            // Quick darting movements (filtered noise bursts)
            let mut dart1 = synth.filtered_noise(0.08, Some(1800.0), Some(600.0),
                Envelope::new(0.0, 0.02, 0.0, 0.06), 42);
            let mut dart2 = synth.filtered_noise(0.06, Some(2200.0), Some(800.0),
                Envelope::new(0.0, 0.015, 0.0, 0.05), 43);
            let mut dart3 = synth.filtered_noise(0.07, Some(1600.0), Some(500.0),
                Envelope::new(0.0, 0.02, 0.0, 0.055), 44);

            // Tail flicks
            let flick = synth.sweep(Waveform::Sine, 400.0, 800.0, 0.04,
                Envelope::new(0.0, 0.015, 0.0, 0.025));

            // Arrange with spacing
            dart1.resize((duration * SAMPLE_RATE as f32) as usize, 0.0);
            dart2.resize((duration * SAMPLE_RATE as f32) as usize, 0.0);
            dart3.resize((duration * SAMPLE_RATE as f32) as usize, 0.0);

            // Offset dart2 and dart3
            let offset2 = (0.12 * SAMPLE_RATE as f32) as usize;
            let offset3 = (0.28 * SAMPLE_RATE as f32) as usize;
            let mut result = dart1.clone();
            for (i, &s) in dart2.iter().enumerate() {
                if i + offset2 < result.len() {
                    result[i + offset2] += s;
                }
            }
            for (i, &s) in dart3.iter().enumerate() {
                if i + offset3 < result.len() {
                    result[i + offset3] += s;
                }
            }

            underwater_filter(&mut result, 0.2);
            normalize_to(&mut result, 0.7);
            result
        }

        "jellyfish" => {
            // Ethereal, pulsing bioluminescent creature
            let pulse_env = Envelope::new(0.05, 0.25, 0.3, 0.35);

            // Soft tonal pulses with harmonics
            let pulse1 = synth.tone(Waveform::Sine, 180.0, 0.7, pulse_env);
            let pulse1_harm = synth.tone(Waveform::Sine, 360.0, 0.6,
                Envelope::new(0.08, 0.2, 0.2, 0.3));

            // Second pulse (higher)
            let mut pulse2 = synth.tone(Waveform::Sine, 270.0, 0.5,
                Envelope::new(0.04, 0.2, 0.25, 0.3));
            let pulse2_harm = synth.tone(Waveform::Sine, 540.0, 0.45,
                Envelope::new(0.06, 0.18, 0.2, 0.25));

            // Subtle shimmer (bioluminescence)
            let shimmer = synth.filtered_noise(1.4, Some(4000.0), Some(1500.0),
                Envelope::new(0.2, 0.4, 0.1, 0.4), 50);

            let p1_mix = mix(&[(&pulse1, 0.35), (&pulse1_harm, 0.15)]);
            let p2_mix = mix(&[(&pulse2, 0.3), (&pulse2_harm, 0.12)]);

            let pause = silence(0.35, SAMPLE_RATE);
            let mut combined = concat(&[&p1_mix, &pause, &p2_mix]);

            // Add shimmer throughout
            for (i, &s) in shimmer.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += s * 0.1;
                }
            }

            underwater_filter(&mut combined, 0.3);
            normalize_to(&mut combined, 0.65);
            combined
        }

        "squid" => {
            // Jet propulsion - water rush with movement
            let jet_env = Envelope::new(0.01, 0.1, 0.2, 0.15);

            // Water jet rush
            let mut jet = synth.filtered_noise(0.4, Some(2500.0), Some(300.0), jet_env, 60);

            // Propulsion whoosh (pitch rise)
            let whoosh = synth.sweep(Waveform::Triangle, 200.0, 600.0, 0.35, jet_env);

            // Body movement tone
            let body = synth.sweep(Waveform::Sine, 150.0, 350.0, 0.3,
                Envelope::new(0.02, 0.08, 0.15, 0.12));

            // Tentacle flutter
            let flutter = synth.filtered_noise(0.25, Some(1200.0), Some(400.0),
                Envelope::new(0.05, 0.15, 0.0, 0.1), 61);

            let mut mixed = mix(&[
                (&jet, 0.35),
                (&whoosh, 0.3),
                (&body, 0.2),
                (&flutter, 0.15),
            ]);

            underwater_filter(&mut mixed, 0.25);
            normalize_to(&mut mixed, 0.75);
            mixed
        }

        "anglerfish_lure" => {
            // Eerie, hypnotic bioluminescent pulsing
            let glow_env = Envelope::new(0.08, 0.15, 0.4, 0.25);

            // Mysterious tonal pulses
            let glow1 = synth.tone(Waveform::Sine, 550.0, 0.35, glow_env);
            let glow1_h1 = synth.tone(Waveform::Sine, 825.0, 0.3, glow_env);
            let glow1_h2 = synth.tone(Waveform::Sine, 1100.0, 0.25,
                Envelope::new(0.1, 0.12, 0.3, 0.2));

            // Detuned for eerie quality
            let glow2 = synth.tone(Waveform::Sine, 660.0, 0.4,
                Envelope::new(0.06, 0.18, 0.35, 0.3));
            let glow2_detune = synth.tone(Waveform::Sine, 665.0, 0.38,
                Envelope::new(0.07, 0.17, 0.3, 0.28));

            // Hypnotic shimmer
            let shimmer = synth.tone(Waveform::Triangle, 1650.0, 0.5,
                Envelope::new(0.15, 0.25, 0.15, 0.2));

            let p1 = mix(&[(&glow1, 0.3), (&glow1_h1, 0.15), (&glow1_h2, 0.08)]);
            let p2 = mix(&[(&glow2, 0.25), (&glow2_detune, 0.2), (&shimmer, 0.1)]);

            let pause = silence(0.4, SAMPLE_RATE);
            let mut combined = concat(&[&p1, &pause, &p2]);

            underwater_filter(&mut combined, 0.35);
            normalize_to(&mut combined, 0.6);
            combined
        }

        "crab_click" => {
            // Sharp crustacean clicking with resonance
            let click_env = Envelope::new(0.0, 0.003, 0.0, 0.015);

            // Main click with body resonance
            let mut click1 = synth.tone(Waveform::Square, 1800.0, 0.025, click_env);
            let resonance1 = synth.tone(Waveform::Sine, 600.0, 0.04,
                Envelope::new(0.0, 0.005, 0.2, 0.03));

            let mut click2 = synth.tone(Waveform::Square, 2100.0, 0.02, click_env);
            let resonance2 = synth.tone(Waveform::Sine, 700.0, 0.035,
                Envelope::new(0.0, 0.004, 0.15, 0.025));

            let mut click3 = synth.tone(Waveform::Square, 1950.0, 0.022, click_env);
            let resonance3 = synth.tone(Waveform::Sine, 650.0, 0.038,
                Envelope::new(0.0, 0.005, 0.18, 0.028));

            low_pass(&mut click1, 4000.0, SAMPLE_RATE);
            low_pass(&mut click2, 4500.0, SAMPLE_RATE);
            low_pass(&mut click3, 4200.0, SAMPLE_RATE);

            let c1 = mix(&[(&click1, 0.5), (&resonance1, 0.3)]);
            let c2 = mix(&[(&click2, 0.45), (&resonance2, 0.25)]);
            let c3 = mix(&[(&click3, 0.48), (&resonance3, 0.28)]);

            let pause = silence(0.06, SAMPLE_RATE);
            let mut combined = concat(&[&c1, &pause, &c2, &pause, &c3]);

            underwater_filter(&mut combined, 0.2);
            combined
        }

        "shrimp_snap" => {
            // Pistol shrimp-style cavitation snap - extremely loud underwater
            // Create sharp transient with bubble collapse

            let snap_env = Envelope::new(0.0, 0.001, 0.0, 0.008);

            // Initial snap (broadband impulse)
            let mut snap = synth.filtered_noise(0.015, Some(8000.0), None, snap_env, 99);

            // Cavitation bubble collapse (low frequency thump)
            let thump = synth.tone(Waveform::Sine, 120.0, 0.03,
                Envelope::new(0.0, 0.003, 0.0, 0.025));

            // High-frequency crack
            let mut crack = synth.tone(Waveform::Square, 4000.0, 0.008, snap_env);
            low_pass(&mut crack, 6000.0, SAMPLE_RATE);

            // Bubble aftermath
            let bubbles = synth.filtered_noise(0.08, Some(3000.0), Some(500.0),
                Envelope::new(0.005, 0.02, 0.0, 0.05), 100);

            let mut combined = mix(&[
                (&snap, 0.5),
                (&thump, 0.4),
                (&crack, 0.3),
                (&bubbles, 0.2),
            ]);

            underwater_filter(&mut combined, 0.15);
            normalize_to(&mut combined, 0.9);
            combined
        }

        "octopus_move" => {
            // Flowing, undulating movement with suckers
            let move_env = Envelope::new(0.05, 0.2, 0.4, 0.25);

            // Flowing body movement
            let flow1 = synth.sweep(Waveform::Sine, 150.0, 280.0, 0.5, move_env);
            let flow2 = synth.sweep(Waveform::Sine, 180.0, 320.0, 0.45,
                Envelope::new(0.07, 0.18, 0.35, 0.22));

            // Tentacle undulation
            let undulate = synth.sweep(Waveform::Triangle, 200.0, 400.0, 0.4,
                Envelope::new(0.08, 0.15, 0.3, 0.2));

            // Sucker sounds (subtle popping)
            let mut suckers = Vec::new();
            for i in 0..5 {
                let mut pop = synth.tone(Waveform::Sine, 800.0 + (i as f32 * 50.0),
                    0.015, Envelope::new(0.0, 0.005, 0.0, 0.01));
                pop.resize((0.5 * SAMPLE_RATE as f32) as usize, 0.0);
                let offset = (i as f32 * 0.08 * SAMPLE_RATE as f32) as usize;
                let mut padded = vec![0.0; offset];
                padded.extend(pop);
                padded.resize((0.5 * SAMPLE_RATE as f32) as usize, 0.0);
                suckers.push(padded);
            }

            let sucker_refs: Vec<(&[f32], f32)> = suckers.iter()
                .map(|s| (s.as_slice(), 0.12))
                .collect();
            let suckers_mixed = mix(&sucker_refs);

            let mut combined = mix(&[
                (&flow1, 0.3),
                (&flow2, 0.25),
                (&undulate, 0.2),
                (&suckers_mixed, 0.15),
            ]);

            underwater_filter(&mut combined, 0.3);
            normalize_to(&mut combined, 0.7);
            combined
        }

        "eel_hiss" => {
            // Threatening gulper eel vocalization
            let hiss_env = Envelope::new(0.03, 0.15, 0.65, 0.25);

            // Low threatening growl
            let growl = synth.tone(Waveform::Saw, 80.0, 0.6, hiss_env);
            let growl_h = synth.tone(Waveform::Saw, 160.0, 0.55,
                Envelope::new(0.04, 0.12, 0.55, 0.22));

            // Hissing component
            let mut hiss = synth.filtered_noise(0.6, Some(3500.0), Some(1000.0), hiss_env, 77);

            // Modulated threat tone
            let threat = synth.sweep(Waveform::Triangle, 100.0, 60.0, 0.55,
                Envelope::new(0.05, 0.2, 0.5, 0.2));

            low_pass(&mut hiss, 3000.0, SAMPLE_RATE);

            let mut combined = mix(&[
                (&growl, 0.25),
                (&growl_h, 0.15),
                (&hiss, 0.35),
                (&threat, 0.2),
            ]);

            underwater_filter(&mut combined, 0.4);
            normalize_to(&mut combined, 0.75);
            combined
        }

        "isopod_scuttle" => {
            // Rapid armored leg movements on substrate
            let tick_env = Envelope::new(0.0, 0.002, 0.0, 0.008);

            // Build pattern of rapid ticks
            let mut ticks = Vec::new();
            let tick_times = [0.0, 0.025, 0.055, 0.08, 0.115, 0.135, 0.17, 0.2, 0.235];
            let tick_freqs = [1400.0, 1600.0, 1500.0, 1700.0, 1450.0, 1650.0, 1550.0, 1400.0, 1600.0];

            for (&time, &freq) in tick_times.iter().zip(tick_freqs.iter()) {
                let mut tick = synth.tone(Waveform::Square, freq, 0.012, tick_env);
                // Add shell resonance
                let resonance = synth.tone(Waveform::Sine, freq * 0.4, 0.02,
                    Envelope::new(0.0, 0.003, 0.1, 0.015));

                let tick_with_res = mix(&[(&tick, 0.4), (&resonance, 0.2)]);

                let offset = (time * SAMPLE_RATE as f32) as usize;
                let mut padded = vec![0.0; offset];
                padded.extend(tick_with_res);
                padded.resize((0.35 * SAMPLE_RATE as f32) as usize, 0.0);
                ticks.push(padded);
            }

            let tick_refs: Vec<(&[f32], f32)> = ticks.iter()
                .map(|t| (t.as_slice(), 1.0))
                .collect();
            let mut combined = mix(&tick_refs);

            // Add substrate scraping
            let scrape = synth.filtered_noise(0.3, Some(2000.0), Some(600.0),
                Envelope::new(0.02, 0.1, 0.2, 0.1), 90);

            for (i, &s) in scrape.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += s * 0.15;
                }
            }

            underwater_filter(&mut combined, 0.25);
            normalize_to(&mut combined, 0.7);
            combined
        }

        // === ENVIRONMENT ===
        "bubbles" => {
            generate_rich_bubbles(synth, 0.8, 42)
        }

        "bubbles_small" => {
            // Smaller, tighter bubble stream
            let duration = 0.4;
            let mut result = Vec::new();

            for i in 0..12 {
                let freq = 1200.0 + ((i % 5) as f32 * 150.0);
                let mut bubble = synth.tone(Waveform::Sine, freq, 0.025,
                    Envelope::new(0.0, 0.008, 0.2, 0.015));

                let offset = (i as f32 * 0.028 * SAMPLE_RATE as f32) as usize;

                if result.len() < offset + bubble.len() {
                    result.resize(offset + bubble.len(), 0.0);
                }
                for (j, &s) in bubble.iter().enumerate() {
                    result[offset + j] += s * 0.4;
                }
            }

            result.resize((duration * SAMPLE_RATE as f32) as usize, 0.0);

            // Add bubble noise
            let mut noise = synth.filtered_noise(duration, Some(3000.0), Some(1000.0),
                Envelope::new(0.02, 0.1, 0.3, 0.1), 43);

            let mut combined = mix(&[(&result, 0.6), (&noise, 0.3)]);
            underwater_filter(&mut combined, 0.2);
            normalize_to(&mut combined, 0.7);
            combined
        }

        "vent" => {
            // Powerful hydrothermal vent with rumble and mineral spray
            let duration = 2.5;
            let sustain_env = Envelope::new(0.2, 0.4, 0.9, 0.5);

            // Deep geological rumble
            let mut rumble = synth.filtered_noise(duration, Some(100.0), Some(15.0), sustain_env, 42);

            // Sub-bass throb
            let bass1 = synth.tone(Waveform::Sine, 28.0, duration, sustain_env);
            let bass2 = synth.tone(Waveform::Saw, 32.0, duration,
                Envelope::new(0.25, 0.4, 0.85, 0.45));

            // Mid-range turbulence
            let turbulence = synth.filtered_noise(duration, Some(600.0), Some(100.0),
                sustain_env, 43);

            // Mineral-rich water ejection
            let spray = synth.filtered_noise(duration, Some(2000.0), Some(400.0),
                Envelope::new(0.3, 0.5, 0.7, 0.4), 44);

            // Occasional pressure bursts
            let bursts = synth.sweep(Waveform::Triangle, 60.0, 150.0, duration,
                Envelope::new(0.5, 0.8, 0.3, 0.6));

            let mut combined = mix(&[
                (&rumble, 0.35),
                (&bass1, 0.2),
                (&bass2, 0.15),
                (&turbulence, 0.2),
                (&spray, 0.12),
                (&bursts, 0.1),
            ]);

            underwater_filter(&mut combined, 0.4);
            normalize_to(&mut combined, 0.75);
            combined
        }

        "vent_hiss" => {
            // Superheated water release - aggressive high-frequency content
            let duration = 1.0;
            let hiss_env = Envelope::new(0.05, 0.2, 0.75, 0.3);

            // Primary steam hiss
            let hiss1 = synth.filtered_noise(duration, Some(7000.0), Some(2500.0), hiss_env, 56);

            // Secondary layer
            let hiss2 = synth.filtered_noise(duration, Some(5000.0), Some(1500.0),
                Envelope::new(0.08, 0.25, 0.7, 0.25), 57);

            // Bubbling undertone
            let bubbles = generate_vent_bubbles(synth, duration, 58);

            // Low pressure base
            let base = synth.filtered_noise(duration, Some(400.0), Some(60.0),
                hiss_env, 59);

            let mut combined = mix(&[
                (&hiss1, 0.35),
                (&hiss2, 0.25),
                (&bubbles, 0.2),
                (&base, 0.2),
            ]);

            underwater_filter(&mut combined, 0.3);
            normalize_to(&mut combined, 0.7);
            combined
        }

        "cave" => {
            // Underwater cave drip with deep resonant echoes
            let drip_env = Envelope::new(0.0, 0.008, 0.0, 0.03);

            // Primary drip with cave resonance
            let drip = synth.tone(Waveform::Sine, 850.0, 0.05, drip_env);
            let drip_res = synth.tone(Waveform::Sine, 425.0, 0.08,
                Envelope::new(0.01, 0.02, 0.3, 0.05));
            let drip_mix = mix(&[(&drip, 0.5), (&drip_res, 0.3)]);

            // Multiple echoes with decreasing clarity
            let echo1 = synth.tone(Waveform::Sine, 835.0, 0.035,
                Envelope::new(0.01, 0.015, 0.0, 0.05));
            let echo2 = synth.tone(Waveform::Sine, 820.0, 0.028,
                Envelope::new(0.015, 0.02, 0.0, 0.06));
            let echo3 = synth.tone(Waveform::Sine, 805.0, 0.02,
                Envelope::new(0.02, 0.025, 0.0, 0.07));
            let echo4 = synth.tone(Waveform::Sine, 790.0, 0.015,
                Envelope::new(0.025, 0.03, 0.0, 0.08));

            // Delays
            let s1 = silence(0.22, SAMPLE_RATE);
            let s2 = silence(0.18, SAMPLE_RATE);
            let s3 = silence(0.15, SAMPLE_RATE);
            let s4 = silence(0.12, SAMPLE_RATE);

            // Cave ambient rumble
            let ambient = synth.filtered_noise(1.2, Some(200.0), Some(30.0),
                Envelope::new(0.3, 0.3, 0.2, 0.3), 70);

            let mut combined = concat(&[&drip_mix, &s1, &echo1, &s2, &echo2, &s3, &echo3, &s4, &echo4]);

            for (i, &s) in ambient.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += s * 0.2;
                }
            }

            underwater_filter(&mut combined, 0.4);
            normalize_to(&mut combined, 0.7);
            combined
        }

        "current" => {
            // Oceanic current - flowing, swirling water movement
            let duration = 2.0;
            let flow_env = Envelope::new(0.3, 0.5, 0.8, 0.5);

            // Multiple frequency bands for full-spectrum flow
            let flow_low = synth.filtered_noise(duration, Some(300.0), Some(40.0), flow_env, 67);
            let flow_mid = synth.filtered_noise(duration, Some(800.0), Some(200.0),
                Envelope::new(0.4, 0.4, 0.7, 0.4), 68);
            let flow_high = synth.filtered_noise(duration, Some(2000.0), Some(500.0),
                Envelope::new(0.5, 0.3, 0.5, 0.3), 69);

            // Swirling motion (modulated tone)
            let swirl = synth.sweep(Waveform::Sine, 100.0, 150.0, duration,
                Envelope::new(0.4, 0.6, 0.5, 0.5));

            let mut combined = mix(&[
                (&flow_low, 0.35),
                (&flow_mid, 0.25),
                (&flow_high, 0.15),
                (&swirl, 0.15),
            ]);

            underwater_filter(&mut combined, 0.35);
            normalize_to(&mut combined, 0.65);
            combined
        }

        "sediment" => {
            // Seafloor disturbance - particles and debris
            let dist_env = Envelope::new(0.02, 0.12, 0.3, 0.2);

            // Particle suspension
            let particles = synth.filtered_noise(0.5, Some(1200.0), Some(200.0), dist_env, 78);

            // Low thump from disturbance
            let thump = synth.tone(Waveform::Sine, 80.0, 0.15,
                Envelope::new(0.01, 0.04, 0.0, 0.1));

            // Swirling debris
            let swirl = synth.filtered_noise(0.45, Some(600.0), Some(100.0),
                Envelope::new(0.05, 0.15, 0.25, 0.15), 79);

            let mut combined = mix(&[
                (&particles, 0.35),
                (&thump, 0.3),
                (&swirl, 0.25),
            ]);

            underwater_filter(&mut combined, 0.35);
            normalize_to(&mut combined, 0.7);
            combined
        }

        // === DISCOVERY & UI ===
        "artifact" => {
            // Magical discovery chime with underwater shimmer
            let chime_env = Envelope::new(0.005, 0.15, 0.35, 0.4);

            // Ascending chord with harmonics
            let note1 = synth.tone(Waveform::Sine, 523.0, 0.6, chime_env); // C5
            let note1_h = synth.tone(Waveform::Sine, 1046.0, 0.5, chime_env);

            let note2 = synth.tone(Waveform::Sine, 659.0, 0.55, chime_env); // E5
            let note2_h = synth.tone(Waveform::Sine, 1318.0, 0.45, chime_env);

            let note3 = synth.tone(Waveform::Sine, 784.0, 0.5, chime_env); // G5
            let note3_h = synth.tone(Waveform::Sine, 1568.0, 0.4, chime_env);

            let note4 = synth.tone(Waveform::Sine, 1046.0, 0.6,
                Envelope::new(0.01, 0.2, 0.4, 0.5)); // C6

            // Shimmer effect
            let shimmer = synth.filtered_noise(0.8, Some(6000.0), Some(2000.0),
                Envelope::new(0.1, 0.3, 0.2, 0.3), 85);

            let chord = mix(&[
                (&note1, 0.2),
                (&note1_h, 0.1),
                (&note2, 0.2),
                (&note2_h, 0.1),
                (&note3, 0.2),
                (&note3_h, 0.1),
                (&note4, 0.25),
                (&shimmer, 0.12),
            ]);

            let mut result = add_underwater_reverb(&chord, 80.0, 0.3);
            underwater_filter(&mut result, 0.15);
            normalize_to(&mut result, 0.8);
            result
        }

        "scan" => {
            // Electronic scanning sweep with data processing
            let scan_env = Envelope::new(0.005, 0.08, 0.3, 0.1);

            // Primary scan sweep
            let sweep1 = synth.sweep(Waveform::Sine, 400.0, 1600.0, 0.25, scan_env);
            let sweep1_h = synth.sweep(Waveform::Sine, 800.0, 3200.0, 0.22,
                Envelope::new(0.01, 0.06, 0.2, 0.08));

            // Return sweep
            let sweep2 = synth.sweep(Waveform::Sine, 1400.0, 600.0, 0.18,
                Envelope::new(0.01, 0.06, 0.25, 0.08));

            // Data chirps
            let chirp1 = synth.tone(Waveform::Square, 1200.0, 0.03,
                Envelope::new(0.0, 0.01, 0.0, 0.02));
            let chirp2 = synth.tone(Waveform::Square, 1500.0, 0.025,
                Envelope::new(0.0, 0.008, 0.0, 0.018));

            let sweep_mix = mix(&[(&sweep1, 0.4), (&sweep1_h, 0.2)]);
            let pause = silence(0.05, SAMPLE_RATE);

            let mut combined = concat(&[&sweep_mix, &pause, &sweep2, &chirp1, &chirp2]);
            underwater_filter(&mut combined, 0.2);
            normalize_to(&mut combined, 0.75);
            combined
        }

        "log" => {
            // Data log confirmation beeps
            let beep_env = Envelope::new(0.002, 0.03, 0.4, 0.04);

            let beep1 = synth.tone(Waveform::Triangle, 680.0, 0.1, beep_env);
            let beep1_h = synth.tone(Waveform::Sine, 1360.0, 0.08, beep_env);

            let beep2 = synth.tone(Waveform::Triangle, 850.0, 0.12, beep_env);
            let beep2_h = synth.tone(Waveform::Sine, 1700.0, 0.1, beep_env);

            let b1 = mix(&[(&beep1, 0.5), (&beep1_h, 0.2)]);
            let b2 = mix(&[(&beep2, 0.5), (&beep2_h, 0.2)]);

            let pause = silence(0.02, SAMPLE_RATE);
            let mut combined = concat(&[&b1, &pause, &b2]);
            underwater_filter(&mut combined, 0.15);
            combined
        }

        "discovery" => {
            // Triumphant new species fanfare
            let fanfare_env = Envelope::new(0.01, 0.12, 0.4, 0.25);

            // Ascending fanfare notes (C major arpeggio + octave)
            let n1 = synth.tone(Waveform::Sine, 523.0, 0.2, fanfare_env);
            let n2 = synth.tone(Waveform::Sine, 659.0, 0.2, fanfare_env);
            let n3 = synth.tone(Waveform::Sine, 784.0, 0.25, fanfare_env);
            let n4 = synth.tone(Waveform::Sine, 1046.0, 0.4,
                Envelope::new(0.01, 0.15, 0.5, 0.35));

            // Harmonics for richness
            let n1_h = synth.tone(Waveform::Sine, 1046.0, 0.15, fanfare_env);
            let n2_h = synth.tone(Waveform::Sine, 1318.0, 0.15, fanfare_env);
            let n3_h = synth.tone(Waveform::Sine, 1568.0, 0.18, fanfare_env);

            // Shimmer
            let shimmer = synth.filtered_noise(0.9, Some(5000.0), Some(2000.0),
                Envelope::new(0.15, 0.3, 0.15, 0.25), 91);

            let chord1 = mix(&[(&n1, 0.3), (&n1_h, 0.12)]);
            let chord2 = mix(&[(&n2, 0.3), (&n2_h, 0.12)]);
            let chord3 = mix(&[(&n3, 0.35), (&n3_h, 0.15)]);

            let pause = silence(0.06, SAMPLE_RATE);
            let short_pause = silence(0.03, SAMPLE_RATE);

            let mut combined = concat(&[
                &chord1, &short_pause, &chord2, &short_pause, &chord3, &pause, &n4
            ]);

            for (i, &s) in shimmer.iter().enumerate() {
                if i < combined.len() {
                    combined[i] += s * 0.1;
                }
            }

            combined = add_underwater_reverb(&combined, 60.0, 0.25);
            underwater_filter(&mut combined, 0.15);
            normalize_to(&mut combined, 0.8);
            combined
        }

        "zone_enter" => {
            // Zone transition - atmospheric shift
            let trans_env = Envelope::new(0.1, 0.25, 0.5, 0.35);

            // Deep transition tone
            let low = synth.tone(Waveform::Sine, 165.0, 0.6, trans_env);
            let low_h = synth.tone(Waveform::Sine, 330.0, 0.5, trans_env);

            // Atmospheric sweep
            let sweep = synth.sweep(Waveform::Triangle, 200.0, 400.0, 0.5,
                Envelope::new(0.15, 0.2, 0.4, 0.3));

            // Pressure change rumble
            let pressure = synth.filtered_noise(0.6, Some(250.0), Some(40.0),
                trans_env, 92);

            let mut combined = mix(&[
                (&low, 0.3),
                (&low_h, 0.2),
                (&sweep, 0.25),
                (&pressure, 0.2),
            ]);

            combined = add_underwater_reverb(&combined, 100.0, 0.3);
            underwater_filter(&mut combined, 0.35);
            normalize_to(&mut combined, 0.7);
            combined
        }

        "depth_milestone" => {
            // Achievement depth marker
            let ding_env = Envelope::new(0.002, 0.06, 0.3, 0.15);

            let ding = synth.tone(Waveform::Sine, 880.0, 0.2, ding_env);
            let ding_h = synth.tone(Waveform::Sine, 1760.0, 0.15, ding_env);

            let dong = synth.tone(Waveform::Sine, 660.0, 0.25,
                Envelope::new(0.003, 0.08, 0.35, 0.18));
            let dong_h = synth.tone(Waveform::Sine, 1320.0, 0.2,
                Envelope::new(0.004, 0.06, 0.3, 0.15));

            let d1 = mix(&[(&ding, 0.5), (&ding_h, 0.2)]);
            let d2 = mix(&[(&dong, 0.5), (&dong_h, 0.2)]);

            let pause = silence(0.1, SAMPLE_RATE);
            let mut combined = concat(&[&d1, &pause, &d2]);

            combined = add_underwater_reverb(&combined, 80.0, 0.25);
            underwater_filter(&mut combined, 0.2);
            combined
        }

        // === ENCOUNTERS ===
        "encounter_start" => {
            // Epic encounter begins - dramatic and ominous
            let duration = 1.5;
            let build_env = Envelope::new(0.3, 0.6, 0.6, 0.4);

            // Deep ominous drone
            let drone1 = synth.tone(Waveform::Sine, 55.0, duration, build_env);
            let drone2 = synth.tone(Waveform::Sine, 57.0, duration, build_env); // Beating
            let drone3 = synth.tone(Waveform::Triangle, 27.5, duration,
                Envelope::new(0.4, 0.5, 0.5, 0.35));

            // Rising tension
            let rise = synth.sweep(Waveform::Sine, 80.0, 220.0, duration,
                Envelope::new(0.2, 0.8, 0.4, 0.3));
            let rise_h = synth.sweep(Waveform::Sine, 160.0, 440.0, 1.3,
                Envelope::new(0.3, 0.6, 0.3, 0.25));

            // Pressure rumble
            let rumble = synth.filtered_noise(duration, Some(150.0), Some(25.0),
                build_env, 93);

            // Distant movement
            let movement = synth.filtered_noise(duration, Some(400.0), Some(80.0),
                Envelope::new(0.5, 0.5, 0.4, 0.3), 94);

            let mut combined = mix(&[
                (&drone1, 0.25),
                (&drone2, 0.25),
                (&drone3, 0.15),
                (&rise, 0.2),
                (&rise_h, 0.12),
                (&rumble, 0.2),
                (&movement, 0.12),
            ]);

            combined = add_underwater_reverb(&combined, 120.0, 0.35);
            underwater_filter(&mut combined, 0.4);
            normalize_to(&mut combined, 0.75);
            combined
        }

        "encounter_end" => {
            // Encounter resolves - fading into distance
            let duration = 1.8;
            let fade_env = Envelope::new(0.1, 0.3, 0.3, 1.0);

            // Descending resolution
            let desc = synth.sweep(Waveform::Sine, 220.0, 80.0, duration, fade_env);
            let desc_h = synth.sweep(Waveform::Sine, 440.0, 160.0, 1.6,
                Envelope::new(0.15, 0.25, 0.25, 0.9));

            // Fading pressure
            let fade = synth.filtered_noise(duration, Some(200.0), Some(30.0),
                fade_env, 95);

            // Distant movement away
            let away = synth.sweep(Waveform::Triangle, 150.0, 60.0, duration,
                Envelope::new(0.2, 0.4, 0.2, 0.8));

            let mut combined = mix(&[
                (&desc, 0.3),
                (&desc_h, 0.2),
                (&fade, 0.25),
                (&away, 0.2),
            ]);

            combined = add_underwater_reverb(&combined, 150.0, 0.4);
            underwater_filter(&mut combined, 0.5);
            normalize_to(&mut combined, 0.6);
            combined
        }

        "danger_near" => {
            // Proximity warning - urgent but underwater
            let pulse_env = Envelope::new(0.01, 0.08, 0.4, 0.1);

            // Warning pulses with harmonics
            let mut pulse1 = synth.tone(Waveform::Saw, 140.0, 0.18, pulse_env);
            let pulse1_h = synth.tone(Waveform::Sine, 280.0, 0.16, pulse_env);
            low_pass(&mut pulse1, 800.0, SAMPLE_RATE);

            let mut pulse2 = synth.tone(Waveform::Saw, 170.0, 0.18, pulse_env);
            let pulse2_h = synth.tone(Waveform::Sine, 340.0, 0.16, pulse_env);
            low_pass(&mut pulse2, 900.0, SAMPLE_RATE);

            // Danger undertone
            let danger = synth.tone(Waveform::Sine, 70.0, 0.8,
                Envelope::new(0.1, 0.2, 0.5, 0.3));

            let p1 = mix(&[(&pulse1, 0.4), (&pulse1_h, 0.2)]);
            let p2 = mix(&[(&pulse2, 0.4), (&pulse2_h, 0.2)]);

            let pause = silence(0.12, SAMPLE_RATE);

            let mut pulses = concat(&[&p1, &pause, &p2, &pause, &p1]);

            // Add danger tone underneath
            for (i, &s) in danger.iter().enumerate() {
                if i < pulses.len() {
                    pulses[i] += s * 0.3;
                }
            }

            underwater_filter(&mut pulses, 0.35);
            normalize_to(&mut pulses, 0.75);
            pulses
        }

        _ => panic!("Unknown sound ID: {}", id),
    };

    // Normalize all sounds
    normalize_to(&mut result, 0.85);
    result
}

/// Generate rich, layered bubble sounds
fn generate_rich_bubbles(synth: &Synth, duration: f32, seed: u64) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut result = vec![0.0; num_samples];

    // Create varied bubble sizes and timings
    let bubble_count = (duration * 15.0) as usize;

    for i in 0..bubble_count {
        // Varied frequencies for different bubble sizes
        let size_factor = ((seed + i as u64) % 5) as f32;
        let freq = 600.0 + size_factor * 200.0 + ((seed + i as u64 * 7) % 100) as f32 * 3.0;

        // Bubble pop with resonance
        let bubble_env = Envelope::new(0.0, 0.01 + size_factor * 0.005, 0.15, 0.03);
        let mut bubble = synth.tone(Waveform::Sine, freq, 0.06, bubble_env);

        // Add resonance
        let resonance = synth.tone(Waveform::Sine, freq * 0.5, 0.08,
            Envelope::new(0.005, 0.02, 0.2, 0.04));

        let bubble_mix = mix(&[(&bubble, 0.5), (&resonance, 0.25)]);

        // Random timing
        let offset = ((seed + i as u64 * 13) % (num_samples as u64 - bubble_mix.len() as u64)) as usize;

        for (j, &s) in bubble_mix.iter().enumerate() {
            if offset + j < result.len() {
                result[offset + j] += s * 0.35;
            }
        }
    }

    // Add overall noise layer
    let mut noise = synth.filtered_noise(duration, Some(2000.0), Some(400.0),
        Envelope::new(0.1, 0.2, 0.5, 0.2), seed + 1000);

    for (i, &s) in noise.iter().enumerate() {
        if i < result.len() {
            result[i] += s * 0.2;
        }
    }

    underwater_filter(&mut result, 0.25);
    normalize_to(&mut result, 0.7);
    result
}

/// Generate ambient bubble background
fn generate_ambient_bubbles(synth: &Synth, duration: f32, seed: u64) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut result = vec![0.0; num_samples];

    // Sparse, gentle bubbles
    let bubble_count = (duration * 5.0) as usize;

    for i in 0..bubble_count {
        let freq = 800.0 + ((seed + i as u64) % 400) as f32;
        let bubble = synth.tone(Waveform::Sine, freq, 0.04,
            Envelope::new(0.0, 0.01, 0.1, 0.025));

        let offset = ((seed + i as u64 * 17) % (num_samples as u64 - bubble.len() as u64)) as usize;

        for (j, &s) in bubble.iter().enumerate() {
            if offset + j < result.len() {
                result[offset + j] += s * 0.25;
            }
        }
    }

    result
}

/// Generate vent-specific aggressive bubbles
fn generate_vent_bubbles(synth: &Synth, duration: f32, seed: u64) -> Vec<f32> {
    let num_samples = (duration * SAMPLE_RATE as f32) as usize;
    let mut result = vec![0.0; num_samples];

    // Rapid, aggressive bubbling
    let bubble_count = (duration * 25.0) as usize;

    for i in 0..bubble_count {
        let freq = 400.0 + ((seed + i as u64) % 600) as f32;
        let bubble = synth.tone(Waveform::Sine, freq, 0.035,
            Envelope::new(0.0, 0.008, 0.05, 0.02));

        let offset = ((seed + i as u64 * 11) % (num_samples as u64 - bubble.len() as u64)) as usize;

        for (j, &s) in bubble.iter().enumerate() {
            if offset + j < result.len() {
                result[offset + j] += s * 0.3;
            }
        }
    }

    // Add hiss layer
    let hiss = synth.filtered_noise(duration, Some(3000.0), Some(800.0),
        Envelope::new(0.1, 0.2, 0.6, 0.2), seed + 500);

    for (i, &s) in hiss.iter().enumerate() {
        if i < result.len() {
            result[i] += s * 0.15;
        }
    }

    result
}
