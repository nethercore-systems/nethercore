//! Percussion instruments: Timpani, Snare, Cymbal

use super::super::common::{SimpleRng, SAMPLE_RATE};
use super::{BiquadHP, BiquadLP};
use std::f32::consts::PI;

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
        let body_env = if t < 0.003 {
            t / 0.003
        } else {
            (-t * 2.8).exp()
        };

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
        let body_env = if t < 0.002 {
            t / 0.002
        } else {
            (-t * 15.0).exp()
        };

        // Snare wires (high frequencies, longer decay)
        let snare = snare_filter.process(noise);
        let snare_env = if t < 0.001 {
            t / 0.001
        } else {
            (-t * 10.0).exp()
        };

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

        let env = if t < 0.008 {
            t / 0.008
        } else {
            (-t * 1.8).exp()
        };

        let sample = mix * env * 26000.0;
        output.push(sample.clamp(-32767.0, 32767.0) as i16);
    }

    output
}
