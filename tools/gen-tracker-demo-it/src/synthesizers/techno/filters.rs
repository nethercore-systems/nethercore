//! DSP filters and utilities for techno synthesis

use std::f32::consts::PI;

const TWO_PI: f32 = 2.0 * PI;

/// Soft saturation using tanh for analog warmth
#[inline]
pub(super) fn soft_saturate(x: f32) -> f32 {
    x.tanh()
}

/// 2-pole state variable filter (for TB-303 bandpass)
pub(super) struct StateVariableFilter {
    low: f32,
    band: f32,
}

impl StateVariableFilter {
    pub(super) fn new() -> Self {
        Self {
            low: 0.0,
            band: 0.0,
        }
    }

    pub(super) fn process(&mut self, input: f32, cutoff: f32, resonance: f32) -> (f32, f32, f32) {
        let f = (cutoff * PI).min(0.99);
        let q = 1.0 - resonance.min(0.95);

        self.low += f * self.band;
        let high = input - self.low - q * self.band;
        self.band += f * high;

        (self.low, self.band, high)
    }
}

/// Biquad low-pass filter (professional quality)
pub(super) struct BiquadLP {
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
    pub(super) fn new() -> Self {
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

    pub(super) fn set_params(&mut self, freq: f32, q: f32) {
        use super::super::common::SAMPLE_RATE;
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

    pub(super) fn process(&mut self, input: f32) -> f32 {
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

/// Biquad high-pass filter (professional quality)
pub(super) struct BiquadHP {
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

impl BiquadHP {
    pub(super) fn new() -> Self {
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

    pub(super) fn set_params(&mut self, freq: f32, q: f32) {
        use super::super::common::SAMPLE_RATE;
        let w0 = TWO_PI * (freq / SAMPLE_RATE).min(0.49);
        let alpha = w0.sin() / (2.0 * q);
        let cos_w0 = w0.cos();

        let a0 = 1.0 + alpha;
        self.b0 = ((1.0 + cos_w0) / 2.0) / a0;
        self.b1 = -(1.0 + cos_w0) / a0;
        self.b2 = self.b0;
        self.a1 = (-2.0 * cos_w0) / a0;
        self.a2 = (1.0 - alpha) / a0;
    }

    pub(super) fn process(&mut self, input: f32) -> f32 {
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
