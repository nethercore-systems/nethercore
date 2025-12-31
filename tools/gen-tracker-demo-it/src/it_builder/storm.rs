//! Nether Storm - DnB/Action IT generator
//!
//! 174 BPM, F minor, 16 channels
//! Features: Fast drums, reese bass, wobble bass, aggressive leads

use super::{make_instrument, make_instrument_continue, make_sample};
use crate::synthesizers;
use nether_it::{ItFlags, ItNote, ItWriter};

/// Generate the Nether Storm IT file
pub fn generate_storm_it() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
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
    let kick_sample = make_sample("Kick DnB", sample_rate);
    writer.add_sample(kick_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Kick", 1));

    // 2. Snare - layered
    let snare_data = synthesizers::generate_snare_dnb();
    samples.push(("snare_dnb", snare_data));
    let snare_sample = make_sample("Snare DnB", sample_rate);
    writer.add_sample(snare_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Snare", 2));

    // 3. Hihat closed
    let hh_closed_data = synthesizers::generate_hihat_closed();
    samples.push(("hihat_closed", hh_closed_data));
    let hh_sample = make_sample("HH Closed", sample_rate);
    writer.add_sample(hh_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("HH Closed", 3));

    // 4. Hihat open
    let hh_open_data = synthesizers::generate_hihat_open();
    samples.push(("hihat_open", hh_open_data));
    let hho_sample = make_sample("HH Open", sample_rate);
    writer.add_sample(hho_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("HH Open", 4));

    // 5. Break slice
    let break_data = synthesizers::generate_break_slice();
    samples.push(("break_slice", break_data));
    let break_sample = make_sample("Break Slice", sample_rate);
    writer.add_sample(break_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Break", 5));

    // 6. Cymbal
    let cymbal_data = synthesizers::generate_cymbal();
    samples.push(("cymbal", cymbal_data));
    let cymbal_sample = make_sample("Cymbal", sample_rate);
    writer.add_sample(cymbal_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Cymbal", 6));

    // 7. Sub bass
    let sub_data = synthesizers::generate_bass_sub_dnb();
    samples.push(("bass_sub", sub_data));
    let sub_sample = make_sample("Sub Bass", sample_rate);
    writer.add_sample(sub_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Sub", 7));

    // 8. Reese bass
    let reese_data = synthesizers::generate_bass_reese();
    samples.push(("bass_reese", reese_data));
    let reese_sample = make_sample("Reese Bass", sample_rate);
    writer.add_sample(reese_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Reese", 8));

    // 9. Wobble bass
    let wobble_data = synthesizers::generate_bass_wobble();
    samples.push(("bass_wobble", wobble_data));
    let wobble_sample = make_sample("Wobble Bass", sample_rate);
    writer.add_sample(wobble_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Wobble", 9));

    // 10. Dark pad
    let pad_data = synthesizers::generate_pad_dark();
    samples.push(("pad_dark", pad_data));
    let pad_sample = make_sample("Dark Pad", sample_rate);
    writer.add_sample(pad_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("Pad", 10));

    // 11. Lead stab
    let stab_data = synthesizers::generate_lead_stab();
    samples.push(("lead_stab", stab_data));
    let stab_sample = make_sample("Lead Stab", sample_rate);
    writer.add_sample(stab_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Stab", 11));

    // 12. Lead main
    let lead_data = synthesizers::generate_lead_main();
    samples.push(("lead_main", lead_data));
    let lead_sample = make_sample("Lead Main", sample_rate);
    writer.add_sample(lead_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Lead", 12));

    // 13. FX Riser
    let riser_data = synthesizers::generate_fx_riser();
    samples.push(("fx_riser", riser_data));
    let riser_sample = make_sample("FX Riser", sample_rate);
    writer.add_sample(riser_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Riser", 13));

    // 14. FX Impact
    let impact_data = synthesizers::generate_fx_impact();
    samples.push(("fx_impact", impact_data));
    let impact_sample = make_sample("FX Impact", sample_rate);
    writer.add_sample(impact_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Impact", 14));

    // 15. Atmosphere
    let atmos_data = synthesizers::generate_atmos_storm();
    samples.push(("atmos_storm", atmos_data));
    let atmos_sample = make_sample("Atmosphere", sample_rate);
    writer.add_sample(atmos_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("Atmos", 15));

    // Create patterns (6 patterns, 64 rows each)
    // Pattern 0: Intro
    let pat0 = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat0);

    // Pattern 1: Build
    let pat1 = writer.add_pattern(64);
    build_build_pattern(&mut writer, pat1);

    // Pattern 2: Drop A
    let pat2 = writer.add_pattern(64);
    build_drop_a_pattern(&mut writer, pat2);

    // Pattern 3: Breakdown
    let pat3 = writer.add_pattern(64);
    build_breakdown_pattern(&mut writer, pat3);

    // Pattern 4: Drop B (climax)
    let pat4 = writer.add_pattern(64);
    build_drop_b_pattern(&mut writer, pat4);

    // Pattern 5: Outro
    let pat5 = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat5);

    // Set order table: Intro, Build, Drop A, Drop A, Breakdown, Drop B, Drop B, Outro
    writer.set_orders(&[pat0, pat1, pat2, pat2, pat3, pat4, pat4, pat5]);

    // Set song message
    writer.set_message("Nether Storm - DnB @ 174 BPM\nGenerated by gen-tracker-demo-it\nNethercore Project");

    (writer.write(), samples)
}

// Helper: F minor note values
// F2=29, G2=31, Ab2=32, Bb2=34, C3=36, Db3=37, Eb3=39, F3=41
const F2: u8 = 29;
const G2: u8 = 31;
const AB2: u8 = 32;
const BB2: u8 = 34;
const C3: u8 = 36;
const DB3: u8 = 37;
const EB3: u8 = 39;
const F3: u8 = 41;
const F4: u8 = 53;
const AB4: u8 = 56;
const C5: u8 = 60;

fn build_intro_pattern(writer: &mut ItWriter, pat: u8) {
    // Intro: Atmosphere builds, sparse elements
    // Ch14 (Atmos): Sustained atmosphere
    writer.set_note(pat, 0, 14, ItNote::play_note(F3, 15, 32));

    // Ch12 (Riser): Building tension
    writer.set_note(pat, 32, 12, ItNote::play_note(F4, 13, 40));

    // Sparse hi-hats
    for row in (8..64).step_by(16) {
        writer.set_note(pat, row as u16, 2, ItNote::play_note(C5, 3, 20));
    }
}

fn build_build_pattern(writer: &mut ItWriter, pat: u8) {
    // Build: Drums come in, tension builds
    // Ch0 (Kick): Quarter notes
    for row in (0..64).step_by(16) {
        writer.set_note(pat, row as u16, 0, ItNote::play_note(F2, 1, 64));
    }

    // Ch2 (HH): Eighth notes
    for row in (0..64).step_by(8) {
        writer.set_note(pat, row as u16, 2, ItNote::play_note(C5, 3, 48));
    }

    // Ch9 (Pad): F minor pad
    writer.set_note(pat, 0, 9, ItNote::play_note(F3, 10, 48));

    // Ch12 (Riser): Full riser
    writer.set_note(pat, 0, 12, ItNote::play_note(F4, 13, 50));

    // Snare rolls at end
    for row in 48..64 {
        if row % 4 == 0 {
            writer.set_note(pat, row as u16, 1, ItNote::play_note(C5, 2, 50));
        }
    }
}

fn build_drop_a_pattern(writer: &mut ItWriter, pat: u8) {
    // Drop A: Full energy DnB beat
    // Classic DnB pattern: kick on 1 and 2.5, snare on 2 and 4
    // At 174 BPM, 16 rows = 1 beat (speed 3)

    for bar in 0..4 {
        let base = bar * 16;

        // Kick on beat 1
        writer.set_note(pat, base, 0, ItNote::play_note(F2, 1, 64));
        // Kick on beat 2.5 (row 10)
        writer.set_note(pat, base + 10, 0, ItNote::play_note(F2, 1, 60));

        // Snare on beat 2 (row 4) and beat 4 (row 12)
        writer.set_note(pat, base + 4, 1, ItNote::play_note(C5, 2, 64));
        writer.set_note(pat, base + 12, 1, ItNote::play_note(C5, 2, 64));

        // Hi-hats - 16th notes
        for row in (0..16).step_by(4) {
            writer.set_note(
                pat,
                (base + row) as u16,
                2,
                ItNote::play_note(C5, 3, if row % 8 == 0 { 55 } else { 40 }),
            );
        }
    }

    // Sub bass - F minor root
    writer.set_note(pat, 0, 6, ItNote::play_note(F2, 7, 64));
    writer.set_note(pat, 32, 6, ItNote::play_note(AB2, 7, 64));

    // Reese bass pattern
    writer.set_note(pat, 0, 7, ItNote::play_note(F3, 8, 56));
    writer.set_note(pat, 16, 7, ItNote::play_note(EB3, 8, 52));
    writer.set_note(pat, 32, 7, ItNote::play_note(AB2, 8, 56));
    writer.set_note(pat, 48, 7, ItNote::play_note(C3, 8, 52));

    // Lead stabs
    writer.set_note(pat, 0, 10, ItNote::play_note(F4, 11, 50));
    writer.set_note(pat, 24, 10, ItNote::play_note(AB4, 11, 45));
}

fn build_breakdown_pattern(writer: &mut ItWriter, pat: u8) {
    // Breakdown: Drums drop, atmospheric
    // Pad sustains
    writer.set_note(pat, 0, 9, ItNote::play_note(F3, 10, 50));

    // Sparse kick
    writer.set_note(pat, 0, 0, ItNote::play_note(F2, 1, 50));
    writer.set_note(pat, 32, 0, ItNote::play_note(F2, 1, 45));

    // Hi-hat pattern - half time feel
    for row in (0..64).step_by(16) {
        writer.set_note(pat, row as u16, 2, ItNote::play_note(C5, 3, 35));
    }

    // Atmosphere
    writer.set_note(pat, 0, 14, ItNote::play_note(F3, 15, 40));

    // Riser building for next drop
    writer.set_note(pat, 32, 12, ItNote::play_note(F4, 13, 45));
}

fn build_drop_b_pattern(writer: &mut ItWriter, pat: u8) {
    // Drop B (Climax): Maximum energy
    // Same as Drop A but with more elements

    for bar in 0..4 {
        let base = bar * 16;

        // Kick pattern - more complex
        writer.set_note(pat, base, 0, ItNote::play_note(F2, 1, 64));
        writer.set_note(pat, base + 6, 0, ItNote::play_note(F2, 1, 55));
        writer.set_note(pat, base + 10, 0, ItNote::play_note(F2, 1, 60));

        // Snare
        writer.set_note(pat, base + 4, 1, ItNote::play_note(C5, 2, 64));
        writer.set_note(pat, base + 12, 1, ItNote::play_note(C5, 2, 64));

        // Hi-hats - faster
        for row in (0..16).step_by(2) {
            let vol = if row % 4 == 0 { 55 } else { 35 };
            writer.set_note(pat, (base + row) as u16, 2, ItNote::play_note(C5, 3, vol));
        }
    }

    // Sub + Reese + Wobble layered
    writer.set_note(pat, 0, 6, ItNote::play_note(F2, 7, 64));
    writer.set_note(pat, 0, 7, ItNote::play_note(F3, 8, 50));
    writer.set_note(pat, 0, 8, ItNote::play_note(F3, 9, 45));

    writer.set_note(pat, 32, 6, ItNote::play_note(AB2, 7, 64));
    writer.set_note(pat, 32, 7, ItNote::play_note(AB2, 8, 50));
    writer.set_note(pat, 32, 8, ItNote::play_note(AB2, 9, 45));

    // Main lead melody
    writer.set_note(pat, 0, 11, ItNote::play_note(F4, 12, 55));
    writer.set_note(pat, 8, 11, ItNote::play_note(AB4, 12, 50));
    writer.set_note(pat, 16, 11, ItNote::play_note(C5, 12, 55));
    writer.set_note(pat, 32, 11, ItNote::play_note(EB3 + 24, 12, 52)); // Eb5

    // Impact at start
    writer.set_note(pat, 0, 13, ItNote::play_note(F3, 14, 60));
}

fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
    // Outro: Elements drop out, transition to loop

    // Sparse kick
    writer.set_note(pat, 0, 0, ItNote::play_note(F2, 1, 55));
    writer.set_note(pat, 32, 0, ItNote::play_note(F2, 1, 45));

    // Sub bass fading
    writer.set_note(pat, 0, 6, ItNote::play_note(F2, 7, 50));

    // Pad sustain
    writer.set_note(pat, 0, 9, ItNote::play_note(F3, 10, 40));

    // Cymbal decay
    writer.set_note(pat, 0, 5, ItNote::play_note(C5, 6, 35));

    // Atmosphere continues
    writer.set_note(pat, 0, 14, ItNote::play_note(F3, 15, 35));
}
