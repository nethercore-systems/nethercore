//! IT resonant low-pass filter implementation

use super::TrackerChannel;

impl TrackerChannel {
    pub(crate) fn reset_filter_envelope(
        &mut self,
        envelope: Option<&nether_tracker::TrackerEnvelope>,
    ) {
        let envelope = envelope.filter(|env| env.is_filter());
        self.filter_envelope_enabled = envelope.is_some_and(|env| env.is_enabled());
        self.filter_envelope_pos = 0;
        self.filter_new_note = true;
        self.filter_envelope_value = None;
        self.envelope_started &= !8;
        self.filter_dirty = true;
        self.filter_envelope_sustain_loop = envelope.and_then(|env| env.sustain_ticks());
        self.filter_envelope_loop = envelope.filter(|env| env.has_loop()).and_then(|env| {
            Some((
                env.points.get(env.loop_begin as usize)?.0,
                env.points.get(env.loop_end as usize)?.0,
            ))
        });
    }

    fn effective_filter_cutoff(&self) -> f32 {
        // OpenMPT Snd_flt.cpp: cutoff * (envModifier + 256) / 512;
        // IT's signed -32..32 envelope is scaled by eight into envModifier.
        self.filter_cutoff
            * self
                .filter_envelope_value
                .map_or(1.0, |value| (value.clamp(-32, 32) as f32 + 32.0) / 64.0)
    }

    /// Load authored defaults; absent values preserve the current filter setting.
    pub(crate) fn apply_instrument_filter_defaults(
        &mut self,
        instrument: &nether_tracker::TrackerInstrument,
    ) {
        if let Some(cutoff) = instrument.filter_cutoff {
            self.filter_cutoff = cutoff as f32 / 127.0;
            self.filter_dirty = true;
        }
        if let Some(resonance) = instrument.filter_resonance {
            self.filter_resonance = resonance as f32 / 127.0;
            self.filter_dirty = true;
        }
    }

    /// Apply resonant low-pass filter to sample (IT only)
    ///
    /// Uses the original IT two-pole recurrence with two output-history samples.
    pub fn apply_filter(&mut self, input: f32, sample_rate: u32) -> f32 {
        // Original IT only disables full cutoff on a note trigger. Otherwise
        // it retains the previous coefficients, not a freshly opened filter.
        let full = self.effective_filter_cutoff() >= 1.0 && self.filter_resonance == 0.0;
        if full && self.filter_new_note {
            self.filter_active = false;
        }
        self.filter_new_note = false;
        if full && !self.filter_active {
            return input;
        }
        if !full && !self.filter_active {
            self.filter_z1 = 0.0;
            self.filter_z2 = 0.0;
            self.filter_active = true;
            self.filter_dirty = true;
        }

        // Full cutoff without a note deliberately retains the old coefficients.
        if !full && (self.filter_dirty || self.filter_sample_rate != sample_rate) {
            self.update_filter_coefficients(sample_rate as f32);
            self.filter_sample_rate = sample_rate;
            self.filter_dirty = false;
        }

        // Keep output history, not coefficient-weighted state: IT permits
        // cutoff/resonance changes while the voice continues.
        let output = self.filter_b0 * input
            - self.filter_a1 * self.filter_z1
            - self.filter_a2 * self.filter_z2;
        self.filter_z2 = self.filter_z1;
        self.filter_z1 = output;
        output
    }

    /// Recalculate filter coefficients from cutoff and resonance (IT only)
    ///
    /// IT formula: freq = 110 * 2^(cutoff/24 + 0.25)
    /// where cutoff is normalized 0.0-1.0 (from IT's 0-127 range)
    pub fn update_filter_coefficients(&mut self, sample_rate: f32) {
        // Convert normalized cutoff (0.0-1.0) to frequency
        // IT uses: freq = 110 * 2^((cutoff * 127)/24 + 0.25)
        let cutoff_it = self.effective_filter_cutoff() * 127.0;
        let freq = 110.0 * 2.0_f32.powf(cutoff_it / 24.0 + 0.25);

        // Original IT two-pole response (OpenMPT Snd_flt.cpp, normal range).
        let freq = freq.clamp(120.0, 20000.0).min(sample_rate * 0.5);
        let resonance = self.filter_resonance.clamp(0.0, 1.0) * 127.0;
        let damping = 10.0_f32.powf(-resonance * (24.0 / 128.0 / 20.0));
        let r = sample_rate / (2.0 * std::f32::consts::PI * freq);
        let d = damping * r + damping - 1.0;
        let e = r * r;
        let denominator = 1.0 + d + e;
        self.filter_b0 = 1.0 / denominator;
        self.filter_b1 = 0.0;
        self.filter_b2 = 0.0;
        self.filter_a1 = -(d + 2.0 * e) / denominator;
        self.filter_a2 = e / denominator;
    }
}

#[cfg(test)]
mod tests {
    use super::TrackerChannel;

    #[test]
    fn it_full_cutoff_retains_until_note_and_reactivation_clears_history() {
        let mut ch = TrackerChannel::default();
        ch.reset();
        ch.filter_cutoff = 0.0;
        ch.filter_dirty = true;
        ch.apply_filter(1.0, 44100);
        let coefficients = (ch.filter_b0, ch.filter_a1, ch.filter_a2);
        assert!(ch.filter_active);
        ch.filter_cutoff = 1.0;
        ch.filter_dirty = true;
        let mut control = ch.clone();
        control.filter_cutoff = 0.0;
        assert_eq!(
            ch.apply_filter(0.0, 44100),
            control.apply_filter(0.0, 44100)
        );
        assert_eq!((ch.filter_b0, ch.filter_a1, ch.filter_a2), coefficients);
        ch.reset_filter_envelope(None); // actual note, not just a cutoff command
        assert_eq!(ch.apply_filter(0.25, 44100), 0.25);
        assert!(!ch.filter_active && !ch.filter_new_note);
        ch.filter_z1 = 123.0;
        ch.filter_z2 = -456.0;
        ch.filter_cutoff = 0.0;
        ch.filter_dirty = true;
        let mut fresh = TrackerChannel::default();
        fresh.reset();
        fresh.filter_cutoff = 0.0;
        fresh.filter_dirty = true;
        assert_eq!(ch.apply_filter(1.0, 44100), fresh.apply_filter(1.0, 44100));
    }

    #[test]
    fn it_two_pole_reference_impulses_and_live_coefficient_change() {
        // Double-precision evaluation of pinned OpenMPT Snd_flt.cpp normal IT branch.
        for (rate, cutoff, resonance, expected) in [
            (
                44100,
                64,
                64,
                [0.013554184, 0.026488554, 0.038647739, 0.049_891_87],
            ),
            (
                22050,
                64,
                64,
                [0.052_182_47, 0.098_068_05, 0.135_693_58, 0.163_661_39],
            ),
            (
                48000,
                127,
                120,
                [0.414_988_93, 0.625_593_1, 0.560_257_55, 0.267_485_95],
            ),
        ] {
            let mut ch = TrackerChannel::default();
            ch.reset();
            ch.filter_cutoff = cutoff as f32 / 127.0;
            ch.filter_resonance = resonance as f32 / 127.0;
            ch.filter_dirty = true;
            for (i, expected) in expected.into_iter().enumerate() {
                let actual = ch.apply_filter(if i == 0 { 1.0 } else { 0.0 }, rate);
                assert!(
                    (actual - expected).abs() < 0.000002,
                    "rate={rate}: {actual} != {expected}"
                );
            }
            if rate == 44100 {
                ch.filter_cutoff = 20.0 / 127.0;
                ch.filter_resonance = 0.0;
                ch.filter_dirty = true;
                let expected = 0.060_709_804;
                assert!((ch.apply_filter(0.0, rate) - expected).abs() < 0.000002);
            }
        }
    }
}
