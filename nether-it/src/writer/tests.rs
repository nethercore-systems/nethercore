//! Tests for IT file writer

use super::*;
use crate::{parse_it, ItInstrument, ItSample, IT_MAGIC};

#[test]
fn test_write_empty_module() {
    let mut writer = ItWriter::new("Test Song");
    writer.set_channels(4);
    writer.set_speed(6);
    writer.set_tempo(125);

    // Add a simple pattern
    let pat = writer.add_pattern(64);
    writer.set_orders(&[pat]);

    let data = writer.write();

    // Verify magic
    assert_eq!(&data[0..4], IT_MAGIC);

    // Try to parse it back
    let result = parse_it(&data);
    assert!(
        result.is_ok(),
        "Failed to parse written IT: {:?}",
        result.err()
    );

    let module = result.unwrap();
    assert_eq!(module.name, "Test Song");
    assert_eq!(module.initial_speed, 6);
    assert_eq!(module.initial_tempo, 125);
}

#[test]
fn test_write_with_instrument() {
    let mut writer = ItWriter::new("Instr Test");
    writer.set_channels(4);
    writer.set_speed(6);
    writer.set_tempo(125);

    // Add an instrument
    let mut instr = ItInstrument::default();
    instr.name = "Kick".to_string();
    writer.add_instrument(instr);

    // Add a sample
    let mut sample = ItSample::default();
    sample.name = "Kick Sample".to_string();
    sample.c5_speed = 22050;
    let audio = vec![0i16; 1000]; // 1000 samples of silence
    writer.add_sample(sample, &audio);

    // Add a pattern and order table
    let pat = writer.add_pattern(64);
    writer.set_orders(&[pat]);

    let data = writer.write();
    let module = parse_it(&data).expect("Failed to parse written IT file");

    assert_eq!(module.num_instruments, 1);
    assert_eq!(module.num_samples, 1);
    assert_eq!(module.instruments[0].name, "Kick");
    assert_eq!(module.samples[0].name, "Kick Sample");
}
