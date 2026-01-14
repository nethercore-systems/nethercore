//! Nether Dawn - Epic/Orchestral IT generator
//!
//! 90 BPM, D major (Lydian mode), 16 channels
//! Features: Strings with NNA Continue, choir with NNA Fade, brass fanfares

use super::{make_instrument, make_instrument_continue, make_instrument_fade, make_sample};
use crate::synthesizers;
use nether_it::{ItFlags, ItWriter};

mod constants;
mod patterns;

use patterns::*;

/// Generate stripped IT file (no sample data, for ROM/external samples)
pub fn generate_dawn_it_stripped() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
    let mut writer = ItWriter::new("Nether Dawn");

    // Set up module parameters
    writer.set_channels(16);
    writer.set_speed(6);
    writer.set_tempo(90);
    writer.set_global_volume(128);
    writer.set_mix_volume(80);
    writer.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES);

    // Generate and collect samples
    let mut samples = Vec::new();
    let sample_rate = 22050;

    // 1. Cello (strings low)
    let cello_data = synthesizers::generate_strings_cello();
    samples.push(("cello", cello_data));
    let cello_sample = make_sample("cello", sample_rate);
    writer.add_sample_header_only(cello_sample);
    writer.add_instrument(make_instrument_continue("cello", 1));

    // 2. Viola (strings mid)
    let viola_data = synthesizers::generate_strings_viola();
    samples.push(("viola", viola_data));
    let viola_sample = make_sample("viola", sample_rate);
    writer.add_sample_header_only(viola_sample);
    writer.add_instrument(make_instrument_continue("viola", 2));

    // 3. Violin (strings high)
    let violin_data = synthesizers::generate_strings_violin();
    samples.push(("violin", violin_data));
    let violin_sample = make_sample("violin", sample_rate);
    writer.add_sample_header_only(violin_sample);
    writer.add_instrument(make_instrument_continue("violin", 3));

    // 4. French horn (brass low)
    let horn_data = synthesizers::generate_brass_horn();
    samples.push(("horn", horn_data));
    let horn_sample = make_sample("horn", sample_rate);
    writer.add_sample_header_only(horn_sample);
    writer.add_instrument(make_instrument("horn", 4));

    // 5. Trumpet (brass high)
    let trumpet_data = synthesizers::generate_brass_trumpet();
    samples.push(("trumpet", trumpet_data));
    let trumpet_sample = make_sample("trumpet", sample_rate);
    writer.add_sample_header_only(trumpet_sample);
    writer.add_instrument(make_instrument("trumpet", 5));

    // 6. Flute
    let flute_data = synthesizers::generate_flute();
    samples.push(("flute", flute_data));
    let flute_sample = make_sample("flute", sample_rate);
    writer.add_sample_header_only(flute_sample);
    writer.add_instrument(make_instrument("flute", 6));

    // 7. Timpani
    let timpani_data = synthesizers::generate_timpani();
    samples.push(("timpani", timpani_data));
    let timpani_sample = make_sample("timpani", sample_rate);
    writer.add_sample_header_only(timpani_sample);
    writer.add_instrument(make_instrument("timpani", 7));

    // 8. Snare (orchestral)
    let snare_data = synthesizers::generate_snare_orch();
    samples.push(("snare_roll", snare_data));
    let snare_sample = make_sample("snare_roll", sample_rate);
    writer.add_sample_header_only(snare_sample);
    writer.add_instrument(make_instrument("snare_roll", 8));

    // 9. Cymbal crash
    let cymbal_data = synthesizers::generate_cymbal_crash();
    samples.push(("cymbal_crash", cymbal_data));
    let cymbal_sample = make_sample("cymbal_crash", sample_rate);
    writer.add_sample_header_only(cymbal_sample);
    writer.add_instrument(make_instrument("cymbal_crash", 9));

    // 10. Harp
    let harp_data = synthesizers::generate_harp_gliss();
    samples.push(("harp", harp_data));
    let harp_sample = make_sample("harp", sample_rate);
    writer.add_sample_header_only(harp_sample);
    writer.add_instrument(make_instrument("harp", 10));

    // 11. Choir Ah
    let choir_ah_data = synthesizers::generate_choir_ah();
    samples.push(("choir_ah", choir_ah_data));
    let choir_ah_sample = make_sample("choir_ah", sample_rate);
    writer.add_sample_header_only(choir_ah_sample);
    writer.add_instrument(make_instrument_fade("choir_ah", 11, 768));

    // 12. Choir Oh
    let choir_oh_data = synthesizers::generate_choir_oh();
    samples.push(("choir_oh", choir_oh_data));
    let choir_oh_sample = make_sample("choir_oh", sample_rate);
    writer.add_sample_header_only(choir_oh_sample);
    writer.add_instrument(make_instrument_fade("choir_oh", 12, 768));

    // 13. Piano
    let piano_data = synthesizers::generate_piano();
    samples.push(("piano", piano_data));
    let piano_sample = make_sample("piano", sample_rate);
    writer.add_sample_header_only(piano_sample);
    writer.add_instrument(make_instrument("piano", 13));

    // 14. Epic bass
    let bass_data = synthesizers::generate_bass_epic();
    samples.push(("epic_bass", bass_data));
    let bass_sample = make_sample("epic_bass", sample_rate);
    writer.add_sample_header_only(bass_sample);
    writer.add_instrument(make_instrument("epic_bass", 14));

    // 15. Orchestra pad
    let pad_data = synthesizers::generate_pad_orchestra();
    samples.push(("pad", pad_data));
    let pad_sample = make_sample("pad", sample_rate);
    writer.add_sample_header_only(pad_sample);
    writer.add_instrument(make_instrument_continue("pad", 15));

    // 16. Epic FX
    let fx_data = synthesizers::generate_fx_epic();
    samples.push(("fx_epic", fx_data));
    let fx_sample = make_sample("fx_epic", sample_rate);
    writer.add_sample_header_only(fx_sample);
    writer.add_instrument(make_instrument("fx_epic", 16));

    // Create patterns (6 patterns, 64 rows each)
    // Pattern 0: Intro
    let pat0 = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat0);

    // Pattern 1: A1 (Theme statement)
    let pat1 = writer.add_pattern(64);
    build_theme_a1_pattern(&mut writer, pat1);

    // Pattern 2: A2 (Theme variation)
    let pat2 = writer.add_pattern(64);
    build_theme_a2_pattern(&mut writer, pat2);

    // Pattern 3: B (Development)
    let pat3 = writer.add_pattern(64);
    build_development_pattern(&mut writer, pat3);

    // Pattern 4: C (Climax)
    let pat4 = writer.add_pattern(64);
    build_climax_pattern(&mut writer, pat4);

    // Pattern 5: Outro
    let pat5 = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat5);

    // Order: Intro, A1, A2, A1, A2, B, C, A1, A2, Outro
    writer.set_orders(&[pat0, pat1, pat2, pat1, pat2, pat3, pat4, pat1, pat2, pat5]);

    writer.set_message("Nether Dawn - Epic Orchestral @ 90 BPM\nD major (Lydian)\nGenerated by gen-tracker-demo-it");

    (writer.write(), samples)
}

/// Generate embedded IT file (with sample data)
pub fn generate_dawn_it_embedded() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
    let mut writer = ItWriter::new("Nether Dawn");

    // Set up module parameters
    writer.set_channels(16);
    writer.set_speed(6);
    writer.set_tempo(90);
    writer.set_global_volume(128);
    writer.set_mix_volume(80);
    writer.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS | ItFlags::LINEAR_SLIDES);

    // Generate and collect samples
    let mut samples = Vec::new();
    let sample_rate = 22050;

    // 1. Cello (strings low)
    let cello_data = synthesizers::generate_strings_cello();
    samples.push(("cello", cello_data));
    let cello_sample = make_sample("cello", sample_rate);
    writer.add_sample(cello_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("cello", 1));

    // 2. Viola (strings mid)
    let viola_data = synthesizers::generate_strings_viola();
    samples.push(("viola", viola_data));
    let viola_sample = make_sample("viola", sample_rate);
    writer.add_sample(viola_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("viola", 2));

    // 3. Violin (strings high)
    let violin_data = synthesizers::generate_strings_violin();
    samples.push(("violin", violin_data));
    let violin_sample = make_sample("violin", sample_rate);
    writer.add_sample(violin_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("violin", 3));

    // 4. French horn (brass low)
    let horn_data = synthesizers::generate_brass_horn();
    samples.push(("horn", horn_data));
    let horn_sample = make_sample("horn", sample_rate);
    writer.add_sample(horn_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("horn", 4));

    // 5. Trumpet (brass high)
    let trumpet_data = synthesizers::generate_brass_trumpet();
    samples.push(("trumpet", trumpet_data));
    let trumpet_sample = make_sample("trumpet", sample_rate);
    writer.add_sample(trumpet_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("trumpet", 5));

    // 6. Flute
    let flute_data = synthesizers::generate_flute();
    samples.push(("flute", flute_data));
    let flute_sample = make_sample("flute", sample_rate);
    writer.add_sample(flute_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("flute", 6));

    // 7. Timpani
    let timpani_data = synthesizers::generate_timpani();
    samples.push(("timpani", timpani_data));
    let timpani_sample = make_sample("timpani", sample_rate);
    writer.add_sample(timpani_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("timpani", 7));

    // 8. Snare (orchestral)
    let snare_data = synthesizers::generate_snare_orch();
    samples.push(("snare_roll", snare_data));
    let snare_sample = make_sample("snare_roll", sample_rate);
    writer.add_sample(snare_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("snare_roll", 8));

    // 9. Cymbal crash
    let cymbal_data = synthesizers::generate_cymbal_crash();
    samples.push(("cymbal_crash", cymbal_data));
    let cymbal_sample = make_sample("cymbal_crash", sample_rate);
    writer.add_sample(cymbal_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("cymbal_crash", 9));

    // 10. Harp
    let harp_data = synthesizers::generate_harp_gliss();
    samples.push(("harp", harp_data));
    let harp_sample = make_sample("harp", sample_rate);
    writer.add_sample(harp_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("harp", 10));

    // 11. Choir Ah
    let choir_ah_data = synthesizers::generate_choir_ah();
    samples.push(("choir_ah", choir_ah_data));
    let choir_ah_sample = make_sample("choir_ah", sample_rate);
    writer.add_sample(choir_ah_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("choir_ah", 11, 768));

    // 12. Choir Oh
    let choir_oh_data = synthesizers::generate_choir_oh();
    samples.push(("choir_oh", choir_oh_data));
    let choir_oh_sample = make_sample("choir_oh", sample_rate);
    writer.add_sample(choir_oh_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("choir_oh", 12, 768));

    // 13. Piano
    let piano_data = synthesizers::generate_piano();
    samples.push(("piano", piano_data));
    let piano_sample = make_sample("piano", sample_rate);
    writer.add_sample(piano_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("piano", 13));

    // 14. Epic bass
    let bass_data = synthesizers::generate_bass_epic();
    samples.push(("epic_bass", bass_data));
    let bass_sample = make_sample("epic_bass", sample_rate);
    writer.add_sample(bass_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("epic_bass", 14));

    // 15. Orchestra pad
    let pad_data = synthesizers::generate_pad_orchestra();
    samples.push(("pad", pad_data));
    let pad_sample = make_sample("pad", sample_rate);
    writer.add_sample(pad_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("pad", 15));

    // 16. Epic FX
    let fx_data = synthesizers::generate_fx_epic();
    samples.push(("fx_epic", fx_data));
    let fx_sample = make_sample("fx_epic", sample_rate);
    writer.add_sample(fx_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("fx_epic", 16));

    // Create patterns (6 patterns, 64 rows each)
    // Pattern 0: Intro
    let pat0 = writer.add_pattern(64);
    build_intro_pattern(&mut writer, pat0);

    // Pattern 1: A1 (Theme statement)
    let pat1 = writer.add_pattern(64);
    build_theme_a1_pattern(&mut writer, pat1);

    // Pattern 2: A2 (Theme variation)
    let pat2 = writer.add_pattern(64);
    build_theme_a2_pattern(&mut writer, pat2);

    // Pattern 3: B (Development)
    let pat3 = writer.add_pattern(64);
    build_development_pattern(&mut writer, pat3);

    // Pattern 4: C (Climax)
    let pat4 = writer.add_pattern(64);
    build_climax_pattern(&mut writer, pat4);

    // Pattern 5: Outro
    let pat5 = writer.add_pattern(64);
    build_outro_pattern(&mut writer, pat5);

    // Order: Intro, A1, A2, A1, A2, B, C, A1, A2, Outro
    writer.set_orders(&[pat0, pat1, pat2, pat1, pat2, pat3, pat4, pat1, pat2, pat5]);

    writer.set_message("Nether Dawn - Epic Orchestral @ 90 BPM\nD major (Lydian)\nGenerated by gen-tracker-demo-it");

    (writer.write(), samples)
}
