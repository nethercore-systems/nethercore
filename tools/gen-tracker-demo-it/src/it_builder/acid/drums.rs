//! Drum pattern helper functions for Nether Acid

use super::{
    C5, CH_CLAP, CH_HIHAT, CH_HIHAT_OPEN, CH_KICK, INST_CLAP, INST_HH_CLOSED, INST_HH_OPEN,
    INST_KICK,
};
use nether_it::{ItNote, ItWriter};

/// 4-on-the-floor kick (classic techno)
pub(super) fn add_kick_4x4(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Every beat
    for beat in 0..4 {
        writer.set_note(
            pat,
            base + beat * 4,
            CH_KICK,
            ItNote::play_note(C5, INST_KICK, 64),
        );
    }
}

/// Claps on 2 and 4 (backbeat)
pub(super) fn add_claps(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 2 (row 4)
    writer.set_note(pat, base + 4, CH_CLAP, ItNote::play_note(C5, INST_CLAP, 64));
    // Beat 4 (row 12)
    writer.set_note(
        pat,
        base + 12,
        CH_CLAP,
        ItNote::play_note(C5, INST_CLAP, 64),
    );
}

/// 16th note hi-hats (every row) with stereo panning
pub(super) fn add_hihat_16ths(writer: &mut ItWriter, pat: u8, bar: u16, use_opens: bool) {
    let base = bar * 16;
    for row in 0..16 {
        let vel = if row % 4 == 0 { 50 } else { 35 }; // Accents on beats

        // Alternate panning for stereo width (8xx effect: 0x10=left, 0x30=right)
        let pan = if row % 2 == 0 { 0x10 } else { 0x30 };

        // Open hats on off-beats for groove
        if use_opens && (row == 6 || row == 14) {
            writer.set_note(
                pat,
                base + row,
                CH_HIHAT_OPEN,
                ItNote::play_note(C5, INST_HH_OPEN, vel + 5).with_effect(0x08, pan),
            );
        } else {
            writer.set_note(
                pat,
                base + row,
                CH_HIHAT,
                ItNote::play_note(C5, INST_HH_CLOSED, vel).with_effect(0x08, pan),
            );
        }
    }
}

/// 8th note hi-hats (every 2 rows)
pub(super) fn add_hihat_8ths(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    for row in 0..8 {
        let vel = if row % 2 == 0 { 48 } else { 32 };
        writer.set_note(
            pat,
            base + row * 2,
            CH_HIHAT,
            ItNote::play_note(C5, INST_HH_CLOSED, vel),
        );
    }
}
