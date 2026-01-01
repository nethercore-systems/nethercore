//! Simple cpal-based audio playback for tracker debugging
//!
//! Uses a callback that pulls samples from a shared player.

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::{Arc, Mutex};

use crate::player::DebugPlayer;

/// Audio player that renders from a shared DebugPlayer
pub struct AudioPlayer {
    _stream: cpal::Stream,
    #[allow(dead_code)] // Stored for potential future use
    sample_rate: u32,
}

impl AudioPlayer {
    /// Create a new audio player with a shared player
    pub fn new(player: Arc<Mutex<DebugPlayer>>) -> Result<Self> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .context("No audio output device available")?;

        let config = device
            .default_output_config()
            .context("Failed to get default output config")?;

        let sample_rate = config.sample_rate().0;

        // Clone for the closure
        let player_clone = Arc::clone(&player);
        let rate = sample_rate;

        // Build the stream
        let stream = match config.sample_format() {
            cpal::SampleFormat::F32 => {
                let config: cpal::StreamConfig = config.into();
                let channels = config.channels as usize;
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                            let mut player = player_clone.lock().unwrap();
                            let mut i = 0;
                            while i < data.len() {
                                let (left, right) = player.render_sample(rate);
                                data[i] = left;
                                if channels > 1 && i + 1 < data.len() {
                                    data[i + 1] = right;
                                }
                                i += channels;
                            }
                        },
                        |_err| {},
                        None,
                    )
                    .context("Failed to build F32 audio stream")?
            }
            cpal::SampleFormat::I16 => {
                let config: cpal::StreamConfig = config.into();
                let channels = config.channels as usize;
                device
                    .build_output_stream(
                        &config,
                        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                            let mut player = player_clone.lock().unwrap();
                            let mut i = 0;
                            while i < data.len() {
                                let (left, right) = player.render_sample(rate);
                                data[i] = (left * 32767.0).clamp(-32768.0, 32767.0) as i16;
                                if channels > 1 && i + 1 < data.len() {
                                    data[i + 1] = (right * 32767.0).clamp(-32768.0, 32767.0) as i16;
                                }
                                i += channels;
                            }
                        },
                        |_err| {},
                        None,
                    )
                    .context("Failed to build I16 audio stream")?
            }
            format => {
                anyhow::bail!("Unsupported sample format: {:?}", format);
            }
        };

        stream.play().context("Failed to start audio stream")?;

        Ok(Self {
            _stream: stream,
            sample_rate,
        })
    }
}
