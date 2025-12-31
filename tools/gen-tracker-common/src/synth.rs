//! Common synthesizer utilities shared across all tracker generators

pub const SAMPLE_RATE: f32 = 22050.0;

/// Fade-in duration in seconds (prevents clicks from abrupt sample starts)
const FADE_IN_SECS: f32 = 0.002; // 2ms

/// Fade-out duration in seconds (prevents clicks from sample cutoffs)
const FADE_OUT_SECS: f32 = 0.005; // 5ms

/// Apply fade-in and fade-out to a sample buffer to prevent clicks
pub fn apply_fades(samples: &mut [i16]) {
    let fade_in_samples = (SAMPLE_RATE * FADE_IN_SECS) as usize;
    let fade_out_samples = (SAMPLE_RATE * FADE_OUT_SECS) as usize;

    // Fade in
    for i in 0..fade_in_samples.min(samples.len()) {
        let factor = i as f32 / fade_in_samples as f32;
        samples[i] = (samples[i] as f32 * factor) as i16;
    }

    // Fade out
    let start = samples.len().saturating_sub(fade_out_samples);
    for i in start..samples.len() {
        let factor = (samples.len() - i) as f32 / fade_out_samples as f32;
        samples[i] = (samples[i] as f32 * factor) as i16;
    }
}

/// Simple PRNG (xorshift32) for deterministic noise generation
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    pub fn next(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x
    }

    pub fn next_f32(&mut self) -> f32 {
        self.next() as f32 / u32::MAX as f32
    }
}

/// Generate formant-filtered waveform for choir/vocal synthesis
/// vowel: 0.0 = "ah", 0.5 = "oh", 1.0 = "ee"
pub fn formant_filter(input: f32, vowel: f32, state: &mut [f32; 4]) -> f32 {
    // Simplified formant frequencies for different vowels
    let (f1, f2) = if vowel < 0.33 {
        (800.0, 1200.0) // "ah"
    } else if vowel < 0.66 {
        (450.0, 800.0) // "oh"
    } else {
        (300.0, 2300.0) // "ee"
    };

    // Two bandpass filters for formants
    let cutoff1 = (2.0 * std::f32::consts::PI * f1 / SAMPLE_RATE).min(0.5);
    let cutoff2 = (2.0 * std::f32::consts::PI * f2 / SAMPLE_RATE).min(0.5);

    state[0] += cutoff1 * (input - state[0]);
    state[1] += cutoff1 * (state[0] - state[1]);
    let formant1 = state[0] - state[1];

    state[2] += cutoff2 * (input - state[2]);
    state[3] += cutoff2 * (state[2] - state[3]);
    let formant2 = state[2] - state[3];

    formant1 * 0.6 + formant2 * 0.4
}
