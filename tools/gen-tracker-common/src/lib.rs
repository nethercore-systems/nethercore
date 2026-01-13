//! Common utilities for procedural tracker music generators
//!
//! This crate provides shared functionality for generating tracker module files:
//! - WAV file writing
//! - Audio synthesis utilities (sample rate, fade functions, RNG, formant filters)

pub mod synth;
pub mod wav_writer;

// Re-export commonly used items at crate root
pub use synth::{apply_fades, formant_filter, SimpleRng, SAMPLE_RATE};
pub use wav_writer::write_wav;

#[cfg(test)]
mod tests {
    use super::{apply_fades, formant_filter, write_wav, SimpleRng};
    use std::fs;

    #[test]
    fn apply_fades_fades_in_and_out_without_increasing_peak() {
        let mut samples = vec![10_000i16; 1000];
        apply_fades(&mut samples);

        assert_eq!(samples[0], 0);
        assert!(samples[43] < 10_000);
        assert_eq!(samples[44], 10_000);

        assert_eq!(samples[889], 10_000);
        assert!(samples[999] < 10_000);

        let peak = samples.iter().map(|s| (*s as i32).abs()).max().unwrap_or(0);
        assert!(peak <= 10_000);
    }

    #[test]
    fn simple_rng_is_deterministic_and_seed_zero_is_not_special() {
        let mut rng_a = SimpleRng::new(123);
        let mut rng_b = SimpleRng::new(123);

        for _ in 0..100 {
            assert_eq!(rng_a.next(), rng_b.next());
        }

        let mut rng_zero = SimpleRng::new(0);
        let mut rng_one = SimpleRng::new(1);
        for _ in 0..10 {
            assert_eq!(rng_zero.next(), rng_one.next());
        }

        let mut rng = SimpleRng::new(999);
        for _ in 0..100 {
            let v = rng.next_f32();
            assert!((0.0..=1.0).contains(&v));
        }
    }

    #[test]
    fn formant_filter_produces_different_output_for_different_vowels() {
        let mut state_ah = [0.0f32; 4];
        let mut state_ee = [0.0f32; 4];

        let mut out_ah = Vec::new();
        let mut out_ee = Vec::new();

        for _ in 0..32 {
            out_ah.push(formant_filter(1.0, 0.0, &mut state_ah));
            out_ee.push(formant_filter(1.0, 1.0, &mut state_ee));
        }

        assert!(out_ah.iter().all(|v| v.is_finite()));
        assert!(out_ee.iter().all(|v| v.is_finite()));
        assert!(
            out_ah
                .iter()
                .zip(&out_ee)
                .any(|(a, b)| (a - b).abs() > 1e-6),
            "expected different output sequences"
        );
    }

    #[test]
    fn write_wav_writes_riff_wave_header_and_pcm_samples() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.wav");

        let samples: [i16; 5] = [0, 1, -1, i16::MAX, i16::MIN];
        write_wav(&path, &samples);

        let bytes = fs::read(&path).unwrap();
        assert!(bytes.len() >= 44);

        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        assert_eq!(&bytes[12..16], b"fmt ");
        assert_eq!(&bytes[36..40], b"data");

        let data_size = u32::from_le_bytes([bytes[40], bytes[41], bytes[42], bytes[43]]);
        assert_eq!(data_size as usize, samples.len() * 2);

        let riff_size = u32::from_le_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        assert_eq!(riff_size as usize, 36 + samples.len() * 2);

        let data = &bytes[44..];
        for (i, sample) in samples.iter().enumerate() {
            let start = i * 2;
            let got = i16::from_le_bytes([data[start], data[start + 1]]);
            assert_eq!(got, *sample);
        }
    }
}
