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
    samples.push(("strings_cello", cello_data));
    let cello_sample = make_sample("Cello", sample_rate);
    writer.add_sample(cello_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("Cello", 1));

    // 2. Viola (strings mid)
    let viola_data = synthesizers::generate_strings_viola();
    samples.push(("strings_viola", viola_data));
    let viola_sample = make_sample("Viola", sample_rate);
    writer.add_sample(viola_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("Viola", 2));

    // 3. Violin (strings high)
    let violin_data = synthesizers::generate_strings_violin();
    samples.push(("strings_violin", violin_data));
    let violin_sample = make_sample("Violin", sample_rate);
    writer.add_sample(violin_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("Violin", 3));

    // 4. French horn (brass low)
    let horn_data = synthesizers::generate_brass_horn();
    samples.push(("brass_horn", horn_data));
    let horn_sample = make_sample("Horn", sample_rate);
    writer.add_sample(horn_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Horn", 4));

    // 5. Trumpet (brass high)
    let trumpet_data = synthesizers::generate_brass_trumpet();
    samples.push(("brass_trumpet", trumpet_data));
    let trumpet_sample = make_sample("Trumpet", sample_rate);
    writer.add_sample(trumpet_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Trumpet", 5));

    // 6. Flute
    let flute_data = synthesizers::generate_flute();
    samples.push(("flute", flute_data));
    let flute_sample = make_sample("Flute", sample_rate);
    writer.add_sample(flute_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Flute", 6));

    // 7. Timpani
    let timpani_data = synthesizers::generate_timpani();
    samples.push(("timpani", timpani_data));
    let timpani_sample = make_sample("Timpani", sample_rate);
    writer.add_sample(timpani_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Timpani", 7));

    // 8. Snare (orchestral)
    let snare_data = synthesizers::generate_snare_orch();
    samples.push(("snare_orch", snare_data));
    let snare_sample = make_sample("Snare Roll", sample_rate);
    writer.add_sample(snare_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Snare", 8));

    // 9. Cymbal crash
    let cymbal_data = synthesizers::generate_cymbal_crash();
    samples.push(("cymbal_crash", cymbal_data));
    let cymbal_sample = make_sample("Cymbal", sample_rate);
    writer.add_sample(cymbal_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Cymbal", 9));

    // 10. Harp
    let harp_data = synthesizers::generate_harp_gliss();
    samples.push(("harp_gliss", harp_data));
    let harp_sample = make_sample("Harp", sample_rate);
    writer.add_sample(harp_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Harp", 10));

    // 11. Choir Ah
    let choir_ah_data = synthesizers::generate_choir_ah();
    samples.push(("choir_ah", choir_ah_data));
    let choir_ah_sample = make_sample("Choir Ah", sample_rate);
    writer.add_sample(choir_ah_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("Choir Ah", 11, 768));

    // 12. Choir Oh
    let choir_oh_data = synthesizers::generate_choir_oh();
    samples.push(("choir_oh", choir_oh_data));
    let choir_oh_sample = make_sample("Choir Oh", sample_rate);
    writer.add_sample(choir_oh_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_fade("Choir Oh", 12, 768));

    // 13. Piano
    let piano_data = synthesizers::generate_piano();
    samples.push(("piano", piano_data));
    let piano_sample = make_sample("Piano", sample_rate);
    writer.add_sample(piano_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Piano", 13));

    // 14. Epic bass
    let bass_data = synthesizers::generate_bass_epic();
    samples.push(("bass_epic", bass_data));
    let bass_sample = make_sample("Epic Bass", sample_rate);
    writer.add_sample(bass_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("Bass", 14));

    // 15. Orchestra pad
    let pad_data = synthesizers::generate_pad_orchestra();
    samples.push(("pad_orchestra", pad_data));
    let pad_sample = make_sample("Pad", sample_rate);
    writer.add_sample(pad_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument_continue("Pad", 15));

    // 16. Epic FX
    let fx_data = synthesizers::generate_fx_epic();
    samples.push(("fx_epic", fx_data));
    let fx_sample = make_sample("FX Epic", sample_rate);
    writer.add_sample(fx_sample, &samples.last().unwrap().1);
    writer.add_instrument(make_instrument("FX", 16));

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
    // Ch0 (Cello): Low D drone
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 30));

    // Ch2 (Violin): High A with vibrato
    writer.set_note(pat, 16, 2, ItNote::play_note(A5, 3, 35));

    // Ch9 (Harp): Rising arpeggio D-F#-A
    writer.set_note(pat, 24, 9, ItNote::play_note(D4, 10, 40));
    writer.set_note(pat, 28, 9, ItNote::play_note(FS4, 10, 38));
    writer.set_note(pat, 32, 9, ItNote::play_note(A4, 10, 42));

    // Ch14 (Pad): Swells in
    writer.set_note(pat, 32, 14, ItNote::play_note(D3, 15, 25));

    // Ch6 (Timpani): Single hit on D
    writer.set_note(pat, 48, 6, ItNote::play_note(D2, 7, 50));
}

fn build_theme_a1_pattern(writer: &mut ItWriter, pat: u8) {
    // A1: Theme statement - D major chord progression
    // I - V/3 - vi - IV (D - A/C# - Bm - G)

    // Ch0-2 (Strings): Sustained chords with NNA Continue
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 45)); // Cello
    writer.set_note(pat, 0, 1, ItNote::play_note(D3, 2, 42)); // Viola
    writer.set_note(pat, 0, 2, ItNote::play_note(FS4, 3, 40)); // Violin

    // Chord change at bar 2
    writer.set_note(pat, 16, 0, ItNote::play_note(A2, 1, 43));
    writer.set_note(pat, 16, 1, ItNote::play_note(A3, 2, 40));

    // Chord change at bar 3
    writer.set_note(pat, 32, 0, ItNote::play_note(B3 - 12, 1, 45)); // B2
    writer.set_note(pat, 32, 1, ItNote::play_note(D3, 2, 42));

    // Chord change at bar 4
    writer.set_note(pat, 48, 0, ItNote::play_note(D2 + 5, 1, 43)); // G2
    writer.set_note(pat, 48, 1, ItNote::play_note(B3, 2, 40));

    // Ch2 (Violin): Main melody - Theme A
    // Bar 1-2: D4 - F#4 - A4 - D5
    writer.set_note(pat, 0, 2, ItNote::play_note(D4, 3, 50));
    writer.set_note(pat, 4, 2, ItNote::play_note(FS4, 3, 48));
    writer.set_note(pat, 8, 2, ItNote::play_note(A4, 3, 52));
    writer.set_note(pat, 12, 2, ItNote::play_note(D5, 3, 50));
    // Bar 3-4: B4 - A4 - F#4 - E4
    writer.set_note(pat, 32, 2, ItNote::play_note(B4, 3, 48));
    writer.set_note(pat, 36, 2, ItNote::play_note(A4, 3, 46));
    writer.set_note(pat, 40, 2, ItNote::play_note(FS4, 3, 45));
    writer.set_note(pat, 44, 2, ItNote::play_note(E4, 3, 44));

    // Ch3 (Horn): Harmonic support
    writer.set_note(pat, 0, 3, ItNote::play_note(D3, 4, 40));
    writer.set_note(pat, 32, 3, ItNote::play_note(B3, 4, 38));

    // Ch6 (Timpani): Downbeats
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 55));
    writer.set_note(pat, 32, 6, ItNote::play_note(D2, 7, 50));

    // Ch9 (Harp): Chord arpeggios
    writer.set_note(pat, 8, 9, ItNote::play_note(D4, 10, 35));
    writer.set_note(pat, 10, 9, ItNote::play_note(FS4, 10, 33));
    writer.set_note(pat, 12, 9, ItNote::play_note(A4, 10, 36));

    // Ch13 (Bass): Follows roots
    writer.set_note(pat, 0, 13, ItNote::play_note(D2, 14, 55));
    writer.set_note(pat, 32, 13, ItNote::play_note(B3 - 12, 14, 52));
}

fn build_theme_a2_pattern(writer: &mut ItWriter, pat: u8) {
    // A2: Theme variation - more elements
    // Same chord progression as A1

    // Strings continue
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 48));
    writer.set_note(pat, 0, 1, ItNote::play_note(D3, 2, 45));
    writer.set_note(pat, 16, 0, ItNote::play_note(A2, 1, 46));
    writer.set_note(pat, 32, 0, ItNote::play_note(B3 - 12, 1, 48));
    writer.set_note(pat, 48, 0, ItNote::play_note(D2 + 5, 1, 45));

    // Violin melody continues
    writer.set_note(pat, 0, 2, ItNote::play_note(D4, 3, 52));
    writer.set_note(pat, 4, 2, ItNote::play_note(E4, 3, 50));
    writer.set_note(pat, 8, 2, ItNote::play_note(FS4, 3, 52));
    writer.set_note(pat, 12, 2, ItNote::play_note(GS4, 3, 54)); // Lydian G#
    writer.set_note(pat, 16, 2, ItNote::play_note(A4, 3, 56));

    // Ch4 (Trumpet): Fanfare accents
    writer.set_note(pat, 32, 4, ItNote::play_note(D5, 5, 50));
    writer.set_note(pat, 36, 4, ItNote::play_note(FS5, 5, 48));

    // Ch7 (Snare): Roll in final bars
    for row in (48..64).step_by(2) {
        writer.set_note(pat, row as u16, 7, ItNote::play_note(60, 8, 35 + (row - 48) as u8));
    }

    // Ch10-11 (Choir): Enters - NNA Fade creates smooth vowels
    writer.set_note(pat, 0, 10, ItNote::play_note(D4, 11, 35)); // Choir Ah
    writer.set_note(pat, 32, 11, ItNote::play_note(A3, 12, 33)); // Choir Oh

    // Ch12 (Piano): Countermelody
    writer.set_note(pat, 0, 12, ItNote::play_note(FS3, 13, 38));
    writer.set_note(pat, 4, 12, ItNote::play_note(A3, 13, 36));
    writer.set_note(pat, 8, 12, ItNote::play_note(D4, 13, 40));

    // Timpani
    writer.set_note(pat, 0, 6, ItNote::play_note(D2, 7, 55));
    writer.set_note(pat, 32, 6, ItNote::play_note(A2, 7, 52));

    // Bass
    writer.set_note(pat, 0, 13, ItNote::play_note(D2, 14, 58));
    writer.set_note(pat, 32, 13, ItNote::play_note(B3 - 12, 14, 55));
}

fn build_development_pattern(writer: &mut ItWriter, pat: u8) {
    // B: Development - relative minor, tension builds
    // vi - iii - IV - V (Bm - F#m - G - A)

    // More active strings
    writer.set_note(pat, 0, 0, ItNote::play_note(B3 - 12, 1, 50));
    writer.set_note(pat, 8, 0, ItNote::play_note(FS3 - 12, 1, 48));
    writer.set_note(pat, 16, 0, ItNote::play_note(B3 - 12, 1, 50));
    writer.set_note(pat, 32, 0, ItNote::play_note(D2 + 5, 1, 52));
    writer.set_note(pat, 48, 0, ItNote::play_note(A2, 1, 54));

    // Violin runs
    writer.set_note(pat, 0, 2, ItNote::play_note(B4, 3, 50));
    writer.set_note(pat, 4, 2, ItNote::play_note(D5, 3, 48));
    writer.set_note(pat, 8, 2, ItNote::play_note(FS5, 3, 52));
    writer.set_note(pat, 16, 2, ItNote::play_note(A5, 3, 55));

    // Brass becomes prominent
    writer.set_note(pat, 0, 3, ItNote::play_note(B3, 4, 48));
    writer.set_note(pat, 16, 3, ItNote::play_note(FS3, 4, 46));
    writer.set_note(pat, 32, 4, ItNote::play_note(D5, 5, 52));
    writer.set_note(pat, 40, 4, ItNote::play_note(A5, 5, 50));

    // Flute ornaments
    writer.set_note(pat, 8, 5, ItNote::play_note(FS5, 6, 40));
    writer.set_note(pat, 12, 5, ItNote::play_note(A5, 6, 42));
    writer.set_note(pat, 32, 5, ItNote::play_note(D5 + 12, 6, 44)); // D6

    // More timpani
    writer.set_note(pat, 0, 6, ItNote::play_note(B3 - 12, 7, 55));
    writer.set_note(pat, 16, 6, ItNote::play_note(FS3 - 12, 7, 52));
    writer.set_note(pat, 32, 6, ItNote::play_note(D2 + 5, 7, 56));
    writer.set_note(pat, 48, 6, ItNote::play_note(A2, 7, 58));

    // Cymbal accents
    writer.set_note(pat, 32, 8, ItNote::play_note(60, 9, 45));

    // Choir sustains
    writer.set_note(pat, 0, 10, ItNote::play_note(B3, 11, 40));
    writer.set_note(pat, 0, 11, ItNote::play_note(FS4, 12, 38));

    // FX: Riser
    writer.set_note(pat, 32, 15, ItNote::play_note(60, 16, 35));

    // Bass driving
    writer.set_note(pat, 0, 13, ItNote::play_note(B3 - 12, 14, 58));
    writer.set_note(pat, 32, 13, ItNote::play_note(D2 + 5, 14, 60));
}

fn build_climax_pattern(writer: &mut ItWriter, pat: u8) {
    // C: Climax - FULL ORCHESTRA, maximum energy
    // I - I/3 - IV - V - vi - IV - I/5 - V - I

    // Strings in high register, full
    writer.set_note(pat, 0, 0, ItNote::play_note(D3, 1, 58));
    writer.set_note(pat, 0, 1, ItNote::play_note(A3, 2, 55));
    writer.set_note(pat, 0, 2, ItNote::play_note(D5, 3, 60));

    writer.set_note(pat, 16, 0, ItNote::play_note(FS3, 1, 56));
    writer.set_note(pat, 16, 2, ItNote::play_note(FS5, 3, 58));

    writer.set_note(pat, 32, 0, ItNote::play_note(D2 + 5, 1, 58));
    writer.set_note(pat, 32, 2, ItNote::play_note(B4, 3, 60));

    writer.set_note(pat, 48, 0, ItNote::play_note(D2, 1, 60));
    writer.set_note(pat, 48, 2, ItNote::play_note(D5, 3, 62));

    // Brass fanfare - call and response
    writer.set_note(pat, 0, 3, ItNote::play_note(D4, 4, 55));
    writer.set_note(pat, 8, 4, ItNote::play_note(D5, 5, 58));
    writer.set_note(pat, 12, 4, ItNote::play_note(A5, 5, 56));
    writer.set_note(pat, 16, 3, ItNote::play_note(FS4, 4, 54));
    writer.set_note(pat, 24, 4, ItNote::play_note(D5, 5, 60));

    // Flute soaring
    writer.set_note(pat, 0, 5, ItNote::play_note(D5 + 12, 6, 48)); // D6
    writer.set_note(pat, 16, 5, ItNote::play_note(A5, 6, 50));
    writer.set_note(pat, 32, 5, ItNote::play_note(FS5 + 12, 6, 52)); // F#6

    // Timpani rolls
    for row in (0..16).step_by(2) {
        writer.set_note(pat, row as u16, 6, ItNote::play_note(D2, 7, 50 + row as u8));
    }
    writer.set_note(pat, 32, 6, ItNote::play_note(D2, 7, 62));

    // Snare building
    for row in (16..32).step_by(2) {
        writer.set_note(pat, row as u16, 7, ItNote::play_note(60, 8, 40));
    }

    // Cymbal crashes
    writer.set_note(pat, 0, 8, ItNote::play_note(60, 9, 55));
    writer.set_note(pat, 32, 8, ItNote::play_note(60, 9, 58));

    // Harp glissando
    for (i, note) in [D4, FS4, A4, D5, FS5].iter().enumerate() {
        writer.set_note(pat, (i * 2) as u16, 9, ItNote::play_note(*note, 10, 45));
    }

    // Choir at full - alternating ah/oh
    writer.set_note(pat, 0, 10, ItNote::play_note(D4, 11, 50));
    writer.set_note(pat, 0, 11, ItNote::play_note(A4, 12, 48));
    writer.set_note(pat, 32, 10, ItNote::play_note(B4, 11, 52));
    writer.set_note(pat, 32, 11, ItNote::play_note(D4, 12, 50));

    // Piano - doubled octaves
    writer.set_note(pat, 0, 12, ItNote::play_note(D4, 13, 55));
    writer.set_note(pat, 16, 12, ItNote::play_note(FS4, 13, 52));
    writer.set_note(pat, 32, 12, ItNote::play_note(A4, 13, 56));
    writer.set_note(pat, 48, 12, ItNote::play_note(D5, 13, 58));

    // Bass driving
    writer.set_note(pat, 0, 13, ItNote::play_note(D2, 14, 62));
    writer.set_note(pat, 16, 13, ItNote::play_note(FS3 - 12, 14, 60));
    writer.set_note(pat, 32, 13, ItNote::play_note(D2 + 5, 14, 62));
    writer.set_note(pat, 48, 13, ItNote::play_note(A2, 14, 60));

    // Pad full
    writer.set_note(pat, 0, 14, ItNote::play_note(D3, 15, 45));

    // Impact hit
    writer.set_note(pat, 0, 15, ItNote::play_note(60, 16, 55));
}

fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
    // Outro: Resolution, prepares loop back

    // Strings sustain D major
    writer.set_note(pat, 0, 0, ItNote::play_note(D2, 1, 45));
    writer.set_note(pat, 0, 1, ItNote::play_note(A3, 2, 42));
    writer.set_note(pat, 0, 2, ItNote::play_note(D4, 3, 40));

    // Violin melody fragment
    writer.set_note(pat, 16, 2, ItNote::play_note(FS4, 3, 42));
    writer.set_note(pat, 24, 2, ItNote::play_note(A4, 3, 40));
    writer.set_note(pat, 32, 2, ItNote::play_note(D5, 3, 38));

    // Horn holds root
    writer.set_note(pat, 0, 3, ItNote::play_note(D3, 4, 38));

    // Final timpani
    writer.set_note(pat, 48, 6, ItNote::play_note(D2, 7, 50));

    // Descending harp
    for (i, note) in [A4, FS4, D4, A3, FS3].iter().enumerate() {
        writer.set_note(pat, (32 + i * 4) as u16, 9, ItNote::play_note(*note, 10, 35 - i as u8 * 3));
    }

    // Pad fades
    writer.set_note(pat, 0, 14, ItNote::play_note(D3, 15, 30));
}
