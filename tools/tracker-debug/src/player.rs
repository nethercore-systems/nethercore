//! Debug player wrapping TrackerEngine
//!
//! Provides a high-level interface for loading and playing tracker modules
//! with state inspection for the debug display.

use anyhow::{Context, Result};
use nethercore_zx::audio::Sound;
use nethercore_zx::state::{TrackerState, tracker_flags};
use nethercore_zx::tracker::TrackerEngine;
use std::path::Path;

use crate::display::DisplayState;
use crate::sound_loader;

/// Commands that can be sent to the player
#[derive(Debug, Clone, Copy)]
pub enum PlayerCommand {
    TogglePause,
    SeekRow(i32),
    SeekPattern(i32),
    AdjustTempo(i16),
    ToggleMute(u8),
    ToggleVerbose,
}

/// Debug player wrapping TrackerEngine
pub struct DebugPlayer {
    engine: TrackerEngine,
    state: TrackerState,
    sounds: Vec<Option<Sound>>,
    filename: String,
    num_channels: u8,
    total_orders: u16,
    paused: bool,
    verbose: bool,
    channel_mutes: [bool; 64],
}

impl DebugPlayer {
    /// Load an XM file
    pub fn load_xm(path: &Path, via_ncxm: bool) -> Result<Self> {
        let data = std::fs::read(path).context("Failed to read XM file")?;
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse the XM module
        let xm_module = if via_ncxm {
            let original = nether_xm::parse_xm(&data).context("Failed to parse XM")?;
            let packed = nether_xm::pack_xm_minimal(&original).context("Failed to pack NCXM")?;
            nether_xm::parse_xm_minimal(&packed).context("Failed to parse NCXM")?
        } else {
            nether_xm::parse_xm(&data).context("Failed to parse XM")?
        };

        // Extract samples from original file
        let sounds = sound_loader::load_xm_samples(&data).context("Failed to extract samples")?;

        // Check if we actually have samples
        let has_samples = sounds.iter().any(|s| s.is_some());
        if !has_samples {
            anyhow::bail!(
                "No embedded samples found in XM file.\n\
                 This appears to be a sample-less XM (samples loaded from ROM).\n\
                 For debug playback, use an XM file with embedded samples.\n\
                 Tip: Try a file ending in '-embedded.xm' if available."
            );
        }

        // Print diagnostic info
        let sample_count = sounds.iter().filter(|s| s.is_some()).count();
        let sample_sizes: Vec<usize> = sounds
            .iter()
            .filter_map(|s| s.as_ref().map(|snd| snd.data.len()))
            .collect();
        println!("Loaded {} samples, sizes: {:?}", sample_count, sample_sizes);
        println!(
            "Module: {} channels, {} instruments, {} orders",
            xm_module.num_channels,
            xm_module.instruments.len(),
            xm_module.song_length
        );

        // Create sound handles: maps instrument index (0-based) to sound array index
        // sounds = [None, sample1, sample2, ...] where index N corresponds to instrument N
        // sound_handles[instr_idx] should equal instr_idx+1 to map to sounds[instr_idx+1]
        // But only for valid instruments (sounds.len() - 1 instruments since index 0 is unused)
        let num_instruments = sounds.len().saturating_sub(1);
        let sound_handles: Vec<u32> = (0..num_instruments).map(|i| (i + 1) as u32).collect();
        println!(
            "sound_handles ({} entries): {:?}",
            sound_handles.len(),
            &sound_handles[..sound_handles.len().min(10)]
        );

        // Debug: Print first pattern's first few rows
        if let Some(pattern) = xm_module.patterns.first() {
            println!(
                "First pattern: {} rows, {} channels",
                pattern.num_rows,
                pattern.notes.first().map(|r| r.len()).unwrap_or(0)
            );
            for (row_idx, row) in pattern.notes.iter().take(4).enumerate() {
                for (ch_idx, note) in row.iter().take(4).enumerate() {
                    if note.note != 0 || note.instrument != 0 {
                        println!(
                            "  Row {} Ch {}: note={} instr={} vol={} eff={:02X} param={:02X}",
                            row_idx,
                            ch_idx,
                            note.note,
                            note.instrument,
                            note.volume,
                            note.effect,
                            note.effect_param
                        );
                    }
                }
            }
        }

        // Load into engine
        let mut engine = TrackerEngine::new();
        let handle = engine.load_xm_module(xm_module.clone(), sound_handles);

        // Initialize state
        let state = TrackerState {
            handle,
            flags: tracker_flags::PLAYING | tracker_flags::LOOPING,
            speed: xm_module.default_speed,
            bpm: xm_module.default_bpm,
            volume: 256,
            ..Default::default()
        };

        let total_orders = xm_module.song_length;
        let num_channels = xm_module.num_channels;

        // Sync engine to initial state
        engine.sync_to_state(&state, &sounds);

        Ok(Self {
            engine,
            state,
            sounds,
            filename,
            num_channels,
            total_orders,
            paused: false,
            verbose: false,
            channel_mutes: [false; 64],
        })
    }

    /// Load an IT file
    pub fn load_it(path: &Path, via_ncit: bool) -> Result<Self> {
        let data = std::fs::read(path).context("Failed to read IT file")?;
        let filename = path
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Parse the IT module
        let it_module = if via_ncit {
            let original = nether_it::parse_it(&data).context("Failed to parse IT")?;
            let packed = nether_it::pack_ncit(&original);
            nether_it::parse_ncit(&packed).context("Failed to parse NCIT")?
        } else {
            nether_it::parse_it(&data).context("Failed to parse IT")?
        };

        // Extract samples from original file
        let original_module =
            nether_it::parse_it(&data).context("Failed to parse IT for samples")?;
        let sounds = sound_loader::load_it_samples(&data, &original_module)
            .context("Failed to extract samples")?;

        // Check if we actually have samples
        let has_samples = sounds.iter().any(|s| s.is_some());
        if !has_samples {
            anyhow::bail!(
                "No embedded samples found in IT file.\n\
                 For debug playback, use an IT file with embedded samples."
            );
        }

        // Create sound handles: maps instrument index (0-based) to sound array index
        // sounds = [None, sample1, sample2, ...] where index N corresponds to sample N
        // sound_handles[sample_idx] should equal sample_idx+1 to map to sounds[sample_idx+1]
        let num_samples = sounds.len().saturating_sub(1);
        let sound_handles: Vec<u32> = (0..num_samples).map(|i| (i + 1) as u32).collect();

        // Load into engine
        let mut engine = TrackerEngine::new();
        let handle = engine.load_it_module(it_module.clone(), sound_handles);

        // Initialize state
        let state = TrackerState {
            handle,
            flags: tracker_flags::PLAYING | tracker_flags::LOOPING,
            speed: it_module.initial_speed as u16,
            bpm: it_module.initial_tempo as u16,
            volume: 256,
            ..Default::default()
        };

        let total_orders = it_module.total_orders();
        let num_channels = it_module.num_channels;

        // Sync engine to initial state
        engine.sync_to_state(&state, &sounds);

        Ok(Self {
            engine,
            state,
            sounds,
            filename,
            num_channels,
            total_orders,
            paused: false,
            verbose: false,
            channel_mutes: [false; 64],
        })
    }

    /// Render one stereo sample
    pub fn render_sample(&mut self, sample_rate: u32) -> (f32, f32) {
        if self.paused {
            return (0.0, 0.0);
        }
        let (left, right) =
            self.engine
                .render_sample_and_advance(&mut self.state, &self.sounds, sample_rate);

        // Debug: track if we're producing non-zero audio
        static SAMPLE_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        static NONZERO_COUNT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
        static MAX_LEVEL: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
        let count = SAMPLE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let level = ((left.abs().max(right.abs())) * 10000.0) as u32;
        if level > 0 {
            NONZERO_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        MAX_LEVEL.fetch_max(level, std::sync::atomic::Ordering::Relaxed);

        // Write to log file every ~2 seconds
        if count.is_multiple_of(88200) && count > 0 {
            use std::io::Write;
            let nonzero = NONZERO_COUNT.load(std::sync::atomic::Ordering::Relaxed);
            let max = MAX_LEVEL.load(std::sync::atomic::Ordering::Relaxed);
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open("tracker-debug.log")
            {
                let _ = writeln!(
                    f,
                    "[Audio] samples: {}, nonzero: {} ({:.1}%), max_level: {:.4}",
                    count,
                    nonzero,
                    100.0 * nonzero as f64 / count.max(1) as f64,
                    max as f64 / 10000.0
                );
            }
        }

        (left, right)
    }

    /// Set verbose mode
    pub fn set_verbose(&mut self, verbose: bool) {
        self.verbose = verbose;
    }

    /// Get the current display state
    pub fn get_display_state(&self) -> DisplayState {
        // Get pattern info from engine
        let module = self.engine.get_module(self.state.handle);
        let (pattern_idx, total_rows) = if let Some(m) = module {
            let order = self.state.order_position as usize;
            let pattern_idx = m.order_table.get(order).copied().unwrap_or(0);
            let rows = m
                .patterns
                .get(pattern_idx as usize)
                .map(|p| p.num_rows)
                .unwrap_or(64);
            (pattern_idx, rows)
        } else {
            (0, 64)
        };

        DisplayState {
            filename: self.filename.clone(),
            playing: !self.paused && (self.state.flags & tracker_flags::PLAYING) != 0,
            bpm: self.state.bpm,
            speed: self.state.speed,
            order: self.state.order_position,
            total_orders: self.total_orders,
            pattern: pattern_idx,
            row: self.state.row,
            total_rows,
            tick: self.state.tick,
            num_channels: self.num_channels,
            verbose: self.verbose,
            channel_mutes: self.channel_mutes,
        }
    }

    /// Handle a player command
    pub fn handle_command(&mut self, cmd: PlayerCommand) {
        match cmd {
            PlayerCommand::TogglePause => {
                self.paused = !self.paused;
            }
            PlayerCommand::SeekRow(delta) => {
                let new_row = (self.state.row as i32 + delta).max(0) as u16;
                self.state.row = new_row;
                self.state.tick = 0;
                self.state.tick_sample_pos = 0;
            }
            PlayerCommand::SeekPattern(delta) => {
                let new_order = (self.state.order_position as i32 + delta)
                    .clamp(0, self.total_orders as i32 - 1) as u16;
                self.state.order_position = new_order;
                self.state.row = 0;
                self.state.tick = 0;
                self.state.tick_sample_pos = 0;
            }
            PlayerCommand::AdjustTempo(delta) => {
                let new_bpm = (self.state.bpm as i32 + delta as i32).clamp(32, 255) as u16;
                self.state.bpm = new_bpm;
            }
            PlayerCommand::ToggleMute(channel) => {
                if (channel as usize) < 64 {
                    self.channel_mutes[channel as usize] = !self.channel_mutes[channel as usize];
                }
            }
            PlayerCommand::ToggleVerbose => {
                self.verbose = !self.verbose;
            }
        }
    }
}
