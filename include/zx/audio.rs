//! Audio Functions

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Load raw PCM sound data (22.05kHz, 16-bit signed, mono).
    ///
    /// Must be called during `init()`.
    ///
    /// # Arguments
    /// * `data_ptr` — Pointer to i16 PCM samples
    /// * `byte_len` — Length in bytes (must be even)
    ///
    /// # Returns
    /// Sound handle for use with playback functions.
    pub fn load_sound(data_ptr: *const i16, byte_len: u32) -> u32;

    /// Play sound on next available channel (fire-and-forget).
    ///
    /// # Arguments
    /// * `volume` — 0.0 to 1.0
    /// * `pan` — -1.0 (left) to 1.0 (right), 0.0 = center
    pub fn play_sound(sound: u32, volume: f32, pan: f32);

    /// Play sound on a specific channel (for managed/looping audio).
    ///
    /// # Arguments
    /// * `channel` — Channel index (0-15)
    /// * `looping` — 1 = loop, 0 = play once
    pub fn channel_play(channel: u32, sound: u32, volume: f32, pan: f32, looping: u32);

    /// Update channel parameters (call every frame for positional audio).
    pub fn channel_set(channel: u32, volume: f32, pan: f32);

    /// Stop a channel.
    pub fn channel_stop(channel: u32);
}
