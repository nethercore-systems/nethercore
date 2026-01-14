//! Pattern builder functions for Nether Storm

use super::bass::{add_bass_rebalanced, add_wobble_accent};
use super::constants::*;
use super::drums::*;
use super::melody::{add_break_layer, add_lead_melody};
use super::types::{BreakStyle, Section};
use nether_it::{ItNote, ItWriter};

pub fn build_intro_pattern(writer: &mut ItWriter, pat: u8) {
    // Atmosphere builds, sparse elements
    writer.set_note(pat, 0, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 32));

    // Riser building tension
    writer.set_note(pat, 32, CH_RISER, ItNote::play_note(F4, INST_RISER, 40));

    // Very sparse hi-hats (every 16 rows)
    for row in (8..64).step_by(16) {
        writer.set_note(
            pat,
            row as u16,
            CH_HIHAT,
            ItNote::play_note(C5, INST_HH_CLOSED, 22),
        );
    }
}

pub fn build_build_a_pattern(writer: &mut ItWriter, pat: u8) {
    // Drums come in, tension builds
    // Quarter note kicks
    for bar in 0u16..4 {
        let base = bar * 16;
        writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 58));
    }

    // Eighth note hi-hats with some variation
    for row in (0..64).step_by(8) {
        let vel = if row % 16 == 0 { 52 } else { 42 };
        writer.set_note(
            pat,
            row as u16,
            CH_HIHAT,
            ItNote::play_note(C5, INST_HH_CLOSED, vel),
        );
    }

    // Pad
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(F3, INST_PAD, 48));

    // Riser
    writer.set_note(pat, 0, CH_RISER, ItNote::play_note(F4, INST_RISER, 45));
}

pub fn build_build_b_pattern(writer: &mut ItWriter, pat: u8) {
    // Build with snare roll at end
    // Continue kick pattern
    for bar in 0u16..4 {
        let base = bar * 16;
        writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 60));
    }

    // Hi-hats get faster
    for row in (0..48).step_by(4) {
        let vel = 35 + (row as u8 / 4);
        writer.set_note(
            pat,
            row as u16,
            CH_HIHAT,
            ItNote::play_note(C5, INST_HH_CLOSED, vel),
        );
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
pub fn build_drop_pattern(
    writer: &mut ItWriter,
    pat: u8,
    section: Section,
    variation: u8,
    break_style: BreakStyle,
) {
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

pub fn build_breakdown_pattern(writer: &mut ItWriter, pat: u8) {
    // Drums drop, atmospheric breathing room
    // Pad sustains
    writer.set_note(pat, 0, CH_PAD, ItNote::play_note(F3, INST_PAD, 50));
    writer.set_note(pat, 32, CH_PAD, ItNote::play_note(DB3, INST_PAD, 48));

    // Sparse kick
    writer.set_note(pat, 0, CH_KICK, ItNote::play_note(F2, INST_KICK, 50));
    writer.set_note(pat, 32, CH_KICK, ItNote::play_note(F2, INST_KICK, 45));

    // Half-time hi-hats
    for row in (0..64).step_by(16) {
        writer.set_note(
            pat,
            row as u16,
            CH_HIHAT,
            ItNote::play_note(C5, INST_HH_CLOSED, 35),
        );
    }

    // Sub bass with movement
    writer.set_note(pat, 0, CH_SUB, ItNote::play_note(F1, INST_SUB, 40));
    writer.set_note(pat, 32, CH_SUB, ItNote::play_note(DB1, INST_SUB, 38));

    // Atmosphere
    writer.set_note(pat, 0, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 40));

    // Riser building for next section
    writer.set_note(pat, 32, CH_RISER, ItNote::play_note(F4, INST_RISER, 45));
}

pub fn build_build_c_pattern(writer: &mut ItWriter, pat: u8) {
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
            writer.set_note(
                pat,
                row as u16,
                CH_HIHAT,
                ItNote::play_note(C5, INST_HH_CLOSED, vel),
            );
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

pub fn build_outro_pattern(writer: &mut ItWriter, pat: u8) {
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
        writer.set_note(
            pat,
            row as u16,
            CH_HIHAT,
            ItNote::play_note(C5, INST_HH_CLOSED, 28),
        );
    }

    // Cymbal decay
    writer.set_note(pat, 0, CH_CYMBAL, ItNote::play_note(C5, INST_CYMBAL, 35));

    // Atmosphere fades
    writer.set_note(pat, 0, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 35));
    writer.set_note(pat, 32, CH_ATMOS, ItNote::play_note(F3, INST_ATMOS, 25));
}
