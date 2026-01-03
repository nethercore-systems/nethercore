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

