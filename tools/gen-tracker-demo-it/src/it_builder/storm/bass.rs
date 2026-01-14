//! Bass helper functions for Nether Storm

use super::constants::*;
use super::types::Section;
use nether_it::{ItNote, ItWriter};

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
pub fn add_bass_rebalanced(writer: &mut ItWriter, pat: u8, section: Section) {
    let roots = if section == Section::DropA {
        get_drop_a_roots()
    } else {
        get_drop_b_roots()
    };

    let reese_vel = if section == Section::DropB { 58 } else { 54 };

    for (row, sub_note, reese_note) in roots {
        // Sub: delayed by 1 row for sidechain feel, lower velocity
        writer.set_note(
            pat,
            row + 1,
            CH_SUB,
            ItNote::play_note(sub_note, INST_SUB, 42),
        );

        // Reese: main audible bass, also delayed for pumping feel
        writer.set_note(
            pat,
            row + 1,
            CH_REESE,
            ItNote::play_note(reese_note, INST_REESE, reese_vel),
        );

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
pub fn add_wobble_accent(writer: &mut ItWriter, pat: u8) {
    // Wobble on accent points in climax
    writer.set_note(pat, 0, CH_WOBBLE, ItNote::play_note(F3, INST_WOBBLE, 40));
    writer.set_note(pat, 32, CH_WOBBLE, ItNote::play_note(AB3, INST_WOBBLE, 38));
}
