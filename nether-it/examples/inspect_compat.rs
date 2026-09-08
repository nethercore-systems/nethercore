fn main() {
    let path = std::env::args().nth(1).unwrap();
    let bytes = std::fs::read(path).unwrap();
    let source = nether_it::parse_it(&bytes).unwrap();
    let packed = nether_it::parse_ncit(&nether_it::pack_ncit(&source)).unwrap();
    for (label, module) in [("source", source), ("packed", packed)] {
        println!(
            "{label}: flags={:?} speed={} tempo={} channels={}",
            module.flags, module.initial_speed, module.initial_tempo, module.num_channels
        );
        for (instrument_index, instrument) in module.instruments.iter().enumerate() {
            println!(
                "instrument {instrument_index} maps 40..72: {:?}",
                &instrument.note_sample_table[40..73]
            );
        }
        for (sample_index, sample) in module.samples.iter().enumerate() {
            println!(
                "sample {sample_index}: c5={} len={} vol={} loops={}..{}",
                sample.c5_speed,
                sample.length,
                sample.default_volume,
                sample.loop_begin,
                sample.loop_end
            );
        }
        for (pattern_index, pattern) in module.patterns.iter().enumerate() {
            println!("pattern {pattern_index}");
            for (row_index, row) in pattern.notes.iter().enumerate() {
                if row.iter().any(|note| {
                    note.note != nether_it::ItNote::NO_NOTE
                        || note.instrument != 0
                        || note.effect != 0
                        || note.volume != 0
                }) {
                    println!("{row_index}: {row:?}");
                }
            }
        }
    }
}
