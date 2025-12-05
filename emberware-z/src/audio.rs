//! Emberware Z audio backend
//!
//! PS1/N64-style audio system with:
//! - 22,050 Hz sample rate (authentic retro)
//! - 16-bit signed PCM mono format
//! - 16 managed channels for sound effects
//! - Dedicated music channel
//! - Rollback-aware audio command buffering
//!
//! Thread-safe architecture:
//! - Audio server runs in background thread, owns rodio OutputStream/Sinks
//! - Main thread sends commands via mpsc channel
//! - Rollback-aware: commands discarded during replay

use rodio::{OutputStream, Sink, Source};
use std::sync::{mpsc, Arc};
use std::thread;
use tracing::{error, trace, warn};

/// Maximum number of sound effect channels
pub const MAX_CHANNELS: usize = 16;

/// Audio sample rate (22.05 kHz - PS1/N64 authentic)
pub const SAMPLE_RATE: u32 = 22_050;

/// Sound data (raw PCM)
#[derive(Clone, Debug)]
pub struct Sound {
    /// Raw PCM data (16-bit signed, mono, 22.05kHz)
    pub data: Arc<Vec<i16>>,
}

impl Sound {
    /// Create a rodio source from this sound
    fn to_source(&self) -> SoundSource {
        SoundSource {
            data: self.data.clone(),
            position: 0,
        }
    }
}

/// Rodio source for playback
struct SoundSource {
    data: Arc<Vec<i16>>,
    position: usize,
}

impl Iterator for SoundSource {
    type Item = i16;

    fn next(&mut self) -> Option<Self::Item> {
        if self.position >= self.data.len() {
            None
        } else {
            let sample = self.data[self.position];
            self.position += 1;
            Some(sample)
        }
    }
}

impl Source for SoundSource {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.data.len() - self.position)
    }

    fn channels(&self) -> u16 {
        1 // Mono
    }

    fn sample_rate(&self) -> u32 {
        SAMPLE_RATE
    }

    fn total_duration(&self) -> Option<std::time::Duration> {
        let samples = self.data.len() as u32;
        let seconds = samples as f64 / SAMPLE_RATE as f64;
        Some(std::time::Duration::from_secs_f64(seconds))
    }
}

/// Audio commands buffered per frame
#[derive(Debug, Clone)]
pub enum AudioCommand {
    /// Play sound on next available channel
    PlaySound { sound: u32, volume: f32, pan: f32 },
    /// Play sound on specific channel
    ChannelPlay {
        channel: u32,
        sound: u32,
        volume: f32,
        pan: f32,
        looping: bool,
    },
    /// Update channel parameters
    ChannelSet { channel: u32, volume: f32, pan: f32 },
    /// Stop channel
    ChannelStop { channel: u32 },
    /// Play music (looping)
    MusicPlay { sound: u32, volume: f32 },
    /// Stop music
    MusicStop,
    /// Set music volume
    MusicSetVolume { volume: f32 },
}

/// Internal command sent to audio server thread
enum ServerCommand {
    /// Process audio commands with sound library
    ProcessCommands {
        commands: Vec<AudioCommand>,
        sounds: Vec<Option<Sound>>,
    },
    /// Shutdown audio server
    Shutdown,
}

/// Channel state tracking
struct ChannelState {
    sink: Sink,
    current_sound: Option<u32>,
    looping: bool,
}

/// Audio server running in background thread
struct AudioServer {
    _stream: OutputStream,
    channels: Vec<ChannelState>,
    music_sink: Sink,
    current_music: Option<u32>,
}

impl AudioServer {
    /// Create new audio server (runs in background thread)
    fn new() -> Result<Self, String> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| format!("Failed to create audio output stream: {}", e))?;

        // Create 16 channel sinks
        let mut channels = Vec::with_capacity(MAX_CHANNELS);
        for _ in 0..MAX_CHANNELS {
            let sink = Sink::try_new(&stream_handle)
                .map_err(|e| format!("Failed to create audio sink: {}", e))?;
            channels.push(ChannelState {
                sink,
                current_sound: None,
                looping: false,
            });
        }

        // Create dedicated music sink
        let music_sink = Sink::try_new(&stream_handle)
            .map_err(|e| format!("Failed to create music sink: {}", e))?;

        Ok(Self {
            _stream: stream,
            channels,
            music_sink,
            current_music: None,
        })
    }

    /// Main server loop
    fn run(mut self, rx: mpsc::Receiver<ServerCommand>) {
        while let Ok(cmd) = rx.recv() {
            match cmd {
                ServerCommand::ProcessCommands { commands, sounds } => {
                    self.process_commands(&commands, &sounds);
                }
                ServerCommand::Shutdown => {
                    trace!("Audio server shutting down");
                    break;
                }
            }
        }
    }

    /// Process buffered audio commands
    fn process_commands(&mut self, commands: &[AudioCommand], sounds: &[Option<Sound>]) {
        for cmd in commands {
            match cmd {
                AudioCommand::PlaySound { sound, volume, pan } => {
                    self.play_sound(*sound, *volume, *pan, sounds);
                }
                AudioCommand::ChannelPlay {
                    channel,
                    sound,
                    volume,
                    pan,
                    looping,
                } => {
                    self.channel_play(*channel, *sound, *volume, *pan, *looping, sounds);
                }
                AudioCommand::ChannelSet {
                    channel,
                    volume,
                    pan,
                } => {
                    self.channel_set(*channel, *volume, *pan);
                }
                AudioCommand::ChannelStop { channel } => {
                    self.channel_stop(*channel);
                }
                AudioCommand::MusicPlay { sound, volume } => {
                    self.music_play(*sound, *volume, sounds);
                }
                AudioCommand::MusicStop => {
                    self.music_stop();
                }
                AudioCommand::MusicSetVolume { volume } => {
                    self.music_set_volume(*volume);
                }
            }
        }
    }

    /// Play sound on next available channel
    fn play_sound(&mut self, sound: u32, volume: f32, pan: f32, sounds: &[Option<Sound>]) {
        let sound_idx = sound as usize;
        if sound_idx >= sounds.len() {
            warn!("play_sound: invalid sound handle {}", sound);
            return;
        }

        let Some(sound_data) = &sounds[sound_idx] else {
            warn!("play_sound: sound {} not loaded", sound);
            return;
        };

        // Find first free channel
        for channel in &mut self.channels {
            if channel.sink.empty() {
                channel.sink.set_volume(volume.clamp(0.0, 1.0));
                // TODO: Implement panning (rodio doesn't have built-in pan control)
                // For now, just play at center
                let _ = pan; // Silence unused warning

                channel.sink.append(sound_data.to_source());
                channel.current_sound = Some(sound);
                channel.looping = false;
                return;
            }
        }

        warn!("play_sound: all channels busy");
    }

    /// Play sound on specific channel
    fn channel_play(
        &mut self,
        channel: u32,
        sound: u32,
        volume: f32,
        pan: f32,
        looping: bool,
        sounds: &[Option<Sound>],
    ) {
        let channel_idx = channel as usize;
        if channel_idx >= MAX_CHANNELS {
            warn!("channel_play: invalid channel {}", channel);
            return;
        }

        let sound_idx = sound as usize;
        if sound_idx >= sounds.len() {
            warn!("channel_play: invalid sound handle {}", sound);
            return;
        }

        let Some(sound_data) = &sounds[sound_idx] else {
            warn!("channel_play: sound {} not loaded", sound);
            return;
        };

        let ch = &mut self.channels[channel_idx];

        // If same sound already playing, just update params (rollback-friendly)
        if ch.current_sound == Some(sound) && !ch.sink.empty() {
            ch.sink.set_volume(volume.clamp(0.0, 1.0));
            // TODO: Update pan
            let _ = pan;
            return;
        }

        // Stop current sound and play new one
        ch.sink.stop();
        ch.sink.set_volume(volume.clamp(0.0, 1.0));

        if looping {
            ch.sink.append(sound_data.to_source().repeat_infinite());
        } else {
            ch.sink.append(sound_data.to_source());
        }

        ch.current_sound = Some(sound);
        ch.looping = looping;
    }

    /// Update channel parameters
    fn channel_set(&mut self, channel: u32, volume: f32, pan: f32) {
        let channel_idx = channel as usize;
        if channel_idx >= MAX_CHANNELS {
            warn!("channel_set: invalid channel {}", channel);
            return;
        }

        let ch = &mut self.channels[channel_idx];
        ch.sink.set_volume(volume.clamp(0.0, 1.0));
        // TODO: Set pan
        let _ = pan;
    }

    /// Stop channel
    fn channel_stop(&mut self, channel: u32) {
        let channel_idx = channel as usize;
        if channel_idx >= MAX_CHANNELS {
            warn!("channel_stop: invalid channel {}", channel);
            return;
        }

        let ch = &mut self.channels[channel_idx];
        ch.sink.stop();
        ch.current_sound = None;
        ch.looping = false;
    }

    /// Play music (looping)
    fn music_play(&mut self, sound: u32, volume: f32, sounds: &[Option<Sound>]) {
        let sound_idx = sound as usize;
        if sound_idx >= sounds.len() {
            warn!("music_play: invalid sound handle {}", sound);
            return;
        }

        let Some(sound_data) = &sounds[sound_idx] else {
            warn!("music_play: sound {} not loaded", sound);
            return;
        };

        // If same music already playing, just update volume
        if self.current_music == Some(sound) && !self.music_sink.empty() {
            self.music_sink.set_volume(volume.clamp(0.0, 1.0));
            return;
        }

        // Stop current music and play new one
        self.music_sink.stop();
        self.music_sink.set_volume(volume.clamp(0.0, 1.0));
        self.music_sink
            .append(sound_data.to_source().repeat_infinite());
        self.current_music = Some(sound);
    }

    /// Stop music
    fn music_stop(&mut self) {
        self.music_sink.stop();
        self.current_music = None;
    }

    /// Set music volume
    fn music_set_volume(&mut self, volume: f32) {
        self.music_sink.set_volume(volume.clamp(0.0, 1.0));
    }
}

/// Emberware Z audio backend
pub struct ZAudio {
    /// Whether audio is in rollback mode (commands discarded)
    rollback_mode: bool,
    /// Channel to send commands to audio server thread
    tx: mpsc::Sender<ServerCommand>,
    /// Audio server thread handle
    _thread: thread::JoinHandle<()>,
}

impl ZAudio {
    /// Create new audio backend
    pub fn new() -> Result<Self, String> {
        let (tx, rx) = mpsc::channel();

        // Spawn audio server thread
        let thread = thread::spawn(move || match AudioServer::new() {
            Ok(server) => {
                trace!("Audio server started");
                server.run(rx);
            }
            Err(e) => {
                error!("Failed to initialize audio server: {}", e);
            }
        });

        Ok(Self {
            rollback_mode: false,
            tx,
            _thread: thread,
        })
    }

    /// Set rollback mode
    pub fn set_rollback_mode(&mut self, rolling_back: bool) {
        self.rollback_mode = rolling_back;
    }

    /// Process buffered audio commands
    pub fn process_commands(&mut self, commands: &[AudioCommand], sounds: &[Option<Sound>]) {
        if self.rollback_mode {
            // Discard playback commands during rollback
            return;
        }

        if commands.is_empty() {
            return;
        }

        // Send commands to audio server thread
        let server_cmd = ServerCommand::ProcessCommands {
            commands: commands.to_vec(),
            sounds: sounds.to_vec(),
        };

        if let Err(e) = self.tx.send(server_cmd) {
            error!("Failed to send commands to audio server: {}", e);
        }
    }
}

impl Drop for ZAudio {
    fn drop(&mut self) {
        // Send shutdown command to audio server
        let _ = self.tx.send(ServerCommand::Shutdown);
    }
}
