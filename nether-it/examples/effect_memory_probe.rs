//! Original finite main-column IT effect/memory fixtures, used by it_effect_probe.py.
use nether_it::*;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out = std::path::Path::new(&args[1]);
    std::fs::create_dir_all(out).unwrap();
    let kinds = [
        ("baseline", 0, 0),
        ("D-down", 4, 1),
        ("D-up", 4, 0x10),
        ("D-fine", 4, 0xf2),
        ("E", 5, 4),
        ("F", 6, 4),
        ("E-fine", 5, 0xf3),
        ("F-extra", 6, 0xe3),
        ("G", 7, 8),
        ("H", 8, 0x34),
        ("I", 9, 0x21),
        ("J", 10, 0x37),
        ("K", 11, 1),
        ("L", 12, 1),
        ("Q", 17, 0x93),
        ("R", 18, 0x34),
        ("T-up", 20, 0x11),
        ("T-down", 20, 1),
        ("U", 21, 0x34),
        ("W", 23, 1),
        ("W-fine", 23, 0xf2),
        ("S-cut", 19, 0xc2),
        ("S-delay", 19, 0xd2),
        ("S-pattern-delay", 19, 0xe1),
        ("H-column", 8, 0x34),
        ("G-linked", 7, 0),
        ("T-gap", 20, 0x11),
        ("T-zero-up", 20, 0x11),
        ("T-set", 20, 150),
        ("T-channel", 20, 0x11),
        ("T-channel-isolation", 20, 0x11),
        ("T-set-recall", 20, 0x11),
    ];
    for old in [false, true] {
        for (name, effect, param) in kinds {
            let mut writer = ItWriter::new("Original IT effect memory");
            writer.set_channels(if name.starts_with("T-channel") { 2 } else { 1 });
            writer.set_speed(6);
            writer.set_tempo(125);
            writer.set_mix_volume(48);
            writer.set_global_volume(128);
            // Finite named scope: linear slides, instruments; old/compatible G paired.
            writer.set_flags(ItFlags::from_bits(
                1 | 4 | 8 | if old { 16 | 32 } else { 0 },
            ));
            writer.add_sample(
                ItSample {
                    name: "original sine".into(),
                    flags: ItSampleFlags::LOOP,
                    loop_end: 64,
                    c5_speed: 22050,
                    default_volume: 32,
                    ..Default::default()
                },
                &(0..64)
                    .map(|i| {
                        (12000.0 * (std::f64::consts::TAU * i as f64 / 32.0).sin()).round() as i16
                    })
                    .collect::<Vec<_>>(),
            );
            let mut instrument = ItInstrument::default();
            for (note, slot) in instrument.note_sample_table.iter_mut().enumerate() {
                *slot = (note as u8, 1);
            }
            writer.add_instrument(instrument);
            let pat = writer.add_pattern(8);
            writer.set_orders(&[pat, 255]);
            if name == "T-channel" {
                writer.set_note(
                    pat,
                    1,
                    1,
                    ItNote {
                        effect: 20,
                        effect_param: 2,
                        ..Default::default()
                    },
                );
            }
            if name == "T-channel-isolation" {
                writer.set_note(
                    pat,
                    2,
                    1,
                    ItNote {
                        effect: 20,
                        effect_param: 2,
                        ..Default::default()
                    },
                );
            }
            writer.set_note(
                pat,
                0,
                0,
                ItNote {
                    note: 60,
                    instrument: 1,
                    ..Default::default()
                },
            );
            for row in 1..6 {
                let mut note = ItNote {
                    effect,
                    effect_param: if row == 1 { param } else { 0 },
                    ..Default::default()
                };
                if matches!(name, "G" | "L" | "G-linked") && row == 1 {
                    note.note = 72;
                }
                if name == "K" && row == 1 {
                    note.effect = 8;
                    note.effect_param = 0x34;
                }
                if name == "L" && row == 1 {
                    note.effect = 7;
                    note.effect_param = 8;
                }
                if matches!(name, "K" | "L") && row == 2 {
                    note.effect_param = 1;
                }
                if name == "G-linked" && row == 1 {
                    note.note = 255;
                    note.effect = 6;
                    note.effect_param = 8;
                }
                if name == "G-linked" && row == 2 {
                    note.note = 72;
                }
                if name == "H-column" {
                    note.volume = 96;
                }
                if name == "T-gap" && row == 2 {
                    note.effect = 0;
                }
                if name == "T-channel-isolation" && row == 2 {
                    note.effect = 0;
                }
                if name == "T-set-recall" && row == 2 {
                    note.effect_param = 150;
                }
                if name == "T-zero-up" && row == 2 {
                    note.effect_param = 0x10;
                }
                if name == "T-set" && row > 1 {
                    note.effect = 0;
                }
                if name == "S-delay" {
                    note.note = if row % 2 == 1 { 72 } else { 60 };
                    note.instrument = 1;
                }
                if name == "S-cut" {
                    note.note = 60;
                    note.instrument = 1;
                }
                if name == "S-pattern-delay" && row > 2 {
                    note.effect = 0;
                }
                writer.set_note(pat, row, 0, note);
            }
            // A pitch/volume reset marker makes the authored timing audible.
            writer.set_note(
                pat,
                6,
                0,
                ItNote {
                    note: 72,
                    instrument: 1,
                    volume: 32,
                    ..Default::default()
                },
            );
            std::fs::write(
                out.join(format!("{name}-old{}.it", u8::from(old))),
                writer.write(),
            )
            .unwrap();
        }
    }
}
