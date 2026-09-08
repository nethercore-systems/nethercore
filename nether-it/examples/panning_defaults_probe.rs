//! Original compatibility fixtures: note-driven pan defaults and surround override.
use nether_it::*;
use std::path::Path;

fn main() {
    let destination = std::env::args().nth(1).expect("output directory");
    for mode in ["sample", "instrument", "both"] {
        let mut writer = ItWriter::new("Original default pan probe");
        writer.set_channels(1);
        writer.set_speed(6);
        if mode == "sample" {
            writer.set_flags(ItFlags::STEREO);
        }
        writer.add_sample(
            ItSample {
                name: "tone".into(),
                flags: ItSampleFlags::LOOP,
                loop_end: 64,
                c5_speed: 22050,
                default_pan: (mode != "instrument").then_some(64),
                ..Default::default()
            },
            &(0..64)
                .map(|i| if i < 32 { 2000i16 } else { -2000 })
                .collect::<Vec<_>>(),
        );
        if mode != "sample" {
            let mut instrument = ItInstrument {
                default_pan: Some(if mode == "both" { 0 } else { 64 }),
                ..Default::default()
            };
            for (i, entry) in instrument.note_sample_table.iter_mut().enumerate() {
                *entry = (i as u8, 1);
            }
            writer.add_instrument(instrument);
        }
        let pattern = writer.add_pattern(6);
        writer.set_orders(&[pattern, 255]);
        for (row, note) in [
            ItNote {
                note: 60,
                instrument: 1,
                effect: 19,
                effect_param: 0x91,
                ..Default::default()
            },
            ItNote {
                note: 60,
                ..Default::default()
            },
            ItNote {
                instrument: 1,
                effect: 24,
                effect_param: 0x80,
                ..Default::default()
            },
            ItNote {
                note: 60,
                effect: 7,
                effect_param: 4,
                ..Default::default()
            },
            ItNote {
                note: 60,
                effect: 24,
                effect_param: 0x80,
                ..Default::default()
            },
            ItNote::default(),
        ]
        .into_iter()
        .enumerate()
        {
            writer.set_note(pattern, row as u16, 0, note);
        }
        std::fs::write(
            Path::new(&destination).join(format!("pan-{mode}.it")),
            writer.write(),
        )
        .unwrap();
    }
}
