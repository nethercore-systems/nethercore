# State Snapshots Specification

## Overview

State Snapshots provide quick save/load functionality during development, allowing developers to save the exact game state at any moment and restore it instantly. Unlike the Replay System (which records inputs), snapshots capture complete state and allow jumping directly to that state without replaying.

This enables:
- **Instant iteration**: Save before a tricky section, retry instantly
- **Bug reproduction**: Snapshot right before a bug, reload to reproduce
- **A/B testing**: Compare behavior from the same starting state
- **State inspection**: Freeze game, examine state, restore and continue

## Architecture

### Snapshot Data

```rust
/// Complete game state snapshot
pub struct StateSnapshot {
    /// Unique identifier
    pub id: String,

    /// User-provided name
    pub name: String,

    /// When the snapshot was taken
    pub timestamp: std::time::SystemTime,

    /// Game frame when taken
    pub frame: u64,

    /// Serialized game state (from save_state)
    pub state: Vec<u8>,

    /// Thumbnail image (optional, for UI)
    pub thumbnail: Option<Vec<u8>>,

    /// User notes
    pub notes: String,

    /// Game ID for compatibility check
    pub game_id: String,

    /// ROM hash for compatibility check
    pub rom_hash: [u8; 32],
}

/// Snapshot storage manager
pub struct SnapshotManager {
    /// In-memory quick slots (1-9)
    pub quick_slots: [Option<StateSnapshot>; 9],

    /// Named snapshots (unlimited, persisted)
    pub named_snapshots: Vec<StateSnapshot>,

    /// Auto-save slot (periodically updated)
    pub auto_save: Option<StateSnapshot>,

    /// Configuration
    pub config: SnapshotConfig,
}

pub struct SnapshotConfig {
    /// Enable auto-save
    pub auto_save_enabled: bool,

    /// Auto-save interval (seconds)
    pub auto_save_interval_secs: u32,

    /// Capture thumbnails
    pub capture_thumbnails: bool,

    /// Thumbnail resolution
    pub thumbnail_size: (u32, u32),

    /// Maximum named snapshots to keep
    pub max_named_snapshots: usize,
}
```

### Quick Slots vs Named Snapshots

**Quick Slots (1-9)**:
- Instant save/load via hotkeys
- Session-only (not persisted to disk)
- No names or metadata
- Overwrite without confirmation

**Named Snapshots**:
- Persisted to disk
- User-provided names and notes
- Thumbnail preview
- Confirmation before overwrite
- Unlimited count (configurable max)

## Workflow

### Quick Save/Load

```
┌─────────────────────────────────────────────────────────────┐
│                    Quick Save/Load                          │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Press F5 to quick save to slot 1                          │
│           │                                                 │
│           ▼                                                 │
│  1. Pause game (optional, configurable)                    │
│           │                                                 │
│           ▼                                                 │
│  2. Call game's save_state export                          │
│           │                                                 │
│           ▼                                                 │
│  3. Store state in quick_slots[0]                          │
│           │                                                 │
│           ▼                                                 │
│  4. Show toast: "Saved to Slot 1"                          │
│           │                                                 │
│           ▼                                                 │
│  5. Resume game                                            │
│                                                             │
│  ─────────────────────────────────────────────────────────  │
│                                                             │
│  Press F9 to quick load from slot 1                        │
│           │                                                 │
│           ▼                                                 │
│  1. Retrieve state from quick_slots[0]                     │
│           │                                                 │
│           ├──── Slot empty ──► Show toast: "Slot 1 empty"  │
│           │                                                 │
│           ▼                                                 │
│  2. Call game's load_state export                          │
│           │                                                 │
│           ▼                                                 │
│  3. Show toast: "Loaded Slot 1 (frame 1234)"              │
│           │                                                 │
│           ▼                                                 │
│  4. Resume game from restored state                        │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Named Snapshot Save

```
┌─────────────────────────────────────────────────────────────┐
│                    Named Snapshot Save                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Press Ctrl+S or use menu                                  │
│           │                                                 │
│           ▼                                                 │
│  1. Pause game                                             │
│           │                                                 │
│           ▼                                                 │
│  2. Capture current frame as thumbnail                     │
│           │                                                 │
│           ▼                                                 │
│  3. Show save dialog:                                      │
│     ┌─────────────────────────────────────────────────┐    │
│     │ Save Snapshot                              [×] │    │
│     ├─────────────────────────────────────────────────┤    │
│     │ Name: [boss_fight_attempt_3              ]    │    │
│     │ Notes: [Before final phase, 1HP left     ]    │    │
│     │ ┌─────────────┐                                │    │
│     │ │  thumbnail  │   Frame: 45,230              │    │
│     │ │             │   Time: 12:34 PM             │    │
│     │ └─────────────┘                                │    │
│     │           [Cancel]  [Save]                     │    │
│     └─────────────────────────────────────────────────┘    │
│           │                                                 │
│           ▼                                                 │
│  4. Serialize state                                        │
│           │                                                 │
│           ▼                                                 │
│  5. Save to ~/.emberware/games/{game}/snapshots/           │
│           │                                                 │
│           ▼                                                 │
│  6. Resume game                                            │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Hotkeys

| Key | Action |
|-----|--------|
| F5 | Quick save to slot 1 |
| Shift+F5 | Quick save dialog (choose slot 1-9) |
| F9 | Quick load from slot 1 |
| Shift+F9 | Quick load dialog (choose slot 1-9) |
| 1-9 (while Shift+F5/F9 open) | Select slot |
| Ctrl+S | Save named snapshot (with dialog) |
| Ctrl+Shift+S | Save named snapshot (quick, auto-name) |
| Ctrl+L | Load snapshot browser |

## UI Design

### Snapshot Browser

```
┌─────────────────────────────────────────────────────────────────────────────┐
│ Snapshots                                                              [×] │
├─────────────────────────────────────────────────────────────────────────────┤
│ Quick Slots                                                                 │
│ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐   │
│ │ [1]  │ │ [2]  │ │ [3]  │ │ [4]  │ │ [5]  │ │ [6]  │ │ [7]  │ │ [8]  │   │
│ │thumb │ │ ---- │ │thumb │ │ ---- │ │ ---- │ │ ---- │ │ ---- │ │ ---- │   │
│ │F:1234│ │empty │ │F:5678│ │empty │ │empty │ │empty │ │empty │ │empty │   │
│ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘ └──────┘   │
├─────────────────────────────────────────────────────────────────────────────┤
│ Named Snapshots                                               [New] [Import]│
│ ┌───────────────────────────────────────────────────────────────────────┐  │
│ │ ┌────────┐                                                            │  │
│ │ │        │  boss_fight_attempt_3                                      │  │
│ │ │ thumb  │  Frame: 45,230  │  2024-01-15 12:34 PM                     │  │
│ │ │        │  Notes: Before final phase, 1HP left                       │  │
│ │ └────────┘                               [Load] [Delete] [Export]     │  │
│ │ ─────────────────────────────────────────────────────────────────────│  │
│ │ ┌────────┐                                                            │  │
│ │ │        │  level_start                                               │  │
│ │ │ thumb  │  Frame: 10,500  │  2024-01-15 10:15 AM                     │  │
│ │ │        │  Notes: Clean start of level 3                             │  │
│ │ └────────┘                               [Load] [Delete] [Export]     │  │
│ └───────────────────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────────────────┤
│ Auto-save: 2 minutes ago (Frame: 44,100)                      [Load Auto]  │
└─────────────────────────────────────────────────────────────────────────────┘
```

### Quick Slot Overlay

When pressing Shift+F5 or Shift+F9:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              SAVE TO SLOT                                  │
│                                                                            │
│   [1]        [2]        [3]        [4]        [5]                         │
│  ┌────┐    ┌────┐    ┌────┐    ┌────┐    ┌────┐                           │
│  │████│    │    │    │████│    │    │    │    │                           │
│  │████│    │    │    │████│    │    │    │    │                           │
│  └────┘    └────┘    └────┘    └────┘    └────┘                           │
│   used     empty      used     empty     empty                             │
│                                                                            │
│   [6]        [7]        [8]        [9]                                    │
│  ┌────┐    ┌────┐    ┌────┐    ┌────┐                                     │
│  │    │    │    │    │    │    │    │                                     │
│  │    │    │    │    │    │    │    │                                     │
│  └────┘    └────┘    └────┘    └────┘                                     │
│  empty     empty     empty     empty                                       │
│                                                                            │
│                    Press 1-9 to select, ESC to cancel                      │
└─────────────────────────────────────────────────────────────────────────────┘
```

## FFI API

State snapshots primarily use the existing `save_state`/`load_state` exports that games already implement for GGRS rollback. No new FFI is strictly required.

### Optional Enhancements

```rust
// Get estimated state size (for UI feedback)
extern "C" fn __state_size_hint() -> u32;

// Get human-readable state version
extern "C" fn __state_version_string(out_ptr: u32, out_len: u32) -> u32;

// Callback after state load (for game to re-sync derived state)
extern "C" fn __on_state_loaded();
```

## Storage

### File Structure

```
~/.emberware/games/{game_id}/snapshots/
├── quickslots.bin          # Quick slots (session persistence optional)
├── boss_fight_attempt_3.snapshot
├── level_start.snapshot
├── auto_save.snapshot
└── thumbnails/
    ├── boss_fight_attempt_3.png
    ├── level_start.png
    └── auto_save.png
```

### Snapshot File Format

```
┌────────────────────────────────────────┐
│ Header                                 │
│ ├─ magic: "EMBS" (4 bytes)            │
│ ├─ version: u32                       │
│ ├─ name_len: u16                      │
│ ├─ name: [u8; name_len]               │
│ ├─ notes_len: u16                     │
│ ├─ notes: [u8; notes_len]             │
│ ├─ timestamp: u64 (unix epoch)        │
│ ├─ frame: u64                         │
│ ├─ game_id_len: u16                   │
│ ├─ game_id: [u8; game_id_len]         │
│ ├─ rom_hash: [u8; 32]                 │
│ ├─ state_len: u32                     │
│ └─ thumbnail_len: u32                 │
├────────────────────────────────────────┤
│ State Data                             │
│ └─ [u8; state_len] (compressed)       │
├────────────────────────────────────────┤
│ Thumbnail                              │
│ └─ [u8; thumbnail_len] (PNG)          │
└────────────────────────────────────────┘
```

## Integration with Other Debug Tools

### Memory Viewer Integration

After loading a snapshot, the Memory Viewer can:
- Show "Snapshot loaded" indicator
- Compare current state to snapshot state
- Highlight memory differences

### Replay System Integration

- Save snapshot during replay playback
- Load snapshot, then record new inputs from that point
- "Branch" replay: Load snapshot, play differently

### Debug Inspection Integration

- All registered debug variables update after load
- Graphs show discontinuity marker at load point

## Auto-Save

### Configuration

```toml
[snapshots]
auto_save_enabled = true
auto_save_interval_secs = 120  # Every 2 minutes
auto_save_keep_count = 3       # Keep last 3 auto-saves
auto_save_on_crash = true      # Save on unhandled error
```

### Behavior

1. Every `auto_save_interval_secs`, silently save state
2. Rotate auto-saves: auto_save_1 → auto_save_2 → auto_save_3 → delete
3. On crash/error, save crash_recovery snapshot
4. Show "Auto-saved" toast briefly

## Edge Cases

### During Netplay
Snapshots are disabled during P2P netplay (would desync players).

### State Incompatibility
If loading a snapshot from a different ROM version:
- Warn user
- Attempt load anyway
- If `load_state` fails, show error

### Snapshot While Paused
Allowed - captures paused state exactly.

### Rapid Save/Load
Debounce saves (100ms minimum between saves).

## Pending Questions

### Q1: Cross-session quick slots?
Should quick slots persist across emulator restarts?
- A) No - session only (current)
- B) Yes - persist to disk
- C) Configurable

**Recommendation**: Option C.

### Q2: Undo load?
Should loading a snapshot save current state to "undo" slot?
- A) No
- B) Yes - automatic undo slot
- C) Prompt "Save current state first?"

**Recommendation**: Option B - non-intrusive safety net.

### Q3: Snapshot sharing?
Should snapshots be shareable (like replays)?
- A) No - local only
- B) Yes - exportable files
- C) Yes with ROM hash verification

**Recommendation**: Option C.

### Q4: Thumbnail timing?
When to capture thumbnail?
- A) Last rendered frame before save
- B) Capture new frame at save time
- C) No thumbnails (simpler)

**Recommendation**: Option A - no extra rendering.

### Q5: State compression?
Should snapshot state be compressed?
- A) No - fast save/load
- B) Yes - smaller files (LZ4)
- C) Configurable

**Recommendation**: Option B - LZ4 is fast enough.

## Pros

1. **Instant**: Load state directly, no replay needed
2. **Simple**: Uses existing save_state/load_state
3. **Versatile**: Quick slots for iteration, named for persistence
4. **Visual**: Thumbnails make browsing easy
5. **Auto-save**: Safety net for crashes

## Cons

1. **File size**: Snapshots can be large (full state)
2. **No inputs**: Can't "continue" from snapshot with recorded inputs
3. **ROM coupling**: Only works with matching ROM version
4. **Memory cost**: Quick slots held in RAM
5. **No netplay**: Disabled in multiplayer

## Implementation Complexity

**Estimated effort:** Low-Medium

**Key components:**
1. Snapshot data structures - 0.5 days
2. Quick slot system - 1 day
3. Named snapshot management - 1 day
4. Snapshot browser UI - 2 days
5. File format & persistence - 1 day
6. Thumbnail capture - 0.5 days
7. Auto-save system - 1 day
8. Testing - 1 day

**Total:** ~8 days

## Console-Agnostic Design

State snapshots are fully console-agnostic:
- Uses game's `save_state`/`load_state` exports
- SnapshotManager lives in core's GameSession
- UI is egui (shared across consoles)
- Thumbnail capture uses existing render target

## Future Enhancements

1. **Snapshot timeline**: Visual timeline of all snapshots
2. **Diff view**: Compare two snapshots
3. **Cloud sync**: Sync snapshots across machines
4. **Snapshot annotations**: Mark specific moments
5. **Checkpoint system**: Auto-save at game events
