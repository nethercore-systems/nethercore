//! IT instrument and sample conversion

use crate::{
    DuplicateCheckAction, DuplicateCheckType, EnvelopeFlags, LoopType, NewNoteAction,
    TrackerEnvelope, TrackerInstrument, TrackerSample,
};

const TARGET_SAMPLE_RATE: u32 = 22_050;

fn resampled_position(position: u32, source_length: u32, source_rate: u32) -> u32 {
    if source_rate == 0 || source_rate == TARGET_SAMPLE_RATE {
        return if source_length == 0 {
            position
        } else {
            position.min(source_length)
        };
    }

    let ratio = TARGET_SAMPLE_RATE as f64 / source_rate as f64;
    // NCIT omits PCM length: zero is an unknown bound, not an empty loop.
    // The mixer still bounds reads against the actual packed sample buffer.
    let resampled_length = if source_length == 0 {
        u32::MAX
    } else {
        (source_length as f64 * ratio).ceil() as u32
    };
    ((position as f64 * ratio).round() as u32).min(resampled_length)
}

pub(super) fn convert_it_instrument(it_instr: &nether_it::ItInstrument) -> TrackerInstrument {
    TrackerInstrument {
        name: it_instr.name.clone(),
        nna: convert_it_nna(it_instr.nna),
        dct: convert_it_dct(it_instr.dct),
        dca: convert_it_dca(it_instr.dca),
        // IT spec: NFC starts at 1024, fadeout subtracted each tick
        // We use 65535 for more precision, so scale by 64 (65535/1024)
        fadeout: it_instr.fadeout.saturating_mul(64),
        global_volume: (it_instr.global_volume / 2).min(64), // IT uses 0-128, we use 0-64
        default_pan: it_instr.default_pan,
        random_volume: it_instr.random_volume.min(100),
        random_pan: it_instr.random_pan.min(64),
        note_sample_table: it_instr.note_sample_table.map(|(note, sample)| (note, u16::from(sample))),
        volume_envelope: it_instr.volume_envelope.as_ref().map(convert_it_envelope),
        panning_envelope: it_instr.panning_envelope.as_ref().map(convert_it_envelope),
        pitch_envelope: it_instr.pitch_envelope.as_ref().map(convert_it_envelope),
        filter_cutoff: it_instr.filter_cutoff,
        filter_resonance: it_instr.filter_resonance,
        pitch_pan_separation: it_instr.pitch_pan_separation,
        pitch_pan_center: it_instr.pitch_pan_center,

        // IT stores sample metadata per-sample, not per-instrument.
        // The playback engine should look up from TrackerModule.samples
        // using note_sample_table. For now, use defaults.
        sample_loop_start: 0,
        sample_loop_end: 0,
        sample_loop_type: LoopType::None,
        sample_is_stereo: false,
        sample_finetune: 0,
        xm_source_tuning: 0,
        xm_source_finetune: 0,
        xm_forward_loop_start: 0.0,
        xm_forward_loop_limit: 0.0,
        // IT uses C-5 as reference pitch, XM uses C-4. Transpose down 12 semitones.
        sample_relative_note: -12,
        sample_default_volume: None,

        // IT doesn't have XM-style auto-vibrato per instrument
        // (IT uses sample vibrato instead)
        auto_vibrato_type: 0,
        auto_vibrato_sweep: 0,
        auto_vibrato_depth: 0,
        auto_vibrato_rate: 0,
    }
}

fn convert_it_nna(nna: nether_it::NewNoteAction) -> NewNoteAction {
    match nna {
        nether_it::NewNoteAction::Cut => NewNoteAction::Cut,
        nether_it::NewNoteAction::Continue => NewNoteAction::Continue,
        nether_it::NewNoteAction::NoteOff => NewNoteAction::NoteOff,
        nether_it::NewNoteAction::NoteFade => NewNoteAction::NoteFade,
    }
}

fn convert_it_dct(dct: nether_it::DuplicateCheckType) -> DuplicateCheckType {
    match dct {
        nether_it::DuplicateCheckType::Off => DuplicateCheckType::Off,
        nether_it::DuplicateCheckType::Note => DuplicateCheckType::Note,
        nether_it::DuplicateCheckType::Sample => DuplicateCheckType::Sample,
        nether_it::DuplicateCheckType::Instrument => DuplicateCheckType::Instrument,
    }
}

fn convert_it_dca(dca: nether_it::DuplicateCheckAction) -> DuplicateCheckAction {
    match dca {
        nether_it::DuplicateCheckAction::Cut => DuplicateCheckAction::Cut,
        nether_it::DuplicateCheckAction::NoteOff => DuplicateCheckAction::NoteOff,
        nether_it::DuplicateCheckAction::NoteFade => DuplicateCheckAction::NoteFade,
    }
}

fn convert_it_envelope(it_env: &nether_it::ItEnvelope) -> TrackerEnvelope {
    TrackerEnvelope {
        points: it_env.points.clone(),
        loop_begin: it_env.loop_begin,
        loop_end: it_env.loop_end,
        sustain_begin: it_env.sustain_begin,
        sustain_end: it_env.sustain_end,
        flags: convert_it_envelope_flags(it_env.flags),
    }
}

fn convert_it_envelope_flags(it_flags: nether_it::ItEnvelopeFlags) -> EnvelopeFlags {
    let mut flags = EnvelopeFlags::empty();

    if it_flags.contains(nether_it::ItEnvelopeFlags::ENABLED) {
        flags = flags | EnvelopeFlags::ENABLED;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::LOOP) {
        flags = flags | EnvelopeFlags::LOOP;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::SUSTAIN_LOOP) {
        flags = flags | EnvelopeFlags::SUSTAIN_LOOP;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::CARRY) {
        flags = flags | EnvelopeFlags::CARRY;
    }
    if it_flags.contains(nether_it::ItEnvelopeFlags::FILTER) {
        flags = flags | EnvelopeFlags::FILTER;
    }

    flags
}

pub(super) fn convert_it_sample(it_smp: &nether_it::ItSample) -> TrackerSample {
    // Packed samples are mono 22050 Hz. Keep loop coordinates in the packed
    // sample's coordinate space; the ceil boundary matches the resampler.
    let length = resampled_position(it_smp.length, it_smp.length, it_smp.playback_c5_speed());
    let loop_begin =
        resampled_position(it_smp.loop_begin, it_smp.length, it_smp.playback_c5_speed());
    let loop_end = resampled_position(it_smp.loop_end, it_smp.length, it_smp.playback_c5_speed());
    let sustain_loop_begin = resampled_position(
        it_smp.sustain_loop_begin,
        it_smp.length,
        it_smp.playback_c5_speed(),
    );
    let sustain_loop_end = resampled_position(
        it_smp.sustain_loop_end,
        it_smp.length,
        it_smp.playback_c5_speed(),
    );

    TrackerSample {
        is_stereo: false,
        name: it_smp.name.clone(),
        global_volume: it_smp.global_volume,
        default_volume: it_smp.default_volume,
        default_pan: it_smp.default_pan,
        length,
        loop_begin,
        loop_end,
        loop_type: if it_smp.has_loop() {
            if it_smp.is_pingpong_loop() {
                LoopType::PingPong
            } else {
                LoopType::Forward
            }
        } else {
            LoopType::None
        },
        // Packed sample data has been resampled to the engine's C-5 baseline.
        c5_speed: TARGET_SAMPLE_RATE,
        xm_source_tuning: 0,
        xm_source_finetune: 0,
        xm_forward_loop_start: 0.0,
        xm_forward_loop_limit: 0.0,
        sustain_loop_begin,
        sustain_loop_end,
        sustain_loop_type: if it_smp.has_sustain_loop() {
            if it_smp.is_pingpong_sustain() {
                LoopType::PingPong
            } else {
                LoopType::Forward
            }
        } else {
            LoopType::None
        },
        // IT stores auto-vibrato per-sample (XM stores per-instrument)
        vibrato_speed: it_smp.vibrato_speed,
        vibrato_depth: it_smp.vibrato_depth,
        vibrato_rate: it_smp.vibrato_rate,
        vibrato_type: it_smp.vibrato_type,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ncit_unknown_sample_length_preserves_resampled_loops() {
        let mut sample = nether_it::ItSample {
            length: 401,
            loop_begin: 0,
            loop_end: 401,
            sustain_loop_begin: 100,
            sustain_loop_end: 300,
            c5_speed: 44100,
            ..Default::default()
        };
        let full = convert_it_sample(&sample);
        sample.length = 0; // NCIT metadata has no source PCM length.
        let packed = convert_it_sample(&sample);
        assert_eq!(packed.loop_end, full.loop_end);
        assert_eq!(packed.sustain_loop_begin, full.sustain_loop_begin);
        assert_eq!(packed.sustain_loop_end, full.sustain_loop_end);
    }

    #[test]
    fn test_convert_it_sample_auto_vibrato() {
        let it_sample = nether_it::ItSample {
            name: "test".to_string(),
            filename: String::new(),
            global_volume: 64,
            flags: nether_it::ItSampleFlags::empty(),
            convert_flags: 1,
            default_volume: 64,
            default_pan: None,
            length: 1000,
            loop_begin: 0,
            loop_end: 0,
            c5_speed: 8363,
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            vibrato_speed: 10,
            vibrato_depth: 20,
            vibrato_rate: 30,
            vibrato_type: 2,
        };

        let tracker_sample = convert_it_sample(&it_sample);

        assert_eq!(tracker_sample.vibrato_speed, 10);
        assert_eq!(tracker_sample.vibrato_depth, 20);
        assert_eq!(tracker_sample.vibrato_rate, 30);
        assert_eq!(tracker_sample.vibrato_type, 2);
    }

    #[test]
    fn test_convert_it_sample_scales_packed_coordinates_and_boundary() {
        let it_sample = nether_it::ItSample {
            length: 1001,
            loop_begin: 100,
            loop_end: 1001,
            sustain_loop_begin: 200,
            sustain_loop_end: 800,
            c5_speed: 44_100,
            ..Default::default()
        };

        let sample = convert_it_sample(&it_sample);
        assert_eq!(sample.length, 501);
        assert_eq!(sample.loop_begin, 50);
        assert_eq!(sample.loop_end, 501);
        assert_eq!(sample.sustain_loop_begin, 100);
        assert_eq!(sample.sustain_loop_end, 400);
        assert_eq!(sample.c5_speed, TARGET_SAMPLE_RATE);
    }
}
