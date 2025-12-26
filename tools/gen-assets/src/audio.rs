//! Showcase audio generation
//!
//! Generates procedural sound effects for the proc-sounds-viewer example.
//! Uses proc-gen-showcase-defs as the single source of truth.

use proc_gen::audio::*;
use std::path::Path;

/// Generate all showcase sounds to the specified output directory
pub fn generate_showcase_sounds(output_dir: &Path) {
    std::fs::create_dir_all(output_dir).expect("Failed to create audio output directory");

    println!(
        "  Generating {} procedural sounds",
        showcase::SHOWCASE_SOUNDS.len()
    );
    println!("  Output -> {}", output_dir.display());

    let synth = Synth::new(SAMPLE_RATE);

    for sound in showcase::SHOWCASE_SOUNDS {
        let samples = showcase::generate_showcase_sound(&synth, sound.id);
        let pcm = to_pcm_i16(&samples);
        let path = output_dir.join(format!("{}.wav", sound.id));

        write_wav(&pcm, SAMPLE_RATE, &path).expect("Failed to write WAV file");

        println!(
            "    -> {}.wav ({} samples, {:.2}s) - {}",
            sound.id,
            pcm.len(),
            pcm.len() as f32 / SAMPLE_RATE as f32,
            sound.name
        );
    }

    println!("  Done! Generated {} sounds", showcase::SHOWCASE_SOUNDS.len());
}
