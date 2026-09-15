//! Audio output using cpal and ring buffer

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use ringbuf::{
    HeapRb,
    traits::{Consumer, Observer, Producer, Split},
};
use tracing::{debug, error};

/// Audio sample rate for output (44.1 kHz - native for most hardware)
pub const OUTPUT_SAMPLE_RATE: u32 = 44_100;

/// Audio sample rate for source sounds (22.05 kHz - PS1/N64 authentic)
pub const SOURCE_SAMPLE_RATE: u32 = 22_050;

/// Ring buffer size in samples (stereo frames * 2 channels)
/// ~100ms buffer at 44.1kHz = 4410 frames * 2 channels = 8820 samples
/// This provides ~6 frames of headroom at 60fps - enough for minor jitter.
const RING_BUFFER_SIZE: usize = 8820; // ~100ms buffer

/// Queue complete device frames: mono downmix, front stereo, other channels silent.
fn enqueue_stereo(
    producer: &mut ringbuf::HeapProd<f32>,
    samples: &[f32],
    channels: usize,
) -> usize {
    let mut frame = [0.0; 8];
    let mut consumed = 0;
    for stereo in samples.chunks_exact(2) {
        if producer.vacant_len() < channels {
            break;
        }
        frame[0] = if channels == 1 {
            (stereo[0] + stereo[1]) * 0.5
        } else {
            stereo[0]
        };
        if channels > 1 {
            frame[1] = stereo[1];
        }
        producer.push_slice(&frame[..channels]);
        consumed += 2;
    }
    consumed
}

pub struct AudioOutput {
    /// Producer side of the ring buffer (main thread writes here)
    producer: ringbuf::HeapProd<f32>,
    channels: usize,
    /// The cpal stream (kept alive for the duration)
    _stream: cpal::Stream,
    /// Output sample rate
    sample_rate: u32,
}

impl AudioOutput {
    /// Create a new audio output
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| "No audio output device available".to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let sample_rate = config.sample_rate().0;
        let channels = config.channels() as usize;
        if !(1..=8).contains(&channels) {
            return Err(format!(
                "Unsupported audio output channel count: {channels}"
            ));
        }

        // Create ring buffer
        let ring = HeapRb::<f32>::new(RING_BUFFER_SIZE / 2 * channels);
        let (producer, mut consumer) = ring.split();

        // Build the stream based on sample format
        // NOTE: Using batch pop_slice() instead of per-sample try_pop() to prevent timing gaps
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let config = config.into();
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            // Batch read all available samples (much more efficient than per-sample)
                            let popped = consumer.pop_slice(data);
                            // Fill any remaining samples with silence
                            data[popped..].fill(0.0);
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::I16 => {
                let config = config.into();
                // Pre-allocate buffer for batch reads
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            // Resize temp buffer if needed
                            if temp_buffer.len() < data.len() {
                                temp_buffer.resize(data.len(), 0.0);
                            }
                            // Batch read f32 samples
                            let popped = consumer.pop_slice(&mut temp_buffer[..data.len()]);
                            // Convert popped samples to i16
                            for (i, &f) in temp_buffer[..popped].iter().enumerate() {
                                data[i] = (f * 32767.0).clamp(-32768.0, 32767.0) as i16;
                            }
                            // Fill remaining with silence
                            for sample in &mut data[popped..] {
                                *sample = 0;
                            }
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::U16 => {
                let config = config.into();
                // Pre-allocate buffer for batch reads
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                            // Resize temp buffer if needed
                            if temp_buffer.len() < data.len() {
                                temp_buffer.resize(data.len(), 0.0);
                            }
                            // Batch read f32 samples
                            let popped = consumer.pop_slice(&mut temp_buffer[..data.len()]);
                            // Convert popped samples to u16
                            for (i, &f) in temp_buffer[..popped].iter().enumerate() {
                                data[i] = ((f * 32767.0 + 32768.0).clamp(0.0, 65535.0)) as u16;
                            }
                            // Fill remaining with silence (0x8000 is silence for u16 audio)
                            for sample in &mut data[popped..] {
                                *sample = 32768;
                            }
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            _ => {
                return Err(format!(
                    "Unsupported sample format: {:?}",
                    config.sample_format()
                ));
            }
        };

        stream
            .play()
            .map_err(|e| format!("Failed to play audio stream: {}", e))?;

        debug!("Audio stream started");

        Ok(Self {
            producer,
            channels,
            _stream: stream,
            sample_rate,
        })
    }

    /// Push audio samples to the ring buffer
    ///
    /// Samples should be interleaved stereo (left, right, left, right, ...)
    pub fn push_samples(&mut self, samples: &[f32]) {
        // Push as many samples as we can fit
        let pushed = enqueue_stereo(&mut self.producer, samples, self.channels);
        if pushed < samples.len() {
            // Ring buffer overflow - this can happen if game is running slow
            // Just drop the extra samples (audio will slightly desync but recover)
            debug!(
                "Audio buffer overflow: dropped {} samples",
                samples.len() - pushed
            );
        }
    }

    /// Get the output sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shipping_audio_layout_adapter_keeps_complete_frames() {
        for channels in [1, 2, 4, 6, 8] {
            let rb = HeapRb::<f32>::new(channels * 2);
            let (mut producer, mut consumer) = rb.split();
            assert_eq!(
                enqueue_stereo(&mut producer, &[-1.0, 1.0, 0.25, 0.5], channels),
                4
            );
            assert_eq!(enqueue_stereo(&mut producer, &[1.0, 1.0], channels), 0);
            let mut output = vec![0.0; channels * 2];
            assert_eq!(consumer.pop_slice(&mut output), output.len());
            if channels == 1 {
                assert_eq!(output, [0.0, 0.375]);
            } else {
                assert_eq!(&output[..2], &[-1.0, 1.0]);
                assert_eq!(&output[channels..channels + 2], &[0.25, 0.5]);
                for frame in output.chunks_exact(channels) {
                    assert!(frame[2..].iter().all(|&v| v == 0.0));
                }
            }
            assert_eq!(consumer.pop_slice(&mut output), 0);
        }
    }
}
