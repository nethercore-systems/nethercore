//! Nether Storm - DnB/Action IT generator (Hero Quality)
//!
//! 174 BPM, F minor (Phrygian), 16 channels
//! Duration: ~90 seconds with seamless loop
//! Features: Ghost notes, humanized drums, break layers, rebalanced bass

use super::{make_instrument, make_instrument_continue, make_sample};
use crate::synthesizers;
use nether_it::{ItFlags, ItNote, ItWriter};

// ============================================================================
// Note Constants - F minor/Phrygian (F Gb Ab Bb C Db Eb)
// ============================================================================

// Octave 1 (sub bass range)
const F1: u8 = 17;
const _GB1: u8 = 18;
const _AB1: u8 = 20;
const _BB1: u8 = 22;
const C2: u8 = 24;
const DB1: u8 = 13;
const EB1: u8 = 15;

// Octave 2 (main bass range)
const F2: u8 = 29;
const _GB2: u8 = 30;
const _AB2: u8 = 32;
const _BB2: u8 = 34;
const C3: u8 = 36;
const DB2: u8 = 25;
const EB2: u8 = 27;

// Octave 3 (upper bass / pad range)
const F3: u8 = 41;
const _GB3: u8 = 42;
const AB3: u8 = 44;
const _BB3: u8 = 46;
const _C4: u8 = 48;
const DB3: u8 = 37;
const _EB3: u8 = 39;

// Octave 4 (lead range)
const F4: u8 = 53;
const _GB4: u8 = 54;
const AB4: u8 = 56;
const _BB4: u8 = 58;
const C5: u8 = 60;
const _DB4: u8 = 49;
const EB4: u8 = 51;

// Octave 5 (high lead range)
const F5: u8 = 65;
const EB5: u8 = 63;

// ============================================================================
// Instrument and Channel Constants
// ============================================================================

// Instruments (1-indexed for IT format)
const INST_KICK: u8 = 1;
const INST_SNARE: u8 = 2;
const INST_HH_CLOSED: u8 = 3;
const INST_HH_OPEN: u8 = 4;
const INST_BREAK: u8 = 5;
const INST_CYMBAL: u8 = 6;
const INST_SUB: u8 = 7;
const INST_REESE: u8 = 8;
const INST_WOBBLE: u8 = 9;
const INST_PAD: u8 = 10;
const INST_STAB: u8 = 11;
const INST_LEAD: u8 = 12;
const INST_RISER: u8 = 13;
const INST_IMPACT: u8 = 14;
const INST_ATMOS: u8 = 15;

// Channels (0-indexed)
const CH_KICK: u8 = 0;
const CH_SNARE: u8 = 1;
const CH_HIHAT: u8 = 2;
const CH_HIHAT_OPEN: u8 = 3;
const CH_BREAK: u8 = 4;
const CH_CYMBAL: u8 = 5;
const CH_SUB: u8 = 6;
const CH_REESE: u8 = 7;
const CH_WOBBLE: u8 = 8;
const CH_PAD: u8 = 9;
const CH_STAB: u8 = 10;
const CH_LEAD: u8 = 11;
const CH_RISER: u8 = 12;
const CH_IMPACT: u8 = 13;
const CH_ATMOS: u8 = 14;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, PartialEq)]
enum Section {
    DropA,
    DropB,
}

#[derive(Clone, Copy)]
enum BreakStyle {
    None,
    Ghost,
    Accent,
    Fill,
}

// ============================================================================
// Main Generator
// ============================================================================

/// Generate the Nether Storm IT file (Hero Quality)
// ============================================================================
// Main Generators
// ============================================================================

/// Generate stripped IT file (no sample data, for ROM/external samples)
pub fn generate_storm_it_stripped() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
    let mut writer = ItWriter::new("Nether Storm");

    // Set up module parameters
    writer.set_channels(16);
    writer.set_speed(3); // Fast speed for DnB
    writer.set_tempo(174);
    writer.set_global_volume(128);
    writer.set_mix_volume(80);
    writer.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES);

    // Generate and collect samples
    let mut samples = Vec::new();
    let sample_rate = 22050;

    // 1. Kick - transient + sine sweep
    let kick_data = synthesizers::generate_kick_dnb();
    samples.push(("kick_dnb", kick_data));
    let kick_sample = make_sample("kick_dnb", sample_rate);
    writer.add_sample_header_only(kick_sample);
    writer.add_instrument(make_instrument("kick_dnb", 1));

    // 2. Snare - layered
    let snare_data = synthesizers::generate_snare_dnb();
    samples.push(("snare_dnb", snare_data));
    let snare_sample = make_sample("snare_dnb", sample_rate);
    writer.add_sample_header_only(snare_sample);
    writer.add_instrument(make_instrument("snare_dnb", 2));

    // 3. Hihat closed
    let hh_closed_data = synthesizers::generate_hihat_closed();
    samples.push(("hh_closed", hh_closed_data));
    let hh_sample = make_sample("hh_closed", sample_rate);
    writer.add_sample_header_only(hh_sample);
    writer.add_instrument(make_instrument("hh_closed", 3));

    // 4. Hihat open
    let hh_open_data = synthesizers::generate_hihat_open();
    samples.push(("hh_open", hh_open_data));
    let hho_sample = make_sample("hh_open", sample_rate);
    writer.add_sample_header_only(hho_sample);
    writer.add_instrument(make_instrument("hh_open", 4));

    // 5. Break slice
    let break_data = synthesizers::generate_break_slice();
    samples.push(("break_slice", break_data));
    let break_sample = make_sample("break_slice", sample_rate);
    writer.add_sample_header_only(break_sample);
    writer.add_instrument(make_instrument("break_slice", 5));

    // 6. Cymbal (storm-specific)
    let cymbal_data = synthesizers::generate_cymbal();
    samples.push(("cymbal_storm", cymbal_data));
    let cymbal_sample = make_sample("cymbal_storm", sample_rate);
    writer.add_sample_header_only(cymbal_sample);
    writer.add_instrument(make_instrument("cymbal_storm", 6));

    // 7. Sub bass
    let sub_data = synthesizers::generate_bass_sub_dnb();
    samples.push(("sub_bass", sub_data));
    let sub_sample = make_sample("sub_bass", sample_rate);
    writer.add_sample_header_only(sub_sample);
    writer.add_instrument(make_instrument("sub_bass", 7));

    // 8. Reese bass
    let reese_data = synthesizers::generate_bass_reese();
    samples.push(("reese_bass", reese_data));
    let reese_sample = make_sample("reese_bass", sample_rate);
    writer.add_sample_header_only(reese_sample);
    writer.add_instrument(make_instrument("reese_bass", 8));

    // 9. Wobble bass
    let wobble_data = synthesizers::generate_bass_wobble();
    samples.push(("wobble_bass", wobble_data));
    let wobble_sample = make_sample("wobble_bass", sample_rate);
    writer.add_sample_header_only(wobble_sample);
    writer.add_instrument(make_instrument("wobble_bass", 9));

    // 10. Dark pad
    let pad_data = synthesizers::generate_pad_dark();
    samples.push(("dark_pad", pad_data));
    let pad_sample = make_sample("dark_pad", sample_rate);
    writer.add_sample_header_only(pad_sample);
    writer.add_instrument(make_instrument_continue("dark_pad", 10));

    // 11. Lead stab
    let stab_data = synthesizers::generate_lead_stab();
    samples.push(("lead_stab", stab_data));
    let stab_sample = make_sample("lead_stab", sample_rate);
    writer.add_sample_header_only(stab_sample);
    writer.add_instrument(make_instrument("lead_stab", 11));

    // 12. Lead main
    let lead_data = synthesizers::generate_lead_main();
    samples.push(("lead_main", lead_data));
    let lead_sample = make_sample("lead_main", sample_rate);
    writer.add_sample_header_only(lead_sample);
    writer.add_instrument(make_instrument("lead_main", 12));

    // 13. FX Riser
    let riser_data = synthesizers::generate_fx_riser();
    samples.push(("fx_riser", riser_data));
    let riser_sample = make_sample("fx_riser", sample_rate);
    writer.add_sample_header_only(riser_sample);
    writer.add_instrument(make_instrument("fx_riser", 13));

    // 14. FX Impact
    let impact_data = synthesizers::generate_fx_impact();
    samples.push(("fx_impact", impact_data));
    let impact_sample = make_sample("fx_impact", sample_rate);
    writer.add_sample_header_only(impact_sample);
    writer.add_instrument(make_instrument("fx_impact", 14));

    // 15. Atmosphere
    let atmos_data = synthesizers::generate_atmos_storm();
    samples.push(("atmosphere", atmos_data));
    let atmos_sample = make_sample("atmosphere", sample_rate);
    writer.add_sample_header_only(atmos_sample);
    writer.add_instrument(make_instrument_continue("atmosphere", 15));

    // ========================================================================
    // Create 12 unique patterns for ~90 second track
    // ========================================================================

    // Pattern 0: Intro
    let pat_intro = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat_intro);

    // Pattern 1: Build A
    let pat_build_a = writer.add_pattern(64);
    build_build_a_pattern(&mut writer, pat_build_a);

    // Pattern 2: Build B (snare roll, riser peak)
    let pat_build_b = writer.add_pattern(64);
    build_build_b_pattern(&mut writer, pat_build_b);

    // Pattern 3: Drop A1 (main drop, clean)
    let pat_drop_a1 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_a1, Section::DropA, 0, BreakStyle::None);

    // Pattern 4: Drop A2 (variation with ghost break)
    let pat_drop_a2 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_a2, Section::DropA, 1, BreakStyle::Ghost);

    // Pattern 5: Drop A3 (full break layer)
    let pat_drop_a3 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_a3, Section::DropA, 2, BreakStyle::Accent);

    // Pattern 6: Breakdown
    let pat_breakdown = writer.add_pattern(64);
    build_breakdown_pattern(&mut writer, pat_breakdown);

    // Pattern 7: Build C (intense pre-climax)
    let pat_build_c = writer.add_pattern(64);
    build_build_c_pattern(&mut writer, pat_build_c);

    // Pattern 8: Drop B1 (climax start)
    let pat_drop_b1 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_b1, Section::DropB, 0, BreakStyle::Ghost);

    // Pattern 9: Drop B2 (climax variation)
    let pat_drop_b2 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_b2, Section::DropB, 1, BreakStyle::Accent);

    // Pattern 10: Drop B3 (maximum intensity)
    let pat_drop_b3 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_b3, Section::DropB, 2, BreakStyle::Fill);

    // Pattern 11: Outro
    let pat_outro = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat_outro);

    // ========================================================================
    // Order table: 19 entries for ~66 second track (proper DnB structure)
    // ========================================================================
    writer.set_orders(&[
        pat_intro,                                      // 0: Intro (4 bars)
        pat_build_a, pat_build_b,                       // 1-2: Build (8 bars)
        pat_drop_a1, pat_drop_a2, pat_drop_a1, pat_drop_a3, // 3-6: Drop A (16 bars)
        pat_breakdown,                                  // 7: Breakdown (4 bars)
        pat_build_a, pat_build_c,                       // 8-9: Build (8 bars)
        pat_drop_b1, pat_drop_b2, pat_drop_b1, pat_drop_b3, // 10-13: Drop B (16 bars)
        pat_breakdown,                                  // 14: Breakdown 2 (4 bars)
        pat_build_c,                                    // 15: Build C (4 bars)
        pat_drop_b1, pat_drop_b3,                       // 16-17: Final drop (8 bars)
        pat_outro,                                      // 18: Outro (4 bars)
    ]);

    // Set song message
    writer.set_message(
        "Nether Storm - DnB @ 174 BPM\n\
         Hero Quality Edition\n\
         Generated by gen-tracker-demo-it\n\
         Nethercore Project",
    );

    (writer.write(), samples)
}

/// Generate embedded IT file (with sample data)
pub fn generate_storm_it_embedded() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
    let mut writer = ItWriter::new("Nether Storm");

    // Set up module parameters
    writer.set_channels(16);
    writer.set_speed(3); // Fast speed for DnB
    writer.set_tempo(174);
    writer.set_global_volume(128);
    writer.set_mix_volume(80);
    writer.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES);

    // Generate and collect samples
    let mut samples = Vec::new();
    let sample_rate = 22050;

    // 1. Kick - transient + sine sweep
    let kick_data = synthesizers::generate_kick_dnb();
    samples.push(("kick_dnb", kick_data));
    let kick_sample = make_sample("kick_dnb", sample_rate);
    writer.add_sample(kick_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("kick_dnb", 1));

    // 2. Snare - layered
    let snare_data = synthesizers::generate_snare_dnb();
    samples.push(("snare_dnb", snare_data));
    let snare_sample = make_sample("snare_dnb", sample_rate);
    writer.add_sample(snare_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("snare_dnb", 2));

    // 3. Hihat closed
    let hh_closed_data = synthesizers::generate_hihat_closed();
    samples.push(("hh_closed", hh_closed_data));
    let hh_sample = make_sample("hh_closed", sample_rate);
    writer.add_sample(hh_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("hh_closed", 3));

    // 4. Hihat open
    let hh_open_data = synthesizers::generate_hihat_open();
    samples.push(("hh_open", hh_open_data));
    let hho_sample = make_sample("hh_open", sample_rate);
    writer.add_sample(hho_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("hh_open", 4));

    // 5. Break slice
    let break_data = synthesizers::generate_break_slice();
    samples.push(("break_slice", break_data));
    let break_sample = make_sample("break_slice", sample_rate);
    writer.add_sample(break_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("break_slice", 5));

    // 6. Cymbal (storm-specific)
    let cymbal_data = synthesizers::generate_cymbal();
    samples.push(("cymbal_storm", cymbal_data));
    let cymbal_sample = make_sample("cymbal_storm", sample_rate);
    writer.add_sample(cymbal_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("cymbal_storm", 6));

    // 7. Sub bass
    let sub_data = synthesizers::generate_bass_sub_dnb();
    samples.push(("sub_bass", sub_data));
    let sub_sample = make_sample("sub_bass", sample_rate);
    writer.add_sample(sub_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("sub_bass", 7));

    // 8. Reese bass
    let reese_data = synthesizers::generate_bass_reese();
    samples.push(("reese_bass", reese_data));
    let reese_sample = make_sample("reese_bass", sample_rate);
    writer.add_sample(reese_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("reese_bass", 8));

    // 9. Wobble bass
    let wobble_data = synthesizers::generate_bass_wobble();
    samples.push(("wobble_bass", wobble_data));
    let wobble_sample = make_sample("wobble_bass", sample_rate);
    writer.add_sample(wobble_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("wobble_bass", 9));

    // 10. Dark pad
    let pad_data = synthesizers::generate_pad_dark();
    samples.push(("dark_pad", pad_data));
    let pad_sample = make_sample("dark_pad", sample_rate);
    writer.add_sample(pad_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("dark_pad", 10));

    // 11. Lead stab
    let stab_data = synthesizers::generate_lead_stab();
    samples.push(("lead_stab", stab_data));
    let stab_sample = make_sample("lead_stab", sample_rate);
    writer.add_sample(stab_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("lead_stab", 11));

    // 12. Lead main
    let lead_data = synthesizers::generate_lead_main();
    samples.push(("lead_main", lead_data));
    let lead_sample = make_sample("lead_main", sample_rate);
    writer.add_sample(lead_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("lead_main", 12));

    // 13. FX Riser
    let riser_data = synthesizers::generate_fx_riser();
    samples.push(("fx_riser", riser_data));
    let riser_sample = make_sample("fx_riser", sample_rate);
    writer.add_sample(riser_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("fx_riser", 13));

    // 14. FX Impact
    let impact_data = synthesizers::generate_fx_impact();
    samples.push(("fx_impact", impact_data));
    let impact_sample = make_sample("fx_impact", sample_rate);
    writer.add_sample(impact_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("fx_impact", 14));

    // 15. Atmosphere
    let atmos_data = synthesizers::generate_atmos_storm();
    samples.push(("atmosphere", atmos_data));
    let atmos_sample = make_sample("atmosphere", sample_rate);
    writer.add_sample(atmos_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("atmosphere", 15));

    // ========================================================================
    // Create 12 unique patterns for ~90 second track
    // ========================================================================

    // Pattern 0: Intro
    let pat_intro = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat_intro);

    // Pattern 1: Build A
    let pat_build_a = writer.add_pattern(64);
    build_build_a_pattern(&mut writer, pat_build_a);

    // Pattern 2: Build B (snare roll, riser peak)
    let pat_build_b = writer.add_pattern(64);
    build_build_b_pattern(&mut writer, pat_build_b);

    // Pattern 3: Drop A1 (main drop, clean)
    let pat_drop_a1 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_a1, Section::DropA, 0, BreakStyle::None);

    // Pattern 4: Drop A2 (variation with ghost break)
    let pat_drop_a2 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_a2, Section::DropA, 1, BreakStyle::Ghost);

    // Pattern 5: Drop A3 (full break layer)
    let pat_drop_a3 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_a3, Section::DropA, 2, BreakStyle::Accent);

    // Pattern 6: Breakdown
    let pat_breakdown = writer.add_pattern(64);
    build_breakdown_pattern(&mut writer, pat_breakdown);

    // Pattern 7: Build C (intense pre-climax)
    let pat_build_c = writer.add_pattern(64);
    build_build_c_pattern(&mut writer, pat_build_c);

    // Pattern 8: Drop B1 (climax start)
    let pat_drop_b1 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_b1, Section::DropB, 0, BreakStyle::Ghost);

    // Pattern 9: Drop B2 (climax variation)
    let pat_drop_b2 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_b2, Section::DropB, 1, BreakStyle::Accent);

    // Pattern 10: Drop B3 (maximum intensity)
    let pat_drop_b3 = writer.add_pattern(64);
    build_drop_pattern(&mut writer, pat_drop_b3, Section::DropB, 2, BreakStyle::Fill);

    // Pattern 11: Outro
    let pat_outro = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat_outro);

    // ========================================================================
    // Order table: 19 entries for ~66 second track (proper DnB structure)
    // ========================================================================
    writer.set_orders(&[
        pat_intro,                                      // 0: Intro (4 bars)
        pat_build_a, pat_build_b,                       // 1-2: Build (8 bars)
        pat_drop_a1, pat_drop_a2, pat_drop_a1, pat_drop_a3, // 3-6: Drop A (16 bars)
        pat_breakdown,                                  // 7: Breakdown (4 bars)
        pat_build_a, pat_build_c,                       // 8-9: Build (8 bars)
        pat_drop_b1, pat_drop_b2, pat_drop_b1, pat_drop_b3, // 10-13: Drop B (16 bars)
        pat_breakdown,                                  // 14: Breakdown 2 (4 bars)
        pat_build_c,                                    // 15: Build C (4 bars)
        pat_drop_b1, pat_drop_b3,                       // 16-17: Final drop (8 bars)
        pat_outro,                                      // 18: Outro (4 bars)
    ]);

    // Set song message
    writer.set_message(
        "Nether Storm - DnB @ 174 BPM\n\
         Hero Quality Edition\n\
         Generated by gen-tracker-demo-it\n\
         Nethercore Project",
    );

    (writer.write(), samples)
}

// ============================================================================
// Drum Helper Functions
// ============================================================================

/// Standard DnB kick pattern (kick on 1 and 2.5)
fn add_kick_dnb(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 1 (row 0)
    writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 64));
    // Beat 2.5 (row 10) - syncopation
    writer.set_note(pat, base + 10, CH_KICK, ItNote::play_note(F2, INST_KICK, 58));
}

/// Enhanced kick pattern for climax (more syncopation)
fn add_kick_climax(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 1
    writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 64));
    // Extra kick on row 6 for energy
    writer.set_note(pat, base + 6, CH_KICK, ItNote::play_note(F2, INST_KICK, 52));
    // Beat 2.5
    writer.set_note(pat, base + 10, CH_KICK, ItNote::play_note(F2, INST_KICK, 60));
}

/// Main snares on beat 2 and 4
fn add_main_snares(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 2 (row 4)
    writer.set_note(pat, base + 4, CH_SNARE, ItNote::play_note(C5, INST_SNARE, 64));
    // Beat 4 (row 12)
    writer.set_note(pat, base + 12, CH_SNARE, ItNote::play_note(C5, INST_SNARE, 64));
}

/// Ghost snares with density control (0=none, 1=light, 2=medium)
fn add_ghost_snares(writer: &mut ItWriter, pat: u8, bar: u16, density: u8) {
    if density == 0 {
        return; // Clean pattern - no ghost notes
    }

    let base = bar * 16;

    // Light (density 1): just before main snares for anticipation
    writer.set_note(pat, base + 2, CH_SNARE, ItNote::play_note(C5, INST_SNARE, 22));
    writer.set_note(pat, base + 10, CH_SNARE, ItNote::play_note(C5, INST_SNARE, 22));

    if density >= 2 {
        // Medium (density 2): add after-snare ghosts for shuffle feel
        writer.set_note(pat, base + 6, CH_SNARE, ItNote::play_note(C5, INST_SNARE, 18));
        writer.set_note(pat, base + 14, CH_SNARE, ItNote::play_note(C5, INST_SNARE, 16));
    }
}

/// Humanized hi-hat groove with velocity variation (8th notes, not 16ths!)
fn add_hihat_groove(writer: &mut ItWriter, pat: u8, bar: u16, use_opens: bool) {
    let base = bar * 16;
    // 8th note pattern: every 2 rows (8 hits per bar, not 16)
    // Velocity: strong on downbeats, medium on upbeats
    let positions: [(u16, u8); 8] = [
        (0, 55),   // Beat 1 - strong
        (2, 38),   // &
        (4, 50),   // Beat 2 - strong
        (6, 35),   // & (open hat position)
        (8, 52),   // Beat 3 - strong
        (10, 38),  // &
        (12, 50),  // Beat 4 - strong
        (14, 35),  // & (open hat position)
    ];

    for (offset, vel) in positions {
        let abs_row = base + offset;
        // Open hi-hats on the "and" of 2 and 4 for groove
        if use_opens && (offset == 6 || offset == 14) {
            writer.set_note(pat, abs_row, CH_HIHAT_OPEN, ItNote::play_note(C5, INST_HH_OPEN, vel + 5));
        } else {
            writer.set_note(pat, abs_row, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, vel));
        }
    }
}

/// Snare roll with accelerating density and crescendo
fn add_snare_roll(writer: &mut ItWriter, pat: u8, start_row: u16, length: u16) {
    let mut row = start_row;
    let end_row = start_row + length;
    let mut spacing = 4u16;
    let vel_start = 35u8;
    let vel_end = 64u8;

    while row < end_row {
        let progress = (row - start_row) as f32 / length as f32;
        let vel = vel_start + ((vel_end - vel_start) as f32 * progress) as u8;

        writer.set_note(pat, row, CH_SNARE, ItNote::play_note(C5, INST_SNARE, vel));

        // Accelerate: 4 -> 2 spacing (don't go to 1, too rapid at 174 BPM)
        if progress > 0.6 {
            spacing = 2;
        }

        row += spacing;
    }
}

/// Kick fill for variation patterns (adds extra kicks, avoids overlap with main pattern)
fn add_kick_fill(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Syncopated fill kicks - only at positions NOT used by add_kick_dnb (which uses 0 and 10)
    let hits = [(3, 48), (6, 50), (14, 45)];
    for (offset, vel) in hits {
        writer.set_note(pat, base + offset, CH_KICK, ItNote::play_note(F2, INST_KICK, vel));
    }
}

// ============================================================================
// Break Layer Helper
// ============================================================================

/// Add break layer with different styles - all sparse to avoid cluttering
fn add_break_layer(writer: &mut ItWriter, pat: u8, style: BreakStyle) {
    match style {
        BreakStyle::None => {}
        BreakStyle::Ghost => {
            // Very sparse - just one accent every 2 bars
            writer.set_note(pat, 8, CH_BREAK, ItNote::play_note(C5, INST_BREAK, 22));
            writer.set_note(pat, 40, CH_BREAK, ItNote::play_note(C5, INST_BREAK, 20));
        }
        BreakStyle::Accent => {
            // Layer with snares only - one per bar on beat 2
            for bar in 0u16..4 {
                let base = bar * 16;
                writer.set_note(pat, base + 4, CH_BREAK, ItNote::play_note(C5, INST_BREAK, 35));
            }
        }
        BreakStyle::Fill => {
            // Accent on snares only, slightly louder for climax
            for bar in 0u16..4 {
                let base = bar * 16;
                writer.set_note(pat, base + 4, CH_BREAK, ItNote::play_note(C5, INST_BREAK, 40));
                writer.set_note(pat, base + 12, CH_BREAK, ItNote::play_note(C5, INST_BREAK, 38));
            }
        }
    }
}

// ============================================================================
// Bass Helper Functions
// ============================================================================

/// Chord progression roots for Drop A (Fm - Db - Eb - Fm)
fn get_drop_a_roots() -> [(u16, u8, u8); 4] {
    // (start_row, sub_note, reese_note)
    [(0, F1, F2), (16, DB1, DB2), (32, EB1, EB2), (48, F1, F2)]
}

/// Chord progression roots for Drop B (Fm - Fm/Eb - Db - C)
fn get_drop_b_roots() -> [(u16, u8, u8); 4] {
    [(0, F1, F2), (16, EB1, EB2), (32, DB1, DB2), (48, C2, C3)]
}

/// Rebalanced bass with sidechain feel (sub quiet, reese prominent)
fn add_bass_rebalanced(writer: &mut ItWriter, pat: u8, section: Section) {
    let roots = if section == Section::DropA {
        get_drop_a_roots()
    } else {
        get_drop_b_roots()
    };

    let reese_vel = if section == Section::DropB { 58 } else { 54 };

    for (row, sub_note, reese_note) in roots {
        // Sub: delayed by 1 row for sidechain feel, lower velocity
        writer.set_note(pat, row + 1, CH_SUB, ItNote::play_note(sub_note, INST_SUB, 42));

        // Reese: main audible bass, also delayed for pumping feel
        writer.set_note(pat, row + 1, CH_REESE, ItNote::play_note(reese_note, INST_REESE, reese_vel));

        // Reese melodic movement (8 rows later)
        if row + 8 < 64 {
            writer.set_note(
                pat,
                row + 8,
                CH_REESE,
                ItNote::play_note(reese_note + 3, INST_REESE, reese_vel - 4),
            );
        }
    }
}

/// Wobble bass accent (Drop B only)
fn add_wobble_accent(writer: &mut ItWriter, pat: u8) {
    // Wobble on accent points in climax
    writer.set_note(pat, 0, CH_WOBBLE, ItNote::play_note(F3, INST_WOBBLE, 40));
    writer.set_note(pat, 32, CH_WOBBLE, ItNote::play_note(AB3, INST_WOBBLE, 38));
}

// ============================================================================
// Lead/Melody Helper Functions
// ============================================================================

/// Lead melody for drops - sparse, punchy accents (not rapid triplets!)
fn add_lead_melody(writer: &mut ItWriter, pat: u8, section: Section, variation: u8) {
    if section == Section::DropA {
        // Drop A: Simple stab accents on key beats - one per bar
        writer.set_note(pat, 0, CH_STAB, ItNote::play_note(F4, INST_STAB, 55));   // Bar 1
        writer.set_note(pat, 16, CH_STAB, ItNote::play_note(AB4, INST_STAB, 52)); // Bar 2
        writer.set_note(pat, 32, CH_STAB, ItNote::play_note(C5, INST_STAB, 55));  // Bar 3
        writer.set_note(pat, 48, CH_STAB, ItNote::play_note(EB4, INST_STAB, 50)); // Bar 4

        if variation >= 1 {
            // Variation: add off-beat accents (still sparse - one extra per bar)
            writer.set_note(pat, 10, CH_STAB, ItNote::play_note(C5, INST_STAB, 45));  // Bar 1 offbeat
            writer.set_note(pat, 26, CH_STAB, ItNote::play_note(F5, INST_STAB, 52));  // Bar 2 offbeat
            writer.set_note(pat, 42, CH_STAB, ItNote::play_note(AB4, INST_STAB, 48)); // Bar 3 offbeat
            writer.set_note(pat, 58, CH_STAB, ItNote::play_note(F5, INST_STAB, 55));  // Bar 4 end
        }
    } else {
        // Drop B: Stronger accents, still sparse - avoid rapid notes
        writer.set_note(pat, 0, CH_LEAD, ItNote::play_note(F5, INST_LEAD, 58));   // Bar 1 - high!
        writer.set_note(pat, 16, CH_LEAD, ItNote::play_note(EB5, INST_LEAD, 55)); // Bar 2
        writer.set_note(pat, 32, CH_LEAD, ItNote::play_note(C5, INST_LEAD, 55));  // Bar 3
        writer.set_note(pat, 48, CH_LEAD, ItNote::play_note(AB4, INST_LEAD, 52)); // Bar 4

        if variation >= 1 {
            // Variation: add off-beat responses (still one per bar, not rapid)
            writer.set_note(pat, 8, CH_LEAD, ItNote::play_note(C5, INST_LEAD, 50));   // Bar 1 response
            writer.set_note(pat, 24, CH_LEAD, ItNote::play_note(AB4, INST_LEAD, 48)); // Bar 2 response
            writer.set_note(pat, 40, CH_LEAD, ItNote::play_note(F5, INST_LEAD, 55));  // Bar 3 response
            writer.set_note(pat, 56, CH_LEAD, ItNote::play_note(F5, INST_LEAD, 60));  // Bar 4 climax
        }

        // Impact at start of Drop B
        writer.set_note(pat, 0, CH_IMPACT, ItNote::play_note(F3, INST_IMPACT, 60));
    }
}

// ============================================================================
// Pattern Builders
// ============================================================================

fn build_intro_pattern(writer: &mut ItWriter, pat: u8) {
    // Atmosphere builds, sparse elements
    writer.set_note(pat, 0, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 32));

    // Riser building tension
    writer.set_note(pat, 32, CH_RISER, ItNote::play_note(F4, INST_RISER, 40));

    // Very sparse hi-hats (every 16 rows)
    for row in (8..64).step_by(16) {
        writer.set_note(pat, row as u16, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, 22));
    }
}

fn build_build_a_pattern(writer: &mut ItWriter, pat: u8) {
    // Drums come in, tension builds
    // Quarter note kicks
    for bar in 0u16..4 {
        let base = bar * 16;
        writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 58));
    }

    // Eighth note hi-hats with some variation
    for row in (0..64).step_by(8) {
        let vel = if row % 16 == 0 { 52 } else { 42 };
        writer.set_note(pat, row as u16, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, vel));
    }

    // Pad
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(F3, INST_PAD, 48));

    // Riser
    writer.set_note(pat, 0, CH_RISER, ItNote::play_note(F4, INST_RISER, 45));
}

fn build_build_b_pattern(writer: &mut ItWriter, pat: u8) {
    // Build with snare roll at end
    // Continue kick pattern
    for bar in 0u16..4 {
        let base = bar * 16;
        writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 60));
    }

    // Hi-hats get faster
    for row in (0..48).step_by(4) {
        let vel = 35 + (row as u8 / 4);
        writer.set_note(pat, row as u16, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, vel));
    }

    // Snare roll from row 48-63 (accelerating crescendo)
    add_snare_roll(writer, pat, 48, 16);

    // Riser peaks
    writer.set_note(pat, 0, CH_RISER, ItNote::play_note(F4, INST_RISER, 55));
    writer.set_note(pat, 32, CH_RISER, ItNote::play_note(AB4, INST_RISER, 58));

    // Cymbal crash at end
    writer.set_note(pat, 60, CH_CYMBAL, ItNote::play_note(C5, INST_CYMBAL, 50));
}

/// Unified drop pattern builder with variation control
fn build_drop_pattern(writer: &mut ItWriter, pat: u8, section: Section, variation: u8, break_style: BreakStyle) {
    // Drums for all 4 bars
    for bar in 0u16..4 {
        // Kick pattern
        if section == Section::DropB {
            add_kick_climax(writer, pat, bar);
        } else {
            add_kick_dnb(writer, pat, bar);
        }

        // Main snares
        add_main_snares(writer, pat, bar);

        // Ghost snares with density based on variation
        add_ghost_snares(writer, pat, bar, variation);

        // Humanized hi-hats (opens in drops)
        add_hihat_groove(writer, pat, bar, true);
    }

    // Kick fill on bar 4 for variations
    if variation >= 1 {
        add_kick_fill(writer, pat, 3);
    }

    // Break layer
    add_break_layer(writer, pat, break_style);

    // Rebalanced bass
    add_bass_rebalanced(writer, pat, section);

    // Wobble in Drop B variations
    if section == Section::DropB && variation >= 1 {
        add_wobble_accent(writer, pat);
    }

    // Lead melody
    add_lead_melody(writer, pat, section, variation);

    // Atmosphere throughout
    writer.set_note(pat, 0, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 25));
}

fn build_breakdown_pattern(writer: &mut ItWriter, pat: u8) {
    // Drums drop, atmospheric breathing room
    // Pad sustains
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(F3, INST_PAD, 50));
    writer.set_note(pat, 32, CH_PAD, ItNote::play_note(DB3, INST_PAD, 48));

    // Sparse kick
    writer.set_note(pat, 0, CH_KICK, ItNote::play_note(F2, INST_KICK, 50));
    writer.set_note(pat, 32, CH_KICK, ItNote::play_note(F2, INST_KICK, 45));

    // Half-time hi-hats
    for row in (0..64).step_by(16) {
        writer.set_note(pat, row as u16, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, 35));
    }

    // Sub bass with movement
    writer.set_note(pat, 0, CH_SUB, ItNote::play_note(F1, INST_SUB, 40));
    writer.set_note(pat, 32, CH_SUB, ItNote::play_note(DB1, INST_SUB, 38));

    // Atmosphere
    writer.set_note(pat, 0, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 40));

    // Riser building for next section
    writer.set_note(pat, 32, CH_RISER, ItNote::play_note(F4, INST_RISER, 45));
}

fn build_build_c_pattern(writer: &mut ItWriter, pat: u8) {
    // Intense pre-climax build
    // Kick pattern
    for bar in 0u16..4 {
        let base = bar * 16;
        writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 62));
        writer.set_note(pat, base + 8, CH_KICK, ItNote::play_note(F2, INST_KICK, 52));
    }

    // Fast hi-hats
    for row in 0..48 {
        if row % 2 == 0 {
            let vel = 30 + (row as u8 / 3);
            writer.set_note(pat, row as u16, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, vel));
        }
    }

    // Intense snare roll (longer)
    add_snare_roll(writer, pat, 40, 24);

    // All risers
    writer.set_note(pat, 0, CH_RISER, ItNote::play_note(F4, INST_RISER, 58));
    writer.set_note(pat, 16, CH_RISER, ItNote::play_note(AB4, INST_RISER, 60));
    writer.set_note(pat, 32, CH_RISER, ItNote::play_note(C5, INST_RISER, 62));

    // Cymbal crash
    writer.set_note(pat, 60, CH_CYMBAL, ItNote::play_note(C5, INST_CYMBAL, 55));

    // Sub bass tension
    writer.set_note(pat, 0, CH_SUB, ItNote::play_note(F1, INST_SUB, 45));
    writer.set_note(pat, 32, CH_SUB, ItNote::play_note(C2, INST_SUB, 48));
}

fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
    // Elements drop out, loop preparation
    // Sparse kick
    writer.set_note(pat, 0, CH_KICK, ItNote::play_note(F2, INST_KICK, 55));
    writer.set_note(pat, 32, CH_KICK, ItNote::play_note(F2, INST_KICK, 45));

    // Sub bass fading
    writer.set_note(pat, 0, CH_SUB, ItNote::play_note(F1, INST_SUB, 45));
    writer.set_note(pat, 32, CH_SUB, ItNote::play_note(F1, INST_SUB, 35));

    // Pad sustain fading
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(F3, INST_PAD, 40));
    writer.set_note(pat, 32, CH_PAD, ItNote::play_note(F3, INST_PAD, 30));

    // Sparse hi-hats
    for row in (0..64).step_by(16) {
        writer.set_note(pat, row as u16, CH_HIHAT, ItNote::play_note(C5, INST_HH_CLOSED, 28));
    }

    // Cymbal decay
    writer.set_note(pat, 0, CH_CYMBAL, ItNote::play_note(C5, INST_CYMBAL, 35));

    // Atmosphere fades
    writer.set_note(pat, 0, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 35));
    writer.set_note(pat, 32, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 25));
}
