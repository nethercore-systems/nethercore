//! Nether Acid - Acid Techno IT generator
//!
//! 130 BPM, E minor, 8 channels
//! Duration: ~60-70 seconds (18 patterns × 4 bars)
//! Features: TB-303 acid bassline with resonant filter, 909 drums, IT effects (portamento, filter automation, panning)

use super::{make_instrument, make_sample};
use crate::synthesizers;
use nether_it::{ItFlags, ItWriter};

mod bass;
mod drums;
mod patterns;

use patterns::{
    build_breakdown_deep_pattern, build_breakdown_pattern, build_build_intense_pattern,
    build_build_pattern, build_drop_b_intense_pattern, build_drop_b_pattern, build_drop_pattern,
    build_drop_variation_pattern, build_groove_a_pattern, build_groove_b_pattern,
    build_intro_pattern, build_outro_pattern,
};

// ============================================================================
// Note Constants - E minor (E F# G A B C D)
// ============================================================================

// Octave 2 (bass range)
pub(super) const E2: u8 = 28;
pub(super) const _FS2: u8 = 30;
pub(super) const G2: u8 = 31;
pub(super) const A2: u8 = 33;
pub(super) const B2: u8 = 35;
pub(super) const _C3: u8 = 36;
pub(super) const D3: u8 = 38;

// Octave 3 (upper bass / pad range)
pub(super) const E3: u8 = 40;
pub(super) const FS3: u8 = 42; // For B minor
pub(super) const _G3: u8 = 43;
pub(super) const A3: u8 = 45; // For B minor
pub(super) const _B3: u8 = 47;
pub(super) const _C4: u8 = 48;
pub(super) const _D4: u8 = 50;

// Octave 4 (lead range)
pub(super) const _E4: u8 = 52;

// Octave 5 (for drums - no pitch shift)
pub(super) const C5: u8 = 60;

// ============================================================================
// Instrument and Channel Constants
// ============================================================================

// Instruments (1-indexed for IT format)
pub(super) const INST_KICK: u8 = 1;
pub(super) const INST_CLAP: u8 = 2;
pub(super) const INST_HH_CLOSED: u8 = 3;
pub(super) const INST_HH_OPEN: u8 = 4;
pub(super) const INST_BASS_303: u8 = 5;
pub(super) const INST_PAD: u8 = 6;
pub(super) const INST_STAB: u8 = 7;
pub(super) const INST_BASS_303_SQUELCH: u8 = 8; // Higher resonance for climax
pub(super) const INST_RISER: u8 = 9; // White noise sweep for builds
pub(super) const INST_ATMOSPHERE: u8 = 10; // Subtle texture layer
pub(super) const INST_CRASH: u8 = 11; // Cymbal crash for transitions

// Channels (0-indexed)
pub(super) const CH_KICK: u8 = 0;
pub(super) const CH_CLAP: u8 = 1;
pub(super) const CH_HIHAT: u8 = 2;
pub(super) const CH_HIHAT_OPEN: u8 = 3;
pub(super) const CH_303: u8 = 4;
pub(super) const CH_PAD: u8 = 5;
pub(super) const CH_STAB: u8 = 6;
pub(super) const CH_FX: u8 = 7; // Used for risers, crashes, and FX

// ============================================================================
// Main Generators
// ============================================================================

/// Generate stripped Nether Acid IT file (no sample data, for ROM/external samples)
pub fn generate_acid_it_stripped() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
    let mut writer = ItWriter::new("Nether Acid");

    // Set up module parameters
    writer.set_channels(8);
    writer.set_speed(6); // Standard speed
    writer.set_tempo(130); // Classic acid house tempo
    writer.set_global_volume(128);
    writer.set_mix_volume(64);
    writer.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES);

    // Generate and collect samples
    let mut samples = Vec::new();
    let sample_rate = 22050;

    // 1. Kick 909
    let kick_data = synthesizers::generate_kick_909();
    samples.push(("kick", kick_data));
    let kick_sample = make_sample("acid_kick", sample_rate);
    writer.add_sample_header_only(kick_sample);
    writer.add_instrument(make_instrument("acid_kick", 1));

    // 2. Clap 909
    let clap_data = synthesizers::generate_clap_909();
    samples.push(("clap", clap_data));
    let clap_sample = make_sample("acid_clap", sample_rate);
    writer.add_sample_header_only(clap_sample);
    writer.add_instrument(make_instrument("acid_clap", 2));

    // 3. Hi-hat closed
    let hh_closed_data = synthesizers::generate_hat_909_closed();
    samples.push(("hat_closed", hh_closed_data));
    let hh_sample = make_sample("acid_hat_closed", sample_rate);
    writer.add_sample_header_only(hh_sample);
    writer.add_instrument(make_instrument("acid_hat_closed", 3));

    // 4. Hi-hat open
    let hh_open_data = synthesizers::generate_hat_909_open();
    samples.push(("hat_open", hh_open_data));
    let hho_sample = make_sample("acid_hat_open", sample_rate);
    writer.add_sample_header_only(hho_sample);
    writer.add_instrument(make_instrument("acid_hat_open", 4));

    // 5. TB-303 Bass - THE STAR
    let bass_303_data = synthesizers::generate_bass_303();
    samples.push(("303", bass_303_data));
    let bass_sample = make_sample("acid_303", 139996); // E2 @ 82.41 Hz
    writer.add_sample_header_only(bass_sample);
    writer.add_instrument(make_instrument("acid_303", 5));

    // 6. Acid Pad
    let pad_data = synthesizers::generate_pad_acid();
    samples.push(("pad", pad_data));
    let pad_sample = make_sample("acid_pad", 69998); // E3 @ 164.81 Hz
    writer.add_sample_header_only(pad_sample);
    writer.add_instrument(make_instrument("acid_pad", 6));

    // 7. Acid Stab
    let stab_data = synthesizers::generate_stab_acid();
    samples.push(("stab", stab_data));
    let stab_sample = make_sample("acid_stab", 34993); // E4 @ 329.63 Hz
    writer.add_sample_header_only(stab_sample);
    writer.add_instrument(make_instrument("acid_stab", 7));

    // 8. TB-303 Squelch (higher resonance for climax)
    let bass_303_squelch_data = synthesizers::generate_bass_303_squelch();
    samples.push(("303_squelch", bass_303_squelch_data));
    let bass_squelch_sample = make_sample("acid_303_squelch", 139996); // E2 @ 82.41 Hz
    writer.add_sample_header_only(bass_squelch_sample);
    writer.add_instrument(make_instrument("acid_303_squelch", 8));

    // 9. Riser (white noise sweep for builds)
    let riser_data = synthesizers::generate_riser_acid();
    samples.push(("riser", riser_data));
    let riser_sample = make_sample("acid_riser", 96129); // ~120 Hz (starting freq)
    writer.add_sample_header_only(riser_sample);
    writer.add_instrument(make_instrument("acid_riser", 9));

    // 10. Atmosphere (subtle texture layer)
    let atmosphere_data = synthesizers::generate_atmosphere_acid();
    samples.push(("atmosphere", atmosphere_data));
    let atmosphere_sample = make_sample("acid_atmosphere", 209762); // A1 @ 55.0 Hz
    writer.add_sample_header_only(atmosphere_sample);
    writer.add_instrument(make_instrument("acid_atmosphere", 10));

    // 11. Crash 909
    let crash_data = synthesizers::generate_crash_909();
    samples.push(("crash", crash_data));
    let crash_sample = make_sample("acid_crash", sample_rate);
    writer.add_sample_header_only(crash_sample);
    writer.add_instrument(make_instrument("acid_crash", 11));

    // ========================================================================
    // Create 12 patterns for ~60-70 second track
    // ========================================================================

    // Pattern 0: Intro
    let pat_intro = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat_intro);

    // Pattern 1: Groove A (main 303 pattern)
    let pat_groove_a = writer.add_pattern(64);
    build_groove_a_pattern(&mut writer, pat_groove_a);

    // Pattern 2: Build (filter opens)
    let pat_build = writer.add_pattern(64);
    build_build_pattern(&mut writer, pat_build);

    // Pattern 3: Breakdown
    let pat_breakdown = writer.add_pattern(64);
    build_breakdown_pattern(&mut writer, pat_breakdown);

    // Pattern 4: Drop (maximum squelch)
    let pat_drop = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop);

    // Pattern 5: Outro
    let pat_outro = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat_outro);

    // Pattern 6: Groove B (variation)
    let pat_groove_b = writer.add_pattern(64);
    build_groove_b_pattern(&mut writer, pat_groove_b);

    // Pattern 7: Build Intense (with riser)
    let pat_build_intense = writer.add_pattern(64);
    build_build_intense_pattern(&mut writer, pat_build_intense);

    // Pattern 8: Drop Variation (with portamento)
    let pat_drop_variation = writer.add_pattern(64);
    build_drop_variation_pattern(&mut writer, pat_drop_variation);

    // Pattern 9: Breakdown Deep (atmospheric)
    let pat_breakdown_deep = writer.add_pattern(64);
    build_breakdown_deep_pattern(&mut writer, pat_breakdown_deep);

    // Pattern 10: Drop B (B minor)
    let pat_drop_b = writer.add_pattern(64);
    build_drop_b_pattern(&mut writer, pat_drop_b);

    // Pattern 11: Drop B Intense (B minor with squelch)
    let pat_drop_b_intense = writer.add_pattern(64);
    build_drop_b_intense_pattern(&mut writer, pat_drop_b_intense);

    // ========================================================================
    // Order table: Enhanced acid techno journey (18 entries = 72 bars)
    // ========================================================================
    writer.set_orders(&[
        pat_intro,          // 0: Intro (4 bars)
        pat_groove_a,       // 1: Main groove (4 bars)
        pat_groove_a,       // 2: Establish groove (4 bars)
        pat_groove_b,       // 3: Variation (4 bars)
        pat_build,          // 4: Building tension (4 bars)
        pat_build_intense,  // 5: Peak tension with riser (4 bars)
        pat_drop,           // 6: First drop E minor (4 bars)
        pat_drop_variation, // 7: Drop with slides (4 bars)
        pat_breakdown,      // 8: Breathing room (4 bars)
        pat_groove_b,       // 9: Return to groove (4 bars)
        pat_build_intense,  // 10: Build to climax (4 bars)
        pat_drop_b,         // 11: Drop B minor (4 bars)
        pat_drop_b_intense, // 12: Maximum energy (4 bars)
        pat_breakdown_deep, // 13: Atmospheric (4 bars)
        pat_groove_a,       // 14: Return to familiar (4 bars)
        pat_build,          // 15: Final build (4 bars)
        pat_drop_variation, // 16: Final drop (4 bars)
        pat_outro,          // 17: Wind down (4 bars)
    ]);

    // Set song message
    writer.set_message(
        "Nether Acid - Acid Techno @ 130 BPM\n\
         TB-303 Resonant Filter Showcase\n\
         Generated by gen-tracker-demo-it\n\
         Nethercore Project",
    );

    (writer.write(), samples)
}

/// Generate embedded Nether Acid IT file (with sample data)
pub fn generate_acid_it_embedded() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
    let mut writer = ItWriter::new("Nether Acid");

    // Set up module parameters
    writer.set_channels(8);
    writer.set_speed(6); // Standard speed
    writer.set_tempo(130); // Classic acid house tempo
    writer.set_global_volume(128);
    writer.set_mix_volume(64);
    writer.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES);

    // Generate and collect samples
    let mut samples = Vec::new();
    let sample_rate = 22050;

    // 1. Kick 909
    let kick_data = synthesizers::generate_kick_909();
    samples.push(("kick", kick_data));
    let kick_sample = make_sample("acid_kick", sample_rate);
    writer.add_sample(kick_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_kick", 1));

    // 2. Clap 909
    let clap_data = synthesizers::generate_clap_909();
    samples.push(("clap", clap_data));
    let clap_sample = make_sample("acid_clap", sample_rate);
    writer.add_sample(clap_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_clap", 2));

    // 3. Hi-hat closed
    let hh_closed_data = synthesizers::generate_hat_909_closed();
    samples.push(("hat_closed", hh_closed_data));
    let hh_sample = make_sample("acid_hat_closed", sample_rate);
    writer.add_sample(hh_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_hat_closed", 3));

    // 4. Hi-hat open
    let hh_open_data = synthesizers::generate_hat_909_open();
    samples.push(("hat_open", hh_open_data));
    let hho_sample = make_sample("acid_hat_open", sample_rate);
    writer.add_sample(hho_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_hat_open", 4));

    // 5. TB-303 Bass - THE STAR
    let bass_303_data = synthesizers::generate_bass_303();
    samples.push(("303", bass_303_data));
    let bass_sample = make_sample("acid_303", 139996); // E2 @ 82.41 Hz
    writer.add_sample(bass_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_303", 5));

    // 6. Acid Pad
    let pad_data = synthesizers::generate_pad_acid();
    samples.push(("pad", pad_data));
    let pad_sample = make_sample("acid_pad", 69998); // E3 @ 164.81 Hz
    writer.add_sample(pad_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_pad", 6));

    // 7. Acid Stab
    let stab_data = synthesizers::generate_stab_acid();
    samples.push(("stab", stab_data));
    let stab_sample = make_sample("acid_stab", 34993); // E4 @ 329.63 Hz
    writer.add_sample(stab_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_stab", 7));

    // 8. TB-303 Squelch (higher resonance for climax)
    let bass_303_squelch_data = synthesizers::generate_bass_303_squelch();
    samples.push(("303_squelch", bass_303_squelch_data));
    let bass_squelch_sample = make_sample("acid_303_squelch", 139996); // E2 @ 82.41 Hz
    writer.add_sample(bass_squelch_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_303_squelch", 8));

    // 9. Riser (white noise sweep for builds)
    let riser_data = synthesizers::generate_riser_acid();
    samples.push(("riser", riser_data));
    let riser_sample = make_sample("acid_riser", 96129); // ~120 Hz (starting freq)
    writer.add_sample(riser_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_riser", 9));

    // 10. Atmosphere (subtle texture layer)
    let atmosphere_data = synthesizers::generate_atmosphere_acid();
    samples.push(("atmosphere", atmosphere_data));
    let atmosphere_sample = make_sample("acid_atmosphere", 209762); // A1 @ 55.0 Hz
    writer.add_sample(atmosphere_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_atmosphere", 10));

    // 11. Crash 909
    let crash_data = synthesizers::generate_crash_909();
    samples.push(("crash", crash_data));
    let crash_sample = make_sample("acid_crash", sample_rate);
    writer.add_sample(crash_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("acid_crash", 11));

    // ========================================================================
    // Create 12 patterns for ~60-70 second track
    // ========================================================================

    // Pattern 0: Intro
    let pat_intro = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat_intro);

    // Pattern 1: Groove A (main 303 pattern)
    let pat_groove_a = writer.add_pattern(64);
    build_groove_a_pattern(&mut writer, pat_groove_a);

    // Pattern 2: Build (filter opens)
    let pat_build = writer.add_pattern(64);
    build_build_pattern(&mut writer, pat_build);

    // Pattern 3: Breakdown
    let pat_breakdown = writer.add_pattern(64);
    build_breakdown_pattern(&mut writer, pat_breakdown);

    // Pattern 4: Drop (maximum squelch)
    let pat_drop = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop);

    // Pattern 5: Outro
    let pat_outro = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat_outro);

    // Pattern 6: Groove B (variation)
    let pat_groove_b = writer.add_pattern(64);
    build_groove_b_pattern(&mut writer, pat_groove_b);

    // Pattern 7: Build Intense (with riser)
    let pat_build_intense = writer.add_pattern(64);
    build_build_intense_pattern(&mut writer, pat_build_intense);

    // Pattern 8: Drop Variation (with portamento)
    let pat_drop_variation = writer.add_pattern(64);
    build_drop_variation_pattern(&mut writer, pat_drop_variation);

    // Pattern 9: Breakdown Deep (atmospheric)
    let pat_breakdown_deep = writer.add_pattern(64);
    build_breakdown_deep_pattern(&mut writer, pat_breakdown_deep);

    // Pattern 10: Drop B (B minor)
    let pat_drop_b = writer.add_pattern(64);
    build_drop_b_pattern(&mut writer, pat_drop_b);

    // Pattern 11: Drop B Intense (B minor with squelch)
    let pat_drop_b_intense = writer.add_pattern(64);
    build_drop_b_intense_pattern(&mut writer, pat_drop_b_intense);

    // ========================================================================
    // Order table: Enhanced acid techno journey (18 entries = 72 bars)
    // ========================================================================
    writer.set_orders(&[
        pat_intro,          // 0: Intro (4 bars)
        pat_groove_a,       // 1: Main groove (4 bars)
        pat_groove_a,       // 2: Establish groove (4 bars)
        pat_groove_b,       // 3: Variation (4 bars)
        pat_build,          // 4: Building tension (4 bars)
        pat_build_intense,  // 5: Peak tension with riser (4 bars)
        pat_drop,           // 6: First drop E minor (4 bars)
        pat_drop_variation, // 7: Drop with slides (4 bars)
        pat_breakdown,      // 8: Breathing room (4 bars)
        pat_groove_b,       // 9: Return to groove (4 bars)
        pat_build_intense,  // 10: Build to climax (4 bars)
        pat_drop_b,         // 11: Drop B minor (4 bars)
        pat_drop_b_intense, // 12: Maximum energy (4 bars)
        pat_breakdown_deep, // 13: Atmospheric (4 bars)
        pat_groove_a,       // 14: Return to familiar (4 bars)
        pat_build,          // 15: Final build (4 bars)
        pat_drop_variation, // 16: Final drop (4 bars)
        pat_outro,          // 17: Wind down (4 bars)
    ]);

    // Set song message
    writer.set_message(
        "Nether Acid - Acid Techno @ 130 BPM\n\
         TB-303 Resonant Filter Showcase\n\
         Generated by gen-tracker-demo-it\n\
         Nethercore Project",
    );

    (writer.write(), samples)
}
