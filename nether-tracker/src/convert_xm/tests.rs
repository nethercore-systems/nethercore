//! Tests for XM conversion

use super::*;
use crate::{LoopType, TrackerEffect};

#[test]
fn test_convert_xm_note() {
    // XM note 49 (C-4, middle C) passes through unchanged
    // note_to_period() expects 1-based XM notes where 49 = C-4
    let xm_note = nether_xm::XmNote {
        note: 49,
        instrument: 1,
        volume: 0x30, // Volume 32
        effect: 0,
        effect_param: 0,
    };

    let tracker_note = convert_xm_note(&xm_note);
    assert_eq!(tracker_note.note, 49); // C-4, unchanged from XM
    assert_eq!(tracker_note.instrument, 1);
    assert_eq!(tracker_note.volume, 32);
}

#[test]
fn test_convert_xm_effect_speed() {
    let effect = effects::convert_xm_effect(nether_xm::effects::SET_SPEED_TEMPO, 6, 0);
    assert_eq!(effect, TrackerEffect::SetSpeed(6));
}

#[test]
fn test_convert_xm_effect_tempo() {
    let effect = effects::convert_xm_effect(nether_xm::effects::SET_SPEED_TEMPO, 125, 0);
    assert_eq!(effect, TrackerEffect::SetTempo(125));
}

#[test]
fn test_convert_xm_volume_slide() {
    let effect = effects::convert_xm_effect(nether_xm::effects::VOLUME_SLIDE, 0x52, 0);
    assert_eq!(effect, TrackerEffect::VolumeSlide { up: 5, down: 2 });
}

#[test]
fn test_convert_xm_instrument_sample_loop() {
    // Create an XM instrument with sample loop info
    // Using relative_note=17, finetune=-16 which gives ~22050 Hz sample rate
    // This means the sample doesn't need resampling, so loop points stay the same
    let xm_instr = nether_xm::XmInstrument {
        name: "Test".to_string(),
        num_samples: 1,
        sample_loop_start: 1000,
        sample_loop_length: 2000,
        sample_loop_type: 1, // Forward loop
        sample_finetune: -16,
        sample_relative_note: 17, // ~22050 Hz, so ratio ≈ 1
        volume_envelope: None,
        panning_envelope: None,
        volume_fadeout: 512,
        vibrato_type: 1,
        vibrato_sweep: 128,
        vibrato_depth: 16,
        vibrato_rate: 8,
    };

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);

    // Loop points should be approximately the same since sample rate ≈ 22050 Hz
    // (small rounding differences may occur)
    assert!(tracker_instr.sample_loop_start >= 990 && tracker_instr.sample_loop_start <= 1010);
    assert_eq!(tracker_instr.sample_loop_type, LoopType::Forward);
    // Finetune and relative_note are zeroed because pitch adjustment is baked
    // into the resampled 22050 Hz sample during ROM packing
    assert_eq!(tracker_instr.sample_finetune, 0);
    assert_eq!(tracker_instr.sample_relative_note, 0);

    // Verify auto-vibrato
    assert_eq!(tracker_instr.auto_vibrato_type, 1);
    assert_eq!(tracker_instr.auto_vibrato_sweep, 128);
    assert_eq!(tracker_instr.auto_vibrato_depth, 16);
    assert_eq!(tracker_instr.auto_vibrato_rate, 8);
}

#[test]
fn test_convert_xm_instrument_loop_points_scaled() {
    // Test that loop points are properly scaled when resampling
    // Using relative_note=0, finetune=0 which gives 8363 Hz base sample rate
    // Ratio = 22050 / 8363 ≈ 2.636
    let xm_instr = nether_xm::XmInstrument {
        name: "ScaledLoop".to_string(),
        num_samples: 1,
        sample_loop_start: 1000,
        sample_loop_length: 2000,
        sample_loop_type: 1, // Forward loop
        sample_finetune: 0,
        sample_relative_note: 0, // 8363 Hz base rate
        volume_envelope: None,
        panning_envelope: None,
        volume_fadeout: 0,
        vibrato_type: 0,
        vibrato_sweep: 0,
        vibrato_depth: 0,
        vibrato_rate: 0,
    };

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);

    // Expected: loop_start = 1000 * (22050/8363) ≈ 2636
    // Expected: loop_length = 2000 * (22050/8363) ≈ 5273
    // Allow small tolerance for rounding
    assert!(
        tracker_instr.sample_loop_start >= 2630 && tracker_instr.sample_loop_start <= 2640,
        "Loop start should be ~2636, got {}",
        tracker_instr.sample_loop_start
    );
    let loop_length = tracker_instr.sample_loop_end - tracker_instr.sample_loop_start;
    assert!(
        (5265..=5280).contains(&loop_length),
        "Loop length should be ~5273, got {}",
        loop_length
    );
}

#[test]
fn test_convert_xm_instrument_pingpong_loop() {
    let xm_instr = nether_xm::XmInstrument {
        name: "PingPong".to_string(),
        num_samples: 1,
        sample_loop_start: 500,
        sample_loop_length: 1500,
        sample_loop_type: 2, // Ping-pong loop
        sample_finetune: 0,
        sample_relative_note: 0,
        volume_envelope: None,
        panning_envelope: None,
        volume_fadeout: 0,
        vibrato_type: 0,
        vibrato_sweep: 0,
        vibrato_depth: 0,
        vibrato_rate: 0,
    };

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);
    assert_eq!(tracker_instr.sample_loop_type, LoopType::PingPong);
}

#[test]
fn test_convert_xm_instrument_no_loop() {
    let xm_instr = nether_xm::XmInstrument {
        name: "NoLoop".to_string(),
        num_samples: 1,
        sample_loop_start: 0,
        sample_loop_length: 0,
        sample_loop_type: 0, // No loop
        sample_finetune: 0,
        sample_relative_note: 0,
        volume_envelope: None,
        panning_envelope: None,
        volume_fadeout: 0,
        vibrato_type: 0,
        vibrato_sweep: 0,
        vibrato_depth: 0,
        vibrato_rate: 0,
    };

    let tracker_instr = instruments::convert_xm_instrument(&xm_instr);
    assert_eq!(tracker_instr.sample_loop_type, LoopType::None);
}

#[test]
fn test_convert_xm_extended_e8x_coarse_panning() {
    // E80 = left, E87-E88 = center, E8F = right
    let left = effects::convert_xm_extended_effect(0x80);
    assert_eq!(left, TrackerEffect::SetPanning(2)); // 0*4 + 2 = 2

    let center = effects::convert_xm_extended_effect(0x88);
    assert_eq!(center, TrackerEffect::SetPanning(34)); // 8*4 + 2 = 34

    let right = effects::convert_xm_extended_effect(0x8F);
    assert_eq!(right, TrackerEffect::SetPanning(62)); // 15*4 + 2 = 62
}

#[test]
fn test_convert_xm_extended_e9x_retrigger() {
    // E93 = retrigger every 3 ticks
    let effect = effects::convert_xm_extended_effect(0x93);
    assert_eq!(
        effect,
        TrackerEffect::Retrigger {
            ticks: 3,
            volume_change: 0
        }
    );

    // E90 = no retrigger (0 ticks)
    let no_retrigger = effects::convert_xm_extended_effect(0x90);
    assert_eq!(
        no_retrigger,
        TrackerEffect::Retrigger {
            ticks: 0,
            volume_change: 0
        }
    );
}

#[test]
fn test_convert_xm_volume_column_vibrato() {
    // 0xE0 = vibrato depth 0 (speed from memory)
    let vib0 = effects::convert_xm_volume_effect(0xE0);
    assert_eq!(vib0, Some(TrackerEffect::Vibrato { speed: 0, depth: 0 }));

    // 0xE8 = vibrato depth 8
    let vib8 = effects::convert_xm_volume_effect(0xE8);
    assert_eq!(vib8, Some(TrackerEffect::Vibrato { speed: 0, depth: 8 }));

    // 0xEF = vibrato depth 15
    let vib15 = effects::convert_xm_volume_effect(0xEF);
    assert_eq!(
        vib15,
        Some(TrackerEffect::Vibrato {
            speed: 0,
            depth: 15
        })
    );
}
