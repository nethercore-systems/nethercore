//! Neon Drift sound generation
//!
//! Procedural audio synthesis for engine, driving, collisions, and race sounds.

use proc_gen::audio::*;
use std::path::Path;

/// Sound ID and description
pub type SoundDef = (&'static str, &'static str);

/// All Neon Drift sounds
pub const SOUNDS: &[SoundDef] = &[
    // Engine
    ("engine_idle", "Engine idle loop"),
    ("engine_rev", "Engine revving"),
    ("boost", "Nitro boost"),

    // Driving
    ("drift", "Tire drift/screech"),
    ("brake", "Hard brake"),
    ("shift", "Gear shift"),

    // Collisions
    ("wall", "Wall collision"),
    ("barrier", "Barrier crash"),

    // Race
    ("countdown", "Race countdown beep"),
    ("checkpoint", "Checkpoint passed"),
    ("finish", "Race finish fanfare"),
];

/// Generate all sounds to the output directory
pub fn generate_all(output_dir: &Path) {
    println!("  Generating {} sounds", SOUNDS.len());

    let synth = Synth::new(SAMPLE_RATE);

    for (id, description) in SOUNDS {
        let samples = generate_sound(&synth, id);
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

/// Synthesize a specific sound by ID
fn generate_sound(synth: &Synth, id: &str) -> Vec<f32> {
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
