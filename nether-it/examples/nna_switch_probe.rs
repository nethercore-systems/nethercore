//! Original stereo fixtures: old-voice NNA versus incoming instrument defaults.
use nether_it::*;
use std::path::Path;
fn main() {
    let out = std::env::args().nth(1).expect("output directory");
    std::fs::create_dir_all(&out).unwrap();
    let fade = std::env::args().any(|arg| arg == "--fade");
    let override_nna = std::env::args().any(|arg| arg == "--override");
    for old_continues in [false, true] {
        let mut w = ItWriter::new("NNA instrument switch");
        w.set_channels(1);
        w.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS);
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
        for (index, continuing) in [old_continues, !old_continues].into_iter().enumerate() {
            let mut ins = ItInstrument {
                nna: if fade && index == 0 && continuing {
                    NewNoteAction::NoteFade
                } else if continuing != (override_nna && index == 0) {
                    NewNoteAction::Continue
                } else {
                    NewNoteAction::Cut
                },
                fadeout: 0,
                default_pan: Some(if index == 0 { 0 } else { 64 }),
                ..Default::default()
            };
            for (note, entry) in ins.note_sample_table.iter_mut().enumerate() {
                *entry = (note as u8, 1);
            }
            w.add_instrument(ins);
        }
        let p = w.add_pattern(4);
        w.set_orders(&[p, 255]);
        w.set_note(
            p,
            0,
            0,
            ItNote {
                note: 60,
                instrument: 1,
                effect: if override_nna { 19 } else { 0 },
                effect_param: if old_continues { 0x74 } else { 0x73 },
                ..Default::default()
            },
        );
        w.set_note(
            p,
            1,
            0,
            ItNote {
                note: 64,
                instrument: 2,
                ..Default::default()
            },
        );
        std::fs::write(
            Path::new(&out).join(if old_continues {
                "continue.it"
            } else {
                "cut.it"
            }),
            w.write(),
        )
        .unwrap();
    }
}
