//! System Functions

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Returns the fixed timestep duration in seconds.
    ///
    /// This is a **constant value** based on the configured tick rate, NOT wall-clock time.
    /// - 60fps → 0.01666... (1/60)
    /// - 30fps → 0.03333... (1/30)
    ///
    /// Safe for rollback netcode: identical across all clients regardless of frame timing.
    pub fn delta_time() -> f32;

    /// Returns total elapsed game time since start in seconds.
    ///
    /// This is the **accumulated fixed timestep**, NOT wall-clock time.
    /// Calculated as `tick_count * delta_time`.
    ///
    /// Safe for rollback netcode: deterministic and identical across all clients.
    pub fn elapsed_time() -> f32;

    /// Returns the current tick number (starts at 0, increments by 1 each update).
    ///
    /// Perfectly deterministic: same inputs always produce the same tick count.
    /// Safe for rollback netcode.
    pub fn tick_count() -> u64;

    /// Logs a message to the console output.
    ///
    /// # Arguments
    /// * `ptr` — Pointer to UTF-8 string data
    /// * `len` — Length of string in bytes
    pub fn log(ptr: *const u8, len: u32);

    /// Exits the game and returns to the library.
    pub fn quit();

    /// Returns a deterministic random u32 from the host's seeded RNG.
    /// Always use this instead of external random sources for rollback compatibility.
    pub fn random() -> u32;

    /// Returns a random i32 in range [min, max).
    /// Uses host's seeded RNG for rollback compatibility.
    pub fn random_range(min: i32, max: i32) -> i32;

    /// Returns a random f32 in range [0.0, 1.0).
    /// Uses host's seeded RNG for rollback compatibility.
    pub fn random_f32() -> f32;

    /// Returns a random f32 in range [min, max).
    /// Uses host's seeded RNG for rollback compatibility.
    pub fn random_f32_range(min: f32, max: f32) -> f32;

    /// Returns the number of players in the session (1-4).
    pub fn player_count() -> u32;

    /// Returns a bitmask of which players are local to this client.
    ///
    /// Example: `(local_player_mask() & (1 << player_id)) != 0` checks if player is local.
    pub fn local_player_mask() -> u32;

    /// Saves data to a slot.
    ///
    /// Slot semantics:
    /// - Slots 0-3 are supported.
    /// - Persistence only applies for local controllers (see `local_player_mask()`);
    ///   remote session slots never write to disk and never overwrite your local saves.
    ///
    /// # Arguments
    /// * `slot` — Save slot (0-3)
    /// * `data_ptr` — Pointer to data in WASM memory
    /// * `data_len` — Length of data in bytes (max 64KB)
    ///
    /// # Returns
    /// 0 on success, 1 if invalid slot, 2 if data too large.
    pub fn save(slot: u32, data_ptr: *const u8, data_len: u32) -> u32;

    /// Loads data from a slot.
    ///
    /// Slot semantics:
    /// - Slots 0-3 are supported.
    /// - Persistent data only exists for local controllers (see `local_player_mask()`).
    ///
    /// # Arguments
    /// * `slot` — Save slot (0-3)
    /// * `data_ptr` — Pointer to buffer in WASM memory
    /// * `max_len` — Maximum bytes to read
    ///
    /// # Returns
    /// Bytes read (0 if empty or error).
    pub fn load(slot: u32, data_ptr: *mut u8, max_len: u32) -> u32;

    /// Deletes a save slot.
    ///
    /// Slot semantics:
    /// - Slots 0-3 are supported.
    /// - Persistence only applies for local controllers (see `local_player_mask()`).
    ///
    /// # Returns
    /// 0 on success, 1 if invalid slot.
    pub fn delete(slot: u32) -> u32;

    /// Set the clear/background color. Must be called during `init()`.
    ///
    /// # Arguments
    /// * `color` — Color in 0xRRGGBBAA format (default: black)
    pub fn set_clear_color(color: u32);
}
