//! Channel mixing - core audio rendering logic

use super::super::TrackerEngine;
use super::super::utils::{SINE_LUT, sample_channels};
use super::{CHANNEL_VOLUME_MAX, PAN_ENVELOPE_CENTER, PAN_NOTE_RANGE, VOLUME_ENVELOPE_MAX};
use crate::audio::Sound;

/// Apply the source XM pan law without a lookup table.
///
/// `pan` is the engine's -1..=1 position. FT2 has a source range of 0..=255,
/// while compatible XM keeps the linear 0..=256 endpoint range.
pub(super) fn xm_pan(sample: f32, pan: f32, ft2: bool) -> (f32, f32) {
    let source_pan = ((pan.clamp(-1.0, 1.0) + 1.0) * 128.0).round();
    let max_pan = if ft2 { 255.0 } else { 256.0 };
    let source_pan = source_pan.clamp(0.0, max_pan);
    let (left_share, right_share) = if ft2 {
        ((256.0 - source_pan).sqrt() / 16.0, source_pan.sqrt() / 16.0)
    } else {
        ((256.0 - source_pan) / 256.0, source_pan / 256.0)
    };
    (sample * left_share, sample * right_share)
}

/// XM retains fractional volume nodes; pan rounds in its unsigned source range.
fn xm_envelope_value(env: &nether_tracker::TrackerEnvelope, tick: u16) -> f32 {
    env.points
        .windows(2)
        .find_map(|p| {
            let [(x1, y1), (x2, y2)] = [p[0], p[1]];
            (tick >= x1 && tick < x2).then(|| {
                f32::from(y1) + f32::from(y2 - y1) * f32::from(tick - x1) / f32::from(x2 - x1)
            })
        })
        .unwrap_or_else(|| env.value_at(tick) as f32)
}

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
        tick_samples: u32,
    ) -> (f32, f32) {
        self.process_channels::<true>(raw_handle, sounds, sample_rate, tick_samples)
    }

    /// State transitions shared by audible and silent advancement.
    /// Silent mode retains interpolation/filter history but skips output-only work.
    pub(super) fn process_channels<const OUTPUT: bool>(
        &mut self,
        raw_handle: u32,
        sounds: &[Option<Sound>],
        sample_rate: u32,
        tick_samples: u32,
    ) -> (f32, f32) {
        let (num_channels, mix_volume, panning_separation, is_it, xm_ft2_mix, it_balance_mix) =
            self.modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .map(|m| {
                    (
                        m.module.num_channels as usize,
                        m.module.mix_volume as f32
                            * if !m
                                .module
                                .format
                                .contains(nether_tracker::FormatFlags::IS_IT_FORMAT)
                                && m.module
                                    .format
                                    .contains(nether_tracker::FormatFlags::XM_LEGACY_MIX)
                            {
                                2.0 / 3.0
                            } else {
                                1.0
                            },
                        m.module.panning_separation,
                        m.module
                            .format
                            .contains(nether_tracker::FormatFlags::IS_IT_FORMAT),
                        m.module
                            .format
                            .contains(nether_tracker::FormatFlags::XM_FT2_MIX),
                        m.module
                            .format
                            .contains(nether_tracker::FormatFlags::IT_BALANCE_MIX),
                    )
                })
                .unwrap_or((0, 128.0, 128, false, false, false));

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

            // Recompute so disabled/missing envelopes cannot retain pitch.
            channel.pitch_envelope_value = 0.0;
            // Apply pitch envelope (IT only)
            if let Some(loaded) = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                && let Some(instr) = loaded.module.instruments.get(instr_idx)
                && let Some(ref env) = instr.pitch_envelope
                && (channel.pitch_envelope_enabled
                    || loaded
                        .module
                        .format
                        .contains(nether_tracker::FormatFlags::IS_IT_FORMAT)
                        && env.is_enabled()
                        && channel.envelope_started & 4 != 0)
                && !env.is_filter()
            {
                let env_val = env.value_at(channel.pitch_envelope_pos) as f32;
                channel.pitch_envelope_value = env_val;
            }

            // IT applies the signed envelope to the base cutoff; never overwrite Zxx/defaults.
            let filter_value = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| m.module.instruments.get(instr_idx))
                .and_then(|i| i.pitch_envelope.as_ref())
                .filter(|e| e.is_filter())
                .and_then(|e| {
                    if channel.filter_envelope_enabled || e.is_enabled() {
                        // IT's authored-on S7B at note start still applies midpoint cutoff.
                        Some(
                            if !channel.filter_envelope_enabled && channel.envelope_started & 8 == 0
                            {
                                0
                            } else {
                                e.value_at(channel.filter_envelope_pos)
                            },
                        )
                    } else {
                        None
                    }
                });
            if channel.filter_envelope_value != filter_value {
                channel.filter_envelope_value = filter_value;
                channel.filter_dirty = true;
            }

            let was_stopped = channel.xm_loop_stopped;
            let (mut sample_left, mut sample_right) =
                sample_channels(channel, &sound.data, sample_rate);
            if !channel.sample_is_stereo {
                sample_left = channel.apply_filter(sample_left, sample_rate);
                sample_right = sample_left;
            }

            let xm_envelopes =
                !is_it && (channel.volume_envelope_enabled || channel.panning_envelope_enabled);
            // XM envelope gains carry a within-tick transition. Silent replay
            // must consume it too, including while the sample voice is stopped.
            // A fresh envelope attack (including E9 and delayed instrument
            // selection) uses the sampler's existing restart ramp. Do not
            // interpolate from the previous envelope's terminal gain as well.
            if !xm_envelopes || channel.envelope_started == 0 && !was_stopped {
                channel.xm_envelope_ramp = None;
            }
            // Measured IT Rxy interpolates its volume modulation over the tick.
            // Advance this history on the silent path as well as audible mixing.
            let it_tremolo = if is_it && channel.tremolo_active {
                channel.it_tremolo_ramp_pos = channel.it_tremolo_ramp_pos.saturating_add(1);
                let progress =
                    (channel.it_tremolo_ramp_pos as f32 / tick_samples.max(1) as f32).min(1.0);
                channel.it_tremolo_from
                    + (channel.it_tremolo_delta - channel.it_tremolo_from) * progress
            } else {
                0.0
            };
            if !OUTPUT && !xm_envelopes {
                continue;
            }

            // Apply volume with envelope processing
            let mut vol = (channel.volume
                + if is_it {
                    it_tremolo
                } else if channel.volume == 0.0 {
                    0.0
                } else {
                    channel.xm_tremolo_delta
                })
            .clamp(0.0, 1.0);

            if let Some(loaded) = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                && let Some(instr) = loaded.module.instruments.get(instr_idx)
                && let Some(ref env) = instr.volume_envelope
                && (channel.volume_envelope_enabled
                    || loaded
                        .module
                        .format
                        .contains(nether_tracker::FormatFlags::IS_IT_FORMAT)
                        && env.is_enabled()
                        && channel.envelope_started & 1 != 0)
            {
                let env_val = if is_it {
                    env.value_at(channel.volume_envelope_pos) as f32
                } else {
                    xm_envelope_value(env, channel.volume_envelope_pos)
                } / VOLUME_ENVELOPE_MAX;
                vol *= env_val;
            }

            if channel.key_off || channel.note_fade {
                use super::VOLUME_FADEOUT_MAX;
                vol *= channel.volume_fadeout as f32 / VOLUME_FADEOUT_MAX;
            }

            vol *= self.global_volume;
            vol *= channel.channel_volume as f32 / CHANNEL_VOLUME_MAX;
            let mut instrument_gain = channel.instrument_global_volume as f32 / CHANNEL_VOLUME_MAX;
            if let Some(sample) = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                .and_then(|m| channel.sample_index.and_then(|i| m.module.samples.get(i)))
            {
                instrument_gain *= sample.global_volume.min(64) as f32 / CHANNEL_VOLUME_MAX;
            }

            vol *= if is_it {
                (instrument_gain + channel.volume_swing).clamp(0.0, 1.0)
            } else {
                instrument_gain
            };

            if channel.tremor_mute && (!is_it || channel.tremor_active) {
                vol = 0.0;
            }

            // Apply panning with envelope
            let mut pan =
                (channel.panning + if is_it { channel.pan_swing } else { 0.0 }).clamp(-1.0, 1.0);

            if channel.pitch_pan_separation != 0 {
                let note_offset = channel.current_note as i16 - channel.pitch_pan_center as i16;
                let pan_offset =
                    (note_offset * channel.pitch_pan_separation as i16) as f32 / PAN_NOTE_RANGE;
                pan = (pan + pan_offset).clamp(-1.0, 1.0);
            }

            if let Some(loaded) = self
                .modules
                .get(raw_handle as usize)
                .and_then(|m| m.as_ref())
                && let Some(instr) = loaded.module.instruments.get(instr_idx)
                && let Some(ref env) = instr.panning_envelope
                && (channel.panning_envelope_enabled
                    || loaded
                        .module
                        .format
                        .contains(nether_tracker::FormatFlags::IS_IT_FORMAT)
                        && env.is_enabled()
                        && channel.envelope_started & 2 != 0)
            {
                let env_val = if is_it {
                    env.value_at(channel.panning_envelope_pos) as f32
                } else {
                    // Original half-node controls establish nearest rounding,
                    // with ties upward in XM's authored unsigned 0..64 range.
                    (xm_envelope_value(env, channel.panning_envelope_pos) + 0.5).floor()
                };
                // Signed envelope offsets scale by distance to the nearest edge.
                // Neutral envelopes preserve channel pan; hard pans remain hard.
                pan = (pan + env_val / PAN_ENVELOPE_CENTER * (1.0 - pan.abs())).clamp(-1.0, 1.0);
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

            let (mut l, mut r) = if channel.sample_is_stereo {
                let (left_gain, right_gain) = if is_it {
                    let right_share = (pan.clamp(-1.0, 1.0) + 1.0) * 0.5;
                    if it_balance_mix {
                        (
                            (2.0 * (1.0 - right_share)).min(1.0),
                            (2.0 * right_share).min(1.0),
                        )
                    } else {
                        (1.0 - right_share, right_share)
                    }
                } else {
                    xm_pan(1.0, pan, xm_ft2_mix)
                };
                (
                    sample_left * vol * left_gain,
                    sample_right * vol * right_gain,
                )
            } else if is_it {
                // IT distributes mono amplitude linearly, not by equal power.
                let right_share = (pan.clamp(-1.0, 1.0) + 1.0) * 0.5;
                let amplitude = sample_left * vol;
                if it_balance_mix {
                    // Explicit RC3 balance: unity at center, attenuate the far side.
                    (
                        amplitude * (2.0 * (1.0 - right_share)).min(1.0),
                        amplitude * (2.0 * right_share).min(1.0),
                    )
                } else {
                    (amplitude * (1.0 - right_share), amplitude * right_share)
                }
            } else {
                xm_pan(sample_left * vol, pan, xm_ft2_mix)
            };

            if xm_envelopes && mix_volume > 0.0 {
                use super::super::channels::XmEnvelopeRamp;
                let (lg, rg) = xm_pan(vol, pan, xm_ft2_mix);
                // Independently measured original step controls resolve final
                // lane gain in 1/4096 units. Keep that precision after preamp;
                // do not change the module's gain or envelope-free mixing.
                let quantum = 128.0 / (4096.0 * mix_volume);
                let target = [lg, rg].map(|g| (g / quantum).floor() * quantum);
                let duration = tick_samples.max(1);
                if channel
                    .xm_envelope_ramp
                    .as_ref()
                    .is_some_and(|r| r.stopped && !was_stopped)
                {
                    channel.xm_envelope_ramp = None;
                }
                let ramp = channel.xm_envelope_ramp.get_or_insert(XmEnvelopeRamp {
                    from: target,
                    target,
                    current: target,
                    position: duration,
                    stopped: false,
                });
                if ramp.target != target {
                    ramp.from = ramp.target;
                    ramp.target = target;
                    ramp.position = 0;
                }
                let mut gain = target;
                if ramp.position < duration {
                    ramp.position += 1;
                    for lane in 0..2 {
                        let delta = target[lane] - ramp.from[lane];
                        if delta == 0.0 {
                            continue;
                        }
                        // The measured rising/falling transitions begin one
                        // gain unit toward their target, then traverse a tick.
                        let value = ramp.from[lane]
                            + delta.signum() * quantum
                            + delta * (ramp.position as f32 / duration as f32);
                        gain[lane] = (value / quantum).floor().max(0.0) * quantum;
                    }
                }
                if ramp.position >= duration {
                    // Rate/BPM changes can exhaust the ramp without incrementing it.
                    ramp.from = target;
                }
                // The stop tail decays the last mixed sample. Envelopes keep
                // their clocks for a later retrigger, but cannot repan or
                // rescale this already-stopped voice's residual output.
                if was_stopped {
                    gain = ramp.current;
                }
                ramp.current = gain;
                ramp.stopped = channel.xm_loop_stopped;
                l = sample_left * gain[0];
                r = sample_right * gain[1];
            }
            if !OUTPUT || self.channel_mutes[ch_idx] {
                continue;
            }

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

        // Source preamp is retained at import, independently of the pan law.
        let mix_scale = mix_volume / 128.0;
        (left * mix_scale, right * mix_scale)
    }
}
