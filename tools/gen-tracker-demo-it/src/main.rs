//! IT Tracker Demo Generator
//!
//! Generates procedural audio samples and IT tracker files:
//! - Nether Dawn (Epic/Orchestral) - 90 BPM in D major
//! - Nether Mist (Ambient) - 70 BPM in D minor
//! - Nether Storm (DnB/Action) - 174 BPM in F minor
//!
//! Each song is generated in two variants:
//! - Embedded: Self-contained IT file with samples embedded
//! - External: IT file with separate WAV sample files

mod it_builder;
mod synthesizers;
mod wav_writer;

use gen_tracker_common::write_wav;
use std::fs;
use std::path::Path;

fn main() {
    println!("IT Tracker Demo Generator");
    println!("=========================");
    println!();

    // Output directory: examples/assets/
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("assets");

    // Ensure output directory exists
    fs::create_dir_all(&output_dir).expect("Failed to create output directory");

    // Generate each song
    generate_storm(&output_dir);
    generate_mist(&output_dir);
    generate_dawn(&output_dir);

    println!();
    println!("Generation complete!");
    println!("Output directory: {}", output_dir.display());
}

fn generate_storm(output_dir: &Path) {
    println!("Generating Nether Storm (DnB @ 174 BPM)...");

    let (it_data, samples) = it_builder::generate_storm_it();

    // Write embedded IT file
    let it_path = output_dir.join("tracker-nether_storm-embedded.it");
    fs::write(&it_path, &it_data).expect("Failed to write IT file");
    println!("  Wrote: {}", it_path.display());

    // Write individual WAV samples
    for (name, data) in &samples {
        let wav_path = output_dir.join(format!("tracker-storm_{}.wav", name));
        write_wav(&wav_path, data);
        println!("  Wrote: {}", wav_path.display());
    }

    println!("  Done: {} bytes IT, {} samples", it_data.len(), samples.len());
}

fn generate_mist(output_dir: &Path) {
    println!("Generating Nether Mist (Ambient @ 70 BPM)...");

    let (it_data, samples) = it_builder::generate_mist_it();

    // Write embedded IT file
    let it_path = output_dir.join("tracker-nether_mist-embedded.it");
    fs::write(&it_path, &it_data).expect("Failed to write IT file");
    println!("  Wrote: {}", it_path.display());

    // Write individual WAV samples
    for (name, data) in &samples {
        let wav_path = output_dir.join(format!("tracker-mist_{}.wav", name));
        write_wav(&wav_path, data);
        println!("  Wrote: {}", wav_path.display());
    }

    println!("  Done: {} bytes IT, {} samples", it_data.len(), samples.len());
}

fn generate_dawn(output_dir: &Path) {
    println!("Generating Nether Dawn (Orchestral @ 90 BPM)...");

    let (it_data, samples) = it_builder::generate_dawn_it();

    // Write embedded IT file
    let it_path = output_dir.join("tracker-nether_dawn-embedded.it");
    fs::write(&it_path, &it_data).expect("Failed to write IT file");
    println!("  Wrote: {}", it_path.display());

    // Write individual WAV samples
    for (name, data) in &samples {
        let wav_path = output_dir.join(format!("tracker-dawn_{}.wav", name));
        write_wav(&wav_path, data);
        println!("  Wrote: {}", wav_path.display());
    }

    println!("  Done: {} bytes IT, {} samples", it_data.len(), samples.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use nether_it::parse_it;

    #[test]
    fn test_storm_parses() {
        let (it_data, _) = it_builder::generate_storm_it();
        let result = parse_it(&it_data);
        assert!(result.is_ok(), "Storm IT parse failed: {:?}", result.err());

        let module = result.unwrap();
        assert_eq!(module.name, "Nether Storm");
        assert_eq!(module.num_channels, 16);
        assert_eq!(module.initial_tempo, 174);
        assert_eq!(module.initial_speed, 3);
        assert_eq!(module.num_instruments, 15);
        assert_eq!(module.num_samples, 15);
        assert_eq!(module.num_patterns, 6);
    }

    #[test]
    fn test_mist_parses() {
        let (it_data, _) = it_builder::generate_mist_it();
        let result = parse_it(&it_data);
        assert!(result.is_ok(), "Mist IT parse failed: {:?}", result.err());

        let module = result.unwrap();
        assert_eq!(module.name, "Nether Mist");
        assert_eq!(module.num_channels, 12);
        assert_eq!(module.initial_tempo, 70);
        assert_eq!(module.initial_speed, 6);
        assert_eq!(module.num_instruments, 10);
        assert_eq!(module.num_samples, 10);
        assert_eq!(module.num_patterns, 6);
    }

    #[test]
    fn test_dawn_parses() {
        let (it_data, _) = it_builder::generate_dawn_it();
        let result = parse_it(&it_data);
        assert!(result.is_ok(), "Dawn IT parse failed: {:?}", result.err());

        let module = result.unwrap();
        assert_eq!(module.name, "Nether Dawn");
        assert_eq!(module.num_channels, 16);
        assert_eq!(module.initial_tempo, 90);
        assert_eq!(module.initial_speed, 6);
        assert_eq!(module.num_instruments, 16);
        assert_eq!(module.num_samples, 16);
        assert_eq!(module.num_patterns, 6);
    }
}
