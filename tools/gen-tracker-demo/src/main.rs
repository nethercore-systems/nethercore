//! Generates procedural audio samples and XM tracker files for tracker-demo example
//!
//! Creates three distinct songs:
//! - nether_groove.xm - Funky Jazz (default, 110 BPM, F Dorian) - Purple theme
//! - nether_fire.xm - Eurobeat (155 BPM, D minor) - Orange theme
//! - nether_drive.xm - Synthwave (105 BPM, A minor) - Green theme
//!
//! Each song has its own instrument set optimized for the genre.

mod synthesizers;
mod wav_writer;
mod xm_builder;

use std::fs;
use std::path::Path;

use synthesizers::{
    apply_fades,
    // Funk instruments
    generate_kick_funk, generate_snare_funk, generate_hihat_funk,
    generate_bass_funk, generate_epiano, generate_lead_jazz,
    // Eurobeat instruments
    generate_kick_euro, generate_snare_euro, generate_hihat_euro,
    generate_bass_euro, generate_supersaw, generate_brass_euro, generate_pad_euro,
    // Synthwave instruments
    generate_kick_synth, generate_snare_synth, generate_hihat_synth,
    generate_bass_synth, generate_lead_synth, generate_arp_synth, generate_pad_synth,
};

use wav_writer::write_wav;

use xm_builder::{
    generate_funk_xm, generate_funk_xm_embedded,
    generate_eurobeat_xm, generate_eurobeat_xm_embedded,
    generate_synthwave_xm, generate_synthwave_xm_embedded,
};

fn main() {
    // Output to shared examples/assets folder with tracker- prefix
    let output_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("examples")
        .join("assets");

    // Create output directory if it doesn't exist
    fs::create_dir_all(&output_dir).expect("Failed to create assets directory");

    println!("Generating tracker-demo assets to shared examples/assets...");

    // Generate Funky Jazz song (default)
    println!("\n=== Generating 'Nether Groove' (Funky Jazz) ===");
    generate_funk_assets(&output_dir);

    // Generate Eurobeat song
    println!("\n=== Generating 'Nether Fire' (Eurobeat) ===");
    generate_eurobeat_assets(&output_dir);

    // Generate Synthwave song
    println!("\n=== Generating 'Nether Drive' (Synthwave) ===");
    generate_synthwave_assets(&output_dir);

    println!("\nDone!");
}

// ============================================================================
// FUNKY JAZZ SONG - "Nether Groove"
// ============================================================================

fn generate_funk_assets(output_dir: &Path) {
    // Generate funk instruments (with tracker- prefix for shared assets folder)
    let mut kick = generate_kick_funk();
    apply_fades(&mut kick);
    write_wav(&output_dir.join("tracker-kick_funk.wav"), &kick);
    println!("  Generated tracker-kick_funk.wav ({} samples)", kick.len());

    let mut snare = generate_snare_funk();
    apply_fades(&mut snare);
    write_wav(&output_dir.join("tracker-snare_funk.wav"), &snare);
    println!("  Generated tracker-snare_funk.wav ({} samples)", snare.len());

    let mut hihat = generate_hihat_funk();
    apply_fades(&mut hihat);
    write_wav(&output_dir.join("tracker-hihat_funk.wav"), &hihat);
    println!("  Generated tracker-hihat_funk.wav ({} samples)", hihat.len());

    let mut bass = generate_bass_funk();
    apply_fades(&mut bass);
    write_wav(&output_dir.join("tracker-bass_funk.wav"), &bass);
    println!("  Generated tracker-bass_funk.wav ({} samples)", bass.len());

    let mut epiano = generate_epiano();
    apply_fades(&mut epiano);
    write_wav(&output_dir.join("tracker-epiano.wav"), &epiano);
    println!("  Generated tracker-epiano.wav ({} samples)", epiano.len());

    let mut lead = generate_lead_jazz();
    apply_fades(&mut lead);
    write_wav(&output_dir.join("tracker-lead_jazz.wav"), &lead);
    println!("  Generated tracker-lead_jazz.wav ({} samples)", lead.len());

    // Generate sample-less XM file
    let xm = generate_funk_xm();
    fs::write(output_dir.join("tracker-nether_groove.xm"), &xm).expect("Failed to write tracker-nether_groove.xm");
    println!("  Generated tracker-nether_groove.xm ({} bytes)", xm.len());

    // Generate embedded XM file
    let samples = vec![kick, snare, hihat, bass, epiano, lead];
    let xm_embedded = generate_funk_xm_embedded(&samples);
    fs::write(output_dir.join("tracker-nether_groove-embedded.xm"), &xm_embedded).expect("Failed to write tracker-nether_groove-embedded.xm");
    println!("  Generated tracker-nether_groove-embedded.xm ({} bytes)", xm_embedded.len());
}

// ============================================================================
// EUROBEAT SONG - "Nether Fire"
// ============================================================================

fn generate_eurobeat_assets(output_dir: &Path) {
    // Generate eurobeat instruments (with tracker- prefix for shared assets folder)
    let mut kick = generate_kick_euro();
    apply_fades(&mut kick);
    write_wav(&output_dir.join("tracker-kick_euro.wav"), &kick);
    println!("  Generated tracker-kick_euro.wav ({} samples)", kick.len());

    let mut snare = generate_snare_euro();
    apply_fades(&mut snare);
    write_wav(&output_dir.join("tracker-snare_euro.wav"), &snare);
    println!("  Generated tracker-snare_euro.wav ({} samples)", snare.len());

    let mut hihat = generate_hihat_euro();
    apply_fades(&mut hihat);
    write_wav(&output_dir.join("tracker-hihat_euro.wav"), &hihat);
    println!("  Generated tracker-hihat_euro.wav ({} samples)", hihat.len());

    let mut bass = generate_bass_euro();
    apply_fades(&mut bass);
    write_wav(&output_dir.join("tracker-bass_euro.wav"), &bass);
    println!("  Generated tracker-bass_euro.wav ({} samples)", bass.len());

    let mut supersaw = generate_supersaw();
    apply_fades(&mut supersaw);
    write_wav(&output_dir.join("tracker-supersaw.wav"), &supersaw);
    println!("  Generated tracker-supersaw.wav ({} samples)", supersaw.len());

    let mut brass = generate_brass_euro();
    apply_fades(&mut brass);
    write_wav(&output_dir.join("tracker-brass_euro.wav"), &brass);
    println!("  Generated tracker-brass_euro.wav ({} samples)", brass.len());

    let mut pad = generate_pad_euro();
    apply_fades(&mut pad);
    write_wav(&output_dir.join("tracker-pad_euro.wav"), &pad);
    println!("  Generated tracker-pad_euro.wav ({} samples)", pad.len());

    // Generate sample-less XM file
    let xm = generate_eurobeat_xm();
    fs::write(output_dir.join("tracker-nether_fire.xm"), &xm).expect("Failed to write tracker-nether_fire.xm");
    println!("  Generated tracker-nether_fire.xm ({} bytes)", xm.len());

    // Generate embedded XM file
    let samples = vec![kick, snare, hihat, bass, supersaw, brass, pad];
    let xm_embedded = generate_eurobeat_xm_embedded(&samples);
    fs::write(output_dir.join("tracker-nether_fire-embedded.xm"), &xm_embedded).expect("Failed to write tracker-nether_fire-embedded.xm");
    println!("  Generated tracker-nether_fire-embedded.xm ({} bytes)", xm_embedded.len());
}

// ============================================================================
// SYNTHWAVE SONG - "Nether Drive"
// ============================================================================

fn generate_synthwave_assets(output_dir: &Path) {
    // Generate synthwave instruments (with tracker- prefix for shared assets folder)
    let mut kick = generate_kick_synth();
    apply_fades(&mut kick);
    write_wav(&output_dir.join("tracker-kick_synth.wav"), &kick);
    println!("  Generated tracker-kick_synth.wav ({} samples)", kick.len());

    let mut snare = generate_snare_synth();
    apply_fades(&mut snare);
    write_wav(&output_dir.join("tracker-snare_synth.wav"), &snare);
    println!("  Generated tracker-snare_synth.wav ({} samples)", snare.len());

    let mut hihat = generate_hihat_synth();
    apply_fades(&mut hihat);
    write_wav(&output_dir.join("tracker-hihat_synth.wav"), &hihat);
    println!("  Generated tracker-hihat_synth.wav ({} samples)", hihat.len());

    let mut bass = generate_bass_synth();
    apply_fades(&mut bass);
    write_wav(&output_dir.join("tracker-bass_synth.wav"), &bass);
    println!("  Generated tracker-bass_synth.wav ({} samples)", bass.len());

    let mut lead = generate_lead_synth();
    apply_fades(&mut lead);
    write_wav(&output_dir.join("tracker-lead_synth.wav"), &lead);
    println!("  Generated tracker-lead_synth.wav ({} samples)", lead.len());

    let mut arp = generate_arp_synth();
    apply_fades(&mut arp);
    write_wav(&output_dir.join("tracker-arp_synth.wav"), &arp);
    println!("  Generated tracker-arp_synth.wav ({} samples)", arp.len());

    let mut pad = generate_pad_synth();
    apply_fades(&mut pad);
    write_wav(&output_dir.join("tracker-pad_synth.wav"), &pad);
    println!("  Generated tracker-pad_synth.wav ({} samples)", pad.len());

    // Generate sample-less XM file
    let xm = generate_synthwave_xm();
    fs::write(output_dir.join("tracker-nether_drive.xm"), &xm).expect("Failed to write tracker-nether_drive.xm");
    println!("  Generated tracker-nether_drive.xm ({} bytes)", xm.len());

    // Generate embedded XM file
    let samples = vec![kick, snare, hihat, bass, lead, arp, pad];
    let xm_embedded = generate_synthwave_xm_embedded(&samples);
    fs::write(output_dir.join("tracker-nether_drive-embedded.xm"), &xm_embedded).expect("Failed to write tracker-nether_drive-embedded.xm");
    println!("  Generated tracker-nether_drive-embedded.xm ({} bytes)", xm_embedded.len());
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_funk_xm_parses() {
        let xm_data = generate_funk_xm();
        let module =
            nether_xm::parse_xm(&xm_data).expect("Funk XM should parse");

        assert_eq!(module.name, "Nether Groove");
        assert_eq!(module.num_channels, 8);
        assert_eq!(module.num_patterns, 6);
        assert_eq!(module.num_instruments, 6);
        assert_eq!(module.default_bpm, 110);
    }

    #[test]
    fn test_eurobeat_xm_parses() {
        let xm_data = generate_eurobeat_xm();
        let module =
            nether_xm::parse_xm(&xm_data).expect("Eurobeat XM should parse");

        assert_eq!(module.name, "Nether Fire");
        assert_eq!(module.num_channels, 8);
        assert_eq!(module.num_patterns, 8);
        assert_eq!(module.num_instruments, 7);
        assert_eq!(module.default_bpm, 155);
    }

    #[test]
    fn test_synthwave_xm_parses() {
        let xm_data = generate_synthwave_xm();
        let module =
            nether_xm::parse_xm(&xm_data).expect("Synthwave XM should parse");

        assert_eq!(module.name, "Nether Drive");
        assert_eq!(module.num_channels, 8);
        assert_eq!(module.num_patterns, 8);
        assert_eq!(module.num_instruments, 7);
        assert_eq!(module.default_bpm, 105);
    }

    #[test]
    fn test_funk_instrument_names() {
        let xm_data = generate_funk_xm();
        let names = nether_xm::get_instrument_names(&xm_data)
            .expect("Should get funk instrument names");

        assert_eq!(names.len(), 6);
        assert_eq!(names[0], "kick_funk");
        assert_eq!(names[1], "snare_funk");
        assert_eq!(names[2], "hihat_funk");
        assert_eq!(names[3], "bass_funk");
        assert_eq!(names[4], "epiano");
        assert_eq!(names[5], "lead_jazz");
    }

    #[test]
    fn test_eurobeat_instrument_names() {
        let xm_data = generate_eurobeat_xm();
        let names = nether_xm::get_instrument_names(&xm_data)
            .expect("Should get eurobeat instrument names");

        assert_eq!(names.len(), 7);
        assert_eq!(names[0], "kick_euro");
        assert_eq!(names[1], "snare_euro");
        assert_eq!(names[2], "hihat_euro");
        assert_eq!(names[3], "bass_euro");
        assert_eq!(names[4], "supersaw");
        assert_eq!(names[5], "brass_euro");
        assert_eq!(names[6], "pad_euro");
    }

    #[test]
    fn test_synthwave_instrument_names() {
        let xm_data = generate_synthwave_xm();
        let names = nether_xm::get_instrument_names(&xm_data)
            .expect("Should get synthwave instrument names");

        assert_eq!(names.len(), 7);
        assert_eq!(names[0], "kick_synth");
        assert_eq!(names[1], "snare_synth");
        assert_eq!(names[2], "hihat_synth");
        assert_eq!(names[3], "bass_synth");
        assert_eq!(names[4], "lead_synth");
        assert_eq!(names[5], "arp_synth");
        assert_eq!(names[6], "pad_synth");
    }
}
