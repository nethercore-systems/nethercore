//! Nether Mist - Ambient IT generator
//!
//! 70 BPM, D minor/Aeolian, 12 channels
//! Features: Slow pads, drones, bells, atmospheric textures

use super::{make_instrument_continue, make_instrument_fade, make_sample};
use crate::synthesizers;
use nether_it::{ItFlags, ItNote, ItWriter};

/// Generate the Nether Mist IT file
pub fn generate_mist_it() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
    let mut writer = ItWriter::new("Nether Mist");

    // Set up module parameters
    writer.set_channels(12);
    writer.set_speed(6); // Slow for ambient
    writer.set_tempo(70);
    writer.set_global_volume(128);
    writer.set_mix_volume(64);
    writer.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES);

    // Generate and collect samples
    let mut samples = Vec::new();
    let sample_rate = 22050;

    // 1. Sub pad
    let sub_data = synthesizers::generate_pad_sub();
    samples.push(("sub_drone", sub_data));
    let sub_sample = make_sample("sub_drone", sample_rate);
    writer.add_sample(sub_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("sub_drone", 1, 512));

    // 2. Air pad
    let air_data = synthesizers::generate_pad_air();
    samples.push(("air_pad", air_data));
    let air_sample = make_sample("air_pad", sample_rate);
    writer.add_sample(air_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("air_pad", 2, 512));

    // 3. Warm pad
    let warm_data = synthesizers::generate_pad_warm();
    samples.push(("warm_pad", warm_data));
    let warm_sample = make_sample("warm_pad", sample_rate);
    writer.add_sample(warm_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("warm_pad", 3, 512));

    // 4. Cold pad
    let cold_data = synthesizers::generate_pad_cold();
    samples.push(("cold_pad", cold_data));
    let cold_sample = make_sample("cold_pad", sample_rate);
    writer.add_sample(cold_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("cold_pad", 4, 512));

    // 5. Noise breath
    let breath_data = synthesizers::generate_noise_breath();
    samples.push(("breath", breath_data));
    let breath_sample = make_sample("breath", sample_rate);
    writer.add_sample(breath_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("breath", 5));

    // 6. Glass bell
    let bell_data = synthesizers::generate_bell_glass();
    samples.push(("glass_bell", bell_data));
    let bell_sample = make_sample("glass_bell", sample_rate);
    writer.add_sample(bell_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("glass_bell", 6));

    // 7. Sub bass
    let bass_data = synthesizers::generate_bass_sub();
    samples.push(("deep_bass", bass_data));
    let bass_sample = make_sample("deep_bass", sample_rate);
    writer.add_sample(bass_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("deep_bass", 7, 256));

    // 8. Ghost lead
    let ghost_data = synthesizers::generate_lead_ghost();
    samples.push(("ghost_lead", ghost_data));
    let ghost_sample = make_sample("ghost_lead", sample_rate);
    writer.add_sample(ghost_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("ghost_lead", 8, 384));

    // 9. Reverb sim
    let reverb_data = synthesizers::generate_reverb_sim();
    samples.push(("reverb", reverb_data));
    let reverb_sample = make_sample("reverb", sample_rate);
    writer.add_sample(reverb_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("reverb", 9));

    // 10. Wind atmosphere
    let wind_data = synthesizers::generate_atmos_wind();
    samples.push(("wind", wind_data));
    let wind_sample = make_sample("wind", sample_rate);
    writer.add_sample(wind_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("wind", 10));

    // 11. Dark hit (accent)
    let hit_data = synthesizers::generate_hit_dark();
    samples.push(("dark_hit", hit_data));
    let hit_sample = make_sample("dark_hit", sample_rate);
    writer.add_sample(hit_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("dark_hit", 11));

    // 12. Echo lead (counter melody)
    let echo_data = synthesizers::generate_lead_echo();
    samples.push(("echo_lead", echo_data));
    let echo_sample = make_sample("echo_lead", sample_rate);
    writer.add_sample(echo_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("echo_lead", 12, 384));

    // Create patterns (6 patterns, 64 rows each)
    // Pattern 0: Intro (emergence)
    let pat0 = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat0);

    // Pattern 1: A1 (texture)
    let pat1 = writer.add_pattern(64);
    build_texture_pattern(&mut writer, pat1);

    // Pattern 2: A2 (thickening)
    let pat2 = writer.add_pattern(64);
    build_thick_pattern(&mut writer, pat2);

    // Pattern 3: B (descent)
    let pat3 = writer.add_pattern(64);
    build_descent_pattern(&mut writer, pat3);

    // Pattern 4: A3 (resolution)
    let pat4 = writer.add_pattern(64);
    build_resolution_pattern(&mut writer, pat4);

    // Pattern 5: Outro
    let pat5 = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat5);

    // Order table with loop
    writer.set_orders(&[pat0, pat1, pat2, pat1, pat2, pat3, pat4, pat1, pat2, pat5]);

    writer.set_message("Nether Mist - Ambient @ 70 BPM\nD minor/Aeolian\nGenerated by gen-tracker-demo-it");

    (writer.write(), samples)
}

// D minor/Aeolian notes
// D2=26, E2=28, F2=29, G2=31, A2=33, Bb2=34, C3=36, D3=38
// D4=50, E4=52, F4=53, G4=55, A4=57, Bb4=58, C5=60, D5=62
const D2: u8 = 26;
const A2: u8 = 33;
const BB2: u8 = 34;
const D3: u8 = 38;
const F3: u8 = 41;
const A3: u8 = 45;
const D4: u8 = 50;
const E4: u8 = 52;
const F4: u8 = 53;
const A4: u8 = 57;
const D5: u8 = 62;

fn build_intro_pattern(writer: &mut ItWriter, pat: u8) {
    // Intro: Emergence from silence
    // Wind fades in first (retriggered for continuous texture)
    writer.set_note(pat, 0, 9, ItNote::play_note(D3, 10, 15));
    writer.set_note(pat, 32, 9, ItNote::play_note(D3, 10, 18));

    // Sub drone enters slowly
    writer.set_note(pat, 16, 0, ItNote::play_note(D2, 1, 22));
    writer.set_note(pat, 48, 0, ItNote::play_note(D2, 1, 26));

    // Breath texture
    writer.set_note(pat, 24, 4, ItNote::play_note(D4, 5, 18));
    writer.set_note(pat, 48, 4, ItNote::play_note(A3, 5, 20));

    // Bell accents (every 12 rows)
    writer.set_note(pat, 36, 5, ItNote::play_note(D5, 6, 28));
    writer.set_note(pat, 52, 5, ItNote::play_note(A4, 6, 30));

    // Ghost melody hint at end
    writer.set_note(pat, 56, 7, ItNote::play_note(D4, 8, 25));
}

fn build_texture_pattern(writer: &mut ItWriter, pat: u8) {
    // A1: Basic texture established
    // Sub drone continuous
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 38));
    writer.set_note(pat, 32, 0, ItNote::play_note(D2, 1, 40));

    // Warm pad layer - chord movement
    writer.set_note(pat, 0, 2, ItNote::play_note(D3, 3, 35));
    writer.set_note(pat, 24, 2, ItNote::play_note(F3, 3, 33));
    writer.set_note(pat, 48, 2, ItNote::play_note(A2, 3, 35));

    // Air pad (high)
    writer.set_note(pat, 8, 1, ItNote::play_note(D5, 2, 30));
    writer.set_note(pat, 40, 1, ItNote::play_note(A4, 2, 28));

    // Wind continuous (retrigger every 24 rows)
    writer.set_note(pat, 0, 9, ItNote::play_note(D3, 10, 24));
    writer.set_note(pat, 24, 9, ItNote::play_note(D3, 10, 26));
    writer.set_note(pat, 48, 9, ItNote::play_note(D3, 10, 25));

    // Bell accents (every 10-12 rows)
    writer.set_note(pat, 8, 5, ItNote::play_note(A4, 6, 35));
    writer.set_note(pat, 20, 5, ItNote::play_note(D5, 6, 32));
    writer.set_note(pat, 32, 5, ItNote::play_note(F4, 6, 34));
    writer.set_note(pat, 48, 5, ItNote::play_note(A4, 6, 30));

    // Breath texture (retriggered)
    writer.set_note(pat, 16, 4, ItNote::play_note(D4, 5, 28));
    writer.set_note(pat, 48, 4, ItNote::play_note(A3, 5, 26));

    // Bass on root
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 42));
    writer.set_note(pat, 32, 6, ItNote::play_note(A2, 7, 40));

    // Ghost melody enters
    writer.set_note(pat, 24, 7, ItNote::play_note(D4, 8, 32));
    writer.set_note(pat, 56, 7, ItNote::play_note(F4, 8, 30));
}

fn build_thick_pattern(writer: &mut ItWriter, pat: u8) {
    // A2: More layers, thickening - PEAK DENSITY
    // All pads active with movement
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 48));
    writer.set_note(pat, 32, 0, ItNote::play_note(D2, 1, 50));

    writer.set_note(pat, 0, 1, ItNote::play_note(D5, 2, 38));
    writer.set_note(pat, 32, 1, ItNote::play_note(A4, 2, 36));

    writer.set_note(pat, 0, 2, ItNote::play_note(D3, 3, 45));
    writer.set_note(pat, 24, 2, ItNote::play_note(F3, 3, 43));
    writer.set_note(pat, 48, 2, ItNote::play_note(A3, 3, 45));

    writer.set_note(pat, 0, 3, ItNote::play_note(A3, 4, 40));
    writer.set_note(pat, 32, 3, ItNote::play_note(F3, 4, 42));

    // Ghost lead melody (more active)
    writer.set_note(pat, 8, 7, ItNote::play_note(D4, 8, 44));
    writer.set_note(pat, 20, 7, ItNote::play_note(F4, 8, 42));
    writer.set_note(pat, 36, 7, ItNote::play_note(E4, 8, 46));
    writer.set_note(pat, 52, 7, ItNote::play_note(D4, 8, 44));

    // Bells every 8-10 rows
    writer.set_note(pat, 0, 5, ItNote::play_note(A4, 6, 45));
    writer.set_note(pat, 10, 5, ItNote::play_note(D5, 6, 43));
    writer.set_note(pat, 20, 5, ItNote::play_note(F4, 6, 42));
    writer.set_note(pat, 32, 5, ItNote::play_note(A4, 6, 48));
    writer.set_note(pat, 44, 5, ItNote::play_note(D5, 6, 44));
    writer.set_note(pat, 56, 5, ItNote::play_note(F4, 6, 42));

    // Breath texture (continuous)
    writer.set_note(pat, 8, 4, ItNote::play_note(D4, 5, 36));
    writer.set_note(pat, 32, 4, ItNote::play_note(A3, 5, 34));
    writer.set_note(pat, 56, 4, ItNote::play_note(D4, 5, 32));

    // Bass movement
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 52));
    writer.set_note(pat, 24, 6, ItNote::play_note(F3 - 12, 7, 50)); // F2
    writer.set_note(pat, 48, 6, ItNote::play_note(A2, 7, 52));

    // Wind continuous
    writer.set_note(pat, 0, 9, ItNote::play_note(D3, 10, 32));
    writer.set_note(pat, 32, 9, ItNote::play_note(D3, 10, 35));

    // NEW: Accent hit at climax points
    writer.set_note(pat, 0, 10, ItNote::play_note(D2, 11, 48));
    writer.set_note(pat, 32, 10, ItNote::play_note(D2, 11, 50));

    // NEW: Counter melody echo
    writer.set_note(pat, 16, 11, ItNote::play_note(A3, 12, 38));
    writer.set_note(pat, 44, 11, ItNote::play_note(D4, 12, 36));
}

fn build_descent_pattern(writer: &mut ItWriter, pat: u8) {
    // B: Descent - darker, lower, building tension
    // Cold pad takes over with Bb for tension
    writer.set_note(pat, 0, 3, ItNote::play_note(D3, 4, 45));
    writer.set_note(pat, 24, 3, ItNote::play_note(BB2, 4, 48)); // Bb for tension
    writer.set_note(pat, 48, 3, ItNote::play_note(A2, 4, 50));

    // Sub stays low, more prominent
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 48));
    writer.set_note(pat, 32, 0, ItNote::play_note(D2, 1, 50));

    // Bass descends chromatically
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 50));
    writer.set_note(pat, 16, 6, ItNote::play_note(BB2 - 12, 7, 48)); // Bb1
    writer.set_note(pat, 32, 6, ItNote::play_note(A2 - 12, 7, 50)); // A1
    writer.set_note(pat, 48, 6, ItNote::play_note(D2, 7, 48));

    // Reverb swells
    writer.set_note(pat, 8, 8, ItNote::play_note(D4, 9, 38));
    writer.set_note(pat, 24, 8, ItNote::play_note(A3, 9, 40));
    writer.set_note(pat, 48, 8, ItNote::play_note(D4, 9, 38));

    // Ghost melody becomes plaintive
    writer.set_note(pat, 0, 7, ItNote::play_note(A4, 8, 45));
    writer.set_note(pat, 16, 7, ItNote::play_note(F4, 8, 43));
    writer.set_note(pat, 32, 7, ItNote::play_note(E4, 8, 46));
    writer.set_note(pat, 48, 7, ItNote::play_note(D4, 8, 44));

    // Bells more sparse but present
    writer.set_note(pat, 12, 5, ItNote::play_note(D4, 6, 40));
    writer.set_note(pat, 36, 5, ItNote::play_note(A4, 6, 38));
    writer.set_note(pat, 56, 5, ItNote::play_note(F4, 6, 36));

    // Wind more prominent
    writer.set_note(pat, 0, 9, ItNote::play_note(D3, 10, 38));
    writer.set_note(pat, 32, 9, ItNote::play_note(D3, 10, 40));

    // Breath continues
    writer.set_note(pat, 16, 4, ItNote::play_note(A3, 5, 35));
    writer.set_note(pat, 48, 4, ItNote::play_note(D4, 5, 33));

    // Accent hits building unease
    writer.set_note(pat, 24, 10, ItNote::play_note(D2, 11, 50));
    writer.set_note(pat, 56, 10, ItNote::play_note(A2 - 12, 11, 52));

    // Counter melody echoes
    writer.set_note(pat, 8, 11, ItNote::play_note(F4, 12, 36));
    writer.set_note(pat, 40, 11, ItNote::play_note(D4, 12, 34));
}

fn build_resolution_pattern(writer: &mut ItWriter, pat: u8) {
    // A3: Resolution - return to warmth
    // Warm pad returns strongly
    writer.set_note(pat, 0, 2, ItNote::play_note(D3, 3, 45));
    writer.set_note(pat, 24, 2, ItNote::play_note(F3, 3, 43));
    writer.set_note(pat, 48, 2, ItNote::play_note(A3, 3, 45));

    // Sub drone
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 44));
    writer.set_note(pat, 32, 0, ItNote::play_note(D2, 1, 46));

    // Air pad
    writer.set_note(pat, 0, 1, ItNote::play_note(D5, 2, 36));
    writer.set_note(pat, 32, 1, ItNote::play_note(A4, 2, 34));

    // Ghost lead resolves melodically
    writer.set_note(pat, 0, 7, ItNote::play_note(A4, 8, 42));
    writer.set_note(pat, 16, 7, ItNote::play_note(F4, 8, 40));
    writer.set_note(pat, 32, 7, ItNote::play_note(E4, 8, 42));
    writer.set_note(pat, 48, 7, ItNote::play_note(D4, 8, 45));

    // Bells in harmony
    writer.set_note(pat, 8, 5, ItNote::play_note(D5, 6, 40));
    writer.set_note(pat, 24, 5, ItNote::play_note(A4, 6, 38));
    writer.set_note(pat, 40, 5, ItNote::play_note(F4, 6, 36));
    writer.set_note(pat, 56, 5, ItNote::play_note(D5, 6, 40));

    // Bass on root
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 48));
    writer.set_note(pat, 32, 6, ItNote::play_note(A2, 7, 46));

    // Wind
    writer.set_note(pat, 0, 9, ItNote::play_note(D3, 10, 30));
    writer.set_note(pat, 32, 9, ItNote::play_note(D3, 10, 32));

    // Breath
    writer.set_note(pat, 16, 4, ItNote::play_note(D4, 5, 30));
    writer.set_note(pat, 48, 4, ItNote::play_note(A3, 5, 28));

    // Counter melody
    writer.set_note(pat, 24, 11, ItNote::play_note(D4, 12, 32));
}

fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
    // Outro: Fade to silence
    // Sub drone fading
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 28));
    writer.set_note(pat, 32, 0, ItNote::play_note(D2, 1, 22));

    // Warm pad fades
    writer.set_note(pat, 0, 2, ItNote::play_note(D3, 3, 26));
    writer.set_note(pat, 32, 2, ItNote::play_note(A2, 3, 20));

    // Wind continues quietly
    writer.set_note(pat, 0, 9, ItNote::play_note(D3, 10, 18));
    writer.set_note(pat, 32, 9, ItNote::play_note(D3, 10, 15));

    // Final bells (gentle)
    writer.set_note(pat, 16, 5, ItNote::play_note(A4, 6, 24));
    writer.set_note(pat, 40, 5, ItNote::play_note(D5, 6, 20));

    // Ghost melody final note
    writer.set_note(pat, 48, 7, ItNote::play_note(D4, 8, 24));

    // Reverb trail
    writer.set_note(pat, 56, 8, ItNote::play_note(D4, 9, 18));

    // Breath fades
    writer.set_note(pat, 24, 4, ItNote::play_note(D4, 5, 18));
}
