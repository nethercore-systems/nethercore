//! Tests for tracker channel state and NNA system

use super::*;

fn create_playing_channel() -> TrackerChannel {
    let mut ch = TrackerChannel::default();
    ch.note_on = true;
    ch.sample_handle = 1;
    ch.volume = 1.0;
    ch.volume_fadeout = 65535;
    ch.current_note = 60; // C-5
    ch.instrument = 1;
    ch
}

#[test]
fn test_is_audible() {
    let mut ch = TrackerChannel::default();
    assert!(!ch.is_audible());

    ch.note_on = true;
    assert!(!ch.is_audible()); // No sample

    ch.sample_handle = 1;
    assert!(!ch.is_audible()); // No fadeout

    ch.volume_fadeout = 65535;
    assert!(ch.is_audible());
}

#[test]
fn test_is_available_for_nna() {
    let mut ch = TrackerChannel::default();
    assert!(ch.is_available_for_nna()); // Not playing

    ch.note_on = true;
    ch.sample_handle = 1;
    ch.volume_fadeout = 65535;
    assert!(!ch.is_available_for_nna()); // Playing

    ch.volume_fadeout = 0;
    assert!(ch.is_available_for_nna()); // Faded out
}

#[test]
fn test_nna_cut() {
    let mut ch = create_playing_channel();
    let needs_background = ch.apply_nna_action(NNA_CUT);

    assert!(!needs_background);
    assert!(!ch.note_on);
    assert_eq!(ch.volume, 0.0);
}

#[test]
fn test_nna_continue() {
    let mut ch = create_playing_channel();
    let needs_background = ch.apply_nna_action(NNA_CONTINUE);

    assert!(needs_background);
    assert!(ch.note_on); // Still playing
    assert!(!ch.key_off);
}

#[test]
fn test_nna_note_off() {
    let mut ch = create_playing_channel();
    let needs_background = ch.apply_nna_action(NNA_NOTE_OFF);

    assert!(needs_background);
    assert!(ch.note_on);
    assert!(ch.key_off); // Key-off triggered
}

#[test]
fn test_nna_note_fade() {
    let mut ch = create_playing_channel();
    ch.instrument_fadeout_rate = 0; // No fadeout set

    let needs_background = ch.apply_nna_action(NNA_NOTE_FADE);

    assert!(needs_background);
    assert!(ch.note_on);
    assert!(ch.key_off);
    assert!(ch.instrument_fadeout_rate > 0); // Default fadeout applied
}

#[test]
fn test_copy_to_background() {
    let ch = create_playing_channel();
    let bg = ch.copy_to_background(3);

    assert!(bg.is_background);
    assert_eq!(bg.parent_channel, 3);
    assert_eq!(bg.current_note, 60);
    assert!(bg.note_on);
}

#[test]
fn test_duplicate_check_off() {
    let ch = create_playing_channel();
    assert!(!ch.matches_duplicate_check(DCT_OFF, 60, 1, 1));
}

#[test]
fn test_duplicate_check_note() {
    let ch = create_playing_channel();
    // Note matches (60), other params don't matter for DCT_NOTE
    assert!(ch.matches_duplicate_check(DCT_NOTE, 60, 99, 99));
    assert!(!ch.matches_duplicate_check(DCT_NOTE, 61, 1, 1));
}

#[test]
fn test_duplicate_check_sample() {
    let ch = create_playing_channel();
    // Sample matches (1), other params don't matter for DCT_SAMPLE
    assert!(ch.matches_duplicate_check(DCT_SAMPLE, 99, 1, 99));
    assert!(!ch.matches_duplicate_check(DCT_SAMPLE, 60, 2, 1));
}

#[test]
fn test_duplicate_check_instrument() {
    let ch = create_playing_channel();
    // Instrument matches (1), other params don't matter for DCT_INSTRUMENT
    assert!(ch.matches_duplicate_check(DCT_INSTRUMENT, 99, 99, 1));
    assert!(!ch.matches_duplicate_check(DCT_INSTRUMENT, 60, 1, 2));
}

#[test]
fn test_dca_cut() {
    let mut ch = create_playing_channel();
    ch.apply_dca(DCA_CUT);

    assert!(!ch.note_on);
    assert_eq!(ch.volume, 0.0);
}

#[test]
fn test_dca_note_off() {
    let mut ch = create_playing_channel();
    ch.apply_dca(DCA_NOTE_OFF);

    assert!(ch.note_on);
    assert!(ch.key_off);
}

#[test]
fn test_dca_note_fade() {
    let mut ch = create_playing_channel();
    ch.instrument_fadeout_rate = 0;
    ch.apply_dca(DCA_NOTE_FADE);

    assert!(ch.note_on);
    assert!(ch.key_off);
    assert!(ch.instrument_fadeout_rate > 0);
}

#[test]
fn test_surround_default_off() {
    let ch = TrackerChannel::default();
    assert!(!ch.surround);
}

#[test]
fn test_surround_reset() {
    let mut ch = TrackerChannel::default();
    ch.surround = true;
    ch.reset();
    assert!(!ch.surround);
}

#[test]
fn test_sample_direction_default() {
    let mut ch = TrackerChannel::default();
    ch.reset();
    assert_eq!(ch.sample_direction, 1); // Forward by default
}

#[test]
fn test_sample_direction_reverse() {
    let mut ch = TrackerChannel::default();
    ch.sample_direction = -1; // S9F reverse playback
    assert_eq!(ch.sample_direction, -1);
}
