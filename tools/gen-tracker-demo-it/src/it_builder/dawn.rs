//! Nether Dawn - Epic/Orchestral IT generator
//!
//! 90 BPM, D major (Lydian mode), 16 channels
//! Features: Strings with NNA Continue, choir with NNA Fade, brass fanfares

use super::{make_instrument, make_instrument_continue, make_instrument_fade, make_sample};
use crate::synthesizers;
use nether_it::{ItFlags, ItNote, ItWriter};

/// Generate the Nether Dawn IT file
pub fn generate_dawn_it() -> (Vec<u8>, Vec<(&'static str, Vec<i16>)>) {
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

// D major (Lydian) notes: D E F# G# A B C#
// D2=26, E2=28, F#2=30, A2=33, B2=35, D3=38, F#3=42, A3=45, B3=47
// D4=50, E4=52, F#4=54, G#4=56, A4=57, B4=59, C#5=61, D5=62, A5=69
const D2: u8 = 26;
const A2: u8 = 33;
const D3: u8 = 38;
const FS3: u8 = 42;
const A3: u8 = 45;
const B3: u8 = 47;
const D4: u8 = 50;
const E4: u8 = 52;
const FS4: u8 = 54;
const GS4: u8 = 56;
const A4: u8 = 57;
const B4: u8 = 59;
const D5: u8 = 62;
const FS5: u8 = 66;
const A5: u8 = 69;

fn build_intro_pattern(writer: &mut ItWriter, pat: u8) {
    // Intro: Dawn breaking - sparse, mysterious
    // Set up stereo panning for all channels (effect 0x18 = X = Set Panning, 32 = center)

    // Ch0 (Cello): Low D drone - starts pp, crescendo via volume slide
    writer.set_note(
        pat,
        0,
        0,
        ItNote::play_note(D2, 1, 20)
            .with_effect(0x18, 20), // Pan left
    );
    // Volume slide up for crescendo
    writer.set_note(
        pat,
        8,
        0,
        ItNote::play_note(D2, 1, 28).with_effect(0x04, 0x02),
    ); // Slow vol up

    // Ch2 (Violin): High A with vibrato - delayed entry, panned center
    writer.set_note(
        pat,
        16,
        2,
        ItNote::play_note(A5, 3, 30)
            .with_effect(0x18, 32), // Center pan
    );
    // Add vibrato on sustain
    writer.set_note(
        pat,
        20,
        2,
        ItNote::play_note(A5, 3, 35).with_effect(0x08, 0x34),
    ); // Vibrato speed=3, depth=4

    // Ch9 (Harp): Rising arpeggio D-F#-A with humanized velocities, panned left
    let harp_vels = [38, 42, 40, 45]; // Varied velocities for natural feel
    writer.set_note(
        pat,
        24,
        9,
        ItNote::play_note(D4, 10, harp_vels[0]).with_effect(0x18, 20),
    ); // Pan left
    writer.set_note(pat, 28, 9, ItNote::play_note(FS4, 10, harp_vels[1]));
    writer.set_note(pat, 32, 9, ItNote::play_note(A4, 10, harp_vels[2]));

    // Ch14 (Pad): Swells in with volume slide, centered
    writer.set_note(
        pat,
        32,
        14,
        ItNote::play_note(D3, 15, 18)
            .with_effect(0x18, 32), // Center
    );
    writer.set_note(
        pat,
        40,
        14,
        ItNote::play_note(D3, 15, 25).with_effect(0x04, 0x02),
    ); // Vol slide up

    // Ch6 (Timpani): Single hit on D, centered
    writer.set_note(
        pat,
        48,
        6,
        ItNote::play_note(D2, 7, 50).with_effect(0x18, 32),
    );

    // Add pickup harp notes leading into theme (anticipation)
    writer.set_note(pat, 56, 9, ItNote::play_note(A4, 10, 38));
    writer.set_note(pat, 58, 9, ItNote::play_note(D5, 10, 42));
    writer.set_note(pat, 60, 9, ItNote::play_note(FS5, 10, 46));
    writer.set_note(pat, 62, 9, ItNote::play_note(A5, 10, 50));
}

fn build_theme_a1_pattern(writer: &mut ItWriter, pat: u8) {
    // A1: Theme statement - D major chord progression
    // I - V/3 - vi - IV (D - A/C# - Bm - G)
    // Downbeat accents: +8 velocity, offbeats: -3

    // Ch0-2 (Strings): Sustained chords with NNA Continue + panning
    writer.set_note(
        pat,
        0,
        0,
        ItNote::play_note(D2, 1, 52).with_effect(0x18, 20),
    ); // Cello left, accent
    writer.set_note(
        pat,
        0,
        1,
        ItNote::play_note(D3, 2, 48).with_effect(0x18, 44),
    ); // Viola right
    writer.set_note(
        pat,
        0,
        2,
        ItNote::play_note(FS4, 3, 45).with_effect(0x18, 32),
    ); // Violin center

    // Chord change at bar 2 (row 16)
    writer.set_note(pat, 16, 0, ItNote::play_note(A2, 1, 48));
    writer.set_note(pat, 16, 1, ItNote::play_note(A3, 2, 44));

    // Chord change at bar 3 (row 32) - accent
    writer.set_note(pat, 32, 0, ItNote::play_note(B3 - 12, 1, 52)); // B2
    writer.set_note(pat, 32, 1, ItNote::play_note(D3, 2, 48));

    // Chord change at bar 4 (row 48)
    writer.set_note(pat, 48, 0, ItNote::play_note(D2 + 5, 1, 48)); // G2
    writer.set_note(pat, 48, 1, ItNote::play_note(B3, 2, 44));

    // Ch2 (Violin): Main melody - Theme A with expressive dynamics
    // Bar 1-2: D4 - F#4 - A4 - D5 (ascending, crescendo)
    writer.set_note(pat, 0, 2, ItNote::play_note(D4, 3, 55)); // Downbeat accent
    writer.set_note(pat, 4, 2, ItNote::play_note(FS4, 3, 50));
    writer.set_note(pat, 8, 2, ItNote::play_note(A4, 3, 56)); // Strong beat
    writer.set_note(
        pat,
        12,
        2,
        ItNote::play_note(D5, 3, 58).with_effect(0x08, 0x34),
    ); // Peak + vibrato
    // Bar 3-4: B4 - A4 - F#4 - E4 (descending with portamento)
    writer.set_note(pat, 32, 2, ItNote::play_note(B4, 3, 54)); // Accent
    writer.set_note(
        pat,
        36,
        2,
        ItNote::play_note(A4, 3, 50).with_effect(0x07, 0x10),
    ); // Portamento
    writer.set_note(
        pat,
        40,
        2,
        ItNote::play_note(FS4, 3, 48).with_effect(0x07, 0x10),
    );
    writer.set_note(
        pat,
        44,
        2,
        ItNote::play_note(E4, 3, 45).with_effect(0x07, 0x10),
    );

    // Ch3 (Horn): Harmonic support with panning
    writer.set_note(
        pat,
        0,
        3,
        ItNote::play_note(D3, 4, 45).with_effect(0x18, 24),
    ); // Left-center
    writer.set_note(pat, 32, 3, ItNote::play_note(B3, 4, 42));

    // Ch6 (Timpani): Downbeats with accent dynamics
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 60)); // Strong accent
    writer.set_note(pat, 32, 6, ItNote::play_note(D2, 7, 55));

    // Ch9 (Harp): Chord arpeggios with humanized velocities
    let harp_vels = [38, 42, 40, 44, 39, 43];
    writer.set_note(pat, 8, 9, ItNote::play_note(D4, 10, harp_vels[0]));
    writer.set_note(pat, 10, 9, ItNote::play_note(FS4, 10, harp_vels[1]));
    writer.set_note(pat, 12, 9, ItNote::play_note(A4, 10, harp_vels[2]));
    // Second arpeggio in bar 3
    writer.set_note(pat, 40, 9, ItNote::play_note(B3, 10, harp_vels[3]));
    writer.set_note(pat, 42, 9, ItNote::play_note(D4, 10, harp_vels[4]));
    writer.set_note(pat, 44, 9, ItNote::play_note(FS4, 10, harp_vels[5]));

    // Ch13 (Bass): Follows roots with accent on downbeats
    writer.set_note(pat, 0, 13, ItNote::play_note(D2, 14, 60)); // Accent
    writer.set_note(pat, 32, 13, ItNote::play_note(B3 - 12, 14, 56));
}

fn build_theme_a2_pattern(writer: &mut ItWriter, pat: u8) {
    // A2: Theme variation - more elements, building intensity
    // Same chord progression as A1 but with fuller orchestration

    // Strings continue with stronger dynamics
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 54)); // Accent
    writer.set_note(pat, 0, 1, ItNote::play_note(D3, 2, 50));
    writer.set_note(pat, 16, 0, ItNote::play_note(A2, 1, 50));
    writer.set_note(pat, 32, 0, ItNote::play_note(B3 - 12, 1, 54)); // Accent
    writer.set_note(pat, 48, 0, ItNote::play_note(D2 + 5, 1, 50));

    // Violin melody continues - Lydian mode showcased
    writer.set_note(pat, 0, 2, ItNote::play_note(D4, 3, 56)); // Accent
    writer.set_note(pat, 4, 2, ItNote::play_note(E4, 3, 52));
    writer.set_note(pat, 8, 2, ItNote::play_note(FS4, 3, 56)); // Strong beat
    writer.set_note(
        pat,
        12,
        2,
        ItNote::play_note(GS4, 3, 60).with_effect(0x08, 0x34),
    ); // Lydian G# + vibrato!
    writer.set_note(
        pat,
        16,
        2,
        ItNote::play_note(A4, 3, 62).with_effect(0x07, 0x18),
    ); // Portamento into peak

    // Ch4 (Trumpet): Fanfare accents with panning
    writer.set_note(
        pat,
        32,
        4,
        ItNote::play_note(D5, 5, 56).with_effect(0x18, 40),
    ); // Right-center
    writer.set_note(pat, 36, 4, ItNote::play_note(FS5, 5, 52));
    // Add echo response
    writer.set_note(pat, 44, 4, ItNote::play_note(A5, 5, 48));

    // Ch7 (Snare): Roll in final bars with crescendo
    for row in (48..64).step_by(2) {
        let vel = 32 + ((row - 48) as u8 * 2); // Crescendo from 32 to 48
        writer.set_note(pat, row as u16, 7, ItNote::play_note(60, 8, vel));
    }

    // Ch10-11 (Choir): Enters with vibrato - NNA Fade creates smooth vowels
    writer.set_note(
        pat,
        0,
        10,
        ItNote::play_note(D4, 11, 40).with_effect(0x18, 28),
    ); // Choir Ah left-center
    writer.set_note(
        pat,
        8,
        10,
        ItNote::play_note(D4, 11, 42).with_effect(0x08, 0x45),
    ); // Add vibrato
    writer.set_note(
        pat,
        32,
        11,
        ItNote::play_note(A3, 12, 38).with_effect(0x18, 36),
    ); // Choir Oh right-center
    writer.set_note(
        pat,
        40,
        11,
        ItNote::play_note(A3, 12, 40).with_effect(0x08, 0x45),
    ); // Vibrato

    // Ch12 (Piano): Countermelody with dynamics
    writer.set_note(pat, 0, 12, ItNote::play_note(FS3, 13, 42));
    writer.set_note(pat, 4, 12, ItNote::play_note(A3, 13, 38));
    writer.set_note(pat, 8, 12, ItNote::play_note(D4, 13, 45)); // Peak
    // Add second phrase
    writer.set_note(pat, 32, 12, ItNote::play_note(E4, 13, 40));
    writer.set_note(pat, 36, 12, ItNote::play_note(D4, 13, 38));
    writer.set_note(pat, 40, 12, ItNote::play_note(B3, 13, 42));

    // Timpani with accents
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 58)); // Accent
    writer.set_note(pat, 32, 6, ItNote::play_note(A2, 7, 55));

    // Bass with accent dynamics
    writer.set_note(pat, 0, 13, ItNote::play_note(D2, 14, 62)); // Accent
    writer.set_note(pat, 32, 13, ItNote::play_note(B3 - 12, 14, 58));
}

fn build_development_pattern(writer: &mut ItWriter, pat: u8) {
    // B: Development - relative minor, tension builds toward climax
    // vi - iii - IV - V (Bm - F#m - G - A)

    // More active strings with crescendo dynamics
    writer.set_note(pat, 0, 0, ItNote::play_note(B3 - 12, 1, 52));
    writer.set_note(pat, 8, 0, ItNote::play_note(FS3 - 12, 1, 50));
    writer.set_note(pat, 16, 0, ItNote::play_note(B3 - 12, 1, 54));
    writer.set_note(
        pat,
        32,
        0,
        ItNote::play_note(D2 + 5, 1, 58).with_effect(0x04, 0x02),
    ); // Vol slide up
    writer.set_note(pat, 48, 0, ItNote::play_note(A2, 1, 62)); // Building to climax

    // Violin runs with portamento for fluid motion
    writer.set_note(pat, 0, 2, ItNote::play_note(B4, 3, 54));
    writer.set_note(
        pat,
        4,
        2,
        ItNote::play_note(D5, 3, 52).with_effect(0x07, 0x10),
    ); // Portamento
    writer.set_note(
        pat,
        8,
        2,
        ItNote::play_note(FS5, 3, 56).with_effect(0x07, 0x10),
    );
    writer.set_note(
        pat,
        16,
        2,
        ItNote::play_note(A5, 3, 60).with_effect(0x08, 0x34),
    ); // Peak with vibrato
    // Second phrase - descending tension
    writer.set_note(pat, 32, 2, ItNote::play_note(FS5, 3, 58));
    writer.set_note(pat, 40, 2, ItNote::play_note(D5, 3, 55));
    writer.set_note(
        pat,
        48,
        2,
        ItNote::play_note(B4, 3, 60).with_effect(0x08, 0x45),
    ); // Tension vibrato

    // Brass becomes prominent - horn then trumpet answer
    writer.set_note(pat, 0, 3, ItNote::play_note(B3, 4, 52));
    writer.set_note(pat, 16, 3, ItNote::play_note(FS3, 4, 50));
    writer.set_note(
        pat,
        32,
        4,
        ItNote::play_note(D5, 5, 58).with_effect(0x18, 40),
    ); // Trumpet right
    writer.set_note(pat, 40, 4, ItNote::play_note(A5, 5, 55));
    writer.set_note(pat, 48, 4, ItNote::play_note(D5 + 12, 5, 60)); // D6 climax prep

    // Flute ornaments with expressive vibrato
    writer.set_note(pat, 8, 5, ItNote::play_note(FS5, 6, 44));
    writer.set_note(
        pat,
        12,
        5,
        ItNote::play_note(A5, 6, 48).with_effect(0x08, 0x34),
    );
    writer.set_note(
        pat,
        32,
        5,
        ItNote::play_note(D5 + 12, 6, 52).with_effect(0x08, 0x34),
    ); // D6 with vibrato

    // Timpani building with crescendo
    writer.set_note(pat, 0, 6, ItNote::play_note(B3 - 12, 7, 55));
    writer.set_note(pat, 16, 6, ItNote::play_note(FS3 - 12, 7, 56));
    writer.set_note(pat, 32, 6, ItNote::play_note(D2 + 5, 7, 60));
    writer.set_note(pat, 48, 6, ItNote::play_note(A2, 7, 64)); // Peak

    // Snare enters for tension
    writer.set_note(pat, 40, 7, ItNote::play_note(60, 8, 35));
    writer.set_note(pat, 44, 7, ItNote::play_note(60, 8, 38));
    writer.set_note(pat, 48, 7, ItNote::play_note(60, 8, 42));
    writer.set_note(pat, 52, 7, ItNote::play_note(60, 8, 45));
    writer.set_note(pat, 56, 7, ItNote::play_note(60, 8, 48));
    writer.set_note(pat, 60, 7, ItNote::play_note(60, 8, 52));

    // Cymbal accents
    writer.set_note(pat, 32, 8, ItNote::play_note(60, 9, 50));

    // Choir sustains with vibrato for emotional tension
    writer.set_note(
        pat,
        0,
        10,
        ItNote::play_note(B3, 11, 45).with_effect(0x08, 0x45),
    );
    writer.set_note(
        pat,
        0,
        11,
        ItNote::play_note(FS4, 12, 42).with_effect(0x08, 0x45),
    );
    // Choir shifts at bar 3
    writer.set_note(pat, 32, 10, ItNote::play_note(D4, 11, 50));
    writer.set_note(pat, 32, 11, ItNote::play_note(A4, 12, 48));

    // FX: Riser with volume slide for tension
    writer.set_note(
        pat,
        32,
        15,
        ItNote::play_note(60, 16, 30).with_effect(0x04, 0x03),
    ); // Vol slide up

    // Bass driving with accent pattern
    writer.set_note(pat, 0, 13, ItNote::play_note(B3 - 12, 14, 60));
    writer.set_note(pat, 16, 13, ItNote::play_note(FS3 - 12, 14, 58));
    writer.set_note(pat, 32, 13, ItNote::play_note(D2 + 5, 14, 62));
    writer.set_note(pat, 48, 13, ItNote::play_note(A2, 14, 64)); // Maximum before climax
}

fn build_climax_pattern(writer: &mut ItWriter, pat: u8) {
    // C: Climax - FULL ORCHESTRA, MAXIMUM ENERGY (ff to fff)
    // I - I/3 - IV - V - vi - IV - I/5 - V - I

    // Strings in high register, fortissimo with vibrato
    writer.set_note(pat, 0, 0, ItNote::play_note(D3, 1, 70)); // ff
    writer.set_note(pat, 0, 1, ItNote::play_note(A3, 2, 68));
    writer.set_note(
        pat,
        0,
        2,
        ItNote::play_note(D5, 3, 72).with_effect(0x08, 0x45),
    ); // Vibrato

    writer.set_note(pat, 16, 0, ItNote::play_note(FS3, 1, 68));
    writer.set_note(
        pat,
        16,
        2,
        ItNote::play_note(FS5, 3, 70).with_effect(0x08, 0x45),
    );

    writer.set_note(pat, 32, 0, ItNote::play_note(D2 + 5, 1, 72));
    writer.set_note(
        pat,
        32,
        2,
        ItNote::play_note(B4, 3, 74).with_effect(0x08, 0x34),
    );

    writer.set_note(pat, 48, 0, ItNote::play_note(D2, 1, 75)); // fff peak
    writer.set_note(
        pat,
        48,
        2,
        ItNote::play_note(D5, 3, 78).with_effect(0x08, 0x45),
    );

    // Brass fanfare - heroic call and response
    writer.set_note(pat, 0, 3, ItNote::play_note(D4, 4, 65));
    writer.set_note(pat, 8, 4, ItNote::play_note(D5, 5, 70));
    writer.set_note(pat, 12, 4, ItNote::play_note(A5, 5, 68));
    writer.set_note(pat, 16, 3, ItNote::play_note(FS4, 4, 65));
    writer.set_note(pat, 24, 4, ItNote::play_note(D5, 5, 72));
    // Second fanfare phrase
    writer.set_note(pat, 32, 3, ItNote::play_note(D4, 4, 68));
    writer.set_note(pat, 40, 4, ItNote::play_note(A5, 5, 72));
    writer.set_note(
        pat,
        48,
        4,
        ItNote::play_note(D5 + 12, 5, 75).with_effect(0x08, 0x34),
    ); // D6 peak

    // Flute soaring with vibrato
    writer.set_note(
        pat,
        0,
        5,
        ItNote::play_note(D5 + 12, 6, 58).with_effect(0x08, 0x34),
    ); // D6
    writer.set_note(
        pat,
        16,
        5,
        ItNote::play_note(A5, 6, 60).with_effect(0x08, 0x34),
    );
    writer.set_note(
        pat,
        32,
        5,
        ItNote::play_note(FS5 + 12, 6, 62).with_effect(0x08, 0x45),
    ); // F#6 peak

    // Timpani rolls with crescendo
    for row in (0..16).step_by(2) {
        let vel = 55 + (row as u8 * 2); // 55 to 85
        writer.set_note(pat, row as u16, 6, ItNote::play_note(D2, 7, vel));
    }
    writer.set_note(pat, 32, 6, ItNote::play_note(D2, 7, 72));
    writer.set_note(pat, 48, 6, ItNote::play_note(D2, 7, 78)); // fff hit

    // Snare driving with accent pattern
    for row in (16..32).step_by(2) {
        let vel = if row % 4 == 0 { 55 } else { 48 }; // Accented pattern
        writer.set_note(pat, row as u16, 7, ItNote::play_note(60, 8, vel));
    }
    // Final roll
    for row in (48..64).step_by(2) {
        writer.set_note(pat, row as u16, 7, ItNote::play_note(60, 8, 50));
    }

    // Cymbal crashes - big accents
    writer.set_note(pat, 0, 8, ItNote::play_note(60, 9, 70)); // Big opening crash
    writer.set_note(pat, 32, 8, ItNote::play_note(60, 9, 72));
    writer.set_note(pat, 48, 8, ItNote::play_note(60, 9, 75)); // Final crash

    // Harp glissando with crescendo
    let harp_vels = [50, 54, 58, 62, 66];
    for (i, note) in [D4, FS4, A4, D5, FS5].iter().enumerate() {
        writer.set_note(pat, (i * 2) as u16, 9, ItNote::play_note(*note, 10, harp_vels[i]));
    }

    // Choir at full power with vibrato - alternating ah/oh
    writer.set_note(
        pat,
        0,
        10,
        ItNote::play_note(D4, 11, 62).with_effect(0x08, 0x45),
    );
    writer.set_note(
        pat,
        0,
        11,
        ItNote::play_note(A4, 12, 60).with_effect(0x08, 0x45),
    );
    writer.set_note(
        pat,
        32,
        10,
        ItNote::play_note(B4, 11, 65).with_effect(0x08, 0x45),
    );
    writer.set_note(
        pat,
        32,
        11,
        ItNote::play_note(D4, 12, 62).with_effect(0x08, 0x45),
    );

    // Piano - powerful doubled octaves
    writer.set_note(pat, 0, 12, ItNote::play_note(D4, 13, 65));
    writer.set_note(pat, 16, 12, ItNote::play_note(FS4, 13, 62));
    writer.set_note(pat, 32, 12, ItNote::play_note(A4, 13, 68));
    writer.set_note(pat, 48, 12, ItNote::play_note(D5, 13, 72)); // Peak

    // Bass driving at full power
    writer.set_note(pat, 0, 13, ItNote::play_note(D2, 14, 75));
    writer.set_note(pat, 16, 13, ItNote::play_note(FS3 - 12, 14, 72));
    writer.set_note(pat, 32, 13, ItNote::play_note(D2 + 5, 14, 75));
    writer.set_note(pat, 48, 13, ItNote::play_note(A2, 14, 72));

    // Pad full
    writer.set_note(pat, 0, 14, ItNote::play_note(D3, 15, 55));

    // Impact hit - big opening
    writer.set_note(pat, 0, 15, ItNote::play_note(60, 16, 70));
}

fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
    // Outro: Resolution, decrescendo to prepare seamless loop back to theme

    // Strings sustain D major with volume slide down (decrescendo)
    writer.set_note(
        pat,
        0,
        0,
        ItNote::play_note(D2, 1, 50).with_effect(0x04, 0x10),
    ); // Vol slide down
    writer.set_note(
        pat,
        0,
        1,
        ItNote::play_note(A3, 2, 48).with_effect(0x04, 0x10),
    );
    writer.set_note(
        pat,
        0,
        2,
        ItNote::play_note(D4, 3, 45).with_effect(0x08, 0x34),
    ); // Gentle vibrato

    // Continue decrescendo
    writer.set_note(
        pat,
        16,
        0,
        ItNote::play_note(D2, 1, 42).with_effect(0x04, 0x10),
    );
    writer.set_note(pat, 16, 1, ItNote::play_note(A3, 2, 40));

    // Violin melody fragment - nostalgic callback
    writer.set_note(
        pat,
        16,
        2,
        ItNote::play_note(FS4, 3, 45).with_effect(0x08, 0x34),
    );
    writer.set_note(
        pat,
        24,
        2,
        ItNote::play_note(A4, 3, 42).with_effect(0x07, 0x10),
    ); // Portamento
    writer.set_note(
        pat,
        32,
        2,
        ItNote::play_note(D5, 3, 40).with_effect(0x08, 0x34),
    );
    // Final resolve
    writer.set_note(pat, 48, 2, ItNote::play_note(D4, 3, 35));

    // Horn holds root, fading
    writer.set_note(
        pat,
        0,
        3,
        ItNote::play_note(D3, 4, 42).with_effect(0x04, 0x10),
    );

    // Final timpani - prepares return to theme
    writer.set_note(pat, 48, 6, ItNote::play_note(D2, 7, 50));
    writer.set_note(pat, 56, 6, ItNote::play_note(D2, 7, 45)); // Echo hit

    // Descending harp with gentle velocities
    let harp_vels = [40, 38, 35, 32, 28];
    for (i, note) in [A4, FS4, D4, A3, FS3].iter().enumerate() {
        writer.set_note(
            pat,
            (32 + i * 4) as u16,
            9,
            ItNote::play_note(*note, 10, harp_vels[i]),
        );
    }

    // Add pickup harp leading into next pattern (seamless loop)
    writer.set_note(pat, 56, 9, ItNote::play_note(D4, 10, 35));
    writer.set_note(pat, 58, 9, ItNote::play_note(FS4, 10, 38));
    writer.set_note(pat, 60, 9, ItNote::play_note(A4, 10, 42));

    // Pad fades with volume slide
    writer.set_note(
        pat,
        0,
        14,
        ItNote::play_note(D3, 15, 35).with_effect(0x04, 0x10),
    );

    // Choir sustains and fades
    writer.set_note(
        pat,
        0,
        10,
        ItNote::play_note(D4, 11, 35).with_effect(0x04, 0x20),
    ); // Faster fade
    writer.set_note(
        pat,
        0,
        11,
        ItNote::play_note(A3, 12, 32).with_effect(0x04, 0x20),
    );
}
