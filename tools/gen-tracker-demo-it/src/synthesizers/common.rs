//! Common synthesizer utilities shared across all genres
//!
//! Provides high-quality synthesis helpers including PolyBLEP anti-aliasing,
//! oversampling, exponential envelopes, and analog warmth.

pub use gen_tracker_common::{SimpleRng, SAMPLE_RATE};

// ============================================================================
// PolyBLEP Anti-Aliasing
// ============================================================================

/// PolyBLEP (Polynomial Bandlimited Step) residual for anti-aliasing
///
/// Corrects the discontinuity at phase wrapping to eliminate aliasing.
/// Apply this to the naive waveform at discontinuities.
///
/// # Arguments
/// * `t` - Phase normalized to 0..1
/// * `dt` - Phase increment per sample (frequency / sample_rate)
#[inline]
pub fn poly_blep(t: f32, dt: f32) -> f32 {
    // Handle discontinuity at t=0 (phase wrap)
    if t < dt {
        let t = t / dt;
        return t + t - t * t - 1.0;
    }
    // Handle discontinuity at t=1 (before phase wrap)
    else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        return t * t + t + t + 1.0;
    }
    0.0
}

/// Generate anti-aliased sawtooth wave using PolyBLEP
///
/// # Arguments
/// * `phase` - Current phase (0..1)
/// * `phase_inc` - Phase increment per sample
#[inline]
pub fn sawtooth_blep(phase: f32, phase_inc: f32) -> f32 {
    let naive = 2.0 * phase - 1.0; // Naive sawtooth
    let correction = poly_blep(phase, phase_inc);
    naive - correction
}

// ============================================================================
// Exponential Envelopes
// ============================================================================

/// Exponential decay envelope (natural sound)
///
/// Much more natural than linear decay. Models capacitor discharge.
///
/// # Arguments
/// * `t` - Time (0..1)
/// * `decay_rate` - Higher = faster decay (typical: 3.0 - 8.0)
#[inline]
pub fn exp_decay(t: f32, decay_rate: f32) -> f32 {
    (-decay_rate * t).exp()
}

/// Exponential attack envelope
///
/// # Arguments
/// * `t` - Time (0..1)
/// * `attack_rate` - Higher = faster attack (typical: 5.0 - 15.0)
#[inline]
pub fn exp_attack(t: f32, attack_rate: f32) -> f32 {
    1.0 - (-attack_rate * t).exp()
}

/// ADSR envelope with exponential curves
///
/// # Arguments
/// * `t` - Time (0..1) normalized to total envelope duration
/// * `attack` - Attack time fraction (0..1)
/// * `decay` - Decay time fraction (0..1)
/// * `sustain` - Sustain level (0..1)
/// * `release` - Release time fraction (0..1)
#[inline]
pub fn adsr_exp(t: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> f32 {
    let attack_end = attack;
    let decay_end = attack + decay;
    let release_start = 1.0 - release;

    if t < attack_end {
        // Attack phase
        exp_attack(t / attack, 8.0)
    } else if t < decay_end {
        // Decay phase
        let decay_t = (t - attack_end) / decay;
        1.0 - (1.0 - sustain) * (1.0 - exp_decay(1.0 - decay_t, 5.0))
    } else if t < release_start {
        // Sustain phase
        sustain
    } else {
        // Release phase
        let release_t = (t - release_start) / release;
        sustain * exp_decay(release_t, 5.0)
    }
}

// ============================================================================
// Analog Warmth & Character
// ============================================================================

/// Soft clipping / saturation (analog warmth)
///
/// Applies gentle waveshaping to add harmonics and prevent harsh clipping.
///
/// # Arguments
/// * `x` - Input signal
/// * `drive` - Saturation amount (1.0 = clean, 2.0 = mild, 4.0 = heavy)
#[inline]
pub fn soft_clip(x: f32, drive: f32) -> f32 {
    let x = x * drive;
    // Cubic soft clipper
    if x > 1.0 {
        2.0 / 3.0
    } else if x < -1.0 {
        -2.0 / 3.0
    } else {
        x - (x * x * x) / 3.0
    }
}

// ============================================================================
// Oversampling / Downsampling
// ============================================================================

/// Simple 4× oversampling with linear interpolation
///
/// Synthesizes at 4× sample rate to reduce aliasing, then downsamples.
///
/// # Arguments
/// * `generate_fn` - Function that generates samples at high sample rate
/// * `num_samples` - Number of output samples (at base rate)
pub fn oversample_4x<F>(mut generate_fn: F, num_samples: usize) -> Vec<i16>
where
    F: FnMut(usize) -> f32,
{
    let mut output = Vec::with_capacity(num_samples);

    // Simple 4× downsample with averaging (crude low-pass filter)
    for i in 0..num_samples {
        let base_idx = i * 4;
        let s0 = generate_fn(base_idx);
        let s1 = generate_fn(base_idx + 1);
        let s2 = generate_fn(base_idx + 2);
        let s3 = generate_fn(base_idx + 3);

        // Average 4 samples (simple low-pass)
        let avg = (s0 + s1 + s2 + s3) * 0.25;
        output.push((avg * 32767.0).clamp(-32768.0, 32767.0) as i16);
    }

    output
}
