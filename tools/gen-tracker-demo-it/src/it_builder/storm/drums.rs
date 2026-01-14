//! Drum helper functions for Nether Storm

use super::constants::*;
use nether_it::{ItNote, ItWriter};

/// Standard DnB kick pattern (kick on 1 and 2.5)
pub fn add_kick_dnb(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 1 (row 0)
    writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 64));
    // Beat 2.5 (row 10) - syncopation
    writer.set_note(
        pat,
        base + 10,
        CH_KICK,
        ItNote::play_note(F2, INST_KICK, 58),
    );
}

/// Enhanced kick pattern for climax (more syncopation)
pub fn add_kick_climax(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 1
    writer.set_note(pat, base, CH_KICK, ItNote::play_note(F2, INST_KICK, 64));
    // Extra kick on row 6 for energy
    writer.set_note(pat, base + 6, CH_KICK, ItNote::play_note(F2, INST_KICK, 52));
    // Beat 2.5
    writer.set_note(
        pat,
        base + 10,
        CH_KICK,
        ItNote::play_note(F2, INST_KICK, 60),
    );
}

/// Main snares on beat 2 and 4
pub fn add_main_snares(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Beat 2 (row 4)
    writer.set_note(
        pat,
        base + 4,
        CH_SNARE,
        ItNote::play_note(C5, INST_SNARE, 64),
    );
    // Beat 4 (row 12)
    writer.set_note(
        pat,
        base + 12,
        CH_SNARE,
        ItNote::play_note(C5, INST_SNARE, 64),
    );
}

/// Ghost snares with density control (0=none, 1=light, 2=medium)
pub fn add_ghost_snares(writer: &mut ItWriter, pat: u8, bar: u16, density: u8) {
    if density == 0 {
        return; // Clean pattern - no ghost notes
    }

    let base = bar * 16;

    // Light (density 1): just before main snares for anticipation
    writer.set_note(
        pat,
        base + 2,
        CH_SNARE,
        ItNote::play_note(C5, INST_SNARE, 22),
    );
    writer.set_note(
        pat,
        base + 10,
        CH_SNARE,
        ItNote::play_note(C5, INST_SNARE, 22),
    );

    if density >= 2 {
        // Medium (density 2): add after-snare ghosts for shuffle feel
        writer.set_note(
            pat,
            base + 6,
            CH_SNARE,
            ItNote::play_note(C5, INST_SNARE, 18),
        );
        writer.set_note(
            pat,
            base + 14,
            CH_SNARE,
            ItNote::play_note(C5, INST_SNARE, 16),
        );
    }
}

/// Humanized hi-hat groove with velocity variation (8th notes, not 16ths!)
pub fn add_hihat_groove(writer: &mut ItWriter, pat: u8, bar: u16, use_opens: bool) {
    let base = bar * 16;
    // 8th note pattern: every 2 rows (8 hits per bar, not 16)
    // Velocity: strong on downbeats, medium on upbeats
    let positions: [(u16, u8); 8] = [
        (0, 55),  // Beat 1 - strong
        (2, 38),  // &
        (4, 50),  // Beat 2 - strong
        (6, 35),  // & (open hat position)
        (8, 52),  // Beat 3 - strong
        (10, 38), // &
        (12, 50), // Beat 4 - strong
        (14, 35), // & (open hat position)
    ];

    for (offset, vel) in positions {
        let abs_row = base + offset;
        // Open hi-hats on the "and" of 2 and 4 for groove
        if use_opens && (offset == 6 || offset == 14) {
            writer.set_note(
                pat,
                abs_row,
                CH_HIHAT_OPEN,
                ItNote::play_note(C5, INST_HH_OPEN, vel + 5),
            );
        } else {
            writer.set_note(
                pat,
                abs_row,
                CH_HIHAT,
                ItNote::play_note(C5, INST_HH_CLOSED, vel),
            );
        }
    }
}

/// Snare roll with accelerating density and crescendo
pub fn add_snare_roll(writer: &mut ItWriter, pat: u8, start_row: u16, length: u16) {
    let mut row = start_row;
    let end_row = start_row + length;
    let mut spacing = 4u16;
    let vel_start = 35u8;
    let vel_end = 64u8;

    while row < end_row {
        let progress = (row - start_row) as f32 / length as f32;
        let vel = vel_start + ((vel_end - vel_start) as f32 * progress) as u8;

        writer.set_note(pat, row, CH_SNARE, ItNote::play_note(C5, INST_SNARE, vel));

        // Accelerate: 4 -> 2 spacing (don't go to 1, too rapid at 174 BPM)
        if progress > 0.6 {
            spacing = 2;
        }

        row += spacing;
    }
}

/// Kick fill for variation patterns (adds extra kicks, avoids overlap with main pattern)
pub fn add_kick_fill(writer: &mut ItWriter, pat: u8, bar: u16) {
    let base = bar * 16;
    // Syncopated fill kicks - only at positions NOT used by add_kick_dnb (which uses 0 and 10)
    let hits = [(3, 48), (6, 50), (14, 45)];
    for (offset, vel) in hits {
        writer.set_note(
            pat,
            base + offset,
            CH_KICK,
            ItNote::play_note(F2, INST_KICK, vel),
        );
    }
}
