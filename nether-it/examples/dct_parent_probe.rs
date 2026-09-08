//! Original fixture: duplicate notes on independent pattern channels.
use nether_it::*;
fn main() {
    let out = std::env::args().nth(1).expect("output file");
    let past = std::env::args().any(|a| a == "--past");
    let dedup = std::env::args().any(|a| a == "--dedup");
    let sample = std::env::args().any(|a| a == "--sample");
    let instruments = std::env::args().any(|a| a == "--instruments");
    let mut w = ItWriter::new("DCT parent isolation");
    w.set_channels(2);
    w.set_flags(ItFlags::STEREO | ItFlags::INSTRUMENTS);
    for _ in 0..2 {
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
    }
    let mut ins = ItInstrument {
        nna: NewNoteAction::Continue,
        dct: if sample {
            DuplicateCheckType::Sample
        } else {
            DuplicateCheckType::Note
        },
        dca: DuplicateCheckAction::Cut,
        default_pan: None,
        ..Default::default()
    };
    for (n, entry) in ins.note_sample_table.iter_mut().enumerate() {
        *entry = (n as u8, if dedup && n > 60 { 2 } else { 1 });
    }
    w.add_instrument(ins.clone());
    w.add_instrument(ins);
    let p = w.add_pattern(5);
    w.set_orders(&[p, 255]);
    let rows = if instruments {
        [(0, 0, 60, 255), (1, 0, 64, 0), (2, 0, 60, 0)]
    } else {
        [(0, 1, 60, 255), (1, 1, 64, 255), (2, 0, 60, 0)]
    };
    for (row, channel, note, pan) in rows {
        w.set_note(
            p,
            row,
            channel,
            ItNote {
                note: if dedup && row == 2 { 65 } else { note },
                instrument: if instruments && !dedup && row > 0 {
                    2
                } else {
                    1
                },
                effect: 24,
                effect_param: pan,
                ..Default::default()
            },
        );
    }
    w.set_note(
        p,
        2,
        1,
        ItNote {
            note: 254,
            ..Default::default()
        },
    );
    if past {
        w.set_note(
            p,
            3,
            0,
            ItNote {
                effect: 19,
                effect_param: 0x70,
                ..Default::default()
            },
        );
    }
    std::fs::write(out, w.write()).unwrap();
}
