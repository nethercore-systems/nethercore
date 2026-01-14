//! Threaded audio output combining ring buffer and generation thread
//!
//! This is the public interface for threaded audio generation.

use tracing::{debug, error};

use super::handle::AudioGenHandle;
use super::snapshot::AudioGenSnapshot;
use super::thread::AudioGenThread;

/// Threaded audio output combining ring buffer and generation thread
///
/// Drop-in replacement for `AudioOutput` that uses threaded generation.
pub struct ThreadedAudioOutput {
    /// Handle to the generation thread
    gen_handle: AudioGenHandle,

    /// The cpal stream (kept alive for the duration)
    _stream: cpal::Stream,

    /// Output sample rate
    sample_rate: u32,
}

impl ThreadedAudioOutput {
    /// Create a new threaded audio output
    ///
    /// This spawns the audio generation thread and sets up the cpal output stream.
    pub fn new() -> Result<Self, String> {
        use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
        use ringbuf::HeapRb;
        use ringbuf::traits::Split;

        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or_else(|| "No audio output device available".to_string())?;

        let config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default output config: {}", e))?;

        let sample_rate = config.sample_rate().0;

        // Create ring buffer - larger than non-threaded version for more headroom
        // ~150ms buffer at 44.1kHz = 6615 frames * 2 channels = 13230 samples
        const RING_BUFFER_SIZE: usize = 13230;
        let ring = HeapRb::<f32>::new(RING_BUFFER_SIZE);
        let (producer, mut consumer) = ring.split();

        // Spawn the generation thread FIRST to get the condvar for callbacks
        let gen_handle = AudioGenThread::spawn(producer, sample_rate);
        let condvar_f32 = gen_handle.condvar.clone();
        let condvar_i16 = gen_handle.condvar.clone();
        let condvar_u16 = gen_handle.condvar.clone();

        // Build the stream based on sample format
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let config = config.into();
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            // Batch read all available samples at once (much more efficient
                            // than per-sample try_pop which causes timing gaps and popping)
                            let popped = consumer.pop_slice(data);
                            // Fill any remaining samples with silence
                            data[popped..].fill(0.0);

                            // Signal audio generation thread that buffer space is available
                            // notify_one() doesn't require holding the lock
                            let (_lock, cvar) = &*condvar_f32;
                            cvar.notify_one();
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::I16 => {
                let config = config.into();
                // Pre-allocate buffer for batch reads (avoids per-sample atomic ops)
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            // Resize temp buffer if needed (rare, only on first use or format change)
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

                            // Signal audio generation thread that buffer space is available
                            // notify_one() doesn't require holding the lock
                            let (_lock, cvar) = &*condvar_i16;
                            cvar.notify_one();
                        },
                        |err| error!("Audio stream error: {}", err),
                        None,
                    )
                    .map_err(|e| format!("Failed to build audio stream: {}", e))?
            }
            cpal::SampleFormat::U16 => {
                let config = config.into();
                // Pre-allocate buffer for batch reads (avoids per-sample atomic ops)
                let mut temp_buffer: Vec<f32> = vec![0.0; 4096];
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [u16], _: &cpal::OutputCallbackInfo| {
                            use ringbuf::traits::Consumer;
                            // Resize temp buffer if needed (rare, only on first use or format change)
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

                            // Signal audio generation thread that buffer space is available
                            // notify_one() doesn't require holding the lock
                            let (_lock, cvar) = &*condvar_u16;
                            cvar.notify_one();
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

        debug!("Threaded audio stream started at {}Hz", sample_rate);

        Ok(Self {
            gen_handle,
            _stream: stream,
            sample_rate,
        })
    }

    /// Send an audio snapshot to the generation thread
    ///
    /// Returns true if the snapshot was queued, false if dropped.
    pub fn send_snapshot(&self, snapshot: AudioGenSnapshot) -> bool {
        self.gen_handle.send_snapshot(snapshot)
    }

    /// Get the output sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Check if the audio thread is still running
    pub fn is_alive(&self) -> bool {
        self.gen_handle.is_alive()
    }
}
