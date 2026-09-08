//! Original paired IT fixtures: volume G0..G9 versus equivalent main-column Gxx.
//! Usage: cargo run -p nether-it --example volume_porta_probe -- OUTPUT_DIRECTORY
use nether_it::*;

fn main() {
    let out = std::path::PathBuf::from(std::env::args_os().nth(1).expect("output directory"));
    std::fs::create_dir_all(&out).unwrap();
    for (digit, speed) in [0, 1, 4, 8, 16, 32, 64, 96, 128, 255]
        .into_iter()
        .enumerate()
    {
        for volume_column in [false, true] {
            let mut writer = ItWriter::new("Original volume porta probe");
            writer.set_channels(1);
            writer.set_speed(6);
            writer.add_sample(
                ItSample {
                    name: "tone".into(),
                    flags: ItSampleFlags::LOOP,
                    loop_end: 64,
                    c5_speed: 22050,
                    ..Default::default()
                },
                &(0..64)
                    .map(|i| if i < 32 { 2000i16 } else { -2000 })
                    .collect::<Vec<_>>(),
            );
            let mut instrument = ItInstrument::default();
            for (note, slot) in instrument.note_sample_table.iter_mut().enumerate() {
                *slot = (note as u8, 1);
            }
            writer.add_instrument(instrument);
            let pat = writer.add_pattern(8);
            writer.set_orders(&[pat, 255]);
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
            // Seed memory for G0 and then exercise target selection and recall.
            writer.set_note(
                pat,
                1,
                0,
                ItNote {
                    note: 72,
                    effect: 7,
                    effect_param: 8,
                    ..Default::default()
                },
            );
            for row in 2..7 {
                writer.set_note(
                    pat,
                    row,
                    0,
                    if volume_column {
                        ItNote {
                            note: 48,
                            volume: 193 + digit as u8,
                            ..Default::default()
                        }
                    } else {
                        ItNote {
                            note: 48,
                            effect: 7,
                            effect_param: speed,
                            ..Default::default()
                        }
                    },
                );
            }
            let column = if volume_column { "volume" } else { "main" };
            std::fs::write(out.join(format!("g{digit}-{column}.it")), writer.write()).unwrap();
        }
    }
}
