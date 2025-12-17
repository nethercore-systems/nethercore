# Synchronized Save Slots Specification (VMU-Style Memory Cards)

**Status:** Ready for Implementation
**Author:** Claude
**Version:** 2.0
**Last Updated:** December 2024

---

## Summary

Implement VMU-style synchronized save slots for networked sessions. Each player "brings" their own save data to a session, enabling fighting games with unlocked characters, RPGs with persistent stats, and any game that benefits from player-specific progression in multiplayer.

**Key concept:** Save data is exchanged during session setup, before `init()` runs. All clients receive an identical slot layout where slot N = player N's save data.

---

## Motivation

Current save system limitations for multiplayer:
- Save slots are local-only, not shared across network
- No way for players to bring persistent unlocks to online matches
- Games can't implement "memory card" style progression

Use cases enabled:
- **Fighting games:** Unlocked characters, costume colors, player stats
- **Racing games:** Unlocked vehicles, custom liveries, ghost data
- **RPGs:** Character levels, equipment, persistent upgrades
- **Party games:** High scores, achievements, player profiles

---

## Current Architecture

### Save System (`core/src/wasm/state.rs`)

```rust
pub const MAX_SAVE_SLOTS: usize = 8;
pub const MAX_SAVE_SIZE: usize = 64 * 1024;  // 64KB per slot

pub struct GameState<I: ConsoleInput> {
    // ...
    pub save_data: [Option<Vec<u8>>; MAX_SAVE_SLOTS],
    pub player_count: u32,
    pub local_player_mask: u32,
    // ...
}
```

### FFI Functions (`core/src/ffi.rs`)

```rust
fn save(slot: u32, data_ptr: u32, data_len: u32) -> u32;   // 0=ok, 1=bad slot, 2=too large
fn load(slot: u32, data_ptr: u32, max_len: u32) -> u32;    // Returns bytes loaded
fn delete(slot: u32) -> u32;                                // 0=ok, 1=bad slot
fn player_count() -> u32;                                   // Number of players in session
fn local_player_mask() -> u32;                              // Bitmask of local players
```

### ConsoleSpecs (`shared/src/console.rs`)

```rust
pub struct ConsoleSpecs {
    pub name: &'static str,
    pub ram_limit: usize,
    pub vram_limit: usize,
    pub rom_limit: usize,
    // ... no save-related fields currently
}
```

---

## Constants

### Console-Specific (from ConsoleSpecs)

These limits are defined per-console, allowing different consoles to have different player/profile counts:

```rust
pub struct ConsoleSpecs {
    // ... existing fields ...

    /// Maximum players supported by this console
    pub max_players: usize,

    /// Maximum save profiles per game (typically equals max_players)
    pub max_profiles: usize,

    /// Maximum save data size per player in bytes
    pub save_data_limit: usize,
}

// Emberware ZX
pub const fn emberware_zx_specs() -> &'static ConsoleSpecs {
    &ConsoleSpecs {
        // ... existing ...
        max_players: 4,
        max_profiles: 4,
        save_data_limit: 64 * 1024,  // 64KB per player
    }
}

// Emberware Chroma (example: 2-player retro console)
pub const fn emberware_chroma_specs() -> &'static ConsoleSpecs {
    &ConsoleSpecs {
        // ... existing ...
        max_players: 2,
        max_profiles: 2,
        save_data_limit: 16 * 1024,  // 16KB per player
    }
}
```

### Global Constants

These are fixed across all consoles:

```rust
/// Total save slots: 0..(max_players-1) = player slots, rest = shared
/// Fixed at 8 for simplicity (all consoles share this limit)
pub const MAX_SAVE_SLOTS: usize = 8;

/// Network chunk size for save transfer (fits in WebRTC MTU)
pub const CHUNK_SIZE: usize = 8 * 1024;

/// Timeout for save synchronization during session setup
pub const SYNC_TIMEOUT_MS: u64 = 5000;

/// Maximum retransmit attempts before aborting
pub const MAX_RETRANSMITS: u32 = 3;

/// Initial retransmit delay (doubles each attempt)
pub const RETRANSMIT_BASE_MS: u64 = 100;
```

**Slot allocation (example for 4-player console):**
| Slots | Purpose | Write Access |
|-------|---------|--------------|
| 0..(max_players-1) | Player save data (slot N = player N) | Owner only |
| max_players..7 | Shared/game data (high scores, etc.) | Any player |

**Note:** Slot count is always 8, but player slots scale with `specs.max_players`.

---

## Proposed Architecture

### 1. Extend ConsoleSpecs

Add per-player save data limit:

```rust
pub struct ConsoleSpecs {
    // ... existing fields ...

    /// Maximum save data size per player in bytes (for synchronized sessions)
    /// Default: 64KB. Set to 0 to disable synchronized saves.
    pub save_data_limit: usize,
}
```

**Console values:**
- Emberware ZX: `64 * 1024` (64KB per player)
- Emberware Chroma: `16 * 1024` (16KB per player, smaller for retro feel)

### 2. SessionSaveData Structure

New structure to hold synchronized save data:

```rust
/// Synchronized save data for all players in a session
pub struct SessionSaveData {
    /// Per-player save buffers, indexed by player number
    /// Length matches specs.max_players for the console
    /// None = player has no save data (new player or spectator)
    pub slots: Vec<Option<Vec<u8>>>,

    /// Whether synchronization is complete
    pub synchronized: bool,
}

impl SessionSaveData {
    /// Create with capacity for the console's max players
    pub fn new(max_players: usize) -> Self {
        Self {
            slots: vec![None; max_players],
            synchronized: false,
        }
    }

    /// Set local player's save data (before session starts)
    pub fn set_local(&mut self, player_index: usize, data: Vec<u8>) {
        if player_index < self.slots.len() {
            self.slots[player_index] = Some(data);
        }
    }

    /// Get save data for a player (after synchronization)
    pub fn get(&self, player_index: usize) -> Option<&[u8]> {
        self.slots.get(player_index)?.as_deref()
    }

    /// Total size of all save data (for validation)
    pub fn total_size(&self) -> usize {
        self.slots.iter()
            .filter_map(|s| s.as_ref())
            .map(|v| v.len())
            .sum()
    }
}
```

### 3. Network Protocol

Save data exchange happens during session setup, before GGRS starts.

**Key design decisions:**
- Uses the **same unreliable data channel** as GGRS (no separate reliable channel)
- Implements **manual ARQ** (Automatic Repeat Request) for reliability
- Player indices are **assigned by matchmaker** before connection (not negotiated)

```
┌────────────────────────────────────────────────────────────────────────┐
│                    Session Setup with Save Sync                        │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  PHASE 0: Player Index Assignment (before connection)                 │
│  - Matchmaker assigns player indices (0, 1, 2, 3)                     │
│  - Each client knows their index before connecting                    │
│  - Index determines save slot ownership                               │
│           │                                                            │
│           ▼                                                            │
│  PHASE 1: Connection & Announcement                                   │
│  1. Players connect via signaling (WebRTC offer/answer)              │
│  2. Each player broadcasts Announce packet:                           │
│     ┌─────────────────────────────────────────────────────────────┐   │
│     │ Announce (0x00)                                             │   │
│     │ ├─ player_index: u8      (assigned by matchmaker)          │   │
│     │ ├─ total_len: u32        (0 if no save data)               │   │
│     │ └─ hash: u32             (CRC32 of full save)              │   │
│     └─────────────────────────────────────────────────────────────┘   │
│           │                                                            │
│           ▼                                                            │
│  PHASE 2: Data Transfer (with ARQ)                                    │
│  3. For each player with save data, send chunked data:               │
│     ┌─────────────────────────────────────────────────────────────┐   │
│     │ Data (0x01)                                                 │   │
│     │ ├─ player_index: u8                                        │   │
│     │ ├─ sequence: u16         (for ordering & ACK tracking)     │   │
│     │ ├─ chunk_index: u16      (0..total_chunks-1)               │   │
│     │ ├─ total_chunks: u16                                       │   │
│     │ ├─ chunk_len: u16        (≤ CHUNK_SIZE)                    │   │
│     │ └─ data: [u8; chunk_len]                                   │   │
│     └─────────────────────────────────────────────────────────────┘   │
│                                                                        │
│  4. Receiver sends periodic ACKs with received bitmask:              │
│     ┌─────────────────────────────────────────────────────────────┐   │
│     │ Ack (0x02)                                                  │   │
│     │ ├─ player_index: u8      (whose data is being ACKed)       │   │
│     │ ├─ last_sequence: u16    (highest sequence received)       │   │
│     │ └─ ack_bitfield: u32     (selective ACK for last 32 seqs)  │   │
│     └─────────────────────────────────────────────────────────────┘   │
│                                                                        │
│  5. Sender retransmits unACKed chunks (see ARQ below)                │
│           │                                                            │
│           ▼                                                            │
│  PHASE 3: Synchronization                                             │
│  6. Once all chunks received, verify hash                            │
│  7. If hash mismatch: request full retransmit (up to 3 attempts)     │
│  8. Once verified, broadcast Ready packet                            │
│  9. When all Ready packets received → populate GameState             │
│           │                                                            │
│           ▼                                                            │
│  PHASE 4: Game Start                                                  │
│  10. Call init() — game can access all player saves                  │
│  11. Start GGRS session for gameplay                                 │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

### ARQ (Automatic Repeat Request) Details

Since we use an unreliable channel, we implement our own reliability:

```rust
pub struct ArqConfig {
    /// Initial retransmit timeout
    pub initial_rto_ms: u64,           // Default: 100ms

    /// Maximum retransmit timeout (after backoff)
    pub max_rto_ms: u64,               // Default: 2000ms

    /// Backoff multiplier for each retry
    pub backoff_factor: f32,           // Default: 2.0

    /// Maximum retransmit attempts before abort
    pub max_retransmits: u32,          // Default: 3

    /// ACK send interval
    pub ack_interval_ms: u64,          // Default: 50ms
}

impl Default for ArqConfig {
    fn default() -> Self {
        Self {
            initial_rto_ms: RETRANSMIT_BASE_MS,  // 100ms
            max_rto_ms: 2000,
            backoff_factor: 2.0,
            max_retransmits: MAX_RETRANSMITS,    // 3
            ack_interval_ms: 50,
        }
    }
}
```

**Retransmit flow:**
1. Send chunk, start timer at `initial_rto_ms` (100ms)
2. If ACK received → mark chunk complete, cancel timer
3. If timer expires → retransmit, double timeout (up to `max_rto_ms`)
4. After `max_retransmits` (3) failures → abort with `Error::Timeout`

**Selective ACK (SACK):**
- `ack_bitfield` is a 32-bit field where bit N = "received sequence (last_sequence - N)"
- Allows sender to know exactly which chunks need retransmit
- Example: `last_sequence=42, ack_bitfield=0b11110111` means seq 39 is missing

**Chunking:**
- WebRTC data channels have MTU limits (~16KB typical)
- Chunk size: 8KB (conservative, fits comfortably)
- 64KB save = 8 chunks maximum

### 4. Packet Definitions

```rust
/// Save sync packet types (first byte of every packet)
#[repr(u8)]
pub enum SaveSyncPacketType {
    Announce = 0x00,
    Data = 0x01,
    Ack = 0x02,
    Ready = 0x03,
    Error = 0xFF,
}

/// Announce packet: declares save data availability
/// Sent once per player at start of sync
#[repr(C, packed)]
pub struct AnnouncePacket {
    pub packet_type: u8,       // 0x00
    pub player_index: u8,      // 0-3 (assigned by matchmaker)
    pub total_len: u32,        // Total save size in bytes (0 = no save)
    pub hash: u32,             // CRC32 of complete save data
}
// Size: 10 bytes

/// Data packet: carries one chunk of save data
#[repr(C, packed)]
pub struct DataPacket {
    pub packet_type: u8,       // 0x01
    pub player_index: u8,      // Whose save this chunk belongs to
    pub sequence: u16,         // Monotonic sequence number (for ARQ)
    pub chunk_index: u16,      // Which chunk (0..total_chunks-1)
    pub total_chunks: u16,     // How many chunks total
    pub chunk_len: u16,        // Bytes of data in this chunk (≤ CHUNK_SIZE)
    // Followed by: data: [u8; chunk_len]
}
// Size: 10 bytes header + chunk_len data

/// Ack packet: acknowledges received chunks with selective ACK
#[repr(C, packed)]
pub struct AckPacket {
    pub packet_type: u8,       // 0x02
    pub player_index: u8,      // Whose data is being ACKed
    pub last_sequence: u16,    // Highest sequence number received
    pub ack_bitfield: u32,     // Bit N = received (last_sequence - N)
}
// Size: 8 bytes

/// Ready packet: signals this client has all saves verified
#[repr(C, packed)]
pub struct ReadyPacket {
    pub packet_type: u8,       // 0x03
    pub player_index: u8,      // Who is ready
}
// Size: 2 bytes

/// Error packet: signals sync failure
#[repr(C, packed)]
pub struct ErrorPacket {
    pub packet_type: u8,       // 0xFF
    pub error_code: u8,        // See SaveSyncError
    pub player_index: u8,      // Which player caused error (if applicable)
}
// Size: 3 bytes

#[repr(u8)]
pub enum SaveSyncError {
    /// Save data exceeds console limit
    TooLarge = 0x01,
    /// Timeout waiting for save data (after MAX_RETRANSMITS)
    Timeout = 0x02,
    /// CRC32 hash mismatch after all chunks received
    HashMismatch = 0x03,
    /// Player disconnected during sync
    Disconnected = 0x04,
    /// Protocol violation (unexpected packet)
    ProtocolError = 0x05,
}
```

### 5. Slot Mapping

**Convention:** Slot index = player index

| Slot | Contents | Access Pattern |
|------|----------|----------------|
| 0 | Player 0's save data | P0 reads/writes (their data) |
| 1 | Player 1's save data | P1 reads/writes (their data) |
| 2 | Player 2's save data | P2 reads/writes (their data) |
| 3 | Player 3's save data | P3 reads/writes (their data) |
| 4-7 | Unused (or game-specific shared data) | Any player |

**Game code pattern:**

```rust
// In init() or update()
let my_player = 0;  // from local_player_mask()
for i in 0..player_count() {
    let player_mask = 1 << i;
    let is_me = (local_player_mask() & player_mask) != 0;

    // Load player i's save data
    let bytes_read = load(i, buffer.as_mut_ptr(), buffer.len());
    if bytes_read > 0 {
        let player_data = deserialize(&buffer[..bytes_read]);
        if is_me {
            // This is my save data - I can modify it
        } else {
            // This is another player's data - read-only access
        }
    }
}
```

---

## Storage

### Local Save File Location

```
~/.emberware/
├── config.toml                    # Global settings
├── profiles.toml                  # Controller-to-profile bindings (per-console)
└── games/{game_id}/
    └── saves/
        ├── profile_0.sav          # Profile 0's save data
        ├── profile_1.sav          # Profile 1's save data
        ├── ...                    # Up to specs.max_profiles
        └── profile_N.sav          # (N = max_profiles - 1)
```

**Note:** Profile count varies by console. Emberware ZX: 4 profiles. Chroma: 2 profiles.

**Migration:** Old `player.sav` format is deprecated. Rename to `profile_0.sav`.

### Save File Format

Simple format with header for versioning:

```
┌────────────────────────────────────────┐
│ Header (16 bytes)                      │
│ ├─ magic: "EMBR" (4 bytes)            │
│ ├─ version: u32                       │
│ ├─ data_len: u32                      │
│ └─ checksum: u32 (CRC32)              │
├────────────────────────────────────────┤
│ Data                                   │
│ └─ [u8; data_len] (game-defined)      │
└────────────────────────────────────────┘
```

### Persistence Lifecycle

```
┌────────────────────────────────────────────────────────────────────────┐
│                       Save Data Lifecycle                              │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  SESSION START (Single Player)                                        │
│  1. Controller 0 → bound profile (from profiles.toml)                │
│  2. Load ~/.emberware/games/{game_id}/saves/profile_{N}.sav          │
│  3. Place in slot 0                                                   │
│  4. Call init()                                                       │
│                                                                        │
│  SESSION START (Local Multiplayer)                                    │
│  1. For each controller, resolve bound profile                       │
│  2. Load each profile's save file                                    │
│  3. Place in slots by player index (controller order)                │
│  4. Call init()                                                       │
│                                                                        │
│  SESSION START (Network)                                              │
│  1. Player index assigned by matchmaker (known before connect)       │
│  2. Load local profile's save from disk                              │
│  3. Save sync protocol exchanges all player saves                    │
│  4. Place in slots 0..player_count (slot N = player N's save)        │
│  5. Call init()                                                       │
│                                                                        │
│  DURING GAME                                                          │
│  - Game calls save(slot, ...) to update in-memory slot               │
│  - Platform persists to disk on GGRS-confirmed frames (background)   │
│  - Speculative/rolled-back saves are NOT persisted                   │
│                                                                        │
│  SESSION END                                                          │
│  1. Final persist of confirmed save state                            │
│  2. Each local player's slot written to their profile file           │
│  3. Remote player slots discarded (they persist on their machine)    │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Profile System

The platform manages save profiles per game, bound to controller slots. The number of profiles matches `specs.max_profiles` for the console (e.g., 4 for Emberware ZX, 2 for Chroma).

### Controller-to-Profile Binding

Stored globally in `~/.emberware/profiles.toml`:

```toml
# Each controller slot maps to a profile index
# Number of entries matches specs.max_players for the console
# Profile index determines which profile_N.sav file is used

# Emberware ZX (4-player console)
[bindings.emberware-zx]
controller_0 = 0    # Controller 0 uses profile_0.sav
controller_1 = 1    # Controller 1 uses profile_1.sav
controller_2 = 2    # Controller 2 uses profile_2.sav
controller_3 = 3    # Controller 3 uses profile_3.sav

# Emberware Chroma (2-player console)
[bindings.emberware-chroma]
controller_0 = 0
controller_1 = 1
```

### ProfileManager Implementation

```rust
pub struct ProfileManager {
    /// Controller slot → profile index mapping
    /// Length matches specs.max_players for the console
    bindings: Vec<usize>,

    /// Max profiles for validation (from specs.max_profiles)
    max_profiles: usize,
}

impl ProfileManager {
    /// Load bindings from profiles.toml, or use defaults (controller N → profile N)
    /// max_players and max_profiles come from ConsoleSpecs
    pub fn load(max_players: usize, max_profiles: usize) -> Self {
        // Default: controller N → profile N
        let bindings = (0..max_players).collect();
        Self { bindings, max_profiles }
    }

    /// Save bindings to profiles.toml
    pub fn save(&self) -> Result<()>;

    /// Get profile index for a controller slot
    pub fn profile_for_controller(&self, controller: usize) -> usize {
        self.bindings.get(controller).copied().unwrap_or(controller)
    }

    /// Get save file path for a controller's profile
    pub fn save_path(&self, game_id: &str, controller: usize) -> PathBuf {
        let profile = self.profile_for_controller(controller);
        data_dir()
            .join("games")
            .join(game_id)
            .join("saves")
            .join(format!("profile_{}.sav", profile))
    }

    /// Validate bindings: no two controllers can use the same profile in a session
    pub fn validate_session(&self, controllers: &[usize]) -> Result<(), ProfileError> {
        let mut used = vec![false; self.max_profiles];
        for &ctrl in controllers {
            let profile = self.profile_for_controller(ctrl);
            if profile >= self.max_profiles {
                return Err(ProfileError::InvalidProfile { profile, controller: ctrl });
            }
            if used[profile] {
                return Err(ProfileError::DuplicateProfile { profile, controller: ctrl });
            }
            used[profile] = true;
        }
        Ok(())
    }
}

#[derive(Debug)]
pub enum ProfileError {
    /// Two controllers in the session are bound to the same profile
    DuplicateProfile { profile: usize, controller: usize },
    /// Profile index exceeds specs.max_profiles
    InvalidProfile { profile: usize, controller: usize },
    /// Controller has no profile binding (should never happen with defaults)
    NoBinding { controller: usize },
}
```

### Local Multiplayer Flow

```
┌────────────────────────────────────────────────────────────────────────┐
│                    Local Multiplayer Profile Resolution                │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  Example: 2-player local session                                      │
│                                                                        │
│  1. Controllers connected: [0, 1]                                     │
│  2. Look up profiles.toml:                                            │
│     - Controller 0 → Profile 2                                        │
│     - Controller 1 → Profile 0                                        │
│  3. Validate: profiles [2, 0] are unique ✓                           │
│  4. Load saves:                                                       │
│     - Slot 0 (player 0): profile_2.sav                               │
│     - Slot 1 (player 1): profile_0.sav                               │
│  5. Set local_player_mask = 0b11                                     │
│  6. Call init()                                                       │
│                                                                        │
│  On session end:                                                      │
│     - Slot 0 → profile_2.sav                                         │
│     - Slot 1 → profile_0.sav                                         │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## Rollback-Safe Persistence

Save data must persist to disk without pausing the simulation, and must handle GGRS rollbacks correctly.

### The Problem

1. Game calls `save(slot, data)` during `update()`
2. GGRS may roll back that frame (prediction was wrong)
3. If we persisted immediately, we'd have saved "future" data that never happened
4. We need to only persist saves from **confirmed** frames

### Solution: Confirmed-Frame Persistence

```rust
pub struct SavePersistence {
    /// Pending writes waiting to be persisted (slot → data)
    pending: Arc<Mutex<PendingWrites>>,

    /// Background writer thread handle
    writer_handle: Option<JoinHandle<()>>,

    /// Signal to stop the writer thread
    shutdown: Arc<AtomicBool>,
}

struct PendingWrites {
    /// Per-slot pending write: (confirmed_frame, data, profile_path)
    slots: [Option<PendingWrite>; MAX_SAVE_SLOTS],

    /// Dirty flags to trigger background writes
    dirty: [bool; MAX_SAVE_SLOTS],
}

struct PendingWrite {
    /// GGRS frame number when this save was confirmed
    frame: Frame,

    /// Save data to persist
    data: Vec<u8>,

    /// Target file path (profile_N.sav)
    path: PathBuf,
}
```

### Integration with GGRS

```rust
impl SavePersistence {
    /// Called when game invokes save() FFI during update()
    /// Does NOT persist immediately - just queues for confirmation
    pub fn queue_save(&self, slot: usize, data: Vec<u8>, path: PathBuf, frame: Frame) {
        let mut pending = self.pending.lock().unwrap();
        pending.slots[slot] = Some(PendingWrite { frame, data, path });
        // NOT marked dirty yet - wait for confirmation
    }

    /// Called when GGRS confirms a frame (via Event::Confirmed)
    /// Marks saves from that frame as safe to persist
    pub fn on_frame_confirmed(&self, confirmed_frame: Frame) {
        let mut pending = self.pending.lock().unwrap();
        for (slot, write) in pending.slots.iter_mut().enumerate() {
            if let Some(w) = write {
                if w.frame <= confirmed_frame {
                    pending.dirty[slot] = true;
                }
            }
        }
    }

    /// Called when GGRS rolls back frames
    /// Discards pending writes for rolled-back frames
    pub fn on_rollback(&self, rollback_to: Frame) {
        let mut pending = self.pending.lock().unwrap();
        for write in pending.slots.iter_mut() {
            if let Some(w) = write {
                if w.frame > rollback_to {
                    // This save was from a speculative frame - discard it
                    *write = None;
                }
            }
        }
    }
}
```

### Background Writer Thread

```rust
fn writer_thread(pending: Arc<Mutex<PendingWrites>>, shutdown: Arc<AtomicBool>) {
    while !shutdown.load(Ordering::Relaxed) {
        // Check for dirty slots
        let writes_to_do: Vec<(usize, PendingWrite)> = {
            let mut pending = pending.lock().unwrap();
            let mut writes = Vec::new();
            for (slot, dirty) in pending.dirty.iter_mut().enumerate() {
                if *dirty {
                    if let Some(write) = pending.slots[slot].take() {
                        writes.push((slot, write));
                        *dirty = false;
                    }
                }
            }
            writes
        };

        // Write to disk outside the lock
        for (slot, write) in writes_to_do {
            if let Err(e) = write_save_file(&write.path, &write.data) {
                log::error!("Failed to persist slot {}: {}", slot, e);
            }
        }

        // Sleep to avoid busy-waiting (100ms poll interval)
        std::thread::sleep(Duration::from_millis(100));
    }
}

fn write_save_file(path: &Path, data: &[u8]) -> Result<()> {
    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Write header + data
    let header = SaveHeader {
        magic: *b"EMBR",
        version: 1,
        data_len: data.len() as u32,
        checksum: crc32fast::hash(data),
    };

    // Atomic write: write to temp file, then rename
    let temp_path = path.with_extension("tmp");
    let mut file = File::create(&temp_path)?;
    file.write_all(bytemuck::bytes_of(&header))?;
    file.write_all(data)?;
    file.sync_all()?;
    std::fs::rename(temp_path, path)?;

    Ok(())
}
```

### Frame Confirmation Flow

```
┌────────────────────────────────────────────────────────────────────────┐
│                    Rollback-Safe Save Persistence                      │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  Frame 100: Game calls save(0, data)                                  │
│             → queue_save(slot=0, frame=100)                           │
│             → pending.slots[0] = Some(frame=100, data)                │
│             → dirty[0] = false (not confirmed yet)                    │
│                                                                        │
│  Frame 101: Game runs normally                                        │
│                                                                        │
│  Frame 102: GGRS rolls back to frame 99 (wrong prediction)            │
│             → on_rollback(rollback_to=99)                             │
│             → pending.slots[0] = None (frame 100 > 99, discard)       │
│                                                                        │
│  Frame 100': Game re-runs frame 100 with correct inputs               │
│              Game calls save(0, different_data)                       │
│              → pending.slots[0] = Some(frame=100, different_data)     │
│                                                                        │
│  Frame 105: GGRS confirms frame 100                                   │
│             → on_frame_confirmed(100)                                 │
│             → dirty[0] = true                                         │
│                                                                        │
│  Background: Writer thread sees dirty[0] = true                       │
│              → Writes profile_0.sav to disk                           │
│              → dirty[0] = false                                       │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## FFI API

### Updated Functions

```rust
/// Save data to a slot
/// - Slots 0-3: Must match local_player_mask (enforced)
/// - Slots 4-7: Any player can write (shared slots)
/// Returns: 0=success, 1=invalid slot, 2=data too large, 3=not your slot
fn save(slot: u32, data_ptr: u32, data_len: u32) -> u32;

/// Load data from any slot (read access is unrestricted)
/// Returns: bytes loaded, or 0 if empty/invalid
fn load(slot: u32, data_ptr: u32, max_len: u32) -> u32;

/// Delete data from a slot (same ownership rules as save)
/// Returns: 0=success, 1=invalid slot, 3=not your slot
fn delete(slot: u32) -> u32;

/// Get number of players in session
fn player_count() -> u32;

/// Get bitmask of local players (bit N = player N is local)
fn local_player_mask() -> u32;
```

### Write Permission Enforcement

```rust
/// Implementation of write permission check
/// max_players comes from ConsoleSpecs (e.g., 4 for Emberware Z, 2 for Classic)
fn check_write_permission(slot: u32, local_player_mask: u32, max_players: u32) -> bool {
    if slot >= MAX_SAVE_SLOTS as u32 {
        return false; // Invalid slot
    }
    if slot >= max_players {
        return true; // Slots beyond player count are shared, anyone can write
    }
    // Slots 0..(max_players-1): must own the slot
    let slot_mask = 1u32 << slot;
    (local_player_mask & slot_mask) != 0
}

// In save() implementation:
pub fn save(slot: u32, data_ptr: u32, data_len: u32) -> u32 {
    let specs = game_state.console_specs();
    if slot >= MAX_SAVE_SLOTS as u32 {
        return 1; // Invalid slot
    }
    if data_len > specs.save_data_limit as u32 {
        return 2; // Too large
    }
    if !check_write_permission(slot, game_state.local_player_mask, specs.max_players as u32) {
        return 3; // Not your slot
    }
    // ... proceed with save
    0
}
```

### New Functions

```rust
/// Get the save data size limit per player (from ConsoleSpecs)
/// Returns: max bytes per player save
fn save_data_limit() -> u32;

/// Check if save slot has data
/// Returns: data length, or 0 if empty
fn save_slot_size(slot: u32) -> u32;

/// Get which slot belongs to local player (convenience)
/// For multi-local (couch co-op), returns first local player's slot
/// Returns: slot index for first local player, or 0xFFFFFFFF if spectator
fn local_save_slot() -> u32;

/// Check if local player can write to a slot
/// Returns: 1 if writable, 0 if not
fn can_write_slot(slot: u32) -> u32;
```

### Error Codes Reference

| Code | Constant | Meaning |
|------|----------|---------|
| 0 | `SAVE_OK` | Success |
| 1 | `SAVE_ERR_INVALID_SLOT` | Slot index out of range (0-7) |
| 2 | `SAVE_ERR_TOO_LARGE` | Data exceeds MAX_SAVE_SIZE (64KB) |
| 3 | `SAVE_ERR_NOT_OWNER` | Tried to write to another player's slot (0-3) |

---

## Edge Cases

### Player Without Save Data

- Slot contains empty buffer (length 0)
- `load()` returns 0 bytes
- Game should handle gracefully (use defaults)

### Save Data Too Large

**Validation:** During session setup, before exchange begins

```
1. Each player announces: [player_index, save_len]
2. If any save_len > specs.save_data_limit:
   - Send Error(TooLarge) packet
   - Abort session with user-friendly error
   - "Player X's save data (128KB) exceeds limit (64KB)"
```

### Player Disconnect During Exchange

- Set timeout (5 seconds default)
- If timeout expires:
  - Send Error(Timeout) to remaining players
  - Abort session
  - Show: "Connection lost during save sync"

### Spectators

- Receive all player save data (for UI display, replay, etc.)
- Do NOT contribute a slot
- `local_player_mask() == 0` indicates spectator
- Slots 0..player_count still filled with player data

### Late Join (Future)

- Not supported initially
- Player must have been in session setup phase
- Future enhancement: join with empty save, sync mid-game

### Hash Mismatch

- Receiver computes CRC32 of received data
- Compare against sender's announced hash
- If mismatch: request retransmit (up to 3 attempts)
- After 3 failures: abort with Error(HashMismatch)

### Local Multiplayer

Each controller is bound to a profile in system settings (see Profile System section).

**Scenario:** 2 local players on same machine

```
Controller 0 bound to Profile 2
Controller 1 bound to Profile 0

local_player_mask = 0b11  // Players 0 and 1 are local

Slot 0 (player 0): Loaded from profile_2.sav (controller 0's profile)
Slot 1 (player 1): Loaded from profile_0.sav (controller 1's profile)
```

On session end, slots write back to their source profiles:
- Slot 0 → profile_2.sav
- Slot 1 → profile_0.sav

### Profile Not Selected

If a controller has no profile binding (edge case, shouldn't happen with defaults):
- Platform shows error: "Controller X has no profile assigned"
- Game cannot start until resolved
- Fix: Assign profile in system settings

### Duplicate Profile in Session

If two controllers are bound to the same profile:
- Platform detects conflict during session setup
- Error: "Controllers 0 and 2 are both using Profile 1"
- Game cannot start until resolved
- Fix: Rebind one controller to a different profile

### Rollback During Save

When GGRS rolls back frames:
1. Pending disk writes for rolled-back frames are discarded
2. In-memory save_data is part of GameState, which GGRS snapshots
3. After rollback, in-memory state is correct (restored from snapshot)
4. Game re-runs and may call save() again with different data
5. New save is queued, waits for confirmation before persist

---

## Implementation Plan

### Phase 1: Extend ConsoleSpecs

**Files:**
- `shared/src/console.rs` — Add `save_data_limit` field

```rust
pub struct ConsoleSpecs {
    // ... existing ...
    pub save_data_limit: usize,  // NEW
}

pub const fn emberware_z_specs() -> &'static ConsoleSpecs {
    &ConsoleSpecs {
        // ... existing ...
        save_data_limit: 64 * 1024,  // 64KB per player
    }
}
```

### Phase 2: SessionSaveData Structure

**Files:**
- `core/src/session/save_data.rs` (new) — SessionSaveData struct
- `core/src/session/mod.rs` — Re-export

```rust
// core/src/session/save_data.rs
pub struct SessionSaveData { /* as defined above */ }

impl SessionSaveData {
    pub fn new() -> Self { ... }
    pub fn set_local(&mut self, idx: usize, data: Vec<u8>) { ... }
    pub fn get(&self, idx: usize) -> Option<&[u8]> { ... }
    pub fn populate_game_state<I: ConsoleInput>(&self, state: &mut GameState<I>) { ... }
}
```

### Phase 3: Save Sync Protocol

**Files:**
- `core/src/session/save_sync.rs` (new) — Protocol implementation
- `core/src/rollback/session.rs` — Integration with RollbackSession

```rust
// core/src/session/save_sync.rs
pub struct SaveSyncState {
    session_data: SessionSaveData,
    pending_chunks: HashMap<u8, Vec<Option<Vec<u8>>>>,
    received_acks: u32,  // Bitmask
    timeout: Instant,
}

impl SaveSyncState {
    pub fn start_sync(&mut self, local_player: u8, local_save: Option<Vec<u8>>) -> Vec<SaveSyncPacket>;
    pub fn handle_packet(&mut self, packet: &[u8]) -> SaveSyncResult;
    pub fn is_complete(&self) -> bool;
    pub fn into_session_data(self) -> SessionSaveData;
}

pub enum SaveSyncResult {
    Continue(Vec<SaveSyncPacket>),  // Packets to send
    Complete(SessionSaveData),       // Ready to start game
    Error(SaveSyncError),           // Abort session
}
```

### Phase 4: Platform Integration

**Files:**
- `library/src/save_manager.rs` (new) — Disk persistence
- `core/src/app/session.rs` — Load/save on session boundaries

```rust
// library/src/save_manager.rs
pub fn load_player_save(game_id: &str) -> Option<Vec<u8>>;
pub fn save_player_save(game_id: &str, data: &[u8]) -> Result<()>;
pub fn save_path(game_id: &str) -> PathBuf;

// In session lifecycle:
// - On session start: load_player_save() → SessionSaveData
// - On session end: SessionSaveData → save_player_save()
```

### Phase 5: Local Session Support

**Files:**
- `core/src/runner.rs` — Populate slots for local play

```rust
// In ConsoleRunner::load_game() or similar:
if session.is_local() {
    let save = load_player_save(&game_id);
    game_state.save_data[0] = save;
}
```

### Phase 6: New FFI Functions

**Files:**
- `core/src/ffi.rs` — Add optional helper functions

```rust
fn save_data_limit<I, S, R>(caller: Caller<WasmGameContext<I, S, R>>) -> u32 {
    caller.data().specs.save_data_limit as u32
}

fn save_slot_size<I, S, R>(caller: Caller<WasmGameContext<I, S, R>>, slot: u32) -> u32 {
    caller.data().game.save_data.get(slot as usize)
        .and_then(|s| s.as_ref())
        .map(|v| v.len() as u32)
        .unwrap_or(0)
}
```

### Phase 7: Testing

- Unit tests for SessionSaveData
- Unit tests for save sync protocol
- Integration tests for local session save/load
- Manual testing of P2P save sync (when P2P socket available)

---

## Files to Modify

| File | Changes |
|------|---------|
| `shared/src/console.rs` | Add `save_data_limit` to ConsoleSpecs |
| `core/src/session/mod.rs` | New module for session save data |
| `core/src/session/save_data.rs` | New: SessionSaveData struct |
| `core/src/session/save_sync.rs` | New: Save sync protocol with ARQ |
| `core/src/session/save_persistence.rs` | New: Rollback-safe background persistence |
| `core/src/rollback/session.rs` | Integrate save sync, frame confirmation hooks |
| `core/src/runner.rs` | Populate save slots on session start |
| `core/src/ffi.rs` | Update save() with write enforcement, add new functions |
| `core/src/app/session.rs` | Session lifecycle hooks |
| `library/src/save_manager.rs` | New: Disk persistence for profiles |
| `library/src/profile_manager.rs` | New: Controller-to-profile bindings |

---

## Memory Impact

**Per-session overhead (scales with console's max_players):**
- `SessionSaveData`: max_players × Option<Vec<u8>> ≈ max_players × 24 bytes (pointers only)
- Actual save data: Up to max_players × save_data_limit
- Example (Emberware Z): 4 × 64KB = 256KB maximum
- Example (Emberware Classic): 2 × 16KB = 32KB maximum

**Network overhead:**
- One-time exchange during session setup
- Packets per player: save_data_limit ÷ CHUNK_SIZE (e.g., 64KB ÷ 8KB = 8)
- Example (4-player, 64KB saves): 32 packets maximum (before GGRS starts)

**Disk overhead:**
- Profile files per game: up to max_profiles (e.g., 4 for Emberware Z)
- File size: up to save_data_limit + 16 byte header per profile
- Global profiles config: `~/.emberware/profiles.toml` (negligible)

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Slot mapping | Slot N = Player N | Simple, predictable, easy for games to use |
| Player/profile limits | Console-specific (in ConsoleSpecs) | Different consoles can have 2, 4, 6+ players |
| Sync timing | Before init() | Game can use saves in initialization |
| Chunk size | 8KB | Fits in typical WebRTC MTU, 8 chunks for max save |
| Persistence timing | Confirmed frames, background thread | Rollback-safe, non-blocking |
| Spectator slots | None | Spectators observe, don't participate |
| Hash algorithm | CRC32 | Fast, sufficient for corruption detection |
| Local multi-profile | Platform-managed profiles | Controller→profile binding in system settings |
| Write enforcement | FFI-level | Prevents games from corrupting other players' saves |
| Network reliability | Manual ARQ on unreliable channel | Same channel as GGRS, full control over retransmit |
| Player assignment | External (matchmaker) | Known before connection, simplifies sync protocol |

---

## Future Enhancements

1. **Cloud sync:** Platform syncs profile saves to cloud storage
2. **Save migration:** Version field allows format upgrades
3. **Compression:** LZ4 for large saves (>16KB)
4. **Partial sync:** Only send changed data for reconnection
5. **Replay saves:** Record which saves were used for replay compatibility
6. **Profile management UI:** In-app UI to create/rename/delete profiles
7. **Profile export/import:** Share profiles between machines

