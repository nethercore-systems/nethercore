//! Sample data structures

/// Unified tracker sample
#[derive(Debug, Clone)]
pub struct TrackerSample {
    /// Sample name
    pub name: String,
    /// Global volume (0-64)
    pub global_volume: u8,
    /// Default volume (0-64)
    pub default_volume: u8,
    /// Default panning (0-64), None if not set
    pub default_pan: Option<u8>,
    /// Sample length in samples
    pub length: u32,
    /// Loop begin
    pub loop_begin: u32,
    /// Loop end
    pub loop_end: u32,
    /// Loop type
    pub loop_type: LoopType,
    /// C5 speed (sample rate for middle C)
    pub c5_speed: u32,
    /// Sustain loop begin
    pub sustain_loop_begin: u32,
    /// Sustain loop end
    pub sustain_loop_end: u32,
    /// Sustain loop type
    pub sustain_loop_type: LoopType,

    // =========================================================================
    // Sample auto-vibrato (IT stores per-sample, XM per-instrument)
    // =========================================================================
    /// Auto-vibrato speed (0-64)
    pub vibrato_speed: u8,
    /// Auto-vibrato depth (0-64)
    pub vibrato_depth: u8,
    /// Auto-vibrato rate/sweep (0-64)
    pub vibrato_rate: u8,
    /// Auto-vibrato waveform (0=sine, 1=ramp down, 2=square, 3=random)
    pub vibrato_type: u8,
}

impl Default for TrackerSample {
    fn default() -> Self {
        Self {
            name: String::new(),
            global_volume: 64,
            default_volume: 64,
            default_pan: None,
            length: 0,
            loop_begin: 0,
            loop_end: 0,
            loop_type: LoopType::None,
            c5_speed: 8363,
            sustain_loop_begin: 0,
            sustain_loop_end: 0,
            sustain_loop_type: LoopType::None,
            // Sample auto-vibrato (IT per-sample feature)
            vibrato_speed: 0,
            vibrato_depth: 0,
            vibrato_rate: 0,
            vibrato_type: 0,
        }
    }
}

/// Sample loop type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopType {
    /// No loop
    #[default]
    None,
    /// Forward loop
    Forward,
    /// Ping-pong (bidirectional) loop
    PingPong,
}
