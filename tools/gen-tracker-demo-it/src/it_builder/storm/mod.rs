//! Nether Storm - DnB/Action IT generator (Hero Quality)
//!
//! 174 BPM, F minor (Phrygian), 16 channels
//! Duration: ~90 seconds with seamless loop
//! Features: Ghost notes, humanized drums, break layers, rebalanced bass

mod bass;
mod constants;
mod drums;
mod melody;
mod patterns;
mod types;

use crate::synthesizers;
use nether_it::{ItFlags, ItWriter};
use patterns::*;
use types::{BreakStyle, Section};

use super::{make_instrument, make_instrument_continue, make_sample};

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
    build_drop_pattern(
        &mut writer,
        pat_drop_a1,
        Section::DropA,
        0,
        BreakStyle::None,
    );

    // Pattern 4: Drop A2 (variation with ghost break)
    let pat_drop_a2 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_a2,
        Section::DropA,
        1,
        BreakStyle::Ghost,
    );

    // Pattern 5: Drop A3 (full break layer)
    let pat_drop_a3 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_a3,
        Section::DropA,
        2,
        BreakStyle::Accent,
    );

    // Pattern 6: Breakdown
    let pat_breakdown = writer.add_pattern(64);
    build_breakdown_pattern(&mut writer, pat_breakdown);

    // Pattern 7: Build C (intense pre-climax)
    let pat_build_c = writer.add_pattern(64);
    build_build_c_pattern(&mut writer, pat_build_c);

    // Pattern 8: Drop B1 (climax start)
    let pat_drop_b1 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_b1,
        Section::DropB,
        0,
        BreakStyle::Ghost,
    );

    // Pattern 9: Drop B2 (climax variation)
    let pat_drop_b2 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_b2,
        Section::DropB,
        1,
        BreakStyle::Accent,
    );

    // Pattern 10: Drop B3 (maximum intensity)
    let pat_drop_b3 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_b3,
        Section::DropB,
        2,
        BreakStyle::Fill,
    );

    // Pattern 11: Outro
    let pat_outro = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat_outro);

    // ========================================================================
    // Order table: 19 entries for ~66 second track (proper DnB structure)
    // ========================================================================
    writer.set_orders(&[
        pat_intro, // 0: Intro (4 bars)
        pat_build_a,
        pat_build_b, // 1-2: Build (8 bars)
        pat_drop_a1,
        pat_drop_a2,
        pat_drop_a1,
        pat_drop_a3,   // 3-6: Drop A (16 bars)
        pat_breakdown, // 7: Breakdown (4 bars)
        pat_build_a,
        pat_build_c, // 8-9: Build (8 bars)
        pat_drop_b1,
        pat_drop_b2,
        pat_drop_b1,
        pat_drop_b3,   // 10-13: Drop B (16 bars)
        pat_breakdown, // 14: Breakdown 2 (4 bars)
        pat_build_c,   // 15: Build C (4 bars)
        pat_drop_b1,
        pat_drop_b3, // 16-17: Final drop (8 bars)
        pat_outro,   // 18: Outro (4 bars)
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
    build_drop_pattern(
        &mut writer,
        pat_drop_a1,
        Section::DropA,
        0,
        BreakStyle::None,
    );

    // Pattern 4: Drop A2 (variation with ghost break)
    let pat_drop_a2 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_a2,
        Section::DropA,
        1,
        BreakStyle::Ghost,
    );

    // Pattern 5: Drop A3 (full break layer)
    let pat_drop_a3 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_a3,
        Section::DropA,
        2,
        BreakStyle::Accent,
    );

    // Pattern 6: Breakdown
    let pat_breakdown = writer.add_pattern(64);
    build_breakdown_pattern(&mut writer, pat_breakdown);

    // Pattern 7: Build C (intense pre-climax)
    let pat_build_c = writer.add_pattern(64);
    build_build_c_pattern(&mut writer, pat_build_c);

    // Pattern 8: Drop B1 (climax start)
    let pat_drop_b1 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_b1,
        Section::DropB,
        0,
        BreakStyle::Ghost,
    );

    // Pattern 9: Drop B2 (climax variation)
    let pat_drop_b2 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_b2,
        Section::DropB,
        1,
        BreakStyle::Accent,
    );

    // Pattern 10: Drop B3 (maximum intensity)
    let pat_drop_b3 = writer.add_pattern(64);
    build_drop_pattern(
        &mut writer,
        pat_drop_b3,
        Section::DropB,
        2,
        BreakStyle::Fill,
    );

    // Pattern 11: Outro
    let pat_outro = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat_outro);

    // ========================================================================
    // Order table: 19 entries for ~66 second track (proper DnB structure)
    // ========================================================================
    writer.set_orders(&[
        pat_intro, // 0: Intro (4 bars)
        pat_build_a,
        pat_build_b, // 1-2: Build (8 bars)
        pat_drop_a1,
        pat_drop_a2,
        pat_drop_a1,
        pat_drop_a3,   // 3-6: Drop A (16 bars)
        pat_breakdown, // 7: Breakdown (4 bars)
        pat_build_a,
        pat_build_c, // 8-9: Build (8 bars)
        pat_drop_b1,
        pat_drop_b2,
        pat_drop_b1,
        pat_drop_b3,   // 10-13: Drop B (16 bars)
        pat_breakdown, // 14: Breakdown 2 (4 bars)
        pat_build_c,   // 15: Build C (4 bars)
        pat_drop_b1,
        pat_drop_b3, // 16-17: Final drop (8 bars)
        pat_outro,   // 18: Outro (4 bars)
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
