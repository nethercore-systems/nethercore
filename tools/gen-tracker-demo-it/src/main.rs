//! IT Tracker Demo Generator
//!
//! Generates procedural audio samples and IT tracker files:
//! - Nether Acid (Acid Techno) - 130 BPM in E minor (~60-70s)
//! - Nether Dawn (Epic/Orchestral) - 90 BPM in D major
//! - Nether Storm (DnB/Action) - 174 BPM in F minor
//!
//! Each song is generated in two variants:
//! - Stripped: IT file without sample data (for ROM/external samples)
//! - Embedded: Self-contained IT file with samples embedded

mod it_builder;
mod synthesizers;

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
    generate_acid(&output_dir);
    generate_storm(&output_dir);
    generate_dawn(&output_dir);

    println!();
    println!("Generation complete!");
    println!("Output directory: {}", output_dir.display());
}

fn generate_storm(output_dir: &Path) {
    println!("Generating Nether Storm (DnB @ 174 BPM)...");

    // Generate stripped IT file
    let (it_stripped, samples) = it_builder::generate_storm_it_stripped();
    let it_stripped_path = output_dir.join("tracker-nether_storm.it");
    fs::write(&it_stripped_path, &it_stripped).expect("Failed to write stripped IT");
    println!("  Wrote: {}", it_stripped_path.display());

    // Generate embedded IT file
    let (it_embedded, _) = it_builder::generate_storm_it_embedded();
    let it_embedded_path = output_dir.join("tracker-nether_storm-embedded.it");
    fs::write(&it_embedded_path, &it_embedded).expect("Failed to write embedded IT");
    println!("  Wrote: {}", it_embedded_path.display());

    // Write individual WAV samples
    for (name, data) in &samples {
        let wav_path = output_dir.join(format!("tracker-storm_{}.wav", name));
        write_wav(&wav_path, data);
        println!("  Wrote: {}", wav_path.display());
    }

    println!("  Done: {} bytes stripped, {} bytes embedded, {} samples",
             it_stripped.len(), it_embedded.len(), samples.len());
}

fn generate_acid(output_dir: &Path) {
    println!("Generating Nether Acid (Acid Techno @ 130 BPM)...");

    // Generate stripped IT file
    let (it_stripped, samples) = it_builder::generate_acid_it_stripped();
    let it_stripped_path = output_dir.join("tracker-nether_acid.it");
    fs::write(&it_stripped_path, &it_stripped).expect("Failed to write stripped IT");
    println!("  Wrote: {}", it_stripped_path.display());

    // Generate embedded IT file
    let (it_embedded, _) = it_builder::generate_acid_it_embedded();
    let it_embedded_path = output_dir.join("tracker-nether_acid-embedded.it");
    fs::write(&it_embedded_path, &it_embedded).expect("Failed to write embedded IT");
    println!("  Wrote: {}", it_embedded_path.display());

    // Write individual WAV samples
    for (name, data) in &samples {
        let wav_path = output_dir.join(format!("tracker-acid_{}.wav", name));
        write_wav(&wav_path, data);
        println!("  Wrote: {}", wav_path.display());
    }

    println!("  Done: {} bytes stripped, {} bytes embedded, {} samples",
             it_stripped.len(), it_embedded.len(), samples.len());
}

fn generate_dawn(output_dir: &Path) {
    println!("Generating Nether Dawn (Orchestral @ 90 BPM)...");

    // Generate stripped IT file
    let (it_stripped, samples) = it_builder::generate_dawn_it_stripped();
    let it_stripped_path = output_dir.join("tracker-nether_dawn.it");
    fs::write(&it_stripped_path, &it_stripped).expect("Failed to write stripped IT");
    println!("  Wrote: {}", it_stripped_path.display());

    // Generate embedded IT file
    let (it_embedded, _) = it_builder::generate_dawn_it_embedded();
    let it_embedded_path = output_dir.join("tracker-nether_dawn-embedded.it");
    fs::write(&it_embedded_path, &it_embedded).expect("Failed to write embedded IT");
    println!("  Wrote: {}", it_embedded_path.display());

    // Write individual WAV samples
    for (name, data) in &samples {
        let wav_path = output_dir.join(format!("tracker-dawn_{}.wav", name));
        write_wav(&wav_path, data);
        println!("  Wrote: {}", wav_path.display());
    }

    println!("  Done: {} bytes stripped, {} bytes embedded, {} samples",
             it_stripped.len(), it_embedded.len(), samples.len());
}

#[cfg(test)]
mod tests {
    use super::*;
    use nether_it::parse_it;

    #[test]
    fn test_storm_parses() {
        let (it_data, _) = it_builder::generate_storm_it_embedded();
        let result = parse_it(&it_data);
        assert!(result.is_ok(), "Storm IT parse failed: {:?}", result.err());

        let module = result.unwrap();
        assert_eq!(module.name, "Nether Storm");
        assert_eq!(module.num_channels, 16);
        assert_eq!(module.initial_tempo, 174);
        assert_eq!(module.initial_speed, 3);
        assert_eq!(module.num_instruments, 15);
        assert_eq!(module.num_samples, 15);
        assert_eq!(module.num_patterns, 12); // Storm has 12 patterns
    }

    #[test]
    fn test_acid_parses() {
        let (it_data, _) = it_builder::generate_acid_it_embedded();
        let result = parse_it(&it_data);
        assert!(result.is_ok(), "Acid IT parse failed: {:?}", result.err());

        let module = result.unwrap();
        assert_eq!(module.name, "Nether Acid");
        assert_eq!(module.num_channels, 8);
        assert_eq!(module.initial_tempo, 130);
        assert_eq!(module.initial_speed, 6);
        assert_eq!(module.num_instruments, 11); // Updated: 7 → 11 instruments
        assert_eq!(module.num_samples, 11);     // Updated: 7 → 11 samples
        assert_eq!(module.num_patterns, 12);    // Updated: 6 → 12 patterns
    }

    #[test]
    fn test_dawn_parses() {
        let (it_data, _) = it_builder::generate_dawn_it_embedded();
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
