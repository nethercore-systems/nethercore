//! Original fixture: direct IT pattern fade, not key-off.
use nether_it::*;
fn main() {
    let out = std::env::args().nth(1).expect("output file");
    let selection_delay = std::env::args().find_map(|a| {
        a.strip_prefix("--selection-delay=")
            .map(|v| v.parse::<u8>().expect("delay tick"))
    });
    let swing = std::env::args().find_map(|a| a.strip_prefix("--swing=").map(str::to_owned));
    let constant = std::env::args().any(|a| a == "--constant");
    let special_ramp = std::env::args().any(|a| a == "--special-ramp");
    let special_pan = std::env::args().any(|a| a == "--special-pan");
    let old_effects = std::env::args().any(|a| a == "--old-effects");
    let terminal = std::env::args().any(|a| a == "--terminal-envelope");
    let held = std::env::args().any(|a| a == "--held");
    let special_off = std::env::args().any(|a| a == "--special-off");
    let special_volume = std::env::args().any(|a| a == "--special-volume");
    let special_selection = std::env::args().any(|a| a == "--special-selection");
    let selection_volume = std::env::args().any(|a| a == "--selection-volume");
    let empty_selection = std::env::args().any(|a| a == "--empty-selection");
    let memory_pitch = std::env::args().any(|a| a == "--memory-pitch");
    let instrument_mode = std::env::args().any(|a| a == "--instrument-mode");
    let valid_swap = std::env::args().any(|a| a == "--valid-swap");
    let recover = std::env::args().any(|a| a == "--recover");
    let invalid = std::env::args().any(|a| a == "--invalid-sample");
    let restart = std::env::args().any(|a| a == "--sample-restart");
    let explicit = std::env::args().any(|a| a == "--explicit");
    let cut = std::env::args().any(|a| a == "--cut");
    let porta = std::env::args().any(|a| a == "--porta");
    let empty = std::env::args().any(|a| a == "--empty");
    let mut w = ItWriter::new("Direct note fade");
    w.set_channels(1);
    let compat = std::env::args().any(|a| a == "--compat");
    w.set_flags(
        ItFlags::STEREO
            | if old_effects {
                ItFlags::OLD_EFFECTS
            } else {
                ItFlags::default()
            }
            | if (restart || invalid) && !instrument_mode {
                ItFlags::default()
            } else {
                ItFlags::INSTRUMENTS
            }
            | if compat {
                ItFlags::LINK_G_MEMORY
            } else {
                ItFlags::default()
            },
    );
    w.add_sample(
        ItSample {
            flags: if restart {
                ItSampleFlags::default()
            } else {
                ItSampleFlags::LOOP
            },
            loop_end: 64,
            ..Default::default()
        },
        &(0..64)
            .map(|i| if constant || i < 32 { 2000 } else { -2000 })
            .collect::<Vec<_>>(),
    );
    if special_volume {
        w.add_sample(
            ItSample {
                default_pan: special_pan.then_some(64),
                default_volume: 16,
                flags: ItSampleFlags::LOOP,
                loop_end: 64,
                ..Default::default()
            },
            &(0..64)
                .map(|i| if i < 32 { 2000 } else { -2000 })
                .collect::<Vec<_>>(),
        );
    }
    if valid_swap {
        w.add_sample(
            ItSample {
                flags: ItSampleFlags::LOOP,
                loop_end: 64,
                ..Default::default()
            },
            &(0..64)
                .map(|i| if i % 32 < 16 { 2000 } else { -2000 })
                .collect::<Vec<_>>(),
        );
    }
    let mut ins = ItInstrument {
        fadeout: 16,
        ..Default::default()
    };
    for (n, entry) in ins.note_sample_table.iter_mut().enumerate() {
        *entry = (n as u8, 1);
    }
    if empty {
        ins.note_sample_table[64].1 = 0;
    }
    if special_selection {
        ins.volume_envelope = Some(ItEnvelope {
            points: vec![(0, 64), (100, 64)],
            flags: ItEnvelopeFlags::ENABLED,
            ..Default::default()
        });
    }
    if terminal {
        ins.volume_envelope = Some(ItEnvelope {
            points: vec![(0, 64), (12, 64)],
            flags: ItEnvelopeFlags::ENABLED,
            ..Default::default()
        });
    }
    if let Some(mode) = &swing {
        ins.global_volume = 64;
        ins.random_volume = if mode == "volume" { 20 } else { 0 };
        ins.random_pan = if mode == "pan" { 8 } else { 0 };
    }
    w.add_instrument(ins.clone());
    if empty_selection {
        if special_selection {
            ins.fadeout = 1024;
            ins.volume_envelope = Some(ItEnvelope {
                points: vec![(0, 16), (100, 16)],
                flags: ItEnvelopeFlags::ENABLED,
                ..Default::default()
            });
        }
        if special_ramp {
            ins.volume_envelope.as_mut().unwrap().points = vec![(0, 16), (12, 64), (100, 64)];
        }
        ins.note_sample_table[64].1 = 0;
        ins.note_sample_table[72].0 = 60;
        if special_volume {
            for entry in &mut ins.note_sample_table {
                entry.1 = 2;
            }
        }
        w.add_instrument(ins);
    }
    let p = w.add_pattern(16);
    w.set_orders(&[p, 255]);
    w.set_note(
        p,
        0,
        0,
        ItNote {
            note: 60,
            instrument: 1,
            ..Default::default()
        },
    );
    w.set_note(
        p,
        1,
        0,
        ItNote {
            note: if cut {
                ItNote::NO_NOTE
            } else if empty {
                64
            } else if held {
                ItNote::NO_NOTE
            } else if special_off {
                NOTE_OFF
            } else {
                NOTE_FADE
            },
            effect: if cut {
                19
            } else if porta {
                7
            } else {
                0
            },
            effect_param: if cut {
                0xc0
            } else if porta {
                16
            } else {
                0
            },
            ..Default::default()
        },
    );
    if restart {
        w.set_note(
            p,
            1,
            0,
            ItNote {
                note: if explicit { 60 } else { ItNote::NO_NOTE },
                instrument: 1,
                ..Default::default()
            },
        );
    }
    if invalid {
        w.set_note(
            p,
            1,
            0,
            ItNote {
                note: if special_selection {
                    if special_off { NOTE_OFF } else { NOTE_FADE }
                } else if explicit {
                    64
                } else {
                    ItNote::NO_NOTE
                },
                instrument: if valid_swap || empty_selection { 2 } else { 99 },
                volume: if selection_volume { 16 } else { 255 },
                effect: if selection_delay.is_some() {
                    19
                } else if porta {
                    7
                } else {
                    0
                },
                effect_param: selection_delay
                    .map_or(if porta { 16 } else { 0 }, |tick| 0xd0 | tick.min(15)),
            },
        );
    }
    if recover {
        w.set_note(
            p,
            2,
            0,
            ItNote {
                note: if memory_pitch { 72 } else { 60 },
                ..Default::default()
            },
        );
        w.set_note(
            p,
            3,
            0,
            ItNote {
                note: 60,
                instrument: 1,
                ..Default::default()
            },
        );
    }
    if swing.is_some() {
        for row in 0..16 {
            w.set_note(
                p,
                row,
                0,
                ItNote {
                    note: 60,
                    instrument: 1,
                    ..Default::default()
                },
            );
        }
    }
    if std::env::args().any(|a| a == "--timing") {
        for row in 0..16 {
            w.set_note(p, row, 0, ItNote::default());
        }
        w.set_note(
            p,
            0,
            0,
            ItNote {
                note: 60,
                instrument: 1,
                effect: 1,
                effect_param: 3,
                ..Default::default()
            },
        );
        w.set_note(
            p,
            2,
            0,
            ItNote {
                note: 60,
                instrument: 1,
                effect: 20,
                effect_param: 150,
                ..Default::default()
            },
        );
        w.set_note(
            p,
            4,
            0,
            ItNote {
                note: 72,
                instrument: 1,
                ..Default::default()
            },
        );
        w.set_note(
            p,
            6,
            0,
            ItNote {
                note: NOTE_CUT,
                ..Default::default()
            },
        );
        w.set_note(
            p,
            8,
            0,
            ItNote {
                note: 60,
                instrument: 1,
                ..Default::default()
            },
        );
    }
    std::fs::write(out, w.write()).unwrap();
}
