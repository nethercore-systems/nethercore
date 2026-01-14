//! DSP utilities and filters for DnB synthesis

use super::super::common::SAMPLE_RATE;
use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

// ============================================================================
// DSP Utilities
// ============================================================================

/// Soft saturation for warmth
pub fn soft_saturate(x: f32) -> f32 {
    x.tanh()
}

/// Hard clip with soft knee
pub fn soft_clip(x: f32, threshold: f32) -> f32 {
    if x.abs() < threshold {
        x
    } else {
        x.signum()
            * (threshold
                + (1.0 - threshold) * soft_saturate((x.abs() - threshold) / (1.0 - threshold)))
    }
}

/// PolyBLEP for anti-aliased discontinuities
pub fn poly_blep(t: f32, dt: f32) -> f32 {
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

/// Band-limited sawtooth
pub fn blep_saw(phase: f32, dt: f32) -> f32 {
    let naive = 2.0 * phase - 1.0;
    naive - poly_blep(phase, dt)
}

/// Band-limited square
pub fn blep_square(phase: f32, dt: f32) -> f32 {
    let naive = if phase < 0.5 { 1.0 } else { -1.0 };
    naive + poly_blep(phase, dt) - poly_blep((phase + 0.5) % 1.0, dt)
}

/// 2-pole state variable filter
pub struct StateVariableFilter {
    low: f32,
    band: f32,
}

impl StateVariableFilter {
    pub fn new() -> Self {
        Self {
            low: 0.0,
            band: 0.0,
        }
    }

    pub fn process(&mut self, input: f32, cutoff: f32, resonance: f32) -> (f32, f32, f32) {
        let f = (cutoff * PI).min(0.99);
        let q = 1.0 - resonance.min(0.95);

        self.low += f * self.band;
        let high = input - self.low - q * self.band;
        self.band += f * high;

        (self.low, self.band, high)
    }
}

/// Biquad low-pass filter
pub struct BiquadLP {
    x1: f32,
    x2: f32,
    y1: f32,
    y2: f32,
    b0: f32,
    b1: f32,
    b2: f32,
    a1: f32,
    a2: f32,
}

impl BiquadLP {
    pub fn new() -> Self {
        Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            b0: 1.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        }
    }

    pub fn set_params(&mut self, freq: f32, q: f32) {
        let w0 = TWO_PI * (freq / SAMPLE_RATE).min(0.49);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 - cos_w0) / 2.0) / a0;
        self.b1 = (1.0 - cos_w0) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    pub fn process(&mut self, input: f32) -> f32 {
        let output = self.b0 * input + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = input;
        self.y2 = self.y1;
        self.y1 = output;
        output
    }
}
