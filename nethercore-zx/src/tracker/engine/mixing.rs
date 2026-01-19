//! Channel mixing - core audio rendering logic

use super::super::TrackerEngine;
use super::super::utils::{SINE_LUT, apply_channel_pan, sample_channel};
use super::{CHANNEL_VOLUME_MAX, PAN_ENVELOPE_CENTER, PAN_NOTE_RANGE, VOLUME_ENVELOPE_MAX};
use crate::audio::Sound;

impl TrackerEngine {
    /// Mix all active channels into a stereo sample.
    ///
    /// This is the core mixing logic shared by `render_sample` and
    /// `render_sample_and_advance`. Extracts the common ~100 lines of
    /// channel processing, envelope handling, and panning.
    ///
    /// Takes `raw_handle` instead of a module reference to avoid borrow conflicts.
    ///
    /// # NNA Background Channels
    ///
    /// When IT modules use NNA settings other than Cut, notes may be moved to
    /// background channels (indices >= num_channels) to continue playing.
    /// This method mixes both regular channels and background channels.
    ///
    /// # IT-specific features
    ///
    /// - **Mix Volume**: Master output scaling (0-128, applied at the end)
    /// - **Panning Separation**: Stereo width control (0=mono, 128=full stereo)
    /// - **Surround Mode**: Phase inversion on one channel for S91 effect
    pub(super) fn mix_channels(
        &mut self,
        raw_handle: u32,
        sounds: &[Option<Sound>],
        sample_rate: u32,
    ) -> (f32, f32) {
        let (num_channels, mix_volume, panning_separation) = self
            .modules
            .get(raw_handle as usize)
            .and_then(|m| m.as_ref())
            .map(|m| {
                (
                    m.module.num_channels as usize,
                    m.module.mix_volume,
                    m.module.panning_separation,
                )
            })
            .unwrap_or((0, 128, 128));

        let mut left = 0.0f32;
        let mut right = 0.0f32;

        // Mix all channels - both regular (0..num_channels) and background (num_channels..MAX)
        // Background channels are used by NNA to continue playing displaced notes
        for (ch_idx, channel) in self.channels.iter_mut().enumerate() {
            // Skip inactive channels in the regular range
            if ch_idx < num_channels {
                if !channel.note_on || channel.sample_handle == 0 {
                    continue;
                }
            } else {
                // For background channels, also check if it's actually playing
                // Background channels that have faded out are cleaned up here
                if !channel.note_on || channel.sample_handle == 0 || channel.volume_fadeout == 0 {
                    // Clean up dead background channels
                    if channel.is_background && channel.volume_fadeout == 0 {
                        channel.note_on = false;
                        channel.is_background = false;
                    }
                    continue;
                }
            }

            let sound = match sounds
                .get(channel.sample_handle as usize)
                .and_then(|s| s.as_ref())
            {
                Some(s) => s,
                None => continue,
            };

            // Get instrument reference for envelope processing (scoped to avoid borrow conflicts)
            let instr_idx = channel.instrument.saturating_sub(1) as usize;

            // Apply pitch envelope (IT only)
            if channel.pitch_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                && let Some(instr) = loaded.module.instruments.get(instr_idx)
                && let Some(ref env) = instr.pitch_envelope
                && env.is_enabled()
                && !env.is_filter()
            {
                let env_val = env.value_at(channel.pitch_envelope_pos) as f32;
                channel.pitch_envelope_value = env_val;
            }

            // Update filter envelope (IT only)
            if channel.filter_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                && let Some(instr) = loaded.module.instruments.get(instr_idx)
                && let Some(ref env) = instr.pitch_envelope
                && env.is_filter()
            {
                let env_val = env.value_at(channel.filter_envelope_pos) as f32;
                channel.filter_cutoff = (env_val / VOLUME_ENVELOPE_MAX).clamp(0.0, 1.0);
                channel.filter_dirty = true;
            }

            // Sample with interpolation
            let mut sample = sample_channel(channel, &sound.data, sample_rate);

            // Apply resonant low-pass filter (IT only)
            if channel.filter_cutoff < 1.0 {
                sample = channel.apply_filter(sample);
            }

            // Apply volume with envelope processing
            let mut vol = channel.volume;

            if channel.volume_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                && let Some(instr) = loaded.module.instruments.get(instr_idx)
                && let Some(ref env) = instr.volume_envelope
                && env.is_enabled()
            {
                let env_val =
                    env.value_at(channel.volume_envelope_pos) as f32 / VOLUME_ENVELOPE_MAX;
                vol *= env_val;
            }

            if channel.key_off {
                use super::VOLUME_FADEOUT_MAX;
                vol *= channel.volume_fadeout as f32 / VOLUME_FADEOUT_MAX;
            }

            vol *= self.global_volume;
            vol *= channel.channel_volume as f32 / CHANNEL_VOLUME_MAX;
            vol *= channel.instrument_global_volume as f32 / CHANNEL_VOLUME_MAX;

            if channel.tremor_mute {
                vol = 0.0;
            }

            // Apply panning with envelope
            let mut pan = channel.panning;

            if channel.pitch_pan_separation != 0 {
                let note_offset = channel.current_note as i16 - channel.pitch_pan_center as i16;
                let pan_offset =
                    (note_offset * channel.pitch_pan_separation as i16) as f32 / PAN_NOTE_RANGE;
                pan = (pan + pan_offset).clamp(-1.0, 1.0);
            }

            if channel.panning_envelope_enabled
                && let Some(loaded) = self
                    .modules
                    .get(raw_handle as usize)
                    .and_then(|m| m.as_ref())
                && let Some(instr) = loaded.module.instruments.get(instr_idx)
                && let Some(ref env) = instr.panning_envelope
                && env.is_enabled()
            {
                let env_val = env.value_at(channel.panning_envelope_pos) as f32;
                pan = (env_val - PAN_ENVELOPE_CENTER) / PAN_ENVELOPE_CENTER;
            }

            if channel.panbrello_active && channel.panbrello_depth > 0 {
                let waveform_value = SINE_LUT[(channel.panbrello_pos >> 4) as usize & 0xF] as f32;
                let panbrello_offset = (waveform_value * channel.panbrello_depth as f32)
                    / (CHANNEL_VOLUME_MAX * PAN_NOTE_RANGE);
                pan = (pan + panbrello_offset).clamp(-1.0, 1.0);
            }

            // Apply panning separation (IT feature)
            // 128 = full stereo, 0 = mono
            // This reduces the stereo width by moving panning toward center
            if panning_separation < 128 {
                let sep_factor = panning_separation as f32 / 128.0;
                pan *= sep_factor;
            }

            let (l, r) = apply_channel_pan(sample * vol, pan);

            // Apply surround mode (IT S91 effect)
            // Inverts phase on right channel for "surround" psychoacoustic effect
            if channel.surround {
                left += l;
                right -= r; // Invert phase on right channel
            } else {
                left += l;
                right += r;
            }
        }

        // Apply mix volume (IT master volume, 0-128)
        let mix_scale = mix_volume as f32 / 128.0;
        (left * mix_scale, right * mix_scale)
    }
}
