//! Lead melody and break layer helpers for Nether Storm

use super::constants::*;
use super::types::{BreakStyle, Section};
use nether_it::{ItNote, ItWriter};

/// Lead melody for drops - sparse, punchy accents (not rapid triplets!)
pub fn add_lead_melody(writer: &mut ItWriter, pat: u8, section: Section, variation: u8) {
    if section == Section::DropA {
        // Drop A: Simple stab accents on key beats - one per bar
        writer.set_note(pat, 0, CH_STAB, ItNote::play_note(F4, INST_STAB, 55)); // Bar 1
        writer.set_note(pat, 16, CH_STAB, ItNote::play_note(AB4, INST_STAB, 52)); // Bar 2
        writer.set_note(pat, 32, CH_STAB, ItNote::play_note(C5, INST_STAB, 55)); // Bar 3
        writer.set_note(pat, 48, CH_STAB, ItNote::play_note(EB4, INST_STAB, 50)); // Bar 4

        if variation >= 1 {
            // Variation: add off-beat accents (still sparse - one extra per bar)
            writer.set_note(pat, 10, CH_STAB, ItNote::play_note(C5, INST_STAB, 45)); // Bar 1 offbeat
            writer.set_note(pat, 26, CH_STAB, ItNote::play_note(F5, INST_STAB, 52)); // Bar 2 offbeat
            writer.set_note(pat, 42, CH_STAB, ItNote::play_note(AB4, INST_STAB, 48)); // Bar 3 offbeat
            writer.set_note(pat, 58, CH_STAB, ItNote::play_note(F5, INST_STAB, 55));
            // Bar 4 end
        }
    } else {
        // Drop B: Stronger accents, still sparse - avoid rapid notes
        writer.set_note(pat, 0, CH_LEAD, ItNote::play_note(F5, INST_LEAD, 58)); // Bar 1 - high!
        writer.set_note(pat, 16, CH_LEAD, ItNote::play_note(EB5, INST_LEAD, 55)); // Bar 2
        writer.set_note(pat, 32, CH_LEAD, ItNote::play_note(C5, INST_LEAD, 55)); // Bar 3
        writer.set_note(pat, 48, CH_LEAD, ItNote::play_note(AB4, INST_LEAD, 52)); // Bar 4

        if variation >= 1 {
            // Variation: add off-beat responses (still one per bar, not rapid)
            writer.set_note(pat, 8, CH_LEAD, ItNote::play_note(C5, INST_LEAD, 50)); // Bar 1 response
            writer.set_note(pat, 24, CH_LEAD, ItNote::play_note(AB4, INST_LEAD, 48)); // Bar 2 response
            writer.set_note(pat, 40, CH_LEAD, ItNote::play_note(F5, INST_LEAD, 55)); // Bar 3 response
            writer.set_note(pat, 56, CH_LEAD, ItNote::play_note(F5, INST_LEAD, 60));
            // Bar 4 climax
        }

        // Impact at start of Drop B
        writer.set_note(pat, 0, CH_IMPACT, ItNote::play_note(F3, INST_IMPACT, 60));
    }
}

/// Add break layer with different styles - all sparse to avoid cluttering
pub fn add_break_layer(writer: &mut ItWriter, pat: u8, style: BreakStyle) {
    match style {
        BreakStyle::None => {}
        BreakStyle::Ghost => {
            // Very sparse - just one accent every 2 bars
            writer.set_note(pat, 8, CH_BREAK, ItNote::play_note(C5, INST_BREAK, 22));
            writer.set_note(pat, 40, CH_BREAK, ItNote::play_note(C5, INST_BREAK, 20));
        }
        BreakStyle::Accent => {
            // Layer with snares only - one per bar on beat 2
            for bar in 0u16..4 {
                let base = bar * 16;
                writer.set_note(
                    pat,
                    base + 4,
                    CH_BREAK,
                    ItNote::play_note(C5, INST_BREAK, 35),
                );
            }
        }
        BreakStyle::Fill => {
            // Accent on snares only, slightly louder for climax
            for bar in 0u16..4 {
                let base = bar * 16;
                writer.set_note(
                    pat,
                    base + 4,
                    CH_BREAK,
                    ItNote::play_note(C5, INST_BREAK, 40),
                );
                writer.set_note(
                    pat,
                    base + 12,
                    CH_BREAK,
                    ItNote::play_note(C5, INST_BREAK, 38),
                );
            }
        }
    }
}
