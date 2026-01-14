//! Orchestral instrument synthesis
//!
//! Instruments for "Nether Dawn" - Epic/Orchestral at 90 BPM in D major
//!
//! Professional quality synthesis with:
//! - PolyBLEP anti-aliased oscillators
//! - Proper biquad filters
//! - Multiple detuned voices for richness
//! - Higher output levels

use super::common::SAMPLE_RATE;
use std::f32::consts::PI;

mod strings;
mod brass_winds;
mod percussion;
mod plucked_vocal;
mod bass_effects;

// Re-export public API
pub use strings::{generate_strings_cello, generate_strings_viola, generate_strings_violin};
pub use brass_winds::{generate_brass_horn, generate_brass_trumpet, generate_flute};
pub use percussion::{generate_timpani, generate_snare_orch, generate_cymbal_crash};
pub use plucked_vocal::{generate_harp_gliss, generate_piano, generate_choir_ah, generate_choir_oh};
pub use bass_effects::{generate_bass_epic, generate_pad_orchestra, generate_fx_epic};

// =============================================================================
// DSP Utilities (shared by all instruments)
// =============================================================================

/// Soft saturation using tanh for analog warmth
pub(super) fn soft_saturate(x: f32) -> f32 {
    x.tanh()
}

/// PolyBLEP (Polynomial Band-Limited Step) for anti-aliased discontinuities
pub(super) fn poly_blep(t: f32, dt: f32) -> f32 {
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
pub(super) fn blep_saw(phase: f32, dt: f32) -> f32 {
    let naive = 2.0 * phase - 1.0;
    naive - poly_blep(phase, dt)
}

/// Band-limited square wave using PolyBLEP
pub(super) fn blep_square(phase: f32, dt: f32) -> f32 {
    let naive = if phase < 0.5 { 1.0 } else { -1.0 };
    naive + poly_blep(phase, dt) - poly_blep((phase + 0.5) % 1.0, dt)
}

/// Band-limited triangle (integrated square)
pub(super) fn _blep_triangle(phase: f32, dt: f32) -> f32 {
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
    pub(super) fn new(cutoff_hz: f32, q: f32) -> Self {
        let mut filter = Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        };
        filter.set_params(cutoff_hz, q);
        filter
    }

    pub(super) fn set_params(&mut self, cutoff_hz: f32, q: f32) {
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

    pub(super) fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// Biquad highpass filter
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
    pub(super) fn new(cutoff_hz: f32, q: f32) -> Self {
        let mut filter = Self {
            x1: 0.0,
            x2: 0.0,
            y1: 0.0,
            y2: 0.0,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            a1: 0.0,
            a2: 0.0,
        };
        filter.set_params(cutoff_hz, q);
        filter
    }

    pub(super) fn set_params(&mut self, cutoff_hz: f32, q: f32) {
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

    pub(super) fn process(&mut self, x: f32) -> f32 {
        let y = self.b0 * x + self.b1 * self.x1 + self.b2 * self.x2
            - self.a1 * self.y1
            - self.a2 * self.y2;
        self.x2 = self.x1;
        self.x1 = x;
        self.y2 = self.y1;
        self.y1 = y;
        y
    }
}

/// State variable filter for smooth modulation
pub(super) struct SVFilter {
    low: f32,
    band: f32,
}

impl SVFilter {
    pub(super) fn new() -> Self {
        Self {
            low: 0.0,
            band: 0.0,
        }
    }

    pub(super) fn process(&mut self, input: f32, cutoff: f32, res: f32) -> (f32, f32, f32) {
        let f = (cutoff / SAMPLE_RATE).min(0.45);
        let q = 1.0 - res.min(0.95);

        self.low += f * self.band;
        let high = input - self.low - q * self.band;
        self.band += f * high;

        (self.low, self.band, high)
    }
}

/// Formant filter for vocal synthesis - improved version
pub(super) struct FormantFilter {
    bp1: SVFilter,
    bp2: SVFilter,
    bp3: SVFilter,
}

impl FormantFilter {
    pub(super) fn new() -> Self {
        Self {
            bp1: SVFilter::new(),
            bp2: SVFilter::new(),
            bp3: SVFilter::new(),
        }
    }

    pub(super) fn process(&mut self, input: f32, vowel: f32) -> f32 {
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
