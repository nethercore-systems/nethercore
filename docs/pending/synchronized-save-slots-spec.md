# Synchronized Save Slots Specification (VMU-Style Memory Cards)

**Status:** Draft
**Author:** Claude
**Version:** 1.0
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
- Emberware Z: `64 * 1024` (64KB per player)
- Emberware Classic: `16 * 1024` (16KB per player, smaller for retro feel)

### 2. SessionSaveData Structure

New structure to hold synchronized save data:

```rust
/// Synchronized save data for all players in a session
pub struct SessionSaveData {
    /// Per-player save buffers, indexed by player number (0-3)
    /// None = player has no save data (new player or spectator)
    pub slots: [Option<Vec<u8>>; MAX_PLAYERS],

    /// Whether synchronization is complete
    pub synchronized: bool,
}

impl SessionSaveData {
    pub fn new() -> Self {
        Self {
            slots: Default::default(),
            synchronized: false,
        }
    }

    /// Set local player's save data (before session starts)
    pub fn set_local(&mut self, player_index: usize, data: Vec<u8>) {
        if player_index < MAX_PLAYERS {
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

Save data exchange happens during session setup, before GGRS starts:

```
┌────────────────────────────────────────────────────────────────────────┐
│                    Session Setup with Save Sync                        │
├────────────────────────────────────────────────────────────────────────┤
│                                                                        │
│  1. Players connect via signaling                                     │
│           │                                                            │
│           ▼                                                            │
│  2. Exchange player info (player_index, has_save)                     │
│           │                                                            │
│           ▼                                                            │
│  3. For each player with save data:                                   │
│     ┌─────────────────────────────────────────────────────────────┐   │
│     │ SaveDataPacket                                              │   │
│     │ ├─ type: u8 = 0x01 (SAVE_DATA)                             │   │
│     │ ├─ player_index: u8                                        │   │
│     │ ├─ chunk_index: u16                                        │   │
│     │ ├─ total_chunks: u16                                       │   │
│     │ ├─ chunk_len: u16                                          │   │
│     │ └─ data: [u8; chunk_len]                                   │   │
│     └─────────────────────────────────────────────────────────────┘   │
│           │                                                            │
│           ▼                                                            │
│  4. Send acknowledgment for each player's data                        │
│     ┌─────────────────────────────────────────────────────────────┐   │
│     │ SaveDataAck                                                 │   │
│     │ ├─ type: u8 = 0x02 (SAVE_DATA_ACK)                         │   │
│     │ └─ player_index: u8                                        │   │
│     └─────────────────────────────────────────────────────────────┘   │
│           │                                                            │
│           ▼                                                            │
│  5. Once all saves received → mark synchronized                       │
│           │                                                            │
│           ▼                                                            │
│  6. Populate GameState.save_data[0..player_count]                     │
│           │                                                            │
│           ▼                                                            │
│  7. Call init() — game can now access all player saves               │
│           │                                                            │
│           ▼                                                            │
│  8. Start GGRS session                                                │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

**Chunking rationale:** WebRTC data channels have MTU limits (~16KB typical). 64KB saves need chunking.

**Chunk size:** 8KB chunks (8 chunks for max 64KB save)

### 4. Packet Definitions

```rust
/// Save sync packet types
#[repr(u8)]
pub enum SaveSyncPacket {
    /// Announce save data availability
    /// [type: u8, player_index: u8, total_len: u32, hash: u32]
    Announce = 0x00,

    /// Save data chunk
    /// [type: u8, player_index: u8, chunk_index: u16, total_chunks: u16,
    ///  chunk_len: u16, data: [u8; chunk_len]]
    Data = 0x01,

    /// Acknowledge receipt of player's save
    /// [type: u8, player_index: u8]
    Ack = 0x02,

    /// All saves synchronized, ready to start
    /// [type: u8]
    Ready = 0x03,

    /// Error during save sync
    /// [type: u8, error_code: u8]
    Error = 0xFF,
}

#[repr(u8)]
pub enum SaveSyncError {
    /// Save data exceeds limit
    TooLarge = 0x01,
    /// Timeout waiting for save data
    Timeout = 0x02,
    /// Hash mismatch (corruption)
    HashMismatch = 0x03,
    /// Player disconnected during sync
    Disconnected = 0x04,
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
~/.emberware/games/{game_id}/saves/
├── player.sav         # Local player's persistent save
└── session/           # Temporary session saves (optional)
    ├── slot_0.sav
    ├── slot_1.sav
    └── ...
```

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
│  SESSION START (Local)                                                │
│  1. Load ~/.emberware/games/{game_id}/saves/player.sav               │
│  2. Place in slot 0                                                   │
│  3. Call init()                                                       │
│                                                                        │
│  SESSION START (Network)                                              │
│  1. Load player.sav from disk                                        │
│  2. Send to all peers during session setup                           │
│  3. Receive all peer saves                                           │
│  4. Place in slots 0..player_count (indexed by player number)        │
│  5. Call init()                                                       │
│                                                                        │
│  DURING GAME                                                          │
│  - Game calls save(slot, ...) to update in-memory slot               │
│  - Changes are NOT automatically persisted                           │
│                                                                        │
│  SESSION END                                                          │
│  1. Find local player's slot (from local_player_mask)                │
│  2. Write that slot to player.sav                                    │
│  3. Discard other slots (they belong to remote players)              │
│                                                                        │
└────────────────────────────────────────────────────────────────────────┘
```

---

## FFI API

### Existing Functions (No Changes Needed)

```rust
// These work as-is with synchronized slots
fn save(slot: u32, data_ptr: u32, data_len: u32) -> u32;
fn load(slot: u32, data_ptr: u32, max_len: u32) -> u32;
fn delete(slot: u32) -> u32;
fn player_count() -> u32;
fn local_player_mask() -> u32;
```

### New Functions (Optional)

```rust
/// Get the save data size limit per player (from ConsoleSpecs)
/// Returns: max bytes per player save
fn save_data_limit() -> u32;

/// Check if save slot has data
/// Returns: data length, or 0 if empty
fn save_slot_size(slot: u32) -> u32;

/// Get which slot belongs to local player (convenience)
/// Returns: slot index for first local player, or 0xFFFFFFFF if spectator
fn local_save_slot() -> u32;
```

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

**Scenario:** 2 local players on same machine

```
local_player_mask = 0b11  // Players 0 and 1 are local

Slot 0: Player 0's save (from player.sav or profile_0.sav)
Slot 1: Player 1's save (from player.sav or profile_1.sav)
```

**Question:** How to handle multiple local players with separate saves?
- Option A: Single save file, game handles multi-profile internally
- Option B: Multiple save files (profile_0.sav, profile_1.sav), selected by UI
- **Recommendation:** Option A for simplicity. Games can store multiple profiles in one save.

---

## Implementation Plan

### Phase 1: Extend ConsoleSpecs (0.5 days)

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

### Phase 2: SessionSaveData Structure (1 day)

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

### Phase 3: Save Sync Protocol (2 days)

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

### Phase 4: Platform Integration (1 day)

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

### Phase 5: Local Session Support (0.5 days)

**Files:**
- `core/src/runner.rs` — Populate slots for local play

```rust
// In ConsoleRunner::load_game() or similar:
if session.is_local() {
    let save = load_player_save(&game_id);
    game_state.save_data[0] = save;
}
```

### Phase 6: New FFI Functions (0.5 days)

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

### Phase 7: Testing (1 day)

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
| `core/src/session/save_sync.rs` | New: Save sync protocol |
| `core/src/rollback/session.rs` | Integrate save sync before GGRS start |
| `core/src/runner.rs` | Populate save slots on session start |
| `core/src/ffi.rs` | Optional new FFI functions |
| `core/src/app/session.rs` | Session lifecycle hooks |
| `library/src/save_manager.rs` | New: Disk persistence |

---

## Memory Impact

**Per-session overhead:**
- `SessionSaveData`: 4 × Optional<Vec<u8>> ≈ 4 × 8 = 32 bytes (pointers only)
- Actual save data: Up to 4 × 64KB = 256KB maximum

**Network overhead:**
- One-time exchange during session setup
- 64KB save ÷ 8KB chunks = 8 packets per player
- 4 players = 32 packets maximum (before GGRS starts)

**Disk overhead:**
- One file per game: `~/.emberware/games/{game_id}/saves/player.sav`
- Maximum 64KB + 16 byte header = 64KB per game

---

## Design Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Slot mapping | Slot N = Player N | Simple, predictable, easy for games to use |
| Sync timing | Before init() | Game can use saves in initialization |
| Chunk size | 8KB | Fits in typical WebRTC MTU, 8 chunks for max save |
| Persistence | Session end only | Avoid mid-game disk I/O, games control when to "commit" |
| Spectator slots | None | Spectators observe, don't participate |
| Hash algorithm | CRC32 | Fast, sufficient for corruption detection |
| Local multi-profile | Game's responsibility | Platform stays simple, games have flexibility |

---

## Future Enhancements

1. **Cloud sync:** Platform layer syncs player.sav to cloud storage
2. **Save migration:** Version field allows format upgrades
3. **Compression:** LZ4 for large saves (>16KB)
4. **Partial sync:** Only send changed data for reconnection
5. **Replay saves:** Record which saves were used for replay compatibility

---

## Estimated Effort

| Component | Effort |
|-----------|--------|
| ConsoleSpecs extension | 0.5 days |
| SessionSaveData struct | 1 day |
| Save sync protocol | 2 days |
| Platform integration | 1 day |
| Local session support | 0.5 days |
| New FFI functions | 0.5 days |
| Testing | 1 day |
| **Total** | **~6.5 days** |
