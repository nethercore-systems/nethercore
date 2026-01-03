//! Nether Acid - Acid Techno IT generator
//!
//! 130 BPM, E minor, 8 channels
//! Duration: ~60-70 seconds (18 patterns × 4 bars)
//! Features: TB-303 acid bassline with resonant filter, 909 drums, IT effects (portamento, filter automation, panning)

use super::{make_instrument, make_sample};
use crate::synthesizers;
use nether_it::{ItFlags, ItNote, ItWriter};

// ============================================================================
// Note Constants - E minor (E F# G A B C D)
// ============================================================================

// Octave 2 (bass range)
const E2: u8 = 28;
const _FS2: u8 = 30;
const G2: u8 = 31;
const A2: u8 = 33;
const B2: u8 = 35;
const _C3: u8 = 36;
const D3: u8 = 38;

// Octave 3 (upper bass / pad range)
const E3: u8 = 40;
const FS3: u8 = 42; // For B minor
const _G3: u8 = 43;
const A3: u8 = 45; // For B minor
const _B3: u8 = 47;
const _C4: u8 = 48;
const _D4: u8 = 50;

// Octave 4 (lead range)
const _E4: u8 = 52;

// Octave 5 (for drums - no pitch shift)
const C5: u8 = 60;

// ============================================================================
// Instrument and Channel Constants
// ============================================================================

// Instruments (1-indexed for IT format)
const INST_KICK: u8 = 1;
const INST_CLAP: u8 = 2;
const INST_HH_CLOSED: u8 = 3;
const INST_HH_OPEN: u8 = 4;
const INST_BASS_303: u8 = 5;
const INST_PAD: u8 = 6;
const INST_STAB: u8 = 7;
const INST_BASS_303_SQUELCH: u8 = 8; // Higher resonance for climax
const INST_RISER: u8 = 9;             // White noise sweep for builds
const INST_ATMOSPHERE: u8 = 10;       // Subtle texture layer
const INST_CRASH: u8 = 11;            // Cymbal crash for transitions

// Channels (0-indexed)
const CH_KICK: u8 = 0;
const CH_CLAP: u8 = 1;
const CH_HIHAT: u8 = 2;
const CH_HIHAT_OPEN: u8 = 3;
const CH_303: u8 = 4;
const CH_PAD: u8 = 5;
const CH_STAB: u8 = 6;
const CH_FX: u8 = 7; // Used for risers, crashes, and FX

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
        pat_intro,                                    // 0: Intro (4 bars)
        pat_groove_a,                                 // 1: Main groove (4 bars)
        pat_groove_a,                                 // 2: Establish groove (4 bars)
        pat_groove_b,                                 // 3: Variation (4 bars)
        pat_build,                                    // 4: Building tension (4 bars)
        pat_build_intense,                            // 5: Peak tension with riser (4 bars)
        pat_drop,                                     // 6: First drop E minor (4 bars)
        pat_drop_variation,                           // 7: Drop with slides (4 bars)
        pat_breakdown,                                // 8: Breathing room (4 bars)
        pat_groove_b,                                 // 9: Return to groove (4 bars)
        pat_build_intense,                            // 10: Build to climax (4 bars)
        pat_drop_b,                                   // 11: Drop B minor (4 bars)
        pat_drop_b_intense,                           // 12: Maximum energy (4 bars)
        pat_breakdown_deep,                           // 13: Atmospheric (4 bars)
        pat_groove_a,                                 // 14: Return to familiar (4 bars)
        pat_build,                                    // 15: Final build (4 bars)
        pat_drop_variation,                           // 16: Final drop (4 bars)
        pat_outro,                                    // 17: Wind down (4 bars)
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
        pat_intro,                                    // 0: Intro (4 bars)
        pat_groove_a,                                 // 1: Main groove (4 bars)
        pat_groove_a,                                 // 2: Establish groove (4 bars)
        pat_groove_b,                                 // 3: Variation (4 bars)
        pat_build,                                    // 4: Building tension (4 bars)
        pat_build_intense,                            // 5: Peak tension with riser (4 bars)
        pat_drop,                                     // 6: First drop E minor (4 bars)
        pat_drop_variation,                           // 7: Drop with slides (4 bars)
        pat_breakdown,                                // 8: Breathing room (4 bars)
        pat_groove_b,                                 // 9: Return to groove (4 bars)
        pat_build_intense,                            // 10: Build to climax (4 bars)
        pat_drop_b,                                   // 11: Drop B minor (4 bars)
        pat_drop_b_intense,                           // 12: Maximum energy (4 bars)
        pat_breakdown_deep,                           // 13: Atmospheric (4 bars)
        pat_groove_a,                                 // 14: Return to familiar (4 bars)
        pat_build,                                    // 15: Final build (4 bars)
        pat_drop_variation,                           // 16: Final drop (4 bars)
        pat_outro,                                    // 17: Wind down (4 bars)
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

// ============================================================================
// Drum Helper Functions
// ============================================================================

/// 4-on-the-floor kick (classic techno)
fn add_kick_4x4(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Every beat
    for beat in 0..4 {
        writer.set_note(pat, base + beat * 4, CH_KICK, ItNote::play_note(C5, INST_KICK, 64));
    }
}

/// Claps on 2 and 4 (backbeat)
fn add_claps(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 2 (row 4)
    writer.set_note(pat, base + 4, CH_CLAP, ItNote::play_note(C5, INST_CLAP, 64));
    // Beat 4 (row 12)
    writer.set_note(pat, base + 12, CH_CLAP, ItNote::play_note(C5, INST_CLAP, 64));
}

/// 16th note hi-hats (every row) with stereo panning
fn add_hihat_16ths(writer: &mut ItWriter, pat: u8, bar: u16, use_opens: bool) {
    let base = bar * 16;
    for row in 0..16 {
        let vel = if row % 4 == 0 { 50 } else { 35 }; // Accents on beats

        // Alternate panning for stereo width (8xx effect: 0x10=left, 0x30=right)
        let pan = if row % 2 == 0 { 0x10 } else { 0x30 };

        // Open hats on off-beats for groove
        if use_opens && (row == 6 || row == 14) {
            writer.set_note(pat, base + row, CH_HIHAT_OPEN, ItNote::play_note(C5, INST_HH_OPEN, vel + 5).with_effect(0x08, pan));
        } else {
            writer.set_note(pat, base + row, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, vel).with_effect(0x08, pan));
        }
    }
}

/// 8th note hi-hats (every 2 rows)
fn add_hihat_8ths(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    for row in 0..8 {
        let vel = if row % 2 == 0 { 48 } else { 32 };
        writer.set_note(pat, base + row * 2, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, vel));
    }
}

// ============================================================================
// TB-303 Pattern Helpers
// ============================================================================

/// Main 303 pattern - classic acid sequence with accents and slides
fn add_303_main_pattern(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;

    // Classic 16th note acid pattern
    // Notes: E-G-A-B pattern with octave jumps
    // Accents (vol 64) trigger filter envelope, no accents (vol 40) stay flat

    let notes = [
        (0, E2, 64),   // Accent - filter opens
        (4, G2, 40),   // No accent
        (8, A2, 64),   // Accent
        (10, B2, 40),  // No accent - quick hit
        (12, E3, 64),  // Accent - octave jump
        (16, D3, 40),  // No accent
        (20, B2, 40),  // No accent
        (24, A2, 64),  // Accent
        (28, G2, 40),  // No accent
        (32, E2, 64),  // Accent
        (36, G2, 40),  // No accent
        (40, B2, 64),  // Accent
        (44, D3, 64),  // Accent
        (48, E3, 64),  // Accent
        (52, B2, 40),  // No accent
    ];

    for (offset, note, vel) in notes {
        writer.set_note(pat, base + offset, CH_303, ItNote::play_note(note, INST_BASS_303, vel));
    }
}

/// Simple 303 pattern for intro/outro
fn add_303_simple(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;

    // Just root notes on beats
    writer.set_note(pat, base, CH_303, ItNote::play_note(E2, INST_BASS_303, 50));
    writer.set_note(pat, base + 8, CH_303, ItNote::play_note(E2, INST_BASS_303, 50));
}

// ============================================================================
// Pattern Builder Functions
// ============================================================================

fn build_intro_pattern(writer: &mut ItWriter, pat: u8) {
    // Bars 0-1: Kick only
    add_kick_4x4(writer, pat, 0);
    add_kick_4x4(writer, pat, 1);

    // Bars 2-3: Add hats and 303 enters
    add_kick_4x4(writer, pat, 2);
    add_hihat_8ths(writer, pat, 2);
    add_303_simple(writer, pat, 2);

    add_kick_4x4(writer, pat, 3);
    add_hihat_8ths(writer, pat, 3);
    add_303_simple(writer, pat, 3);
}

fn build_groove_a_pattern(writer: &mut ItWriter, pat: u8) {
    // Full groove with main 303 pattern
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true); // With open hats
        add_303_main_pattern(writer, pat, bar);
    }

    // Add pad on bar 0 and 2 for warmth
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(E3, INST_PAD, 45));
    writer.set_note(pat, 32, CH_PAD, ItNote::play_note(E3, INST_PAD, 45));
}

fn build_build_pattern(writer: &mut ItWriter, pat: u8) {
    // Building energy with progressive filter opening (Zxx effect)
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // 303 pattern with filter automation - gradually open filter (Zxx: 0x1A effect)
        let base = bar * 16;
        let notes = [
            (0, E2, 64),   (4, G2, 40),   (8, A2, 64),
            (10, B2, 40),  (12, E3, 64),  (16, D3, 40),
            (20, B2, 40),  (24, A2, 64),  (28, G2, 40),
            (32, E2, 64),  (36, G2, 40),  (40, B2, 64),
            (44, D3, 64),  (48, E3, 64),  (52, B2, 40),
        ];

        // Filter cutoff increases each bar (Z40 → Z50 → Z60 → Z70)
        let cutoff = 0x40 + (bar as u8) * 0x10;

        for (offset, note, vel) in &notes {
            // Add filter automation to first note of each bar
            let note_obj = if *offset == 0 {
                ItNote::play_note(*note, INST_BASS_303, *vel).with_effect(0x1A, cutoff) // Zxx filter cutoff
            } else {
                ItNote::play_note(*note, INST_BASS_303, *vel)
            };
            writer.set_note(pat, base + offset, CH_303, note_obj);
        }
    }

    // Add chord stabs on bars 2-3
    writer.set_note(pat, 32, CH_STAB, ItNote::play_note(E3, INST_STAB, 55));
    writer.set_note(pat, 40, CH_STAB, ItNote::play_note(E3, INST_STAB, 55));
    writer.set_note(pat, 48, CH_STAB, ItNote::play_note(E3, INST_STAB, 60));
    writer.set_note(pat, 56, CH_STAB, ItNote::play_note(E3, INST_STAB, 60));
}

fn build_breakdown_pattern(writer: &mut ItWriter, pat: u8) {
    // Sparse - just kick on beat 1 and simple 303
    for bar in 0..4 {
        writer.set_note(pat, bar * 16, CH_KICK, ItNote::play_note(C5, INST_KICK, 60));
        add_303_simple(writer, pat, bar);
    }

    // Pad sustains throughout
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(E3, INST_PAD, 50));
}

fn build_drop_pattern(writer: &mut ItWriter, pat: u8) {
    // MAXIMUM ENERGY - all accents on 303, full drums
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // 303 with MORE accents (more filter action)
        let base = bar * 16;
        let notes = [
            (0, E2, 64),   // All accents!
            (4, G2, 64),
            (8, A2, 64),
            (10, B2, 64),
            (12, E3, 64),
            (16, D3, 64),
            (20, B2, 64),
            (24, A2, 64),
            (28, G2, 64),
            (32, E2, 64),
            (36, G2, 64),
            (40, B2, 64),
            (44, D3, 64),
            (48, E3, 64),
            (52, B2, 64),
        ];

        for (offset, note, vel) in notes {
            writer.set_note(pat, base + offset, CH_303, ItNote::play_note(note, INST_BASS_303, vel));
        }
    }

    // Stabs for extra punch
    writer.set_note(pat, 0, CH_STAB, ItNote::play_note(E3, INST_STAB, 60));
    writer.set_note(pat, 32, CH_STAB, ItNote::play_note(E3, INST_STAB, 60));
}

fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
    // Wind down - kick continues, everything else fades
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        if bar < 2 {
            add_hihat_8ths(writer, pat, bar);
            add_303_simple(writer, pat, bar);
        }
    }

    // Final pad note
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(E3, INST_PAD, 40));
}

// ============================================================================
// New Pattern Builders for Enhanced Acid Track
// ============================================================================

fn build_groove_b_pattern(writer: &mut ItWriter, pat: u8) {
    // Variation groove with different 303 pattern
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // Different 303 sequence - A minor feel (A-C-E-G instead of E-G-A-B)
        let base = bar * 16;
        let notes = [
            (0, A2, 64),   // Accent
            (4, E2, 40),   // No accent
            (8, G2, 64),   // Accent
            (10, A2, 40),  // No accent
            (12, E3, 64),  // Accent - octave jump
            (16, D3, 40),  // No accent
            (20, A2, 40),  // No accent
            (24, G2, 64),  // Accent
            (28, E2, 40),  // No accent
            (32, A2, 64),  // Accent
            (36, E2, 40),  // No accent
            (40, G2, 64),  // Accent
            (44, D3, 64),  // Accent
            (48, E3, 64),  // Accent
            (52, A2, 40),  // No accent
        ];

        for (offset, note, vel) in notes {
            writer.set_note(pat, base + offset, CH_303, ItNote::play_note(note, INST_BASS_303, vel));
        }
    }

    // Atmosphere layer for subtle texture
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(A2, INST_ATMOSPHERE, 30));
}

fn build_build_intense_pattern(writer: &mut ItWriter, pat: u8) {
    // High-energy build with riser and snare roll
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);
        add_303_main_pattern(writer, pat, bar);
    }

    // Riser sweeps across all 4 bars
    writer.set_note(pat, 0, CH_FX, ItNote::play_note(E3, INST_RISER, 50));

    // Crash at the end for transition
    writer.set_note(pat, 63, CH_FX, ItNote::play_note(C5, INST_CRASH, 55));

    // Stabs getting more intense
    writer.set_note(pat, 32, CH_STAB, ItNote::play_note(E3, INST_STAB, 58));
    writer.set_note(pat, 40, CH_STAB, ItNote::play_note(E3, INST_STAB, 60));
    writer.set_note(pat, 48, CH_STAB, ItNote::play_note(E3, INST_STAB, 62));
    writer.set_note(pat, 56, CH_STAB, ItNote::play_note(E3, INST_STAB, 64));
}

fn build_drop_variation_pattern(writer: &mut ItWriter, pat: u8) {
    // Same as drop but with portamento slides (Gxx effect)
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // 303 with slides (Gxx portamento effect = 0x07)
        let base = bar * 16;
        let notes = [
            (0, E2, 64),
            (4, G2, 64),
            // Slide from G2 to A2
            (8, A2, 64),
            (10, B2, 64),
            // Slide from B2 to E3
            (12, E3, 64),
            (16, D3, 64),
            (20, B2, 64),
            // Slide from B2 to A2
            (24, A2, 64),
            (28, G2, 64),
            (32, E2, 64),
            (36, G2, 64),
            // Slide from G2 to B2
            (40, B2, 64),
            (44, D3, 64),
            (48, E3, 64),
            (52, B2, 64),
        ];

        for (i, (offset, note, vel)) in notes.iter().enumerate() {
            // Add portamento on selected notes for slides
            let note_obj = if i == 2 || i == 4 || i == 7 || i == 11 {
                ItNote::play_note(*note, INST_BASS_303, *vel).with_effect(0x07, 0x20) // Gxx portamento
            } else {
                ItNote::play_note(*note, INST_BASS_303, *vel)
            };
            writer.set_note(pat, base + offset, CH_303, note_obj);
        }
    }

    // Crash for extra punch
    writer.set_note(pat, 0, CH_FX, ItNote::play_note(C5, INST_CRASH, 60));
}

fn build_breakdown_deep_pattern(writer: &mut ItWriter, pat: u8) {
    // Atmospheric breakdown with low bass
    for bar in 0..4 {
        // Only kick on beat 1
        writer.set_note(pat, bar * 16, CH_KICK, ItNote::play_note(C5, INST_KICK, 55));

        // Very simple low 303
        let base = bar * 16;
        writer.set_note(pat, base, CH_303, ItNote::play_note(E2, INST_BASS_303, 45));
        writer.set_note(pat, base + 8, CH_303, ItNote::play_note(G2, INST_BASS_303, 40));
    }

    // Atmosphere layer throughout for texture
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(E2, INST_ATMOSPHERE, 40));
    writer.set_note(pat, 32, CH_PAD, ItNote::play_note(G2, INST_ATMOSPHERE, 40));

    // Pad sustains for warmth
    writer.set_note(pat, 0, CH_STAB, ItNote::play_note(E3, INST_PAD, 35));
}

fn build_drop_b_pattern(writer: &mut ItWriter, pat: u8) {
    // Drop in B minor for harmonic contrast
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // 303 in B minor (B-D-E-F#-A)
        let base = bar * 16;
        let notes = [
            (0, B2, 64),   // Accent
            (4, D3, 64),   // Accent
            (8, E3, 64),   // Accent
            (10, FS3, 64), // Accent (F# for B minor)
            (12, A3, 64),  // Accent - octave jump
            (16, FS3, 64), // Accent
            (20, E3, 64),  // Accent
            (24, D3, 64),  // Accent
            (28, B2, 64),  // Accent
            (32, D3, 64),  // Accent
            (36, E3, 64),  // Accent
            (40, FS3, 64), // Accent
            (44, A3, 64),  // Accent
            (48, B2, 64),  // Accent
            (52, D3, 64),  // Accent
        ];

        for (offset, note, vel) in notes {
            writer.set_note(pat, base + offset, CH_303, ItNote::play_note(note, INST_BASS_303, vel));
        }
    }

    // Crash for section transition
    writer.set_note(pat, 0, CH_FX, ItNote::play_note(C5, INST_CRASH, 62));
}

fn build_drop_b_intense_pattern(writer: &mut ItWriter, pat: u8) {
    // Maximum energy - B minor with squelch bass
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // Use the higher-resonance 303 squelch for maximum energy
        let base = bar * 16;
        let notes = [
            (0, B2, 64),
            (4, D3, 64),
            (8, E3, 64),
            (10, FS3, 64),
            (12, A3, 64),
            (16, FS3, 64),
            (20, E3, 64),
            (24, D3, 64),
            (28, B2, 64),
            (32, D3, 64),
            (36, E3, 64),
            (40, FS3, 64),
            (44, A3, 64),
            (48, B2, 64),
            (52, D3, 64),
        ];

        for (offset, note, vel) in notes {
            writer.set_note(pat, base + offset, CH_303, ItNote::play_note(note, INST_BASS_303_SQUELCH, vel));
        }
    }

    // Crashes for massive impact
    writer.set_note(pat, 0, CH_FX, ItNote::play_note(C5, INST_CRASH, 64));
    writer.set_note(pat, 32, CH_FX, ItNote::play_note(C5, INST_CRASH, 64));

    // Stabs for extra punch
    writer.set_note(pat, 0, CH_STAB, ItNote::play_note(B2, INST_STAB, 64));
    writer.set_note(pat, 32, CH_STAB, ItNote::play_note(B2, INST_STAB, 64));
}
