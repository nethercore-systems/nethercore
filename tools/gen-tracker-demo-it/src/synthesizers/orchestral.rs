//! Orchestral instrument synthesis
//!
//! Instruments for "Nether Dawn" - Epic/Orchestral at 90 BPM in D major
//!
//! Professional quality synthesis with:
//! - PolyBLEP anti-aliased oscillators
//! - Proper biquad filters
//! - Multiple detuned voices for richness
//! - Higher output levels

use super::common::{SimpleRng, SAMPLE_RATE};
use std::f32::consts::PI;

// =============================================================================
// DSP Utilities
// =============================================================================

/// Soft saturation using tanh for analog warmth
fn soft_saturate(x: f32) -> f32 {
    x.tanh()
}

/// PolyBLEP (Polynomial Band-Limited Step) for anti-aliased discontinuities
fn poly_blep(t: f32, dt: f32) -> f32 {
    if t < dt {
        let t = t / dt;
        2.0 * t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + 2.0 * t + 1.0
    } else {
        0.0
    }
}

/// Band-limited sawtooth using PolyBLEP
fn blep_saw(phase: f32, dt: f32) -> f32 {
    let naive = 2.0 * phase - 1.0;
    naive - poly_blep(phase, dt)
}

/// Band-limited square wave using PolyBLEP
fn blep_square(phase: f32, dt: f32) -> f32 {
    let naive = if phase < 0.5 { 1.0 } else { -1.0 };
    naive + poly_blep(phase, dt) - poly_blep((phase + 0.5) % 1.0, dt)
}

/// Band-limited triangle (integrated square)
fn blep_triangle(phase: f32, dt: f32) -> f32 {
    // Use a simple approximation that's still cleaner than naive
    let square = blep_square(phase, dt);
    // First-order integrator (leaky)
    static mut INTEGRATOR: f32 = 0.0;
    unsafe {
        INTEGRATOR = 0.995 * INTEGRATOR + square * dt * 4.0;
        INTEGRATOR.clamp(-1.0, 1.0)
    }
}

/// Biquad lowpass filter with resonance
struct BiquadLP {
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
}

impl BiquadLP {
    fn new(cutoff_hz: f32, q: f32) -> Self {
        let mut filter = Self {
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
            b0: 0.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
        };
        filter.set_params(cutoff_hz, q);
        filter
    }

    fn set_params(&mut self, cutoff_hz: f32, q: f32) {
        let w0 = 2.0 * PI * cutoff_hz / SAMPLE_RATE;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cos_w0) / 2.0) / a0;
        self.b1 = (1.0 - cos_w0) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
              - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Biquad highpass filter
struct BiquadHP {
    x1: f32, x2: f32,
    y1: f32, y2: f32,
    b0: f32, b1: f32, b2: f32,
    a1: f32, a2: f32,
}

impl BiquadHP {
    fn new(cutoff_hz: f32, q: f32) -> Self {
        let mut filter = Self {
            x1: 0.0, x2: 0.0,
            y1: 0.0, y2: 0.0,
            b0: 0.0, b1: 0.0, b2: 0.0,
            a1: 0.0, a2: 0.0,
        };
        filter.set_params(cutoff_hz, q);
        filter
    }

    fn set_params(&mut self, cutoff_hz: f32, q: f32) {
        let w0 = 2.0 * PI * cutoff_hz / SAMPLE_RATE;
        let cos_w0 = w0.cos();
        let sin_w0 = w0.sin();
        let alpha = sin_w0 / (2.0 * q);

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 + cos_w0) / 2.0) / a0;
        self.b1 = -(1.0 + cos_w0) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
              - self.a1 * self.y1 - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// State variable filter for smooth modulation
struct SVFilter {
    low: f32,
    band: f32,
}

impl SVFilter {
    fn new() -> Self {
        Self { low: 0.0, band: 0.0 }
    }

    fn process(&mut self, input: f32, cutoff: f32, res: f32) -> (f32, f32, f32) {
        let f = (cutoff / SAMPLE_RATE).min(0.45);
        let q = 1.0 - res.min(0.95);

        self.low += f * self.band;
        let high = input - self.low - q * self.band;
        self.band += f * high;

        (self.low, self.band, high)
    }
}

/// Formant filter for vocal synthesis - improved version
struct FormantFilter {
    bp1: SVFilter,
    bp2: SVFilter,
    bp3: SVFilter,
}

impl FormantFilter {
    fn new() -> Self {
        Self {
            bp1: SVFilter::new(),
            bp2: SVFilter::new(),
            bp3: SVFilter::new(),
        }
    }

    fn process(&mut self, input: f32, vowel: f32) -> f32 {
        // Formant frequencies for different vowels
        // vowel 0.0 = "ah", 0.5 = "oh", 1.0 = "ee"
        let (f1, f2, f3) = if vowel < 0.5 {
            let t = vowel * 2.0;
            // "ah" to "oh"
            let f1 = 800.0 * (1.0 - t) + 450.0 * t;
            let f2 = 1200.0 * (1.0 - t) + 800.0 * t;
            let f3 = 2500.0 * (1.0 - t) + 2500.0 * t;
            (f1, f2, f3)
        } else {
            let t = (vowel - 0.5) * 2.0;
            // "oh" to "ee"
            let f1 = 450.0 * (1.0 - t) + 300.0 * t;
            let f2 = 800.0 * (1.0 - t) + 2300.0 * t;
            let f3 = 2500.0 * (1.0 - t) + 3000.0 * t;
            (f1, f2, f3)
        };

        let res = 0.7;
        let (_, band1, _) = self.bp1.process(input, f1, res);
        let (_, band2, _) = self.bp2.process(input, f2, res);
        let (_, band3, _) = self.bp3.process(input, f3, res);

        // Mix formants with decreasing amplitude
        band1 * 1.0 + band2 * 0.7 + band3 * 0.4
    }
}

// =============================================================================
// String Instruments
// =============================================================================

/// Cello: Rich ensemble of detuned saws with warm filtering
pub fn generate_strings_cello() -> Vec<i16> {
    let duration = 3.0;
    let freq = 146.83; // D3
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // 6 detuned oscillators for richness
    let detune = [-12.0, -5.0, -2.0, 2.0, 5.0, 12.0]; // cents
    let mut phases = [0.0f32; 6];

    let mut filter = BiquadLP::new(1200.0, 0.7);
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Subtle vibrato that develops over time
        let vib_amount = (t / 0.5).min(1.0) * 0.004;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.5 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 6.0;

        // Warm lowpass that opens slightly over attack
        let cutoff = 800.0 + 600.0 * (1.0 - (-t * 3.0).exp());
        filter.set_params(cutoff, 0.7);
        let filtered = filter.process(sum);

        // Apply soft saturation for warmth
        let saturated = soft_saturate(filtered * 1.5) * 0.7;

        // Smooth envelope: slow attack, sustain, gentle release
        let env = if t < 0.12 {
            (t / 0.12).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Viola: Mid-range strings, slightly brighter than cello
pub fn generate_strings_viola() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-10.0, -4.0, -1.0, 1.0, 4.0, 10.0];
    let mut phases = [0.0f32; 6];

    let mut filter = BiquadLP::new(2000.0, 0.6);
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let vib_amount = (t / 0.4).min(1.0) * 0.005;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.8 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 6.0;

        let cutoff = 1200.0 + 1000.0 * (1.0 - (-t * 3.0).exp());
        filter.set_params(cutoff, 0.6);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.4) * 0.75;

        let env = if t < 0.10 {
            (t / 0.10).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Violin: Brightest strings with expressive vibrato
pub fn generate_strings_violin() -> Vec<i16> {
    let duration = 3.0;
    let freq = 587.33; // D5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-8.0, -3.0, 0.0, 3.0, 8.0];
    let mut phases = [0.0f32; 5];

    let mut filter = BiquadLP::new(3500.0, 0.5);
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Delayed vibrato that intensifies
        let vib_depth = if t < 0.15 { 0.0 } else { ((t - 0.15) / 0.3).min(1.0) };
        let vibrato = 1.0 + 0.008 * vib_depth * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.5 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 5.0;

        let cutoff = 2000.0 + 2000.0 * (1.0 - (-t * 4.0).exp());
        filter.set_params(cutoff, 0.5);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.3) * 0.8;

        let env = if t < 0.08 {
            (t / 0.08).powf(1.8)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 29000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// =============================================================================
// Brass Instruments
// =============================================================================

/// French horn: Warm, mellow brass with resonant filter
pub fn generate_brass_horn() -> Vec<i16> {
    let duration = 2.0;
    let freq = 146.83; // D3
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-6.0, -2.0, 2.0, 6.0];
    let mut phases = [0.0f32; 4];

    let mut filter = BiquadLP::new(1000.0, 1.5); // Resonant for horn character
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let vib_amount = (t / 0.3).min(1.0) * 0.003;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.0 / SAMPLE_RATE;

        // Mix of square and saw for horn character
        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            let sq = blep_square(phases[j], dt);
            let saw = blep_saw(phases[j], dt);
            sum += sq * 0.6 + saw * 0.4;
        }
        sum /= 4.0;

        // Filter opens during attack for "blat"
        let cutoff = 600.0 + 800.0 * (1.0 - (-t * 5.0).exp());
        filter.set_params(cutoff, 1.5);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.6) * 0.65;

        let env = if t < 0.08 {
            (t / 0.08).powf(1.3)
        } else if t < 1.5 {
            1.0
        } else {
            (-(t - 1.5) * 2.0).exp()
        };

        let sample = saturated * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Trumpet: Bright, punchy brass for fanfares
pub fn generate_brass_trumpet() -> Vec<i16> {
    let duration = 1.5;
    let freq = 587.33; // D5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-4.0, 0.0, 4.0];
    let mut phases = [0.0f32; 3];

    let mut filter = BiquadLP::new(3000.0, 1.2);

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0);
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 3.0;

        // Bright attack that mellows
        let cutoff = 4000.0 + 2000.0 * (-t * 4.0).exp();
        filter.set_params(cutoff, 1.2);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.5) * 0.7;

        let env = if t < 0.025 {
            (t / 0.025).powf(0.8)
        } else if t < 1.0 {
            1.0 - (t - 0.025) * 0.05
        } else {
            0.95 * (-(t - 1.0) * 3.0).exp()
        };

        let sample = saturated * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// =============================================================================
// Woodwinds
// =============================================================================

/// Flute: Pure, airy tone with subtle breath noise
pub fn generate_flute() -> Vec<i16> {
    let duration = 2.0;
    let freq = 587.33; // D5
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phase = 0.0f32;
    let mut vibrato_phase = 0.0f32;
    let mut rng = SimpleRng::new(54321);
    let mut noise_filter = BiquadLP::new(8000.0, 0.5);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Light vibrato
        let vibrato = 1.0 + 0.004 * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 6.0 / SAMPLE_RATE;

        phase += freq * vibrato / SAMPLE_RATE;
        phase %= 1.0;

        // Mix of sine and triangle for flute tone
        let sine = (phase * 2.0 * PI).sin();
        let tri = if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        };
        let tone = sine * 0.7 + tri * 0.3;

        // Subtle breath noise
        let noise = rng.next_f32() * 2.0 - 1.0;
        let filtered_noise = noise_filter.process(noise);

        // Breath more prominent during attack
        let breath_amount = 0.05 + 0.1 * (-t * 8.0).exp();
        let mix = tone + filtered_noise * breath_amount;

        let env = if t < 0.06 {
            (t / 0.06).powf(1.2)
        } else if t < 1.7 {
            1.0
        } else {
            (-(t - 1.7) * 3.5).exp()
        };

        let sample = mix * env * 28000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// =============================================================================
// Percussion
// =============================================================================

/// Timpani: Deep, resonant orchestral drum
pub fn generate_timpani() -> Vec<i16> {
    let duration = 1.5;
    let freq = 73.42; // D2
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(77777);

    let mut phase = 0.0f32;
    let mut noise_filter = BiquadLP::new(400.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Pitch drops slightly at attack (membrane behavior)
        let pitch_drop = 1.0 + 0.1 * (-t * 30.0).exp();
        phase += freq * pitch_drop / SAMPLE_RATE;
        let sine = (phase * 2.0 * PI).sin();

        // Add harmonics for fullness
        let h2 = (phase * 2.0 * 2.0 * PI).sin() * 0.3;
        let h3 = (phase * 3.0 * 2.0 * PI).sin() * 0.15;
        let tone = sine + h2 + h3;

        // Transient noise from mallet
        let noise = rng.next_f32() * 2.0 - 1.0;
        let filtered_noise = noise_filter.process(noise);
        let noise_env = (-t * 40.0).exp();

        // Body resonance envelope
        let body_env = if t < 0.003 { t / 0.003 } else { (-t * 2.8).exp() };

        let sample = (tone * body_env * 0.85 + filtered_noise * noise_env * 0.5) * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Orchestral snare: Crisp with snare wire buzz
pub fn generate_snare_orch() -> Vec<i16> {
    let duration = 0.4;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(88888);

    // Two bandpass filters for body and snare
    let mut body_filter = BiquadLP::new(400.0, 2.0);
    let mut snare_filter = BiquadHP::new(3000.0, 1.5);
    let mut body_hp = BiquadHP::new(100.0, 0.7);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Body (low-mid frequencies)
        let body = body_hp.process(body_filter.process(noise));
        let body_env = if t < 0.002 { t / 0.002 } else { (-t * 15.0).exp() };

        // Snare wires (high frequencies, longer decay)
        let snare = snare_filter.process(noise);
        let snare_env = if t < 0.001 { t / 0.001 } else { (-t * 10.0).exp() };

        let sample = (body * body_env * 0.7 + snare * snare_env * 0.5) * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Cymbal crash: Metallic, shimmering
pub fn generate_cymbal_crash() -> Vec<i16> {
    let duration = 2.0;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(99999);

    // Multiple bandpass filters for metallic complexity
    let mut bp1 = BiquadHP::new(4000.0, 1.0);
    let mut bp2 = BiquadHP::new(6000.0, 1.2);
    let mut bp3 = BiquadHP::new(8000.0, 0.8);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;
        let noise = rng.next_f32() * 2.0 - 1.0;

        // Multiple bands for shimmer
        let band1 = bp1.process(noise);
        let band2 = bp2.process(noise);
        let band3 = bp3.process(noise);

        let mix = band1 * 0.5 + band2 * 0.3 + band3 * 0.2;

        let env = if t < 0.008 { t / 0.008 } else { (-t * 1.8).exp() };

        let sample = mix * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// =============================================================================
// Plucked Instruments
// =============================================================================

/// Harp: Crystalline plucked strings
pub fn generate_harp_gliss() -> Vec<i16> {
    let duration = 1.5;
    let freq = 440.0; // A4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phase = 0.0f32;
    let mut filter = BiquadLP::new(6000.0, 0.5);

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        phase += freq / SAMPLE_RATE;
        phase %= 1.0;

        // Triangle + sine for plucked character
        let tri = if phase < 0.5 {
            4.0 * phase - 1.0
        } else {
            3.0 - 4.0 * phase
        };
        let sine = (phase * 2.0 * PI).sin();
        let tone = tri * 0.6 + sine * 0.4;

        // Filter darkens over time (string damping)
        let cutoff = 8000.0 * (-t * 2.5).exp() + 800.0;
        filter.set_params(cutoff, 0.5);
        let filtered = filter.process(tone);

        // Fast attack, smooth decay
        let env = if t < 0.004 { t / 0.004 } else { (-t * 3.0).exp() };

        let sample = filtered * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Piano: Rich hammer sound with decaying harmonics
pub fn generate_piano() -> Vec<i16> {
    let duration = 2.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phases = [0.0f32; 6];
    let harmonics = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let amplitudes = [1.0, 0.5, 0.33, 0.25, 0.15, 0.1];

    let mut filter = BiquadLP::new(4000.0, 0.7);
    let mut rng = SimpleRng::new(11111);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, (harm, amp)) in harmonics.iter().zip(amplitudes.iter()).enumerate() {
            phases[j] += freq * harm / SAMPLE_RATE;
            phases[j] %= 1.0;
            // Each harmonic decays at different rate
            let harm_decay = (-t * (1.5 + j as f32 * 0.5)).exp();
            sum += (phases[j] * 2.0 * PI).sin() * amp * harm_decay;
        }
        sum /= 2.0;

        // Hammer transient (noise + click)
        let hammer_noise = rng.next_f32() * 2.0 - 1.0;
        let hammer_env = if t < 0.002 { t / 0.002 } else { (-t * 80.0).exp() };
        let hammer = hammer_noise * hammer_env * 0.2;

        // Filter darkens over time
        let cutoff = 5000.0 * (-t * 1.5).exp() + 1000.0;
        filter.set_params(cutoff, 0.7);
        let filtered = filter.process(sum + hammer);

        let env = if t < 0.003 { t / 0.003 } else { (-t * 1.8).exp() };

        let sample = filtered * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// =============================================================================
// Vocal
// =============================================================================

/// Choir "ah": Lush ensemble with formant filtering
pub fn generate_choir_ah() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // 8 voices for rich choir
    let detune = [-15.0, -8.0, -4.0, -1.0, 1.0, 4.0, 8.0, 15.0];
    let mut phases = [0.0f32; 8];

    let mut formant = FormantFilter::new();
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow, subtle vibrato
        let vib_amount = (t / 0.4).min(1.0) * 0.004;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.0 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            // Slight random variation per voice
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 8.0;

        // "ah" vowel formant
        let filtered = formant.process(sum, 0.0);

        let saturated = soft_saturate(filtered * 2.0) * 0.55;

        // Very slow attack for vocal quality
        let env = if t < 0.2 {
            (t / 0.2).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Choir "oh": Rounder, darker vowel
pub fn generate_choir_oh() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let detune = [-15.0, -8.0, -4.0, -1.0, 1.0, 4.0, 8.0, 15.0];
    let mut phases = [0.0f32; 8];

    let mut formant = FormantFilter::new();
    let mut vibrato_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        let vib_amount = (t / 0.4).min(1.0) * 0.004;
        let vibrato = 1.0 + vib_amount * (vibrato_phase * 2.0 * PI).sin();
        vibrato_phase += 5.2 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0) * vibrato;
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 8.0;

        // "oh" vowel formant
        let filtered = formant.process(sum, 0.5);

        let saturated = soft_saturate(filtered * 2.0) * 0.55;

        let env = if t < 0.2 {
            (t / 0.2).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// =============================================================================
// Bass & Pads
// =============================================================================

/// Epic bass: Powerful sub with overtones
pub fn generate_bass_epic() -> Vec<i16> {
    let duration = 2.0;
    let freq = 73.42; // D2
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    let mut phase_sub = 0.0f32;
    let mut phases = [0.0f32; 3];
    let detune = [-5.0, 0.0, 5.0];

    let mut filter = BiquadLP::new(600.0, 1.2);

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Pure sub sine
        phase_sub += freq * 0.5 / SAMPLE_RATE;
        let sub = (phase_sub * 2.0 * PI).sin();

        // Detuned saws an octave up
        let mut saw_sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0);
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            saw_sum += blep_saw(phases[j], dt);
        }
        saw_sum /= 3.0;

        let mix = sub * 0.6 + saw_sum * 0.4;

        // Resonant filter adds growl
        let cutoff = 400.0 + 400.0 * (1.0 - (-t * 5.0).exp());
        filter.set_params(cutoff, 1.2);
        let filtered = filter.process(mix);

        let saturated = soft_saturate(filtered * 1.8) * 0.6;

        let env = if t < 0.05 {
            (t / 0.05).powf(1.2)
        } else if t < 1.5 {
            1.0
        } else {
            (-(t - 1.5) * 2.5).exp()
        };

        let sample = saturated * env * 31000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

/// Orchestra pad: Lush background texture
pub fn generate_pad_orchestra() -> Vec<i16> {
    let duration = 3.0;
    let freq = 293.66; // D4
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);

    // 7 voices for rich pad
    let detune = [-15.0, -8.0, -3.0, 0.0, 3.0, 8.0, 15.0];
    let mut phases = [0.0f32; 7];

    let mut filter = BiquadLP::new(1500.0, 0.5);
    let mut lfo_phase = 0.0f32;

    let dt = freq / SAMPLE_RATE;

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Slow filter modulation
        let lfo = 0.5 + 0.5 * (lfo_phase * 2.0 * PI).sin();
        lfo_phase += 0.1 / SAMPLE_RATE;

        let mut sum = 0.0f32;
        for (j, &cents) in detune.iter().enumerate() {
            let freq_mult = (2.0f32).powf(cents / 1200.0);
            phases[j] += freq * freq_mult / SAMPLE_RATE;
            phases[j] %= 1.0;
            sum += blep_saw(phases[j], dt);
        }
        sum /= 7.0;

        // Slowly modulating filter
        let cutoff = 800.0 + 800.0 * lfo;
        filter.set_params(cutoff, 0.5);
        let filtered = filter.process(sum);

        let saturated = soft_saturate(filtered * 1.3) * 0.8;

        // Very slow attack/release for pad character
        let env = if t < 0.4 {
            (t / 0.4).powf(2.0)
        } else if t < 2.5 {
            1.0
        } else {
            (-(t - 2.5) * 2.0).exp()
        };

        let sample = saturated * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}

// =============================================================================
// Effects
// =============================================================================

/// Epic FX: Cinematic riser/impact
pub fn generate_fx_epic() -> Vec<i16> {
    let duration = 2.0;
    let samples = (SAMPLE_RATE * duration) as usize;
    let mut output = Vec::with_capacity(samples);
    let mut rng = SimpleRng::new(12121);

    let mut phase = 0.0f32;
    let mut noise_filter = BiquadLP::new(2000.0, 1.0);
    let mut tone_filter = BiquadLP::new(1000.0, 2.0);

    for i in 0..samples {
        let t = i as f32 / SAMPLE_RATE;

        // Rising pitch
        let freq = 80.0 + t * t * 300.0; // Exponential rise
        phase += freq / SAMPLE_RATE;
        let sine = (phase * 2.0 * PI).sin();

        // Add harmonics for fullness
        let h2 = (phase * 2.0 * 2.0 * PI).sin() * 0.5;
        let h3 = (phase * 3.0 * 2.0 * PI).sin() * 0.3;
        let tone = sine + h2 + h3;

        // Rising noise
        let noise = rng.next_f32() * 2.0 - 1.0;
        let noise_cutoff = 500.0 + t * 3000.0;
        noise_filter.set_params(noise_cutoff, 1.0);
        let filtered_noise = noise_filter.process(noise);

        // Rising filter on tone
        let tone_cutoff = 200.0 + t * 2000.0;
        tone_filter.set_params(tone_cutoff, 2.0);
        let filtered_tone = tone_filter.process(tone);

        let mix = filtered_tone * 0.6 + filtered_noise * 0.4;

        // Build envelope
        let env = (t / 2.0).powf(1.5);

        let saturated = soft_saturate(mix * env * 2.0) * 0.6;

        let sample = saturated * 30000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
