// Quick test to verify Python-generated XM parses
use nether_xm::parse_xm;

fn main() {
    let xm_data = std::fs::read("test_generated.xm").expect("Could not read test_generated.xm");
    
    match parse_xm(&xm_data) {
        Ok(module) => {
            println!("SUCCESS: XM parsed correctly!");
            println!("  Name: {}", module.name);
            println!("  Channels: {}", module.num_channels);
            println!("  Patterns: {}", module.num_patterns);
            println!("  Instruments: {}", module.num_instruments);
            println!("  Speed: {}", module.default_speed);
            println!("  BPM: {}", module.default_bpm);
            println!("  Song length: {}", module.song_length);
            println!("\nInstruments:");
            for (i, inst) in module.instruments.iter().enumerate() {
                println!("  {}: {}", i + 1, inst.name);
            }
            println!("\nPattern 0 has {} rows", module.patterns[0].num_rows);
        }
        Err(e) => {
            eprintln!("FAILED: {:?}", e);
            std::process::exit(1);
        }
    }
}
