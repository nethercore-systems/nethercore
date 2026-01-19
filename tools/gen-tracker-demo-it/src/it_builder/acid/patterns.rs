//! Pattern builder functions for Nether Acid

use super::bass::{add_303_main_pattern, add_303_simple};
use super::drums::{add_claps, add_hihat_16ths, add_hihat_8ths, add_kick_4x4};
use super::{
    A2, A3, B2, C5, CH_303, CH_FX, CH_KICK, CH_PAD, CH_STAB, D3, E2, E3, FS3, G2, INST_ATMOSPHERE,
    INST_BASS_303, INST_BASS_303_SQUELCH, INST_CRASH, INST_KICK, INST_PAD, INST_RISER, INST_STAB,
};
use nether_it::{ItNote, ItWriter};

pub(super) fn build_intro_pattern(writer: &mut ItWriter, pat: u8) {
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

pub(super) fn build_groove_a_pattern(writer: &mut ItWriter, pat: u8) {
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

pub(super) fn build_build_pattern(writer: &mut ItWriter, pat: u8) {
    // Building energy with progressive filter opening (Zxx effect)
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // 303 pattern with filter automation - gradually open filter (Zxx: 0x1A effect)
        let base = bar * 16;
        let notes = [
            (0, E2, 64),
            (4, G2, 40),
            (8, A2, 64),
            (10, B2, 40),
            (12, E3, 64),
            (16, D3, 40),
            (20, B2, 40),
            (24, A2, 64),
            (28, G2, 40),
            (32, E2, 64),
            (36, G2, 40),
            (40, B2, 64),
            (44, D3, 64),
            (48, E3, 64),
            (52, B2, 40),
        ];

        // Filter cutoff increases each bar (Z40 → Z50 → Z60 → Z70)
        let cutoff = 0x40 + (bar as u8) * 0x10;

        for (offset, note, vel) in &notes {
            // Add filter automation to first note of each bar
            let note_obj = if *offset == 0 {
                ItNote::play_note(*note, INST_BASS_303, *vel).with_effect(0x1A, cutoff)
            // Zxx filter cutoff
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

pub(super) fn build_breakdown_pattern(writer: &mut ItWriter, pat: u8) {
    // Sparse - just kick on beat 1 and simple 303
    for bar in 0..4 {
        writer.set_note(pat, bar * 16, CH_KICK, ItNote::play_note(C5, INST_KICK, 60));
        add_303_simple(writer, pat, bar);
    }

    // Pad sustains throughout
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(E3, INST_PAD, 50));
}

pub(super) fn build_drop_pattern(writer: &mut ItWriter, pat: u8) {
    // MAXIMUM ENERGY - all accents on 303, full drums
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // 303 with MORE accents (more filter action)
        let base = bar * 16;
        let notes = [
            (0, E2, 64), // All accents!
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
            writer.set_note(
                pat,
                base + offset,
                CH_303,
                ItNote::play_note(note, INST_BASS_303, vel),
            );
        }
    }

    // Stabs for extra punch
    writer.set_note(pat, 0, CH_STAB, ItNote::play_note(E3, INST_STAB, 60));
    writer.set_note(pat, 32, CH_STAB, ItNote::play_note(E3, INST_STAB, 60));
}

pub(super) fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
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

pub(super) fn build_groove_b_pattern(writer: &mut ItWriter, pat: u8) {
    // Variation groove with different 303 pattern
    for bar in 0..4 {
        add_kick_4x4(writer, pat, bar);
        add_claps(writer, pat, bar);
        add_hihat_16ths(writer, pat, bar, true);

        // Different 303 sequence - A minor feel (A-C-E-G instead of E-G-A-B)
        let base = bar * 16;
        let notes = [
            (0, A2, 64),  // Accent
            (4, E2, 40),  // No accent
            (8, G2, 64),  // Accent
            (10, A2, 40), // No accent
            (12, E3, 64), // Accent - octave jump
            (16, D3, 40), // No accent
            (20, A2, 40), // No accent
            (24, G2, 64), // Accent
            (28, E2, 40), // No accent
            (32, A2, 64), // Accent
            (36, E2, 40), // No accent
            (40, G2, 64), // Accent
            (44, D3, 64), // Accent
            (48, E3, 64), // Accent
            (52, A2, 40), // No accent
        ];

        for (offset, note, vel) in notes {
            writer.set_note(
                pat,
                base + offset,
                CH_303,
                ItNote::play_note(note, INST_BASS_303, vel),
            );
        }
    }

    // Atmosphere layer for subtle texture
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(A2, INST_ATMOSPHERE, 30));
}

pub(super) fn build_build_intense_pattern(writer: &mut ItWriter, pat: u8) {
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

pub(super) fn build_drop_variation_pattern(writer: &mut ItWriter, pat: u8) {
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
                ItNote::play_note(*note, INST_BASS_303, *vel).with_effect(0x07, 0x20)
            // Gxx portamento
            } else {
                ItNote::play_note(*note, INST_BASS_303, *vel)
            };
            writer.set_note(pat, base + offset, CH_303, note_obj);
        }
    }

    // Crash for extra punch
    writer.set_note(pat, 0, CH_FX, ItNote::play_note(C5, INST_CRASH, 60));
}

pub(super) fn build_breakdown_deep_pattern(writer: &mut ItWriter, pat: u8) {
    // Atmospheric breakdown with low bass
    for bar in 0..4 {
        // Only kick on beat 1
        writer.set_note(pat, bar * 16, CH_KICK, ItNote::play_note(C5, INST_KICK, 55));

        // Very simple low 303
        let base = bar * 16;
        writer.set_note(pat, base, CH_303, ItNote::play_note(E2, INST_BASS_303, 45));
        writer.set_note(
            pat,
            base + 8,
            CH_303,
            ItNote::play_note(G2, INST_BASS_303, 40),
        );
    }

    // Atmosphere layer throughout for texture
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(E2, INST_ATMOSPHERE, 40));
    writer.set_note(pat, 32, CH_PAD, ItNote::play_note(G2, INST_ATMOSPHERE, 40));

    // Pad sustains for warmth
    writer.set_note(pat, 0, CH_STAB, ItNote::play_note(E3, INST_PAD, 35));
}

pub(super) fn build_drop_b_pattern(writer: &mut ItWriter, pat: u8) {
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
            writer.set_note(
                pat,
                base + offset,
                CH_303,
                ItNote::play_note(note, INST_BASS_303, vel),
            );
        }
    }

    // Crash for section transition
    writer.set_note(pat, 0, CH_FX, ItNote::play_note(C5, INST_CRASH, 62));
}

pub(super) fn build_drop_b_intense_pattern(writer: &mut ItWriter, pat: u8) {
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
            writer.set_note(
                pat,
                base + offset,
                CH_303,
                ItNote::play_note(note, INST_BASS_303_SQUELCH, vel),
            );
        }
    }

    // Crashes for massive impact
    writer.set_note(pat, 0, CH_FX, ItNote::play_note(C5, INST_CRASH, 64));
    writer.set_note(pat, 32, CH_FX, ItNote::play_note(C5, INST_CRASH, 64));

    // Stabs for extra punch
    writer.set_note(pat, 0, CH_STAB, ItNote::play_note(B2, INST_STAB, 64));
    writer.set_note(pat, 32, CH_STAB, ItNote::play_note(B2, INST_STAB, 64));
}
