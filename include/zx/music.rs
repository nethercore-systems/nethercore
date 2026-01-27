//! Unified Music API (PCM + Tracker)
//!
//! A single API for both PCM music and XM tracker modules.
//! The handle type is detected automatically:
//! - PCM sound handles (from load_sound/rom_sound) have bit 31 = 0
//! - Tracker handles (from load_tracker/rom_tracker) have bit 31 = 1
//!
//! Starting one type automatically stops the other (mutually exclusive).
//! Supports rollback netcode: state is snapshotted and restored.

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load a tracker module from ROM data pack by ID.
    ///
    /// Must be called during `init()`.
    /// Returns a handle with bit 31 set (tracker handle).
    ///
    /// # Arguments
    /// * `id_ptr` — Pointer to tracker ID string
    /// * `id_len` — Length of tracker ID string
    ///
    /// # Returns
    /// Tracker handle (>0) on success, 0 on failure.
    pub fn rom_tracker(id_ptr: *const u8, id_len: u32) -> u32;

    /// Load a tracker module from raw XM data.
    ///
    /// Must be called during `init()`.
    /// Returns a handle with bit 31 set (tracker handle).
    ///
    /// # Arguments
    /// * `data_ptr` — Pointer to XM file data
    /// * `data_len` — Length of XM data in bytes
    ///
    /// # Returns
    /// Tracker handle (>0) on success, 0 on failure.
    pub fn load_tracker(data_ptr: *const u8, data_len: u32) -> u32;

    /// Play music (PCM sound or tracker module).
    ///
    /// Automatically stops any currently playing music of the other type.
    /// Handle type is detected by bit 31 (0=PCM, 1=tracker).
    ///
    /// # Arguments
    /// * `handle` — Sound handle (from load_sound) or tracker handle (from rom_tracker)
    /// * `volume` — 0.0 to 1.0
    /// * `looping` — 1 = loop, 0 = play once
    pub fn music_play(handle: u32, volume: f32, looping: u32);

    /// Stop music (both PCM and tracker).
    pub fn music_stop();

    /// Pause or resume music (tracker only, no-op for PCM).
    ///
    /// # Arguments
    /// * `paused` — 1 = pause, 0 = resume
    pub fn music_pause(paused: u32);

    /// Set music volume (works for both PCM and tracker).
    ///
    /// # Arguments
    /// * `volume` — 0.0 to 1.0
    pub fn music_set_volume(volume: f32);

    /// Check if music is currently playing.
    ///
    /// # Returns
    /// 1 if playing (and not paused), 0 otherwise.
    pub fn music_is_playing() -> u32;

    /// Get current music type.
    ///
    /// # Returns
    /// 0 = none, 1 = PCM, 2 = tracker
    pub fn music_type() -> u32;

    /// Jump to a specific position (tracker only, no-op for PCM).
    ///
    /// Use for dynamic music systems (e.g., jump to outro pattern).
    ///
    /// # Arguments
    /// * `order` — Order position (0-based)
    /// * `row` — Row within the pattern (0-based)
    pub fn music_jump(order: u32, row: u32);

    /// Get current music position.
    ///
    /// For tracker: (order << 16) | row
    /// For PCM: sample position
    ///
    /// # Returns
    /// Position value (format depends on music type).
    pub fn music_position() -> u32;

    /// Get music length.
    ///
    /// For tracker: number of orders in the song.
    /// For PCM: number of samples.
    ///
    /// # Arguments
    /// * `handle` — Music handle (PCM or tracker)
    ///
    /// # Returns
    /// Length value.
    pub fn music_length(handle: u32) -> u32;

    /// Set music speed (tracker only, ticks per row).
    ///
    /// # Arguments
    /// * `speed` — 1-31 (XM default is 6)
    pub fn music_set_speed(speed: u32);

    /// Set music tempo (tracker only, BPM).
    ///
    /// # Arguments
    /// * `bpm` — 32-255 (XM default is 125)
    pub fn music_set_tempo(bpm: u32);

    /// Get music info.
    ///
    /// For tracker: (num_channels << 24) | (num_patterns << 16) | (num_instruments << 8) | song_length
    /// For PCM: (sample_rate << 16) | (channels << 8) | bits_per_sample
    ///
    /// # Arguments
    /// * `handle` — Music handle (PCM or tracker)
    ///
    /// # Returns
    /// Packed info value.
    pub fn music_info(handle: u32) -> u32;

    /// Get music name (tracker only, returns 0 for PCM).
    ///
    /// # Arguments
    /// * `handle` — Music handle
    /// * `out_ptr` — Pointer to output buffer
    /// * `max_len` — Maximum bytes to write
    ///
    /// # Returns
    /// Actual length written (0 if PCM or invalid handle).
    pub fn music_name(handle: u32, out_ptr: *mut u8, max_len: u32) -> u32;
}
