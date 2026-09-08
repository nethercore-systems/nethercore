//! Original IT fixtures for initially disabled S78/S7A envelopes.
use nether_it::*;
use std::path::Path;
fn main() {
    let output = std::env::args().nth(1).expect("output directory");
    std::fs::create_dir_all(&output).unwrap();
    for mode in [
        "volume",
        "pan",
        "pitch",
        "pitch_down",
        "pitch_control",
        "volume_sustain",
        "volume_pause",
        "pan_pause",
        "pitch_pause",
        "filter_cutoff",
        "filter_delay",
        "filter_delay_zero",
        "filter_delay_sample",
        "filter_delay_volume",
        "filter_delay_cut",
        "filter_delay_fine",
        "filter_delay_memory",
        "filter_delay_memory_control",
        "filter_delay_memory_fine",
        "filter_delay_memory_fine_control",
        "filter_delay_memory_row",
        "filter_delay_memory_row_control",
        "filter_delay_memory_shared",
        "filter_delay_memory_shared_control",
        "filter_delay_repeat",
        "filter_delay_repeat_control",
        "filter_delay_env",
        "filter_delay_env_control",
        "filter_delay_outside",
        "filter_full_hold",
        "filter_full_control",
        "filter_resonance",
        "filter_resonance_open",
        "filter_defaults",
        "filter_env",
        "filter_env_pause",
        "filter_env_start_disabled",
        "filter_env_midpoint",
    ] {
        let pan = mode.starts_with("pan");
        let pause = mode.ends_with("_pause");
        let mut w = ItWriter::new("Disabled envelope enable");
        w.set_channels(
            if mode == "filter_delay_fine"
                || mode.starts_with("filter_delay_repeat")
                || mode.starts_with("filter_delay_memory_fine")
                || mode.starts_with("filter_delay_memory_row")
            {
                2
            } else {
                1
            },
        );
        w.set_speed(6);
        w.set_flags(
            ItFlags::STEREO
                | if mode == "filter_delay_sample" {
                    ItFlags::from_bits(0)
                } else {
                    ItFlags::INSTRUMENTS
                },
        );
        w.add_sample(
            ItSample {
                flags: ItSampleFlags::LOOP,
                loop_end: 64,
                ..Default::default()
            },
            &(0..64)
                .map(|i| if i < 32 { 2000 } else { -2000 })
                .collect::<Vec<_>>(),
        );
        let env = Some(ItEnvelope {
            points: if mode == "volume_sustain" {
                vec![(0, 64), (1, 16), (3, 48), (5, 64)]
            } else if (mode == "pitch_control" || mode == "pitch_pause") {
                vec![(0, 24), (64, 24)]
            } else if mode.starts_with("pitch") {
                vec![
                    (0, 0),
                    (6, if mode == "pitch_down" { -24 } else { 24 }),
                    (12, 0),
                ]
            } else {
                vec![(0, 32), (64, 32)]
            },
            sustain_begin: if pause { 0 } else { 1 },
            sustain_end: if pause { 0 } else { 2 },
            loop_begin: 0,
            loop_end: 3,
            flags: if pause {
                ItEnvelopeFlags::ENABLED | ItEnvelopeFlags::SUSTAIN_LOOP
            } else if mode == "volume_sustain" {
                ItEnvelopeFlags::ENABLED | ItEnvelopeFlags::SUSTAIN_LOOP | ItEnvelopeFlags::LOOP
            } else if pause || mode.starts_with("pitch") && mode != "pitch_control" {
                ItEnvelopeFlags::ENABLED
            } else {
                ItEnvelopeFlags::empty()
            },
            ..Default::default()
        });
        let mut ins = ItInstrument::default();
        for (i, entry) in ins.note_sample_table.iter_mut().enumerate() {
            *entry = (i as u8, 1);
        }
        if mode.starts_with("pitch") {
            ins.pitch_envelope = env;
        } else if pan {
            ins.panning_envelope = env;
        } else if !mode.starts_with("filter_") {
            ins.volume_envelope = env;
        }
        if mode.starts_with("filter_env") {
            ins.filter_cutoff = Some(127);
            ins.pitch_envelope = Some(ItEnvelope {
                points: if mode == "filter_env_start_disabled" {
                    vec![(0, 32), (24, 32)]
                } else if mode == "filter_env_midpoint" {
                    vec![(0, 0), (24, 0)]
                } else if pause {
                    vec![(0, -16), (24, 32)]
                } else {
                    vec![(0, -32), (5, -32), (6, 0), (11, 0), (12, 32), (24, 32)]
                },
                flags: ItEnvelopeFlags::ENABLED
                    | ItEnvelopeFlags::FILTER
                    | if pause {
                        ItEnvelopeFlags::SUSTAIN_LOOP
                    } else {
                        ItEnvelopeFlags::empty()
                    },
                ..Default::default()
            });
        }
        if mode.starts_with("filter_delay") {
            ins.filter_cutoff = Some(64);
        }
        if mode.starts_with("filter_delay_repeat") {
            ins.volume_envelope = Some(ItEnvelope {
                points: vec![(0, 64), (1, 64), (2, 0)],
                flags: ItEnvelopeFlags::ENABLED,
                ..Default::default()
            });
        }
        if mode.starts_with("filter_delay_env") {
            ins.pitch_envelope = Some(ItEnvelope {
                points: vec![(0, -32), (1, 32), (64, 32)],
                flags: ItEnvelopeFlags::ENABLED | ItEnvelopeFlags::FILTER,
                ..Default::default()
            });
        }
        if mode == "filter_resonance" {
            ins.filter_cutoff = Some(64);
        }
        w.add_instrument(ins);
        let p = w.add_pattern(if mode.starts_with("filter_delay_memory_row") {
            3
        } else {
            4
        });
        w.set_orders(&[p, 255]);
        w.set_note(
            p,
            0,
            0,
            ItNote {
                note: if mode.starts_with("filter_") { 84 } else { 60 },
                instrument: 1,
                ..Default::default()
            },
        );
        w.set_note(
            p,
            1,
            0,
            ItNote {
                effect: 19,
                effect_param: if pause {
                    if pan {
                        0x79
                    } else if mode.starts_with("pitch") {
                        0x7b
                    } else {
                        0x77
                    }
                } else if mode == "volume_sustain" {
                    0
                } else if mode == "pitch_control" {
                    0x7c
                } else if mode.starts_with("pitch") {
                    0
                } else if pan {
                    0x7a
                } else {
                    0x78
                },
                ..Default::default()
            },
        );
        w.set_note(
            p,
            2,
            0,
            ItNote {
                note: if mode == "volume_sustain" {
                    NOTE_OFF
                } else {
                    ItNote::default().note
                },
                effect: 19,
                effect_param: if pause {
                    if pan {
                        0x7a
                    } else if mode.starts_with("pitch") {
                        0x7c
                    } else {
                        0x78
                    }
                } else if mode == "volume_sustain" {
                    0
                } else if mode == "pitch_control" {
                    0x7b
                } else if mode.starts_with("pitch") {
                    0
                } else if pan {
                    0x79
                } else {
                    0x77
                },
                ..Default::default()
            },
        );
        if mode.starts_with("filter_env") {
            for row in [1, 2] {
                w.set_note(
                    p,
                    row,
                    0,
                    ItNote {
                        effect: if pause { 19 } else { 0 },
                        effect_param: if pause {
                            if row == 1 { 0x7b } else { 0x7c }
                        } else {
                            0
                        },
                        ..Default::default()
                    },
                );
            }
        }
        if mode == "filter_env_start_disabled" {
            w.set_note(
                p,
                0,
                0,
                ItNote {
                    note: 84,
                    instrument: 1,
                    effect: 19,
                    effect_param: 0x7b,
                    ..Default::default()
                },
            );
        }
        if mode == "filter_cutoff" {
            for (row, cutoff) in [0, 64, 127].into_iter().enumerate() {
                w.set_note(
                    p,
                    row as u16,
                    0,
                    ItNote {
                        note: if row == 0 { 84 } else { ItNote::default().note },
                        instrument: if row == 0 { 1 } else { 0 },
                        effect: effects::MIDI_MACRO,
                        effect_param: cutoff,
                        ..Default::default()
                    },
                );
            }
        }
        if mode.starts_with("filter_resonance") {
            for (row, resonance) in [0x80, 0x88, 0x8f].into_iter().enumerate() {
                w.set_note(
                    p,
                    row as u16,
                    0,
                    ItNote {
                        note: if row == 0 { 84 } else { ItNote::default().note },
                        instrument: if row == 0 { 1 } else { 0 },
                        effect: effects::MIDI_MACRO,
                        effect_param: resonance,
                        ..Default::default()
                    },
                );
            }
        }
        if mode == "filter_full_hold" || mode == "filter_full_control" {
            for row in 0..3 {
                w.set_note(
                    p,
                    row,
                    0,
                    ItNote {
                        note: if row == 1 { ItNote::default().note } else { 84 },
                        instrument: if row == 1 { 0 } else { 1 },
                        effect: if row == 1 && mode == "filter_full_control" {
                            0
                        } else {
                            effects::MIDI_MACRO
                        },
                        effect_param: if row == 0 { 0 } else { 0x7f },
                        ..Default::default()
                    },
                );
            }
        }
        if mode.starts_with("filter_delay") {
            for row in 0..3 {
                w.set_note(
                    p,
                    row,
                    0,
                    if row == 0 {
                        ItNote {
                            note: 84,
                            instrument: 1,
                            effect: 19,
                            effect_param: if mode == "filter_delay_env_control" {
                                0
                            } else if mode == "filter_delay_outside" || mode == "filter_delay_fine"
                            {
                                0xd6
                            } else if mode == "filter_delay_zero" {
                                0xd0
                            } else {
                                0xd1
                            },
                            ..Default::default()
                        }
                    } else {
                        ItNote::default()
                    },
                );
            }
        }
        if mode == "filter_delay_volume" || mode == "filter_delay_cut" {
            w.set_note(
                p,
                0,
                0,
                ItNote {
                    note: 84,
                    instrument: 1,
                    ..Default::default()
                },
            );
            w.set_note(
                p,
                1,
                0,
                ItNote {
                    note: if mode == "filter_delay_cut" {
                        NOTE_CUT
                    } else {
                        ItNote::default().note
                    },
                    volume: if mode == "filter_delay_volume" {
                        0
                    } else {
                        255
                    },
                    effect: 19,
                    effect_param: 0xd3,
                    ..Default::default()
                },
            );
        }
        if mode.starts_with("filter_delay_memory") {
            for row in 0..2 {
                w.set_note(
                    p,
                    row,
                    0,
                    ItNote {
                        note: 84,
                        instrument: 1,
                        effect: 19,
                        effect_param: if row == 0 || mode.ends_with("control") {
                            0xd3
                        } else {
                            0
                        },
                        ..Default::default()
                    },
                );
            }
        }
        if mode.starts_with("filter_delay_memory_fine") {
            for row in 0..2 {
                w.set_note(
                    p,
                    row,
                    0,
                    ItNote {
                        note: 84,
                        instrument: 1,
                        effect: 19,
                        effect_param: 0xd6,
                        ..Default::default()
                    },
                );
                w.set_note(
                    p,
                    row,
                    1,
                    ItNote {
                        effect: 19,
                        effect_param: if row == 0 || mode.ends_with("control") {
                            0x62
                        } else {
                            0
                        },
                        ..Default::default()
                    },
                );
            }
        }
        if mode.starts_with("filter_delay_memory_row") {
            for row in 0..2 {
                w.set_note(
                    p,
                    row,
                    0,
                    ItNote {
                        note: 84,
                        instrument: 1,
                        effect: 19,
                        effect_param: 0xd1,
                        ..Default::default()
                    },
                );
                w.set_note(
                    p,
                    row,
                    1,
                    ItNote {
                        effect: 19,
                        effect_param: if row == 0 || mode.ends_with("control") {
                            0xe1
                        } else {
                            0
                        },
                        ..Default::default()
                    },
                );
            }
        }
        if mode.starts_with("filter_delay_memory_shared") {
            w.set_note(
                p,
                1,
                0,
                ItNote {
                    effect: 19,
                    effect_param: 0x81,
                    ..Default::default()
                },
            );
            w.set_note(
                p,
                2,
                0,
                ItNote {
                    note: 84,
                    instrument: 1,
                    effect: 19,
                    effect_param: if mode.ends_with("control") { 0x81 } else { 0 },
                    ..Default::default()
                },
            );
        }
        if mode == "filter_delay_repeat" {
            w.set_note(
                p,
                0,
                1,
                ItNote {
                    effect: 19,
                    effect_param: 0xe1,
                    ..Default::default()
                },
            );
        }
        if mode == "filter_delay_repeat_control" {
            w.set_note(
                p,
                1,
                0,
                ItNote {
                    note: 84,
                    instrument: 1,
                    effect: 19,
                    effect_param: 0xd1,
                    ..Default::default()
                },
            );
        }
        if mode == "filter_delay_fine" {
            w.set_note(
                p,
                0,
                1,
                ItNote {
                    effect: 19,
                    effect_param: 0x62,
                    ..Default::default()
                },
            );
        }
        if mode == "filter_delay_outside" || mode == "filter_delay_fine" {
            // Audible later control keeps the renderer's silence guard strict.
            w.set_note(
                p,
                3,
                0,
                ItNote {
                    note: 84,
                    instrument: 1,
                    ..Default::default()
                },
            );
        }
        if mode == "filter_defaults" {
            // Match the Zxx sweep using authored instrument cutoff defaults.
            for (row, cutoff) in [0, 64, 127].into_iter().enumerate() {
                let mut instrument = ItInstrument::default();
                instrument.filter_cutoff = Some(cutoff);
                for (i, entry) in instrument.note_sample_table.iter_mut().enumerate() {
                    *entry = (i as u8, 1);
                }
                let index = w.add_instrument(instrument);
                w.set_note(
                    p,
                    row as u16,
                    0,
                    ItNote {
                        note: 84,
                        instrument: index,
                        ..Default::default()
                    },
                );
            }
        }
        std::fs::write(Path::new(&output).join(format!("{mode}.it")), w.write()).unwrap();
    }
}
